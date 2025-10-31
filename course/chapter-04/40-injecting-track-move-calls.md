# Section 40: Injecting track_move Calls

## Learning Objectives

By the end of this section, you will:
- Detect move semantics in assignments
- Track ownership transfers
- Handle moves in function arguments
- Distinguish moves from copies
- Insert tracking before moves occur

## Prerequisites

- Completed Section 39 (track_borrow injection)
- Understanding of Rust move semantics
- Familiarity with Copy vs Move types

---

## Move Semantics in Rust

A move occurs when:
1. Assigning a non-Copy value to another variable
2. Passing a non-Copy value to a function
3. Returning a non-Copy value from a function

```rust
let s1 = String::from("hello");
let s2 = s1;  // Move: s1 is no longer valid
```

---

## Detection Strategy

**Challenge:** At macro expansion time, we don't know if a type implements Copy.

**Solution:** Track all assignments and let the runtime handle it.

```rust
// We transform this
let y = x;

// Into this
borrowscope_runtime::track_move(1, 2, "line:2:9");
let y = x;
```

**Note:** Insert tracking *before* the move, since `x` becomes invalid after.

---

## Implementation

### Step 1: Detect Assignments

Add to `borrowscope-macro/src/visitor.rs`:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                // Check if this is a move assignment
                if self.is_move_assignment(local) {
                    self.handle_move_in_local(local);
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
    fn is_move_assignment(&self, local: &Local) -> bool {
        if let Some(init) = &local.init {
            // Check if initializer is a simple path (variable)
            matches!(*init.expr, Expr::Path(_))
        } else {
            false
        }
    }
    
    fn handle_move_in_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            if let Expr::Path(ref path_expr) = *init.expr {
                // Extract source variable name
                if let Some(source_ident) = path_expr.path.get_ident() {
                    let source_name = source_ident.to_string();
                    let source_id = *self.var_ids.get(&source_name).unwrap_or(&0);
                    
                    // Generate new ID for destination
                    let dest_id = self.next_id();
                    let dest_name = self.extract_var_name(&local.pat);
                    
                    // Store destination ID
                    self.var_ids.insert(dest_name.clone(), dest_id);
                    
                    // We need to insert track_move BEFORE the let statement
                    // This requires modifying the parent block, which we'll handle differently
                    
                    // For now, wrap the expression
                    let location = self.get_location(local.span());
                    let original_expr = init.expr.clone();
                    
                    let new_expr: Expr = syn::parse_quote! {
                        {
                            borrowscope_runtime::track_move(#source_id, #dest_id, #location);
                            #original_expr
                        }
                    };
                    
                    *init.expr = new_expr;
                }
            }
        }
    }
}
```

---

## Better Approach: Block-Level Transformation

Instead of wrapping in a block, collect moves and insert statements:

```rust
use syn::Stmt;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
    var_ids: HashMap<String, usize>,
    /// Statements to insert (index, statement)
    pending_stmts: Vec<(usize, Stmt)>,
}

impl OwnershipVisitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        self.pending_stmts.clear();
        
        // Visit all statements
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            self.visit_stmt_mut(stmt);
        }
        
        // Insert pending statements in reverse order
        for (idx, stmt) in self.pending_stmts.drain(..).rev() {
            block.stmts.insert(idx, stmt);
        }
        
        self.scope_depth -= 1;
    }
    
    fn handle_move_in_local(&mut self, local: &mut Local) {
        if let Some(init) = &local.init {
            if let Expr::Path(ref path_expr) = *init.expr {
                if let Some(source_ident) = path_expr.path.get_ident() {
                    let source_name = source_ident.to_string();
                    let source_id = *self.var_ids.get(&source_name).unwrap_or(&0);
                    
                    let dest_id = self.next_id();
                    let dest_name = self.extract_var_name(&local.pat);
                    self.var_ids.insert(dest_name.clone(), dest_id);
                    
                    let location = self.get_location(local.span());
                    
                    // Create track_move statement
                    let move_stmt: Stmt = syn::parse_quote! {
                        borrowscope_runtime::track_move(#source_id, #dest_id, #location);
                    };
                    
                    // Insert before current statement
                    self.pending_stmts.push((self.current_stmt_index, move_stmt));
                }
            }
        }
        
        // Still transform the local normally
        self.transform_local(local);
    }
}
```

---

## Complete Implementation

Here's the full implementation with move tracking:

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Block, Stmt, Local, Expr, ExprPath, Pat,
};
use std::collections::HashMap;
use quote::quote;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
    var_ids: HashMap<String, usize>,
    current_stmt_index: usize,
    pending_inserts: Vec<(usize, Stmt)>,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
            var_ids: HashMap::new(),
            current_stmt_index: 0,
            pending_inserts: Vec::new(),
        }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl VisitMut for OwnershipVisitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        self.pending_inserts.clear();
        
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            self.visit_stmt_mut(stmt);
        }
        
        // Insert pending statements
        for (idx, stmt) in self.pending_inserts.drain(..).rev() {
            block.stmts.insert(idx, stmt);
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
                    
                    let location = self.get_location(local.span());
                    
                    // Insert track_move before this statement
                    let move_stmt: Stmt = syn::parse_quote! {
                        borrowscope_runtime::track_move(#source_id, #dest_id, #location);
                    };
                    
                    self.pending_inserts.push((self.current_stmt_index, move_stmt));
                }
            }
        }
        
        // Don't wrap with track_new for moves
        // The destination variable doesn't need tracking since it's a move
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/track_move_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_simple_move() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = String::from("hello");
            let y = x;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have track_new for x
    assert!(output.contains("track_new"));
    
    // Should have track_move before y assignment
    assert!(output.contains("track_move"));
}

#[test]
fn test_move_chain() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = String::from("hello");
            let y = x;
            let z = y;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have two track_move calls
    assert_eq!(output.matches("track_move").count(), 2);
}

#[test]
fn test_move_vs_copy() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let y = x;  // This is actually a copy, but we track it anyway
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should still have track_move (runtime will handle Copy types)
    assert!(output.contains("track_move"));
}
```

---

## Handling Function Arguments

Moves also occur when passing values to functions:

```rust
fn take_ownership(s: String) {
    // s is moved here
}

let s = String::from("hello");
take_ownership(s);  // Move occurs here
```

**Challenge:** Detecting moves in function calls requires more complex analysis.

**Simplified approach:** Track all function calls with non-Copy arguments:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Call(call_expr) => {
                self.transform_call(call_expr);
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

impl OwnershipVisitor {
    fn transform_call(&mut self, call_expr: &mut syn::ExprCall) {
        // Visit arguments to detect moves
        for arg in &mut call_expr.args {
            if let Expr::Path(path_expr) = arg {
                if let Some(ident) = path_expr.path.get_ident() {
                    let var_name = ident.to_string();
                    if let Some(&var_id) = self.var_ids.get(&var_name) {
                        // Potentially a move - wrap the argument
                        let location = self.get_location(arg.span());
                        let temp_id = self.next_id();
                        
                        let wrapped: Expr = syn::parse_quote! {
                            {
                                borrowscope_runtime::track_move(#var_id, #temp_id, #location);
                                #arg
                            }
                        };
                        
                        *arg = wrapped;
                    }
                }
            }
        }
        
        // Continue visiting nested expressions
        visit_mut::visit_expr_call_mut(self, call_expr);
    }
}
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/move_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_simple_move() {
    reset_tracker();
    
    let s1 = track_new(1, "s1", "String", "test.rs:1:1", String::from("hello"));
    track_move(1, 2, "test.rs:2:1");
    let s2 = s1;
    
    assert_eq!(s2, "hello");
    
    let events = get_events();
    assert_eq!(events.len(), 2);
    
    match &events[1] {
        Event::Move { from_id, to_id, .. } => {
            assert_eq!(*from_id, 1);
            assert_eq!(*to_id, 2);
        }
        _ => panic!("Expected Move event"),
    }
}

#[test]
fn test_move_chain() {
    reset_tracker();
    
    let s1 = track_new(1, "s1", "String", "test.rs:1:1", String::from("hello"));
    track_move(1, 2, "test.rs:2:1");
    let s2 = s1;
    track_move(2, 3, "test.rs:3:1");
    let s3 = s2;
    
    assert_eq!(s3, "hello");
    
    let events = get_events();
    
    // Should have: New, Move, Move
    assert_eq!(events.len(), 3);
}

#[test]
fn test_copy_type() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    track_move(1, 2, "test.rs:2:1");
    let y = x;
    
    // Both x and y are valid (Copy type)
    assert_eq!(x, 42);
    assert_eq!(y, 42);
    
    let events = get_events();
    
    // Move event is recorded even for Copy types
    assert_eq!(events.len(), 2);
}
```

---

## Example Output

**Input:**
```rust
#[track_ownership]
fn example() {
    let s1 = String::from("hello");
    let s2 = s1;
    let s3 = s2;
}
```

**Output:**
```rust
fn example() {
    let s1 = borrowscope_runtime::track_new(
        1,
        "s1",
        "inferred",
        "line:3:9",
        String::from("hello")
    );
    borrowscope_runtime::track_move(1, 2, "line:4:9");
    let s2 = s1;
    borrowscope_runtime::track_move(2, 3, "line:5:9");
    let s3 = s2;
}
```

---

## Edge Cases

### Case 1: Partial Moves

```rust
struct Pair {
    x: String,
    y: String,
}

let p = Pair { x: String::from("a"), y: String::from("b") };
let x = p.x;  // Partial move
// p.y is still valid, but p is not
```

**Solution:** Track the struct, note that it's partially moved.

### Case 2: Moves in Match

```rust
match some_option {
    Some(value) => {
        // value is moved here
    }
    None => {}
}
```

**Solution:** Handle in pattern matching section (Section 42).

### Case 3: Conditional Moves

```rust
let s = String::from("hello");
if condition {
    let s2 = s;  // Move only happens if condition is true
}
// s may or may not be valid here
```

**Solution:** Track moves in all branches, runtime handles actual execution.

---

## Optimization: Skip Copy Types

For better performance, we could skip tracking moves of known Copy types:

```rust
impl OwnershipVisitor {
    fn is_known_copy_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16" | "i32" | "i64" | "i128" |
            "u8" | "u16" | "u32" | "u64" | "u128" |
            "f32" | "f64" | "bool" | "char" |
            "&_" | "&mut_"  // References are Copy
        )
    }
    
    fn handle_move_assignment(&mut self, local: &mut Local) {
        let type_name = self.extract_type_name(&local.pat);
        
        if self.is_known_copy_type(&type_name) {
            // Skip move tracking for Copy types
            self.transform_local(local);
            return;
        }
        
        // ... rest of move handling
    }
}
```

**Tradeoff:** More complex logic, but fewer runtime tracking calls.

---

## Key Takeaways

✅ **Insert before move** - track_move must come before assignment  
✅ **Track all assignments** - Can't distinguish Copy at macro time  
✅ **Use pending inserts** - Modify block after iteration  
✅ **Handle function calls** - Moves occur in arguments  
✅ **Runtime handles Copy** - Let runtime determine actual moves  

---

## Further Reading

- [Rust move semantics](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html)
- [Move vs Copy](https://doc.rust-lang.org/nomicon/ownership.html)

---

**Previous:** [39-injecting-track-borrow-calls.md](./39-injecting-track-borrow-calls.md)  
**Next:** [41-handling-scope-boundaries.md](./41-handling-scope-boundaries.md)

**Progress:** 5/15 ⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
