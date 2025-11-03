use crate::{OwnershipGraph, Relationship};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use petgraph::Direction;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    MultipleMutableBorrows,
    MutableWithImmutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowConflict {
    pub conflict_type: ConflictType,
    pub owner_id: usize,
    pub borrowers: Vec<usize>,
    pub time_range: (u64, u64),
}

#[derive(Debug, Clone)]
struct BorrowInterval {
    borrower_id: usize,
    is_mut: bool,
    start: u64,
    end: u64,
}

impl BorrowConflict {
    pub fn format(&self, graph: &OwnershipGraph) -> String {
        let owner = graph
            .get_variable(self.owner_id)
            .map(|v| v.name.as_str())
            .unwrap_or("<unknown>");

        let borrower_names: Vec<_> = self
            .borrowers
            .iter()
            .filter_map(|&id| graph.get_variable(id))
            .map(|v| v.name.as_str())
            .collect();

        match self.conflict_type {
            ConflictType::MultipleMutableBorrows => {
                format!(
                    "Multiple mutable borrows of '{}' by: {}",
                    owner,
                    borrower_names.join(", ")
                )
            }
            ConflictType::MutableWithImmutable => {
                format!(
                    "Mutable and immutable borrows of '{}' by: {}",
                    owner,
                    borrower_names.join(", ")
                )
            }
        }
    }
}

impl OwnershipGraph {
    pub fn active_borrows_at_time(&self, owner_id: usize, time: u64) -> Vec<(usize, bool)> {
        let owner_node = match self.id_to_node.get(&owner_id) {
            Some(&n) => n,
            None => return vec![],
        };

        self.graph
            .edges_directed(owner_node, Direction::Incoming)
            .filter_map(|edge| {
                let borrower_node = edge.source();
                let borrower = self.graph.node_weight(borrower_node)?;

                let (borrow_time, is_mut) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (*at, false),
                    Relationship::BorrowsMut { at } => (*at, true),
                    Relationship::RefCellBorrow { at, is_mut } => (*at, *is_mut),
                    _ => return None,
                };

                if borrow_time <= time && borrower.dropped_at.map_or(true, |d| d > time) {
                    Some((borrower.id, is_mut))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn check_conflicts_at(&self, owner_id: usize, time: u64) -> Option<BorrowConflict> {
        let active = self.active_borrows_at_time(owner_id, time);

        if active.is_empty() {
            return None;
        }

        let mut mutable_borrows = vec![];
        let mut immutable_borrows = vec![];

        for (borrower_id, is_mut) in active {
            if is_mut {
                mutable_borrows.push(borrower_id);
            } else {
                immutable_borrows.push(borrower_id);
            }
        }

        if mutable_borrows.len() > 1 {
            return Some(BorrowConflict {
                conflict_type: ConflictType::MultipleMutableBorrows,
                owner_id,
                borrowers: mutable_borrows,
                time_range: (time, time),
            });
        }

        if !mutable_borrows.is_empty() && !immutable_borrows.is_empty() {
            let mut all_borrowers = mutable_borrows;
            all_borrowers.extend(immutable_borrows);

            return Some(BorrowConflict {
                conflict_type: ConflictType::MutableWithImmutable,
                owner_id,
                borrowers: all_borrowers,
                time_range: (time, time),
            });
        }

        None
    }

    fn get_borrow_intervals(&self, owner_id: usize) -> Vec<BorrowInterval> {
        let owner_node = match self.id_to_node.get(&owner_id) {
            Some(&n) => n,
            None => return vec![],
        };

        self.graph
            .edges_directed(owner_node, Direction::Incoming)
            .filter_map(|edge| {
                let borrower_node = edge.source();
                let borrower = self.graph.node_weight(borrower_node)?;

                let (start, is_mut) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (*at, false),
                    Relationship::BorrowsMut { at } => (*at, true),
                    Relationship::RefCellBorrow { at, is_mut } => (*at, *is_mut),
                    _ => return None,
                };

                let end = borrower.dropped_at.unwrap_or(u64::MAX);

                Some(BorrowInterval {
                    borrower_id: borrower.id,
                    is_mut,
                    start,
                    end,
                })
            })
            .collect()
    }

    pub fn find_conflicts(&self) -> Vec<BorrowConflict> {
        let mut conflicts = vec![];
        let mut all_times = BTreeSet::new();

        for var in self.graph.node_weights() {
            all_times.insert(var.created_at);
            if let Some(dropped) = var.dropped_at {
                all_times.insert(dropped);
            }
        }

        for edge in self.graph.edge_references() {
            let time = match edge.weight() {
                Relationship::BorrowsImmut { at } => *at,
                Relationship::BorrowsMut { at } => *at,
                Relationship::RefCellBorrow { at, .. } => *at,
                _ => continue,
            };
            all_times.insert(time);
        }

        for var in self.graph.node_weights() {
            for &time in &all_times {
                if let Some(conflict) = self.check_conflicts_at(var.id, time) {
                    if !conflicts.iter().any(|c: &BorrowConflict| {
                        c.owner_id == conflict.owner_id && c.borrowers == conflict.borrowers
                    }) {
                        conflicts.push(conflict);
                    }
                }
            }
        }

        conflicts
    }

    pub fn find_conflicts_optimized(&self) -> Vec<BorrowConflict> {
        let mut conflicts = vec![];

        for var in self.graph.node_weights() {
            let intervals = self.get_borrow_intervals(var.id);

            for i in 0..intervals.len() {
                for j in (i + 1)..intervals.len() {
                    let a = &intervals[i];
                    let b = &intervals[j];

                    let overlap_start = a.start.max(b.start);
                    let overlap_end = a.end.min(b.end);

                    if overlap_start < overlap_end && (a.is_mut || b.is_mut) {
                        let conflict_type = if a.is_mut && b.is_mut {
                            ConflictType::MultipleMutableBorrows
                        } else {
                            ConflictType::MutableWithImmutable
                        };

                        let mut borrowers = vec![a.borrower_id, b.borrower_id];
                        borrowers.sort_unstable();

                        if !conflicts.iter().any(|c: &BorrowConflict| {
                            c.owner_id == var.id
                                && c.conflict_type == conflict_type
                                && c.borrowers == borrowers
                        }) {
                            conflicts.push(BorrowConflict {
                                conflict_type,
                                owner_id: var.id,
                                borrowers,
                                time_range: (overlap_start, overlap_end),
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    pub fn report_conflicts(&self) -> String {
        let conflicts = self.find_conflicts_optimized();

        if conflicts.is_empty() {
            return "No borrow conflicts detected.".to_string();
        }

        let mut report = format!("Found {} conflict(s):\n\n", conflicts.len());

        for (i, conflict) in conflicts.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, conflict.format(self)));
            report.push_str(&format!(
                "   Time range: {} - {}\n\n",
                conflict.time_range.0, conflict.time_range.1
            ));
        }

        report
    }

    pub fn conflict_timeline(&self, owner_id: usize) -> Vec<(u64, Vec<(usize, bool)>)> {
        let mut timeline = vec![];
        let mut times = BTreeSet::new();

        let owner_node = match self.id_to_node.get(&owner_id) {
            Some(&n) => n,
            None => return timeline,
        };

        for edge in self.graph.edges_directed(owner_node, Direction::Incoming) {
            let borrower = match self.graph.node_weight(edge.source()) {
                Some(b) => b,
                None => continue,
            };

            let borrow_time = match edge.weight() {
                Relationship::BorrowsImmut { at } => *at,
                Relationship::BorrowsMut { at } => *at,
                Relationship::RefCellBorrow { at, .. } => *at,
                _ => continue,
            };

            times.insert(borrow_time);
            if let Some(dropped) = borrower.dropped_at {
                times.insert(dropped);
            }
        }

        for &time in &times {
            let active = self.active_borrows_at_time(owner_id, time);
            if !active.is_empty() {
                timeline.push((time, active));
            }
        }

        timeline
    }
}
