# Borrow Conflicts

Comprehensive demonstration of Rust borrow checking with BorrowScope conflict detection.

## Scenarios Covered

| Section | Scenarios |
|---------|-----------|
| **1. Valid Patterns** | Multiple immutable, NLL, reborrowing, split_at_mut |
| **2. Compile-Time Conflicts** | Use after move, mut+immut overlap, double mut, outlives |
| **3. Nested Borrows** | Borrow chains, nested mutation, outer access conflicts |
| **4. Struct Fields** | Disjoint field borrows, whole-struct conflicts |
| **5. RefCell** | Sequential access, multiple readers, try_borrow |
| **6. Rc\<RefCell\>** | Shared mutation, cross-handle conflicts |
| **7. Complex Lifetimes** | Interleaved borrows, borrow depth, connected components |

## Run

```bash
cargo run
```

## Key Demonstrations

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

### Conflict Detection
```
Conflict: Mutable + Immutable overlap
  t=10: r borrows data (immut)
  t=30: m borrows data (mut)  ← CONFLICT!
  t=50: r dropped

✗ CONFLICT: Mutable and immutable borrows of 'data' by: r, m
  Time range: 30 - 50

Timeline:
  t=10: [r (immut)]
  t=30: [m (mut), r (immut)]  ← Both active!
  t=50: [m (mut)]
```

### RefCell Runtime Checking
```rust
// try_borrow avoids panics
let m = cell.borrow_mut();
match cell.try_borrow() {
    Err(_) => println!("Already borrowed"),
    Ok(r) => { /* use r */ }
}

// Rc<RefCell> tracks across handles
let clone1 = Rc::clone(&shared);
let clone2 = Rc::clone(&shared);
let r = clone1.borrow();
let m = clone2.borrow_mut();  // PANIC! Same RefCell
```

### Graph Analysis Features
```rust
// Conflict detection
graph.find_conflicts_optimized()

// Borrow timeline
graph.conflict_timeline(owner_id)

// Borrow chain depth
graph.borrow_depth(var_id)

// Connected components
graph.connected_components()

// Validation
graph.validate()  // Checks lifetimes, cycles
```

## Output

Exported to `/tmp/borrow-conflicts.json`
