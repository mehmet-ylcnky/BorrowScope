use crate::{GraphExport, OwnershipGraph, Relationship, Variable};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub version: String,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedGraphExport {
    pub metadata: GraphMetadata,
    pub nodes: Vec<Variable>,
    pub edges: Vec<crate::EdgeExport>,
}

impl OwnershipGraph {
    pub fn export_with_metadata(&self) -> EnhancedGraphExport {
        let export = self.export();
        EnhancedGraphExport {
            metadata: GraphMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                node_count: self.node_count(),
                edge_count: self.edge_count(),
            },
            nodes: export.nodes,
            edges: export.edges,
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.export_with_metadata())
    }

    pub fn to_messagepack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self.export_with_metadata())
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        // Try enhanced format first
        if let Ok(export) = serde_json::from_str::<EnhancedGraphExport>(json) {
            return Ok(Self::from_export(export));
        }

        // Fall back to basic format
        let export: GraphExport = serde_json::from_str(json)?;
        Ok(Self::from_basic_export(export))
    }

    fn from_basic_export(export: GraphExport) -> Self {
        let mut graph = Self::new();

        for node in export.nodes {
            graph.add_variable(node);
        }

        for edge in export.edges {
            match edge.relationship {
                Relationship::BorrowsImmut { at } => {
                    graph.add_borrow(edge.from_id, edge.to_id, false, at);
                }
                Relationship::BorrowsMut { at } => {
                    graph.add_borrow(edge.from_id, edge.to_id, true, at);
                }
                Relationship::Moves { at } => {
                    graph.add_move(edge.from_id, edge.to_id, at);
                }
                Relationship::RcClone { at, strong_count } => {
                    graph.add_rc_clone(edge.from_id, edge.to_id, strong_count, at);
                }
                Relationship::ArcClone { at, strong_count } => {
                    graph.add_arc_clone(edge.from_id, edge.to_id, strong_count, at);
                }
                Relationship::RefCellBorrow { at, is_mut } => {
                    graph.add_refcell_borrow(edge.from_id, edge.to_id, is_mut, at);
                }
            }
        }

        graph
    }

    pub fn from_messagepack(data: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        let export: EnhancedGraphExport = rmp_serde::from_slice(data)?;
        Ok(Self::from_export(export))
    }

    fn from_export(export: EnhancedGraphExport) -> Self {
        let mut graph = Self::new();

        for node in export.nodes {
            graph.add_variable(node);
        }

        for edge in export.edges {
            match edge.relationship {
                Relationship::BorrowsImmut { at } => {
                    graph.add_borrow(edge.from_id, edge.to_id, false, at);
                }
                Relationship::BorrowsMut { at } => {
                    graph.add_borrow(edge.from_id, edge.to_id, true, at);
                }
                Relationship::Moves { at } => {
                    graph.add_move(edge.from_id, edge.to_id, at);
                }
                Relationship::RcClone { at, strong_count } => {
                    graph.add_rc_clone(edge.from_id, edge.to_id, strong_count, at);
                }
                Relationship::ArcClone { at, strong_count } => {
                    graph.add_arc_clone(edge.from_id, edge.to_id, strong_count, at);
                }
                Relationship::RefCellBorrow { at, is_mut } => {
                    graph.add_refcell_borrow(edge.from_id, edge.to_id, is_mut, at);
                }
            }
        }

        graph
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph OwnershipGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for var in self.graph.node_weights() {
            let label = format!("{}\\n{}\\n@{}", var.name, var.type_name, var.created_at);
            let color = if var.dropped_at.is_some() {
                "lightgray"
            } else {
                "lightblue"
            };
            dot.push_str(&format!(
                "  n{} [label=\"{}\", fillcolor={}, style=filled];\n",
                var.id, label, color
            ));
        }

        dot.push('\n');

        for edge in self.graph.edge_references() {
            if let (Some(from), Some(to)) = (
                self.graph.node_weight(edge.source()),
                self.graph.node_weight(edge.target()),
            ) {
                let (label, color, style) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (format!("&@{}", at), "blue", "solid"),
                    Relationship::BorrowsMut { at } => (format!("&mut@{}", at), "red", "solid"),
                    Relationship::Moves { at } => (format!("move@{}", at), "black", "bold"),
                    Relationship::RcClone { at, strong_count } => {
                        (format!("Rc({})@{}", strong_count, at), "green", "dashed")
                    }
                    Relationship::ArcClone { at, strong_count } => {
                        (format!("Arc({})@{}", strong_count, at), "purple", "dashed")
                    }
                    Relationship::RefCellBorrow { at, is_mut } => {
                        let prefix = if *is_mut { "RefMut" } else { "Ref" };
                        (format!("{}@{}", prefix, at), "orange", "dotted")
                    }
                };

                dot.push_str(&format!(
                    "  n{} -> n{} [label=\"{}\", color={}, style={}];\n",
                    from.id, to.id, label, color, style
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    pub fn export_delta(&self, previous: &GraphExport) -> GraphDelta {
        let prev_nodes: HashMap<usize, &Variable> =
            previous.nodes.iter().map(|n| (n.id, n)).collect();
        let prev_edges: HashMap<(usize, usize), &Relationship> = previous
            .edges
            .iter()
            .map(|e| ((e.from_id, e.to_id), &e.relationship))
            .collect();

        let current = self.export();

        let added_nodes: Vec<Variable> = current
            .nodes
            .iter()
            .filter(|n| !prev_nodes.contains_key(&n.id))
            .cloned()
            .collect();

        let removed_nodes: Vec<usize> = previous
            .nodes
            .iter()
            .filter(|n| !current.nodes.iter().any(|cn| cn.id == n.id))
            .map(|n| n.id)
            .collect();

        let modified_nodes: Vec<Variable> = current
            .nodes
            .iter()
            .filter(|n| {
                prev_nodes
                    .get(&n.id)
                    .is_some_and(|prev| prev.dropped_at != n.dropped_at)
            })
            .cloned()
            .collect();

        let curr_edges: HashMap<(usize, usize), &Relationship> = current
            .edges
            .iter()
            .map(|e| ((e.from_id, e.to_id), &e.relationship))
            .collect();

        let added_edges: Vec<crate::EdgeExport> = curr_edges
            .iter()
            .filter(|(key, _)| !prev_edges.contains_key(key))
            .map(|((from_id, to_id), rel)| crate::EdgeExport {
                from_id: *from_id,
                to_id: *to_id,
                relationship: (*rel).clone(),
            })
            .collect();

        let removed_edges: Vec<(usize, usize)> = prev_edges
            .keys()
            .filter(|key| !curr_edges.contains_key(key))
            .copied()
            .collect();

        GraphDelta {
            added_nodes,
            removed_nodes,
            modified_nodes,
            added_edges,
            removed_edges,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDelta {
    pub added_nodes: Vec<Variable>,
    pub removed_nodes: Vec<usize>,
    pub modified_nodes: Vec<Variable>,
    pub added_edges: Vec<crate::EdgeExport>,
    pub removed_edges: Vec<(usize, usize)>,
}

impl GraphDelta {
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.modified_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
    }
}
