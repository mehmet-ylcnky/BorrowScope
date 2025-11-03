use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use petgraph::{Directed, Direction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variable {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Relationship {
    BorrowsImmut { at: u64 },
    BorrowsMut { at: u64 },
    Moves { at: u64 },
    RcClone { at: u64, strong_count: usize },
    ArcClone { at: u64, strong_count: usize },
    RefCellBorrow { at: u64, is_mut: bool },
}

// ============================================================================
// Ownership Graph
// ============================================================================

pub struct OwnershipGraph {
    graph: StableGraph<Variable, Relationship, Directed>,
    id_to_node: HashMap<usize, NodeIndex>,
}

impl OwnershipGraph {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            id_to_node: HashMap::new(),
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: StableGraph::with_capacity(nodes, edges),
            id_to_node: HashMap::with_capacity(nodes),
        }
    }

    // ========================================================================
    // Construction Methods
    // ========================================================================

    pub fn add_variable(&mut self, var: Variable) -> NodeIndex {
        let node = self.graph.add_node(var.clone());
        self.id_to_node.insert(var.id, node);
        node
    }

    pub fn add_borrow(
        &mut self,
        borrower_id: usize,
        owner_id: usize,
        is_mut: bool,
        at: u64,
    ) -> Option<EdgeIndex> {
        let borrower = *self.id_to_node.get(&borrower_id)?;
        let owner = *self.id_to_node.get(&owner_id)?;
        let rel = if is_mut {
            Relationship::BorrowsMut { at }
        } else {
            Relationship::BorrowsImmut { at }
        };
        Some(self.graph.add_edge(borrower, owner, rel))
    }

    pub fn add_move(&mut self, from_id: usize, to_id: usize, at: u64) -> Option<EdgeIndex> {
        let from = *self.id_to_node.get(&from_id)?;
        let to = *self.id_to_node.get(&to_id)?;
        Some(self.graph.add_edge(to, from, Relationship::Moves { at }))
    }

    pub fn add_rc_clone(
        &mut self,
        clone_id: usize,
        original_id: usize,
        strong_count: usize,
        at: u64,
    ) -> Option<EdgeIndex> {
        let clone = *self.id_to_node.get(&clone_id)?;
        let original = *self.id_to_node.get(&original_id)?;
        Some(
            self.graph
                .add_edge(clone, original, Relationship::RcClone { at, strong_count }),
        )
    }

    pub fn add_arc_clone(
        &mut self,
        clone_id: usize,
        original_id: usize,
        strong_count: usize,
        at: u64,
    ) -> Option<EdgeIndex> {
        let clone = *self.id_to_node.get(&clone_id)?;
        let original = *self.id_to_node.get(&original_id)?;
        Some(
            self.graph
                .add_edge(clone, original, Relationship::ArcClone { at, strong_count }),
        )
    }

    pub fn add_refcell_borrow(
        &mut self,
        borrower_id: usize,
        owner_id: usize,
        is_mut: bool,
        at: u64,
    ) -> Option<EdgeIndex> {
        let borrower = *self.id_to_node.get(&borrower_id)?;
        let owner = *self.id_to_node.get(&owner_id)?;
        Some(
            self.graph
                .add_edge(borrower, owner, Relationship::RefCellBorrow { at, is_mut }),
        )
    }

    pub fn mark_dropped(&mut self, id: usize, at: u64) -> bool {
        if let Some(&node) = self.id_to_node.get(&id) {
            if let Some(var) = self.graph.node_weight_mut(node) {
                var.dropped_at = Some(at);
                return true;
            }
        }
        false
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    pub fn get_variable(&self, id: usize) -> Option<&Variable> {
        self.id_to_node
            .get(&id)
            .and_then(|&node| self.graph.node_weight(node))
    }

    pub fn borrowers_of(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node
            .get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph
                    .neighbors_directed(node, Direction::Incoming)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }

    pub fn borrows(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node
            .get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph
                    .neighbors(node)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }

    pub fn is_alive(&self, id: usize, at: u64) -> bool {
        self.get_variable(id)
            .map(|var| var.created_at <= at && var.dropped_at.map_or(true, |d| d > at))
            .unwrap_or(false)
    }

    pub fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.graph.node_weights()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn clear(&mut self) {
        self.graph.clear();
        self.id_to_node.clear();
    }

    // ========================================================================
    // Advanced Query Methods
    // ========================================================================

    pub fn active_borrows_at(&self, id: usize, at: u64) -> Vec<(&Variable, &Relationship)> {
        self.id_to_node
            .get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph
                    .edges_directed(node, Direction::Incoming)
                    .filter_map(|edge| {
                        let rel = edge.weight();
                        let borrower = self.graph.node_weight(edge.source())?;

                        let borrow_time = match rel {
                            Relationship::BorrowsImmut { at: t } => *t,
                            Relationship::BorrowsMut { at: t } => *t,
                            Relationship::RefCellBorrow { at: t, .. } => *t,
                            _ => return None,
                        };

                        if borrow_time <= at && borrower.dropped_at.map_or(true, |d| d > at) {
                            Some((borrower, rel))
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}

// ============================================================================
// Serialization Support
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExport {
    pub nodes: Vec<Variable>,
    pub edges: Vec<EdgeExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeExport {
    pub from_id: usize,
    pub to_id: usize,
    pub relationship: Relationship,
}

impl OwnershipGraph {
    pub fn export(&self) -> GraphExport {
        let nodes = self.graph.node_weights().cloned().collect();
        let edges = self
            .graph
            .edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                Some(EdgeExport {
                    from_id: from.id,
                    to_id: to.id,
                    relationship: edge.weight().clone(),
                })
            })
            .collect();

        GraphExport { nodes, edges }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.export())
    }

    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.export())
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

    pub fn add_variables(&mut self, vars: Vec<Variable>) -> Vec<NodeIndex> {
        vars.into_iter().map(|var| self.add_variable(var)).collect()
    }

    pub fn mark_dropped_batch(&mut self, ids: &[usize], at: u64) -> usize {
        ids.iter().filter(|&&id| self.mark_dropped(id, at)).count()
    }

    // ========================================================================
    // Validation Methods
    // ========================================================================

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for cycles (should never happen in valid Rust)
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            errors.push("Graph contains cycles (invalid ownership)".to_string());
        }

        // Validate timestamps
        for var in self.graph.node_weights() {
            if let Some(dropped) = var.dropped_at {
                if dropped < var.created_at {
                    errors.push(format!(
                        "Variable '{}' (id={}) dropped before creation",
                        var.name, var.id
                    ));
                }
            }
        }

        // Validate borrow lifetimes
        for edge in self.graph.edge_references() {
            let borrower = self.graph.node_weight(edge.source());
            let owner = self.graph.node_weight(edge.target());

            if let (Some(borrower), Some(owner)) = (borrower, owner) {
                let borrow_time = match edge.weight() {
                    Relationship::BorrowsImmut { at } => Some(*at),
                    Relationship::BorrowsMut { at } => Some(*at),
                    Relationship::RefCellBorrow { at, .. } => Some(*at),
                    _ => None,
                };

                if let Some(at) = borrow_time {
                    // Borrow should happen after owner creation
                    if at < owner.created_at {
                        errors.push(format!(
                            "Borrow of '{}' happens before owner '{}' creation",
                            borrower.name, owner.name
                        ));
                    }

                    // Borrow should happen before owner drop
                    if let Some(owner_dropped) = owner.dropped_at {
                        if at >= owner_dropped {
                            errors.push(format!(
                                "Borrow of '{}' happens after owner '{}' dropped",
                                borrower.name, owner.name
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn has_cycles(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.graph)
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    pub fn statistics(&self) -> GraphStatistics {
        let mut immutable_borrows = 0;
        let mut mutable_borrows = 0;
        let mut moves = 0;
        let mut rc_clones = 0;
        let mut arc_clones = 0;
        let mut refcell_borrows = 0;

        for edge in self.graph.edge_references() {
            match edge.weight() {
                Relationship::BorrowsImmut { .. } => immutable_borrows += 1,
                Relationship::BorrowsMut { .. } => mutable_borrows += 1,
                Relationship::Moves { .. } => moves += 1,
                Relationship::RcClone { .. } => rc_clones += 1,
                Relationship::ArcClone { .. } => arc_clones += 1,
                Relationship::RefCellBorrow { .. } => refcell_borrows += 1,
            }
        }

        let alive_count = self
            .graph
            .node_weights()
            .filter(|v| v.dropped_at.is_none())
            .count();

        GraphStatistics {
            total_variables: self.node_count(),
            alive_variables: alive_count,
            total_edges: self.edge_count(),
            immutable_borrows,
            mutable_borrows,
            moves,
            rc_clones,
            arc_clones,
            refcell_borrows,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStatistics {
    pub total_variables: usize,
    pub alive_variables: usize,
    pub total_edges: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub moves: usize,
    pub rc_clones: usize,
    pub arc_clones: usize,
    pub refcell_borrows: usize,
}

impl Default for OwnershipGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = OwnershipGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let graph = OwnershipGraph::with_capacity(100, 200);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_variable() {
        let mut graph = OwnershipGraph::new();
        let var = Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        };
        graph.add_variable(var.clone());
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.get_variable(1), Some(&var));
    }

    #[test]
    fn test_add_multiple_variables() {
        let mut graph = OwnershipGraph::new();
        for i in 0..10 {
            graph.add_variable(Variable {
                id: i,
                name: format!("var_{}", i),
                type_name: "i32".into(),
                created_at: i as u64 * 100,
                dropped_at: None,
                scope_depth: 0,
            });
        }
        assert_eq!(graph.node_count(), 10);
    }

    #[test]
    fn test_add_immutable_borrow() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "r".into(),
            type_name: "&i32".into(),
            created_at: 1050,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_borrow(2, 1, false, 1050);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_add_mutable_borrow() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "r".into(),
            type_name: "&mut i32".into(),
            created_at: 1050,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_borrow(2, 1, true, 1050);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_borrowers_of() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "r1".into(),
            type_name: "&i32".into(),
            created_at: 1050,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 3,
            name: "r2".into(),
            type_name: "&i32".into(),
            created_at: 1100,
            dropped_at: None,
            scope_depth: 0,
        });

        graph.add_borrow(2, 1, false, 1050);
        graph.add_borrow(3, 1, false, 1100);

        let borrowers = graph.borrowers_of(1);
        assert_eq!(borrowers.len(), 2);
        let names: Vec<_> = borrowers.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"r1"));
        assert!(names.contains(&"r2"));
    }

    #[test]
    fn test_mark_dropped() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });

        assert!(graph.mark_dropped(1, 2000));
        assert_eq!(graph.get_variable(1).unwrap().dropped_at, Some(2000));
    }

    #[test]
    fn test_is_alive() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(2000),
            scope_depth: 0,
        });

        assert!(!graph.is_alive(1, 500));
        assert!(graph.is_alive(1, 1500));
        assert!(!graph.is_alive(1, 2500));
    }

    #[test]
    fn test_add_move() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "String".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "y".into(),
            type_name: "String".into(),
            created_at: 1100,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_move(1, 2, 1100);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_add_rc_clone() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "Rc<i32>".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "y".into(),
            type_name: "Rc<i32>".into(),
            created_at: 1100,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_rc_clone(2, 1, 2, 1100);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_add_arc_clone() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "Arc<i32>".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "y".into(),
            type_name: "Arc<i32>".into(),
            created_at: 1100,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_arc_clone(2, 1, 2, 1100);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_add_refcell_borrow() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "RefCell<i32>".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2,
            name: "r".into(),
            type_name: "Ref<i32>".into(),
            created_at: 1100,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_refcell_borrow(2, 1, false, 1100);
        assert!(edge.is_some());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });

        graph.clear();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_missing_node_returns_none() {
        let mut graph = OwnershipGraph::new();
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });

        let edge = graph.add_borrow(2, 1, false, 1050);
        assert!(edge.is_none());
    }

    #[test]
    fn test_all_variables_iterator() {
        let mut graph = OwnershipGraph::new();
        for i in 0..5 {
            graph.add_variable(Variable {
                id: i,
                name: format!("var_{}", i),
                type_name: "i32".into(),
                created_at: i as u64 * 100,
                dropped_at: None,
                scope_depth: 0,
            });
        }

        let count = graph.all_variables().count();
        assert_eq!(count, 5);
    }
}
