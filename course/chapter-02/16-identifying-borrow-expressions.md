# Section 16: Identifying Borrow Expressions

## Learning Objectives

By the end of this section, you will:
- Find borrows in all expression contexts
- Handle borrows in function arguments
- Track borrows in method calls
- Support borrows in complex expressions
- Implement comprehensive borrow detection
- Handle nested and chained borrows

## Prerequisites

- Completed Section 15
- Understanding of Rust expressions
- Familiarity with expression contexts

---

## Borrow Contexts in Rust

### Let Statement Initialization

```rust
let r = &x;           // Already handled
let r = &mut x;       // Already handled
```

### Function Arguments

```rust
foo(&x);              // Need to handle
bar(&mut x);          // Need to handle
baz(&x, &y);          // Multiple borrows
```

### Method Calls

```rust
x.method(&y);         // Borrow as argument
(&x).method();        // Borrow as receiver
```

### Return Expressions

```rust
return &x;            // Borrow in return
&x                    // Implicit return
```

### Complex Expressions

```rust
vec![&x, &y];         // Borrows in collection
Some(&x);             // Borrow in constructor
if cond { &x } else { &y }  // Borrows in branches
```

---

## Step 1: Borrow Detection Utilities

### File: `borrowscope-macro/src/borrow_detection.rs`

```rust
//! Borrow detection utilities

use syn::{Expr, ExprReference, ExprCall, ExprMethodCall};

/// Information about a borrow expression
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    /// Is this a mutable borrow?
    pub is_mutable: bool,
    
    /// The expression being borrowed
    pub borrowed_expr: Box<Expr>,
    
    /// Context where the borrow occurs
    pub context: BorrowContext,
}

/// Context where a borrow occurs
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowContext {
    /// In a let statement initialization
    LetInit,
    
    /// As a function argument
    FunctionArg,
    
    /// As a method argument
    MethodArg,
    
    /// In a return expression
    Return,
    
    /// In another expression
    Other,
}

impl BorrowInfo {
    /// Extract borrow information from an expression
    pub fn from_expr(expr: &Expr, context: BorrowContext) -> Option<Self> {
        if let Expr::Reference(reference) = expr {
            Some(BorrowInfo {
                is_mutable: reference.mutability.is_some(),
                borrowed_expr: reference.expr.clone(),
                context,
            })
        } else {
            None
        }
    }
}

/// Find all borrows in an expression
pub fn find_borrows_in_expr(expr: &Expr) -> Vec<BorrowInfo> {
    let mut borrows = Vec::new();
    collect_borrows(expr, &mut borrows, BorrowContext::Other);
    borrows
}

/// Recursively collect borrows from an expression
fn collect_borrows(expr: &Expr, borrows: &mut Vec<BorrowInfo>, context: BorrowContext) {
    match expr {
        Expr::Reference(reference) => {
            borrows.push(BorrowInfo {
                is_mutable: reference.mutability.is_some(),
                borrowed_expr: reference.expr.clone(),
                context: context.clone(),
            });
            
            // Continue searching in borrowed expression
            collect_borrows(&reference.expr, borrows, context);
        }
        
        Expr::Call(call) => {
            // Check function arguments
            for arg in &call.args {
                collect_borrows(arg, borrows, BorrowContext::FunctionArg);
            }
            
            // Check function expression
            collect_borrows(&call.func, borrows, context);
        }
        
        Expr::MethodCall(method) => {
            // Check receiver
            collect_borrows(&method.receiver, borrows, context.clone());
            
            // Check arguments
            for arg in &method.args {
                collect_borrows(arg, borrows, BorrowContext::MethodArg);
            }
        }
        
        Expr::Return(ret) => {
            if let Some(value) = &ret.expr {
                collect_borrows(value, borrows, BorrowContext::Return);
            }
        }
        
        Expr::Array(array) => {
            for elem in &array.elems {
                collect_borrows(elem, borrows, context.clone());
            }
        }
        
        Expr::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_borrows(elem, borrows, context.clone());
            }
        }
        
        Expr::Binary(binary) => {
            collect_borrows(&binary.left, borrows, context.clone());
            collect_borrows(&binary.right, borrows, context);
        }
        
        Expr::Unary(unary) => {
            collect_borrows(&unary.expr, borrows, context);
        }
        
        Expr::If(if_expr) => {
            collect_borrows(&if_expr.cond, borrows, context.clone());
            
            for stmt in &if_expr.then_branch.stmts {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    collect_borrows(expr, borrows, context.clone());
                }
            }
            
            if let Some((_, else_branch)) = &if_expr.else_branch {
                collect_borrows(else_branch, borrows, context);
            }
        }
        
        Expr::Match(match_expr) => {
            collect_borrows(&match_expr.expr, borrows, context.clone());
            
            for arm in &match_expr.arms {
                collect_borrows(&arm.body, borrows, context.clone());
            }
        }
        
        Expr::Block(block) => {
            for stmt in &block.block.stmts {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    collect_borrows(expr, borrows, context.clone());
                }
            }
        }
        
        Expr::Paren(paren) => {
            collect_borrows(&paren.expr, borrows, context);
        }
        
        _ => {
            // Other expression types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_borrow() {
        let expr: Expr = parse_quote!(&x);
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 1);
        assert!(!borrows[0].is_mutable);
    }

    #[test]
    fn test_mutable_borrow() {
        let expr: Expr = parse_quote!(&mut x);
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 1);
        assert!(borrows[0].is_mutable);
    }

    #[test]
    fn test_function_call_with_borrow() {
        let expr: Expr = parse_quote!(foo(&x, &mut y));
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 2);
        assert!(!borrows[0].is_mutable);
        assert!(borrows[1].is_mutable);
    }

    #[test]
    fn test_method_call_with_borrow() {
        let expr: Expr = parse_quote!(obj.method(&x));
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 1);
        assert_eq!(borrows[0].context, BorrowContext::MethodArg);
    }

    #[test]
    fn test_nested_borrows() {
        let expr: Expr = parse_quote!(vec![&x, &y, &z]);
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 3);
    }

    #[test]
    fn test_return_borrow() {
        let expr: Expr = parse_quote!(return &x);
        let borrows = find_borrows_in_expr(&expr);
        
        assert_eq!(borrows.len(), 1);
        assert_eq!(borrows[0].context, BorrowContext::Return);
    }
}
```

---

## Step 2: Expression Transformation

### File: `borrowscope-macro/src/expr_transform.rs`

```rust
//! Expression transformation for borrows

use syn::{Expr, ExprCall, ExprMethodCall};
use quote::quote;
use crate::borrow_detection::{find_borrows_in_expr, BorrowContext};

/// Check if an expression contains borrows that need tracking
pub fn contains_trackable_borrows(expr: &Expr) -> bool {
    let borrows = find_borrows_in_expr(expr);
    !borrows.is_empty()
}

/// Transform function call to track borrow arguments
///
/// # Example
///
/// ```ignore
/// // Input:
/// foo(&x, &mut y)
///
/// // Output:
/// foo(
///     borrowscope_runtime::track_borrow("temp_0", &x),
///     borrowscope_runtime::track_borrow_mut("temp_1", &mut y)
/// )
/// ```
pub fn transform_call_with_borrows(call: &ExprCall) -> Expr {
    let func = &call.func;
    let mut new_args = Vec::new();
    
    for (i, arg) in call.args.iter().enumerate() {
        if let Expr::Reference(reference) = arg {
            let temp_name = format!("__borrow_arg_{}", i);
            let temp_ident = syn::Ident::new(&temp_name, proc_macro2::Span::call_site());
            let borrowed_expr = &reference.expr;
            let is_mutable = reference.mutability.is_some();
            
            let tracked_arg = if is_mutable {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_mut(
                        stringify!(#temp_ident),
                        &mut #borrowed_expr
                    )
                }
            } else {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow(
                        stringify!(#temp_ident),
                        &#borrowed_expr
                    )
                }
            };
            
            new_args.push(tracked_arg);
        } else {
            new_args.push(arg.clone());
        }
    }
    
    syn::parse_quote! {
        #func(#(#new_args),*)
    }
}

/// Transform method call to track borrow arguments
pub fn transform_method_with_borrows(method: &ExprMethodCall) -> Expr {
    let receiver = &method.receiver;
    let method_name = &method.method;
    let mut new_args = Vec::new();
    
    for (i, arg) in method.args.iter().enumerate() {
        if let Expr::Reference(reference) = arg {
            let temp_name = format!("__borrow_arg_{}", i);
            let temp_ident = syn::Ident::new(&temp_name, proc_macro2::Span::call_site());
            let borrowed_expr = &reference.expr;
            let is_mutable = reference.mutability.is_some();
            
            let tracked_arg = if is_mutable {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_mut(
                        stringify!(#temp_ident),
                        &mut #borrowed_expr
                    )
                }
            } else {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow(
                        stringify!(#temp_ident),
                        &#borrowed_expr
                    )
                }
            };
            
            new_args.push(tracked_arg);
        } else {
            new_args.push(arg.clone());
        }
    }
    
    syn::parse_quote! {
        #receiver.#method_name(#(#new_args),*)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_contains_trackable_borrows() {
        let expr1: Expr = parse_quote!(foo(&x));
        assert!(contains_trackable_borrows(&expr1));
        
        let expr2: Expr = parse_quote!(foo(x));
        assert!(!contains_trackable_borrows(&expr2));
    }

    #[test]
    fn test_transform_call_with_borrows() {
        let call: ExprCall = parse_quote!(foo(&x, &mut y));
        let transformed = transform_call_with_borrows(&call);
        
        let result = quote::quote! { #transformed }.to_string();
        assert!(result.contains("track_borrow"));
        assert!(result.contains("track_borrow_mut"));
    }

    #[test]
    fn test_transform_method_with_borrows() {
        let method: ExprMethodCall = parse_quote!(obj.method(&x));
        let transformed = transform_method_with_borrows(&method);
        
        let result = quote::quote! { #transformed }.to_string();
        assert!(result.contains("track_borrow"));
    }
}
```

---

## Step 3: Update Visitor for Expression Borrows

### File: `borrowscope-macro/src/visitor.rs` (Add expression handling)

```rust
// Add to existing visitor.rs

use crate::borrow_detection::find_borrows_in_expr;
use crate::expr_transform::{
    contains_trackable_borrows,
    transform_call_with_borrows,
    transform_method_with_borrows,
};

impl<'ctx> VisitMut for BorrowVisitor<'ctx> {
    // ... existing methods ...
    
    /// Visit an expression
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Transform function calls with borrow arguments
        if let Expr::Call(call) = expr {
            if contains_trackable_borrows(expr) {
                *expr = transform_call_with_borrows(call);
            }
        }
        
        // Transform method calls with borrow arguments
        if let Expr::MethodCall(method) = expr {
            if contains_trackable_borrows(expr) {
                *expr = transform_method_with_borrows(method);
            }
        }
        
        // Continue visiting children
        visit_mut::visit_expr_mut(self, expr);
    }
}
```

---

## Step 4: Handle Temporary Borrows

### File: `borrowscope-macro/src/temp_tracking.rs`

```rust
//! Temporary borrow tracking

use syn::Ident;
use std::collections::HashMap;

/// Manager for temporary borrow variables
pub struct TempBorrowTracker {
    /// Counter for generating unique names
    counter: usize,
    
    /// Map of temporary names to their original expressions
    temps: HashMap<String, String>,
}

impl TempBorrowTracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            counter: 0,
            temps: HashMap::new(),
        }
    }
    
    /// Generate a unique temporary name
    pub fn next_temp(&mut self) -> Ident {
        let name = format!("__borrow_temp_{}", self.counter);
        self.counter += 1;
        Ident::new(&name, proc_macro2::Span::call_site())
    }
    
    /// Register a temporary borrow
    pub fn register(&mut self, temp_name: String, original: String) {
        self.temps.insert(temp_name, original);
    }
    
    /// Get all temporary names
    pub fn temp_names(&self) -> Vec<Ident> {
        self.temps
            .keys()
            .map(|name| Ident::new(name, proc_macro2::Span::call_site()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_temp() {
        let mut tracker = TempBorrowTracker::new();
        
        let temp1 = tracker.next_temp();
        let temp2 = tracker.next_temp();
        
        assert_eq!(temp1.to_string(), "__borrow_temp_0");
        assert_eq!(temp2.to_string(), "__borrow_temp_1");
    }

    #[test]
    fn test_register() {
        let mut tracker = TempBorrowTracker::new();
        
        tracker.register("temp_0".to_string(), "x".to_string());
        tracker.register("temp_1".to_string(), "y".to_string());
        
        assert_eq!(tracker.temp_names().len(), 2);
    }
}
```

---

## Step 5: Integration Tests

### File: `borrowscope-macro/tests/borrow_expressions.rs`

```rust
//! Tests for borrow expressions in various contexts

use borrowscope_macro::trace_borrow;

#[test]
fn test_borrow_in_function_call() {
    fn takes_ref(x: &i32) {
        println!("{}", x);
    }
    
    #[trace_borrow]
    fn example() {
        let x = 5;
        takes_ref(&x);
    }
    
    example();
}

#[test]
fn test_multiple_borrows_in_call() {
    fn takes_two_refs(x: &i32, y: &i32) {
        println!("{} {}", x, y);
    }
    
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        takes_two_refs(&x, &y);
    }
    
    example();
}

#[test]
fn test_mutable_borrow_in_call() {
    fn takes_mut_ref(x: &mut i32) {
        *x += 1;
    }
    
    #[trace_borrow]
    fn example() {
        let mut x = 5;
        takes_mut_ref(&mut x);
        println!("{}", x);
    }
    
    example();
}

#[test]
fn test_borrow_in_method_call() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r = &s;
        println!("{}", r.len());
    }
    
    example();
}

#[test]
fn test_borrow_in_vec() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        let refs = vec![&x, &y];
        println!("{:?}", refs);
    }
    
    example();
}

#[test]
fn test_borrow_in_return() {
    #[trace_borrow]
    fn example() -> &'static str {
        let s = "hello";
        return s;
    }
    
    example();
}

#[test]
fn test_nested_borrow_calls() {
    fn outer(x: &i32) -> i32 {
        inner(x)
    }
    
    fn inner(x: &i32) -> i32 {
        *x * 2
    }
    
    #[trace_borrow]
    fn example() {
        let x = 5;
        let result = outer(&x);
        println!("{}", result);
    }
    
    example();
}

#[test]
fn test_borrow_in_if_expression() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        let r = if x > 3 { &x } else { &y };
        println!("{}", r);
    }
    
    example();
}

#[test]
fn test_borrow_in_match() {
    #[trace_borrow]
    fn example() {
        let x = Some(5);
        match x {
            Some(ref val) => println!("{}", val),
            None => println!("None"),
        }
    }
    
    example();
}
```

---

## Step 6: Handle Chained Method Calls

### File: `borrowscope-macro/src/chain_transform.rs`

```rust
//! Transformation for method chains

use syn::{Expr, ExprMethodCall};

/// Check if expression is a method chain
pub fn is_method_chain(expr: &Expr) -> bool {
    if let Expr::MethodCall(method) = expr {
        matches!(*method.receiver, Expr::MethodCall(_))
    } else {
        false
    }
}

/// Count methods in a chain
pub fn count_chain_length(expr: &Expr) -> usize {
    match expr {
        Expr::MethodCall(method) => {
            1 + count_chain_length(&method.receiver)
        }
        _ => 0,
    }
}

/// Extract all methods in a chain
pub fn extract_chain_methods(expr: &Expr) -> Vec<syn::Ident> {
    let mut methods = Vec::new();
    extract_methods_recursive(expr, &mut methods);
    methods.reverse(); // Reverse to get correct order
    methods
}

fn extract_methods_recursive(expr: &Expr, methods: &mut Vec<syn::Ident>) {
    if let Expr::MethodCall(method) = expr {
        methods.push(method.method.clone());
        extract_methods_recursive(&method.receiver, methods);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_method_chain() {
        let expr1: Expr = parse_quote!(x.foo().bar());
        assert!(is_method_chain(&expr1));
        
        let expr2: Expr = parse_quote!(x.foo());
        assert!(!is_method_chain(&expr2));
    }

    #[test]
    fn test_count_chain_length() {
        let expr: Expr = parse_quote!(x.foo().bar().baz());
        assert_eq!(count_chain_length(&expr), 3);
    }

    #[test]
    fn test_extract_chain_methods() {
        let expr: Expr = parse_quote!(x.foo().bar().baz());
        let methods = extract_chain_methods(&expr);
        
        assert_eq!(methods.len(), 3);
        assert_eq!(methods[0].to_string(), "foo");
        assert_eq!(methods[1].to_string(), "bar");
        assert_eq!(methods[2].to_string(), "baz");
    }
}
```

---

## Step 7: Comprehensive Test Suite

### File: `borrowscope-macro/tests/complex_borrows.rs`

```rust
//! Complex borrow scenarios

use borrowscope_macro::trace_borrow;

#[test]
fn test_borrow_in_closure() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let closure = || {
            println!("{}", &x);
        };
        closure();
    }
    
    example();
}

#[test]
fn test_borrow_with_lifetime() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r: &str = &s;
        println!("{}", r);
    }
    
    example();
}

#[test]
fn test_double_borrow() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let r1 = &x;
        let r2 = &r1;
        println!("{}", r2);
    }
    
    example();
}

#[test]
fn test_borrow_in_struct_init() {
    struct Container<'a> {
        value: &'a i32,
    }
    
    #[trace_borrow]
    fn example() {
        let x = 5;
        let container = Container { value: &x };
        println!("{}", container.value);
    }
    
    example();
}

#[test]
fn test_borrow_in_array() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        let z = 15;
        let refs = [&x, &y, &z];
        println!("{:?}", refs);
    }
    
    example();
}
```

---

## Step 8: Update Module Structure

### File: `borrowscope-macro/src/lib.rs` (Add new modules)

```rust
mod borrow_detection;     // NEW
mod expr_transform;       // NEW
mod temp_tracking;        // NEW
mod chain_transform;      // NEW

// ... rest of the code
```

---

## Key Takeaways

### Borrow Detection

✅ **All contexts** - Let, function args, method args, returns  
✅ **Recursive search** - Find nested borrows  
✅ **Context tracking** - Know where borrow occurs  
✅ **Mutable vs immutable** - Distinguish borrow types  
✅ **Complex expressions** - Arrays, tuples, conditionals  

### Transformation

✅ **Function calls** - Track borrow arguments  
✅ **Method calls** - Track receiver and arguments  
✅ **Temporary names** - Generate unique identifiers  
✅ **Preserve semantics** - Code behaves the same  
✅ **Chain handling** - Support method chains  

### Testing

✅ **Unit tests** - Test detection and transformation  
✅ **Integration tests** - Test with real code  
✅ **Edge cases** - Nested, chained, complex borrows  
✅ **Comprehensive coverage** - All borrow contexts  

---

## Exercises

### Exercise 1: Borrow in Closure

Extend to track borrows captured by closures:
```rust
let x = 5;
let closure = || println!("{}", &x);
```

### Exercise 2: Borrow in Async

Add support for borrows in async contexts:
```rust
async fn example() {
    let x = 5;
    async_fn(&x).await;
}
```

### Exercise 3: Borrow Statistics

Add a feature to count and report borrow statistics:
- Total borrows
- Mutable vs immutable
- By context type

---

## What's Next?

In **Section 17: Code Generation with quote**, we'll:
- Master quote! macro patterns
- Generate clean, readable code
- Handle edge cases in generation
- Optimize generated code
- Add code formatting

---

**Previous Section:** [15-identifying-variable-declarations.md](./15-identifying-variable-declarations.md)  
**Next Section:** [17-code-generation-with-quote.md](./17-code-generation-with-quote.md)

**Chapter Progress:** 8/12 sections complete ⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜

---

*"Borrows are everywhere. Track them all, and you see the complete picture." 🔍*
