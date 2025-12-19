# Graph Visualization

ASCII visualization demonstrating all BorrowScope graph structures, algorithms, and APIs.

## Run

```bash
cargo run
```

## Sections

| Section | APIs Demonstrated |
|---------|-------------------|
| **1. Basic Graph** | `add_variable()`, `node_count()` |
| **2. Borrow Relationships** | `add_borrow()`, `borrowers_of()` |
| **3. Reference Counting** | `add_rc_clone()`, `add_arc_clone()` |
| **4. Graph Traversal** | `dfs_from()`, `bfs_from()`, `shortest_path()`, `topological_order()`, `can_reach()` |
| **5. Conflict Detection** | `find_conflicts_optimized()`, `active_borrows_at_time()`, `conflict_timeline()` |
| **6. Timeline** | `is_alive()`, `created_at`, `dropped_at` |
| **7. Statistics** | `statistics()`, `connected_components()`, `validate()`, `has_cycles()` |
| **8. Serialization** | `to_json()`, `to_dot()`, `to_messagepack()`, `from_json()` |

## ASCII Output Examples

### Borrow Graph
```
                ┌──────────────┐
                │     data     │
                └──────┬───────┘
                       │
       ┌───────────────┼───────────────┐
  ┌────────┐      ┌────────┐      ┌────────┐
  │   r1   │      │   r2   │      │   m    │
  │ (imm)  │      │ (imm)  │      │ (mut)  │
  └────────┘      └────────┘      └────────┘
```

### Conflict Timeline
```
t=0        t=10       t=30       t=60       t=80      t=100
│          │          │          │          │          │
├──────────┴──────────┴──────────┴──────────┴──────────┤ data
│          ├──────────────────────┤                    │ r (imm)
│                     ├──────────────────────┤         │ m (mut)
│                     │◄─ CONFLICT ─►│                 │
```

### Variable Lifetimes
```
t=0   10   20   30   40   50   60   70   80   90   100
│     │    │    │    │    │    │    │    │    │    │
├───────────────────────────────────┤               a
          ├───────────────────────────────────┤     b
     ├───────────────┤                              r_a
               ├───────────────┤                    r_b
```

### Statistics Box
```
┌─────────────────────────────────┐
│       Graph Statistics          │
├─────────────────────────────────┤
│ Total variables:             5  │
│ Immutable borrows:           2  │
│ Mutable borrows:             1  │
│ Rc clones:                   1  │
└─────────────────────────────────┘
```

## Graph API Reference

### Construction
```rust
graph.add_variable(Variable { id, name, type_name, created_at, dropped_at, scope_depth })
graph.add_borrow(borrower_id, owner_id, is_mut, timestamp)
graph.add_move(from_id, to_id, timestamp)
graph.add_rc_clone(clone_id, original_id, strong_count, timestamp)
graph.add_arc_clone(clone_id, original_id, strong_count, timestamp)
graph.add_refcell_borrow(borrower_id, owner_id, is_mut, timestamp)
graph.mark_dropped(id, timestamp)
```

### Query
```rust
graph.get_variable(id) -> Option<&Variable>
graph.borrowers_of(id) -> Vec<&Variable>
graph.borrows(id) -> Vec<&Variable>
graph.is_alive(id, timestamp) -> bool
graph.all_variables() -> Iterator<&Variable>
graph.active_borrows_at(id, timestamp) -> Vec<(&Variable, &Relationship)>
```

### Traversal
```rust
graph.dfs_from(start_id) -> Vec<usize>
graph.bfs_from(start_id) -> Vec<usize>
graph.shortest_path(from_id, to_id) -> Option<Vec<usize>>
graph.topological_order() -> Result<Vec<usize>, String>
graph.drop_order() -> Vec<usize>
graph.can_reach(from_id, to_id) -> bool
graph.find_all_borrowers(id) -> Vec<usize>
graph.borrow_depth(id) -> usize
graph.borrow_chain(from_id, to_id) -> Option<Vec<usize>>
graph.connected_components() -> Vec<Vec<usize>>
```

### Conflict Detection
```rust
graph.find_conflicts() -> Vec<BorrowConflict>
graph.find_conflicts_optimized() -> Vec<BorrowConflict>
graph.check_conflicts_at(owner_id, timestamp) -> Option<BorrowConflict>
graph.active_borrows_at_time(owner_id, timestamp) -> Vec<(usize, bool)>
graph.conflict_timeline(owner_id) -> Vec<(u64, Vec<(usize, bool)>)>
```

### Validation & Statistics
```rust
graph.validate() -> Result<(), Vec<String>>
graph.has_cycles() -> bool
graph.statistics() -> GraphStatistics
graph.node_count() -> usize
graph.edge_count() -> usize
```

### Serialization
```rust
graph.to_json() -> Result<String, Error>
graph.to_json_compact() -> Result<String, Error>
graph.to_json_pretty() -> Result<String, Error>
graph.to_dot() -> String
graph.to_messagepack() -> Result<Vec<u8>, Error>
graph.from_json(json: &str) -> Result<Self, Error>
graph.from_messagepack(data: &[u8]) -> Result<Self, Error>
graph.export() -> GraphExport
graph.export_with_metadata() -> EnhancedGraphExport
graph.export_delta(previous: &GraphExport) -> GraphDelta
```

## Output

Exported to `/tmp/graph-visualization.json`
