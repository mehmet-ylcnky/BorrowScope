//! Borrow detection utilities for finding borrow expressions in various contexts

#![allow(dead_code)]

use syn::{Expr, ExprCall, ExprMethodCall, ExprReference};

/// Information about a borrow expression
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub is_mutable: bool,
    pub context: BorrowContext,
}

/// Context where a borrow occurs
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowContext {
    LetInit,
    FunctionArg,
    MethodArg,
    Return,
    Other,
}

impl BorrowInfo {
    /// Extract borrow information from an expression
    pub fn from_expr(expr: &Expr, context: BorrowContext) -> Option<Self> {
        if let Expr::Reference(reference) = expr {
            Some(BorrowInfo {
                is_mutable: reference.mutability.is_some(),
                context,
            })
        } else {
            None
        }
    }
}

/// Find all borrows in an expression
pub fn find_borrows_in_expr(expr: &Expr) -> Vec<BorrowInfo> {
    let mut borrows = Vec::new();
    collect_borrows(expr, &mut borrows, BorrowContext::Other);
    borrows
}

/// Recursively collect borrows from an expression
fn collect_borrows(expr: &Expr, borrows: &mut Vec<BorrowInfo>, context: BorrowContext) {
    match expr {
        Expr::Reference(ExprReference { mutability, .. }) => {
            borrows.push(BorrowInfo {
                is_mutable: mutability.is_some(),
                context: context.clone(),
            });
        }
        Expr::Call(ExprCall { args, .. }) => {
            for arg in args {
                collect_borrows(arg, borrows, BorrowContext::FunctionArg);
            }
        }
        Expr::MethodCall(ExprMethodCall { args, .. }) => {
            for arg in args {
                collect_borrows(arg, borrows, BorrowContext::MethodArg);
            }
        }
        Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                collect_borrows(expr, borrows, BorrowContext::Return);
            }
        }
        Expr::Array(array) => {
            for elem in &array.elems {
                collect_borrows(elem, borrows, context.clone());
            }
        }
        Expr::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_borrows(elem, borrows, context.clone());
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_borrow() {
        let expr: Expr = parse_quote! { &x };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 1);
        assert!(!borrows[0].is_mutable);
    }

    #[test]
    fn test_mutable_borrow() {
        let expr: Expr = parse_quote! { &mut x };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 1);
        assert!(borrows[0].is_mutable);
    }

    #[test]
    fn test_function_arg_borrow() {
        let expr: Expr = parse_quote! { foo(&x) };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 1);
        assert_eq!(borrows[0].context, BorrowContext::FunctionArg);
    }

    #[test]
    fn test_multiple_borrows() {
        let expr: Expr = parse_quote! { foo(&x, &y) };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 2);
    }

    #[test]
    fn test_method_call_borrow() {
        let expr: Expr = parse_quote! { obj.method(&x) };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 1);
        assert_eq!(borrows[0].context, BorrowContext::MethodArg);
    }

    #[test]
    fn test_array_borrows() {
        let expr: Expr = parse_quote! { vec![&x, &y] };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 2);
    }

    #[test]
    fn test_no_borrows() {
        let expr: Expr = parse_quote! { x + y };
        let borrows = find_borrows_in_expr(&expr);
        assert_eq!(borrows.len(), 0);
    }
}
