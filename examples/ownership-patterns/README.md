# Ownership Patterns

A hands-on example demonstrating Rust ownership patterns with BorrowScope runtime tracking, including RAII guards and filtering API.

## What It Demonstrates

| Pattern | Functions Used |
|---------|---------------|
| Basic ownership | `track_new`, `track_move`, `track_drop` |
| Borrowing | `track_borrow`, `track_borrow_mut` |
| Shared ownership (Rc) | `track_rc_new`, `track_rc_clone` |
| Interior mutability (RefCell) | `track_refcell_new`, `refcell_borrow!`, `refcell_borrow_mut!` |
| **RAII Guards** | `track_new_guard`, `track_borrow_guard` |
| **Filtering API** | `get_borrow_events`, `get_events_filtered` |
| **Pretty Print** | `print_summary` |

## Run

```bash
cargo run
```

## New Features

### RAII Guards
```rust
let data = track_new_guard("guarded_data", vec![1, 2, 3]);
{
    let r = track_borrow_guard("guarded_ref", &*data);
    // track_drop called automatically when r goes out of scope
}
// track_drop called automatically when data goes out of scope
```

### Filtering API
```rust
let borrows = get_borrow_events();
let rc_events = get_events_filtered(|e| e.is_rc());
```

### Pretty Print Summary
```rust
print_summary();
// Output:
// === BorrowScope Summary ===
// Variables: 5 created, 12 dropped
// Borrows: 3 immutable, 2 mutable
// Smart pointers: 3 Rc, 0 Arc
// Interior mutability: 5 RefCell, 0 Cell
```

## Sample Event Output

```
New { timestamp: 0, var_name: "s1", var_id: "s1_0", type_name: "String" }
Move { timestamp: 1, from_id: "s1", to_name: "s2", to_id: "s2_1" }
RcClone { timestamp: 12, var_name: "rc2", source_id: "rc1", strong_count: 2 }
RefCellBorrow { borrow_id: "cell_ref", is_mutable: false, location: "src/main.rs:114" }
```

## Exported JSON

The tracking data is exported to `/tmp/ownership-patterns.json` for visualization or further analysis.
