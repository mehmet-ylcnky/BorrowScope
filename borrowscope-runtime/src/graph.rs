//! Ownership graph data structures

use crate::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A variable in the ownership graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
}

/// A relationship between variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Relationship {
    Owns {
        from: String,
        to: String,
    },
    BorrowsImmut {
        from: String,
        to: String,
        start: u64,
        end: u64,
    },
    BorrowsMut {
        from: String,
        to: String,
        start: u64,
        end: u64,
    },
}

/// The complete ownership graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraph {
    pub nodes: Vec<Variable>,
    pub edges: Vec<Relationship>,
}

impl OwnershipGraph {
    /// Create an empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a variable node
    pub fn add_variable(&mut self, var: Variable) {
        self.nodes.push(var);
    }

    /// Add a relationship edge
    pub fn add_relationship(&mut self, rel: Relationship) {
        self.edges.push(rel);
    }

    /// Find a variable by ID
    pub fn find_variable(&self, id: &str) -> Option<&Variable> {
        self.nodes.iter().find(|v| v.id == id)
    }

    /// Find all borrows of a variable
    pub fn find_borrows(&self, var_id: &str) -> Vec<&Relationship> {
        self.edges
            .iter()
            .filter(|rel| match rel {
                Relationship::BorrowsImmut { to, .. } | Relationship::BorrowsMut { to, .. } => {
                    to == var_id
                }
                _ => false,
            })
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> GraphStats {
        let mut immut_borrows = 0;
        let mut mut_borrows = 0;

        for edge in &self.edges {
            match edge {
                Relationship::BorrowsImmut { .. } => immut_borrows += 1,
                Relationship::BorrowsMut { .. } => mut_borrows += 1,
                _ => {}
            }
        }

        GraphStats {
            total_variables: self.nodes.len(),
            total_relationships: self.edges.len(),
            immutable_borrows: immut_borrows,
            mutable_borrows: mut_borrows,
        }
    }
}

impl Default for OwnershipGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_variables: usize,
    pub total_relationships: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
}

/// Build a graph from events
pub fn build_graph(events: &[Event]) -> OwnershipGraph {
    let mut graph = OwnershipGraph::new();
    let mut var_map: HashMap<String, Variable> = HashMap::new();
    let mut borrow_map: HashMap<String, (String, bool, u64)> = HashMap::new();

    for event in events {
        match event {
            Event::New {
                var_name,
                var_id,
                type_name,
                timestamp,
            } => {
                let var = Variable {
                    id: var_id.clone(),
                    name: var_name.clone(),
                    type_name: type_name.clone(),
                    created_at: *timestamp,
                    dropped_at: None,
                };
                var_map.insert(var_id.clone(), var);
            }

            Event::Borrow {
                borrower_id,
                owner_id,
                mutable,
                timestamp,
                ..
            } => {
                borrow_map.insert(
                    borrower_id.clone(),
                    (owner_id.clone(), *mutable, *timestamp),
                );
            }

            Event::Drop { var_id, timestamp } => {
                // Mark variable as dropped
                if let Some(var) = var_map.get_mut(var_id) {
                    var.dropped_at = Some(*timestamp);
                }

                // End borrow if this is a borrower
                if let Some((owner_id, is_mutable, start)) = borrow_map.remove(var_id) {
                    let rel = if is_mutable {
                        Relationship::BorrowsMut {
                            from: var_id.clone(),
                            to: owner_id,
                            start,
                            end: *timestamp,
                        }
                    } else {
                        Relationship::BorrowsImmut {
                            from: var_id.clone(),
                            to: owner_id,
                            start,
                            end: *timestamp,
                        }
                    };
                    graph.add_relationship(rel);
                }
            }

            _ => {}
        }
    }

    // Add all variables to graph
    for var in var_map.into_values() {
        graph.add_variable(var);
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = OwnershipGraph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_add_variable() {
        let mut graph = OwnershipGraph::new();

        let var = Variable {
            id: "x_0".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 1,
            dropped_at: None,
        };

        graph.add_variable(var);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_add_relationship() {
        let mut graph = OwnershipGraph::new();

        let rel = Relationship::BorrowsImmut {
            from: "r_1".to_string(),
            to: "x_0".to_string(),
            start: 2,
            end: 3,
        };

        graph.add_relationship(rel);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_find_variable() {
        let mut graph = OwnershipGraph::new();

        let var = Variable {
            id: "x_0".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 1,
            dropped_at: None,
        };

        graph.add_variable(var);

        let found = graph.find_variable("x_0");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");

        let not_found = graph.find_variable("y_0");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_borrows() {
        let mut graph = OwnershipGraph::new();

        graph.add_relationship(Relationship::BorrowsImmut {
            from: "r_1".to_string(),
            to: "x_0".to_string(),
            start: 2,
            end: 3,
        });

        graph.add_relationship(Relationship::BorrowsMut {
            from: "s_2".to_string(),
            to: "x_0".to_string(),
            start: 4,
            end: 5,
        });

        let borrows = graph.find_borrows("x_0");
        assert_eq!(borrows.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut graph = OwnershipGraph::new();

        graph.add_variable(Variable {
            id: "x_0".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 1,
            dropped_at: None,
        });

        graph.add_relationship(Relationship::BorrowsImmut {
            from: "r_1".to_string(),
            to: "x_0".to_string(),
            start: 2,
            end: 3,
        });

        graph.add_relationship(Relationship::BorrowsMut {
            from: "s_2".to_string(),
            to: "x_0".to_string(),
            start: 4,
            end: 5,
        });

        let stats = graph.stats();
        assert_eq!(stats.total_variables, 1);
        assert_eq!(stats.total_relationships, 2);
        assert_eq!(stats.immutable_borrows, 1);
        assert_eq!(stats.mutable_borrows, 1);
    }

    #[test]
    fn test_build_graph_simple() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Drop {
                timestamp: 2,
                var_id: "x_0".to_string(),
            },
        ];

        let graph = build_graph(&events);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].dropped_at, Some(2));
    }

    #[test]
    fn test_build_graph_with_borrow() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Borrow {
                timestamp: 2,
                borrower_name: "r".to_string(),
                borrower_id: "r_1".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 3,
                var_id: "r_1".to_string(),
            },
            Event::Drop {
                timestamp: 4,
                var_id: "x_0".to_string(),
            },
        ];

        let graph = build_graph(&events);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);

        match &graph.edges[0] {
            Relationship::BorrowsImmut {
                from,
                to,
                start,
                end,
            } => {
                assert_eq!(from, "r_1");
                assert_eq!(to, "x_0");
                assert_eq!(*start, 2);
                assert_eq!(*end, 3);
            }
            _ => panic!("Expected BorrowsImmut"),
        }
    }

    #[test]
    fn test_build_graph_mutable_borrow() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "Vec<i32>".to_string(),
            },
            Event::Borrow {
                timestamp: 2,
                borrower_name: "r".to_string(),
                borrower_id: "r_1".to_string(),
                owner_id: "x_0".to_string(),
                mutable: true,
            },
            Event::Drop {
                timestamp: 3,
                var_id: "r_1".to_string(),
            },
        ];

        let graph = build_graph(&events);
        assert_eq!(graph.edges.len(), 1);

        match &graph.edges[0] {
            Relationship::BorrowsMut { .. } => {}
            _ => panic!("Expected BorrowsMut"),
        }
    }

    #[test]
    fn test_serialization() {
        let graph = OwnershipGraph {
            nodes: vec![Variable {
                id: "x_0".to_string(),
                name: "x".to_string(),
                type_name: "i32".to_string(),
                created_at: 1,
                dropped_at: Some(2),
            }],
            edges: vec![Relationship::BorrowsImmut {
                from: "r_1".to_string(),
                to: "x_0".to_string(),
                start: 2,
                end: 3,
            }],
        };

        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: OwnershipGraph = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.nodes.len(), 1);
        assert_eq!(deserialized.edges.len(), 1);
    }
}
