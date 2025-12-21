//! Input validation utilities

#![allow(dead_code)]

use syn::{Error, Expr, ItemFn, Local};

/// Validate that an expression is safe to transform
pub fn is_transformable_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Reference(_) | Expr::Path(_) | Expr::Lit(_) | Expr::Call(_) | Expr::MethodCall(_)
    )
}

/// Validate that a local binding can be tracked
pub fn is_trackable_local(local: &Local) -> bool {
    local.init.is_some() && matches!(local.pat, syn::Pat::Ident(_))
}

/// Check if function has a body
pub fn has_body(func: &ItemFn) -> bool {
    !func.block.stmts.is_empty()
}

/// Validate function signature is compatible
pub fn validate_signature(func: &ItemFn) -> syn::Result<()> {
    if func.sig.ident.to_string().is_empty() {
        return Err(Error::new_spanned(
            &func.sig.ident,
            "Function must have a name",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_transformable_expr_reference() {
        let expr: Expr = parse_quote!(&x);
        assert!(is_transformable_expr(&expr));
    }

    #[test]
    fn test_is_transformable_expr_path() {
        let expr: Expr = parse_quote!(x);
        assert!(is_transformable_expr(&expr));
    }

    #[test]
    fn test_is_transformable_expr_literal() {
        let expr: Expr = parse_quote!(42);
        assert!(is_transformable_expr(&expr));
    }

    #[test]
    fn test_is_transformable_expr_call() {
        let expr: Expr = parse_quote!(foo());
        assert!(is_transformable_expr(&expr));
    }

    #[test]
    fn test_is_trackable_local_with_init() {
        let stmt: syn::Stmt = parse_quote!(let x = 5;);
        if let syn::Stmt::Local(local) = stmt {
            assert!(is_trackable_local(&local));
        }
    }

    #[test]
    fn test_is_trackable_local_without_init() {
        let stmt: syn::Stmt = parse_quote!(let x;);
        if let syn::Stmt::Local(local) = stmt {
            assert!(!is_trackable_local(&local));
        }
    }

    #[test]
    fn test_has_body_true() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        assert!(has_body(&func));
    }

    #[test]
    fn test_has_body_false() {
        let func: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(!has_body(&func));
    }

    #[test]
    fn test_validate_signature_valid() {
        let func: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(validate_signature(&func).is_ok());
    }

    #[test]
    fn test_validate_signature_with_params() {
        let func: ItemFn = parse_quote! {
            fn example(x: i32) {}
        };
        assert!(validate_signature(&func).is_ok());
    }

    #[test]
    fn test_validate_signature_with_return() {
        let func: ItemFn = parse_quote! {
            fn example() -> i32 { 42 }
        };
        assert!(validate_signature(&func).is_ok());
    }
}
