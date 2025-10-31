# Section 56: Tracking RefCell and Cell

## Learning Objectives

By the end of this section, you will:
- Understand interior mutability patterns
- Track RefCell::borrow() and borrow_mut() calls
- Detect runtime borrow violations
- Handle Cell::get() and set() operations
- Visualize dynamic borrow checking

## Prerequisites

- Completed Section 55 (Rc/Arc Tracking)
- Understanding of interior mutability
- Familiarity with RefCell and Cell

---

## Interior Mutability

RefCell allows mutation through immutable references:

```rust
use std::cell::RefCell;

let x = RefCell::new(42);
*x.borrow_mut() = 100;  // Mutate through immutable x
```

**Key insight:** Borrow checking happens at runtime, not compile time.

---

## RefCell Operations

### borrow() - Immutable Borrow

```rust
let x = RefCell::new(42);
let r1 = x.borrow();  // Returns Ref<i32>
let r2 = x.borrow();  // OK: multiple immutable borrows
```

### borrow_mut() - Mutable Borrow

```rust
let x = RefCell::new(42);
let m = x.borrow_mut();  // Returns RefMut<i32>
// Can't borrow again until m is dropped
```

### Runtime Panic

```rust
let x = RefCell::new(42);
let r = x.borrow();
let m = x.borrow_mut();  // PANIC: already borrowed
```

---

## Detection Strategy

```rust
impl OwnershipVisitor {
    fn detect_refcell_op(&self, expr: &Expr) -> Option<RefCellOp> {
        if let Expr::MethodCall(method) = expr {
            let method_name = method.method.to_string();
            
            // Check if receiver is RefCell
            if self.is_refcell_type(&method.receiver) {
                match method_name.as_str() {
                    "borrow" => return Some(RefCellOp::Borrow),
                    "borrow_mut" => return Some(RefCellOp::BorrowMut),
                    _ => {}
                }
            }
            
            // Check for Cell operations
            if self.is_cell_type(&method.receiver) {
                match method_name.as_str() {
                    "get" => return Some(RefCellOp::CellGet),
                    "set" => return Some(RefCellOp::CellSet),
                    _ => {}
                }
            }
        }
        None
    }
    
    fn is_refcell_type(&self, expr: &Expr) -> bool {
        // Heuristic: check if variable name or type contains RefCell
        if let Expr::Path(path) = expr {
            if let Some(ident) = path.path.get_ident() {
                let var_name = ident.to_string();
                // Could check type annotations or maintain type info
                return true; // Simplified
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
enum RefCellOp {
    Borrow,
    BorrowMut,
    CellGet,
    CellSet,
}
```

---

## Runtime Implementation

Add to `borrowscope-runtime/src/tracker.rs`:

```rust
use std::cell::{RefCell, Ref, RefMut, Cell};

/// Track RefCell::borrow()
#[inline(always)]
pub fn track_refcell_borrow<'a, T>(
    borrow_id: usize,
    refcell_id: usize,
    location: &str,
    value: Ref<'a, T>
) -> Ref<'a, T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RefCellBorrow {
        borrow_id,
        refcell_id,
        is_mutable: false,
        location: location.to_string(),
        timestamp,
    });
    
    value
}

/// Track RefCell::borrow_mut()
#[inline(always)]
pub fn track_refcell_borrow_mut<'a, T>(
    borrow_id: usize,
    refcell_id: usize,
    location: &str,
    value: RefMut<'a, T>
) -> RefMut<'a, T> {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RefCellBorrow {
        borrow_id,
        refcell_id,
        is_mutable: true,
        location: location.to_string(),
        timestamp,
    });
    
    value
}

/// Track RefCell drop (when Ref/RefMut is dropped)
#[inline(always)]
pub fn track_refcell_drop(borrow_id: usize, location: &str) {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::RefCellDrop {
        borrow_id,
        location: location.to_string(),
        timestamp,
    });
}

/// Track Cell::get()
#[inline(always)]
pub fn track_cell_get<T: Copy>(
    cell_id: usize,
    location: &str,
    value: T
) -> T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::CellGet {
        cell_id,
        location: location.to_string(),
        timestamp,
    });
    
    value
}

/// Track Cell::set()
#[inline(always)]
pub fn track_cell_set<T>(
    cell_id: usize,
    location: &str,
) {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::CellSet {
        cell_id,
        location: location.to_string(),
        timestamp,
    });
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
    
    RefCellBorrow {
        borrow_id: usize,
        refcell_id: usize,
        is_mutable: bool,
        location: String,
        timestamp: u64,
    },
    
    RefCellDrop {
        borrow_id: usize,
        location: String,
        timestamp: u64,
    },
    
    CellGet {
        cell_id: usize,
        location: String,
        timestamp: u64,
    },
    
    CellSet {
        cell_id: usize,
        location: String,
        timestamp: u64,
    },
}
```

---

## Macro Transformation

### RefCell::borrow()

**Input:**
```rust
let x = RefCell::new(42);
let r = x.borrow();
```

**Output:**
```rust
let x = track_new(1, "x", "RefCell<i32>", "line:1:9", RefCell::new(42));
let r = track_new(
    2,
    "r",
    "Ref<i32>",
    "line:2:9",
    track_refcell_borrow(3, 1, "line:2:13", x.borrow())
);
```

### Implementation

```rust
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            // Check for RefCell operations
            if let Some(op) = self.detect_refcell_op(&init.expr) {
                match op {
                    RefCellOp::Borrow => {
                        self.transform_refcell_borrow(local);
                        return;
                    }
                    RefCellOp::BorrowMut => {
                        self.transform_refcell_borrow_mut(local);
                        return;
                    }
                    _ => {}
                }
            }
            
            // Regular transformation
            self.transform_regular_local(local);
        }
    }
    
    fn transform_refcell_borrow(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let var_id = self.next_id();
            let borrow_id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            // Extract RefCell variable ID
            let refcell_id = self.extract_refcell_id(&init.expr);
            
            self.var_ids.insert(var_name.clone(), var_id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(var_id);
            }
            
            let original_expr = &init.expr;
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #var_id,
                    #var_name,
                    "Ref<T>",
                    #location,
                    borrowscope_runtime::track_refcell_borrow(
                        #borrow_id,
                        #refcell_id,
                        #location,
                        #original_expr
                    )
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn transform_refcell_borrow_mut(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let var_id = self.next_id();
            let borrow_id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            let refcell_id = self.extract_refcell_id(&init.expr);
            
            self.var_ids.insert(var_name.clone(), var_id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(var_id);
            }
            
            let original_expr = &init.expr;
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #var_id,
                    #var_name,
                    "RefMut<T>",
                    #location,
                    borrowscope_runtime::track_refcell_borrow_mut(
                        #borrow_id,
                        #refcell_id,
                        #location,
                        #original_expr
                    )
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn extract_refcell_id(&self, expr: &Expr) -> usize {
        // Extract variable from x.borrow()
        if let Expr::MethodCall(method) = expr {
            if let Expr::Path(path) = &*method.receiver {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    return *self.var_ids.get(&var_name).unwrap_or(&0);
                }
            }
        }
        0
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/refcell_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_refcell_borrow() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = RefCell::new(42);
            let r = x.borrow();
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_refcell_borrow"));
}

#[test]
fn test_refcell_borrow_mut() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = RefCell::new(42);
            let m = x.borrow_mut();
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_refcell_borrow_mut"));
}

#[test]
fn test_multiple_borrows() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = RefCell::new(42);
            let r1 = x.borrow();
            let r2 = x.borrow();
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert_eq!(output.matches("track_refcell_borrow").count(), 2);
}
```

---

## Integration Test

Create `borrowscope-runtime/tests/refcell_integration.rs`:

```rust
use borrowscope_runtime::*;
use std::cell::RefCell;

#[test]
fn test_refcell_immutable_borrows() {
    reset_tracker();
    
    let x = track_new(1, "x", "RefCell<i32>", "test.rs:1:1", RefCell::new(42));
    
    let r1 = track_new(
        2,
        "r1",
        "Ref<i32>",
        "test.rs:2:1",
        track_refcell_borrow(3, 1, "test.rs:2:14", x.borrow())
    );
    
    let r2 = track_new(
        4,
        "r2",
        "Ref<i32>",
        "test.rs:3:1",
        track_refcell_borrow(5, 1, "test.rs:3:14", x.borrow())
    );
    
    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);
    
    drop(r1);
    track_refcell_drop(3, "test.rs:5:1");
    track_drop(2, "test.rs:5:1");
    
    drop(r2);
    track_refcell_drop(5, "test.rs:6:1");
    track_drop(4, "test.rs:6:1");
    
    let events = get_events();
    
    // Should have: New(x), New(r1), RefCellBorrow, New(r2), RefCellBorrow, drops
    assert!(events.len() >= 5);
}

#[test]
fn test_refcell_mutable_borrow() {
    reset_tracker();
    
    let x = track_new(1, "x", "RefCell<i32>", "test.rs:1:1", RefCell::new(42));
    
    let mut m = track_new(
        2,
        "m",
        "RefMut<i32>",
        "test.rs:2:1",
        track_refcell_borrow_mut(3, 1, "test.rs:2:18", x.borrow_mut())
    );
    
    *m = 100;
    assert_eq!(*m, 100);
    
    drop(m);
    track_refcell_drop(3, "test.rs:5:1");
    track_drop(2, "test.rs:5:1");
    
    let events = get_events();
    
    match &events[2] {
        Event::RefCellBorrow { is_mutable, .. } => {
            assert!(*is_mutable);
        }
        _ => panic!("Expected RefCellBorrow event"),
    }
}

#[test]
#[should_panic(expected = "already borrowed")]
fn test_refcell_panic() {
    let x = RefCell::new(42);
    let r = x.borrow();
    let m = x.borrow_mut();  // Should panic
}
```

---

## Borrow Violation Detection

Add analysis to detect potential violations:

```rust
// borrowscope-runtime/src/graph.rs
impl OwnershipGraph {
    pub fn detect_refcell_violations(&self) -> Vec<BorrowViolation> {
        let mut violations = Vec::new();
        let mut active_borrows: HashMap<usize, Vec<(usize, bool, u64)>> = HashMap::new();
        
        for event in &self.events {
            match event {
                Event::RefCellBorrow { borrow_id, refcell_id, is_mutable, timestamp, .. } => {
                    let borrows = active_borrows.entry(*refcell_id).or_insert_with(Vec::new);
                    
                    // Check for violations
                    if *is_mutable && !borrows.is_empty() {
                        violations.push(BorrowViolation {
                            refcell_id: *refcell_id,
                            violation_type: ViolationType::MutableWhileBorrowed,
                            timestamp: *timestamp,
                        });
                    }
                    
                    if !is_mutable {
                        for (_, is_mut, _) in borrows.iter() {
                            if *is_mut {
                                violations.push(BorrowViolation {
                                    refcell_id: *refcell_id,
                                    violation_type: ViolationType::ImmutableWhileMutablyBorrowed,
                                    timestamp: *timestamp,
                                });
                            }
                        }
                    }
                    
                    borrows.push((*borrow_id, *is_mutable, *timestamp));
                }
                
                Event::RefCellDrop { borrow_id, .. } => {
                    // Remove from active borrows
                    for borrows in active_borrows.values_mut() {
                        borrows.retain(|(id, _, _)| id != borrow_id);
                    }
                }
                
                _ => {}
            }
        }
        
        violations
    }
}

#[derive(Debug, Clone)]
pub struct BorrowViolation {
    pub refcell_id: usize,
    pub violation_type: ViolationType,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    MutableWhileBorrowed,
    ImmutableWhileMutablyBorrowed,
}
```

---

## Visualization

Show RefCell borrows with special indicators:

```json
{
  "refcell_borrows": [
    {
      "refcell_id": 1,
      "borrows": [
        {
          "borrow_id": 3,
          "is_mutable": false,
          "start_time": 1001,
          "end_time": 1005
        },
        {
          "borrow_id": 5,
          "is_mutable": false,
          "start_time": 1002,
          "end_time": 1006
        }
      ]
    }
  ]
}
```

---

## Key Takeaways

✅ **Interior mutability** - Mutation through immutable references  
✅ **Runtime borrow checking** - Track borrow() and borrow_mut()  
✅ **Detect violations** - Analyze events for borrow conflicts  
✅ **Ref/RefMut lifetimes** - Track when borrows are active  
✅ **Visualize dynamic borrows** - Show runtime borrow state  

---

## Further Reading

- [RefCell documentation](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
- [Interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [Cell documentation](https://doc.rust-lang.org/std/cell/struct.Cell.html)

---

**Previous:** [55-tracking-rc-and-arc.md](./55-tracking-rc-and-arc.md)  
**Next:** [57-chapter-summary.md](./57-chapter-summary.md)

**Progress:** 6/15 ⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜
