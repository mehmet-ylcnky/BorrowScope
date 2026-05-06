//! Temporal queries and lifetime analysis.



use serde::{Deserialize, Serialize};

use crate::edge::{EdgeId, EdgeKind};
use crate::graph::OwnershipGraph;
use crate::node::NodeId;

// ═══════════════════════════════════════════════════════════════════════════
// 4.1 Variable lifetime spans
// ═══════════════════════════════════════════════════════════════════════════

/// A variable's lifetime span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeSpan {
    pub node: NodeId,
    pub name: String,
    pub start: u64,
    pub end: Option<u64>,
}

impl LifetimeSpan {
    /// Duration in timestamp units. None if still alive.
    pub fn duration(&self) -> Option<u64> {
        self.end.map(|e| e - self.start)
    }

    /// Whether this span is alive at the given timestamp.
    pub fn is_alive_at(&self, timestamp: u64) -> bool {
        timestamp >= self.start && self.end.map_or(true, |e| timestamp < e)
    }

    /// Whether this span overlaps with another.
    pub fn overlaps(&self, other: &LifetimeSpan) -> bool {
        let self_end = self.end.unwrap_or(u64::MAX);
        let other_end = other.end.unwrap_or(u64::MAX);
        self.start < other_end && other.start < self_end
    }
}

/// Get lifetime spans for all variable nodes in the graph.
pub fn all_lifetimes(graph: &OwnershipGraph) -> Vec<LifetimeSpan> {
    graph.nodes().iter().map(|n| LifetimeSpan {
        node: n.id(),
        name: n.name().to_string(),
        start: n.start_time(),
        end: n.end_time(),
    }).collect()
}

/// Get lifetime span for a specific node.
pub fn lifetime_of(graph: &OwnershipGraph, node: NodeId) -> Option<LifetimeSpan> {
    graph.get_node(node).map(|n| LifetimeSpan {
        node: n.id(),
        name: n.name().to_string(),
        start: n.start_time(),
        end: n.end_time(),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.2 Overlapping lifetimes
// ═══════════════════════════════════════════════════════════════════════════

/// Find all pairs of nodes with overlapping lifetimes.
pub fn overlapping_lifetimes(graph: &OwnershipGraph) -> Vec<(NodeId, NodeId)> {
    let spans = all_lifetimes(graph);
    let mut pairs = Vec::new();
    for i in 0..spans.len() {
        for j in (i + 1)..spans.len() {
            if spans[i].overlaps(&spans[j]) {
                pairs.push((spans[i].node, spans[j].node));
            }
        }
    }
    pairs
}

/// Check if two specific nodes have overlapping lifetimes.
pub fn lifetimes_overlap(graph: &OwnershipGraph, a: NodeId, b: NodeId) -> bool {
    let span_a = match lifetime_of(graph, a) {
        Some(s) => s,
        None => return false,
    };
    let span_b = match lifetime_of(graph, b) {
        Some(s) => s,
        None => return false,
    };
    span_a.overlaps(&span_b)
}

/// Find all nodes whose lifetime overlaps with a given node.
pub fn contemporaries(graph: &OwnershipGraph, node: NodeId) -> Vec<NodeId> {
    let span = match lifetime_of(graph, node) {
        Some(s) => s,
        None => return vec![],
    };
    all_lifetimes(graph)
        .iter()
        .filter(|s| s.node != node && span.overlaps(s))
        .map(|s| s.node)
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.3 Active variables at timestamp
// ═══════════════════════════════════════════════════════════════════════════

/// Snapshot of ownership state at a timestamp.
#[derive(Debug, Clone)]
pub struct OwnershipSnapshot {
    pub timestamp: u64,
    pub alive_variables: Vec<NodeId>,
    pub active_borrows: Vec<EdgeId>,
    pub active_locks: Vec<EdgeId>,
}

/// Get ownership snapshot at a specific timestamp.
pub fn snapshot_at(graph: &OwnershipGraph, timestamp: u64) -> OwnershipSnapshot {
    let alive_variables: Vec<NodeId> = graph.nodes()
        .iter()
        .filter(|n| n.is_alive_at(timestamp))
        .map(|n| n.id())
        .collect();

    let active_borrows: Vec<EdgeId> = graph.edges()
        .iter()
        .filter(|e| e.is_borrow() && e.is_active_at(timestamp))
        .map(|e| e.id)
        .collect();

    let active_locks: Vec<EdgeId> = graph.edges()
        .iter()
        .filter(|e| matches!(e.kind, EdgeKind::LockAcquire { .. }) && e.is_active_at(timestamp))
        .map(|e| e.id)
        .collect();

    OwnershipSnapshot { timestamp, alive_variables, active_borrows, active_locks }
}

/// Get all nodes alive at a timestamp.
pub fn alive_at(graph: &OwnershipGraph, timestamp: u64) -> Vec<NodeId> {
    graph.nodes()
        .iter()
        .filter(|n| n.is_alive_at(timestamp))
        .map(|n| n.id())
        .collect()
}

/// Get nodes alive throughout an entire interval [start, end).
pub fn alive_during(graph: &OwnershipGraph, start: u64, end: u64) -> Vec<NodeId> {
    graph.nodes()
        .iter()
        .filter(|n| {
            let node_start = n.start_time();
            let node_end = n.end_time().unwrap_or(u64::MAX);
            node_start <= start && node_end >= end
        })
        .map(|n| n.id())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.4 Borrow scope computation
// ═══════════════════════════════════════════════════════════════════════════

/// A computed borrow scope.
#[derive(Debug, Clone)]
pub struct BorrowScope {
    pub edge: EdgeId,
    pub borrower: NodeId,
    pub owner: NodeId,
    pub mutable: bool,
    pub start: u64,
    /// Effective end (last use or drop, whichever is earlier).
    pub effective_end: u64,
    /// Actual drop time of the borrower.
    pub drop_time: Option<u64>,
}

/// Compute borrow scopes for all borrow edges.
pub fn borrow_scopes(graph: &OwnershipGraph) -> Vec<BorrowScope> {
    graph.edges()
        .iter()
        .filter(|e| e.is_borrow())
        .filter_map(|e| borrow_scope_of(graph, e.id))
        .collect()
}

/// Get the borrow scope for a specific edge.
pub fn borrow_scope_of(graph: &OwnershipGraph, edge_id: EdgeId) -> Option<BorrowScope> {
    let edge = graph.get_edge(edge_id)?;
    if !edge.is_borrow() {
        return None;
    }

    let borrower_node = graph.get_node(edge.source)?;
    let drop_time = borrower_node.end_time();
    let effective_end = edge.ended_at
        .or(drop_time)
        .unwrap_or(u64::MAX);

    Some(BorrowScope {
        edge: edge_id,
        borrower: edge.source,
        owner: edge.target,
        mutable: edge.is_mutable(),
        start: edge.created_at,
        effective_end,
        drop_time,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.5 Ownership transfer timeline
// ═══════════════════════════════════════════════════════════════════════════

/// A single ownership transfer.
#[derive(Debug, Clone)]
pub struct OwnershipTransfer {
    pub from: NodeId,
    pub to: NodeId,
    pub timestamp: u64,
    pub edge: EdgeId,
}

/// The complete ownership timeline for a value.
#[derive(Debug, Clone)]
pub struct OwnershipTimeline {
    /// The original creator of the value.
    pub origin: NodeId,
    /// Ordered list of transfers.
    pub transfers: Vec<OwnershipTransfer>,
    /// Current owner (last in chain, or origin if never moved).
    pub current_owner: NodeId,
}

/// Build the ownership timeline for a value, starting from its origin.
pub fn ownership_timeline(graph: &OwnershipGraph, origin: NodeId) -> OwnershipTimeline {
    let mut transfers = Vec::new();
    let mut current = origin;

    loop {
        // Find outgoing Move edge from current
        let move_edge = graph.outgoing_edges(current)
            .iter()
            .filter_map(|eid| graph.get_edge(*eid))
            .find(|e| e.kind == EdgeKind::Move);

        match move_edge {
            Some(e) => {
                transfers.push(OwnershipTransfer {
                    from: current,
                    to: e.target,
                    timestamp: e.created_at,
                    edge: e.id,
                });
                current = e.target;
            }
            None => break,
        }
    }

    OwnershipTimeline { origin, transfers, current_owner: current }
}

/// Find the original creator of a value (trace moves backward).
pub fn find_origin(graph: &OwnershipGraph, node: NodeId) -> NodeId {
    let mut current = node;
    loop {
        let incoming_move = graph.incoming_edges(current)
            .iter()
            .filter_map(|eid| graph.get_edge(*eid))
            .find(|e| e.kind == EdgeKind::Move);

        match incoming_move {
            Some(e) => current = e.source,
            None => return current,
        }
    }
}

/// Find the current owner of a value (trace moves forward).
pub fn find_current_owner(graph: &OwnershipGraph, node: NodeId) -> NodeId {
    ownership_timeline(graph, node).current_owner
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.6 Reference count history (Rc/Arc)
// ═══════════════════════════════════════════════════════════════════════════

/// A single entry in the reference count history.
#[derive(Debug, Clone)]
pub struct RefCountEntry {
    pub timestamp: u64,
    pub count: u32,
    pub event: RefCountEvent,
}

/// What caused the reference count change.
#[derive(Debug, Clone, PartialEq)]
pub enum RefCountEvent {
    Created,
    Cloned { clone_id: NodeId },
    Dropped { dropped_id: NodeId },
}

/// Complete reference count history for an Rc/Arc value.
#[derive(Debug, Clone)]
pub struct RefCountHistory {
    pub origin: NodeId,
    pub entries: Vec<RefCountEntry>,
    pub peak_count: u32,
    pub final_count: u32,
    pub is_leaked: bool,
}

/// Build reference count history for an Rc/Arc node.
pub fn ref_count_history(graph: &OwnershipGraph, rc_node: NodeId) -> RefCountHistory {
    let mut entries = Vec::new();
    let mut count: u32 = 1;
    let mut peak: u32 = 1;

    // Find creation time
    let created_at = graph.get_node(rc_node)
        .map(|n| n.start_time())
        .unwrap_or(0);

    entries.push(RefCountEntry {
        timestamp: created_at,
        count: 1,
        event: RefCountEvent::Created,
    });

    // Collect all clone edges where this node is the source (target of RcClone edge)
    let mut events: Vec<(u64, RefCountEvent, i32)> = Vec::new();

    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::RcClone { .. } | EdgeKind::ArcClone { .. } => {
                if edge.target == rc_node {
                    // Someone cloned from this node
                    events.push((edge.created_at, RefCountEvent::Cloned { clone_id: edge.source }, 1));
                }
            }
            _ => {}
        }
    }

    // Find drops of clones (nodes that have RcClone/ArcClone edges pointing to rc_node)
    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::RcClone { .. } | EdgeKind::ArcClone { .. } => {
                if edge.target == rc_node {
                    // The clone (source) may have been dropped
                    if let Some(clone_node) = graph.get_node(edge.source) {
                        if let Some(drop_time) = clone_node.end_time() {
                            events.push((drop_time, RefCountEvent::Dropped { dropped_id: edge.source }, -1));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Check if the origin itself was dropped
    if let Some(origin_node) = graph.get_node(rc_node) {
        if let Some(drop_time) = origin_node.end_time() {
            events.push((drop_time, RefCountEvent::Dropped { dropped_id: rc_node }, -1));
        }
    }

    // Sort by timestamp
    events.sort_by_key(|(ts, _, _)| *ts);

    for (ts, event, delta) in events {
        count = (count as i32 + delta).max(0) as u32;
        peak = peak.max(count);
        entries.push(RefCountEntry { timestamp: ts, count, event });
    }

    let final_count = count;
    let is_leaked = final_count > 0;

    RefCountHistory { origin: rc_node, entries, peak_count: peak, final_count, is_leaked }
}

/// Get the reference count at a specific timestamp.
pub fn ref_count_at(graph: &OwnershipGraph, rc_node: NodeId, timestamp: u64) -> u32 {
    let history = ref_count_history(graph, rc_node);
    let mut count = 0;
    for entry in &history.entries {
        if entry.timestamp <= timestamp {
            count = entry.count;
        } else {
            break;
        }
    }
    count
}

/// Find all Rc/Arc nodes that are potentially leaked (final count > 0).
pub fn find_leaked_refs(graph: &OwnershipGraph) -> Vec<NodeId> {
    // Find nodes that are targets of RcClone/ArcClone edges (they are Rc/Arc origins)
    let mut rc_origins: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::RcClone { .. } | EdgeKind::ArcClone { .. } => {
                rc_origins.insert(edge.target);
            }
            _ => {}
        }
    }

    // Also include nodes that are sources of RcClone but not targets (they are origins too)
    // Actually, any node involved in Rc/Arc relationships that hasn't been fully dropped
    rc_origins.iter()
        .filter(|&&node| ref_count_history(graph, node).is_leaked)
        .copied()
        .collect()
}
