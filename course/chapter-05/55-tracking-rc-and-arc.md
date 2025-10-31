# Section 55: Tracking Rc and Arc

## Learning Objectives

By the end of this section, you will:
- Detect Rc::new and Arc::new calls
- Track Rc::clone and Arc::clone operations
- Understand reference counting semantics
- Implement shared ownership tracking
- Visualize reference count changes

## Prerequisites

- Completed Section 54 (Box Tracking)
- Understanding of reference counting
- Familiarity with Rc and Arc

---

## Reference Counting Basics

```rust
use std::rc::Rc;

let x = Rc::new(42);        // ref count = 1
let y = Rc::clone(&x);      // ref count = 2
let z = Rc::clone(&x);      // ref count = 3
drop(y);                    // ref count = 2
drop(z);                    // ref count = 1
drop(x);                    // ref count = 0, data deallocated
```

---

## Detection Strategy

### Detect Rc::new / Arc::new

```rust
impl OwnershipVisitor {
    fn detect_smart_pointer(&self, expr: &Expr) -> Option<SmartPointerOp> {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                
                if path_str.contains("Rc") && path_str.contains("new") {
                    return Some(SmartPointerOp::RcNew);
                }
                if path_str.contains("Arc") && path_str.contains("new") {
                    return Some(SmartPointerOp::ArcNew);
                }
                if path_str.contains("Rc") && path_str.contains("clone") {
                    return Some(SmartPointerOp::RcClone);
                }
                if path_str.contains("Arc") && path_str.contains("clone") {
                    return Some(SmartPointerOp::ArcClone);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum SmartPointerOp {
    RcNew,
    ArcNew,
    RcClone,
    ArcClone,
}
```

---

## Transformation Strategy

### Rc::new

**Input:**
```rust
let x = Rc::new(42);
```

**Output:**
```rust
let x = track_rc_new(1, "x", "Rc<i32>", "line:1:9", Rc::new(42));
```

### Rc::clone

**Input:**
```rust
let y = Rc::clone(&x);
```

**Output:**
```rust
let y = track_rc_clone(2, 1, "y", "Rc<i32>", "line:2:9", Rc::clone(&x));
```

---

## Runtime Implementation

Add to `borrowscope-runtime/src/tracker.rs`:

```rust
/// Track Rc::new allocation
#[inline(always)]
pub fn track_rc_new<T>(
    id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: std::rc::Rc<T>
) -> std::rc::Rc<T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RcNew {
        id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
        ref_count: 1,
    });
    
    value
}

/// Track Rc::clone operation
#[inline(always)]
pub fn track_rc_clone<T>(
    new_id: usize,
    source_id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: std::rc::Rc<T>
) -> std::rc::Rc<T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    let ref_count = std::rc::Rc::strong_count(&value);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RcClone {
        new_id,
        source_id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
        ref_count,
    });
    
    value
}

/// Track Arc::new allocation
#[inline(always)]
pub fn track_arc_new<T>(
    id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: std::sync::Arc<T>
) -> std::sync::Arc<T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::ArcNew {
        id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
        ref_count: 1,
    });
    
    value
}

/// Track Arc::clone operation
#[inline(always)]
pub fn track_arc_clone<T>(
    new_id: usize,
    source_id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: std::sync::Arc<T>
) -> std::sync::Arc<T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    let ref_count = std::sync::Arc::strong_count(&value);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::ArcClone {
        new_id,
        source_id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
        ref_count,
    });
    
    value
}
```

---

## Event Types

Add to `borrowscope-runtime/src/event.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    // ... existing variants
    
    RcNew {
        id: usize,
        name: String,
        type_name: String,
        location: String,
        timestamp: u64,
        ref_count: usize,
    },
    
    RcClone {
        new_id: usize,
        source_id: usize,
        name: String,
        type_name: String,
        location: String,
        timestamp: u64,
        ref_count: usize,
    },
    
    ArcNew {
        id: usize,
        name: String,
        type_name: String,
        location: String,
        timestamp: u64,
        ref_count: usize,
    },
    
    ArcClone {
        new_id: usize,
        source_id: usize,
        name: String,
        type_name: String,
        location: String,
        timestamp: u64,
        ref_count: usize,
    },
}
```

---

## Macro Implementation

Add to `borrowscope-macro/src/visitor.rs`:

```rust
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            // Detect smart pointer operations
            match self.detect_smart_pointer(&init.expr) {
                Some(SmartPointerOp::RcNew) => {
                    self.transform_rc_new(local, id, var_name, location);
                    return;
                }
                Some(SmartPointerOp::ArcNew) => {
                    self.transform_arc_new(local, id, var_name, location);
                    return;
                }
                Some(SmartPointerOp::RcClone) => {
                    self.transform_rc_clone(local, id, var_name, location);
                    return;
                }
                Some(SmartPointerOp::ArcClone) => {
                    self.transform_arc_clone(local, id, var_name, location);
                    return;
                }
                None => {
                    // Regular transformation
                    self.transform_regular_local(local, id, var_name, location);
                }
            }
        }
    }
    
    fn transform_rc_new(&mut self, local: &mut Local, id: usize, var_name: String, location: String) {
        if let Some(init) = &mut local.init {
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            let type_name = "Rc<T>"; // Could be improved
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_rc_new(
                    #id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn transform_rc_clone(&mut self, local: &mut Local, id: usize, var_name: String, location: String) {
        if let Some(init) = &mut local.init {
            // Extract source variable from Rc::clone(&x)
            let source_id = self.extract_rc_clone_source(&init.expr);
            
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            let type_name = "Rc<T>";
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_rc_clone(
                    #id,
                    #source_id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn extract_rc_clone_source(&self, expr: &Expr) -> usize {
        // Extract variable from Rc::clone(&x)
        if let Expr::Call(call) = expr {
            if let Some(arg) = call.args.first() {
                if let Expr::Reference(ref_expr) = arg {
                    if let Expr::Path(path) = &*ref_expr.expr {
                        if let Some(ident) = path.path.get_ident() {
                            let var_name = ident.to_string();
                            return *self.var_ids.get(&var_name).unwrap_or(&0);
                        }
                    }
                }
            }
        }
        0
    }
    
    fn transform_arc_new(&mut self, local: &mut Local, id: usize, var_name: String, location: String) {
        // Similar to transform_rc_new but with track_arc_new
        if let Some(init) = &mut local.init {
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            let type_name = "Arc<T>";
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_arc_new(
                    #id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn transform_arc_clone(&mut self, local: &mut Local, id: usize, var_name: String, location: String) {
        // Similar to transform_rc_clone but with track_arc_clone
        if let Some(init) = &mut local.init {
            let source_id = self.extract_rc_clone_source(&init.expr);
            
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            let type_name = "Arc<T>";
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_arc_clone(
                    #id,
                    #source_id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/rc_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_rc_new() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Rc::new(42);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_rc_new"));
}

#[test]
fn test_rc_clone() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Rc::new(42);
            let y = Rc::clone(&x);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_rc_new"));
    assert!(output.contains("track_rc_clone"));
}

#[test]
fn test_multiple_clones() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = Rc::new(42);
            let y = Rc::clone(&x);
            let z = Rc::clone(&x);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert_eq!(output.matches("track_rc_clone").count(), 2);
}
```

---

## Integration Test

Create `borrowscope-runtime/tests/rc_integration.rs`:

```rust
use borrowscope_runtime::*;
use std::rc::Rc;

#[test]
fn test_rc_lifecycle() {
    reset_tracker();
    
    let x = track_rc_new(1, "x", "Rc<i32>", "test.rs:1:1", Rc::new(42));
    assert_eq!(*x, 42);
    
    let y = track_rc_clone(2, 1, "y", "Rc<i32>", "test.rs:2:1", Rc::clone(&x));
    assert_eq!(*y, 42);
    
    let z = track_rc_clone(3, 1, "z", "Rc<i32>", "test.rs:3:1", Rc::clone(&x));
    assert_eq!(*z, 42);
    
    track_drop(3, "scope_end");
    track_drop(2, "scope_end");
    track_drop(1, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 6);
    
    // Verify ref counts
    match &events[0] {
        Event::RcNew { ref_count, .. } => assert_eq!(*ref_count, 1),
        _ => panic!("Expected RcNew"),
    }
    
    match &events[1] {
        Event::RcClone { ref_count, .. } => assert_eq!(*ref_count, 2),
        _ => panic!("Expected RcClone"),
    }
    
    match &events[2] {
        Event::RcClone { ref_count, .. } => assert_eq!(*ref_count, 3),
        _ => panic!("Expected RcClone"),
    }
}

#[test]
fn test_arc_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    reset_tracker();
    
    let x = track_arc_new(1, "x", "Arc<i32>", "test.rs:1:1", Arc::new(42));
    let x_clone = track_arc_clone(2, 1, "x_clone", "Arc<i32>", "test.rs:2:1", Arc::clone(&x));
    
    let handle = thread::spawn(move || {
        assert_eq!(*x_clone, 42);
    });
    
    handle.join().unwrap();
    
    let events = get_events();
    assert!(events.len() >= 2);
}
```

---

## Graph Representation

Add to `borrowscope-runtime/src/graph.rs`:

```rust
impl OwnershipGraph {
    pub fn from_events(events: &[Event]) -> Self {
        let mut graph = DiGraph::new();
        let mut id_to_node = HashMap::new();
        
        for event in events {
            match event {
                Event::RcNew { id, name, type_name, timestamp, .. } => {
                    let var = Variable {
                        id: *id,
                        name: name.clone(),
                        type_name: type_name.clone(),
                        created_at: *timestamp,
                        dropped_at: None,
                    };
                    let node_idx = graph.add_node(var);
                    id_to_node.insert(*id, node_idx);
                }
                
                Event::RcClone { new_id, source_id, name, type_name, timestamp, .. } => {
                    // Add new node for clone
                    let var = Variable {
                        id: *new_id,
                        name: name.clone(),
                        type_name: type_name.clone(),
                        created_at: *timestamp,
                        dropped_at: None,
                    };
                    let new_node_idx = graph.add_node(var);
                    id_to_node.insert(*new_id, new_node_idx);
                    
                    // Add edge showing shared ownership
                    if let Some(&source_node) = id_to_node.get(source_id) {
                        graph.add_edge(new_node_idx, source_node, Relationship::SharesOwnership);
                    }
                }
                
                // Similar for ArcNew and ArcClone
                _ => {}
            }
        }
        
        Self { graph, id_to_node }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    Owns,
    BorrowsImmut,
    BorrowsMut,
    SharesOwnership,  // New: for Rc/Arc
}
```

---

## Visualization

Export format for UI:

```json
{
  "nodes": [
    {
      "id": 1,
      "name": "x",
      "type": "Rc<i32>",
      "ref_count": 3,
      "created_at": 1000
    },
    {
      "id": 2,
      "name": "y",
      "type": "Rc<i32>",
      "shares_with": 1,
      "created_at": 1001
    }
  ],
  "edges": [
    {
      "from": 2,
      "to": 1,
      "type": "shares_ownership"
    }
  ]
}
```

---

## Key Takeaways

✅ **Detect Rc/Arc operations** - Pattern match on call expressions  
✅ **Track clones separately** - Each clone gets unique ID  
✅ **Record ref counts** - Use strong_count() at runtime  
✅ **Shared ownership edges** - New relationship type in graph  
✅ **Thread safety** - Arc works across threads  

---

## Further Reading

- [Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Reference counting](https://en.wikipedia.org/wiki/Reference_counting)

---

**Previous:** [54-tracking-box-allocations.md](./54-tracking-box-allocations.md)  
**Next:** [56-tracking-refcell-and-cell.md](./56-tracking-refcell-and-cell.md)

**Progress:** 5/15 ⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
