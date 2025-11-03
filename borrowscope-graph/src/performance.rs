use crate::{OwnershipGraph, Relationship, Variable};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Caching
// ============================================================================

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: f64,
    pub max_depth: usize,
}

pub struct CachedOwnershipGraph {
    graph: OwnershipGraph,
    borrowers_cache: RefCell<HashMap<usize, Vec<usize>>>,
    stats_cache: RefCell<Option<GraphStats>>,
}

impl CachedOwnershipGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            borrowers_cache: RefCell::new(HashMap::new()),
            stats_cache: RefCell::new(None),
        }
    }

    pub fn borrowers_of(&self, id: usize) -> Vec<usize> {
        if let Some(cached) = self.borrowers_cache.borrow().get(&id) {
            return cached.clone();
        }

        let borrowers: Vec<usize> = self
            .graph
            .borrowers_of(id)
            .into_iter()
            .map(|v| v.id)
            .collect();

        self.borrowers_cache
            .borrow_mut()
            .insert(id, borrowers.clone());
        borrowers
    }

    pub fn stats(&self) -> GraphStats {
        if let Some(cached) = self.stats_cache.borrow().as_ref() {
            return cached.clone();
        }

        let node_count = self.graph.node_count();
        let edge_count = self.graph.edge_count();
        let avg_degree = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        let max_depth = (0..self.graph.node_count())
            .filter_map(|i| {
                self.graph
                    .graph
                    .node_indices()
                    .nth(i)
                    .and_then(|idx| self.graph.graph.node_weight(idx))
                    .map(|v| v.scope_depth)
            })
            .max()
            .unwrap_or(0);

        let stats = GraphStats {
            node_count,
            edge_count,
            avg_degree,
            max_depth,
        };

        *self.stats_cache.borrow_mut() = Some(stats.clone());
        stats
    }

    pub fn invalidate(&self) {
        self.borrowers_cache.borrow_mut().clear();
        *self.stats_cache.borrow_mut() = None;
    }

    pub fn graph(&self) -> &OwnershipGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut OwnershipGraph {
        self.invalidate();
        &mut self.graph
    }
}

// ============================================================================
// Concurrent Access
// ============================================================================

pub struct ConcurrentGraph {
    graph: Arc<RwLock<OwnershipGraph>>,
}

impl ConcurrentGraph {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(OwnershipGraph::new())),
        }
    }

    pub fn from_graph(graph: OwnershipGraph) -> Self {
        Self {
            graph: Arc::new(RwLock::new(graph)),
        }
    }

    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&OwnershipGraph) -> R,
    {
        let graph = self.graph.read();
        f(&graph)
    }

    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut OwnershipGraph) -> R,
    {
        let mut graph = self.graph.write();
        f(&mut graph)
    }

    pub fn node_count(&self) -> usize {
        self.graph.read().node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.read().edge_count()
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            graph: Arc::clone(&self.graph),
        }
    }
}

impl Default for ConcurrentGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Batch Operations
// ============================================================================

pub struct BatchGraph {
    graph: OwnershipGraph,
    pending_nodes: Vec<Variable>,
    pending_edges: Vec<(usize, usize, Relationship)>,
}

impl BatchGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            pending_nodes: Vec::new(),
            pending_edges: Vec::new(),
        }
    }

    pub fn queue_variable(&mut self, var: Variable) {
        self.pending_nodes.push(var);
    }

    pub fn queue_borrow(&mut self, from: usize, to: usize, is_mut: bool, at: u64) {
        let rel = if is_mut {
            Relationship::BorrowsMut { at }
        } else {
            Relationship::BorrowsImmut { at }
        };
        self.pending_edges.push((from, to, rel));
    }

    pub fn queue_move(&mut self, from: usize, to: usize, at: u64) {
        self.pending_edges
            .push((from, to, Relationship::Moves { at }));
    }

    pub fn flush(&mut self) -> usize {
        let count = self.pending_nodes.len() + self.pending_edges.len();

        for var in self.pending_nodes.drain(..) {
            self.graph.add_variable(var);
        }

        for (from, to, rel) in self.pending_edges.drain(..) {
            match rel {
                Relationship::BorrowsImmut { at } => {
                    self.graph.add_borrow(from, to, false, at);
                }
                Relationship::BorrowsMut { at } => {
                    self.graph.add_borrow(from, to, true, at);
                }
                Relationship::Moves { at } => {
                    self.graph.add_move(from, to, at);
                }
                Relationship::RcClone { at, strong_count } => {
                    self.graph.add_rc_clone(from, to, strong_count, at);
                }
                Relationship::ArcClone { at, strong_count } => {
                    self.graph.add_arc_clone(from, to, strong_count, at);
                }
                Relationship::RefCellBorrow { at, is_mut } => {
                    self.graph.add_refcell_borrow(from, to, is_mut, at);
                }
            }
        }

        count
    }

    pub fn pending_count(&self) -> usize {
        self.pending_nodes.len() + self.pending_edges.len()
    }

    pub fn graph(&self) -> &OwnershipGraph {
        &self.graph
    }

    pub fn into_graph(mut self) -> OwnershipGraph {
        self.flush();
        self.graph
    }
}

// ============================================================================
// Memory Statistics
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    pub nodes_memory: usize,
    pub edges_memory: usize,
    pub string_memory: usize,
    pub total: usize,
}

impl OwnershipGraph {
    pub fn memory_usage(&self) -> MemoryStats {
        let node_size = std::mem::size_of::<Variable>();
        let edge_size = std::mem::size_of::<Relationship>();

        let nodes_memory = self.node_count() * node_size;
        let edges_memory = self.edge_count() * edge_size;

        let string_memory: usize = self
            .graph
            .node_weights()
            .map(|v| v.name.len() + v.type_name.len())
            .sum();

        MemoryStats {
            nodes_memory,
            edges_memory,
            string_memory,
            total: nodes_memory + edges_memory + string_memory,
        }
    }
}

// ============================================================================
// Lazy Serialization
// ============================================================================

pub struct LazyGraph {
    graph: OwnershipGraph,
    json_cache: RefCell<Option<String>>,
}

impl LazyGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            json_cache: RefCell::new(None),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        if let Some(cached) = self.json_cache.borrow().as_ref() {
            return Ok(cached.clone());
        }

        let json = self.graph.to_json()?;
        *self.json_cache.borrow_mut() = Some(json.clone());
        Ok(json)
    }

    pub fn invalidate_cache(&self) {
        *self.json_cache.borrow_mut() = None;
    }

    pub fn graph(&self) -> &OwnershipGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut OwnershipGraph {
        self.invalidate_cache();
        &mut self.graph
    }
}

// ============================================================================
// Performance Metrics
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct GraphMetrics {
    node_count: AtomicUsize,
    edge_count: AtomicUsize,
    query_count: AtomicUsize,
    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
}

impl GraphMetrics {
    pub fn new() -> Self {
        Self {
            node_count: AtomicUsize::new(0),
            edge_count: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        }
    }

    pub fn increment_nodes(&self) {
        self.node_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_edges(&self) {
        self.edge_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_queries(&self) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_node_count(&self) -> usize {
        self.node_count.load(Ordering::Relaxed)
    }

    pub fn get_edge_count(&self) -> usize {
        self.edge_count.load(Ordering::Relaxed)
    }

    pub fn get_query_count(&self) -> usize {
        self.query_count.load(Ordering::Relaxed)
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn reset(&self) {
        self.node_count.store(0, Ordering::Relaxed);
        self.edge_count.store(0, Ordering::Relaxed);
        self.query_count.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }
}

impl Default for GraphMetrics {
    fn default() -> Self {
        Self::new()
    }
}
