use crate::{OwnershipGraph, Variable};
use std::collections::HashMap;

pub struct QueryBuilder<'a> {
    graph: &'a OwnershipGraph,
}

impl OwnershipGraph {
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder { graph: self }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Variable> {
        self.graph.node_weights().find(|v| v.name == name)
    }

    pub fn find_all_by_name(&self, name: &str) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.name == name)
            .collect()
    }

    pub fn find_by_type(&self, type_name: &str) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.type_name == type_name)
            .collect()
    }

    pub fn find_references(&self) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.type_name.starts_with('&'))
            .collect()
    }

    pub fn find_mutable_references(&self) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.type_name.starts_with("&mut"))
            .collect()
    }

    pub fn alive_at(&self, time: u64) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.created_at <= time && v.dropped_at.map_or(true, |d| d > time))
            .collect()
    }

    pub fn created_between(&self, start: u64, end: u64) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.created_at >= start && v.created_at <= end)
            .collect()
    }

    pub fn dropped_between(&self, start: u64, end: u64) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.dropped_at.is_some_and(|d| d >= start && d <= end))
            .collect()
    }

    pub fn find_by_scope_depth(&self, depth: usize) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.scope_depth == depth)
            .collect()
    }

    pub fn degree_centrality(&self) -> HashMap<usize, f64> {
        let node_count = self.node_count();
        if node_count <= 1 {
            return HashMap::new();
        }

        let mut centrality = HashMap::new();
        let max_degree = (node_count - 1) as f64;

        for var in self.graph.node_weights() {
            if let Some(&node) = self.id_to_node.get(&var.id) {
                let in_degree = self
                    .graph
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .count();
                let out_degree = self
                    .graph
                    .neighbors_directed(node, petgraph::Direction::Outgoing)
                    .count();
                let degree = (in_degree + out_degree) as f64;
                centrality.insert(var.id, degree / max_degree);
            }
        }

        centrality
    }

    pub fn longest_borrow_chain(&self) -> Vec<usize> {
        let mut longest = Vec::new();

        for var in self.graph.node_weights() {
            let chain = self.dfs_from(var.id);
            if chain.len() > longest.len() {
                longest = chain;
            }
        }

        longest
    }

    pub fn most_borrowed_variable(&self) -> Option<&Variable> {
        self.graph
            .node_weights()
            .max_by_key(|v| self.borrowers_of(v.id).len())
    }

    pub fn least_borrowed_variable(&self) -> Option<&Variable> {
        self.graph
            .node_weights()
            .min_by_key(|v| self.borrowers_of(v.id).len())
    }

    pub fn variables_with_no_borrowers(&self) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| self.borrowers_of(v.id).is_empty())
            .collect()
    }

    pub fn variables_with_no_borrows(&self) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| self.borrows(v.id).is_empty())
            .collect()
    }

    pub fn find_by_name_pattern(&self, pattern: &str) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.name.contains(pattern))
            .collect()
    }

    pub fn find_by_type_pattern(&self, pattern: &str) -> Vec<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.type_name.contains(pattern))
            .collect()
    }

    pub fn oldest_variable(&self) -> Option<&Variable> {
        self.graph.node_weights().min_by_key(|v| v.created_at)
    }

    pub fn newest_variable(&self) -> Option<&Variable> {
        self.graph.node_weights().max_by_key(|v| v.created_at)
    }

    pub fn longest_lived_variable(&self) -> Option<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.dropped_at.is_some())
            .max_by_key(|v| v.dropped_at.unwrap() - v.created_at)
    }

    pub fn shortest_lived_variable(&self) -> Option<&Variable> {
        self.graph
            .node_weights()
            .filter(|v| v.dropped_at.is_some())
            .min_by_key(|v| v.dropped_at.unwrap() - v.created_at)
    }

    pub fn average_lifetime(&self) -> Option<f64> {
        let lifetimes: Vec<u64> = self
            .graph
            .node_weights()
            .filter_map(|v| v.dropped_at.map(|d| d - v.created_at))
            .collect();

        if lifetimes.is_empty() {
            None
        } else {
            Some(lifetimes.iter().sum::<u64>() as f64 / lifetimes.len() as f64)
        }
    }

    pub fn variables_by_lifetime(&self) -> Vec<(&Variable, u64)> {
        let mut vars: Vec<_> = self
            .graph
            .node_weights()
            .filter_map(|v| v.dropped_at.map(|d| (v, d - v.created_at)))
            .collect();
        vars.sort_by_key(|(_, lifetime)| *lifetime);
        vars.reverse();
        vars
    }
}

impl<'a> QueryBuilder<'a> {
    pub fn by_name(self, name: &str) -> FilteredQuery<'a> {
        let vars: Vec<_> = self
            .graph
            .graph
            .node_weights()
            .filter(|v| v.name == name)
            .collect();
        FilteredQuery { variables: vars }
    }

    pub fn by_type(self, type_name: &str) -> FilteredQuery<'a> {
        let vars: Vec<_> = self
            .graph
            .graph
            .node_weights()
            .filter(|v| v.type_name == type_name)
            .collect();
        FilteredQuery { variables: vars }
    }

    pub fn alive_at(self, time: u64) -> FilteredQuery<'a> {
        let vars: Vec<_> = self
            .graph
            .graph
            .node_weights()
            .filter(|v| v.created_at <= time && v.dropped_at.map_or(true, |d| d > time))
            .collect();
        FilteredQuery { variables: vars }
    }

    pub fn in_scope(self, depth: usize) -> FilteredQuery<'a> {
        let vars: Vec<_> = self
            .graph
            .graph
            .node_weights()
            .filter(|v| v.scope_depth == depth)
            .collect();
        FilteredQuery { variables: vars }
    }

    pub fn all(self) -> FilteredQuery<'a> {
        let vars: Vec<_> = self.graph.graph.node_weights().collect();
        FilteredQuery { variables: vars }
    }
}

pub struct FilteredQuery<'a> {
    variables: Vec<&'a Variable>,
}

impl<'a> FilteredQuery<'a> {
    pub fn and_by_name(mut self, name: &str) -> Self {
        self.variables.retain(|v| v.name == name);
        self
    }

    pub fn and_by_type(mut self, type_name: &str) -> Self {
        self.variables.retain(|v| v.type_name == type_name);
        self
    }

    pub fn and_alive_at(mut self, time: u64) -> Self {
        self.variables
            .retain(|v| v.created_at <= time && v.dropped_at.map_or(true, |d| d > time));
        self
    }

    pub fn and_in_scope(mut self, depth: usize) -> Self {
        self.variables.retain(|v| v.scope_depth == depth);
        self
    }

    pub fn and_created_after(mut self, time: u64) -> Self {
        self.variables.retain(|v| v.created_at > time);
        self
    }

    pub fn and_created_before(mut self, time: u64) -> Self {
        self.variables.retain(|v| v.created_at < time);
        self
    }

    pub fn and_dropped(mut self) -> Self {
        self.variables.retain(|v| v.dropped_at.is_some());
        self
    }

    pub fn and_not_dropped(mut self) -> Self {
        self.variables.retain(|v| v.dropped_at.is_none());
        self
    }

    pub fn count(self) -> usize {
        self.variables.len()
    }

    pub fn collect(self) -> Vec<&'a Variable> {
        self.variables
    }

    pub fn first(self) -> Option<&'a Variable> {
        self.variables.into_iter().next()
    }

    pub fn ids(self) -> Vec<usize> {
        self.variables.iter().map(|v| v.id).collect()
    }

    pub fn names(self) -> Vec<&'a str> {
        self.variables.iter().map(|v| v.name.as_str()).collect()
    }

    pub fn types(self) -> Vec<&'a str> {
        self.variables
            .iter()
            .map(|v| v.type_name.as_str())
            .collect()
    }
}
