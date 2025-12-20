# Borrow Conflicts

Comprehensive demonstration of Rust borrow checking with BorrowScope tracking, including RAII guards and filtering API.

## Scenarios Covered

| Section | Scenarios |
|---------|-----------|
| **1. Valid Patterns** | Multiple immutable, NLL, reborrowing, split_at_mut |
| **RAII Guards** | Automatic borrow tracking with `track_borrow_guard` |
| **2. Nested Borrows** | Borrow chains, nested mutation |
| **3. Struct Fields** | Disjoint field borrows |
| **4. RefCell** | Sequential access, multiple readers, try_borrow |
| **5. Rc\<RefCell\>** | Shared mutation, cross-handle conflicts |

## New Features Demonstrated

- **RAII Guards**: `track_new_guard()`, `track_borrow_guard()` for automatic drop tracking
- **Filtering API**: `get_borrow_events()`, `get_events_filtered()` for analysis
- **Pretty Print**: `print_summary()` for human-readable output

## Run

```bash
cargo run
```

## Key Demonstrations

### RAII Guards
```rust
let data = track_new_guard("data", vec![1, 2, 3]);
{
    let r1 = track_borrow_guard("r1", &*data);
    let r2 = track_borrow_guard("r2", &*data);
    // track_drop called automatically when guards go out of scope
}
```

### Valid Patterns
```rust
// Multiple immutable - OK
let r1 = &data;
let r2 = &data;

// NLL - borrow ends at last use
let r = &data;
let _ = r[0];  // last use
let m = &mut data;  // OK, r is done

// Reborrowing
let m = &mut data;
let reborrow = &*m;  // OK
drop(reborrow);
m.push(1);  // OK

// Split borrows
let (left, right) = data.split_at_mut(3);
left[0] = 1;   // OK
right[0] = 2;  // OK, disjoint
```

### Filtering API
```rust
let borrows = get_borrow_events();
let mutable_borrows = get_events_filtered(|e| {
    matches!(e, Event::Borrow { mutable: true, .. })
});
println!("Mutable: {}, Immutable: {}", 
    mutable_borrows.len(), 
    borrows.len() - mutable_borrows.len());
```

## Sample Output

```
=== BorrowScope Summary ===
Variables: 4 created, 26 dropped
Borrows: 11 immutable, 7 mutable
Smart pointers: 3 Rc, 0 Arc
Interior mutability: 19 RefCell, 0 Cell

Borrow Analysis:
  Total borrows: 18
  Mutable borrows: 7
  Immutable borrows: 11
```

## Exported JSON

Tracking data is exported to `/tmp/borrow-conflicts.json`.
