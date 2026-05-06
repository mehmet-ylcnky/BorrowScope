//! Statistics and metrics for ownership graphs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::edge::EdgeKind;
use crate::graph::OwnershipGraph;
use crate::node::{Node, NodeId};

// ═══════════════════════════════════════════════════════════════════════════
// 5.1 Graph statistics
// ═══════════════════════════════════════════════════════════════════════════

/// Comprehensive graph statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    // Node counts
    pub total_nodes: usize,
    pub variable_nodes: usize,
    pub scope_nodes: usize,
    pub alive_variables: usize,
    pub dropped_variables: usize,

    // Edge counts by kind
    pub total_edges: usize,
    pub shared_borrows: usize,
    pub mutable_borrows: usize,
    pub moves: usize,
    pub rc_clones: usize,
    pub arc_clones: usize,
    pub weak_downgrades: usize,
    pub refcell_borrows: usize,
    pub lock_acquires: usize,
    pub closure_captures: usize,
    pub channel_sends: usize,

    // Derived metrics
    pub avg_borrows_per_variable: f64,
    pub max_borrows_on_single_variable: usize,
    pub move_ratio: f64,
    pub shared_ownership_ratio: f64,
}

/// Compute full statistics for the graph.
pub fn statistics(graph: &OwnershipGraph) -> GraphStatistics {
    let mut variable_nodes = 0usize;
    let mut scope_nodes = 0usize;
    let mut alive_variables = 0usize;
    let mut dropped_variables = 0usize;

    for node in graph.nodes() {
        match node {
            Node::Variable(v) => {
                variable_nodes += 1;
                if v.dropped_at.is_some() {
                    dropped_variables += 1;
                } else {
                    alive_variables += 1;
                }
            }
            Node::Scope(_) => scope_nodes += 1,
        }
    }

    let mut shared_borrows = 0usize;
    let mut mutable_borrows = 0usize;
    let mut moves = 0usize;
    let mut rc_clones = 0usize;
    let mut arc_clones = 0usize;
    let mut weak_downgrades = 0usize;
    let mut refcell_borrows = 0usize;
    let mut lock_acquires = 0usize;
    let mut closure_captures = 0usize;
    let mut channel_sends = 0usize;

    // Count borrows per variable (target of borrow edges)
    let mut borrows_per_var: HashMap<NodeId, usize> = HashMap::new();

    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::BorrowShared => {
                shared_borrows += 1;
                *borrows_per_var.entry(edge.target).or_default() += 1;
            }
            EdgeKind::BorrowMut => {
                mutable_borrows += 1;
                *borrows_per_var.entry(edge.target).or_default() += 1;
            }
            EdgeKind::Move => moves += 1,
            EdgeKind::RcClone { .. } => rc_clones += 1,
            EdgeKind::ArcClone { .. } => arc_clones += 1,
            EdgeKind::WeakDowngrade => weak_downgrades += 1,
            EdgeKind::RefCellBorrow { .. } => refcell_borrows += 1,
            EdgeKind::LockAcquire { .. } => lock_acquires += 1,
            EdgeKind::ClosureCapture { .. } => closure_captures += 1,
            EdgeKind::ChannelSend => channel_sends += 1,
            EdgeKind::ScopeContains => {}
        }
    }

    let total_borrows = shared_borrows + mutable_borrows;
    let avg_borrows_per_variable = if variable_nodes > 0 {
        total_borrows as f64 / variable_nodes as f64
    } else {
        0.0
    };
    let max_borrows_on_single_variable = borrows_per_var.values().copied().max().unwrap_or(0);
    let move_ratio = if variable_nodes > 0 {
        moves as f64 / variable_nodes as f64
    } else {
        0.0
    };
    let shared_ownership_ratio = if variable_nodes > 0 {
        (rc_clones + arc_clones) as f64 / variable_nodes as f64
    } else {
        0.0
    };

    GraphStatistics {
        total_nodes: graph.node_count(),
        variable_nodes,
        scope_nodes,
        alive_variables,
        dropped_variables,
        total_edges: graph.edge_count(),
        shared_borrows,
        mutable_borrows,
        moves,
        rc_clones,
        arc_clones,
        weak_downgrades,
        refcell_borrows,
        lock_acquires,
        closure_captures,
        channel_sends,
        avg_borrows_per_variable,
        max_borrows_on_single_variable,
        move_ratio,
        shared_ownership_ratio,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.2 Ownership hotspot detection
// ═══════════════════════════════════════════════════════════════════════════

/// A variable identified as an ownership hotspot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub node: NodeId,
    pub name: String,
    pub type_name: String,
    pub total_edges: usize,
    pub incoming_borrows: usize,
    pub outgoing_borrows: usize,
    pub moves_in: usize,
    pub moves_out: usize,
    pub clones: usize,
    pub score: f64,
}

/// Find the top-N ownership hotspots (most connected variables).
pub fn hotspots(graph: &OwnershipGraph, top_n: usize) -> Vec<Hotspot> {
    let mut spots: Vec<Hotspot> = graph
        .nodes()
        .iter()
        .filter_map(|n| {
            if let Node::Variable(v) = n {
                Some(compute_hotspot(graph, v.id, &v.name, &v.type_name))
            } else {
                None
            }
        })
        .collect();

    // Sort by total_edges descending
    spots.sort_by(|a, b| b.total_edges.cmp(&a.total_edges));

    // Normalize scores
    let max_edges = spots.first().map(|h| h.total_edges).unwrap_or(1).max(1);
    for spot in &mut spots {
        spot.score = spot.total_edges as f64 / max_edges as f64;
    }

    spots.truncate(top_n);
    spots
}

/// Find variables with borrow counts above the threshold.
pub fn heavily_borrowed(graph: &OwnershipGraph, min_borrows: usize) -> Vec<Hotspot> {
    graph
        .nodes()
        .iter()
        .filter_map(|n| {
            if let Node::Variable(v) = n {
                let spot = compute_hotspot(graph, v.id, &v.name, &v.type_name);
                if spot.incoming_borrows >= min_borrows {
                    Some(spot)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Find variables involved in the most ownership transfers.
pub fn most_transferred(graph: &OwnershipGraph, top_n: usize) -> Vec<Hotspot> {
    let mut spots: Vec<Hotspot> = graph
        .nodes()
        .iter()
        .filter_map(|n| {
            if let Node::Variable(v) = n {
                let spot = compute_hotspot(graph, v.id, &v.name, &v.type_name);
                if spot.moves_in + spot.moves_out > 0 {
                    Some(spot)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    spots.sort_by(|a, b| (b.moves_in + b.moves_out).cmp(&(a.moves_in + a.moves_out)));
    spots.truncate(top_n);
    spots
}

fn compute_hotspot(graph: &OwnershipGraph, id: NodeId, name: &str, type_name: &str) -> Hotspot {
    let mut incoming_borrows = 0;
    let mut outgoing_borrows = 0;
    let mut moves_in = 0;
    let mut moves_out = 0;
    let mut clones = 0;

    for eid in graph.incoming_edges(id) {
        if let Some(e) = graph.get_edge(*eid) {
            match &e.kind {
                EdgeKind::BorrowShared | EdgeKind::BorrowMut => incoming_borrows += 1,
                EdgeKind::Move => moves_in += 1,
                EdgeKind::RcClone { .. } | EdgeKind::ArcClone { .. } => clones += 1,
                _ => {}
            }
        }
    }
    for eid in graph.outgoing_edges(id) {
        if let Some(e) = graph.get_edge(*eid) {
            match &e.kind {
                EdgeKind::BorrowShared | EdgeKind::BorrowMut => outgoing_borrows += 1,
                EdgeKind::Move => moves_out += 1,
                EdgeKind::RcClone { .. } | EdgeKind::ArcClone { .. } => clones += 1,
                _ => {}
            }
        }
    }

    let total_edges = incoming_borrows + outgoing_borrows + moves_in + moves_out + clones;

    Hotspot {
        node: id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        total_edges,
        incoming_borrows,
        outgoing_borrows,
        moves_in,
        moves_out,
        clones,
        score: 0.0, // normalized later
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.3 Borrow frequency analysis
// ═══════════════════════════════════════════════════════════════════════════

/// A burst of borrow activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowBurst {
    pub start: u64,
    pub end: u64,
    pub borrow_count: usize,
}

/// Borrow frequency analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowFrequencyAnalysis {
    pub total_borrows: usize,
    pub shared_borrows: usize,
    pub mutable_borrows: usize,
    pub avg_duration: f64,
    pub max_duration: u64,
    pub min_duration: u64,
    pub median_duration: u64,
    /// Borrows per 100 timestamp units.
    pub frequency: f64,
    /// Maximum concurrent borrows observed.
    pub max_concurrent: usize,
    /// Time windows with high borrow activity.
    pub bursts: Vec<BorrowBurst>,
}

/// Analyze borrow frequency and patterns.
pub fn borrow_frequency(graph: &OwnershipGraph) -> BorrowFrequencyAnalysis {
    let borrow_edges: Vec<_> = graph.edges().iter().filter(|e| e.is_borrow()).collect();

    if borrow_edges.is_empty() {
        return BorrowFrequencyAnalysis {
            total_borrows: 0,
            shared_borrows: 0,
            mutable_borrows: 0,
            avg_duration: 0.0,
            max_duration: 0,
            min_duration: 0,
            median_duration: 0,
            frequency: 0.0,
            max_concurrent: 0,
            bursts: vec![],
        };
    }

    let shared_borrows = borrow_edges.iter().filter(|e| !e.is_mutable()).count();
    let mutable_borrows = borrow_edges.iter().filter(|e| e.is_mutable()).count();

    // Compute durations
    let mut durations: Vec<u64> = borrow_edges.iter().filter_map(|e| e.duration()).collect();
    durations.sort();

    let avg_duration = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<u64>() as f64 / durations.len() as f64
    };
    let max_duration = durations.last().copied().unwrap_or(0);
    let min_duration = durations.first().copied().unwrap_or(0);
    let median_duration = if durations.is_empty() {
        0
    } else {
        durations[durations.len() / 2]
    };

    // Compute frequency (borrows per 100 timestamp units)
    let min_ts = borrow_edges.iter().map(|e| e.created_at).min().unwrap_or(0);
    let max_ts = borrow_edges
        .iter()
        .map(|e| e.ended_at.unwrap_or(e.created_at))
        .max()
        .unwrap_or(0);
    let time_span = (max_ts - min_ts).max(1);
    let frequency = borrow_edges.len() as f64 * 100.0 / time_span as f64;

    // Compute max concurrent borrows (sweep line)
    let mut events: Vec<(u64, i32)> = Vec::new();
    for e in &borrow_edges {
        events.push((e.created_at, 1));
        if let Some(end) = e.ended_at {
            events.push((end, -1));
        }
    }
    events.sort_by_key(|(ts, _)| *ts);

    let mut concurrent = 0i32;
    let mut max_concurrent = 0usize;
    for (_, delta) in &events {
        concurrent += delta;
        max_concurrent = max_concurrent.max(concurrent as usize);
    }

    // Detect bursts (windows with >= 3 borrows starting within 10 timestamp units)
    let mut bursts = Vec::new();
    let starts: Vec<u64> = borrow_edges.iter().map(|e| e.created_at).collect();
    let mut sorted_starts = starts.clone();
    sorted_starts.sort();
    sorted_starts.dedup();

    let window_size = time_span / 10; // adaptive window
    let burst_threshold = 3;
    let mut i = 0;
    while i < sorted_starts.len() {
        let window_start = sorted_starts[i];
        let window_end = window_start + window_size.max(1);
        let count = sorted_starts
            .iter()
            .filter(|&&ts| ts >= window_start && ts < window_end)
            .count();
        if count >= burst_threshold {
            bursts.push(BorrowBurst {
                start: window_start,
                end: window_end,
                borrow_count: count,
            });
            // Skip past this burst
            while i < sorted_starts.len() && sorted_starts[i] < window_end {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    BorrowFrequencyAnalysis {
        total_borrows: borrow_edges.len(),
        shared_borrows,
        mutable_borrows,
        avg_duration,
        max_duration,
        min_duration,
        median_duration,
        frequency,
        max_concurrent,
        bursts,
    }
}

/// Borrow frequency for a specific variable.
pub fn borrow_frequency_of(graph: &OwnershipGraph, node: NodeId) -> BorrowFrequencyAnalysis {
    // Build a subgraph with only borrows on this variable
    let borrow_edges: Vec<_> = graph
        .edges()
        .iter()
        .filter(|e| e.is_borrow() && e.target == node)
        .collect();

    if borrow_edges.is_empty() {
        return BorrowFrequencyAnalysis {
            total_borrows: 0,
            shared_borrows: 0,
            mutable_borrows: 0,
            avg_duration: 0.0,
            max_duration: 0,
            min_duration: 0,
            median_duration: 0,
            frequency: 0.0,
            max_concurrent: 0,
            bursts: vec![],
        };
    }

    let shared_borrows = borrow_edges.iter().filter(|e| !e.is_mutable()).count();
    let mutable_borrows = borrow_edges.iter().filter(|e| e.is_mutable()).count();

    let mut durations: Vec<u64> = borrow_edges.iter().filter_map(|e| e.duration()).collect();
    durations.sort();

    let avg_duration = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<u64>() as f64 / durations.len() as f64
    };
    let max_duration = durations.last().copied().unwrap_or(0);
    let min_duration = durations.first().copied().unwrap_or(0);
    let median_duration = if durations.is_empty() {
        0
    } else {
        durations[durations.len() / 2]
    };

    let min_ts = borrow_edges.iter().map(|e| e.created_at).min().unwrap_or(0);
    let max_ts = borrow_edges
        .iter()
        .map(|e| e.ended_at.unwrap_or(e.created_at))
        .max()
        .unwrap_or(0);
    let time_span = (max_ts - min_ts).max(1);
    let frequency = borrow_edges.len() as f64 * 100.0 / time_span as f64;

    let mut events: Vec<(u64, i32)> = Vec::new();
    for e in &borrow_edges {
        events.push((e.created_at, 1));
        if let Some(end) = e.ended_at {
            events.push((end, -1));
        }
    }
    events.sort_by_key(|(ts, _)| *ts);
    let mut concurrent = 0i32;
    let mut max_concurrent = 0usize;
    for (_, delta) in &events {
        concurrent += delta;
        max_concurrent = max_concurrent.max(concurrent as usize);
    }

    BorrowFrequencyAnalysis {
        total_borrows: borrow_edges.len(),
        shared_borrows,
        mutable_borrows,
        avg_duration,
        max_duration,
        min_duration,
        median_duration,
        frequency,
        max_concurrent,
        bursts: vec![], // simplified for per-variable
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.4 Scope depth distribution
// ═══════════════════════════════════════════════════════════════════════════

/// Scope depth distribution analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthDistribution {
    /// Histogram: depth -> count of variables at that depth.
    pub histogram: Vec<(u32, usize)>,
    pub max_depth: u32,
    pub avg_depth: f64,
    /// Variables at the maximum depth.
    pub deepest_variables: Vec<NodeId>,
}

/// Compute scope depth distribution.
pub fn depth_distribution(graph: &OwnershipGraph) -> DepthDistribution {
    let mut depth_counts: HashMap<u32, usize> = HashMap::new();
    let mut max_depth = 0u32;
    let mut deepest_variables = Vec::new();
    let mut total_depth = 0u64;
    let mut var_count = 0usize;

    for node in graph.nodes() {
        if let Node::Variable(v) = node {
            let depth = v.scope_depth;
            *depth_counts.entry(depth).or_default() += 1;
            total_depth += depth as u64;
            var_count += 1;

            if depth > max_depth {
                max_depth = depth;
                deepest_variables.clear();
                deepest_variables.push(v.id);
            } else if depth == max_depth {
                deepest_variables.push(v.id);
            }
        }
    }

    let avg_depth = if var_count > 0 {
        total_depth as f64 / var_count as f64
    } else {
        0.0
    };

    let mut histogram: Vec<(u32, usize)> = depth_counts.into_iter().collect();
    histogram.sort_by_key(|(d, _)| *d);

    DepthDistribution {
        histogram,
        max_depth,
        avg_depth,
        deepest_variables,
    }
}

/// Get the scope depth of a specific variable.
pub fn scope_depth(graph: &OwnershipGraph, node: NodeId) -> u32 {
    match graph.get_node(node) {
        Some(Node::Variable(v)) => v.scope_depth,
        _ => 0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.5 Smart pointer usage patterns
// ═══════════════════════════════════════════════════════════════════════════

/// Smart pointer usage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPointerReport {
    pub rc_families: Vec<RcFamily>,
    pub arc_families: Vec<ArcFamily>,
    pub refcell_usage: Vec<RefCellUsage>,
    pub mutex_usage: Vec<MutexUsage>,
}

/// An Rc clone family (all clones of the same value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcFamily {
    pub origin: NodeId,
    pub clone_count: usize,
    pub peak_ref_count: u32,
    pub total_lifetime: u64,
    pub is_leaked: bool,
}

/// An Arc clone family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcFamily {
    pub origin: NodeId,
    pub clone_count: usize,
    pub peak_ref_count: u32,
    pub total_lifetime: u64,
    pub is_leaked: bool,
}

/// RefCell usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefCellUsage {
    pub node: NodeId,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub max_concurrent_borrows: usize,
}

/// Mutex usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutexUsage {
    pub node: NodeId,
    pub lock_count: usize,
    pub avg_hold_time: f64,
    pub max_hold_time: u64,
}

/// Generate smart pointer usage report.
pub fn smart_pointer_report(graph: &OwnershipGraph) -> SmartPointerReport {
    let mut rc_origins: HashMap<NodeId, usize> = HashMap::new();
    let mut arc_origins: HashMap<NodeId, usize> = HashMap::new();
    let mut refcell_nodes: HashMap<NodeId, (usize, usize)> = HashMap::new(); // (immut, mut)
    let mut mutex_nodes: HashMap<NodeId, Vec<u64>> = HashMap::new(); // hold times

    for edge in graph.edges() {
        match &edge.kind {
            EdgeKind::RcClone { .. } => {
                *rc_origins.entry(edge.target).or_default() += 1;
            }
            EdgeKind::ArcClone { .. } => {
                *arc_origins.entry(edge.target).or_default() += 1;
            }
            EdgeKind::RefCellBorrow { mutable } => {
                let entry = refcell_nodes.entry(edge.target).or_default();
                if *mutable {
                    entry.1 += 1;
                } else {
                    entry.0 += 1;
                }
            }
            EdgeKind::LockAcquire { .. } => {
                let hold_time = edge.duration().unwrap_or(0);
                mutex_nodes.entry(edge.target).or_default().push(hold_time);
            }
            _ => {}
        }
    }

    let rc_families: Vec<RcFamily> = rc_origins
        .iter()
        .map(|(&origin, &clone_count)| {
            let history = crate::temporal::ref_count_history(graph, origin);
            let total_lifetime = graph
                .get_node(origin)
                .and_then(|n| n.end_time().map(|e| e - n.start_time()))
                .unwrap_or(0);
            RcFamily {
                origin,
                clone_count,
                peak_ref_count: history.peak_count,
                total_lifetime,
                is_leaked: history.is_leaked,
            }
        })
        .collect();

    let arc_families: Vec<ArcFamily> = arc_origins
        .iter()
        .map(|(&origin, &clone_count)| {
            let history = crate::temporal::ref_count_history(graph, origin);
            let total_lifetime = graph
                .get_node(origin)
                .and_then(|n| n.end_time().map(|e| e - n.start_time()))
                .unwrap_or(0);
            ArcFamily {
                origin,
                clone_count,
                peak_ref_count: history.peak_count,
                total_lifetime,
                is_leaked: history.is_leaked,
            }
        })
        .collect();

    let refcell_usage: Vec<RefCellUsage> = refcell_nodes
        .iter()
        .map(|(&node, &(immut, mutable))| {
            // Compute max concurrent borrows via sweep line
            let borrow_edges: Vec<_> = graph
                .edges()
                .iter()
                .filter(|e| matches!(e.kind, EdgeKind::RefCellBorrow { .. }) && e.target == node)
                .collect();
            let mut events: Vec<(u64, i32)> = Vec::new();
            for e in &borrow_edges {
                events.push((e.created_at, 1));
                if let Some(end) = e.ended_at {
                    events.push((end, -1));
                }
            }
            events.sort_by_key(|(ts, _)| *ts);
            let mut concurrent = 0i32;
            let mut max_concurrent = 0usize;
            for (_, delta) in &events {
                concurrent += delta;
                max_concurrent = max_concurrent.max(concurrent as usize);
            }

            RefCellUsage {
                node,
                immutable_borrows: immut,
                mutable_borrows: mutable,
                max_concurrent_borrows: max_concurrent,
            }
        })
        .collect();

    let mutex_usage: Vec<MutexUsage> = mutex_nodes
        .iter()
        .map(|(&node, hold_times)| {
            let lock_count = hold_times.len();
            let avg_hold_time = if lock_count > 0 {
                hold_times.iter().sum::<u64>() as f64 / lock_count as f64
            } else {
                0.0
            };
            let max_hold_time = hold_times.iter().copied().max().unwrap_or(0);
            MutexUsage {
                node,
                lock_count,
                avg_hold_time,
                max_hold_time,
            }
        })
        .collect();

    SmartPointerReport {
        rc_families,
        arc_families,
        refcell_usage,
        mutex_usage,
    }
}
