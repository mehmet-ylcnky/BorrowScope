//! Optimized code transformation

#![allow(dead_code)]

use quote::quote;
use syn::{Expr, Ident, ItemFn, Local};

use crate::codegen::{generate_drop_calls, generate_track_borrow, generate_track_new};
use crate::pattern::PatternInfo;

/// Transform function with optimizations
pub fn transform_function_optimized(
    func: &ItemFn,
    tracked_vars: &[Ident],
) -> proc_macro2::TokenStream {
    let attrs = &func.attrs;
    let vis = &func.vis;
    let sig = &func.sig;
    let stmts = &func.block.stmts;

    let drop_calls = generate_drop_calls(tracked_vars);

    quote! {
        #(#attrs)*
        #vis #sig {
            #(#stmts)*

            // Cleanup tracking
            #drop_calls
        }
    }
}

/// Transform let statement with optimization
pub fn transform_let_optimized(local: &Local) -> Option<proc_macro2::TokenStream> {
    let pattern_info = PatternInfo::analyze(&local.pat);

    if !pattern_info.is_trackable() {
        return None;
    }

    if !pattern_info.is_simple {
        return None;
    }

    let var_name = &pattern_info.variables[0];
    let init = local.init.as_ref()?;
    let init_expr = &init.expr;

    if let Expr::Reference(reference) = init_expr.as_ref() {
        let is_mutable = reference.mutability.is_some();
        let borrowed_expr = &reference.expr;

        Some(generate_track_borrow(var_name, borrowed_expr, is_mutable))
    } else {
        Some(generate_track_new(var_name, init_expr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Stmt};

    #[test]
    fn test_transform_let_optimized_simple() {
        let stmt: Stmt = parse_quote! { let x = 5; };
        if let Stmt::Local(local) = stmt {
            let result = transform_let_optimized(&local);

            assert!(result.is_some());
            let code = result.unwrap();
            assert!(code.to_string().contains("track_new"));
        } else {
            panic!("Expected Local statement");
        }
    }

    #[test]
    fn test_transform_let_optimized_borrow() {
        let stmt: Stmt = parse_quote! { let y = &x; };
        if let Stmt::Local(local) = stmt {
            let result = transform_let_optimized(&local);

            assert!(result.is_some());
            let code = result.unwrap();
            assert!(code.to_string().contains("track_borrow"));
        } else {
            panic!("Expected Local statement");
        }
    }

    #[test]
    fn test_transform_let_optimized_mut_borrow() {
        let stmt: Stmt = parse_quote! { let y = &mut x; };
        if let Stmt::Local(local) = stmt {
            let result = transform_let_optimized(&local);

            assert!(result.is_some());
            let code = result.unwrap();
            assert!(code.to_string().contains("track_borrow_mut"));
        } else {
            panic!("Expected Local statement");
        }
    }

    #[test]
    fn test_transform_let_optimized_underscore() {
        let stmt: Stmt = parse_quote! { let _x = 5; };
        if let Stmt::Local(local) = stmt {
            let result = transform_let_optimized(&local);
            assert!(result.is_none());
        } else {
            panic!("Expected Local statement");
        }
    }

    #[test]
    fn test_transform_function_optimized() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        let vars = vec![parse_quote!(x)];

        let code = transform_function_optimized(&func, &vars);
        let result = code.to_string();

        assert!(result.contains("fn example"));
        assert!(result.contains("track_drop"));
    }

    #[test]
    fn test_transform_function_optimized_empty_vars() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        let vars: Vec<Ident> = vec![];

        let code = transform_function_optimized(&func, &vars);
        let result = code.to_string();

        assert!(result.contains("fn example"));
    }
}
