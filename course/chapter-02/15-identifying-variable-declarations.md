# Section 15: Identifying Variable Declarations

## Learning Objectives

By the end of this section, you will:
- Handle complex variable patterns
- Track tuple destructuring
- Support struct patterns
- Handle pattern matching in let statements
- Implement pattern analysis utilities
- Extend the visitor for complex cases

## Prerequisites

- Completed Section 14
- Understanding of Rust patterns
- Familiarity with pattern matching

---

## Rust Pattern Types

### Simple Patterns

```rust
let x = 5;              // Pat::Ident
let mut x = 5;          // Pat::Ident with mutability
let _ = 5;              // Pat::Wild (wildcard)
```

### Tuple Patterns

```rust
let (x, y) = (1, 2);                    // Pat::Tuple
let (a, b, c) = (1, 2, 3);              // Pat::Tuple
let (first, .., last) = (1, 2, 3, 4);   // Pat::Tuple with rest
```

### Struct Patterns

```rust
let Point { x, y } = point;             // Pat::Struct
let Point { x: a, y: b } = point;       // Pat::Struct with rename
let Point { x, .. } = point;            // Pat::Struct with rest
```

### Tuple Struct Patterns

```rust
let Some(x) = option;                   // Pat::TupleStruct
let Ok(value) = result;                 // Pat::TupleStruct
```

---

## Step 1: Pattern Analysis Utilities

### File: `borrowscope-macro/src/pattern.rs`

```rust
//! Pattern analysis utilities

use syn::{Pat, PatIdent, PatTuple, PatStruct, Ident};

/// Information extracted from a pattern
#[derive(Debug, Clone)]
pub struct PatternInfo {
    /// Variable names bound by this pattern
    pub variables: Vec<Ident>,
    
    /// Whether the pattern is simple (single identifier)
    pub is_simple: bool,
    
    /// Whether the pattern contains wildcards
    pub has_wildcards: bool,
}

impl PatternInfo {
    /// Analyze a pattern and extract information
    pub fn analyze(pat: &Pat) -> Self {
        let mut info = PatternInfo {
            variables: Vec::new(),
            is_simple: false,
            has_wildcards: false,
        };
        
        extract_variables(pat, &mut info);
        
        info.is_simple = info.variables.len() == 1 && !info.has_wildcards;
        
        info
    }
    
    /// Check if pattern should be tracked
    pub fn is_trackable(&self) -> bool {
        // Must have at least one variable
        !self.variables.is_empty() &&
        // No wildcards (we can't track _)
        !self.has_wildcards
    }
}

/// Extract all variable names from a pattern
fn extract_variables(pat: &Pat, info: &mut PatternInfo) {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => {
            // Skip underscore variables
            if !ident.to_string().starts_with('_') {
                info.variables.push(ident.clone());
            }
        }
        
        Pat::Tuple(PatTuple { elems, .. }) => {
            for elem in elems {
                extract_variables(elem, info);
            }
        }
        
        Pat::Struct(PatStruct { fields, .. }) => {
            for field in fields {
                extract_variables(&field.pat, info);
            }
        }
        
        Pat::TupleStruct(tuple_struct) => {
            for elem in &tuple_struct.elems {
                extract_variables(elem, info);
            }
        }
        
        Pat::Wild(_) => {
            info.has_wildcards = true;
        }
        
        Pat::Or(or_pat) => {
            // For or patterns, extract from first case
            if let Some(first) = or_pat.cases.first() {
                extract_variables(first, info);
            }
        }
        
        Pat::Paren(paren) => {
            extract_variables(&paren.pat, info);
        }
        
        Pat::Reference(reference) => {
            extract_variables(&reference.pat, info);
        }
        
        _ => {
            // Other pattern types not yet supported
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_pattern() {
        let pat: Pat = parse_quote!(x);
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 1);
        assert_eq!(info.variables[0].to_string(), "x");
        assert!(info.is_simple);
        assert!(!info.has_wildcards);
        assert!(info.is_trackable());
    }

    #[test]
    fn test_tuple_pattern() {
        let pat: Pat = parse_quote!((x, y));
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 2);
        assert!(!info.is_simple);
        assert!(info.is_trackable());
    }

    #[test]
    fn test_struct_pattern() {
        let pat: Pat = parse_quote!(Point { x, y });
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 2);
        assert!(!info.is_simple);
        assert!(info.is_trackable());
    }

    #[test]
    fn test_wildcard_pattern() {
        let pat: Pat = parse_quote!(_);
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 0);
        assert!(info.has_wildcards);
        assert!(!info.is_trackable());
    }

    #[test]
    fn test_underscore_variable() {
        let pat: Pat = parse_quote!(_unused);
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 0);
    }

    #[test]
    fn test_nested_pattern() {
        let pat: Pat = parse_quote!((x, (y, z)));
        let info = PatternInfo::analyze(&pat);
        
        assert_eq!(info.variables.len(), 3);
    }
}
```

---

## Step 2: Transform Complex Patterns

### File: `borrowscope-macro/src/pattern_transform.rs`

```rust
//! Pattern transformation for complex cases

use syn::{Local, Pat, Expr, Ident};
use quote::quote;
use crate::pattern::PatternInfo;

/// Transform a let statement with complex pattern
///
/// For complex patterns, we need to:
/// 1. Track the initialization expression
/// 2. Track each variable bound by the pattern
pub fn transform_complex_let(
    local: &Local,
    pattern_info: &PatternInfo,
) -> Option<Vec<syn::Stmt>> {
    if pattern_info.variables.is_empty() {
        return None;
    }
    
    let init = local.init.as_ref()?;
    let init_expr = &init.expr;
    let pat = &local.pat;
    
    // For complex patterns, we:
    // 1. Create a temporary variable for the initialization
    // 2. Track the temporary
    // 3. Destructure into the pattern
    // 4. Track each variable
    
    let temp_name = syn::Ident::new("__borrowscope_temp", proc_macro2::Span::call_site());
    
    let mut stmts = Vec::new();
    
    // Step 1: Track the initialization
    let track_init: syn::Stmt = syn::parse_quote! {
        let #temp_name = borrowscope_runtime::track_new(
            stringify!(#temp_name),
            #init_expr
        );
    };
    stmts.push(track_init);
    
    // Step 2: Destructure
    let destructure: syn::Stmt = syn::parse_quote! {
        let #pat = #temp_name;
    };
    stmts.push(destructure);
    
    // Step 3: Track each variable
    for var in &pattern_info.variables {
        let track_var: syn::Stmt = syn::parse_quote! {
            let #var = borrowscope_runtime::track_new(
                stringify!(#var),
                #var
            );
        };
        stmts.push(track_var);
    }
    
    Some(stmts)
}

/// Generate tracking for tuple destructuring
///
/// # Example
///
/// ```ignore
/// // Input:
/// let (x, y) = (1, 2);
///
/// // Output:
/// let __temp = track_new("__temp", (1, 2));
/// let (x, y) = __temp;
/// let x = track_new("x", x);
/// let y = track_new("y", y);
/// ```
pub fn transform_tuple_pattern(
    pat: &Pat,
    init_expr: &Expr,
    variables: &[Ident],
) -> Vec<syn::Stmt> {
    let temp = syn::Ident::new("__borrowscope_tuple", proc_macro2::Span::call_site());
    
    let mut stmts = Vec::new();
    
    // Track initialization
    stmts.push(syn::parse_quote! {
        let #temp = borrowscope_runtime::track_new(
            stringify!(#temp),
            #init_expr
        );
    });
    
    // Destructure
    stmts.push(syn::parse_quote! {
        let #pat = #temp;
    });
    
    // Track each variable
    for var in variables {
        stmts.push(syn::parse_quote! {
            let #var = borrowscope_runtime::track_new(
                stringify!(#var),
                #var
            );
        });
    }
    
    stmts
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_transform_tuple_pattern() {
        let pat: Pat = parse_quote!((x, y));
        let expr: Expr = parse_quote!((1, 2));
        let vars = vec![
            parse_quote!(x),
            parse_quote!(y),
        ];
        
        let stmts = transform_tuple_pattern(&pat, &expr, &vars);
        
        assert_eq!(stmts.len(), 4); // temp + destructure + 2 vars
        
        let result = quote::quote! { #(#stmts)* }.to_string();
        assert!(result.contains("__borrowscope_tuple"));
        assert!(result.contains("track_new"));
    }

    #[test]
    fn test_transform_complex_let() {
        let local: Local = parse_quote! {
            let (x, y) = (1, 2);
        };
        
        let info = PatternInfo::analyze(&local.pat);
        let stmts = transform_complex_let(&local, &info);
        
        assert!(stmts.is_some());
        let stmts = stmts.unwrap();
        assert!(stmts.len() >= 3);
    }
}
```

---

## Step 3: Update Visitor for Complex Patterns

### File: `borrowscope-macro/src/visitor.rs` (Updated)

```rust
//! AST visitor for transforming borrow patterns

use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Local, Stmt, Pat, Block};
use quote::quote;
use crate::context::TransformContext;
use crate::pattern::{PatternInfo};
use crate::pattern_transform::{transform_complex_let};

/// Visitor that transforms ownership operations
pub struct BorrowVisitor<'ctx> {
    /// Transformation context
    context: &'ctx mut TransformContext,
    
    /// Variables to track for drops
    tracked_variables: Vec<syn::Ident>,
    
    /// Statements to insert (for complex patterns)
    pending_stmts: Vec<Stmt>,
}

impl<'ctx> BorrowVisitor<'ctx> {
    /// Create a new visitor
    pub fn new(context: &'ctx mut TransformContext) -> Self {
        Self {
            context,
            tracked_variables: Vec::new(),
            pending_stmts: Vec::new(),
        }
    }
    
    /// Get the list of tracked variables
    pub fn tracked_variables(&self) -> &[syn::Ident] {
        &self.tracked_variables
    }
    
    /// Transform a let statement
    fn transform_let(&mut self, local: &mut Local) -> Option<Vec<Stmt>> {
        // Analyze the pattern
        let pattern_info = PatternInfo::analyze(&local.pat);
        
        // Skip if not trackable
        if !pattern_info.is_trackable() {
            return None;
        }
        
        // Track all variables
        self.tracked_variables.extend(pattern_info.variables.clone());
        
        // Must have initializer
        let init = local.init.as_ref()?;
        
        // Handle simple patterns (single identifier)
        if pattern_info.is_simple {
            let var_name = &pattern_info.variables[0];
            let init_expr = &init.expr;
            
            // Check if it's a borrow
            if let Expr::Reference(reference) = init_expr.as_ref() {
                let is_mutable = reference.mutability.is_some();
                let borrowed_expr = &reference.expr;
                
                let new_init = if is_mutable {
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow_mut(
                            stringify!(#var_name),
                            &mut #borrowed_expr
                        )
                    }
                } else {
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow(
                            stringify!(#var_name),
                            &#borrowed_expr
                        )
                    }
                };
                
                local.init = Some(syn::LocalInit {
                    eq_token: init.eq_token,
                    expr: Box::new(new_init),
                    diverge: None,
                });
            } else {
                // Regular initialization
                let new_init = syn::parse_quote! {
                    borrowscope_runtime::track_new(
                        stringify!(#var_name),
                        #init_expr
                    )
                };
                
                local.init = Some(syn::LocalInit {
                    eq_token: init.eq_token,
                    expr: Box::new(new_init),
                    diverge: None,
                });
            }
            
            None
        } else {
            // Complex pattern - return replacement statements
            transform_complex_let(local, &pattern_info)
        }
    }
}

impl<'ctx> VisitMut for BorrowVisitor<'ctx> {
    /// Visit a block
    fn visit_block_mut(&mut self, block: &mut Block) {
        let mut new_stmts = Vec::new();
        
        for mut stmt in block.stmts.drain(..) {
            // Check if it's a local statement
            if let Stmt::Local(ref mut local) = stmt {
                if let Some(replacement_stmts) = self.transform_let(local) {
                    // Complex pattern - use replacement statements
                    new_stmts.extend(replacement_stmts);
                    continue;
                }
            }
            
            // Visit the statement
            self.visit_stmt_mut(&mut stmt);
            new_stmts.push(stmt);
        }
        
        block.stmts = new_stmts;
    }
    
    /// Visit a statement
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        visit_mut::visit_stmt_mut(self, stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use crate::options::TraceBorrowOptions;
    use crate::metadata::FunctionMetadata;

    #[test]
    fn test_simple_pattern() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local: Local = parse_quote! { let x = 5; };
        let result = visitor.transform_let(&mut local);
        
        assert!(result.is_none()); // Simple pattern modifies in place
        assert_eq!(visitor.tracked_variables().len(), 1);
    }

    #[test]
    fn test_tuple_pattern() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local: Local = parse_quote! { let (x, y) = (1, 2); };
        let result = visitor.transform_let(&mut local);
        
        assert!(result.is_some()); // Complex pattern returns statements
        let stmts = result.unwrap();
        assert!(stmts.len() >= 3);
        assert_eq!(visitor.tracked_variables().len(), 2);
    }

    #[test]
    fn test_struct_pattern() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local: Local = parse_quote! {
            let Point { x, y } = point;
        };
        let result = visitor.transform_let(&mut local);
        
        assert!(result.is_some());
        assert_eq!(visitor.tracked_variables().len(), 2);
    }
}
```

---

## Step 4: Integration Tests

### File: `borrowscope-macro/tests/complex_patterns.rs`

```rust
//! Tests for complex pattern handling

use borrowscope_macro::trace_borrow;

#[test]
fn test_tuple_destructuring() {
    #[trace_borrow]
    fn example() {
        let (x, y) = (1, 2);
        println!("{} {}", x, y);
    }
    
    example();
}

#[test]
fn test_nested_tuple() {
    #[trace_borrow]
    fn example() {
        let (a, (b, c)) = (1, (2, 3));
        println!("{} {} {}", a, b, c);
    }
    
    example();
}

#[test]
fn test_struct_destructuring() {
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }
    
    #[trace_borrow]
    fn example() {
        let point = Point { x: 1, y: 2 };
        let Point { x, y } = point;
        println!("{} {}", x, y);
    }
    
    example();
}

#[test]
fn test_mixed_patterns() {
    #[trace_borrow]
    fn example() {
        let x = 5;              // Simple
        let (y, z) = (10, 15);  // Tuple
        let sum = x + y + z;
        println!("{}", sum);
    }
    
    example();
}

#[test]
fn test_wildcard_in_tuple() {
    #[trace_borrow]
    fn example() {
        let (x, _) = (1, 2);
        println!("{}", x);
    }
    
    example();
}
```

---

## Step 5: Handle Edge Cases

### File: `borrowscope-macro/src/pattern_edge_cases.rs`

```rust
//! Edge case handling for patterns

use syn::{Pat, Local};

/// Check if pattern contains only wildcards
pub fn is_all_wildcards(pat: &Pat) -> bool {
    match pat {
        Pat::Wild(_) => true,
        Pat::Tuple(tuple) => {
            tuple.elems.iter().all(is_all_wildcards)
        }
        _ => false,
    }
}

/// Check if pattern is a simple underscore variable
pub fn is_underscore_var(pat: &Pat) -> bool {
    if let Pat::Ident(ident) = pat {
        ident.ident.to_string().starts_with('_')
    } else {
        false
    }
}

/// Check if local should be completely skipped
pub fn should_skip_local(local: &Local) -> bool {
    // Skip if no initializer
    if local.init.is_none() {
        return true;
    }
    
    // Skip if all wildcards
    if is_all_wildcards(&local.pat) {
        return true;
    }
    
    // Skip if underscore variable
    if is_underscore_var(&local.pat) {
        return true;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_all_wildcards() {
        let pat1: Pat = parse_quote!(_);
        assert!(is_all_wildcards(&pat1));
        
        let pat2: Pat = parse_quote!((_, _));
        assert!(is_all_wildcards(&pat2));
        
        let pat3: Pat = parse_quote!((x, _));
        assert!(!is_all_wildcards(&pat3));
    }

    #[test]
    fn test_is_underscore_var() {
        let pat1: Pat = parse_quote!(_unused);
        assert!(is_underscore_var(&pat1));
        
        let pat2: Pat = parse_quote!(x);
        assert!(!is_underscore_var(&pat2));
    }

    #[test]
    fn test_should_skip_local() {
        let local1: Local = parse_quote! { let x; };
        assert!(should_skip_local(&local1));
        
        let local2: Local = parse_quote! { let _ = 5; };
        assert!(should_skip_local(&local2));
        
        let local3: Local = parse_quote! { let x = 5; };
        assert!(!should_skip_local(&local3));
    }
}
```

---

## Step 6: Update Module Structure

### File: `borrowscope-macro/src/lib.rs` (Add modules)

```rust
//! Procedural macros for BorrowScope

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

mod options;
mod validate;
mod metadata;
mod context;
mod visitor;
mod transform;
mod utils;
mod attr_utils;
mod pattern;              // NEW
mod pattern_transform;    // NEW
mod pattern_edge_cases;   // NEW

use options::TraceBorrowOptions;
use validate::validate_function;
use metadata::FunctionMetadata;
use context::TransformContext;
use visitor::BorrowVisitor;
use transform::insert_drop_calls;

// ... rest of the code remains the same
```

---

## Step 7: Comprehensive Tests

### File: `borrowscope-macro/tests/pattern_edge_cases.rs`

```rust
//! Edge case tests for patterns

use borrowscope_macro::trace_borrow;

#[test]
fn test_all_wildcards() {
    #[trace_borrow]
    fn example() {
        let _ = 5;
        let (_, _) = (1, 2);
    }
    
    example();
}

#[test]
fn test_partial_wildcards() {
    #[trace_borrow]
    fn example() {
        let (x, _) = (1, 2);
        let (_, y) = (3, 4);
        println!("{} {}", x, y);
    }
    
    example();
}

#[test]
fn test_nested_struct() {
    #[derive(Debug)]
    struct Inner {
        value: i32,
    }
    
    #[derive(Debug)]
    struct Outer {
        inner: Inner,
    }
    
    #[trace_borrow]
    fn example() {
        let outer = Outer {
            inner: Inner { value: 42 },
        };
        let Outer { inner: Inner { value } } = outer;
        println!("{}", value);
    }
    
    example();
}

#[test]
fn test_or_pattern() {
    #[trace_borrow]
    fn example() {
        let x = Some(5);
        if let Some(value) = x {
            println!("{}", value);
        }
    }
    
    example();
}
```

---

## Step 8: Documentation

### Add to `borrowscope-macro/README.md`

```markdown
# Pattern Support

## Supported Patterns

### Simple Patterns
```rust
#[trace_borrow]
fn example() {
    let x = 5;          // ✅ Supported
    let mut x = 5;      // ✅ Supported
}
```

### Tuple Patterns
```rust
#[trace_borrow]
fn example() {
    let (x, y) = (1, 2);              // ✅ Supported
    let (a, (b, c)) = (1, (2, 3));    // ✅ Supported
    let (x, _) = (1, 2);              // ✅ Supported
}
```

### Struct Patterns
```rust
#[trace_borrow]
fn example() {
    let Point { x, y } = point;       // ✅ Supported
    let Point { x: a, y: b } = point; // ✅ Supported
}
```

### Not Yet Supported
```rust
let [a, b, c] = array;      // ❌ Array patterns
let x @ 1..=5 = value;      // ❌ At patterns with ranges
```
```

---

## Key Takeaways

### Pattern Handling

✅ **Simple patterns** - Single identifier, most common  
✅ **Tuple patterns** - Destructuring tuples  
✅ **Struct patterns** - Destructuring structs  
✅ **Nested patterns** - Recursive handling  
✅ **Wildcards** - Skip tracking for _  

### Implementation Strategy

✅ **Pattern analysis** - Extract variables first  
✅ **Simple vs complex** - Different transformation strategies  
✅ **Temporary variables** - For complex patterns  
✅ **Statement replacement** - Return multiple statements  
✅ **Edge case handling** - Skip wildcards, underscores  

### Testing

✅ **Unit tests** - Test pattern analysis  
✅ **Integration tests** - Test with real code  
✅ **Edge cases** - Test wildcards, nesting  
✅ **Comprehensive coverage** - All pattern types  

---

## Exercises

### Exercise 1: Array Patterns

Add support for array patterns:
```rust
let [a, b, c] = [1, 2, 3];
```

### Exercise 2: Slice Patterns

Add support for slice patterns:
```rust
let [first, .., last] = &array[..];
```

### Exercise 3: At Patterns

Add support for at patterns:
```rust
let x @ 1..=5 = value;
```

---

## What's Next?

In **Section 16: Identifying Borrow Expressions**, we'll:
- Find borrows in all contexts
- Handle borrows in expressions
- Track borrows in function arguments
- Handle method call receivers
- Support complex borrow patterns

---

**Previous Section:** [14-implementing-basic-attribute-macro.md](./14-implementing-basic-attribute-macro.md)  
**Next Section:** [16-identifying-borrow-expressions.md](./16-identifying-borrow-expressions.md)

**Chapter Progress:** 7/12 sections complete ⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜

---

*"Patterns are everywhere in Rust. Master them, and you master the language." 🎯*
