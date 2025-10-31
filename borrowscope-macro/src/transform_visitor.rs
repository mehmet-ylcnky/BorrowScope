//! AST transformation visitor using VisitMut
//!
//! This module implements the OwnershipVisitor that transforms Rust code
//! to inject runtime tracking calls.

use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ItemFn, Local, Pat, Stmt,
};

/// Visitor that transforms AST to inject tracking calls
pub struct OwnershipVisitor {
    /// Current scope depth (for future drop tracking)
    scope_depth: usize,
}

impl OwnershipVisitor {
    /// Create a new visitor
    pub fn new() -> Self {
        Self { scope_depth: 0 }
    }

    /// Extract variable name from pattern
    fn extract_pattern_name(pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            Pat::Type(pat_type) => Self::extract_pattern_name(&pat_type.pat),
            _ => "unknown".to_string(),
        }
    }

    /// Transform a let statement to inject track_new
    fn transform_local(&mut self, local: &mut Local) {
        // Only transform if there's an initializer
        if let Some(init) = &mut local.init {
            let var_name = Self::extract_pattern_name(&local.pat);
            let original_expr = &init.expr;

            // Check if this is a borrow expression
            let new_expr: Expr = if let Expr::Reference(ref_expr) = original_expr.as_ref() {
                let borrowed = &ref_expr.expr;
                if ref_expr.mutability.is_some() {
                    // Mutable borrow
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow_mut(#var_name, &mut #borrowed)
                    }
                } else {
                    // Immutable borrow
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow(#var_name, &#borrowed)
                    }
                }
            } else {
                // Regular variable creation
                syn::parse_quote! {
                    borrowscope_runtime::track_new(#var_name, #original_expr)
                }
            };

            *init.expr = new_expr;
        }

        // Continue visiting nested expressions
        visit_mut::visit_local_mut(self, local);
    }
}

impl Default for OwnershipVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitMut for OwnershipVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        // Only visit the function body, not nested items
        self.visit_block_mut(&mut func.block);
    }

    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;

        // Visit all statements in the block
        for stmt in &mut block.stmts {
            self.visit_stmt_mut(stmt);
        }

        self.scope_depth -= 1;
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                // Transform let statements
                self.transform_local(local);
            }
            Stmt::Expr(expr, _) => {
                // Visit expressions in statements
                self.visit_expr_mut(expr);
            }
            _ => {
                // Use default visitor for other statement types
                visit_mut::visit_stmt_mut(self, stmt);
            }
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // For now, just recursively visit nested expressions
        // We'll add more transformation logic in later sections
        visit_mut::visit_expr_mut(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_simple_let_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut stmt: Stmt = parse_quote! {
            let x = 42;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        assert!(output.contains("track_new"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_multiple_variables() {
        let mut visitor = OwnershipVisitor::new();

        let mut block: Block = parse_quote! {
            {
                let x = 42;
                let y = 100;
            }
        };

        visitor.visit_block_mut(&mut block);

        let output = block.to_token_stream().to_string();

        // Should have two track_new calls
        assert!(output.contains("track_new"));
        assert_eq!(output.matches("track_new").count(), 2);
    }

    #[test]
    fn test_nested_blocks() {
        let mut visitor = OwnershipVisitor::new();

        let mut block: Block = parse_quote! {
            {
                let x = 42;
                {
                    let y = 100;
                }
            }
        };

        visitor.visit_block_mut(&mut block);

        let output = block.to_token_stream().to_string();
        assert_eq!(output.matches("track_new").count(), 2);
    }

    #[test]
    fn test_borrow_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut stmt: Stmt = parse_quote! {
            let r = &x;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        assert!(output.contains("track_borrow"));
    }

    #[test]
    fn test_mut_borrow_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut stmt: Stmt = parse_quote! {
            let r = &mut x;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        assert!(output.contains("track_borrow_mut"));
    }

    #[test]
    fn test_scope_depth_tracking() {
        let mut visitor = OwnershipVisitor::new();
        assert_eq!(visitor.scope_depth, 0);

        let mut block: Block = parse_quote! {
            {
                let x = 42;
            }
        };

        visitor.visit_block_mut(&mut block);

        // Should return to 0 after visiting
        assert_eq!(visitor.scope_depth, 0);
    }

    #[test]
    fn test_extract_pattern_name() {
        let pat: Pat = parse_quote! { x };
        assert_eq!(OwnershipVisitor::extract_pattern_name(&pat), "x");

        let pat: Pat = parse_quote! { my_var };
        assert_eq!(OwnershipVisitor::extract_pattern_name(&pat), "my_var");
    }

    #[test]
    fn test_extract_pattern_name_with_type() {
        let stmt: Stmt = parse_quote! {
            let x: i32 = 5;
        };
        
        if let Stmt::Local(local) = stmt {
            assert_eq!(OwnershipVisitor::extract_pattern_name(&local.pat), "x");
        } else {
            panic!("Expected Local statement");
        }
    }

    #[test]
    fn test_no_transform_without_init() {
        let mut visitor = OwnershipVisitor::new();

        let mut stmt: Stmt = parse_quote! {
            let x;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        // Should not add tracking for uninitialized variables
        assert!(!output.contains("track_new"));
    }

    #[test]
    fn test_preserves_complex_expressions() {
        let mut visitor = OwnershipVisitor::new();

        let mut stmt: Stmt = parse_quote! {
            let x = expensive_function(a, b, c);
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        assert!(output.contains("track_new"));
        assert!(output.contains("expensive_function"));
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
    }
}
