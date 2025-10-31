# Section 43: Handling Control Flow

## Learning Objectives

By the end of this section, you will:
- Transform if/else expressions
- Handle match expressions and patterns
- Process loop constructs (for, while, loop)
- Track variables across control flow branches
- Insert drops in all execution paths

## Prerequisites

- Completed Section 42 (Pattern Handling)
- Understanding of Rust control flow
- Familiarity with expression-based control flow

---

## Control Flow in Rust

Rust's control flow constructs are expressions:

```rust
// if expression
let x = if condition { 1 } else { 2 };

// match expression
let y = match value {
    Some(v) => v,
    None => 0,
};

// loop expression
let z = loop {
    break 42;
};
```

---

## If/Else Expressions

### Simple If

```rust
if condition {
    let x = 42;
}
```

**Transformation:** Each branch is a block, already handled by scope tracking.

```rust
if condition {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "line:2:13", 42);
    borrowscope_runtime::track_drop(1, "scope_end");
}
```

### If/Else

```rust
let x = if condition {
    let y = 1;
    y
} else {
    let z = 2;
    z
};
```

**Transformation:**

```rust
let x = borrowscope_runtime::track_new(
    1,
    "x",
    "inferred",
    "line:1:9",
    if condition {
        let y = borrowscope_runtime::track_new(2, "y", "i32", "line:3:13", 1);
        let __result = y;
        borrowscope_runtime::track_drop(2, "scope_end");
        __result
    } else {
        let z = borrowscope_runtime::track_new(3, "z", "i32", "line:6:13", 2);
        let __result = z;
        borrowscope_runtime::track_drop(3, "scope_end");
        __result
    }
);
```

---

## Implementation

### Step 1: Visit If Expressions

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::If(if_expr) => {
                self.transform_if(if_expr);
            }
            Expr::Match(match_expr) => {
                self.transform_match(match_expr);
            }
            Expr::ForLoop(for_loop) => {
                self.transform_for_loop(for_loop);
            }
            Expr::While(while_loop) => {
                self.transform_while(while_loop);
            }
            Expr::Loop(loop_expr) => {
                self.transform_loop(loop_expr);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
```

### Step 2: Transform If Expressions

```rust
impl OwnershipVisitor {
    fn transform_if(&mut self, if_expr: &mut syn::ExprIf) {
        // Visit condition
        self.visit_expr_mut(&mut if_expr.cond);
        
        // Visit then branch (it's a block)
        self.visit_block_mut(&mut if_expr.then_branch);
        
        // Visit else branch if present
        if let Some((_, else_branch)) = &mut if_expr.else_branch {
            self.visit_expr_mut(else_branch);
        }
    }
}
```

**Note:** Blocks are already handled by `visit_block_mut`, which inserts drops automatically.

---

## Match Expressions

Match is more complex due to pattern matching:

```rust
match value {
    Some(x) => {
        // x is bound here
    }
    None => {}
}
```

### Implementation

```rust
impl OwnershipVisitor {
    fn transform_match(&mut self, match_expr: &mut syn::ExprMatch) {
        // Visit the matched expression
        self.visit_expr_mut(&mut match_expr.expr);
        
        // Visit each arm
        for arm in &mut match_expr.arms {
            self.transform_match_arm(arm);
        }
    }
    
    fn transform_match_arm(&mut self, arm: &mut syn::Arm) {
        // Extract variables from pattern
        let vars = self.extract_pattern_vars(&arm.pat);
        
        // Track pattern variables
        // This is complex because patterns bind variables implicitly
        
        // For now, just visit the body
        self.visit_expr_mut(&mut arm.body);
        
        // Visit guard if present
        if let Some((_, guard)) = &mut arm.guard {
            self.visit_expr_mut(guard);
        }
    }
}
```

### Challenge: Pattern Bindings

Match patterns bind variables implicitly:

```rust
match some_option {
    Some(x) => {
        // x is available here
    }
    None => {}
}
```

**Problem:** We can't insert `track_new` for `x` because it's bound by the pattern.

**Solution:** Track the matched value, note the binding:

```rust
let __match_temp = borrowscope_runtime::track_new(1, "__match", "Option<i32>", "line:1:7", some_option);
match __match_temp {
    Some(x) => {
        // Track x as a "pattern binding"
        borrowscope_runtime::track_pattern_binding(2, "x", 1);
        // ... rest of arm
    }
    None => {}
}
```

**Simplified approach:** Don't track pattern-bound variables for now.

---

## Loop Constructs

### For Loop

```rust
for item in vec {
    // item is bound for each iteration
}
```

**Implementation:**

```rust
impl OwnershipVisitor {
    fn transform_for_loop(&mut self, for_loop: &mut syn::ExprForLoop) {
        // Visit the iterator expression
        self.visit_expr_mut(&mut for_loop.expr);
        
        // The loop body is a block
        self.visit_block_mut(&mut for_loop.body);
        
        // Note: The loop variable (for_loop.pat) is bound implicitly
        // We could track it, but it's created/dropped each iteration
    }
}
```

### While Loop

```rust
while condition {
    let x = 42;
    // x is dropped at end of each iteration
}
```

**Implementation:**

```rust
impl OwnershipVisitor {
    fn transform_while(&mut self, while_loop: &mut syn::ExprWhile) {
        // Visit condition
        self.visit_expr_mut(&mut while_loop.cond);
        
        // Visit body
        self.visit_block_mut(&mut while_loop.body);
    }
}
```

### Infinite Loop

```rust
loop {
    let x = 42;
    if condition {
        break;
    }
}
```

**Implementation:**

```rust
impl OwnershipVisitor {
    fn transform_loop(&mut self, loop_expr: &mut syn::ExprLoop) {
        // Visit body
        self.visit_block_mut(&mut loop_expr.body);
    }
}
```

---

## Complete Implementation

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Expr, ExprIf, ExprMatch, ExprForLoop, ExprWhile, ExprLoop,
    Arm, Block,
};

impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::If(if_expr) => {
                self.visit_expr_mut(&mut if_expr.cond);
                self.visit_block_mut(&mut if_expr.then_branch);
                if let Some((_, else_branch)) = &mut if_expr.else_branch {
                    self.visit_expr_mut(else_branch);
                }
            }
            Expr::Match(match_expr) => {
                self.visit_expr_mut(&mut match_expr.expr);
                for arm in &mut match_expr.arms {
                    // Visit guard
                    if let Some((_, guard)) = &mut arm.guard {
                        self.visit_expr_mut(guard);
                    }
                    // Visit body
                    self.visit_expr_mut(&mut arm.body);
                }
            }
            Expr::ForLoop(for_loop) => {
                self.visit_expr_mut(&mut for_loop.expr);
                self.visit_block_mut(&mut for_loop.body);
            }
            Expr::While(while_loop) => {
                self.visit_expr_mut(&mut while_loop.cond);
                self.visit_block_mut(&mut while_loop.body);
            }
            Expr::Loop(loop_expr) => {
                self.visit_block_mut(&mut loop_expr.body);
            }
            Expr::Reference(ref_expr) => {
                self.transform_reference(expr, ref_expr);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/control_flow_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_if_expression() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            if true {
                let x = 42;
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("track_drop"));
}

#[test]
fn test_if_else() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            if condition {
                let x = 1;
            } else {
                let y = 2;
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should track both x and y
    assert!(output.contains("\"x\""));
    assert!(output.contains("\"y\""));
}

#[test]
fn test_match_expression() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            match value {
                Some(x) => {
                    let y = x;
                }
                None => {}
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should track y (x is pattern-bound)
    assert!(output.contains("\"y\""));
}

#[test]
fn test_for_loop() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            for i in 0..3 {
                let x = i;
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should track x
    assert!(output.contains("\"x\""));
    assert!(output.contains("track_drop"));
}

#[test]
fn test_while_loop() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            while condition {
                let x = 42;
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
}

#[test]
fn test_nested_control_flow() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            if condition {
                for i in 0..3 {
                    let x = i;
                }
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("track_drop"));
}
```

---

## Example Transformations

### Example 1: If/Else

**Input:**
```rust
if condition {
    let x = 1;
} else {
    let y = 2;
}
```

**Output:**
```rust
if condition {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "line:2:13", 1);
    borrowscope_runtime::track_drop(1, "scope_end");
} else {
    let y = borrowscope_runtime::track_new(2, "y", "i32", "line:4:13", 2);
    borrowscope_runtime::track_drop(2, "scope_end");
}
```

### Example 2: Match

**Input:**
```rust
match value {
    Some(x) => {
        let y = x + 1;
    }
    None => {}
}
```

**Output:**
```rust
match value {
    Some(x) => {
        let y = borrowscope_runtime::track_new(1, "y", "inferred", "line:3:13", x + 1);
        borrowscope_runtime::track_drop(1, "scope_end");
    }
    None => {}
}
```

### Example 3: For Loop

**Input:**
```rust
for i in 0..3 {
    let x = i * 2;
}
```

**Output:**
```rust
for i in 0..3 {
    let x = borrowscope_runtime::track_new(1, "x", "inferred", "line:2:13", i * 2);
    borrowscope_runtime::track_drop(1, "scope_end");
}
```

**Note:** Variable `x` is created and dropped on each iteration.

---

## Edge Cases

### Case 1: Break with Value

```rust
let x = loop {
    let y = 42;
    break y;
};
```

**Challenge:** `y` is moved out of the loop.

**Solution:** Track the move:

```rust
let x = borrowscope_runtime::track_new(
    1,
    "x",
    "inferred",
    "line:1:9",
    loop {
        let y = borrowscope_runtime::track_new(2, "y", "i32", "line:2:13", 42);
        borrowscope_runtime::track_move(2, 1, "line:3:11");
        break y;
    }
);
```

### Case 2: Continue

```rust
for i in 0..10 {
    let x = i;
    if x % 2 == 0 {
        continue;
    }
    let y = x * 2;
}
```

**Solution:** Drops are inserted at scope end, which happens on continue.

### Case 3: Nested Loops

```rust
for i in 0..3 {
    for j in 0..3 {
        let x = i + j;
    }
}
```

**Solution:** Each loop body is a separate scope, handled automatically.

---

## Handling Break and Continue

Break and continue affect control flow:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Break(break_expr) => {
                // Variables in scope are dropped before break
                // This is handled automatically by Rust
                visit_mut::visit_expr_break_mut(self, break_expr);
            }
            Expr::Continue(_) => {
                // Variables in scope are dropped before continue
                // Also handled automatically
                visit_mut::visit_expr_mut(self, expr);
            }
            _ => {
                // ... other cases
            }
        }
    }
}
```

**Note:** Rust automatically drops variables before break/continue, so we don't need special handling.

---

## Integration Test

Create `borrowscope-macro/tests/integration/control_flow_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_if_branches() {
    reset_tracker();
    
    let condition = true;
    if condition {
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        track_drop(1, "scope_end");
    } else {
        let y = track_new(2, "y", "i32", "test.rs:2:1", 100);
        track_drop(2, "scope_end");
    }
    
    let events = get_events();
    
    // Only x branch executed
    assert_eq!(events.len(), 2);
    
    match &events[0] {
        Event::New { name, .. } => assert_eq!(name, "x"),
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_loop_iterations() {
    reset_tracker();
    
    for i in 0..3 {
        let x = track_new(i + 1, "x", "i32", "test.rs:1:1", i);
        track_drop(i + 1, "scope_end");
    }
    
    let events = get_events();
    
    // 3 iterations: New, Drop, New, Drop, New, Drop
    assert_eq!(events.len(), 6);
}
```

---

## Key Takeaways

✅ **Blocks handle scopes** - If/else/match arms are blocks  
✅ **Recursive visiting** - Visit all nested expressions  
✅ **Pattern bindings** - Match patterns bind variables implicitly  
✅ **Loop iterations** - Each iteration is a new scope  
✅ **Break/continue** - Rust handles drops automatically  

---

## Further Reading

- [Rust control flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [Match expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html)
- [Loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html)

---

**Previous:** [42-dealing-with-patterns.md](./42-dealing-with-patterns.md)  
**Next:** [44-method-call-transformations.md](./44-method-call-transformations.md)

**Progress:** 8/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜
