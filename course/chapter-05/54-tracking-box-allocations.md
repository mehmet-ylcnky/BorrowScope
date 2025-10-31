# Section 54: Tracking Box Allocations

## Learning Objectives

By the end of this section, you will:
- Detect Box::new calls in AST
- Transform Box allocations
- Track heap vs stack allocation
- Handle deref coercion
- Visualize Box ownership

## Prerequisites

- Completed Section 53 (Smart Pointers Overview)
- Understanding of heap allocation
- Familiarity with Deref trait

---

## Box Basics

`Box<T>` allocates data on the heap:

```rust
let x = Box::new(42);  // 42 is on the heap
let y = *x;            // Dereference to get value
```

**Ownership:** Box owns its data. When Box is dropped, data is deallocated.

---

## Detection Strategy

Detect `Box::new` calls:

```rust
impl OwnershipVisitor {
    fn is_box_new(&self, expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                return path_str.contains("Box") && path_str.contains("new");
            }
        }
        false
    }
}
```

---

## Transformation

### Simple Box

**Input:**
```rust
let x = Box::new(42);
```

**Output:**
```rust
let x = track_new(1, "x", "Box<i32>", "line:1:9", Box::new(42));
```

**No special handling needed!** Box is just an owned value.

### Box with Move

**Input:**
```rust
let x = Box::new(42);
let y = x;  // Move
```

**Output:**
```rust
let x = track_new(1, "x", "Box<i32>", "line:1:9", Box::new(42));
track_move(1, 2, "line:2:9");
let y = x;
```

---

## Deref Coercion

Box implements `Deref`, allowing automatic dereferencing:

```rust
let x = Box::new(42);
let r = &*x;  // Explicit deref
let s = &x;   // Implicit deref coercion in some contexts
```

### Explicit Deref

**Input:**
```rust
let x = Box::new(42);
let r = &*x;
```

**Output:**
```rust
let x = track_new(1, "x", "Box<i32>", "line:1:9", Box::new(42));
let r = track_borrow(2, 1, false, "line:2:10", &*x);
```

**Note:** We track the borrow of the dereferenced value.

---

## Implementation

Add to `borrowscope-macro/src/visitor.rs`:

```rust
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            // Detect Box::new
            let type_name = if self.is_box_new(&init.expr) {
                self.extract_box_type(&init.expr)
            } else {
                self.extract_type_name(&local.pat)
            };
            
            // Store variable ID
            self.var_ids.insert(var_name.clone(), id);
            
            // Add to current scope
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
        
        visit_mut::visit_local_mut(self, local);
    }
    
    fn is_box_new(&self, expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                return path_str.contains("Box") && path_str.contains("new");
            }
        }
        false
    }
    
    fn extract_box_type(&self, expr: &Expr) -> String {
        // Try to infer type from Box::new argument
        if let Expr::Call(call) = expr {
            if let Some(arg) = call.args.first() {
                // Simple heuristic: check argument type
                match arg {
                    Expr::Lit(lit) => {
                        match &lit.lit {
                            syn::Lit::Int(_) => return "Box<i32>".to_string(),
                            syn::Lit::Str(_) => return "Box<&str>".to_string(),
                            syn::Lit::Bool(_) => return "Box<bool>".to_string(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        "Box<T>".to_string()
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/box_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_box_new() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Box::new(42);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("Box :: new"));
}

#[test]
fn test_box_move() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Box::new(42);
            let y = x;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_move"));
}

#[test]
fn test_box_deref() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Box::new(42);
            let r = &*x;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_borrow"));
}
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/box_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_box_allocation() {
    reset_tracker();
    
    let x = track_new(1, "x", "Box<i32>", "test.rs:1:1", Box::new(42));
    
    assert_eq!(*x, 42);
    
    track_drop(1, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 2);
    
    match &events[0] {
        Event::New { name, type_name, .. } => {
            assert_eq!(name, "x");
            assert_eq!(type_name, "Box<i32>");
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_box_move() {
    reset_tracker();
    
    let x = track_new(1, "x", "Box<i32>", "test.rs:1:1", Box::new(42));
    track_move(1, 2, "test.rs:2:1");
    let y = x;
    
    assert_eq!(*y, 42);
    
    track_drop(2, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 3);  // New, Move, Drop
}

#[test]
fn test_box_deref_borrow() {
    reset_tracker();
    
    let x = track_new(1, "x", "Box<i32>", "test.rs:1:1", Box::new(42));
    let r = track_borrow(2, 1, false, "test.rs:2:1", &*x);
    
    assert_eq!(*r, 42);
    
    track_drop(2, "scope_end");
    track_drop(1, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 4);  // New, Borrow, Drop, Drop
}
```

---

## Visualization

For the UI, show Box allocations differently:

```json
{
  "nodes": [
    {
      "id": 1,
      "name": "x",
      "type": "Box<i32>",
      "allocation": "heap",
      "created_at": 1000,
      "dropped_at": 2000
    }
  ]
}
```

**Visual indicator:** Use a different color or icon for heap-allocated values.

---

## Advanced: Tracking Allocation Size

For more detailed tracking, we could track allocation size:

```rust
// Runtime function
pub fn track_box_new<T>(
    id: usize,
    name: &str,
    location: &str,
    value: Box<T>
) -> Box<T> {
    let size = std::mem::size_of::<T>();
    
    TRACKER.lock().events.push(Event::BoxAlloc {
        id,
        name: name.to_string(),
        location: location.to_string(),
        size,
        timestamp: GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst),
    });
    
    value
}
```

**Use case:** Show memory usage over time.

---

## Edge Cases

### Case 1: Box of Box

```rust
let x = Box::new(Box::new(42));
```

**Tracking:**
```rust
let x = track_new(1, "x", "Box<Box<i32>>", "line:1:9", Box::new(Box::new(42)));
```

**Works automatically!**

### Case 2: Box in Struct

```rust
struct Container {
    data: Box<i32>,
}

let c = Container { data: Box::new(42) };
```

**Tracking:**
```rust
let c = track_new(
    1,
    "c",
    "Container",
    "line:5:9",
    Container { data: Box::new(42) }
);
```

**The inner Box::new is not tracked separately.**

### Case 3: Box from Function

```rust
fn create_box() -> Box<i32> {
    Box::new(42)
}

let x = create_box();
```

**Tracking:**
```rust
let x = track_new(1, "x", "Box<i32>", "line:5:9", create_box());
```

**We track the assignment, not the allocation inside the function.**

---

## Key Takeaways

✅ **Box is an owned value** - Track like any other variable  
✅ **Detect Box::new** - Pattern match on call expressions  
✅ **Deref is transparent** - Track borrows of dereferenced values  
✅ **Heap allocation** - Can annotate in visualization  
✅ **No special runtime support needed** - Existing tracking works  

---

## Further Reading

- [Box documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html)
- [Deref coercion](https://doc.rust-lang.org/book/ch15-02-deref.html)
- [Heap vs stack](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#the-stack-and-the-heap)

---

**Previous:** [53-smart-pointers-overview.md](./53-smart-pointers-overview.md)  
**Next:** [55-tracking-rc-and-arc.md](./55-tracking-rc-and-arc.md)

**Progress:** 4/15 ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
