# Section 60: Const and Static Variables

## Learning Objectives

By the end of this section, you will:
- Understand const vs static
- Recognize when to track static variables
- Handle const evaluation
- Track static initialization
- Deal with static lifetimes

## Prerequisites

- Completed Section 59 (Trait Objects)
- Understanding of const and static
- Familiarity with compile-time evaluation

---

## Const vs Static

### Const

```rust
const MAX_SIZE: usize = 100;
```

**Properties:**
- Inlined at compile time
- No memory address
- Can be used in const contexts

**Tracking:** Don't track - no runtime existence.

### Static

```rust
static GLOBAL: i32 = 42;
```

**Properties:**
- Has fixed memory address
- Lives for entire program ('static lifetime)
- Initialized once

**Tracking:** Can track, but usually not useful.

---

## Static Mut

```rust
static mut COUNTER: i32 = 0;

unsafe {
    COUNTER += 1;
}
```

**Properties:**
- Mutable global state
- Requires unsafe to access
- Can cause data races

**Tracking:** Track mutations in unsafe blocks.

---

## Detection

```rust
impl OwnershipVisitor {
    fn visit_item_mut(&mut self, item: &mut Item) {
        match item {
            Item::Static(static_item) => {
                // Found a static variable
                if static_item.mutability.is_some() {
                    // It's static mut
                    self.handle_static_mut(static_item);
                }
            }
            Item::Const(_) => {
                // Const - don't track
            }
            _ => {
                visit_mut::visit_item_mut(self, item);
            }
        }
    }
}
```

---

## Tracking Strategy

### Don't Track Const

```rust
const PI: f64 = 3.14159;
let x = PI;  // PI is inlined, no tracking needed
```

### Optionally Track Static

```rust
static GLOBAL: i32 = 42;

fn example() {
    let x = GLOBAL;  // Could track as "read from static"
}
```

**Decision:** For simplicity, don't track static reads.

### Track Static Mut

```rust
static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;  // Track this mutation
    }
}
```

---

## Implementation

```rust
impl OwnershipVisitor {
    fn handle_static_mut(&mut self, static_item: &mut ItemStatic) {
        // Add tracking to static mut initialization
        if let Some(init_expr) = &mut static_item.expr {
            let static_name = static_item.ident.to_string();
            let id = self.next_id();
            
            let original_expr = init_expr.as_ref();
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_static_init(
                    #id,
                    #static_name,
                    "static mut",
                    "static_init",
                    #original_expr
                )
            };
            
            *init_expr = Box::new(new_expr);
        }
    }
}
```

---

## Runtime Support

```rust
// borrowscope-runtime/src/tracker.rs

/// Track static variable initialization
#[inline(always)]
pub fn track_static_init<T>(
    id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: T
) -> T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::StaticInit {
        id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
    });
    
    value
}

/// Track static mut access
#[inline(always)]
pub fn track_static_access(
    id: usize,
    name: &str,
    is_write: bool,
    location: &str,
) {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::StaticAccess {
        id,
        name: name.to_string(),
        is_write,
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
    
    StaticInit {
        id: usize,
        name: String,
        type_name: String,
        location: String,
        timestamp: u64,
    },
    
    StaticAccess {
        id: usize,
        name: String,
        is_write: bool,
        location: String,
        timestamp: u64,
    },
}
```

---

## Testing

```rust
#[test]
fn test_static_tracking() {
    static GLOBAL: i32 = 42;
    
    reset_tracker();
    
    let x = GLOBAL;  // Read from static
    assert_eq!(x, 42);
    
    // No events - we don't track static reads
    let events = get_events();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_static_mut_tracking() {
    static mut COUNTER: i32 = 0;
    
    reset_tracker();
    
    unsafe {
        track_static_access(1, "COUNTER", true, "test.rs:1:1");
        COUNTER += 1;
        
        track_static_access(1, "COUNTER", false, "test.rs:2:1");
        let x = COUNTER;
        assert_eq!(x, 1);
    }
    
    let events = get_events();
    assert_eq!(events.len(), 2);
}
```

---

## Lazy Static

For lazy_static! macro:

```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL: String = String::from("hello");
}
```

**Tracking:** Treat like regular static, but initialization happens on first access.

---

## Thread-Local Storage

```rust
use std::thread_local;

thread_local! {
    static COUNTER: RefCell<i32> = RefCell::new(0);
}

COUNTER.with(|c| {
    *c.borrow_mut() += 1;
});
```

**Tracking:** Track the RefCell operations, not the thread_local itself.

---

## Key Takeaways

✅ **Const is compile-time** - Don't track  
✅ **Static has 'static lifetime** - Lives forever  
✅ **Static mut requires unsafe** - Track mutations  
✅ **Lazy initialization** - Track on first access  
✅ **Thread-local** - Track inner operations  

---

## Further Reading

- [Const and static](https://doc.rust-lang.org/reference/items/constant-items.html)
- [Static items](https://doc.rust-lang.org/reference/items/static-items.html)
- [lazy_static](https://docs.rs/lazy_static/)
- [thread_local](https://doc.rust-lang.org/std/macro.thread_local.html)

---

**Previous:** [59-trait-objects-and-dynamic-dispatch.md](./59-trait-objects-and-dynamic-dispatch.md)  
**Next:** [61-unsafe-code-tracking.md](./61-unsafe-code-tracking.md)

**Progress:** 10/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜
