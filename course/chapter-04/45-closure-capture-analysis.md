# Section 45: Closure Capture Analysis

## Learning Objectives

By the end of this section, you will:
- Understand closure syntax in AST
- Detect closure capture modes
- Track captured variables
- Transform closure bodies
- Handle move closures

## Prerequisites

- Completed Section 44 (Method Calls)
- Understanding of closures and captures
- Familiarity with closure syntax

---

## Closure Basics

```rust
let x = 42;
let closure = |y| x + y;  // Captures x by reference
```

**Capture modes:**
- **By reference** - `|y| x + y` (default)
- **By mutable reference** - `|y| { x += y; }`
- **By move** - `move |y| x + y`

---

## Closure AST

```rust
ExprClosure {
    movability: Option<Token![static]>,
    asyncness: Option<Token![async]>,
    capture: Option<Token![move]>,
    inputs: Punctuated<Pat, Token![,]>,
    output: ReturnType,
    body: Box<Expr>,
}
```

---

## Detection Strategy

```rust
impl OwnershipVisitor {
    fn detect_closure(&self, expr: &Expr) -> Option<&syn::ExprClosure> {
        if let Expr::Closure(closure) = expr {
            Some(closure)
        } else {
            None
        }
    }
    
    fn is_move_closure(&self, closure: &syn::ExprClosure) -> bool {
        closure.capture.is_some()
    }
    
    fn extract_captured_vars(&self, closure_body: &Expr) -> Vec<String> {
        let mut captured = Vec::new();
        self.find_variables_in_expr(closure_body, &mut captured);
        captured
    }
    
    fn find_variables_in_expr(&self, expr: &Expr, vars: &mut Vec<String>) {
        match expr {
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    // Check if it's a captured variable (not a parameter)
                    if self.var_ids.contains_key(&var_name) {
                        vars.push(var_name);
                    }
                }
            }
            Expr::Binary(binary) => {
                self.find_variables_in_expr(&binary.left, vars);
                self.find_variables_in_expr(&binary.right, vars);
            }
            Expr::Block(block) => {
                for stmt in &block.block.stmts {
                    if let Stmt::Expr(expr, _) = stmt {
                        self.find_variables_in_expr(expr, vars);
                    }
                }
            }
            _ => {
                // Recursively visit other expression types
            }
        }
    }
}
```

---

## Transformation Strategy

### Closure by Reference

**Input:**
```rust
let x = 42;
let closure = |y| x + y;
```

**Output:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", 42);
let closure = track_new(
    2,
    "closure",
    "closure",
    "line:2:9",
    |y| track_borrow(3, 1, false, "line:2:19", &x) + y
);
```

**Note:** We track the borrow of `x` inside the closure body.

### Move Closure

**Input:**
```rust
let x = String::from("hello");
let closure = move |y| x.len() + y;
```

**Output:**
```rust
let x = track_new(1, "x", "String", "line:1:9", String::from("hello"));
track_move(1, 2, "line:2:15");
let closure = track_new(
    2,
    "closure",
    "closure",
    "line:2:9",
    move |y| x.len() + y
);
```

---

## Implementation

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Closure(closure) => {
                self.transform_closure(closure);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

impl OwnershipVisitor {
    fn transform_closure(&mut self, closure: &mut syn::ExprClosure) {
        let is_move = self.is_move_closure(closure);
        let captured_vars = self.extract_captured_vars(&closure.body);
        
        if is_move {
            // For move closures, track moves of captured variables
            for var_name in &captured_vars {
                if let Some(&var_id) = self.var_ids.get(var_name) {
                    // We need to insert track_move before the closure
                    // This requires statement-level transformation
                    // Store for later insertion
                    self.pending_moves.push((var_id, var_name.clone()));
                }
            }
        } else {
            // For non-move closures, track borrows inside closure body
            self.transform_closure_body(&mut closure.body, &captured_vars);
        }
        
        // Visit closure parameters
        for input in &mut closure.inputs {
            self.visit_pat_mut(input);
        }
    }
    
    fn transform_closure_body(&mut self, body: &mut Expr, captured_vars: &[String]) {
        // Visit the body and wrap captured variable uses with track_borrow
        self.wrap_captured_uses(body, captured_vars);
    }
    
    fn wrap_captured_uses(&mut self, expr: &mut Expr, captured_vars: &[String]) {
        match expr {
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    
                    if captured_vars.contains(&var_name) {
                        if let Some(&var_id) = self.var_ids.get(&var_name) {
                            let borrow_id = self.next_id();
                            let location = self.get_location(path.span());
                            
                            // Wrap with track_borrow
                            let wrapped: Expr = syn::parse_quote! {
                                *borrowscope_runtime::track_borrow(
                                    #borrow_id,
                                    #var_id,
                                    false,
                                    #location,
                                    &#var_name
                                )
                            };
                            
                            *expr = wrapped;
                        }
                    }
                }
            }
            Expr::Binary(binary) => {
                self.wrap_captured_uses(&mut binary.left, captured_vars);
                self.wrap_captured_uses(&mut binary.right, captured_vars);
            }
            Expr::Block(block) => {
                for stmt in &mut block.block.stmts {
                    self.visit_stmt_mut(stmt);
                }
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
```

---

## Simplified Approach

**Challenge:** Tracking inside closures is complex because:
1. Closures can be called multiple times
2. Captures happen at closure creation, not call time
3. Closure bodies are separate scopes

**Simplified solution:** Track closure creation and captured variables:

```rust
impl OwnershipVisitor {
    fn transform_closure_simple(&mut self, closure: &mut syn::ExprClosure) {
        let is_move = self.is_move_closure(closure);
        let captured_vars = self.extract_captured_vars(&closure.body);
        
        // Just record that these variables are captured
        for var_name in &captured_vars {
            if let Some(&var_id) = self.var_ids.get(var_name) {
                let capture_id = self.next_id();
                let location = self.get_location(closure.span());
                
                // Record capture event
                self.record_capture(capture_id, var_id, is_move, &location);
            }
        }
        
        // Don't transform closure body
        // Just visit it normally
        self.visit_expr_mut(&mut closure.body);
    }
    
    fn record_capture(&mut self, capture_id: usize, var_id: usize, is_move: bool, location: &str) {
        // This would require a new event type: Event::Capture
        // For now, we can use existing events
        if is_move {
            // Track as move
            self.pending_moves.push((var_id, capture_id));
        } else {
            // Track as borrow
            // Insert at statement level
        }
    }
}
```

---

## Runtime Support

Add closure capture tracking:

```rust
// borrowscope-runtime/src/tracker.rs

/// Track closure capture
#[inline(always)]
pub fn track_capture(
    closure_id: usize,
    captured_id: usize,
    is_move: bool,
    location: &str,
) {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::Capture {
        closure_id,
        captured_id,
        is_move,
        location: location.to_string(),
        timestamp,
    });
}
```

### Event Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    // ... existing variants
    
    Capture {
        closure_id: usize,
        captured_id: usize,
        is_move: bool,
        location: String,
        timestamp: u64,
    },
}
```

---

## Testing

```rust
#[test]
fn test_closure_by_reference() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let closure = |y| x + y;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should track x and closure
    assert!(output.contains("track_new"));
}

#[test]
fn test_move_closure() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = String::from("hello");
            let closure = move |y| x.len() + y;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have move tracking
    assert!(output.contains("move"));
}
```

---

## Integration Test

```rust
use borrowscope_runtime::*;

#[test]
fn test_closure_capture() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    
    track_capture(2, 1, false, "test.rs:2:1");
    let closure = |y: i32| x + y;
    
    let result = closure(10);
    assert_eq!(result, 52);
    
    let events = get_events();
    
    // Should have: New(x), Capture(x)
    assert_eq!(events.len(), 2);
}
```

---

## Edge Cases

### Case 1: Nested Closures

```rust
let x = 42;
let outer = |y| {
    let inner = |z| x + y + z;
    inner(10)
};
```

**Solution:** Track captures at each level.

### Case 2: Closure as Argument

```rust
vec.iter().map(|item| item * 2)
```

**Solution:** Track the closure creation, not the iterator.

### Case 3: Fn Traits

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}
```

**Solution:** Track at call site, not inside generic function.

---

## Practical Limitations

### Can't Track Closure Calls

```rust
let closure = |x| x + 1;
let result = closure(42);  // Can't track this call
```

**Reason:** Closures are called through trait methods (Fn, FnMut, FnOnce), which we can't easily intercept.

### Can't Track Multiple Calls

```rust
let closure = |x| x + 1;
closure(1);
closure(2);
closure(3);
```

**Reason:** Each call would need separate tracking, but we transform at compile time.

---

## Recommended Approach

**For BorrowScope v1:** Track closure creation and captures, but not individual calls.

```rust
impl OwnershipVisitor {
    fn transform_closure_v1(&mut self, closure: &mut syn::ExprClosure) {
        let captured_vars = self.extract_captured_vars(&closure.body);
        let is_move = self.is_move_closure(closure);
        
        // Record what variables are captured
        for var_name in captured_vars {
            if let Some(&var_id) = self.var_ids.get(&var_name) {
                // Add metadata about capture
                self.closure_captures.push(ClosureCapture {
                    var_id,
                    is_move,
                });
            }
        }
        
        // Don't transform closure body
        // Just visit it to handle nested structures
        self.visit_expr_mut(&mut closure.body);
    }
}
```

---

## Key Takeaways

✅ **Closures capture variables** - Track at creation time  
✅ **Move vs borrow** - Detect from `move` keyword  
✅ **Extract captured vars** - Analyze closure body  
✅ **Simplified tracking** - Don't track individual calls  
✅ **Document limitations** - Can't track call-by-call  

---

## Further Reading

- [Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Closure traits](https://doc.rust-lang.org/book/ch13-01-closures.html#closure-type-inference-and-annotation)
- [Capturing environment](https://doc.rust-lang.org/reference/expressions/closure-expr.html)

---

**Previous:** [44-method-call-transformations.md](./44-method-call-transformations.md)  
**Next:** [46-macro-expansion-considerations.md](./46-macro-expansion-considerations.md)

**Progress:** 10/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜
