//! BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.

mod best_practices;
mod borrow_detection;
mod codegen;
mod examples;
mod formatting;
mod hygiene;
mod optimized_transform;
mod parser;
mod pattern;
mod span_utils;
mod validation;
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

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
        assert!(output.contains("stringify"));
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
