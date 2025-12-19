# Smart Pointers

Deep dive into Rust smart pointers (`Box`, `Rc`, `Arc`, `RefCell`, `Weak`) with BorrowScope tracking.

## Demos

| Demo | Smart Pointer | Pattern |
|------|---------------|---------|
| 1 | `Box<T>` | Heap allocation, single ownership, move |
| 2 | `Rc<T>` | Reference counting, clone, drop order |
| 3 | `Rc<T>` Tree | Shared nodes in data structures |
| 4 | `Rc<RefCell<T>>` | Shared mutable state |
| 5 | `Weak<T>` | Breaking reference cycles (parent-child) |
| 6 | `Arc<T>` | Thread-safe sharing across threads |

## Run

```bash
cargo run
```

## Key Concepts Demonstrated

### Reference Count Tracking
```
RcNew { var_name: "rc1", strong_count: 1, weak_count: 0 }
RcClone { var_name: "rc2", source_id: "rc1", strong_count: 2 }
RcClone { var_name: "rc3", source_id: "rc1", strong_count: 3 }
Drop { var_id: "rc3" }  // count goes back to 2
```

### Shared Nodes in Trees
```
shared leaf count: 1
  → parent1 clones shared → count: 2
  → parent2 clones shared → count: 3
  → drop parent2 → count: 2
  → drop parent1 → count: 1
```

### Rc<RefCell<T>> Mutation
```
RefCellBorrow { borrow_id: "mut1", is_mutable: true }
  → push through first Rc
RefCellBorrow { borrow_id: "mut2", is_mutable: true }
  → push through cloned Rc (same underlying data)
```

### Weak References
```
Parent strong: 1, weak: 1  (child has Weak to parent)
Child strong: 2, weak: 0   (parent has Rc to child)
```

### Arc Across Threads
```
ArcClone { var_name: "thread_0", strong_count: 2 }
ArcClone { var_name: "thread_1", strong_count: 3 }
ArcClone { var_name: "thread_2", strong_count: 4 }
  → threads complete, drops happen
All threads done, count: 1
```

## Event Summary

| Event Type | Count | Description |
|------------|-------|-------------|
| RcNew | 9 | Rc allocations |
| RcClone | 8 | Rc clones with count tracking |
| ArcNew | 1 | Arc allocation |
| ArcClone | 3 | Arc clones for threads |
| RefCellBorrow | 3 | Interior mutability borrows |
| RefCellDrop | 3 | Borrow releases |
| Other | 23 | New, Move, Drop, Borrow |

## Output

Tracking data exported to `/tmp/smart-pointers.json`
