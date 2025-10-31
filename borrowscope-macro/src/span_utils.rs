//! Span utilities for better error messages

#![allow(dead_code)]

use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Expr, Ident};

/// Generate code with preserved span for better error messages
pub fn generate_with_span(span: Span, var_name: &Ident, expr: &Expr) -> proc_macro2::TokenStream {
    quote_spanned! { span =>
        borrowscope_runtime::track_new(stringify!(#var_name), #expr)
    }
}

/// Create identifier with specific span
pub fn ident_with_span(name: &str, span: Span) -> Ident {
    Ident::new(name, span)
}

/// Get span from expression
pub fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Path(path) => path
            .path
            .segments
            .first()
            .map(|seg| seg.ident.span())
            .unwrap_or_else(Span::call_site),
        Expr::Lit(lit) => lit.lit.span(),
        _ => expr.span(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_ident_with_span() {
        let span = Span::call_site();
        let ident = ident_with_span("test", span);
        assert_eq!(ident.to_string(), "test");
    }

    #[test]
    fn test_ident_with_span_different_names() {
        let span = Span::call_site();
        let ident1 = ident_with_span("foo", span);
        let ident2 = ident_with_span("bar", span);
        assert_eq!(ident1.to_string(), "foo");
        assert_eq!(ident2.to_string(), "bar");
    }

    #[test]
    fn test_expr_span_path() {
        let expr: Expr = parse_quote!(x);
        let _span = expr_span(&expr);
        // Verify it doesn't panic
    }

    #[test]
    fn test_expr_span_literal() {
        let expr: Expr = parse_quote!(42);
        let _span = expr_span(&expr);
        // Verify it doesn't panic
    }

    #[test]
    fn test_generate_with_span() {
        let span = Span::call_site();
        let var: Ident = parse_quote!(x);
        let expr: Expr = parse_quote!(5);

        let code = generate_with_span(span, &var, &expr);
        let result = code.to_string();

        assert!(result.contains("track_new"));
        assert!(result.contains("x"));
    }
}
