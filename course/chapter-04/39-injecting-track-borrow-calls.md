# Section 39: Injecting track_borrow Calls

## Learning Objectives

By the end of this section, you will:
- Detect borrow expressions in AST
- Transform & and &mut references
- Track borrowed variable IDs
- Handle nested borrows
- Preserve reference semantics

## Prerequisites

- Completed Section 38 (track_new injection)
- Understanding of Rust borrowing rules
- Familiarity with reference expressions

---

## The Transformation Goal

Transform borrows:

```rust
let x = 42;
let r = &x;
let m = &mut x;
```

Into:

```rust
let x = track_new(1, "x", "i32", "line:1:9", 42);
let r = track_borrow(2, 1, false, "line:2:9", &x);
let m = track_borrow_mut(3, 1, true, "line:3:9", &mut x);
```

---

## Detecting Reference Expressions

References are represented as `ExprReference` in syn:

```rust
pub struct ExprReference {
    pub and_token: Token![&],
    pub mutability: Option<Token![mut]>,
    pub expr: Box<Expr>,
}
```

**Example AST:**
```rust
&x  // ExprReference { mutability: None, expr: Path("x") }
&mut x  // ExprReference { mutability: Some(_), expr: Path("x") }
```

---

## Implementation

Update `borrowscope-macro/src/visitor.rs`:

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Expr, ExprReference, ExprPath,
};

impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Reference(ref_expr) => {
                self.transform_reference(expr, ref_expr);
            }
            _ => {
                // Recursively visit nested expressions
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

impl OwnershipVisitor {
    fn transform_reference(&mut self, expr: &mut Expr, ref_expr: &ExprReference) {
        let borrow_id = self.next_id();
        let is_mutable = ref_expr.mutability.is_some();
        let location = self.get_location(ref_expr.span());
        
        // Try to extract the borrowed variable ID
        let borrowed_id = self.extract_borrowed_id(&ref_expr.expr);
        
        // Clone the original reference expression
        let original_ref = expr.clone();
        
        // Generate tracking call
        let tracking_call: Expr = if is_mutable {
            syn::parse_quote! {
                borrowscope_runtime::track_borrow_mut(
                    #borrow_id,
                    #borrowed_id,
                    true,
                    #location,
                    #original_ref
                )
            }
        } else {
            syn::parse_quote! {
                borrowscope_runtime::track_borrow(
                    #borrow_id,
                    #borrowed_id,
                    false,
                    #location,
                    #original_ref
                )
            }
        };
        
        *expr = tracking_call;
    }
    
    fn extract_borrowed_id(&self, expr: &Expr) -> usize {
        // For now, return 0 (unknown)
        // We'll implement proper ID tracking in the next section
        0
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/track_borrow_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_immutable_borrow() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let r = &x;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));  // For 'r'
    assert!(output.contains("track_borrow"));  // For '&x'
    assert!(output.contains("false"));  // is_mutable = false
}

#[test]
fn test_mutable_borrow() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let r = &mut x;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_borrow_mut"));
    assert!(output.contains("true"));  // is_mutable = true
}

#[test]
fn test_borrow_in_expression() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        foo(&x);
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_borrow"));
}

#[test]
fn test_multiple_borrows() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let r1 = &x;
            let r2 = &x;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have two track_borrow calls
    assert_eq!(output.matches("track_borrow").count(), 2);
}
```

---

## Handling Nested Borrows

### Case 1: Borrow of Borrow

```rust
let x = 42;
let r1 = &x;
let r2 = &r1;  // Borrow of a borrow
```

**Transformation:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", 42);
let r1 = track_borrow(2, 1, false, "line:2:10", &x);
let r2 = track_borrow(3, 2, false, "line:3:10", &r1);
```

**Implementation:** Already works! The visitor recursively processes nested expressions.

### Case 2: Borrow in Complex Expression

```rust
let x = 42;
let y = 100;
let sum = *&x + *&y;
```

**Transformation:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", 42);
let y = track_new(2, "y", "i32", "line:2:9", 100);
let sum = track_new(
    3,
    "sum",
    "inferred",
    "line:3:9",
    *track_borrow(4, 1, false, "line:3:12", &x) + 
    *track_borrow(5, 2, false, "line:3:18", &y)
);
```

---

## Tracking Borrowed Variable IDs

To properly track which variable is borrowed, we need a symbol table.

Add to `borrowscope-macro/src/visitor.rs`:

```rust
use std::collections::HashMap;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
    /// Map variable names to their IDs
    var_ids: HashMap<String, usize>,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
            var_ids: HashMap::new(),
        }
    }
    
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            
            // Store variable ID
            self.var_ids.insert(var_name.clone(), id);
            
            // ... rest of transformation
        }
    }
    
    fn extract_borrowed_id(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Path(expr_path) => {
                // Extract variable name from path
                if let Some(ident) = expr_path.path.get_ident() {
                    let var_name = ident.to_string();
                    return *self.var_ids.get(&var_name).unwrap_or(&0);
                }
            }
            _ => {}
        }
        0  // Unknown
    }
}
```

---

## Testing with ID Tracking

```rust
#[test]
fn test_borrow_id_tracking() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let r = &x;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // First track_new should have ID 1
    assert!(output.contains("track_new (1"));
    
    // track_borrow should reference ID 1
    assert!(output.contains("track_borrow (2 , 1"));
}
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/borrow_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_immutable_borrow() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    
    assert_eq!(*r, 42);
    
    let events = get_events();
    assert_eq!(events.len(), 2);
    
    match &events[1] {
        Event::Borrow { id, borrowed_id, is_mutable, .. } => {
            assert_eq!(*id, 2);
            assert_eq!(*borrowed_id, 1);
            assert_eq!(*is_mutable, false);
        }
        _ => panic!("Expected Borrow event"),
    }
}

#[test]
fn test_mutable_borrow() {
    reset_tracker();
    
    let mut x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow_mut(2, 1, true, "test.rs:2:1", &mut x);
    
    *r = 100;
    assert_eq!(*r, 100);
    
    let events = get_events();
    
    match &events[1] {
        Event::Borrow { is_mutable, .. } => {
            assert_eq!(*is_mutable, true);
        }
        _ => panic!("Expected Borrow event"),
    }
}
```

---

## Edge Cases

### Case 1: Borrow of Field

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 10, y: 20 };
let r = &p.x;
```

**Challenge:** `p.x` is not a simple variable.

**Solution:** Track the struct `p`, not the field:

```rust
let r = track_borrow(2, 1, false, "line:3:9", &p.x);
//                         ^
//                         ID of 'p'
```

### Case 2: Borrow of Array Element

```rust
let arr = [1, 2, 3];
let r = &arr[0];
```

**Solution:** Track the array:

```rust
let r = track_borrow(2, 1, false, "line:2:9", &arr[0]);
```

### Case 3: Temporary Borrow

```rust
foo(&42);  // Borrow of temporary
```

**Solution:** Don't track (no variable to reference):

```rust
fn transform_reference(&mut self, expr: &mut Expr, ref_expr: &ExprReference) {
    // Check if borrowing a variable
    if !self.is_variable_borrow(&ref_expr.expr) {
        return;  // Skip temporaries
    }
    
    // ... rest of transformation
}

fn is_variable_borrow(&self, expr: &Expr) -> bool {
    matches!(expr, Expr::Path(_))
}
```

---

## Example Output

**Input:**
```rust
#[track_ownership]
fn example() {
    let x = 42;
    let r1 = &x;
    let r2 = &x;
}
```

**Output:**
```rust
fn example() {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "line:3:9", 42);
    let r1 = borrowscope_runtime::track_new(
        2,
        "r1",
        "inferred",
        "line:4:9",
        borrowscope_runtime::track_borrow(3, 1, false, "line:4:14", &x)
    );
    let r2 = borrowscope_runtime::track_new(
        4,
        "r2",
        "inferred",
        "line:5:9",
        borrowscope_runtime::track_borrow(5, 1, false, "line:5:14", &x)
    );
}
```

**Note:** Both `r1` and `r2` get their own IDs (2 and 4), and their borrows get IDs (3 and 5).

---

## Key Takeaways

✅ **Detect ExprReference** - Match on Expr::Reference  
✅ **Track mutability** - Check ref_expr.mutability  
✅ **Maintain symbol table** - Map variable names to IDs  
✅ **Handle nested borrows** - Recursive visitor handles this  
✅ **Skip temporaries** - Only track variable borrows  

---

## Further Reading

- [Rust borrowing rules](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [syn::ExprReference](https://docs.rs/syn/latest/syn/struct.ExprReference.html)
- [Reference expressions](https://doc.rust-lang.org/reference/expressions/operator-expr.html#borrow-operators)

---

**Previous:** [38-injecting-track-new-calls.md](./38-injecting-track-new-calls.md)  
**Next:** [40-injecting-track-move-calls.md](./40-injecting-track-move-calls.md)

**Progress:** 4/15 ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
