# Ownership Patterns

A hands-on example demonstrating Rust ownership patterns with BorrowScope runtime tracking.

## What It Demonstrates

| Pattern | Functions Used |
|---------|---------------|
| Basic ownership | `track_new`, `track_move`, `track_drop` |
| Borrowing | `track_borrow`, `track_borrow_mut` |
| Shared ownership (Rc) | `track_rc_new`, `track_rc_clone` |
| Interior mutability (RefCell) | `track_refcell_new`, `refcell_borrow!`, `refcell_borrow_mut!` |

## Run

```bash
cargo run
```

## Output

The example prints:
1. Demo output showing each pattern in action
2. All captured events with timestamps and variable IDs
3. Graph statistics
4. Path to exported JSON file

## Sample Event Output

```
New { timestamp: 0, var_name: "s1", var_id: "s1_0", type_name: "String" }
Move { timestamp: 1, from_id: "s1", to_name: "s2", to_id: "s2_1" }
RcClone { timestamp: 12, var_name: "rc2", source_id: "rc1", strong_count: 2 }
RefCellBorrow { borrow_id: "cell_ref", is_mutable: false, location: "src/main.rs:114" }
```

## Exported JSON

The tracking data is exported to `/tmp/ownership-patterns.json` for visualization or further analysis.
