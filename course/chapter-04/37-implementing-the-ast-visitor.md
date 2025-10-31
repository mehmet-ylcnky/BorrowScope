# Section 37: Implementing the AST Visitor

## Learning Objectives

By the end of this section, you will:
- Implement syn's VisitMut trait
- Traverse and modify AST nodes
- Track transformation state
- Handle nested structures
- Build a reusable visitor pattern

## Prerequisites

- Completed Section 36 (Transformation Strategy)
- Understanding of visitor pattern
- Familiarity with syn's AST types

---

## The VisitMut Trait

syn provides `VisitMut` for modifying AST nodes in-place:

```rust
use syn::visit_mut::{self, VisitMut};

pub trait VisitMut {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) { /* ... */ }
    fn visit_stmt_mut(&mut self, node: &mut Stmt) { /* ... */ }
    fn visit_expr_mut(&mut self, node: &mut Expr) { /* ... */ }
    // ... hundreds more methods
}
```

**Key insight:** Override methods for nodes you want to transform.

---

## Basic Visitor Structure

Create `borrowscope-macro/src/visitor.rs`:

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Expr, Stmt, Block, Local, Pat, ItemFn,
};
use quote::quote;

pub struct OwnershipVisitor {
    /// Counter for generating unique IDs
    next_id: usize,
    
    /// Current scope depth
    scope_depth: usize,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
        }
    }
    
    /// Generate next unique ID
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl VisitMut for OwnershipVisitor {
    // We'll implement specific methods next
}
```

---

## Visiting Functions

Override `visit_item_fn_mut` to process function bodies:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        // Visit the function body
        if let Some(block) = &mut func.block.as_mut() {
            self.visit_block_mut(block);
        }
        
        // Don't visit nested items (functions inside functions)
        // visit_mut::visit_item_fn_mut(self, func);
    }
}
```

**Why skip nested items?** Each `#[track_ownership]` attribute applies to one function only.

---

## Visiting Blocks

Blocks define scopes. Track scope depth:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        
        // Visit all statements in the block
        for stmt in &mut block.stmts {
            self.visit_stmt_mut(stmt);
        }
        
        self.scope_depth -= 1;
    }
}
```

---

## Visiting Statements

Statements are the core transformation target:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                // This is a `let` statement - transform it!
                self.transform_local(local);
            }
            Stmt::Expr(expr, _) | Stmt::Semi(expr, _) => {
                // Visit expressions in statements
                self.visit_expr_mut(expr);
            }
            _ => {
                // Use default visitor for other statement types
                visit_mut::visit_stmt_mut(self, stmt);
            }
        }
    }
}
```

---

## Visiting Expressions

Expressions can contain borrows:

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Reference(ref_expr) => {
                // This is a borrow - transform it!
                self.transform_reference(ref_expr);
            }
            _ => {
                // Recursively visit nested expressions
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
```

---

## Transformation Methods

Add helper methods for actual transformations:

```rust
impl OwnershipVisitor {
    /// Transform a let statement
    fn transform_local(&mut self, local: &mut Local) {
        // Extract variable name from pattern
        let var_name = self.extract_pattern_name(&local.pat);
        
        // Generate unique ID
        let id = self.next_id();
        
        // Transform the initializer
        if let Some(init) = &mut local.init {
            let original_expr = &init.expr;
            
            // Wrap with track_new
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #var_name,
                    "unknown",  // Type name - we'll improve this later
                    "unknown",  // Location - we'll improve this later
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
        
        // Continue visiting nested expressions
        visit_mut::visit_local_mut(self, local);
    }
    
    /// Transform a reference expression
    fn transform_reference(&mut self, ref_expr: &mut syn::ExprReference) {
        let id = self.next_id();
        let is_mutable = ref_expr.mutability.is_some();
        
        // We'll implement this fully in the next section
        // For now, just visit the inner expression
        self.visit_expr_mut(&mut ref_expr.expr);
    }
    
    /// Extract variable name from pattern
    fn extract_pattern_name(&self, pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            Pat::Type(pat_type) => self.extract_pattern_name(&pat_type.pat),
            _ => "unknown".to_string(),
        }
    }
}
```

---

## Testing the Visitor

Create `borrowscope-macro/tests/visitor_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_simple_let_transformation() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let x = 42;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    assert!(output.contains("track_new"));
    assert!(output.contains("42"));
}

#[test]
fn test_multiple_variables() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let y = 100;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have two track_new calls with different IDs
    assert!(output.contains("track_new"));
    assert!(output.matches("track_new").count() == 2);
}

#[test]
fn test_nested_blocks() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            {
                let y = 100;
            }
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    assert!(output.matches("track_new").count() == 2);
}
```

Run tests:

```bash
cargo test --package borrowscope-macro visitor_test
```

---

## Integrating with the Macro

Update `borrowscope-macro/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, visit_mut::VisitMut};
use quote::quote;

mod visitor;
use visitor::OwnershipVisitor;

#[proc_macro_attribute]
pub fn track_ownership(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function
    let mut func = parse_macro_input!(item as ItemFn);
    
    // Create visitor and transform
    let mut visitor = OwnershipVisitor::new();
    visitor.visit_item_fn_mut(&mut func);
    
    // Return transformed function
    TokenStream::from(quote! { #func })
}
```

---

## Example Transformation

**Input:**
```rust
#[track_ownership]
fn example() {
    let x = 42;
    let y = 100;
}
```

**Output (formatted):**
```rust
fn example() {
    let x = borrowscope_runtime::track_new(1, "x", "unknown", "unknown", 42);
    let y = borrowscope_runtime::track_new(2, "y", "unknown", "unknown", 100);
}
```

**Progress:** We're wrapping initializers! Next we'll add:
- Type names
- Source locations
- Drop tracking

---

## Visitor Pattern Benefits

### 1. Separation of Concerns

```rust
// Visitor handles traversal
impl VisitMut for OwnershipVisitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) { /* ... */ }
}

// Helper methods handle transformation
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) { /* ... */ }
}
```

### 2. Composability

```rust
// Multiple visitors can be chained
let mut visitor1 = OwnershipVisitor::new();
let mut visitor2 = DropInserter::new();

visitor1.visit_item_fn_mut(&mut func);
visitor2.visit_item_fn_mut(&mut func);
```

### 3. Testability

Each visitor method can be tested independently.

---

## Debugging Tips

### Print AST Structure

```rust
#[test]
fn debug_ast() {
    let stmt: syn::Stmt = parse_quote! {
        let x = 42;
    };
    
    println!("{:#?}", stmt);
}
```

Output:
```
Local(
    Local {
        pat: Ident(PatIdent { ident: Ident { ident: "x" } }),
        init: Some(LocalInit {
            expr: Lit(ExprLit { lit: Int(LitInt { token: 42 }) })
        })
    }
)
```

### Print Transformed Code

```rust
#[test]
fn debug_transformation() {
    let mut visitor = OwnershipVisitor::new();
    let mut stmt: syn::Stmt = parse_quote! { let x = 42; };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    println!("{}", stmt.to_token_stream());
}
```

---

## Common Pitfalls

### Pitfall 1: Infinite Recursion

**Problem:**
```rust
fn visit_expr_mut(&mut self, expr: &mut Expr) {
    self.visit_expr_mut(expr);  // Infinite loop!
}
```

**Solution:** Use `visit_mut::visit_expr_mut(self, expr)` for default behavior.

### Pitfall 2: Forgetting to Visit Children

**Problem:**
```rust
fn visit_block_mut(&mut self, block: &mut Block) {
    // Transform block but don't visit statements
    // Nested expressions won't be transformed!
}
```

**Solution:** Always call visitor methods on children.

### Pitfall 3: Modifying While Iterating

**Problem:**
```rust
for stmt in &mut block.stmts {
    block.stmts.push(new_stmt);  // Modifying while iterating!
}
```

**Solution:** Collect modifications, apply after iteration.

---

## Key Takeaways

✅ **VisitMut enables AST transformation** - Modify nodes in-place  
✅ **Override specific methods** - Only transform what you need  
✅ **Track state** - Use visitor fields for IDs, scope depth  
✅ **Recursive traversal** - Visit children to handle nesting  
✅ **Test incrementally** - Verify each transformation step  

---

## Further Reading

- [syn::visit_mut documentation](https://docs.rs/syn/latest/syn/visit_mut/)
- [Visitor pattern](https://en.wikipedia.org/wiki/Visitor_pattern)
- [AST transformation techniques](https://rustc-dev-guide.rust-lang.org/macro-expansion.html)

---

**Previous:** [36-planning-the-transformation-strategy.md](./36-planning-the-transformation-strategy.md)  
**Next:** [38-injecting-track-new-calls.md](./38-injecting-track-new-calls.md)

**Progress:** 2/15 ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
