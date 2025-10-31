# Section 75: Optimizing Graph Performance

## Learning Objectives

By the end of this section, you will:
- Identify performance bottlenecks
- Implement caching strategies
- Optimize memory usage
- Use incremental updates
- Benchmark graph operations

## Prerequisites

- Section 74 (Graph Visualization Data Format)
- Understanding of performance profiling
- Familiarity with benchmarking

---

## Performance Challenges

**Common bottlenecks:**
1. Lock contention (concurrent access)
2. Repeated graph traversals
3. Large serialization overhead
4. Memory allocation churn
5. Expensive queries

---

## Caching Strategy

### Query Result Cache

```rust
use std::cell::RefCell;
use std::collections::HashMap;

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
        
        let borrowers: Vec<usize> = self.graph.query()
            .direct_borrowers(id)
            .into_iter()
            .map(|(v, _)| v.id)
            .collect();
        
        self.borrowers_cache.borrow_mut().insert(id, borrowers.clone());
        borrowers
    }
    
    pub fn invalidate(&self) {
        self.borrowers_cache.borrow_mut().clear();
        *self.stats_cache.borrow_mut() = None;
    }
    
    pub fn add_variable(&mut self, var: Variable) {
        self.graph.add_variable(var);
        self.invalidate();
    }
}
```

---

## Incremental Updates

### Delta-Based Serialization

```rust
#[derive(Serialize, Deserialize)]
pub struct GraphDelta {
    pub added_nodes: Vec<NodeExport>,
    pub removed_nodes: Vec<usize>,
    pub added_edges: Vec<EdgeExport>,
    pub removed_edges: Vec<String>,
}

pub struct IncrementalGraph {
    graph: OwnershipGraph,
    last_export_version: RefCell<usize>,
    changes: RefCell<Vec<GraphChange>>,
}

#[derive(Clone)]
enum GraphChange {
    NodeAdded(Variable),
    NodeRemoved(usize),
    EdgeAdded { from: usize, to: usize, rel: Relationship },
}

impl IncrementalGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            last_export_version: RefCell::new(0),
            changes: RefCell::new(Vec::new()),
        }
    }
    
    pub fn add_variable(&mut self, var: Variable) {
        self.changes.borrow_mut().push(GraphChange::NodeAdded(var.clone()));
        self.graph.add_variable(var);
    }
    
    pub fn export_delta(&self) -> GraphDelta {
        let changes = self.changes.borrow();
        
        let mut added_nodes = Vec::new();
        let mut removed_nodes = Vec::new();
        let mut added_edges = Vec::new();
        
        for change in changes.iter() {
            match change {
                GraphChange::NodeAdded(var) => {
                    added_nodes.push(NodeExport {
                        id: var.id,
                        name: var.name.clone(),
                        type_name: var.type_name.clone(),
                        created_at: var.created_at,
                        dropped_at: var.dropped_at,
                        scope_depth: var.scope_depth,
                    });
                }
                GraphChange::NodeRemoved(id) => {
                    removed_nodes.push(*id);
                }
                GraphChange::EdgeAdded { from, to, rel } => {
                    added_edges.push(EdgeExport {
                        from_id: *from,
                        to_id: *to,
                        relationship: rel.clone(),
                    });
                }
            }
        }
        
        GraphDelta {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges: vec![],
        }
    }
    
    pub fn clear_changes(&self) {
        self.changes.borrow_mut().clear();
    }
}
```

---

## Memory Optimization

### String Interning

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct StringInterner {
    strings: HashMap<String, Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }
    
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            return Arc::clone(interned);
        }
        
        let arc: Arc<str> = Arc::from(s);
        self.strings.insert(s.to_string(), Arc::clone(&arc));
        arc
    }
}

#[derive(Clone)]
pub struct InternedVariable {
    pub id: usize,
    pub name: Arc<str>,
    pub type_name: Arc<str>,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}

pub struct OptimizedGraph {
    graph: StableGraph<InternedVariable, Relationship, Directed>,
    interner: StringInterner,
}

impl OptimizedGraph {
    pub fn add_variable(&mut self, id: usize, name: &str, type_name: &str, 
                        created_at: u64, scope_depth: usize) {
        let var = InternedVariable {
            id,
            name: self.interner.intern(name),
            type_name: self.interner.intern(type_name),
            created_at,
            dropped_at: None,
            scope_depth,
        };
        
        self.graph.add_node(var);
    }
}
```

### Compact Representation

```rust
// Instead of storing full Variable structs
#[derive(Clone)]
pub struct CompactVariable {
    pub id: u32,              // 4 bytes instead of 8
    pub name_idx: u32,        // Index into string table
    pub type_idx: u32,        // Index into string table
    pub created_at: u32,      // Relative timestamp
    pub dropped_at: Option<u32>,
    pub scope_depth: u8,      // 1 byte instead of 8
}

pub struct CompactGraph {
    graph: StableGraph<CompactVariable, u8, Directed>,  // u8 for relationship type
    strings: Vec<String>,
    base_time: u64,
}
```

---

## Lock-Free Operations

### Read-Write Lock

```rust
use parking_lot::RwLock;

pub struct ConcurrentGraph {
    graph: Arc<RwLock<OwnershipGraph>>,
}

impl ConcurrentGraph {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(OwnershipGraph::new())),
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
    
    // Fast read-only queries
    pub fn node_count(&self) -> usize {
        self.graph.read().node_count()
    }
}
```

### Atomic Counters

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct GraphMetrics {
    node_count: AtomicUsize,
    edge_count: AtomicUsize,
    query_count: AtomicUsize,
}

impl GraphMetrics {
    pub fn new() -> Self {
        Self {
            node_count: AtomicUsize::new(0),
            edge_count: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
        }
    }
    
    pub fn increment_nodes(&self) {
        self.node_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_node_count(&self) -> usize {
        self.node_count.load(Ordering::Relaxed)
    }
}
```

---

## Lazy Evaluation

```rust
pub struct LazyGraph {
    graph: OwnershipGraph,
    serialized: RefCell<Option<String>>,
}

impl LazyGraph {
    pub fn new(graph: OwnershipGraph) -> Self {
        Self {
            graph,
            serialized: RefCell::new(None),
        }
    }
    
    pub fn to_json(&self) -> &str {
        if self.serialized.borrow().is_none() {
            let json = self.graph.to_json().unwrap();
            *self.serialized.borrow_mut() = Some(json);
        }
        
        // Safe because we just ensured it's Some
        unsafe {
            self.serialized.borrow().as_ref().unwrap_unchecked()
        }
    }
    
    pub fn invalidate_cache(&self) {
        *self.serialized.borrow_mut() = None;
    }
}
```

---

## Batch Operations

```rust
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
    
    pub fn queue_borrow(&mut self, from: usize, to: usize, rel: Relationship) {
        self.pending_edges.push((from, to, rel));
    }
    
    pub fn flush(&mut self) {
        // Add all nodes
        for var in self.pending_nodes.drain(..) {
            self.graph.add_variable(var);
        }
        
        // Add all edges
        for (from, to, rel) in self.pending_edges.drain(..) {
            match rel {
                Relationship::BorrowsImmut { at } => {
                    self.graph.add_borrow(from, to, false, at);
                }
                Relationship::BorrowsMut { at } => {
                    self.graph.add_borrow(from, to, true, at);
                }
                _ => {}
            }
        }
    }
}
```

---

## Benchmarking

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_add_nodes() {
        let mut graph = OwnershipGraph::new();
        
        let start = Instant::now();
        for i in 0..10000 {
            graph.add_variable(Variable {
                id: i,
                name: format!("var_{}", i),
                type_name: "i32".into(),
                created_at: i as u64,
                dropped_at: None,
                scope_depth: 0,
            });
        }
        let duration = start.elapsed();
        
        println!("Add 10k nodes: {:?}", duration);
        println!("Per node: {:?}", duration / 10000);
    }

    #[test]
    fn bench_query_borrowers() {
        let mut graph = OwnershipGraph::new();
        
        // Setup: 1 node with 1000 borrowers
        graph.add_variable(Variable {
            id: 0, name: "x".into(), type_name: "i32".into(),
            created_at: 0, dropped_at: None, scope_depth: 0,
        });
        
        for i in 1..=1000 {
            graph.add_variable(Variable {
                id: i, name: format!("r{}", i), type_name: "&i32".into(),
                created_at: i as u64, dropped_at: None, scope_depth: 0,
            });
            graph.add_borrow(i, 0, false, i as u64);
        }
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = graph.query().direct_borrowers(0);
        }
        let duration = start.elapsed();
        
        println!("1000 queries: {:?}", duration);
        println!("Per query: {:?}", duration / 1000);
    }

    #[test]
    fn bench_serialization() {
        let mut graph = OwnershipGraph::new();
        
        for i in 0..1000 {
            graph.add_variable(Variable {
                id: i, name: format!("var_{}", i), type_name: "i32".into(),
                created_at: i as u64, dropped_at: None, scope_depth: 0,
            });
        }
        
        let start = Instant::now();
        let _ = graph.to_json();
        let duration = start.elapsed();
        
        println!("Serialize 1000 nodes: {:?}", duration);
    }

    #[test]
    fn bench_cached_vs_uncached() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 0, name: "x".into(), type_name: "i32".into(),
            created_at: 0, dropped_at: None, scope_depth: 0,
        });
        
        for i in 1..=100 {
            graph.add_variable(Variable {
                id: i, name: format!("r{}", i), type_name: "&i32".into(),
                created_at: i as u64, dropped_at: None, scope_depth: 0,
            });
            graph.add_borrow(i, 0, false, i as u64);
        }
        
        // Uncached
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = graph.query().direct_borrowers(0);
        }
        let uncached = start.elapsed();
        
        // Cached
        let cached_graph = CachedOwnershipGraph::new(graph);
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = cached_graph.borrowers_of(0);
        }
        let cached = start.elapsed();
        
        println!("Uncached: {:?}", uncached);
        println!("Cached: {:?}", cached);
        println!("Speedup: {:.2}x", uncached.as_nanos() as f64 / cached.as_nanos() as f64);
    }
}
```

---

## Profiling

```rust
#[cfg(feature = "profiling")]
pub mod profiling {
    use std::time::Instant;
    
    pub struct ProfileScope {
        name: &'static str,
        start: Instant,
    }
    
    impl ProfileScope {
        pub fn new(name: &'static str) -> Self {
            Self {
                name,
                start: Instant::now(),
            }
        }
    }
    
    impl Drop for ProfileScope {
        fn drop(&mut self) {
            let duration = self.start.elapsed();
            println!("[PROFILE] {}: {:?}", self.name, duration);
        }
    }
    
    macro_rules! profile {
        ($name:expr) => {
            #[cfg(feature = "profiling")]
            let _scope = $crate::profiling::ProfileScope::new($name);
        };
    }
}
```

**Usage:**
```rust
pub fn expensive_query(&self) -> Vec<usize> {
    profile!("expensive_query");
    // ... implementation
}
```

---

## Memory Profiling

```rust
impl OwnershipGraph {
    pub fn memory_usage(&self) -> MemoryStats {
        let node_size = std::mem::size_of::<Variable>();
        let edge_size = std::mem::size_of::<Relationship>();
        
        let nodes_memory = self.graph.node_count() * node_size;
        let edges_memory = self.graph.edge_count() * edge_size;
        
        // Approximate string memory
        let string_memory: usize = self.graph.node_weights()
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

#[derive(Debug)]
pub struct MemoryStats {
    pub nodes_memory: usize,
    pub edges_memory: usize,
    pub string_memory: usize,
    pub total: usize,
}
```

---

## Performance Guidelines

**Best practices:**

1. **Cache expensive queries** - Store results of repeated queries
2. **Use RwLock over Mutex** - Allow concurrent reads
3. **Batch operations** - Reduce lock acquisitions
4. **Lazy serialization** - Only serialize when needed
5. **String interning** - Reduce memory for duplicate strings
6. **Incremental updates** - Send only changes to UI
7. **Profile first** - Measure before optimizing

---

## Key Takeaways

✅ **Caching** - Store expensive query results  
✅ **Incremental updates** - Delta-based serialization  
✅ **Memory optimization** - String interning, compact types  
✅ **Concurrency** - RwLock for read-heavy workloads  
✅ **Benchmarking** - Measure performance improvements  

---

## Further Reading

- [Rust performance book](https://nnethercote.github.io/perf-book/)
- [Flamegraph profiling](https://github.com/flamegraph-rs/flamegraph)
- [Criterion benchmarking](https://github.com/bheisler/criterion.rs)

---

**Previous:** [74-graph-visualization-data-format.md](./74-graph-visualization-data-format.md)  
**Next:** [CHAPTER_SUMMARY.md](./CHAPTER_SUMMARY.md)

**Progress:** 10/10 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
