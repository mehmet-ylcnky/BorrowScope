# Section 41: Handling Scope Boundaries

## Learning Objectives

By the end of this section, you will:
- Track variable lifetimes across scopes
- Insert drop calls at scope boundaries
- Maintain LIFO drop order
- Handle nested scopes correctly
- Implement scope-aware variable tracking

## Prerequisites

- Completed Section 40 (track_move injection)
- Understanding of Rust's drop semantics
- Familiarity with scope and lifetime concepts

---

## Drop Semantics in Rust

Variables are dropped at the end of their scope in LIFO (Last In, First Out) order:

```rust
{
    let x = String::from("x");  // Created first
    let y = String::from("y");  // Created second
    let z = String::from("z");  // Created third
}  // Dropped in order: z, y, x (LIFO)
```

---

## Scope Tracking Strategy

We need to:
1. Track which variables are created in each scope
2. Insert drop calls at the end of each scope
3. Maintain LIFO order

### Data Structure

```rust
use std::collections::HashMap;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
    var_ids: HashMap<String, usize>,
    current_stmt_index: usize,
    pending_inserts: Vec<(usize, Stmt)>,
    /// Stack of scopes, each containing variable IDs created in that scope
    scope_stack: Vec<Vec<usize>>,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
            var_ids: HashMap::new(),
            current_stmt_index: 0,
            pending_inserts: Vec::new(),
            scope_stack: vec![Vec::new()],  // Start with root scope
        }
    }
}
```

---

## Implementation

### Step 1: Track Variables Per Scope

```rust
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let type_name = self.extract_type_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
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
}
```

### Step 2: Insert Drops at Scope End

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        self.pending_inserts.clear();
        
        // Push new scope
        self.scope_stack.push(Vec::new());
        
        // Visit all statements
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            self.visit_stmt_mut(stmt);
        }
        
        // Insert pending statements
        for (idx, stmt) in self.pending_inserts.drain(..).rev() {
            block.stmts.insert(idx, stmt);
        }
        
        // Pop scope and insert drops
        if let Some(scope_vars) = self.scope_stack.pop() {
            self.insert_drops(block, scope_vars);
        }
        
        self.scope_depth -= 1;
    }
    
    fn insert_drops(&mut self, block: &mut Block, var_ids: Vec<usize>) {
        // Insert drops in LIFO order (reverse of creation)
        for var_id in var_ids.into_iter().rev() {
            let location = "scope_end";  // We'll improve this
            
            let drop_stmt: Stmt = syn::parse_quote! {
                borrowscope_runtime::track_drop(#var_id, #location);
            };
            
            block.stmts.push(drop_stmt);
        }
    }
}
```

---

## Complete Implementation

Here's the full implementation with scope tracking:

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Block, Stmt, Local, Expr, ItemFn, Pat,
};
use std::collections::HashMap;
use quote::quote;
use proc_macro2::Span;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
    var_ids: HashMap<String, usize>,
    current_stmt_index: usize,
    pending_inserts: Vec<(usize, Stmt)>,
    scope_stack: Vec<Vec<usize>>,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
            var_ids: HashMap::new(),
            current_stmt_index: 0,
            pending_inserts: Vec::new(),
            scope_stack: vec![Vec::new()],
        }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    fn extract_var_name(&self, pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            Pat::Type(pat_type) => self.extract_var_name(&pat_type.pat),
            _ => "unknown".to_string(),
        }
    }
    
    fn extract_type_name(&self, pat: &Pat) -> String {
        match pat {
            Pat::Type(pat_type) => {
                quote!(#pat_type.ty).to_string().replace(" ", "")
            }
            _ => "inferred".to_string(),
        }
    }
    
    fn get_location(&self, span: Span) -> String {
        let start = span.start();
        format!("line:{}:{}", start.line, start.column)
    }
}

impl VisitMut for OwnershipVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        self.visit_block_mut(&mut func.block);
    }
    
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        self.pending_inserts.clear();
        
        // Push new scope
        self.scope_stack.push(Vec::new());
        
        // Visit statements
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            self.visit_stmt_mut(stmt);
        }
        
        // Insert pending statements
        for (idx, stmt) in self.pending_inserts.drain(..).rev() {
            block.stmts.insert(idx, stmt);
        }
        
        // Pop scope and insert drops
        if let Some(scope_vars) = self.scope_stack.pop() {
            // Insert drops in LIFO order
            for var_id in scope_vars.into_iter().rev() {
                let drop_stmt: Stmt = syn::parse_quote! {
                    borrowscope_runtime::track_drop(#var_id, "scope_end");
                };
                block.stmts.push(drop_stmt);
            }
        }
        
        self.scope_depth -= 1;
    }
    
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                if self.is_potential_move(local) {
                    self.handle_move_assignment(local);
                } else {
                    self.transform_local(local);
                }
            }
            _ => {
                visit_mut::visit_stmt_mut(self, stmt);
            }
        }
    }
}

impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let type_name = self.extract_type_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
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
    
    fn is_potential_move(&self, local: &Local) -> bool {
        if let Some(init) = &local.init {
            matches!(*init.expr, Expr::Path(_))
        } else {
            false
        }
    }
    
    fn handle_move_assignment(&mut self, local: &mut Local) {
        if let Some(init) = &local.init {
            if let Expr::Path(ref path_expr) = *init.expr {
                if let Some(source_ident) = path_expr.path.get_ident() {
                    let source_name = source_ident.to_string();
                    let source_id = *self.var_ids.get(&source_name).unwrap_or(&0);
                    
                    let dest_id = self.next_id();
                    let dest_name = self.extract_var_name(&local.pat);
                    self.var_ids.insert(dest_name.clone(), dest_id);
                    
                    // Add to current scope
                    if let Some(current_scope) = self.scope_stack.last_mut() {
                        current_scope.push(dest_id);
                    }
                    
                    let location = self.get_location(local.span());
                    
                    let move_stmt: Stmt = syn::parse_quote! {
                        borrowscope_runtime::track_move(#source_id, #dest_id, #location);
                    };
                    
                    self.pending_inserts.push((self.current_stmt_index, move_stmt));
                }
            }
        }
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/scope_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_simple_scope() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let y = 100;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have two track_new calls
    assert_eq!(output.matches("track_new").count(), 2);
    
    // Should have two track_drop calls
    assert_eq!(output.matches("track_drop").count(), 2);
    
    // Drops should be in LIFO order (y before x)
    let drop_y_pos = output.find("track_drop (2").unwrap();
    let drop_x_pos = output.find("track_drop (1").unwrap();
    assert!(drop_y_pos < drop_x_pos, "y should be dropped before x");
}

#[test]
fn test_nested_scopes() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            {
                let y = 100;
            }
            let z = 200;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have 3 track_new and 3 track_drop
    assert_eq!(output.matches("track_new").count(), 3);
    assert_eq!(output.matches("track_drop").count(), 3);
}

#[test]
fn test_empty_scope() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            {
                // Empty inner scope
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have no tracking calls
    assert_eq!(output.matches("track_").count(), 0);
}

#[test]
fn test_multiple_nested_scopes() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let a = 1;
            {
                let b = 2;
                {
                    let c = 3;
                }
                let d = 4;
            }
            let e = 5;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have 5 variables
    assert_eq!(output.matches("track_new").count(), 5);
    assert_eq!(output.matches("track_drop").count(), 5);
}
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/scope_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_scope_drops() {
    reset_tracker();
    
    {
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        let y = track_new(2, "y", "i32", "test.rs:2:1", 100);
        
        track_drop(2, "scope_end");
        track_drop(1, "scope_end");
    }
    
    let events = get_events();
    
    // Should have: New(x), New(y), Drop(y), Drop(x)
    assert_eq!(events.len(), 4);
    
    match &events[2] {
        Event::Drop { id, .. } => assert_eq!(*id, 2),  // y dropped first
        _ => panic!("Expected Drop event"),
    }
    
    match &events[3] {
        Event::Drop { id, .. } => assert_eq!(*id, 1),  // x dropped second
        _ => panic!("Expected Drop event"),
    }
}

#[test]
fn test_nested_scopes() {
    reset_tracker();
    
    {
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        {
            let y = track_new(2, "y", "i32", "test.rs:3:1", 100);
            track_drop(2, "scope_end");
        }
        track_drop(1, "scope_end");
    }
    
    let events = get_events();
    
    // Should have: New(x), New(y), Drop(y), Drop(x)
    assert_eq!(events.len(), 4);
}
```

---

## Example Output

**Input:**
```rust
#[track_ownership]
fn example() {
    let x = 42;
    {
        let y = 100;
    }
    let z = 200;
}
```

**Output:**
```rust
fn example() {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "line:3:9", 42);
    {
        let y = borrowscope_runtime::track_new(2, "y", "i32", "line:5:13", 100);
        borrowscope_runtime::track_drop(2, "scope_end");
    }
    let z = borrowscope_runtime::track_new(3, "z", "i32", "line:7:9", 200);
    borrowscope_runtime::track_drop(3, "scope_end");
    borrowscope_runtime::track_drop(1, "scope_end");
}
```

**Drop order:**
1. `y` dropped at end of inner scope
2. `z` dropped at end of function (before `x`)
3. `x` dropped at end of function

---

## Handling Early Returns

Early returns complicate drop tracking:

```rust
fn example(condition: bool) -> i32 {
    let x = 42;
    if condition {
        return x;  // x is moved, not dropped
    }
    let y = 100;
    y  // y is moved, x is dropped
}
```

**Solution:** Insert drops before each return:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Return(return_expr) => {
                self.handle_early_return(return_expr);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

impl OwnershipVisitor {
    fn handle_early_return(&mut self, return_expr: &mut syn::ExprReturn) {
        // Get all variables in current scope
        let vars_to_drop: Vec<usize> = self.scope_stack
            .iter()
            .flatten()
            .copied()
            .collect();
        
        // Wrap return in a block with drops
        let drops: Vec<Stmt> = vars_to_drop
            .into_iter()
            .rev()
            .map(|id| {
                syn::parse_quote! {
                    borrowscope_runtime::track_drop(#id, "early_return");
                }
            })
            .collect();
        
        let return_value = &return_expr.expr;
        
        let wrapped: Expr = syn::parse_quote! {
            {
                #(#drops)*
                return #return_value;
            }
        };
        
        // Replace the return expression
        // Note: This is simplified; actual implementation needs more care
    }
}
```

---

## Handling Explicit drop()

Rust allows explicit drops:

```rust
let x = String::from("hello");
drop(x);  // Explicit drop
// x is no longer valid
```

**Detection:**

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Call(call_expr) => {
                if self.is_drop_call(call_expr) {
                    self.handle_explicit_drop(call_expr);
                }
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

impl OwnershipVisitor {
    fn is_drop_call(&self, call_expr: &syn::ExprCall) -> bool {
        if let Expr::Path(path_expr) = &*call_expr.func {
            if let Some(ident) = path_expr.path.get_ident() {
                return ident == "drop";
            }
        }
        false
    }
    
    fn handle_explicit_drop(&mut self, call_expr: &mut syn::ExprCall) {
        // Extract the argument (variable being dropped)
        if let Some(arg) = call_expr.args.first() {
            if let Expr::Path(path_expr) = arg {
                if let Some(ident) = path_expr.path.get_ident() {
                    let var_name = ident.to_string();
                    if let Some(&var_id) = self.var_ids.get(&var_name) {
                        // Insert track_drop before the drop() call
                        // This requires statement-level transformation
                        
                        // For now, wrap the drop call
                        let location = self.get_location(call_expr.span());
                        
                        let wrapped: Expr = syn::parse_quote! {
                            {
                                borrowscope_runtime::track_drop(#var_id, #location);
                                drop(#arg)
                            }
                        };
                        
                        // Replace call_expr with wrapped version
                        // Note: Need to modify parent statement
                    }
                }
            }
        }
    }
}
```

---

## Edge Cases

### Case 1: Loop Scopes

```rust
for i in 0..3 {
    let x = i;
    // x is dropped at end of each iteration
}
```

**Solution:** Each loop iteration is a scope, drops are inserted automatically.

### Case 2: Match Arms

```rust
match value {
    Some(x) => {
        // x is dropped here
    }
    None => {}
}
```

**Solution:** Each match arm is a scope.

### Case 3: Shadowing

```rust
let x = 1;
let x = 2;  // First x is dropped here
```

**Solution:** Track as drop of first `x`, then new variable.

---

## Key Takeaways

✅ **Track scopes with stack** - Push/pop on block entry/exit  
✅ **LIFO drop order** - Reverse variable creation order  
✅ **Insert at scope end** - Append drops to block statements  
✅ **Handle early returns** - Insert drops before return  
✅ **Track explicit drops** - Detect drop() calls  

---

## Further Reading

- [Rust drop semantics](https://doc.rust-lang.org/reference/destructors.html)
- [RAII pattern](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
- [Scope and lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

---

**Previous:** [40-injecting-track-move-calls.md](./40-injecting-track-move-calls.md)  
**Next:** [42-dealing-with-patterns.md](./42-dealing-with-patterns.md)

**Progress:** 6/15 ⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜
