# Chapter 6 Summary: Graph Data Structures

## Overview

Chapter 6 covered the design and implementation of the ownership graph data structure, which is the core of BorrowScope's visualization capabilities. You learned how to model ownership relationships as a directed graph, implement efficient queries, and optimize for performance.

---

## What You Built

### Core Components

1. **OwnershipGraph** - Wrapper around petgraph with ownership-specific logic
2. **Variable nodes** - Track ID, name, type, timestamps, scope depth
3. **Relationship edges** - Model borrows, moves, Rc/Arc clones
4. **Query API** - Flexible interface for graph exploration
5. **Serialization** - Export to JSON, DOT, and visualization formats
6. **Performance optimizations** - Caching, incremental updates, memory efficiency

---

## Key Concepts

### Graph Theory Fundamentals (Section 66)

- **Directed graphs** - Edges have direction (borrower → owner)
- **Graph representations** - Adjacency list (best for sparse graphs)
- **Graph properties** - Acyclic (no cycles in valid Rust)
- **Graph algorithms** - DFS, BFS, topological sort, cycle detection

### Petgraph Library (Section 67)

- **StableGraph** - Stable node indices after removal
- **Generic types** - Customize node/edge data
- **Rich API** - Traversal, algorithms, serialization
- **Performance** - O(1) operations for most use cases

### Graph Design (Section 68)

```rust
pub struct Variable {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}

pub enum Relationship {
    BorrowsImmut { at: u64 },
    BorrowsMut { at: u64 },
    Moves { at: u64 },
    RcClone { at: u64, strong_count: usize },
    ArcClone { at: u64, strong_count: usize },
    RefCellBorrow { at: u64, is_mut: bool },
}
```

**Edge direction:** borrower → owner

### Graph Construction (Section 69)

```rust
impl OwnershipGraph {
    pub fn add_variable(&mut self, var: Variable) -> NodeIndex;
    pub fn add_borrow(&mut self, borrower_id: usize, owner_id: usize, 
                      is_mut: bool, at: u64) -> Option<EdgeIndex>;
    pub fn add_move(&mut self, from_id: usize, to_id: usize, at: u64) -> Option<EdgeIndex>;
    pub fn mark_dropped(&mut self, id: usize, at: u64) -> bool;
}
```

### Traversal Algorithms (Section 70)

- **DFS** - Deep exploration, find all dependencies
- **BFS** - Level-by-level, shortest paths
- **Topological sort** - Determine drop order
- **Cycle detection** - Validate ownership integrity
- **Connected components** - Group related variables

### Conflict Detection (Section 71)

**Rust's borrowing rules:**
1. Multiple immutable borrows allowed
2. Only one mutable borrow at a time
3. No immutable borrows while mutable borrow exists

**Detection algorithm:**
- Find active borrows at each time
- Check for overlapping intervals
- Report conflicts with context

### Serialization (Section 72)

**Formats:**
- **JSON** - Standard, human-readable
- **Compact JSON** - Optimized for large graphs
- **DOT** - Graphviz visualization
- **MessagePack** - Binary format
- **Streaming** - Handle large graphs

### Query API (Section 73)

```rust
// Basic queries
graph.query().find_by_name("x");
graph.query().find_references();
graph.query().alive_at(1500);

// Relationship queries
graph.query().direct_borrowers(id);
graph.query().all_borrowers(id);
graph.query().borrow_chain(from_id, to_id);

// Statistics
let stats = graph.statistics();

// Query DSL
graph.query()
    .filter()
    .name_contains("temp")
    .alive_at(1500)
    .execute();
```

### Visualization Format (Section 74)

**Cytoscape.js structure:**
```json
{
  "elements": {
    "nodes": [...],
    "edges": [...]
  },
  "style": [...],
  "layout": {...}
}
```

**Features:**
- Node/edge styling with CSS classes
- Layout algorithm configuration
- Tooltip data
- Timeline frames for animation

### Performance Optimization (Section 75)

**Strategies:**
1. **Caching** - Store expensive query results
2. **Incremental updates** - Delta-based serialization
3. **String interning** - Reduce memory for duplicates
4. **RwLock** - Allow concurrent reads
5. **Batch operations** - Reduce lock contention
6. **Lazy evaluation** - Compute only when needed

---

## Code Artifacts

All sections include complete implementations:

- `borrowscope-graph/src/lib.rs` - Core graph implementation
- Graph construction methods
- Traversal algorithms
- Conflict detection
- Serialization formats
- Query API with DSL
- Visualization export
- Performance optimizations
- Comprehensive tests

---

## Testing Strategy

**Unit tests:**
- Graph construction
- Traversal algorithms
- Conflict detection
- Serialization roundtrips
- Query correctness

**Benchmarks:**
- Add nodes/edges
- Query performance
- Serialization speed
- Cached vs uncached
- Memory usage

---

## Integration Points

### With Runtime Tracker (Chapter 3)

```rust
pub fn track_new<T>(name: &str, value: T) -> T {
    let mut graph = GRAPH.lock();
    graph.add_variable(Variable { ... });
    value
}

pub fn track_borrow<T>(borrower_name: &str, owner_id: usize, value: &T) -> &T {
    let mut graph = GRAPH.lock();
    graph.add_variable(Variable { ... });
    graph.add_borrow(borrower_id, owner_id, false, timestamp());
    value
}
```

### With UI (Chapter 8)

```rust
#[tauri::command]
fn load_graph() -> VisualizationExport {
    let graph = GRAPH.lock();
    graph.export_for_visualization()
}

#[tauri::command]
fn get_conflicts() -> Vec<BorrowConflict> {
    let graph = GRAPH.lock();
    graph.find_conflicts_optimized()
}
```

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Add node | O(1) | Amortized |
| Add edge | O(1) | Amortized |
| Find node | O(1) | HashMap lookup |
| Direct neighbors | O(degree) | Iterate edges |
| DFS/BFS | O(V + E) | Visit all nodes/edges |
| Topological sort | O(V + E) | Kahn's algorithm |
| Serialization | O(V + E) | Linear in graph size |

---

## Common Patterns

### Wrapper Pattern

```rust
pub struct OwnershipGraph {
    graph: StableGraph<Variable, Relationship, Directed>,
    id_to_node: HashMap<usize, NodeIndex>,
}
```

Encapsulate petgraph with domain-specific methods.

### Builder Pattern

```rust
graph.query()
    .filter()
    .name_contains("x")
    .alive_at(1500)
    .execute();
```

Chainable API for complex queries.

### Cache Invalidation

```rust
pub struct CachedGraph {
    graph: OwnershipGraph,
    cache: RefCell<HashMap<...>>,
}

impl CachedGraph {
    pub fn invalidate(&self) {
        self.cache.borrow_mut().clear();
    }
}
```

---

## Key Takeaways

✅ **Directed graphs** - Model ownership relationships with direction  
✅ **StableGraph** - Petgraph type with stable indices  
✅ **Rich queries** - Flexible API for graph exploration  
✅ **Conflict detection** - Validate Rust's borrowing rules  
✅ **Multiple formats** - JSON, DOT, visualization-ready  
✅ **Performance** - Caching, incremental updates, optimization  
✅ **Testing** - Comprehensive unit tests and benchmarks  

---

## What's Next?

**Chapter 7: CLI Development with Clap**

You'll build the command-line interface for BorrowScope:
- Subcommand structure (visualize, run, export)
- Argument parsing with Clap v4
- File instrumentation engine
- Cargo integration
- User-friendly error messages

The graph data structures you built in this chapter will be consumed by the CLI and UI to provide powerful ownership visualization.

---

## Further Practice

**Exercises:**

1. **Add graph validation** - Check for invalid patterns
2. **Implement graph diff** - Compare two graphs
3. **Add more queries** - Find longest borrow chain, most complex variable
4. **Optimize serialization** - Use binary format for large graphs
5. **Add graph metrics** - Compute centrality, clustering coefficient
6. **Implement undo/redo** - Track graph history

**Challenge:**

Build a graph simplification algorithm that merges nodes with similar properties to reduce visual complexity while preserving important relationships.

---

**Completion:** Chapter 6 complete! 🎉

**Progress:** 75/210+ sections (36% overall)

**Next:** [Chapter 7: CLI Development with Clap](../chapter-07/76-command-line-interface-design.md)
