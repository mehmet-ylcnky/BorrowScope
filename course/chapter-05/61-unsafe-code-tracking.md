# Section 61: Unsafe Code Tracking

## Learning Objectives

By the end of this section, you will:
- Understand unsafe operations
- Track raw pointer operations
- Handle unsafe blocks
- Recognize tracking limitations
- Document unsafe assumptions

## Prerequisites

- Completed Section 60 (Const/Static)
- Understanding of unsafe Rust
- Familiarity with raw pointers

---

## Unsafe Operations

Rust's five unsafe superpowers:

1. Dereference raw pointers
2. Call unsafe functions
3. Access/modify mutable statics
4. Implement unsafe traits
5. Access union fields

---

## Raw Pointers

```rust
let x = 42;
let ptr: *const i32 = &x;

unsafe {
    let value = *ptr;  // Dereference raw pointer
}
```

**Tracking challenge:** Raw pointers bypass borrow checker.

---

## Detection

```rust
impl OwnershipVisitor {
    fn detect_unsafe_block(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Unsafe(_))
    }
    
    fn detect_raw_pointer(&self, ty: &Type) -> bool {
        if let Type::Ptr(_) = ty {
            return true;
        }
        false
    }
}
```

---

## Tracking Strategy

### Track Pointer Creation

```rust
let x = 42;
let ptr = &x as *const i32;
```

**Track:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", 42);
let ptr = track_new(
    2,
    "ptr",
    "*const i32",
    "line:2:9",
    &x as *const i32
);
```

### Track Dereference (Best Effort)

```rust
unsafe {
    let value = *ptr;
}
```

**Track:**
```rust
unsafe {
    let value = track_new(
        3,
        "value",
        "i32",
        "line:5:13",
        *ptr
    );
}
```

**Limitation:** We can't verify the pointer is valid.

---

## Implementation

```rust
impl OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Unsafe(unsafe_expr) => {
                self.transform_unsafe_block(unsafe_expr);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
    
    fn transform_unsafe_block(&mut self, unsafe_expr: &mut syn::ExprUnsafe) {
        // Visit statements inside unsafe block
        self.visit_block_mut(&mut unsafe_expr.block);
        
        // Could add warning annotation
        let warning = "// WARNING: Unsafe block - tracking may be incomplete";
        // Add as comment in generated code
    }
}
```

---

## Runtime Support

```rust
/// Track raw pointer creation
#[inline(always)]
pub fn track_raw_ptr<T>(
    id: usize,
    name: &str,
    location: &str,
    ptr: *const T
) -> *const T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RawPtrCreated {
        id,
        name: name.to_string(),
        location: location.to_string(),
        timestamp,
        address: ptr as usize,
    });
    
    ptr
}

/// Track raw pointer dereference
#[inline(always)]
pub fn track_raw_ptr_deref(
    ptr_id: usize,
    location: &str,
) {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RawPtrDeref {
        ptr_id,
        location: location.to_string(),
        timestamp,
    });
}
```

---

## Event Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    // ... existing variants
    
    RawPtrCreated {
        id: usize,
        name: String,
        location: String,
        timestamp: u64,
        address: usize,
    },
    
    RawPtrDeref {
        ptr_id: usize,
        location: String,
        timestamp: u64,
    },
    
    UnsafeBlockEnter {
        location: String,
        timestamp: u64,
    },
    
    UnsafeBlockExit {
        location: String,
        timestamp: u64,
    },
}
```

---

## Testing

```rust
#[test]
fn test_raw_pointer_tracking() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let ptr = track_raw_ptr(2, "ptr", "test.rs:2:1", &x as *const i32);
    
    unsafe {
        track_raw_ptr_deref(2, "test.rs:4:1");
        let value = *ptr;
        assert_eq!(value, 42);
    }
    
    let events = get_events();
    assert!(events.len() >= 3);
}
```

---

## Unsafe Function Calls

```rust
unsafe fn dangerous() {
    // Unsafe operations
}

unsafe {
    dangerous();
}
```

**Tracking:** Track the call site, not inside the function (unless it's also annotated).

---

## FFI (Foreign Function Interface)

```rust
extern "C" {
    fn external_function(x: i32) -> i32;
}

unsafe {
    let result = external_function(42);
}
```

**Tracking:** Track the call, but can't track inside external code.

```rust
unsafe {
    track_ffi_call("external_function", "test.rs:5:1");
    let result = external_function(42);
}
```

---

## Transmute

```rust
unsafe {
    let x: i32 = 42;
    let y: f32 = std::mem::transmute(x);
}
```

**Tracking:** Track as a transformation:

```rust
unsafe {
    let x = track_new(1, "x", "i32", "line:2:13", 42);
    track_transmute(1, 2, "i32", "f32", "line:3:13");
    let y: f32 = std::mem::transmute(x);
}
```

---

## Limitations

### Can't Verify Safety

```rust
unsafe {
    let ptr = 0x1234 as *const i32;  // Arbitrary address
    let value = *ptr;  // May crash!
}
```

**Tracking:** We record the operation, but can't prevent crashes.

### Can't Track Aliasing

```rust
unsafe {
    let mut x = 42;
    let ptr1 = &mut x as *mut i32;
    let ptr2 = &mut x as *mut i32;  // Aliasing!
    *ptr1 = 100;
    *ptr2 = 200;
}
```

**Tracking:** We see two pointers, but can't detect they alias.

---

## Best Practices

### Document Unsafe Assumptions

```rust
/// SAFETY: ptr must be valid and aligned
unsafe fn read_ptr(ptr: *const i32) -> i32 {
    *ptr
}
```

### Minimize Unsafe Scope

```rust
// Good: Small unsafe block
let value = unsafe { *ptr };

// Bad: Large unsafe block
unsafe {
    // Many operations
}
```

### Use Safe Abstractions

```rust
// Instead of raw pointers
let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

// Use safe slice operations
for item in slice {
    println!("{}", item);
}
```

---

## Visualization

Show unsafe operations with warnings:

```json
{
  "events": [
    {
      "type": "RawPtrCreated",
      "data": {
        "id": 1,
        "name": "ptr",
        "address": "0x7fff5fbff8ac",
        "warning": "Unsafe: Raw pointer created"
      }
    },
    {
      "type": "RawPtrDeref",
      "data": {
        "ptr_id": 1,
        "warning": "Unsafe: Raw pointer dereferenced"
      }
    }
  ]
}
```

---

## Key Takeaways

✅ **Unsafe bypasses borrow checker** - Tracking is best-effort  
✅ **Track pointer creation** - Record address and type  
✅ **Track dereferences** - Note when pointers are used  
✅ **Can't verify safety** - Compiler can't help in unsafe  
✅ **Document assumptions** - Critical for unsafe code  
✅ **Minimize unsafe scope** - Keep it small and isolated  

---

## Further Reading

- [Unsafe Rust](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
- [Nomicon](https://doc.rust-lang.org/nomicon/)
- [Raw pointers](https://doc.rust-lang.org/std/primitive.pointer.html)
- [FFI](https://doc.rust-lang.org/nomicon/ffi.html)

---

**Previous:** [60-const-and-static-variables.md](./60-const-and-static-variables.md)  
**Next:** [62-macro-generated-code.md](./62-macro-generated-code.md)

**Progress:** 11/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜
