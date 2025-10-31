//! AST visitor for traversing function bodies
//!
//! This module provides utilities for walking through the Abstract Syntax Tree
//! of a function to find and analyze statements and expressions.

#![allow(dead_code)]

use syn::{Expr, ItemFn, Local, Stmt};

/// Statistics about a function's AST
#[derive(Debug, Clone, PartialEq)]
pub struct AstStats {
    pub total_statements: usize,
    pub let_bindings: usize,
    pub expressions: usize,
    pub borrow_count: usize,
}

/// Visit all statements in a function and collect statistics
pub fn analyze_function(func: &ItemFn) -> AstStats {
    let mut stats = AstStats {
        total_statements: 0,
        let_bindings: 0,
        expressions: 0,
        borrow_count: 0,
    };

    for stmt in &func.block.stmts {
        stats.total_statements += 1;
        visit_statement(stmt, &mut stats);
    }

    stats
}

/// Visit a single statement and update statistics
fn visit_statement(stmt: &Stmt, stats: &mut AstStats) {
    match stmt {
        Stmt::Local(local) => {
            stats.let_bindings += 1;
            if let Some(init) = &local.init {
                visit_expression(&init.expr, stats);
            }
        }
        Stmt::Expr(expr, _) => {
            stats.expressions += 1;
            visit_expression(expr, stats);
        }
        _ => {}
    }
}

/// Visit an expression and count borrows
fn visit_expression(expr: &Expr, stats: &mut AstStats) {
    match expr {
        Expr::Reference(_) => {
            stats.borrow_count += 1;
        }
        Expr::Block(block) => {
            for stmt in &block.block.stmts {
                visit_statement(stmt, stats);
            }
        }
        Expr::If(expr_if) => {
            visit_expression(&expr_if.cond, stats);
            for stmt in &expr_if.then_branch.stmts {
                visit_statement(stmt, stats);
            }
            if let Some((_, else_branch)) = &expr_if.else_branch {
                visit_expression(else_branch, stats);
            }
        }
        Expr::Match(expr_match) => {
            visit_expression(&expr_match.expr, stats);
            for arm in &expr_match.arms {
                visit_expression(&arm.body, stats);
            }
        }
        Expr::Loop(expr_loop) => {
            for stmt in &expr_loop.body.stmts {
                visit_statement(stmt, stats);
            }
        }
        Expr::ForLoop(expr_for) => {
            visit_expression(&expr_for.expr, stats);
            for stmt in &expr_for.body.stmts {
                visit_statement(stmt, stats);
            }
        }
        Expr::While(expr_while) => {
            visit_expression(&expr_while.cond, stats);
            for stmt in &expr_while.body.stmts {
                visit_statement(stmt, stats);
            }
        }
        _ => {}
    }
}

/// Find all let bindings in a function
pub fn find_let_bindings(func: &ItemFn) -> Vec<String> {
    let mut bindings = Vec::new();
    for stmt in &func.block.stmts {
        if let Stmt::Local(Local { pat, .. }) = stmt {
            if let syn::Pat::Ident(pat_ident) = pat {
                bindings.push(pat_ident.ident.to_string());
            }
        }
    }
    bindings
}

/// Count borrow expressions in a function
pub fn count_borrows(func: &ItemFn) -> usize {
    let stats = analyze_function(func);
    stats.borrow_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_analyze_simple_function() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                let y = 10;
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.total_statements, 2);
        assert_eq!(stats.let_bindings, 2);
    }

    #[test]
    fn test_analyze_function_with_borrows() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = String::from("hello");
                let r1 = &x;
                let r2 = &x;
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.let_bindings, 3);
        assert_eq!(stats.borrow_count, 2);
    }

    #[test]
    fn test_analyze_function_with_expressions() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                println!("{}", x);
                x + 1;
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.total_statements, 3);
        assert_eq!(stats.let_bindings, 1);
        assert_eq!(stats.expressions, 2);
    }

    #[test]
    fn test_analyze_function_with_control_flow() {
        let func: ItemFn = parse_quote! {
            fn example(condition: bool) {
                if condition {
                    let x = 10;
                } else {
                    let y = 20;
                }
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.total_statements, 1);
        assert_eq!(stats.let_bindings, 2);
    }

    #[test]
    fn test_find_let_bindings() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                let y = 10;
                let result = x + y;
            }
        };
        let bindings = find_let_bindings(&func);
        assert_eq!(bindings, vec!["x", "y", "result"]);
    }

    #[test]
    fn test_find_let_bindings_empty() {
        let func: ItemFn = parse_quote! {
            fn example() {
                println!("hello");
            }
        };
        let bindings = find_let_bindings(&func);
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_count_borrows() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = vec![1, 2, 3];
                let r1 = &x;
                let r2 = &x;
                let r3 = &x;
            }
        };
        let count = count_borrows(&func);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_borrows_with_mut() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let mut x = 5;
                let r = &mut x;
                *r += 1;
            }
        };
        let count = count_borrows(&func);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_analyze_nested_blocks() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                {
                    let y = 10;
                    let z = &y;
                }
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.let_bindings, 3);
        assert_eq!(stats.borrow_count, 1);
    }

    #[test]
    fn test_analyze_loop() {
        let func: ItemFn = parse_quote! {
            fn example() {
                for i in 0..5 {
                    let x = i * 2;
                }
            }
        };
        let stats = analyze_function(&func);
        assert_eq!(stats.let_bindings, 1);
    }
}
