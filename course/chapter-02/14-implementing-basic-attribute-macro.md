# Section 14: Implementing Basic Attribute Macro

## Learning Objectives

By the end of this section, you will:
- Implement the trace_borrow macro transformation
- Transform let statements to inject tracking
- Handle simple variable declarations
- Generate tracking calls correctly
- Test the transformation thoroughly
- Understand the complete transformation pipeline

## Prerequisites

- Completed Section 13
- Understanding of AST structure
- Familiarity with visitor pattern

---

## The Transformation Strategy

### What We're Building

**Input:**
```rust
#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r = &s;
    println!("{}", r);
}
```

**Output:**
```rust
fn example() {
    let s = borrowscope_runtime::track_new("s", String::from("hello"));
    let r = borrowscope_runtime::track_borrow("r", &s);
    println!("{}", r);
    borrowscope_runtime::track_drop("r");
    borrowscope_runtime::track_drop("s");
}
```

### Transformation Steps

1. **Parse** the function
2. **Validate** it's transformable
3. **Visit** each statement
4. **Transform** let statements
5. **Insert** drop calls
6. **Generate** output

---

## Step 1: Implement the Visitor

### File: `borrowscope-macro/src/visitor.rs`

```rust
//! AST visitor for transforming borrow patterns

use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Local, Stmt, Pat, Block};
use quote::quote;
use crate::context::TransformContext;

/// Visitor that transforms ownership operations
pub struct BorrowVisitor<'ctx> {
    /// Transformation context
    context: &'ctx mut TransformContext,
    
    /// Variables to track for drops
    tracked_variables: Vec<syn::Ident>,
}

impl<'ctx> BorrowVisitor<'ctx> {
    /// Create a new visitor
    pub fn new(context: &'ctx mut TransformContext) -> Self {
        Self {
            context,
            tracked_variables: Vec::new(),
        }
    }
    
    /// Get the list of tracked variables
    pub fn tracked_variables(&self) -> &[syn::Ident] {
        &self.tracked_variables
    }
    
    /// Check if a variable should be tracked
    fn should_track(&self, local: &Local) -> bool {
        // Must have simple identifier pattern
        if !matches!(local.pat, Pat::Ident(_)) {
            return false;
        }
        
        // Must have initializer
        if local.init.is_none() {
            return false;
        }
        
        // Don't track underscore variables
        if let Pat::Ident(pat_ident) = &local.pat {
            if pat_ident.ident.to_string().starts_with('_') {
                return false;
            }
        }
        
        true
    }
    
    /// Transform a let statement
    fn transform_let(&mut self, local: &mut Local) {
        if !self.should_track(local) {
            return;
        }
        
        // Extract variable name
        let var_name = if let Pat::Ident(pat_ident) = &local.pat {
            pat_ident.ident.clone()
        } else {
            return;
        };
        
        // Track this variable for drop
        self.tracked_variables.push(var_name.clone());
        
        // Get initializer
        let init = match &local.init {
            Some(init) => init,
            None => return,
        };
        
        let init_expr = &init.expr;
        
        // Check if it's a borrow
        if let Expr::Reference(reference) = init_expr.as_ref() {
            // Transform borrow
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
            
            // Replace initializer
            local.init = Some(syn::LocalInit {
                eq_token: init.eq_token,
                expr: Box::new(new_init),
                diverge: None,
            });
        } else {
            // Transform regular initialization
            let new_init = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    stringify!(#var_name),
                    #init_expr
                )
            };
            
            // Replace initializer
            local.init = Some(syn::LocalInit {
                eq_token: init.eq_token,
                expr: Box::new(new_init),
                diverge: None,
            });
        }
    }
}

impl<'ctx> VisitMut for BorrowVisitor<'ctx> {
    /// Visit a local variable declaration
    fn visit_local_mut(&mut self, local: &mut Local) {
        // Transform the let statement
        self.transform_let(local);
        
        // Continue visiting children
        visit_mut::visit_local_mut(self, local);
    }
    
    /// Visit a statement
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        // Visit children
        visit_mut::visit_stmt_mut(self, stmt);
    }
    
    /// Visit a block
    fn visit_block_mut(&mut self, block: &mut Block) {
        // Visit children
        visit_mut::visit_block_mut(self, block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use crate::options::TraceBorrowOptions;
    use crate::metadata::FunctionMetadata;

    #[test]
    fn test_should_track() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let visitor = BorrowVisitor::new(&mut context);
        
        // Should track
        let local1: Local = parse_quote! { let x = 5; };
        assert!(visitor.should_track(&local1));
        
        // Should not track (underscore)
        let local2: Local = parse_quote! { let _unused = 5; };
        assert!(!visitor.should_track(&local2));
        
        // Should not track (no initializer)
        let local3: Local = parse_quote! { let x; };
        assert!(!visitor.should_track(&local3));
    }

    #[test]
    fn test_transform_simple_let() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local: Local = parse_quote! { let x = 5; };
        visitor.transform_let(&mut local);
        
        let result = quote! { #local }.to_string();
        assert!(result.contains("track_new"));
        assert!(result.contains("stringify"));
    }

    #[test]
    fn test_transform_borrow() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local: Local = parse_quote! { let r = &s; };
        visitor.transform_let(&mut local);
        
        let result = quote! { #local }.to_string();
        assert!(result.contains("track_borrow"));
    }

    #[test]
    fn test_tracked_variables() {
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        let mut context = TransformContext::new(
            TraceBorrowOptions::default(),
            metadata
        );
        let mut visitor = BorrowVisitor::new(&mut context);
        
        let mut local1: Local = parse_quote! { let x = 5; };
        let mut local2: Local = parse_quote! { let y = 10; };
        
        visitor.transform_let(&mut local1);
        visitor.transform_let(&mut local2);
        
        assert_eq!(visitor.tracked_variables().len(), 2);
    }
}
```

---

## Step 2: Insert Drop Calls

### File: `borrowscope-macro/src/transform.rs` (Updated)

```rust
//! Code transformation utilities

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Block, Ident};

/// Insert drop tracking calls at the end of a block
///
/// # Example
///
/// ```ignore
/// // Input block:
/// {
///     let x = 5;
///     println!("{}", x);
/// }
///
/// // Output block:
/// {
///     let x = 5;
///     println!("{}", x);
///     borrowscope_runtime::track_drop("x");
/// }
/// ```
pub fn insert_drop_calls(
    block: &Block,
    variables: &[Ident],
    skip_drops: bool,
) -> Block {
    if skip_drops || variables.is_empty() {
        return block.clone();
    }
    
    let mut new_stmts = block.stmts.clone();
    
    // Add drop calls in reverse order (LIFO)
    for var in variables.iter().rev() {
        let drop_stmt: syn::Stmt = syn::parse_quote! {
            borrowscope_runtime::track_drop(stringify!(#var));
        };
        new_stmts.push(drop_stmt);
    }
    
    Block {
        brace_token: block.brace_token,
        stmts: new_stmts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_insert_drop_calls() {
        let block: Block = parse_quote! {
            {
                let x = 5;
                println!("{}", x);
            }
        };
        
        let vars = vec![
            parse_quote!(x),
        ];
        
        let new_block = insert_drop_calls(&block, &vars, false);
        
        let result = quote! { #new_block }.to_string();
        assert!(result.contains("track_drop"));
    }

    #[test]
    fn test_skip_drops() {
        let block: Block = parse_quote! {
            {
                let x = 5;
            }
        };
        
        let vars = vec![parse_quote!(x)];
        
        let new_block = insert_drop_calls(&block, &vars, true);
        
        let result = quote! { #new_block }.to_string();
        assert!(!result.contains("track_drop"));
    }

    #[test]
    fn test_reverse_order() {
        let block: Block = parse_quote! { {} };
        
        let vars = vec![
            parse_quote!(x),
            parse_quote!(y),
            parse_quote!(z),
        ];
        
        let new_block = insert_drop_calls(&block, &vars, false);
        let result = quote! { #new_block }.to_string();
        
        // Should drop in reverse order: z, y, x
        let z_pos = result.find("z").unwrap();
        let y_pos = result.find("y").unwrap();
        let x_pos = result.find("x").unwrap();
        
        assert!(z_pos < y_pos);
        assert!(y_pos < x_pos);
    }
}
```

---

## Step 3: Complete Macro Implementation

### File: `borrowscope-macro/src/lib.rs` (Final)

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

use options::TraceBorrowOptions;
use validate::validate_function;
use metadata::FunctionMetadata;
use context::TransformContext;
use visitor::BorrowVisitor;
use transform::insert_drop_calls;

#[cfg(test)]
mod tests;

/// Attribute macro to track ownership and borrowing
///
/// Transforms a function to track all ownership operations:
/// - Variable creation (let bindings)
/// - Borrows (& and &mut)
/// - Drops (end of scope)
///
/// # Options
///
/// - `verbose` - Enable verbose tracking output
/// - `skip_drops` - Don't track drop events
/// - `id_prefix` - Custom prefix for variable IDs
///
/// # Examples
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
/// Transforms to:
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
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute options
    let options = match syn::parse::<TraceBorrowOptions>(attr) {
        Ok(opts) => opts,
        Err(e) => return e.to_compile_error().into(),
    };
    
    // Parse the function
    let mut function = parse_macro_input!(item as ItemFn);
    
    // Validate the function
    if let Err(e) = validate_function(&function) {
        return e.to_compile_error().into();
    }
    
    // Extract metadata
    let metadata = FunctionMetadata::from_function(&function);
    
    // Create transformation context
    let mut context = TransformContext::new(options.clone(), metadata);
    
    // Create visitor and transform the function
    let mut visitor = BorrowVisitor::new(&mut context);
    visitor.visit_item_fn_mut(&mut function);
    
    // Get tracked variables
    let tracked_vars: Vec<_> = visitor.tracked_variables().to_vec();
    
    // Insert drop calls at the end of the function
    let new_block = insert_drop_calls(
        &function.block,
        &tracked_vars,
        options.skip_drops
    );
    
    function.block = Box::new(new_block);
    
    // Generate the transformed function
    quote! { #function }.into()
}
```

---

## Step 4: Integration Tests

### File: `borrowscope-macro/tests/transformation.rs`

```rust
//! Tests for macro transformation

use borrowscope_macro::trace_borrow;

#[test]
fn test_simple_transformation() {
    #[trace_borrow]
    fn example() {
        let x = 5;
    }
    
    example();
}

#[test]
fn test_with_borrow() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r = &s;
        println!("{}", r);
    }
    
    example();
}

#[test]
fn test_with_mutable_borrow() {
    #[trace_borrow]
    fn example() {
        let mut s = String::from("hello");
        let r = &mut s;
        r.push_str(" world");
        println!("{}", r);
    }
    
    example();
}

#[test]
fn test_multiple_variables() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        let z = x + y;
        println!("{}", z);
    }
    
    example();
}

#[test]
fn test_skip_underscore() {
    #[trace_borrow]
    fn example() {
        let _unused = 5;
        let x = 10;
        println!("{}", x);
    }
    
    example();
}

#[test]
fn test_with_return() {
    #[trace_borrow]
    fn example() -> i32 {
        let x = 5;
        let y = 10;
        x + y
    }
    
    assert_eq!(example(), 15);
}
```

---

## Step 5: Expansion Tests

### File: `borrowscope-macro/tests/expansion.rs`

```rust
//! Test macro expansion output

use quote::quote;
use syn::parse_quote;

#[test]
fn test_expansion_simple() {
    let input = quote! {
        #[trace_borrow]
        fn example() {
            let x = 5;
        }
    };
    
    // Parse and expand
    let output = borrowscope_macro::trace_borrow(
        Default::default(),
        input.into()
    );
    
    let output_str = output.to_string();
    
    // Should contain track_new
    assert!(output_str.contains("track_new"));
    
    // Should contain track_drop
    assert!(output_str.contains("track_drop"));
    
    // Should contain stringify
    assert!(output_str.contains("stringify"));
}

#[test]
fn test_expansion_borrow() {
    let input = quote! {
        #[trace_borrow]
        fn example() {
            let s = String::from("hello");
            let r = &s;
        }
    };
    
    let output = borrowscope_macro::trace_borrow(
        Default::default(),
        input.into()
    );
    
    let output_str = output.to_string();
    
    // Should contain track_new for s
    assert!(output_str.contains("track_new"));
    
    // Should contain track_borrow for r
    assert!(output_str.contains("track_borrow"));
    
    // Should have two track_drop calls
    assert_eq!(output_str.matches("track_drop").count(), 2);
}

#[test]
fn test_expansion_skip_drops() {
    let input = quote! {
        #[trace_borrow(skip_drops = true)]
        fn example() {
            let x = 5;
        }
    };
    
    let output = borrowscope_macro::trace_borrow(
        quote! { skip_drops = true }.into(),
        quote! {
            fn example() {
                let x = 5;
            }
        }.into()
    );
    
    let output_str = output.to_string();
    
    // Should contain track_new
    assert!(output_str.contains("track_new"));
    
    // Should NOT contain track_drop
    assert!(!output_str.contains("track_drop"));
}
```

---

## Step 6: Visual Verification

### Use cargo-expand

```bash
# Install cargo-expand
cargo install cargo-expand

# Create a test file
cat > examples/test_macro.rs << 'EOF'
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r = &s;
    println!("{}", r);
}

fn main() {
    example();
}
EOF

# Expand the macro
cargo expand --example test_macro
```

**Expected output:**
```rust
fn example() {
    let s = borrowscope_runtime::track_new(
        "s",
        String::from("hello")
    );
    let r = borrowscope_runtime::track_borrow(
        "r",
        &s
    );
    println!("{}", r);
    borrowscope_runtime::track_drop("r");
    borrowscope_runtime::track_drop("s");
}
```

---

## Step 7: End-to-End Test with Runtime

### File: `borrowscope-macro/tests/e2e.rs`

```rust
//! End-to-end tests with runtime

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{reset, get_events, Event};

#[test]
fn test_e2e_simple() {
    #[trace_borrow]
    fn example() {
        let x = 5;
    }
    
    reset();
    example();
    
    let events = get_events();
    
    // Should have New and Drop events
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
    assert!(events.iter().any(|e| matches!(e, Event::Drop { .. })));
}

#[test]
fn test_e2e_borrow() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r = &s;
        println!("{}", r);
    }
    
    reset();
    example();
    
    let events = get_events();
    
    // Should have New, Borrow, and Drop events
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
    assert!(events.iter().any(|e| matches!(e, Event::Drop { .. })));
}
```

---

## Step 8: Build and Test Everything

### Build

```bash
cargo build --workspace
```

### Run All Tests

```bash
cargo test --workspace
```

Expected output:
```
running 25 tests
test visitor::tests::test_should_track ... ok
test visitor::tests::test_transform_simple_let ... ok
test visitor::tests::test_transform_borrow ... ok
test transform::tests::test_insert_drop_calls ... ok
test transformation::test_simple_transformation ... ok
test transformation::test_with_borrow ... ok
test expansion::test_expansion_simple ... ok
test expansion::test_expansion_borrow ... ok
...

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured
```

---

## Understanding the Complete Flow

### 1. Parse

```rust
let function = parse_macro_input!(item as ItemFn);
```

Converts `TokenStream` to `ItemFn`.

### 2. Validate

```rust
validate_function(&function)?;
```

Ensures function is transformable.

### 3. Create Context

```rust
let mut context = TransformContext::new(options, metadata);
```

Bundles options and metadata.

### 4. Transform

```rust
let mut visitor = BorrowVisitor::new(&mut context);
visitor.visit_item_fn_mut(&mut function);
```

Visits and transforms each statement.

### 5. Insert Drops

```rust
let new_block = insert_drop_calls(&function.block, &tracked_vars, skip_drops);
```

Adds drop tracking at end.

### 6. Generate

```rust
quote! { #function }.into()
```

Converts back to `TokenStream`.

---

## Key Takeaways

### Implementation

✅ **Visitor pattern** - Clean separation of concerns  
✅ **Mutable transformation** - Modify AST in place  
✅ **Context management** - Bundle state and options  
✅ **Drop tracking** - LIFO order (reverse)  
✅ **Conditional logic** - Skip underscore variables  

### Testing

✅ **Unit tests** - Test each component  
✅ **Integration tests** - Test with runtime  
✅ **Expansion tests** - Verify generated code  
✅ **E2E tests** - Full workflow  
✅ **Visual verification** - cargo-expand  

### Best Practices

✅ **Preserve semantics** - Code behaves the same  
✅ **Handle edge cases** - Underscore, no init  
✅ **Clear errors** - Helpful messages  
✅ **Comprehensive tests** - High confidence  
✅ **Modular design** - Easy to extend  

---

## Exercises

### Exercise 1: Add Verbose Mode

Implement verbose mode that prints when tracking occurs:
```rust
#[trace_borrow(verbose = true)]
fn example() {
    let x = 5;
}
// Should print: "Tracking new: x"
```

### Exercise 2: Track Function Entry/Exit

Add tracking for function entry and exit:
```rust
#[trace_borrow]
fn example() {
    // Should track: "Entering: example"
    let x = 5;
    // Should track: "Exiting: example"
}
```

### Exercise 3: Handle Nested Blocks

Extend to handle nested blocks correctly:
```rust
#[trace_borrow]
fn example() {
    let x = 5;
    {
        let y = 10;
        // y should drop here
    }
    // x should drop here
}
```

---

## What's Next?

In **Section 15: Identifying Variable Declarations**, we'll:
- Handle complex patterns (tuples, structs)
- Track destructuring
- Handle pattern matching
- Support more declaration types
- Improve pattern recognition

---

**Previous Section:** [13-abstract-syntax-tree-basics.md](./13-abstract-syntax-tree-basics.md)  
**Next Section:** [15-identifying-variable-declarations.md](./15-identifying-variable-declarations.md)

**Chapter Progress:** 6/12 sections complete ⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜

---

*"The first working version is always the most exciting!" 🎉*
