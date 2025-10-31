# Section 10: Creating the Macro Crate

## Learning Objectives

By the end of this section, you will:
- Properly configure a procedural macro crate
- Set up syn and quote dependencies
- Understand proc-macro crate structure and limitations
- Create the foundation for the trace_borrow macro
- Set up comprehensive testing infrastructure
- Implement helper utilities for macro development

## Prerequisites

- Completed Section 9
- Understanding of procedural macros
- borrowscope workspace already created

---

## Procedural Macro Crate Requirements

### Special Crate Type

Procedural macro crates are **special**:

```toml
[lib]
proc-macro = true
```

**This means:**
- ✅ Can export procedural macros
- ❌ Cannot export regular functions or types
- ❌ Cannot be used as a normal dependency
- ✅ Compiled as a dynamic library
- ✅ Loaded by the compiler at compile time

### Dependency Restrictions

**Can depend on:**
- ✅ syn, quote, proc-macro2
- ✅ Other utility crates
- ✅ Regular libraries

**Cannot depend on:**
- ❌ Other proc-macro crates (usually)
- ❌ Crates that depend on this crate (circular)

---

## Step 1: Configure Cargo.toml

Update `borrowscope-macro/Cargo.toml`:

```toml
[package]
name = "borrowscope-macro"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Procedural macros for BorrowScope ownership tracking"
keywords = ["rust", "macro", "ownership", "borrow-checker"]
categories = ["development-tools", "rust-patterns"]

# This is a procedural macro crate
[lib]
proc-macro = true

[dependencies]
# Procedural macro essentials
syn = { workspace = true }
quote = { workspace = true }
proc-macro2 = { workspace = true }

# For better error messages
proc-macro-error = "1.0"

# Optional: for attribute parsing
darling = "0.20"

[dev-dependencies]
# For compile-time testing
trybuild = "1.0"

# For snapshot testing
insta = "1.34"

# For testing macro output
prettyplease = "0.2"

# Link to runtime for integration tests
borrowscope-runtime = { path = "../borrowscope-runtime" }
```

### Understanding Dependencies

#### syn (Syntax Parsing)

```rust
use syn::{parse_macro_input, ItemFn};

let function = parse_macro_input!(input as ItemFn);
```

**Features we need:**
- `full` - Parse all Rust syntax
- `visit-mut` - Modify AST in place
- `extra-traits` - Debug implementations

#### quote (Code Generation)

```rust
use quote::quote;

let output = quote! {
    fn generated_function() {
        println!("Hello!");
    }
};
```

**Converts Rust syntax to TokenStream**

#### proc-macro2 (Wrapper)

```rust
use proc_macro2::TokenStream;
```

**Why proc-macro2?**
- Works in tests (proc_macro doesn't)
- Better API
- More features

#### proc-macro-error (Error Handling)

```rust
use proc_macro_error::{abort, proc_macro_error};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn my_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    if something_wrong {
        abort!(item, "Error message");
    }
    item
}
```

**Better error messages with spans**

---

## Step 2: Create Module Structure

### File: `borrowscope-macro/src/lib.rs`

```rust
//! Procedural macros for BorrowScope
//!
//! This crate provides the `#[trace_borrow]` attribute macro for tracking
//! ownership and borrowing in Rust programs.
//!
//! # Example
//!
//! ```ignore
//! use borrowscope_macro::trace_borrow;
//!
//! #[trace_borrow]
//! fn example() {
//!     let s = String::from("hello");
//!     let r = &s;
//!     println!("{}", r);
//! }
//! ```

use proc_macro::TokenStream;

mod visitor;
mod transform;
mod utils;

#[cfg(test)]
mod tests;

/// Attribute macro to track ownership and borrowing
///
/// This macro instruments a function to track all ownership operations:
/// - Variable creation (let bindings)
/// - Borrows (& and &mut)
/// - Moves (ownership transfers)
/// - Drops (end of scope)
///
/// # Example
///
/// ```ignore
/// #[trace_borrow]
/// fn example() {
///     let s = String::from("hello");
///     let r = &s;
///     println!("{}", r);
/// }
/// ```
///
/// The macro transforms this into:
///
/// ```ignore
/// fn example() {
///     let s = borrowscope_runtime::track_new("s", String::from("hello"));
///     let r = borrowscope_runtime::track_borrow("r", &s);
///     println!("{}", r);
///     borrowscope_runtime::track_drop("r");
///     borrowscope_runtime::track_drop("s");
/// }
/// ```
///
/// # Limitations
///
/// - Only works on functions
/// - Cannot track moves accurately without type information
/// - May have false positives for Copy types
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the input unchanged
    // We'll implement the transformation in later sections
    let _ = attr; // Silence unused warning
    item
}
```

---

## Step 3: Create Visitor Module

### File: `borrowscope-macro/src/visitor.rs`

```rust
//! AST visitor for finding and transforming borrow patterns

use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Local, Stmt};

/// Visitor that finds and transforms ownership operations
pub struct BorrowVisitor {
    /// Counter for generating unique variable IDs
    var_counter: usize,
}

impl BorrowVisitor {
    /// Create a new visitor
    pub fn new() -> Self {
        Self { var_counter: 0 }
    }

    /// Generate a unique variable ID
    fn next_var_id(&mut self) -> String {
        let id = format!("var_{}", self var_counter);
        self.var_counter += 1;
        id
    }
}

impl VisitMut for BorrowVisitor {
    /// Visit a local variable declaration (let statement)
    fn visit_local_mut(&mut self, local: &mut Local) {
        // TODO: Transform let statements to inject tracking
        // For now, just visit children
        visit_mut::visit_local_mut(self, local);
    }

    /// Visit an expression
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // TODO: Transform expressions to track borrows
        // For now, just visit children
        visit_mut::visit_expr_mut(self, expr);
    }

    /// Visit a statement
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        // TODO: Handle different statement types
        // For now, just visit children
        visit_mut::visit_stmt_mut(self, stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_visitor_creation() {
        let visitor = BorrowVisitor::new();
        assert_eq!(visitor.var_counter, 0);
    }

    #[test]
    fn test_var_id_generation() {
        let mut visitor = BorrowVisitor::new();
        assert_eq!(visitor.next_var_id(), "var_0");
        assert_eq!(visitor.next_var_id(), "var_1");
        assert_eq!(visitor.next_var_id(), "var_2");
    }
}
```

---

## Step 4: Create Transform Module

### File: `borrowscope-macro/src/transform.rs`

```rust
//! Code transformation utilities

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, Local, Pat};

/// Transform a let statement to inject tracking
///
/// # Example
///
/// ```ignore
/// // Input:
/// let s = String::from("hello");
///
/// // Output:
/// let s = borrowscope_runtime::track_new("s", String::from("hello"));
/// ```
pub fn transform_let_statement(local: &Local) -> Option<TokenStream> {
    // Extract variable name
    let var_name = extract_var_name(&local.pat)?;
    
    // Get the initializer expression
    let init = local.init.as_ref()?;
    let init_expr = &init.expr;
    
    // Generate tracking call
    Some(quote! {
        let #var_name = borrowscope_runtime::track_new(
            stringify!(#var_name),
            #init_expr
        );
    })
}

/// Transform a borrow expression to inject tracking
///
/// # Example
///
/// ```ignore
/// // Input:
/// let r = &s;
///
/// // Output:
/// let r = borrowscope_runtime::track_borrow("r", &s);
/// ```
pub fn transform_borrow_expr(
    var_name: &Ident,
    borrowed_expr: &Expr,
    mutable: bool,
) -> TokenStream {
    if mutable {
        quote! {
            borrowscope_runtime::track_borrow_mut(
                stringify!(#var_name),
                #borrowed_expr
            )
        }
    } else {
        quote! {
            borrowscope_runtime::track_borrow(
                stringify!(#var_name),
                #borrowed_expr
            )
        }
    }
}

/// Generate a drop tracking call
///
/// # Example
///
/// ```ignore
/// borrowscope_runtime::track_drop("s");
/// ```
pub fn generate_drop_call(var_name: &Ident) -> TokenStream {
    quote! {
        borrowscope_runtime::track_drop(stringify!(#var_name));
    }
}

/// Extract variable name from a pattern
///
/// Handles simple patterns like `x` and `mut x`
fn extract_var_name(pat: &Pat) -> Option<&Ident> {
    match pat {
        Pat::Ident(pat_ident) => Some(&pat_ident.ident),
        _ => None, // TODO: Handle more complex patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_var_name_simple() {
        let pat: Pat = parse_quote!(x);
        let name = extract_var_name(&pat);
        assert!(name.is_some());
        assert_eq!(name.unwrap().to_string(), "x");
    }

    #[test]
    fn test_extract_var_name_mut() {
        let pat: Pat = parse_quote!(mut x);
        let name = extract_var_name(&pat);
        assert!(name.is_some());
        assert_eq!(name.unwrap().to_string(), "x");
    }

    #[test]
    fn test_generate_drop_call() {
        let var_name: Ident = parse_quote!(s);
        let drop_call = generate_drop_call(&var_name);
        let expected = quote! {
            borrowscope_runtime::track_drop(stringify!(s));
        };
        assert_eq!(drop_call.to_string(), expected.to_string());
    }
}
```

---

## Step 5: Create Utilities Module

### File: `borrowscope-macro/src/utils.rs`

```rust
//! Utility functions for macro development

use proc_macro2::Span;
use syn::{Expr, ExprReference};

/// Check if an expression is a borrow (&expr or &mut expr)
pub fn is_borrow_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Reference(_))
}

/// Check if a borrow is mutable
pub fn is_mutable_borrow(expr: &Expr) -> bool {
    if let Expr::Reference(ExprReference { mutability, .. }) = expr {
        mutability.is_some()
    } else {
        false
    }
}

/// Get the borrowed expression from a reference
///
/// # Example
///
/// ```ignore
/// // For &x, returns x
/// // For &mut x, returns x
/// ```
pub fn get_borrowed_expr(expr: &Expr) -> Option<&Expr> {
    if let Expr::Reference(ExprReference { expr, .. }) = expr {
        Some(expr.as_ref())
    } else {
        None
    }
}

/// Create a compiler error with a span
pub fn compile_error(span: Span, message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(span, message).to_compile_error()
}

/// Check if a pattern is simple (just an identifier)
pub fn is_simple_pattern(pat: &syn::Pat) -> bool {
    matches!(pat, syn::Pat::Ident(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_borrow_expr() {
        let expr: Expr = parse_quote!(&x);
        assert!(is_borrow_expr(&expr));

        let expr: Expr = parse_quote!(&mut x);
        assert!(is_borrow_expr(&expr));

        let expr: Expr = parse_quote!(x);
        assert!(!is_borrow_expr(&expr));
    }

    #[test]
    fn test_is_mutable_borrow() {
        let expr: Expr = parse_quote!(&x);
        assert!(!is_mutable_borrow(&expr));

        let expr: Expr = parse_quote!(&mut x);
        assert!(is_mutable_borrow(&expr));
    }

    #[test]
    fn test_get_borrowed_expr() {
        let expr: Expr = parse_quote!(&x);
        let borrowed = get_borrowed_expr(&expr);
        assert!(borrowed.is_some());

        let expr: Expr = parse_quote!(x);
        let borrowed = get_borrowed_expr(&expr);
        assert!(borrowed.is_none());
    }
}
```

---

## Step 6: Set Up Testing Infrastructure

### File: `borrowscope-macro/tests/compile_pass.rs`

```rust
//! Tests that should compile successfully

#[test]
fn test_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
}
```

### File: `borrowscope-macro/tests/compile_fail.rs`

```rust
//! Tests that should fail to compile

#[test]
fn test_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
}
```

### Create Test Directories

```bash
mkdir -p borrowscope-macro/tests/ui/pass
mkdir -p borrowscope-macro/tests/ui/fail
```

### File: `borrowscope-macro/tests/ui/pass/simple.rs`

```rust
//! Simple test that should compile

use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn simple_function() {
    let x = 5;
    println!("{}", x);
}

fn main() {
    simple_function();
}
```

### File: `borrowscope-macro/tests/ui/pass/with_borrow.rs`

```rust
//! Test with borrowing

use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn with_borrow() {
    let s = String::from("hello");
    let r = &s;
    println!("{}", r);
}

fn main() {
    with_borrow();
}
```

### File: `borrowscope-macro/tests/ui/fail/not_a_function.rs`

```rust
//! This should fail - attribute on non-function

use borrowscope_macro::trace_borrow;

#[trace_borrow]
struct NotAFunction {
    field: i32,
}

fn main() {}
```

---

## Step 7: Integration Tests

### File: `borrowscope-macro/tests/integration.rs`

```rust
//! Integration tests with runtime

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{reset, get_events};

#[trace_borrow]
fn test_function() {
    let x = 5;
    let y = 10;
    println!("{} {}", x, y);
}

#[test]
fn test_macro_with_runtime() {
    reset();
    test_function();
    
    // Once we implement tracking, we can verify events
    let events = get_events();
    // TODO: Assert events are correct
}
```

---

## Step 8: Snapshot Testing

### File: `borrowscope-macro/tests/snapshots.rs`

```rust
//! Snapshot tests for macro output

use quote::quote;
use syn::parse_quote;

#[test]
fn test_simple_function_snapshot() {
    let input = quote! {
        fn example() {
            let x = 5;
        }
    };
    
    // Parse and transform
    let output = borrowscope_macro::trace_borrow(
        Default::default(),
        input.into()
    );
    
    // Convert to pretty-printed string
    let syntax_tree = syn::parse_file(&output.to_string()).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    
    // Snapshot test
    insta::assert_snapshot!(formatted);
}
```

---

## Step 9: Helper Macros for Testing

### File: `borrowscope-macro/src/tests.rs`

```rust
//! Test utilities

#[cfg(test)]
macro_rules! assert_tokens_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(
            $left.to_string().replace(" ", ""),
            $right.to_string().replace(" ", "")
        );
    };
}

#[cfg(test)]
pub(crate) use assert_tokens_eq;

#[cfg(test)]
mod test_helpers {
    use super::*;
    use quote::quote;

    #[test]
    fn test_assert_tokens_eq() {
        let a = quote! { fn test() { } };
        let b = quote! { fn test(){} };
        assert_tokens_eq!(a, b);
    }
}
```

---

## Step 10: Build and Verify

### Build the Crate

```bash
cd borrowscope-macro
cargo build
```

Expected output:
```
   Compiling proc-macro2 v1.0.70
   Compiling quote v1.0.33
   Compiling syn v2.0.39
   Compiling borrowscope-macro v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 15.23s
```

### Run Tests

```bash
cargo test
```

Expected output:
```
   Compiling borrowscope-macro v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 3.45s
     Running unittests src/lib.rs

running 6 tests
test visitor::tests::test_visitor_creation ... ok
test visitor::tests::test_var_id_generation ... ok
test transform::tests::test_extract_var_name_simple ... ok
test transform::tests::test_extract_var_name_mut ... ok
test transform::tests::test_generate_drop_call ... ok
test utils::tests::test_is_borrow_expr ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Run Compile Tests

```bash
cargo test --test compile_pass
cargo test --test compile_fail
```

---

## Understanding the Structure

### Module Organization

```
borrowscope-macro/
├── src/
│   ├── lib.rs          # Main entry point, exports trace_borrow
│   ├── visitor.rs      # AST visitor for finding patterns
│   ├── transform.rs    # Code transformation functions
│   ├── utils.rs        # Helper utilities
│   └── tests.rs        # Test utilities
├── tests/
│   ├── compile_pass.rs # Tests that should compile
│   ├── compile_fail.rs # Tests that should fail
│   ├── integration.rs  # Integration with runtime
│   ├── snapshots.rs    # Snapshot tests
│   └── ui/
│       ├── pass/       # Passing test cases
│       └── fail/       # Failing test cases
└── Cargo.toml
```

### Separation of Concerns

**lib.rs:**
- Entry point
- Macro definition
- High-level orchestration

**visitor.rs:**
- AST traversal
- Pattern detection
- Mutation logic

**transform.rs:**
- Code generation
- Transformation functions
- Pure functions (no side effects)

**utils.rs:**
- Helper functions
- Common patterns
- Reusable utilities

---

## Common Patterns

### Pattern 1: Parse and Transform

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse
    let mut input = parse_macro_input!(item as ItemFn);
    
    // 2. Transform
    let mut visitor = MyVisitor::new();
    visitor.visit_item_fn_mut(&mut input);
    
    // 3. Generate
    quote! { #input }.into()
}
```

### Pattern 2: Error Handling

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = match syn::parse::<ItemFn>(item) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };
    
    // Process...
    
    quote! { #input }.into()
}
```

### Pattern 3: Conditional Transformation

```rust
impl VisitMut for MyVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if should_transform(expr) {
            *expr = transform_expr(expr);
        }
        
        // Continue visiting children
        visit_mut::visit_expr_mut(self, expr);
    }
}
```

---

## Debugging Macros

### Print Expanded Code

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand macros
cargo expand --lib
```

### Debug with eprintln!

```rust
#[proc_macro_attribute]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("Input: {}", item);
    
    // Process...
    
    eprintln!("Output: {}", output);
    output
}
```

**Note:** Output goes to stderr during compilation.

### Use RUST_LOG

```bash
RUST_LOG=debug cargo build
```

---

## Best Practices

### Do's

✅ **Keep modules focused** - One responsibility per module  
✅ **Write tests first** - TDD for macros  
✅ **Use helper functions** - Keep visitor clean  
✅ **Document thoroughly** - Macros are complex  
✅ **Handle errors gracefully** - Good error messages  

### Don'ts

❌ **Don't panic** - Return compile errors instead  
❌ **Don't export non-macros** - proc-macro crate limitation  
❌ **Don't mutate global state** - Macros should be pure  
❌ **Don't do I/O** - Macros run at compile time  
❌ **Don't assume types** - No type information available  

---

## Key Takeaways

### Crate Structure

✅ **proc-macro = true** - Special crate type  
✅ **Modular design** - visitor, transform, utils  
✅ **Comprehensive tests** - compile, integration, snapshot  
✅ **Helper utilities** - Reusable functions  
✅ **Clear separation** - Parsing, transformation, generation  

### Dependencies

✅ **syn** - Parse Rust syntax  
✅ **quote** - Generate code  
✅ **proc-macro2** - Better API  
✅ **trybuild** - Compile-time tests  
✅ **insta** - Snapshot tests  

### Testing Strategy

✅ **Unit tests** - Test individual functions  
✅ **Compile tests** - Verify macro compiles  
✅ **Integration tests** - Test with runtime  
✅ **Snapshot tests** - Verify output  
✅ **Fail tests** - Ensure errors work  

---

## Exercises

### Exercise 1: Add a Module

Create a new module `borrowscope-macro/src/patterns.rs` that identifies common Rust patterns.

### Exercise 2: Write a Test

Add a new compile test in `tests/ui/pass/` for a function with multiple variables.

### Exercise 3: Implement a Helper

Add a function to `utils.rs` that checks if an expression is a method call.

---

## What's Next?

In **Section 11: Understanding syn and quote**, we'll:
- Deep dive into syn's AST types
- Learn quote's code generation
- Master pattern matching on syntax
- Understand spans and hygiene
- Practice parsing and generating code

---

**Previous Section:** [09-introduction-to-procedural-macros.md](./09-introduction-to-procedural-macros.md)  
**Next Section:** [11-understanding-syn-and-quote.md](./11-understanding-syn-and-quote.md)

**Chapter Progress:** 2/12 sections complete ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Good structure is the foundation of good code." 🏗️*
