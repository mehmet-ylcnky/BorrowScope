# Section 12: Parsing Function Attributes

## Learning Objectives

By the end of this section, you will:
- Parse attribute macro arguments
- Validate function signatures
- Extract function metadata
- Handle attribute options
- Implement proper error handling
- Prepare functions for transformation

## Prerequisites

- Completed Section 11
- Understanding of syn and quote
- Familiarity with Rust attributes

---

## Understanding Attribute Macros

### Attribute Syntax

```rust
#[trace_borrow]
fn simple() { }

#[trace_borrow(verbose)]
fn with_arg() { }

#[trace_borrow(verbose = true, skip_drops = false)]
fn with_options() { }
```

### Macro Signature

```rust
#[proc_macro_attribute]
pub fn trace_borrow(
    attr: TokenStream,    // Arguments: (verbose = true)
    item: TokenStream,    // The item: fn simple() { }
) -> TokenStream {
    // Parse and transform
}
```

---

## Step 1: Define Attribute Options

### File: `borrowscope-macro/src/options.rs`

```rust
//! Attribute options for #[trace_borrow]

use syn::parse::{Parse, ParseStream};
use syn::{Token, Ident, Lit, Error};

/// Options for the trace_borrow attribute
///
/// # Example
///
/// ```ignore
/// #[trace_borrow(verbose = true, skip_drops = false)]
/// ```
#[derive(Debug, Clone, Default)]
pub struct TraceBorrowOptions {
    /// Enable verbose output
    pub verbose: bool,
    
    /// Skip tracking drop events
    pub skip_drops: bool,
    
    /// Custom prefix for generated variable IDs
    pub id_prefix: Option<String>,
}

impl Parse for TraceBorrowOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut options = TraceBorrowOptions::default();
        
        // Parse comma-separated options
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            
            match key.to_string().as_str() {
                "verbose" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let value: Lit = input.parse()?;
                        options.verbose = parse_bool_lit(&value)?;
                    } else {
                        options.verbose = true;
                    }
                }
                "skip_drops" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let value: Lit = input.parse()?;
                        options.skip_drops = parse_bool_lit(&value)?;
                    } else {
                        options.skip_drops = true;
                    }
                }
                "id_prefix" => {
                    input.parse::<Token![=]>()?;
                    let value: Lit = input.parse()?;
                    options.id_prefix = Some(parse_string_lit(&value)?);
                }
                _ => {
                    return Err(Error::new(
                        key.span(),
                        format!("Unknown option: {}", key)
                    ));
                }
            }
            
            // Parse comma if not at end
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(options)
    }
}

/// Parse a boolean literal
fn parse_bool_lit(lit: &Lit) -> syn::Result<bool> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.value),
        _ => Err(Error::new(lit.span(), "Expected boolean literal")),
    }
}

/// Parse a string literal
fn parse_string_lit(lit: &Lit) -> syn::Result<String> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        _ => Err(Error::new(lit.span(), "Expected string literal")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_empty_options() {
        let options: TraceBorrowOptions = parse_quote!();
        assert!(!options.verbose);
        assert!(!options.skip_drops);
        assert!(options.id_prefix.is_none());
    }

    #[test]
    fn test_parse_verbose() {
        let options: TraceBorrowOptions = parse_quote!(verbose);
        assert!(options.verbose);
    }

    #[test]
    fn test_parse_verbose_explicit() {
        let options: TraceBorrowOptions = parse_quote!(verbose = true);
        assert!(options.verbose);
        
        let options: TraceBorrowOptions = parse_quote!(verbose = false);
        assert!(!options.verbose);
    }

    #[test]
    fn test_parse_multiple_options() {
        let options: TraceBorrowOptions = parse_quote!(
            verbose = true,
            skip_drops = false,
            id_prefix = "test"
        );
        
        assert!(options.verbose);
        assert!(!options.skip_drops);
        assert_eq!(options.id_prefix, Some("test".to_string()));
    }
}
```

---

## Step 2: Validate Function Signatures

### File: `borrowscope-macro/src/validate.rs`

```rust
//! Function validation

use syn::{ItemFn, Error, ReturnType};

/// Validate that a function can be traced
pub fn validate_function(func: &ItemFn) -> syn::Result<()> {
    // Check if function is async
    if func.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            &func.sig.asyncness,
            "trace_borrow does not yet support async functions"
        ));
    }
    
    // Check if function is unsafe
    if func.sig.unsafety.is_some() {
        return Err(Error::new_spanned(
            &func.sig.unsafety,
            "trace_borrow cannot be used on unsafe functions"
        ));
    }
    
    // Check if function is const
    if func.sig.constness.is_some() {
        return Err(Error::new_spanned(
            &func.sig.constness,
            "trace_borrow cannot be used on const functions"
        ));
    }
    
    // Warn if function has parameters (we can track them, but it's complex)
    // For now, we'll allow it but note it in docs
    
    Ok(())
}

/// Check if function returns a value
pub fn has_return_value(func: &ItemFn) -> bool {
    !matches!(func.sig.output, ReturnType::Default)
}

/// Get function name as string
pub fn get_function_name(func: &ItemFn) -> String {
    func.sig.ident.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_validate_simple_function() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        
        assert!(validate_function(&func).is_ok());
    }

    #[test]
    fn test_validate_async_function() {
        let func: ItemFn = parse_quote! {
            async fn example() {
                let x = 5;
            }
        };
        
        assert!(validate_function(&func).is_err());
    }

    #[test]
    fn test_validate_unsafe_function() {
        let func: ItemFn = parse_quote! {
            unsafe fn example() {
                let x = 5;
            }
        };
        
        assert!(validate_function(&func).is_err());
    }

    #[test]
    fn test_validate_const_function() {
        let func: ItemFn = parse_quote! {
            const fn example() -> i32 {
                5
            }
        };
        
        assert!(validate_function(&func).is_err());
    }

    #[test]
    fn test_has_return_value() {
        let func1: ItemFn = parse_quote! {
            fn example() { }
        };
        assert!(!has_return_value(&func1));
        
        let func2: ItemFn = parse_quote! {
            fn example() -> i32 { 5 }
        };
        assert!(has_return_value(&func2));
    }

    #[test]
    fn test_get_function_name() {
        let func: ItemFn = parse_quote! {
            fn my_function() { }
        };
        
        assert_eq!(get_function_name(&func), "my_function");
    }
}
```

---

## Step 3: Extract Function Metadata

### File: `borrowscope-macro/src/metadata.rs`

```rust
//! Function metadata extraction

use syn::{ItemFn, Attribute, Visibility};

/// Metadata about a function
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    
    /// Is the function public?
    pub is_public: bool,
    
    /// Does it have a return value?
    pub has_return: bool,
    
    /// Number of parameters
    pub param_count: usize,
    
    /// Other attributes on the function
    pub attributes: Vec<Attribute>,
}

impl FunctionMetadata {
    /// Extract metadata from a function
    pub fn from_function(func: &ItemFn) -> Self {
        Self {
            name: func.sig.ident.to_string(),
            is_public: matches!(func.vis, Visibility::Public(_)),
            has_return: !matches!(func.sig.output, syn::ReturnType::Default),
            param_count: func.sig.inputs.len(),
            attributes: func.attrs.clone(),
        }
    }
    
    /// Check if function has a specific attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|attr| {
            attr.path().is_ident(name)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_metadata() {
        let func: ItemFn = parse_quote! {
            pub fn example(x: i32, y: i32) -> i32 {
                x + y
            }
        };
        
        let metadata = FunctionMetadata::from_function(&func);
        
        assert_eq!(metadata.name, "example");
        assert!(metadata.is_public);
        assert!(metadata.has_return);
        assert_eq!(metadata.param_count, 2);
    }

    #[test]
    fn test_private_function() {
        let func: ItemFn = parse_quote! {
            fn private_func() { }
        };
        
        let metadata = FunctionMetadata::from_function(&func);
        assert!(!metadata.is_public);
    }

    #[test]
    fn test_has_attribute() {
        let func: ItemFn = parse_quote! {
            #[inline]
            #[test]
            fn example() { }
        };
        
        let metadata = FunctionMetadata::from_function(&func);
        assert!(metadata.has_attribute("inline"));
        assert!(metadata.has_attribute("test"));
        assert!(!metadata.has_attribute("derive"));
    }
}
```

---

## Step 4: Update Main Macro

### File: `borrowscope-macro/src/lib.rs`

```rust
//! Procedural macros for BorrowScope

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

mod options;
mod validate;
mod metadata;
mod visitor;
mod transform;
mod utils;

use options::TraceBorrowOptions;
use validate::validate_function;
use metadata::FunctionMetadata;

#[cfg(test)]
mod tests;

/// Attribute macro to track ownership and borrowing
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
/// // Basic usage
/// #[trace_borrow]
/// fn example() {
///     let s = String::from("hello");
/// }
///
/// // With options
/// #[trace_borrow(verbose = true)]
/// fn verbose_example() {
///     let s = String::from("hello");
/// }
///
/// // Multiple options
/// #[trace_borrow(verbose = true, skip_drops = false)]
/// fn custom_example() {
///     let s = String::from("hello");
/// }
/// ```
///
/// # Limitations
///
/// - Cannot be used on async functions
/// - Cannot be used on unsafe functions
/// - Cannot be used on const functions
/// - May have false positives for Copy types
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute options
    let options = match syn::parse::<TraceBorrowOptions>(attr) {
        Ok(opts) => opts,
        Err(e) => return e.to_compile_error().into(),
    };
    
    // Parse the function
    let function = parse_macro_input!(item as ItemFn);
    
    // Validate the function
    if let Err(e) = validate_function(&function) {
        return e.to_compile_error().into();
    }
    
    // Extract metadata
    let _metadata = FunctionMetadata::from_function(&function);
    
    // For now, just return the function unchanged
    // We'll implement transformation in the next sections
    let _ = options; // Silence unused warning
    
    quote! { #function }.into()
}
```

---

## Step 5: Integration Tests

### File: `borrowscope-macro/tests/attribute_parsing.rs`

```rust
//! Tests for attribute parsing

use borrowscope_macro::trace_borrow;

#[test]
fn test_no_options() {
    #[trace_borrow]
    fn example() {
        let x = 5;
    }
    
    example();
}

#[test]
fn test_verbose_option() {
    #[trace_borrow(verbose)]
    fn example() {
        let x = 5;
    }
    
    example();
}

#[test]
fn test_multiple_options() {
    #[trace_borrow(verbose = true, skip_drops = false)]
    fn example() {
        let x = 5;
    }
    
    example();
}

#[test]
fn test_with_parameters() {
    #[trace_borrow]
    fn example(x: i32, y: i32) -> i32 {
        x + y
    }
    
    assert_eq!(example(2, 3), 5);
}

#[test]
fn test_with_return_value() {
    #[trace_borrow]
    fn example() -> i32 {
        let x = 5;
        x
    }
    
    assert_eq!(example(), 5);
}
```

### File: `borrowscope-macro/tests/ui/fail/async_function.rs`

```rust
//! Should fail: async function

use borrowscope_macro::trace_borrow;

#[trace_borrow]
async fn example() {
    let x = 5;
}

fn main() {}
```

### File: `borrowscope-macro/tests/ui/fail/unsafe_function.rs`

```rust
//! Should fail: unsafe function

use borrowscope_macro::trace_borrow;

#[trace_borrow]
unsafe fn example() {
    let x = 5;
}

fn main() {}
```

### File: `borrowscope-macro/tests/ui/fail/const_function.rs`

```rust
//! Should fail: const function

use borrowscope_macro::trace_borrow;

#[trace_borrow]
const fn example() -> i32 {
    5
}

fn main() {}
```

---

## Step 6: Advanced Option Parsing with darling

For more complex attribute parsing, we can use the `darling` crate:

### File: `borrowscope-macro/src/options_darling.rs`

```rust
//! Alternative options parsing using darling

use darling::FromMeta;
use syn::AttributeArgs;

#[derive(Debug, FromMeta, Default)]
pub struct TraceBorrowOptionsDarling {
    #[darling(default)]
    pub verbose: bool,
    
    #[darling(default)]
    pub skip_drops: bool,
    
    #[darling(default)]
    pub id_prefix: Option<String>,
}

impl TraceBorrowOptionsDarling {
    /// Parse from attribute arguments
    pub fn from_args(args: &AttributeArgs) -> darling::Result<Self> {
        Self::from_list(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_darling_parse() {
        let args: AttributeArgs = parse_quote!(verbose = true, skip_drops = false);
        let options = TraceBorrowOptionsDarling::from_args(&args).unwrap();
        
        assert!(options.verbose);
        assert!(!options.skip_drops);
    }
}
```

---

## Step 7: Helper Functions for Attribute Handling

### File: `borrowscope-macro/src/attr_utils.rs`

```rust
//! Attribute utilities

use syn::{Attribute, Meta};

/// Check if an attribute has a specific name
pub fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

/// Get the value of a named attribute
pub fn get_attribute_value(attrs: &[Attribute], name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(name) {
            if let Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
        }
    }
    None
}

/// Remove a specific attribute from a list
pub fn remove_attribute(attrs: &mut Vec<Attribute>, name: &str) {
    attrs.retain(|attr| !attr.path().is_ident(name));
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_has_attribute() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[inline]),
            parse_quote!(#[test]),
        ];
        
        assert!(has_attribute(&attrs, "inline"));
        assert!(has_attribute(&attrs, "test"));
        assert!(!has_attribute(&attrs, "derive"));
    }

    #[test]
    fn test_remove_attribute() {
        let mut attrs: Vec<Attribute> = vec![
            parse_quote!(#[inline]),
            parse_quote!(#[test]),
        ];
        
        remove_attribute(&mut attrs, "inline");
        
        assert_eq!(attrs.len(), 1);
        assert!(!has_attribute(&attrs, "inline"));
        assert!(has_attribute(&attrs, "test"));
    }
}
```

---

## Step 8: Context for Transformation

### File: `borrowscope-macro/src/context.rs`

```rust
//! Transformation context

use crate::options::TraceBorrowOptions;
use crate::metadata::FunctionMetadata;

/// Context for transforming a function
#[derive(Debug, Clone)]
pub struct TransformContext {
    /// Attribute options
    pub options: TraceBorrowOptions,
    
    /// Function metadata
    pub metadata: FunctionMetadata,
    
    /// Counter for generating unique IDs
    pub id_counter: usize,
}

impl TransformContext {
    /// Create a new context
    pub fn new(options: TraceBorrowOptions, metadata: FunctionMetadata) -> Self {
        Self {
            options,
            metadata,
            id_counter: 0,
        }
    }
    
    /// Generate a unique variable ID
    pub fn next_id(&mut self) -> String {
        let prefix = self.options.id_prefix
            .as_deref()
            .unwrap_or("var");
        
        let id = format!("{}_{}", prefix, self.id_counter);
        self.id_counter += 1;
        id
    }
    
    /// Check if we should skip drops
    pub fn should_skip_drops(&self) -> bool {
        self.options.skip_drops
    }
    
    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.options.verbose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_next_id() {
        let options = TraceBorrowOptions::default();
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        
        let mut ctx = TransformContext::new(options, metadata);
        
        assert_eq!(ctx.next_id(), "var_0");
        assert_eq!(ctx.next_id(), "var_1");
        assert_eq!(ctx.next_id(), "var_2");
    }

    #[test]
    fn test_custom_prefix() {
        let mut options = TraceBorrowOptions::default();
        options.id_prefix = Some("test".to_string());
        
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        
        let mut ctx = TransformContext::new(options, metadata);
        
        assert_eq!(ctx.next_id(), "test_0");
    }

    #[test]
    fn test_skip_drops() {
        let mut options = TraceBorrowOptions::default();
        options.skip_drops = true;
        
        let func: syn::ItemFn = parse_quote! { fn test() {} };
        let metadata = FunctionMetadata::from_function(&func);
        
        let ctx = TransformContext::new(options, metadata);
        
        assert!(ctx.should_skip_drops());
    }
}
```

---

## Step 9: Update lib.rs with Context

### Updated `borrowscope-macro/src/lib.rs`

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

#[cfg(test)]
mod tests;

#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute options
    let options = match syn::parse::<TraceBorrowOptions>(attr) {
        Ok(opts) => opts,
        Err(e) => return e.to_compile_error().into(),
    };
    
    // Parse the function
    let function = parse_macro_input!(item as ItemFn);
    
    // Validate the function
    if let Err(e) = validate_function(&function) {
        return e.to_compile_error().into();
    }
    
    // Extract metadata
    let metadata = FunctionMetadata::from_function(&function);
    
    // Create transformation context
    let _context = TransformContext::new(options, metadata);
    
    // TODO: Transform the function using the context
    // For now, return unchanged
    
    quote! { #function }.into()
}
```

---

## Step 10: Build and Test

### Build

```bash
cargo build -p borrowscope-macro
```

### Run Tests

```bash
cargo test -p borrowscope-macro
```

Expected output:
```
running 15 tests
test options::tests::test_parse_empty_options ... ok
test options::tests::test_parse_verbose ... ok
test options::tests::test_parse_multiple_options ... ok
test validate::tests::test_validate_simple_function ... ok
test validate::tests::test_validate_async_function ... ok
test metadata::tests::test_extract_metadata ... ok
test context::tests::test_next_id ... ok
test context::tests::test_custom_prefix ... ok
...

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

### Run Compile Tests

```bash
cargo test --test compile_fail
```

Should show that async, unsafe, and const functions fail to compile.

---

## Key Takeaways

### Attribute Parsing

✅ **Parse options** - Custom derive or manual parsing  
✅ **Validate input** - Check function signatures  
✅ **Extract metadata** - Gather function information  
✅ **Create context** - Bundle options and metadata  
✅ **Handle errors** - Proper error messages with spans  

### Validation

✅ **Check function type** - No async, unsafe, const  
✅ **Validate signature** - Parameters, return type  
✅ **Preserve attributes** - Keep other attributes  
✅ **Error messages** - Clear, helpful errors  

### Context Management

✅ **Bundle state** - Options + metadata + counters  
✅ **Generate IDs** - Unique variable identifiers  
✅ **Configuration** - Respect user options  
✅ **Immutable where possible** - Easier to reason about  

---

## Exercises

### Exercise 1: Add New Option

Add a `max_depth` option that limits tracking depth:
```rust
#[trace_borrow(max_depth = 5)]
fn example() { }
```

### Exercise 2: Validate Parameters

Add validation to warn if function has more than 5 parameters.

### Exercise 3: Extract More Metadata

Add fields to `FunctionMetadata` for:
- Generic parameters
- Where clauses
- Lifetime parameters

---

## What's Next?

In **Section 13: Abstract Syntax Tree Basics**, we'll:
- Deep dive into AST structure
- Understand different node types
- Learn AST traversal patterns
- Practice pattern matching on AST
- Prepare for transformation

---

**Previous Section:** [11-understanding-syn-and-quote.md](./11-understanding-syn-and-quote.md)  
**Next Section:** [13-abstract-syntax-tree-basics.md](./13-abstract-syntax-tree-basics.md)

**Chapter Progress:** 4/12 sections complete ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Good parsing is the foundation of good transformation." 🎯*
