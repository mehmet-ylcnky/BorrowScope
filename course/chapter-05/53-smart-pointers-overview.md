# Section 53: Smart Pointers Overview

## Learning Objectives

By the end of this section, you will:
- Understand different smart pointer types
- Recognize when to use each smart pointer
- Identify tracking challenges for smart pointers
- Design tracking strategies for Box, Rc, Arc, RefCell
- Prepare for smart pointer transformations

## Prerequisites

- Completed Section 52 (Lifetime Tracking)
- Understanding of heap vs stack allocation
- Familiarity with Rust's ownership model

---

## What Are Smart Pointers?

Smart pointers are data structures that:
1. Act like pointers (implement `Deref`)
2. Have additional metadata and capabilities
3. Own the data they point to

**Key types:**
- `Box<T>` - Heap allocation
- `Rc<T>` - Reference counting
- `Arc<T>` - Atomic reference counting
- `RefCell<T>` - Interior mutability
- `Cell<T>` - Interior mutability for Copy types

---

## Box<T> - Heap Allocation

### Purpose

Move data to the heap:

```rust
let x = Box::new(42);  // 42 is on the heap
```

### Use Cases

1. **Large data** - Avoid stack overflow
2. **Recursive types** - Enable self-referential structures
3. **Trait objects** - `Box<dyn Trait>`

### Ownership

```rust
let x = Box::new(42);
let y = x;  // Move: x is no longer valid
```

**Tracking strategy:** Treat like any owned value.

```rust
let x = track_new(1, "x", "Box<i32>", "line:1:9", Box::new(42));
track_move(1, 2, "line:2:9");
let y = x;
```

---

## Rc<T> - Reference Counting

### Purpose

Multiple ownership through reference counting:

```rust
use std::rc::Rc;

let x = Rc::new(42);
let y = Rc::clone(&x);  // Both own the data
// Data dropped when last Rc is dropped
```

### Use Cases

1. **Shared ownership** - Multiple parts need access
2. **Graph structures** - Nodes with multiple parents
3. **Caching** - Share expensive computations

### Ownership

```rust
let x = Rc::new(42);
let y = Rc::clone(&x);  // Increments ref count
drop(x);                // Decrements ref count
// Data still alive because y exists
```

**Tracking strategy:** Track clones as creating new references.

```rust
let x = track_new(1, "x", "Rc<i32>", "line:1:9", Rc::new(42));
let y = track_new(2, "y", "Rc<i32>", "line:2:9", Rc::clone(
    track_borrow(3, 1, false, "line:2:32", &x)
));
```

---

## Arc<T> - Atomic Reference Counting

### Purpose

Thread-safe reference counting:

```rust
use std::sync::Arc;
use std::thread;

let x = Arc::new(42);
let x_clone = Arc::clone(&x);

thread::spawn(move || {
    println!("{}", x_clone);
});
```

### Use Cases

1. **Multi-threaded shared ownership**
2. **Concurrent data structures**
3. **Thread pools**

### Ownership

Same as `Rc`, but thread-safe:

```rust
let x = Arc::new(42);
let y = Arc::clone(&x);  // Atomic increment
```

**Tracking strategy:** Same as `Rc`, but note thread safety.

---

## RefCell<T> - Interior Mutability

### Purpose

Runtime borrow checking:

```rust
use std::cell::RefCell;

let x = RefCell::new(42);
let r1 = x.borrow();      // Immutable borrow at runtime
let r2 = x.borrow();      // OK: multiple immutable borrows
drop(r1);
drop(r2);
let m = x.borrow_mut();   // Mutable borrow at runtime
```

### Use Cases

1. **Mutability in immutable contexts**
2. **Mock objects in tests**
3. **Interior mutability patterns**

### Ownership

```rust
let x = RefCell::new(42);
*x.borrow_mut() = 100;  // Mutate through immutable reference
```

**Tracking strategy:** Track dynamic borrows.

```rust
let x = track_new(1, "x", "RefCell<i32>", "line:1:9", RefCell::new(42));
let r = track_new(
    2,
    "r",
    "Ref<i32>",
    "line:2:9",
    track_borrow(3, 1, false, "line:2:13", x.borrow())
);
```

---

## Cell<T> - Interior Mutability for Copy Types

### Purpose

Interior mutability without runtime checks (for Copy types):

```rust
use std::cell::Cell;

let x = Cell::new(42);
x.set(100);  // Mutate without &mut
let val = x.get();
```

### Use Cases

1. **Simple counters**
2. **Flags**
3. **Copy types that need mutation**

**Tracking strategy:** Track set/get operations.

---

## Comparison Table

| Type | Ownership | Thread-Safe | Runtime Cost | Use Case |
|------|-----------|-------------|--------------|----------|
| `Box<T>` | Single | N/A | None | Heap allocation |
| `Rc<T>` | Shared | No | Ref counting | Single-threaded sharing |
| `Arc<T>` | Shared | Yes | Atomic ref counting | Multi-threaded sharing |
| `RefCell<T>` | Single | No | Borrow checking | Interior mutability |
| `Cell<T>` | Single | No | None | Copy type mutation |

---

## Tracking Challenges

### Challenge 1: Deref Coercion

```rust
let x = Box::new(42);
let r = &*x;  // Deref to get &i32
```

**Problem:** The `*` operator is implicit in many contexts.

**Solution:** Track explicit derefs, accept that implicit ones are invisible.

### Challenge 2: Clone vs Borrow

```rust
let x = Rc::new(42);
let y = Rc::clone(&x);  // Not a borrow, creates new Rc
```

**Problem:** `clone` looks like it might copy, but it shares ownership.

**Solution:** Special handling for `Rc::clone` and `Arc::clone`.

### Challenge 3: Dynamic Borrows

```rust
let x = RefCell::new(42);
let r = x.borrow();  // Runtime borrow
```

**Problem:** Borrow happens at runtime, not compile time.

**Solution:** Track `borrow()` and `borrow_mut()` calls.

### Challenge 4: Weak References

```rust
let x = Rc::new(42);
let weak = Rc::downgrade(&x);  // Weak reference
```

**Problem:** Weak references don't prevent deallocation.

**Solution:** Track weak references separately.

---

## Tracking Strategy Summary

### Box<T>

```rust
// Original
let x = Box::new(42);

// Transformed
let x = track_new(1, "x", "Box<i32>", "line:1:9", Box::new(42));
```

**Treat as regular owned value.**

### Rc<T> / Arc<T>

```rust
// Original
let x = Rc::new(42);
let y = Rc::clone(&x);

// Transformed
let x = track_new(1, "x", "Rc<i32>", "line:1:9", Rc::new(42));
let y = track_new(
    2,
    "y",
    "Rc<i32>",
    "line:2:9",
    Rc::clone(track_borrow(3, 1, false, "line:2:20", &x))
);
```

**Track clones as creating shared ownership.**

### RefCell<T>

```rust
// Original
let x = RefCell::new(42);
let r = x.borrow();

// Transformed
let x = track_new(1, "x", "RefCell<i32>", "line:1:9", RefCell::new(42));
let r = track_new(
    2,
    "r",
    "Ref<i32>",
    "line:2:9",
    track_dynamic_borrow(3, 1, false, "line:2:13", x.borrow())
);
```

**Track dynamic borrows with special function.**

---

## Implementation Preview

We'll need to detect smart pointer operations:

```rust
impl OwnershipVisitor {
    fn is_smart_pointer_new(&self, expr: &Expr) -> Option<SmartPointerType> {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                
                if path_str.contains("Box :: new") {
                    return Some(SmartPointerType::Box);
                }
                if path_str.contains("Rc :: new") {
                    return Some(SmartPointerType::Rc);
                }
                if path_str.contains("Arc :: new") {
                    return Some(SmartPointerType::Arc);
                }
                if path_str.contains("RefCell :: new") {
                    return Some(SmartPointerType::RefCell);
                }
            }
        }
        None
    }
    
    fn is_rc_clone(&self, expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                return path_str.contains("Rc :: clone") || 
                       path_str.contains("Arc :: clone");
            }
        }
        false
    }
}

enum SmartPointerType {
    Box,
    Rc,
    Arc,
    RefCell,
    Cell,
}
```

---

## Testing Strategy

For each smart pointer type, test:

1. **Creation** - Track allocation
2. **Cloning** - Track shared ownership (Rc/Arc)
3. **Borrowing** - Track dynamic borrows (RefCell)
4. **Dropping** - Track deallocation

---

## Key Takeaways

✅ **Box = heap allocation** - Track like owned values  
✅ **Rc/Arc = shared ownership** - Track clones as references  
✅ **RefCell = runtime borrows** - Track dynamic borrow operations  
✅ **Deref coercion** - Accept some operations are invisible  
✅ **Special handling needed** - Detect smart pointer patterns  

---

## Further Reading

- [Rust Book - Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Box documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html)
- [Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [RefCell documentation](https://doc.rust-lang.org/std/cell/struct.RefCell.html)

---

**Previous:** [52-lifetime-tracking-challenges.md](./52-lifetime-tracking-challenges.md)  
**Next:** [54-tracking-box-allocations.md](./54-tracking-box-allocations.md)

**Progress:** 3/15 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
