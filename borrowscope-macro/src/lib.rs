//! BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.

mod examples;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate it's a function
    if input_fn.sig.ident.to_string().is_empty() {
        abort!(
            input_fn.sig.ident,
            "trace_borrow can only be applied to functions"
        );
    }

    // For now, just return the function unchanged
    // We'll add instrumentation in later sections
    let output = quote! {
        #input_fn
    };

    output.into()
}
