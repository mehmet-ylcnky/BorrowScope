//! Examples demonstrating syn and quote usage
//!
//! This module contains helper functions that show how to use syn for parsing
//! and quote for code generation.

#![allow(dead_code)]

use quote::quote;
use syn::{Expr, ItemFn, Local, Pat, Stmt};

/// Extract function name from a parsed function
pub fn get_function_name(func: &ItemFn) -> String {
    func.sig.ident.to_string()
}

/// Count the number of statements in a function body
pub fn count_statements(func: &ItemFn) -> usize {
    func.block.stmts.len()
}

/// Check if a statement is a let binding
pub fn is_let_binding(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Local(_))
}

/// Extract variable name from a let binding
pub fn extract_variable_name(stmt: &Stmt) -> Option<String> {
    if let Stmt::Local(Local {
        pat: Pat::Ident(pat_ident),
        ..
    }) = stmt
    {
        return Some(pat_ident.ident.to_string());
    }
    None
}

/// Check if an expression is a borrow (&x)
pub fn is_borrow_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Reference(_))
}

/// Generate a simple function using quote
pub fn generate_hello_function() -> proc_macro2::TokenStream {
    quote! {
        fn hello() {
            println!("Hello from generated code!");
        }
    }
}

/// Generate a function with a parameter
pub fn generate_function_with_param(name: &str, param_name: &str) -> proc_macro2::TokenStream {
    let fn_name = syn::Ident::new(name, proc_macro2::Span::call_site());
    let param = syn::Ident::new(param_name, proc_macro2::Span::call_site());

    quote! {
        fn #fn_name(#param: i32) -> i32 {
            #param * 2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_get_function_name() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        assert_eq!(get_function_name(&func), "example");
    }

    #[test]
    fn test_count_statements() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                let y = 10;
                println!("{}", x);
            }
        };
        assert_eq!(count_statements(&func), 3);
    }

    #[test]
    fn test_is_let_binding() {
        let stmt: Stmt = parse_quote! { let x = 5; };
        assert!(is_let_binding(&stmt));

        let stmt: Stmt = parse_quote! { println!("hello"); };
        assert!(!is_let_binding(&stmt));
    }

    #[test]
    fn test_extract_variable_name() {
        let stmt: Stmt = parse_quote! { let x = 5; };
        assert_eq!(extract_variable_name(&stmt), Some("x".to_string()));

        let stmt: Stmt = parse_quote! { let my_var = 10; };
        assert_eq!(extract_variable_name(&stmt), Some("my_var".to_string()));

        let stmt: Stmt = parse_quote! { println!("hello"); };
        assert_eq!(extract_variable_name(&stmt), None);
    }

    #[test]
    fn test_is_borrow_expr() {
        let expr: Expr = parse_quote! { &x };
        assert!(is_borrow_expr(&expr));

        let expr: Expr = parse_quote! { &mut x };
        assert!(is_borrow_expr(&expr));

        let expr: Expr = parse_quote! { x };
        assert!(!is_borrow_expr(&expr));
    }

    #[test]
    fn test_generate_hello_function() {
        let generated = generate_hello_function();
        let generated_str = generated.to_string();
        assert!(generated_str.contains("fn hello"));
        assert!(generated_str.contains("Hello from generated code"));
    }

    #[test]
    fn test_generate_function_with_param() {
        let generated = generate_function_with_param("double", "value");
        let generated_str = generated.to_string();
        assert!(generated_str.contains("fn double"));
        assert!(generated_str.contains("value : i32"));
        assert!(generated_str.contains("value * 2"));
    }
}
