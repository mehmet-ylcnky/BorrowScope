//! Conflict detection and graph validation algorithms.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId, EdgeKind};
use crate::graph::OwnershipGraph;
use crate::node::NodeId;

// ═══════════════════════════════════════════════════════════════════════════
// 3.1 Active borrows at timestamp
// ═══════════════════════════════════════════════════════════════════════════

/// All active borrows at a given timestamp, grouped by the borrowed variable (target).
pub fn active_borrows_at(graph: &OwnershipGraph) -> HashMap<NodeId, Vec<&Edge>> {
    active_borrows_at_time(graph, None)
}

/// Active borrows at a specific timestamp, grouped by the borrowed variable.
pub fn active_borrows_at_time(
    graph: &OwnershipGraph,
    timestamp: Option<u64>,
) -> HashMap<NodeId, Vec<&Edge>> {
    let mut result: HashMap<NodeId, Vec<&Edge>> = HashMap::new();
    for edge in graph.edges() {
        if !edge.is_borrow() {
            continue;
        }
        let active = match timestamp {
            Some(ts) => edge.is_active_at(ts),
            None => edge.ended_at.is_none(),
        };
        if active {
            result.entry(edge.target).or_default().push(edge);
        }
    }
    result
}

/// Active borrows on a specific variable at a timestamp.
pub fn borrows_on_at<'a>(
    graph: &'a OwnershipGraph,
    owner: NodeId,
    timestamp: u64,
) -> Vec<&'a Edge> {
    graph
        .edges()
        .iter()
        .filter(|e| e.is_borrow() && e.target == owner && e.is_active_at(timestamp))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.2 Mutable/Immutable borrow conflict detection
// ═══════════════════════════════════════════════════════════════════════════

/// Type of borrow conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    /// &mut + & on same variable
    MutableAndShared,
    /// &mut + &mut on same variable
    MultipleMutable,
}

/// A detected borrow conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowConflict {
    /// The variable being borrowed.
    pub owner: NodeId,
    /// First conflicting borrow edge.
    pub borrow_a: EdgeId,
    /// Second conflicting borrow edge.
    pub borrow_b: EdgeId,
    /// Start of the overlap window.
    pub conflict_start: u64,
    /// End of the overlap window.
    pub conflict_end: u64,
    /// Type of conflict.
    pub kind: ConflictKind,
}

/// Find all borrow conflicts in the graph.
pub fn find_conflicts(graph: &OwnershipGraph) -> Vec<BorrowConflict> {
    let mut conflicts = Vec::new();

    // Group borrow edges by target (owner)
    let mut borrows_by_owner: HashMap<NodeId, Vec<&Edge>> = HashMap::new();
    for edge in graph.edges() {
        if edge.is_borrow() {
            borrows_by_owner.entry(edge.target).or_default().push(edge);
        }
    }

    for (owner, borrows) in &borrows_by_owner {
        // Check each pair for overlap
        for i in 0..borrows.len() {
            for j in (i + 1)..borrows.len() {
                if let Some(conflict) = check_overlap(borrows[i], borrows[j], *owner) {
                    conflicts.push(conflict);
                }
            }
        }
    }
    conflicts
}

fn check_overlap(a: &Edge, b: &Edge, owner: NodeId) -> Option<BorrowConflict> {
    let a_is_mut = a.is_mutable();
    let b_is_mut = b.is_mutable();

    // No conflict if both are shared borrows
    if !a_is_mut && !b_is_mut {
        return None;
    }

    // Check temporal overlap
    let a_end = a.ended_at.unwrap_or(u64::MAX);
    let b_end = b.ended_at.unwrap_or(u64::MAX);

    let overlap_start = a.created_at.max(b.created_at);
    let overlap_end = a_end.min(b_end);

    if overlap_start >= overlap_end {
        return None; // No temporal overlap
    }

    let kind = if a_is_mut && b_is_mut {
        ConflictKind::MultipleMutable
    } else {
        ConflictKind::MutableAndShared
    };

    Some(BorrowConflict {
        owner,
        borrow_a: a.id,
        borrow_b: b.id,
        conflict_start: overlap_start,
        conflict_end: overlap_end,
        kind,
    })
}

/// Check conflicts at a specific timestamp.
pub fn conflicts_at(graph: &OwnershipGraph, timestamp: u64) -> Vec<BorrowConflict> {
    find_conflicts(graph)
        .into_iter()
        .filter(|c| timestamp >= c.conflict_start && timestamp < c.conflict_end)
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.3 Conflict timeline generation
// ═══════════════════════════════════════════════════════════════════════════

/// A window of time during which a conflict exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictWindow {
    pub owner: NodeId,
    pub start: u64,
    pub end: u64,
    pub kind: ConflictKind,
    pub active_borrows: Vec<EdgeId>,
}

/// Generate a timeline of all conflict windows.
pub fn conflict_timeline(graph: &OwnershipGraph) -> Vec<ConflictWindow> {
    let conflicts = find_conflicts(graph);
    conflicts
        .into_iter()
        .map(|c| ConflictWindow {
            owner: c.owner,
            start: c.conflict_start,
            end: c.conflict_end,
            kind: c.kind,
            active_borrows: vec![c.borrow_a, c.borrow_b],
        })
        .collect()
}

/// Check if any conflicts exist at a specific timestamp.
pub fn has_conflicts_at(graph: &OwnershipGraph, timestamp: u64) -> bool {
    !conflicts_at(graph, timestamp).is_empty()
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.4 Cycle detection (Rc/Arc reference cycles)
// ═══════════════════════════════════════════════════════════════════════════

/// A detected reference cycle.
#[derive(Debug, Clone)]
pub struct ReferenceCycle {
    /// Nodes forming the cycle, in order.
    pub nodes: Vec<NodeId>,
    /// Edges forming the cycle.
    pub edges: Vec<EdgeId>,
    /// Whether this involves Arc (vs Rc).
    pub is_arc: bool,
}

/// Detect reference cycles in Rc/Arc clone relationships.
pub fn detect_reference_cycles(graph: &OwnershipGraph) -> Vec<ReferenceCycle> {
    let mut cycles = Vec::new();

    // Build adjacency for only RcClone/ArcClone edges
    let mut rc_adj: HashMap<NodeId, Vec<(NodeId, EdgeId, bool)>> = HashMap::new();
    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::RcClone { .. } => {
                rc_adj
                    .entry(edge.source)
                    .or_default()
                    .push((edge.target, edge.id, false));
            }
            EdgeKind::ArcClone { .. } => {
                rc_adj
                    .entry(edge.source)
                    .or_default()
                    .push((edge.target, edge.id, true));
            }
            _ => {}
        }
    }

    // DFS with coloring for back-edge detection
    let mut white: HashSet<NodeId> = rc_adj.keys().copied().collect();
    let mut gray: HashSet<NodeId> = HashSet::new();
    let mut black: HashSet<NodeId> = HashSet::new();
    let mut path: Vec<(NodeId, Option<EdgeId>)> = Vec::new();

    let start_nodes: Vec<NodeId> = white.iter().copied().collect();
    for start in start_nodes {
        if !white.contains(&start) {
            continue;
        }
        dfs_cycle_detect(
            &rc_adj,
            start,
            &mut white,
            &mut gray,
            &mut black,
            &mut path,
            &mut cycles,
        );
    }

    cycles
}

fn dfs_cycle_detect(
    adj: &HashMap<NodeId, Vec<(NodeId, EdgeId, bool)>>,
    node: NodeId,
    white: &mut HashSet<NodeId>,
    gray: &mut HashSet<NodeId>,
    black: &mut HashSet<NodeId>,
    path: &mut Vec<(NodeId, Option<EdgeId>)>,
    cycles: &mut Vec<ReferenceCycle>,
) {
    white.remove(&node);
    gray.insert(node);
    path.push((node, None));

    if let Some(neighbors) = adj.get(&node) {
        for &(neighbor, edge_id, is_arc) in neighbors {
            if gray.contains(&neighbor) {
                // Back edge found - extract cycle
                let cycle_start = path.iter().position(|(n, _)| *n == neighbor).unwrap();
                let cycle_nodes: Vec<NodeId> =
                    path[cycle_start..].iter().map(|(n, _)| *n).collect();
                let mut cycle_edges: Vec<EdgeId> = path[cycle_start + 1..]
                    .iter()
                    .filter_map(|(_, e)| *e)
                    .collect();
                cycle_edges.push(edge_id);

                cycles.push(ReferenceCycle {
                    nodes: cycle_nodes,
                    edges: cycle_edges,
                    is_arc,
                });
            } else if white.contains(&neighbor) {
                path.last_mut().unwrap().1 = Some(edge_id);
                dfs_cycle_detect(adj, neighbor, white, gray, black, path, cycles);
            }
        }
    }

    path.pop();
    gray.remove(&node);
    black.insert(node);
}

/// Check if a specific node participates in a reference cycle.
pub fn is_in_cycle(graph: &OwnershipGraph, node: NodeId) -> bool {
    detect_reference_cycles(graph)
        .iter()
        .any(|c| c.nodes.contains(&node))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.5 Graph validation (invariant checking)
// ═══════════════════════════════════════════════════════════════════════════

/// Kind of validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    BorrowOutlivesOwner,
    MoveWhileBorrowed,
    DanglingEdgeReference,
    InvalidTimestamps,
    DuplicateNodeId,
}

/// A validation error with context.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub message: String,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
}

/// Validate graph invariants. Returns empty vec if valid.
pub fn validate(graph: &OwnershipGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check: all edge references point to valid nodes
    let node_ids: HashSet<NodeId> = graph.nodes().iter().map(|n| n.id()).collect();
    for edge in graph.edges() {
        if !node_ids.contains(&edge.source) {
            errors.push(ValidationError {
                kind: ValidationErrorKind::DanglingEdgeReference,
                message: format!(
                    "Edge {:?} references non-existent source node {:?}",
                    edge.id, edge.source
                ),
                nodes: vec![edge.source],
                edges: vec![edge.id],
            });
        }
        if !node_ids.contains(&edge.target) {
            errors.push(ValidationError {
                kind: ValidationErrorKind::DanglingEdgeReference,
                message: format!(
                    "Edge {:?} references non-existent target node {:?}",
                    edge.id, edge.target
                ),
                nodes: vec![edge.target],
                edges: vec![edge.id],
            });
        }
    }

    // Check: no borrow outlives its owner
    for edge in graph.edges() {
        if !edge.is_borrow() {
            continue;
        }
        if let Some(owner_node) = graph.get_node(edge.target) {
            if let Some(owner_end) = owner_node.end_time() {
                let borrow_end = edge.ended_at.unwrap_or(u64::MAX);
                if borrow_end > owner_end {
                    errors.push(ValidationError {
                        kind: ValidationErrorKind::BorrowOutlivesOwner,
                        message: format!(
                            "Borrow edge {:?} ends at {} but owner {:?} dropped at {}",
                            edge.id, borrow_end, edge.target, owner_end
                        ),
                        nodes: vec![edge.source, edge.target],
                        edges: vec![edge.id],
                    });
                }
            }
        }
    }

    // Check: no move while active borrows exist on the source
    for edge in graph.edges() {
        if edge.kind != EdgeKind::Move {
            continue;
        }
        let move_time = edge.created_at;
        // Check if source has active borrows at move time
        let active = graph
            .incoming_edges(edge.source)
            .iter()
            .filter_map(|eid| graph.get_edge(*eid))
            .filter(|e| e.is_borrow() && e.is_active_at(move_time))
            .count();
        if active > 0 {
            errors.push(ValidationError {
                kind: ValidationErrorKind::MoveWhileBorrowed,
                message: format!(
                    "Move edge {:?} at t={} but source {:?} has {} active borrows",
                    edge.id, move_time, edge.source, active
                ),
                nodes: vec![edge.source, edge.target],
                edges: vec![edge.id],
            });
        }
    }

    // Check: no edge has created_at > ended_at
    for edge in graph.edges() {
        if let Some(ended) = edge.ended_at {
            if edge.created_at > ended {
                errors.push(ValidationError {
                    kind: ValidationErrorKind::InvalidTimestamps,
                    message: format!(
                        "Edge {:?} has created_at={} > ended_at={}",
                        edge.id, edge.created_at, ended
                    ),
                    nodes: vec![edge.source, edge.target],
                    edges: vec![edge.id],
                });
            }
        }
    }

    // Check: no duplicate node IDs
    let mut seen_ids: HashSet<NodeId> = HashSet::new();
    for node in graph.nodes() {
        if !seen_ids.insert(node.id()) {
            errors.push(ValidationError {
                kind: ValidationErrorKind::DuplicateNodeId,
                message: format!("Duplicate node ID {:?}", node.id()),
                nodes: vec![node.id()],
                edges: vec![],
            });
        }
    }

    errors
}

/// Quick check: is the graph valid?
pub fn is_valid(graph: &OwnershipGraph) -> bool {
    validate(graph).is_empty()
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.6 Use-after-move detection
// ═══════════════════════════════════════════════════════════════════════════

/// A detected use-after-move violation.
#[derive(Debug, Clone)]
pub struct UseAfterMove {
    pub variable: NodeId,
    pub moved_at: u64,
    pub move_edge: EdgeId,
    pub used_at: u64,
    pub use_edge: EdgeId,
}

/// Detect use-after-move patterns.
pub fn detect_use_after_move(graph: &OwnershipGraph) -> Vec<UseAfterMove> {
    let mut violations = Vec::new();

    // Find all move edges and their source nodes
    let moves: Vec<(&Edge, NodeId)> = graph
        .edges()
        .iter()
        .filter(|e| e.kind == EdgeKind::Move)
        .map(|e| (e, e.source))
        .collect();

    for (move_edge, moved_var) in &moves {
        let move_time = move_edge.created_at;

        // Check if any borrow edge on this variable starts after the move
        for eid in graph.incoming_edges(*moved_var) {
            if let Some(edge) = graph.get_edge(*eid) {
                if edge.is_borrow() && edge.created_at > move_time {
                    violations.push(UseAfterMove {
                        variable: *moved_var,
                        moved_at: move_time,
                        move_edge: move_edge.id,
                        used_at: edge.created_at,
                        use_edge: edge.id,
                    });
                }
            }
        }

        // Also check outgoing borrows from the moved variable
        for eid in graph.outgoing_edges(*moved_var) {
            if let Some(edge) = graph.get_edge(*eid) {
                if edge.is_borrow() && edge.created_at > move_time {
                    violations.push(UseAfterMove {
                        variable: *moved_var,
                        moved_at: move_time,
                        move_edge: move_edge.id,
                        used_at: edge.created_at,
                        use_edge: edge.id,
                    });
                }
            }
        }
    }

    violations
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.7 Double-free / dangling pointer detection
// ═══════════════════════════════════════════════════════════════════════════

/// A detected dangling pointer access.
#[derive(Debug, Clone)]
pub struct DanglingAccess {
    pub pointer: NodeId,
    pub source: NodeId,
    pub source_dropped_at: u64,
    pub access_at: u64,
}

/// Detect dangling pointer accesses (deref after source dropped).
/// Checks if any edge references a node that was already dropped at the edge's creation time.
pub fn detect_dangling_pointers(graph: &OwnershipGraph) -> Vec<DanglingAccess> {
    let mut violations = Vec::new();

    for edge in graph.edges() {
        if !edge.is_borrow() {
            continue;
        }
        // Check if the target (owner) was already dropped when this borrow was created
        if let Some(target_node) = graph.get_node(edge.target) {
            if let Some(dropped_at) = target_node.end_time() {
                if edge.created_at >= dropped_at {
                    violations.push(DanglingAccess {
                        pointer: edge.source,
                        source: edge.target,
                        source_dropped_at: dropped_at,
                        access_at: edge.created_at,
                    });
                }
            }
        }
    }

    violations
}

/// Detect variables that appear to be dropped multiple times.
/// Returns (NodeId, Vec of drop timestamps) for each double-free.
pub fn detect_double_free(graph: &OwnershipGraph) -> Vec<(NodeId, Vec<u64>)> {
    // In our graph model, a node can only have one dropped_at.
    // Double-free would manifest as multiple nodes with the same name
    // that are both dropped, or as a node that has events after its drop.
    // For now, we detect nodes referenced by edges after their drop time.
    let mut violations = Vec::new();

    // Group nodes by name to find potential double-frees
    let mut by_name: HashMap<&str, Vec<&crate::node::Node>> = HashMap::new();
    for node in graph.nodes() {
        by_name.entry(node.name()).or_default().push(node);
    }

    for (_, nodes) in &by_name {
        let drop_times: Vec<u64> = nodes.iter().filter_map(|n| n.end_time()).collect();
        if drop_times.len() > 1 {
            // Multiple drops of same-named variable (could be shadowing, not necessarily double-free)
            // Only flag if they share edges (same logical variable)
            let node_ids: HashSet<NodeId> = nodes.iter().map(|n| n.id()).collect();
            let connected = nodes.iter().any(|n| {
                graph
                    .outgoing_edges(n.id())
                    .iter()
                    .filter_map(|eid| graph.get_edge(*eid))
                    .any(|e| node_ids.contains(&e.target))
            });
            if connected && drop_times.len() > 1 {
                violations.push((nodes[0].id(), drop_times));
            }
        }
    }

    violations
}
