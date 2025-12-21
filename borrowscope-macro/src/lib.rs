//! BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.

mod best_practices;
mod borrow_detection;
mod codegen;
mod examples;
mod formatting;
mod generic_handler;
mod hygiene;
mod optimized_transform;
mod parser;
mod pattern;
mod smart_pointer;
mod span_utils;
mod transform_visitor;
mod validation;
mod visitor;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, ItemFn};
use transform_visitor::OwnershipVisitor;

/// Validate function before transformation
fn validate_function(func: &ItemFn) {
    // Check for const functions
    if func.sig.constness.is_some() {
        abort!(
            func.sig.constness,
            "const functions cannot be tracked";
            help = "remove the `const` keyword to enable tracking";
            note = "tracking requires runtime operations, but const functions are evaluated at compile time"
        );
    }

    // Check for extern functions
    if let Some(abi) = &func.sig.abi {
        abort!(
            abi,
            "extern functions cannot be tracked";
            help = "only Rust ABI functions can be tracked"
        );
    }

    // Warn about async functions (they work but with limitations)
    if func.sig.asyncness.is_some() {
        // Note: We allow async but could add a warning in the future
    }

    // Warn about unsafe functions (they work but can't verify safety)
    if func.sig.unsafety.is_some() {
        // Note: We allow unsafe but tracking may be incomplete
    }
}

/// Attribute macro to trace ownership and borrowing in a function
///
/// # Example
/// ```ignore
/// #[trace_borrow]
/// fn example() {
///     let x = String::from("hello");
///     let y = &x;
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a function
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function
    validate_function(&input_fn);

    // Transform the function body using OwnershipVisitor
    let mut visitor = OwnershipVisitor::new();
    visitor.visit_item_fn_mut(&mut input_fn);

    // Generate output
    let output = quote! {
        #input_fn
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn transform_function(func: &mut ItemFn) {
        let mut visitor = OwnershipVisitor::new();
        visitor.visit_item_fn_mut(func);
    }

    #[test]
    fn test_transform_simple_variable() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_new"));
    }

    #[test]
    fn test_transform_borrow() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                let y = &x;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_borrow"));
    }

    #[test]
    fn test_transform_mut_borrow() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let mut x = 5;
                let y = &mut x;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_borrow_mut"));
    }

    #[test]
    fn test_preserves_function_signature() {
        let mut func: ItemFn = parse_quote! {
            fn example(a: i32) -> i32 {
                let x = a;
                x
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("fn example"));
        assert!(output.contains("a : i32"));
        assert!(output.contains("-> i32"));
    }

    #[test]
    fn test_preserves_generics() {
        let mut func: ItemFn = parse_quote! {
            fn example<T>(value: T) -> T {
                value
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("fn example"));
        assert!(output.contains("< T >"));
    }

    #[test]
    fn test_no_transform_without_init() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x;
                x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        // Should not add tracking for uninitialized variables
        assert!(!output.contains("track_new"));
    }

    #[test]
    fn test_preserves_visibility() {
        let mut func: ItemFn = parse_quote! {
            pub fn example() {
                let x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("pub fn example"));
    }
}
