# Smart Pointers

Deep dive into Rust smart pointers (`Box`, `Rc`, `Arc`, `RefCell`, `Weak`) with BorrowScope tracking, including filtering API and pretty print summary.

## Demos

| Demo | Smart Pointer | Pattern |
|------|---------------|---------|
| 1 | `Box<T>` | Heap allocation, single ownership, move |
| 2 | `Rc<T>` | Reference counting, clone, drop order |
| 3 | `Rc<T>` Tree | Shared nodes in data structures |
| 4 | `Rc<RefCell<T>>` | Shared mutable state |
| 5 | `Weak<T>` | Breaking reference cycles (parent-child) |
| 6 | `Arc<T>` | Thread-safe sharing across threads |

## New Features Demonstrated

- **Filtering API**: `get_events_filtered()` with `is_rc()`, `is_arc()`, `is_refcell()` predicates
- **Pretty Print**: `print_summary()` for human-readable output
- **Summary Struct**: `get_summary()` for programmatic access to statistics

## Run

```bash
cargo run
```

## Filtering API Example

```rust
let rc_events = get_events_filtered(|e| e.is_rc());
let arc_events = get_events_filtered(|e| e.is_arc());
let refcell_events = get_events_filtered(|e| e.is_refcell());

println!("Rc events: {}", rc_events.len());
println!("Arc events: {}", arc_events.len());
println!("RefCell events: {}", refcell_events.len());

let summary = get_summary();
println!("Summary: {} vars, {} Rc ops", summary.new_count, summary.rc_new_count + summary.rc_clone_count);
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
WeakNew { var_name: "weak", strong_count: 1, weak_count: 1 }
WeakUpgrade { var_name: "upgraded", success: true }
  → after strong dropped: upgrade returns None
```

## Sample Output

```
=== BorrowScope Summary ===
Variables: 1 created, 17 dropped
Borrows: 0 immutable, 0 mutable
Smart pointers: 14 Rc, 4 Arc
Interior mutability: 6 RefCell, 0 Cell

Rc events: 14
Arc events: 4
RefCell events: 6

Summary struct: 1 vars created, 17 Rc ops, 4 Arc ops
```

## Exported JSON

Tracking data is exported to `/tmp/smart-pointers.json`.
