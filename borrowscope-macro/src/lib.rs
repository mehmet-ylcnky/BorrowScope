//! BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.

mod borrow_detection;
mod codegen;
mod examples;
mod formatting;
mod optimized_transform;
mod parser;
mod pattern;
mod span_utils;
mod visitor;

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
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Validate it's a function
    if input_fn.sig.ident.to_string().is_empty() {
        abort!(
            input_fn.sig.ident,
            "trace_borrow can only be applied to functions"
        );
    }

    // Transform the function body
    transform_function(&mut input_fn);

    // Generate output
    let output = quote! {
        #input_fn
    };

    output.into()
}

/// Transform a function to add tracking calls
fn transform_function(func: &mut ItemFn) {
    use syn::visit_mut::{self, VisitMut};
    use syn::{Expr, Local};

    struct Transformer;

    impl VisitMut for Transformer {
        fn visit_local_mut(&mut self, local: &mut Local) {
            // Only transform simple let bindings with initializers
            if let (syn::Pat::Ident(pat_ident), Some(init)) = (&local.pat, &local.init) {
                let var_name = &pat_ident.ident;
                let init_expr = &init.expr;

                // Check if initializer is a borrow
                let new_expr = if let Expr::Reference(reference) = init_expr.as_ref() {
                    let borrowed = &reference.expr;
                    if reference.mutability.is_some() {
                        syn::parse_quote! {
                            borrowscope_runtime::track_borrow_mut(stringify!(#var_name), &mut #borrowed)
                        }
                    } else {
                        syn::parse_quote! {
                            borrowscope_runtime::track_borrow(stringify!(#var_name), &#borrowed)
                        }
                    }
                } else {
                    syn::parse_quote! {
                        borrowscope_runtime::track_new(stringify!(#var_name), #init_expr)
                    }
                };

                local.init = Some(syn::LocalInit {
                    eq_token: init.eq_token,
                    expr: Box::new(new_expr),
                    diverge: None,
                });
            }

            visit_mut::visit_local_mut(self, local);
        }
    }

    Transformer.visit_item_fn_mut(func);
}
