//! AST transformation visitor using VisitMut
//!
//! This module implements the OwnershipVisitor that transforms Rust code
//! to inject runtime tracking calls.

use std::collections::HashMap;
use syn::{
    visit_mut::{self, VisitMut},
    Block, Expr, ExprClosure, ExprMethodCall, ExprReference, Ident, Index, ItemFn, Local, Pat,
    Stmt,
};

/// Type of self borrow in method call
#[derive(Debug, Clone, Copy, PartialEq)]
enum SelfBorrowType {
    Immutable,
    Mutable,
    Consuming,
}

/// Visitor that transforms AST to inject tracking calls
pub struct OwnershipVisitor {
    /// Current scope depth (for future drop tracking)
    scope_depth: usize,
    /// Map variable names to their tracking IDs
    var_ids: HashMap<String, usize>,
    /// Counter for generating unique IDs
    next_id: usize,
    /// Stack of scopes, each containing variable names created in that scope
    scope_stack: Vec<Vec<String>>,
    /// Current statement index for inserting statements
    current_stmt_index: usize,
    /// Statements to insert after current statement
    pending_inserts: Vec<(usize, Stmt)>,
}

impl OwnershipVisitor {
    /// Create a new visitor
    pub fn new() -> Self {
        Self {
            scope_depth: 0,
            var_ids: HashMap::new(),
            next_id: 1,
            scope_stack: vec![Vec::new()], // Start with root scope
            current_stmt_index: 0,
            pending_inserts: Vec::new(),
        }
    }

    /// Generate next unique ID
    fn gen_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Extract source location from span (simplified for proc_macro2)
    fn extract_location(_span: proc_macro2::Span) -> String {
        // proc_macro2::Span doesn't have start()/end() methods
        // In a real proc macro, we'd use proc_macro::Span
        // For now, return a placeholder
        "unknown".to_string()
    }

    /// Extract variable name from pattern
    fn extract_pattern_name(pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            Pat::Type(pat_type) => Self::extract_pattern_name(&pat_type.pat),
            _ => "unknown".to_string(),
        }
    }

    /// Extract borrowed variable ID from expression
    #[allow(dead_code)]
    fn extract_borrowed_id(&self, expr: &Expr) -> usize {
        if let Expr::Path(expr_path) = expr {
            if let Some(ident) = expr_path.path.get_ident() {
                let var_name = ident.to_string();
                return *self.var_ids.get(&var_name).unwrap_or(&0);
            }
        }
        0 // Unknown
    }

    /// Check if expression is a simple variable path
    fn is_variable_path(expr: &Expr) -> bool {
        matches!(expr, Expr::Path(_))
    }

    /// Check if pattern is complex (tuple, struct, etc.)
    fn is_complex_pattern(pat: &Pat) -> bool {
        matches!(
            pat,
            Pat::Tuple(_) | Pat::Struct(_) | Pat::TupleStruct(_) | Pat::Slice(_)
        )
    }

    /// Get simple identifier from pattern if possible
    fn get_simple_ident(pat: &Pat) -> Option<String> {
        match pat {
            Pat::Ident(pat_ident) => Some(pat_ident.ident.to_string()),
            Pat::Type(pat_type) => Self::get_simple_ident(&pat_type.pat),
            _ => None,
        }
    }

    /// Build field/tuple access expression
    fn build_access_expr(source: &Ident, indices: &[usize], fields: &[Ident]) -> Expr {
        let mut expr: Expr = syn::parse_quote! { #source };

        for &idx in indices {
            let index = Index::from(idx);
            expr = syn::parse_quote! { #expr.#index };
        }

        for field in fields {
            expr = syn::parse_quote! { #expr.#field };
        }

        expr
    }

    /// Generate destructuring statements for a pattern
    fn generate_destructure_stmts(
        &mut self,
        pat: &Pat,
        source: &Ident,
        indices: &[usize],
        fields: &[Ident],
    ) -> Vec<Stmt> {
        match pat {
            Pat::Tuple(pat_tuple) => {
                let mut stmts = Vec::new();

                for (idx, elem_pat) in pat_tuple.elems.iter().enumerate() {
                    let mut new_indices = indices.to_vec();
                    new_indices.push(idx);

                    if let Some(var_name) = Self::get_simple_ident(elem_pat) {
                        // Simple binding - generate track_new
                        let access_expr = Self::build_access_expr(source, &new_indices, fields);

                        self.var_ids.insert(var_name.clone(), self.next_id);
                        if let Some(current_scope) = self.scope_stack.last_mut() {
                            current_scope.push(var_name.clone());
                        }

                        let stmt: Stmt = syn::parse_quote! {
                            let #elem_pat = borrowscope_runtime::track_new(#var_name, #access_expr);
                        };

                        stmts.push(stmt);
                        self.next_id += 1;
                    } else {
                        // Nested pattern - recurse
                        let nested_stmts =
                            self.generate_destructure_stmts(elem_pat, source, &new_indices, fields);
                        stmts.extend(nested_stmts);
                    }
                }

                stmts
            }
            Pat::Struct(pat_struct) => {
                let mut stmts = Vec::new();

                for field in &pat_struct.fields {
                    let field_name = match &field.member {
                        syn::Member::Named(ident) => ident.clone(),
                        syn::Member::Unnamed(index) => {
                            syn::parse_str(&format!("_{}", index.index)).unwrap()
                        }
                    };

                    let mut new_fields = fields.to_vec();
                    new_fields.push(field_name);

                    if let Some(var_name) = Self::get_simple_ident(&field.pat) {
                        let access_expr = Self::build_access_expr(source, indices, &new_fields);

                        self.var_ids.insert(var_name.clone(), self.next_id);
                        if let Some(current_scope) = self.scope_stack.last_mut() {
                            current_scope.push(var_name.clone());
                        }

                        let pat = &field.pat;
                        let stmt: Stmt = syn::parse_quote! {
                            let #pat = borrowscope_runtime::track_new(#var_name, #access_expr);
                        };

                        stmts.push(stmt);
                        self.next_id += 1;
                    } else {
                        let nested_stmts = self.generate_destructure_stmts(
                            &field.pat,
                            source,
                            indices,
                            &new_fields,
                        );
                        stmts.extend(nested_stmts);
                    }
                }

                stmts
            }
            _ => vec![],
        }
    }

    /// Transform complex pattern into temp + destructuring
    fn transform_complex_pattern(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let temp_name = format!("__pattern_temp_{}", self.next_id);
            let temp_ident: Ident = syn::parse_str(&temp_name).unwrap();

            let original_expr = init.expr.clone();
            let original_pat = local.pat.clone();

            // Replace with temporary variable
            let temp_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(#temp_name, #original_expr)
            };

            local.pat = syn::parse_quote! { #temp_ident };
            *init.expr = temp_expr;

            self.var_ids.insert(temp_name.clone(), self.next_id);
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(temp_name);
            }
            self.next_id += 1;

            // Generate destructuring statements
            let destructure_stmts =
                self.generate_destructure_stmts(&original_pat, &temp_ident, &[], &[]);

            // Insert after current statement - all at the same index since they'll be inserted in reverse
            for stmt in destructure_stmts {
                self.pending_inserts
                    .push((self.current_stmt_index + 1, stmt));
            }
        }
    }

    /// Infer self borrow type from method name using heuristics
    fn infer_self_borrow_type(method_name: &str) -> SelfBorrowType {
        // Immutable borrows (common patterns)
        if method_name.starts_with("as_")
            || method_name.starts_with("to_")
            || method_name.starts_with("is_")
            || method_name.starts_with("get")
            || matches!(
                method_name,
                "len"
                    | "capacity"
                    | "iter"
                    | "chars"
                    | "bytes"
                    | "lines"
                    | "split"
                    | "trim"
                    | "contains"
                    | "starts_with"
                    | "ends_with"
                    | "find"
                    | "clone"
                    | "first"
                    | "last"
            )
        {
            return SelfBorrowType::Immutable;
        }

        // Mutable borrows (common patterns)
        if method_name.starts_with("push")
            || method_name.starts_with("pop")
            || method_name.starts_with("insert")
            || method_name.starts_with("remove")
            || method_name.starts_with("append")
            || matches!(
                method_name,
                "clear" | "truncate" | "extend" | "drain" | "sort" | "reverse" | "dedup" | "retain"
            )
        {
            return SelfBorrowType::Mutable;
        }

        // Consuming methods (common patterns)
        if method_name.starts_with("into_") || matches!(method_name, "unwrap" | "expect") {
            return SelfBorrowType::Consuming;
        }

        // Default: immutable borrow
        SelfBorrowType::Immutable
    }

    /// Check if expression is a simple variable (not a temporary or field access)
    fn is_simple_variable(expr: &Expr) -> bool {
        matches!(expr, Expr::Path(_))
    }

    /// Extract receiver variable name from expression
    fn extract_receiver_name(receiver: &Expr) -> Option<String> {
        if let Expr::Path(path) = receiver {
            if let Some(ident) = path.path.get_ident() {
                return Some(ident.to_string());
            }
        }
        None
    }

    /// Transform method call to track self borrows
    fn transform_method_call(&mut self, method_call: &mut ExprMethodCall) {
        // Only track method calls on simple variables
        if !Self::is_simple_variable(&method_call.receiver) {
            // Visit receiver and arguments normally
            self.visit_expr_mut(&mut method_call.receiver);
            for arg in &mut method_call.args {
                self.visit_expr_mut(arg);
            }
            return;
        }

        let method_name = method_call.method.to_string();
        let borrow_type = Self::infer_self_borrow_type(&method_name);

        // For consuming methods, just visit normally (move tracking happens at assignment level)
        if borrow_type == SelfBorrowType::Consuming {
            self.visit_expr_mut(&mut method_call.receiver);
            for arg in &mut method_call.args {
                self.visit_expr_mut(arg);
            }
            return;
        }

        // Extract receiver name for tracking
        if Self::extract_receiver_name(&method_call.receiver).is_some() {
            let receiver_expr = method_call.receiver.clone();

            // Wrap receiver with appropriate borrow tracking
            let wrapped_receiver: Expr = match borrow_type {
                SelfBorrowType::Immutable => {
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow("method_borrow", &#receiver_expr)
                    }
                }
                SelfBorrowType::Mutable => {
                    syn::parse_quote! {
                        borrowscope_runtime::track_borrow_mut("method_borrow", &mut #receiver_expr)
                    }
                }
                SelfBorrowType::Consuming => unreachable!(),
            };

            method_call.receiver = Box::new(wrapped_receiver);
        }

        // Visit arguments
        for arg in &mut method_call.args {
            self.visit_expr_mut(arg);
        }
    }

    /// Check if closure has move keyword
    #[allow(dead_code)]
    fn is_move_closure(closure: &ExprClosure) -> bool {
        closure.capture.is_some()
    }

    /// Extract variables used in closure body
    fn extract_captured_vars(&self, expr: &Expr, vars: &mut Vec<String>) {
        match expr {
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    // Check if it's a known variable (not a parameter or function)
                    if self.var_ids.contains_key(&var_name) && !vars.contains(&var_name) {
                        vars.push(var_name);
                    }
                }
            }
            Expr::Binary(binary) => {
                self.extract_captured_vars(&binary.left, vars);
                self.extract_captured_vars(&binary.right, vars);
            }
            Expr::Unary(unary) => {
                self.extract_captured_vars(&unary.expr, vars);
            }
            Expr::Call(call) => {
                self.extract_captured_vars(&call.func, vars);
                for arg in &call.args {
                    self.extract_captured_vars(arg, vars);
                }
            }
            Expr::MethodCall(method) => {
                self.extract_captured_vars(&method.receiver, vars);
                for arg in &method.args {
                    self.extract_captured_vars(arg, vars);
                }
            }
            Expr::Block(block) => {
                for stmt in &block.block.stmts {
                    match stmt {
                        Stmt::Local(local) => {
                            if let Some(init) = &local.init {
                                self.extract_captured_vars(&init.expr, vars);
                            }
                        }
                        Stmt::Expr(expr, _) => {
                            self.extract_captured_vars(expr, vars);
                        }
                        _ => {}
                    }
                }
            }
            Expr::If(if_expr) => {
                self.extract_captured_vars(&if_expr.cond, vars);
                for stmt in &if_expr.then_branch.stmts {
                    if let Stmt::Expr(expr, _) = stmt {
                        self.extract_captured_vars(expr, vars);
                    }
                }
                if let Some((_, else_branch)) = &if_expr.else_branch {
                    self.extract_captured_vars(else_branch, vars);
                }
            }
            Expr::Match(match_expr) => {
                self.extract_captured_vars(&match_expr.expr, vars);
                for arm in &match_expr.arms {
                    self.extract_captured_vars(&arm.body, vars);
                }
            }
            Expr::Field(field) => {
                self.extract_captured_vars(&field.base, vars);
            }
            Expr::Index(index) => {
                self.extract_captured_vars(&index.expr, vars);
                self.extract_captured_vars(&index.index, vars);
            }
            Expr::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.extract_captured_vars(expr, vars);
                }
            }
            _ => {}
        }
    }

    /// Transform closure expression
    fn transform_closure(&mut self, closure: &mut ExprClosure) {
        // Extract captured variables
        let mut captured_vars = Vec::new();
        self.extract_captured_vars(&closure.body, &mut captured_vars);

        // For move closures, the variables are moved into the closure
        // This is tracked at the assignment level (let closure = move |x| ...)
        // For non-move closures, variables are borrowed
        // We don't transform the closure body itself to avoid complexity

        // Just visit the closure body normally to handle any nested structures
        self.visit_expr_mut(&mut closure.body);

        // Note: We could add metadata tracking here for captured variables
        // but for v1, we keep it simple and let the outer scope tracking handle it
    }

    /// Check if expression is a potential move (simple variable path)
    fn is_potential_move(expr: &Expr) -> bool {
        matches!(expr, Expr::Path(_))
    }

    /// Transform a let statement to inject track_new
    fn transform_local(&mut self, local: &mut Local) {
        // Only transform if there's an initializer
        if let Some(init) = &mut local.init {
            // Check if this is a complex pattern
            if Self::is_complex_pattern(&local.pat) {
                self.transform_complex_pattern(local);
                return;
            }

            let var_name = Self::extract_pattern_name(&local.pat);
            let var_id = self.gen_id();

            // Store variable ID for later reference
            self.var_ids.insert(var_name.clone(), var_id);

            // Add to current scope for drop tracking
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(var_name.clone());
            }

            let original_expr = &init.expr;

            // Check if this is a potential move (assignment from another variable)
            let new_expr: Expr = if Self::is_potential_move(original_expr) {
                // Extract source variable name
                if let Expr::Path(path_expr) = original_expr.as_ref() {
                    if let Some(source_ident) = path_expr.path.get_ident() {
                        let source_name = source_ident.to_string();
                        // Wrap with track_move
                        syn::parse_quote! {
                            borrowscope_runtime::track_move(#source_name, #var_name, #original_expr)
                        }
                    } else {
                        // Not a simple identifier, just track_new
                        syn::parse_quote! {
                            borrowscope_runtime::track_new(#var_name, #original_expr)
                        }
                    }
                } else {
                    syn::parse_quote! {
                        borrowscope_runtime::track_new(#var_name, #original_expr)
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

    /// Transform reference expressions to inject track_borrow
    fn transform_reference(&mut self, expr: &mut Expr, ref_expr: &ExprReference) {
        // Only track borrows of simple variables
        if !Self::is_variable_path(&ref_expr.expr) {
            return;
        }

        let is_mutable = ref_expr.mutability.is_some();
        let borrowed_expr = &ref_expr.expr;

        // Generate tracking call
        let tracking_call: Expr = if is_mutable {
            syn::parse_quote! {
                borrowscope_runtime::track_borrow_mut("borrow", &mut #borrowed_expr)
            }
        } else {
            syn::parse_quote! {
                borrowscope_runtime::track_borrow("borrow", &#borrowed_expr)
            }
        };

        *expr = tracking_call;
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

        // Push new scope
        self.scope_stack.push(Vec::new());

        // Clear pending inserts for this block
        self.pending_inserts.clear();

        // Visit all statements in the block
        for (idx, stmt) in block.stmts.iter_mut().enumerate() {
            self.current_stmt_index = idx;
            self.visit_stmt_mut(stmt);
        }

        // Insert pending statements in reverse order to maintain indices
        for (idx, stmt) in self.pending_inserts.drain(..).rev() {
            block.stmts.insert(idx, stmt);
        }

        // Pop scope and insert drops in LIFO order
        if let Some(scope_vars) = self.scope_stack.pop() {
            // Check if the last statement is an expression without semicolon (implicit return)
            let has_trailing_expr = block
                .stmts
                .last()
                .map(|stmt| matches!(stmt, Stmt::Expr(_, None)))
                .unwrap_or(false);

            if has_trailing_expr && !scope_vars.is_empty() {
                // Insert drops before the last expression
                let last_stmt = block.stmts.pop();
                for var_name in scope_vars.into_iter().rev() {
                    let drop_stmt: Stmt = syn::parse_quote! {
                        borrowscope_runtime::track_drop(#var_name);
                    };
                    block.stmts.push(drop_stmt);
                }
                // Re-add the last expression
                if let Some(stmt) = last_stmt {
                    block.stmts.push(stmt);
                }
            } else {
                // No trailing expression, just append drops
                for var_name in scope_vars.into_iter().rev() {
                    let drop_stmt: Stmt = syn::parse_quote! {
                        borrowscope_runtime::track_drop(#var_name);
                    };
                    block.stmts.push(drop_stmt);
                }
            }
        }

        self.scope_depth -= 1;
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                // Transform the local statement
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
        // Handle closures before default traversal
        if let Expr::Closure(closure) = expr {
            self.transform_closure(closure);
            return;
        }

        // Handle method calls before default traversal
        if let Expr::MethodCall(method_call) = expr {
            self.transform_method_call(method_call);
            return;
        }

        // First recursively visit nested expressions
        visit_mut::visit_expr_mut(self, expr);

        // Then transform reference expressions at this level
        if let Expr::Reference(ref_expr) = expr.clone() {
            self.transform_reference(expr, &ref_expr);
        }
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
