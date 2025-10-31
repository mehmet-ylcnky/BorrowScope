# Section 42: Dealing with Patterns

## Learning Objectives

By the end of this section, you will:
- Handle tuple destructuring patterns
- Transform struct patterns
- Process nested patterns recursively
- Track all variables in complex patterns
- Preserve pattern matching semantics

## Prerequisites

- Completed Section 41 (Scope Boundaries)
- Understanding of Rust pattern matching
- Familiarity with destructuring syntax

---

## Pattern Types in Rust

Rust supports many pattern types:

```rust
// Simple identifier
let x = 42;

// Tuple pattern
let (x, y) = (1, 2);

// Struct pattern
let Point { x, y } = point;

// Nested pattern
let ((a, b), c) = ((1, 2), 3);

// Array/slice pattern
let [first, second] = arr;

// Wildcard
let (x, _) = (1, 2);

// Reference pattern
let &x = &42;
```

---

## Strategy

For complex patterns, we need to:
1. Extract all variable bindings
2. Track each variable individually
3. Preserve the original pattern structure

**Approach:** Replace the pattern with a temporary, then destructure:

```rust
// Original
let (x, y) = (1, 2);

// Transformed
let __temp = borrowscope_runtime::track_new(1, "__temp", "inferred", "line:1:9", (1, 2));
let x = borrowscope_runtime::track_new(2, "x", "inferred", "line:1:10", __temp.0);
let y = borrowscope_runtime::track_new(3, "y", "inferred", "line:1:13", __temp.1);
```

---

## Implementation

### Step 1: Detect Complex Patterns

```rust
impl OwnershipVisitor {
    fn is_simple_pattern(&self, pat: &Pat) -> bool {
        matches!(pat, Pat::Ident(_) | Pat::Type(_))
    }
    
    fn is_complex_pattern(&self, pat: &Pat) -> bool {
        matches!(
            pat,
            Pat::Tuple(_) | Pat::Struct(_) | Pat::TupleStruct(_) | 
            Pat::Slice(_) | Pat::Or(_)
        )
    }
}
```

### Step 2: Extract Variables from Patterns

```rust
impl OwnershipVisitor {
    fn extract_pattern_vars(&self, pat: &Pat) -> Vec<String> {
        match pat {
            Pat::Ident(pat_ident) => {
                vec![pat_ident.ident.to_string()]
            }
            Pat::Tuple(pat_tuple) => {
                pat_tuple.elems
                    .iter()
                    .flat_map(|p| self.extract_pattern_vars(p))
                    .collect()
            }
            Pat::Struct(pat_struct) => {
                pat_struct.fields
                    .iter()
                    .flat_map(|field| self.extract_pattern_vars(&field.pat))
                    .collect()
            }
            Pat::Type(pat_type) => {
                self.extract_pattern_vars(&pat_type.pat)
            }
            Pat::Wild(_) => {
                vec![]  // Wildcard doesn't bind
            }
            _ => vec![],
        }
    }
}
```

### Step 3: Transform Complex Patterns

```rust
use syn::{Ident, Index};

impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            if self.is_complex_pattern(&local.pat) {
                self.transform_complex_pattern(local);
            } else {
                self.transform_simple_pattern(local);
            }
        }
    }
    
    fn transform_complex_pattern(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            // Generate temporary variable
            let temp_id = self.next_id();
            let temp_name = format!("__temp_{}", temp_id);
            let location = self.get_location(local.span());
            
            let original_expr = &init.expr;
            let original_pat = &local.pat;
            
            // Track the temporary
            let temp_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #temp_id,
                    #temp_name,
                    "inferred",
                    #location,
                    #original_expr
                )
            };
            
            // Replace pattern with temporary identifier
            let temp_ident: Ident = syn::parse_str(&temp_name).unwrap();
            local.pat = syn::parse_quote! { #temp_ident };
            *init.expr = temp_expr;
            
            // Generate destructuring statements
            let destructure_stmts = self.generate_destructure_stmts(
                original_pat,
                &temp_ident,
                &[]
            );
            
            // Store statements to insert after this one
            for stmt in destructure_stmts {
                self.pending_inserts.push((self.current_stmt_index + 1, stmt));
            }
        }
    }
    
    fn generate_destructure_stmts(
        &mut self,
        pat: &Pat,
        source: &Ident,
        path: &[Index],
    ) -> Vec<Stmt> {
        match pat {
            Pat::Tuple(pat_tuple) => {
                let mut stmts = Vec::new();
                
                for (idx, elem_pat) in pat_tuple.elems.iter().enumerate() {
                    let index = Index::from(idx);
                    let mut new_path = path.to_vec();
                    new_path.push(index.clone());
                    
                    // Generate access expression
                    let access_expr = self.build_access_expr(source, &new_path);
                    
                    // Extract variable name
                    if let Some(var_name) = self.get_simple_ident(elem_pat) {
                        let var_id = self.next_id();
                        self.var_ids.insert(var_name.clone(), var_id);
                        
                        // Add to current scope
                        if let Some(current_scope) = self.scope_stack.last_mut() {
                            current_scope.push(var_id);
                        }
                        
                        let stmt: Stmt = syn::parse_quote! {
                            let #elem_pat = borrowscope_runtime::track_new(
                                #var_id,
                                #var_name,
                                "inferred",
                                "destructure",
                                #access_expr
                            );
                        };
                        
                        stmts.push(stmt);
                    } else {
                        // Nested pattern - recurse
                        let nested_stmts = self.generate_destructure_stmts(
                            elem_pat,
                            source,
                            &new_path
                        );
                        stmts.extend(nested_stmts);
                    }
                }
                
                stmts
            }
            Pat::Struct(pat_struct) => {
                let mut stmts = Vec::new();
                
                for field in &pat_struct.fields {
                    let field_name = match &field.member {
                        syn::Member::Named(ident) => ident.clone(),
                        syn::Member::Unnamed(index) => {
                            syn::parse_str(&format!("_{}", index.index)).unwrap()
                        }
                    };
                    
                    // Generate field access
                    let access_expr: Expr = syn::parse_quote! {
                        #source.#field_name
                    };
                    
                    if let Some(var_name) = self.get_simple_ident(&field.pat) {
                        let var_id = self.next_id();
                        self.var_ids.insert(var_name.clone(), var_id);
                        
                        if let Some(current_scope) = self.scope_stack.last_mut() {
                            current_scope.push(var_id);
                        }
                        
                        let pat = &field.pat;
                        let stmt: Stmt = syn::parse_quote! {
                            let #pat = borrowscope_runtime::track_new(
                                #var_id,
                                #var_name,
                                "inferred",
                                "destructure",
                                #access_expr
                            );
                        };
                        
                        stmts.push(stmt);
                    }
                }
                
                stmts
            }
            _ => vec![],
        }
    }
    
    fn build_access_expr(&self, source: &Ident, path: &[Index]) -> Expr {
        let mut expr: Expr = syn::parse_quote! { #source };
        
        for index in path {
            expr = syn::parse_quote! { #expr.#index };
        }
        
        expr
    }
    
    fn get_simple_ident(&self, pat: &Pat) -> Option<String> {
        match pat {
            Pat::Ident(pat_ident) => Some(pat_ident.ident.to_string()),
            Pat::Type(pat_type) => self.get_simple_ident(&pat_type.pat),
            _ => None,
        }
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/pattern_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_tuple_pattern() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let (x, y) = (1, 2);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have temp variable + two destructured variables
    assert!(output.contains("__temp"));
    assert!(output.contains("\"x\""));
    assert!(output.contains("\"y\""));
}

#[test]
fn test_nested_tuple_pattern() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let ((a, b), c) = ((1, 2), 3);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("\"a\""));
    assert!(output.contains("\"b\""));
    assert!(output.contains("\"c\""));
}

#[test]
fn test_struct_pattern() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let Point { x, y } = point;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("\"x\""));
    assert!(output.contains("\"y\""));
}

#[test]
fn test_wildcard_pattern() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let (x, _) = (1, 2);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should only track x, not the wildcard
    assert!(output.contains("\"x\""));
    assert_eq!(output.matches("track_new").count(), 2);  // temp + x
}
```

---

## Example Transformations

### Example 1: Tuple Pattern

**Input:**
```rust
let (x, y) = (1, 2);
```

**Output:**
```rust
let __temp_1 = borrowscope_runtime::track_new(1, "__temp_1", "inferred", "line:1:9", (1, 2));
let x = borrowscope_runtime::track_new(2, "x", "inferred", "destructure", __temp_1.0);
let y = borrowscope_runtime::track_new(3, "y", "inferred", "destructure", __temp_1.1);
```

### Example 2: Nested Tuple

**Input:**
```rust
let ((a, b), c) = ((1, 2), 3);
```

**Output:**
```rust
let __temp_1 = borrowscope_runtime::track_new(1, "__temp_1", "inferred", "line:1:9", ((1, 2), 3));
let a = borrowscope_runtime::track_new(2, "a", "inferred", "destructure", __temp_1.0.0);
let b = borrowscope_runtime::track_new(3, "b", "inferred", "destructure", __temp_1.0.1);
let c = borrowscope_runtime::track_new(4, "c", "inferred", "destructure", __temp_1.1);
```

### Example 3: Struct Pattern

**Input:**
```rust
struct Point { x: i32, y: i32 }
let Point { x, y } = point;
```

**Output:**
```rust
let __temp_1 = borrowscope_runtime::track_new(1, "__temp_1", "inferred", "line:2:9", point);
let x = borrowscope_runtime::track_new(2, "x", "inferred", "destructure", __temp_1.x);
let y = borrowscope_runtime::track_new(3, "y", "inferred", "destructure", __temp_1.y);
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/pattern_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_tuple_destructure() {
    reset_tracker();
    
    let __temp = track_new(1, "__temp", "inferred", "test.rs:1:1", (42, 100));
    let x = track_new(2, "x", "inferred", "destructure", __temp.0);
    let y = track_new(3, "y", "inferred", "destructure", __temp.1);
    
    assert_eq!(x, 42);
    assert_eq!(y, 100);
    
    let events = get_events();
    assert_eq!(events.len(), 3);  // temp, x, y
}

#[test]
fn test_struct_destructure() {
    #[derive(Debug)]
    struct Point { x: i32, y: i32 }
    
    reset_tracker();
    
    let point = Point { x: 10, y: 20 };
    let __temp = track_new(1, "__temp", "Point", "test.rs:1:1", point);
    let x = track_new(2, "x", "i32", "destructure", __temp.x);
    let y = track_new(3, "y", "i32", "destructure", __temp.y);
    
    assert_eq!(x, 10);
    assert_eq!(y, 20);
    
    let events = get_events();
    assert_eq!(events.len(), 3);
}
```

---

## Edge Cases

### Case 1: Reference Patterns

```rust
let &x = &42;
```

**Solution:** Track the dereferenced value:

```rust
let __temp = borrowscope_runtime::track_new(1, "__temp", "inferred", "line:1:9", &42);
let x = borrowscope_runtime::track_new(2, "x", "inferred", "destructure", *__temp);
```

### Case 2: Mutable Patterns

```rust
let (mut x, y) = (1, 2);
```

**Solution:** Preserve mutability:

```rust
let __temp = borrowscope_runtime::track_new(1, "__temp", "inferred", "line:1:9", (1, 2));
let mut x = borrowscope_runtime::track_new(2, "x", "inferred", "destructure", __temp.0);
let y = borrowscope_runtime::track_new(3, "y", "inferred", "destructure", __temp.1);
```

### Case 3: Ignored Fields

```rust
let Point { x, .. } = point;
```

**Solution:** Only track bound variables:

```rust
let __temp = borrowscope_runtime::track_new(1, "__temp", "Point", "line:1:9", point);
let x = borrowscope_runtime::track_new(2, "x", "inferred", "destructure", __temp.x);
// Don't track other fields
```

### Case 4: Array Patterns

```rust
let [first, second] = [1, 2];
```

**Solution:** Similar to tuples, use indexing:

```rust
let __temp = borrowscope_runtime::track_new(1, "__temp", "inferred", "line:1:9", [1, 2]);
let first = borrowscope_runtime::track_new(2, "first", "inferred", "destructure", __temp[0]);
let second = borrowscope_runtime::track_new(3, "second", "inferred", "destructure", __temp[1]);
```

---

## Optimization: Avoid Temporary for Copy Types

For Copy types, we can avoid the temporary:

```rust
// Original
let (x, y) = (1, 2);

// Optimized (if we know it's Copy)
let (x, y) = (
    borrowscope_runtime::track_new(1, "x", "i32", "line:1:10", 1),
    borrowscope_runtime::track_new(2, "y", "i32", "line:1:13", 2)
);
```

**Implementation:**

```rust
impl OwnershipVisitor {
    fn can_inline_pattern(&self, pat: &Pat, init_expr: &Expr) -> bool {
        // Check if all elements are simple literals
        if let Expr::Tuple(tuple_expr) = init_expr {
            tuple_expr.elems.iter().all(|e| matches!(e, Expr::Lit(_)))
        } else {
            false
        }
    }
    
    fn transform_inline_pattern(&mut self, local: &mut Local) {
        // Transform each element in place
        // More complex, but avoids temporary variable
    }
}
```

---

## Key Takeaways

✅ **Use temporary variables** - Simplifies complex patterns  
✅ **Extract all bindings** - Recursively process nested patterns  
✅ **Preserve structure** - Maintain original pattern semantics  
✅ **Handle wildcards** - Don't track ignored bindings  
✅ **Support all pattern types** - Tuples, structs, arrays, references  

---

## Further Reading

- [Rust patterns](https://doc.rust-lang.org/book/ch18-00-patterns.html)
- [Pattern syntax](https://doc.rust-lang.org/reference/patterns.html)
- [Destructuring](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring.html)

---

**Previous:** [41-handling-scope-boundaries.md](./41-handling-scope-boundaries.md)  
**Next:** [43-handling-control-flow.md](./43-handling-control-flow.md)

**Progress:** 7/15 ⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜
