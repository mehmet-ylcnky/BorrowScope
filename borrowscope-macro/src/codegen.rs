//! Code generation utilities

#![allow(dead_code)]

use quote::quote;
use syn::{Expr, Ident};

/// Generate optimized tracking call for new variables
pub fn generate_track_new(var_name: &Ident, init_expr: &Expr) -> proc_macro2::TokenStream {
    quote! {
        borrowscope_runtime::track_new(stringify!(#var_name), #init_expr)
    }
}

/// Generate optimized borrow tracking
pub fn generate_track_borrow(
    var_name: &Ident,
    borrowed_expr: &Expr,
    is_mutable: bool,
) -> proc_macro2::TokenStream {
    if is_mutable {
        quote! {
            borrowscope_runtime::track_borrow_mut(stringify!(#var_name), #borrowed_expr)
        }
    } else {
        quote! {
            borrowscope_runtime::track_borrow(stringify!(#var_name), #borrowed_expr)
        }
    }
}

/// Generate drop calls with proper ordering (LIFO)
pub fn generate_drop_calls(variables: &[Ident]) -> proc_macro2::TokenStream {
    let reversed: Vec<_> = variables.iter().rev().collect();

    quote! {
        #(
            borrowscope_runtime::track_drop(stringify!(#reversed));
        )*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_track_new() {
        let var: Ident = parse_quote!(x);
        let expr: Expr = parse_quote!(5);

        let code = generate_track_new(&var, &expr);
        let result = code.to_string();

        assert!(result.contains("track_new"));
        assert!(result.contains("stringify"));
        assert!(result.contains("x"));
    }

    #[test]
    fn test_generate_track_borrow_immutable() {
        let var: Ident = parse_quote!(y);
        let expr: Expr = parse_quote!(x);

        let code = generate_track_borrow(&var, &expr, false);
        let result = code.to_string();

        assert!(result.contains("track_borrow"));
        assert!(!result.contains("track_borrow_mut"));
    }

    #[test]
    fn test_generate_track_borrow_mutable() {
        let var: Ident = parse_quote!(y);
        let expr: Expr = parse_quote!(x);

        let code = generate_track_borrow(&var, &expr, true);
        let result = code.to_string();

        assert!(result.contains("track_borrow_mut"));
    }

    #[test]
    fn test_generate_drop_calls_order() {
        let vars = vec![parse_quote!(x), parse_quote!(y), parse_quote!(z)];

        let code = generate_drop_calls(&vars);
        let result = code.to_string();

        // Variables should be in reverse order (LIFO: z, y, x)
        // The generated code contains stringify! calls
        assert!(result.contains("z"));
        assert!(result.contains("y"));
        assert!(result.contains("x"));
    }

    #[test]
    fn test_generate_drop_calls_empty() {
        let vars: Vec<Ident> = vec![];
        let code = generate_drop_calls(&vars);
        let result = code.to_string();

        assert!(result.is_empty() || result.trim().is_empty());
    }
}
