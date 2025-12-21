//! AST transformation visitor using VisitMut
//!
//! This module implements the OwnershipVisitor that transforms Rust code
//! to inject runtime tracking calls.

use crate::smart_pointer::{detect_rc_clone, detect_smart_pointer_new, SmartPointerType};
use std::collections::HashMap;
use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprCall, ExprCast, ExprClosure, ExprMethodCall, ExprReference, ExprUnsafe,
    Ident, Index, ItemFn, Local, Pat, Stmt,
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

    /// Generate location expression that will be evaluated at compile time
    /// Returns a token stream that produces the location string
    fn location_tokens(span: proc_macro2::Span) -> proc_macro2::TokenStream {
        // Use file!() and line!() macros which are evaluated at the call site
        syn::parse_quote_spanned! { span =>
            concat!(file!(), ":", line!())
        }
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
                "clear"
                    | "truncate"
                    | "extend"
                    | "drain"
                    | "sort"
                    | "reverse"
                    | "dedup"
                    | "retain"
                    | "tick"
                    | "recv"
                    | "send"
                    | "changed"
                    | "wait"
                    | "acquire"
                    | "lock"
                    | "write"
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

    /// Transform a let statement to inject track_new_with_id
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
            let location = Self::location_tokens(local.pat.span());

            // Store variable ID for later reference
            self.var_ids.insert(var_name.clone(), var_id);

            // Add to current scope for drop tracking
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(var_name.clone());
            }

            let original_expr = &init.expr;

            // Check for smart pointer operations first
            if let Some(sp_type) = detect_smart_pointer_new(original_expr) {
                let new_expr = match sp_type {
                    SmartPointerType::Rc => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_rc_new_with_id(#var_id, #var_name, "Rc<T>", #location, #original_expr)
                        }
                    }
                    SmartPointerType::Arc => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_arc_new_with_id(#var_id, #var_name, "Arc<T>", #location, #original_expr)
                        }
                    }
                    SmartPointerType::RefCell => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_refcell_new(#var_name, std::cell::RefCell::new(#original_expr))
                        }
                    }
                    SmartPointerType::Cell => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_cell_new(#var_name, std::cell::Cell::new(#original_expr))
                        }
                    }
                    _ => {
                        // Box and others use regular tracking
                        syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        }
                    }
                };
                *init.expr = new_expr;
            } else if let Some(sp_type) = detect_rc_clone(original_expr) {
                // Extract source ID from Rc::clone(&x) or Arc::clone(&x)
                let source_id = self.extract_clone_source_id(original_expr);
                let new_expr = match sp_type {
                    SmartPointerType::Rc => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_rc_clone_with_id(#var_id, #source_id, #var_name, #location, #original_expr)
                        }
                    }
                    SmartPointerType::Arc => {
                        syn::parse_quote! {
                            borrowscope_runtime::track_arc_clone_with_id(#var_id, #source_id, #var_name, #location, #original_expr)
                        }
                    }
                    _ => {
                        syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        }
                    }
                };
                *init.expr = new_expr;
            } else if Self::is_potential_move(original_expr) {
                // Check if this is a potential move (assignment from another variable)
                // Extract source variable name and ID
                if let Expr::Path(path_expr) = original_expr.as_ref() {
                    if let Some(source_ident) = path_expr.path.get_ident() {
                        let source_name = source_ident.to_string();
                        if let Some(&source_id) = self.var_ids.get(&source_name) {
                            // Use advanced move API with IDs
                            let new_expr: Expr = syn::parse_quote! {
                                borrowscope_runtime::track_move_with_id(#source_id, #var_id, #var_name, #location, #original_expr)
                            };
                            *init.expr = new_expr;
                        } else {
                            // Fallback to simple API if source ID not found
                            let new_expr: Expr = syn::parse_quote! {
                                borrowscope_runtime::track_move(#source_name, #var_name, #original_expr)
                            };
                            *init.expr = new_expr;
                        }
                    } else {
                        // Not a simple identifier - use helper function that extracts type
                        let new_expr: Expr = syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        };
                        *init.expr = new_expr;
                    }
                } else {
                    let new_expr: Expr = syn::parse_quote! {
                        borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                    };
                    *init.expr = new_expr;
                }
            } else {
                // Regular variable creation - use helper function
                let new_expr: Expr = syn::parse_quote! {
                    borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                };
                *init.expr = new_expr;
            }
        }

        // Continue visiting nested expressions
        visit_mut::visit_local_mut(self, local);
    }

    /// Extract source variable ID from Rc::clone(&x) or Arc::clone(&x)
    fn extract_clone_source_id(&self, expr: &Expr) -> usize {
        if let Expr::Call(call) = expr {
            if let Some(Expr::Reference(ref_expr)) = call.args.first() {
                if let Expr::Path(path) = ref_expr.expr.as_ref() {
                    if let Some(ident) = path.path.get_ident() {
                        let var_name = ident.to_string();
                        return *self.var_ids.get(&var_name).unwrap_or(&0);
                    }
                }
            }
        }
        0
    }

    /// Transform reference expressions to inject track_borrow_with_id
    fn transform_reference(&mut self, expr: &mut Expr, ref_expr: &ExprReference) {
        // Only track borrows of simple variables
        if !Self::is_variable_path(&ref_expr.expr) {
            return;
        }

        let is_mutable = ref_expr.mutability.is_some();
        let borrowed_expr = &ref_expr.expr;
        let location = Self::location_tokens(ref_expr.span());

        // Try to get owner ID
        let owner_id = if let Expr::Path(path) = borrowed_expr.as_ref() {
            if let Some(ident) = path.path.get_ident() {
                self.var_ids.get(&ident.to_string()).copied()
            } else {
                None
            }
        } else {
            None
        };

        // Generate tracking call
        let tracking_call: Expr = if let Some(owner_id) = owner_id {
            let borrower_id = self.gen_id();
            // Use advanced API with IDs
            if is_mutable {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_mut_with_id(#borrower_id, #owner_id, "borrow", #location, &mut #borrowed_expr)
                }
            } else {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_with_id(#borrower_id, #owner_id, "borrow", #location, false, &#borrowed_expr)
                }
            }
        } else {
            // Fallback to simple API if owner ID not found
            if is_mutable {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_mut("borrow", &mut #borrowed_expr)
                }
            } else {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow("borrow", &#borrowed_expr)
                }
            }
        };

        *expr = tracking_call;
    }

    /// Transform unsafe block to add enter/exit tracking
    fn transform_unsafe_block(&mut self, unsafe_expr: &mut ExprUnsafe) {
        let block_id = self.gen_id();
        let location = Self::location_tokens(unsafe_expr.unsafe_token.span);
        let inner_block = &unsafe_expr.block;

        // Replace block content with tracked version
        unsafe_expr.block = syn::parse_quote! {
            {
                borrowscope_runtime::track_unsafe_block_enter(#block_id, #location);
                let __unsafe_result = #inner_block;
                borrowscope_runtime::track_unsafe_block_exit(#block_id, #location);
                __unsafe_result
            }
        };
    }

    /// Transform raw pointer cast expressions
    fn transform_ptr_cast(&mut self, expr: &mut Expr, cast_expr: &ExprCast) {
        // Check if casting to raw pointer type
        if let syn::Type::Ptr(type_ptr) = cast_expr.ty.as_ref() {
            let ptr_id = self.gen_id();
            let location = Self::location_tokens(cast_expr.as_token.span);
            let inner = &cast_expr.expr;
            let ty = &cast_expr.ty;
            let ptr_type = quote::quote!(#ty).to_string();
            let var_name = format!("ptr_{}", ptr_id);

            if type_ptr.mutability.is_some() {
                *expr = syn::parse_quote! {
                    borrowscope_runtime::track_raw_ptr_mut(#var_name, #ptr_id, #ptr_type, #location, #inner as #ty)
                };
            } else {
                *expr = syn::parse_quote! {
                    borrowscope_runtime::track_raw_ptr(#var_name, #ptr_id, #ptr_type, #location, #inner as #ty)
                };
            }
        }
    }

    /// Transform call expressions (transmute only - FFI/unsafe fn require type info)
    fn transform_call_expr(&mut self, expr: &mut Expr, call_expr: &ExprCall) {
        if let Expr::Path(path) = call_expr.func.as_ref() {
            let path_str = quote::quote!(#path).to_string();

            // Check for transmute (reliably detectable by name)
            if path_str.contains("transmute") {
                let location = Self::location_tokens(
                    path.path
                        .segments
                        .last()
                        .map(|s| s.ident.span())
                        .unwrap_or_else(proc_macro2::Span::call_site),
                );
                let args = &call_expr.args;
                let func = &call_expr.func;

                *expr = syn::parse_quote! {
                    {
                        borrowscope_runtime::track_transmute("unknown", "unknown", #location);
                        #func(#args)
                    }
                };
            }
            // Note: FFI calls and unsafe fn calls cannot be detected without type information
        }
    }

    /// Transform async block expressions
    fn transform_async_block(&mut self, async_expr: &mut syn::ExprAsync) {
        let block_id = self.gen_id();
        let location = Self::location_tokens(async_expr.async_token.span);

        // Visit the inner block first
        self.visit_block_mut(&mut async_expr.block);

        // Wrap the block content with tracking
        let inner_stmts = &async_expr.block.stmts;
        async_expr.block = syn::parse_quote! {
            {
                borrowscope_runtime::track_async_block_enter(#block_id, #location);
                let __async_result = { #(#inner_stmts)* };
                borrowscope_runtime::track_async_block_exit(#block_id, #location);
                __async_result
            }
        };
    }

    /// Transform await expressions
    fn transform_await(&mut self, expr: &mut Expr, await_expr: &syn::ExprAwait) {
        let await_id = self.gen_id();
        let location = Self::location_tokens(await_expr.await_token.span);
        let base = &await_expr.base;
        let future_name = Self::extract_future_name(base);

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_await_start(#await_id, #future_name, #location);
                let __await_result = #base.await;
                borrowscope_runtime::track_await_end(#await_id, #location);
                __await_result
            }
        };
    }

    /// Extract a name for the future being awaited
    fn extract_future_name(expr: &Expr) -> String {
        match expr {
            Expr::Path(path) => quote::quote!(#path).to_string().replace(' ', ""),
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    quote::quote!(#path).to_string().replace(' ', "")
                } else {
                    "future".to_string()
                }
            }
            Expr::MethodCall(method) => method.method.to_string(),
            Expr::Field(field) => {
                if let syn::Member::Named(ident) = &field.member {
                    ident.to_string()
                } else {
                    "future".to_string()
                }
            }
            _ => "future".to_string(),
        }
    }

    // ========== Phase 5: Extended Tracking Transformations ==========

    /// Transform for loop
    fn transform_for_loop(&mut self, expr: &mut Expr, for_loop: &syn::ExprForLoop) {
        let loop_id = self.gen_id();
        let location = Self::location_tokens(for_loop.for_token.span);
        let pat = &for_loop.pat;
        let iter_expr = &for_loop.expr;
        let body = &for_loop.body;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "for", #location);
                let mut __iter_count = 0usize;
                for #pat in #iter_expr {
                    borrowscope_runtime::track_loop_iteration(#loop_id, __iter_count, #location);
                    __iter_count += 1;
                    #body
                }
                borrowscope_runtime::track_loop_exit(#loop_id, #location);
            }
        };
    }

    /// Transform while loop
    fn transform_while_loop(&mut self, expr: &mut Expr, while_loop: &syn::ExprWhile) {
        let loop_id = self.gen_id();
        let location = Self::location_tokens(while_loop.while_token.span);
        let cond = &while_loop.cond;
        let body = &while_loop.body;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "while", #location);
                let mut __iter_count = 0usize;
                while #cond {
                    borrowscope_runtime::track_loop_iteration(#loop_id, __iter_count, #location);
                    __iter_count += 1;
                    #body
                }
                borrowscope_runtime::track_loop_exit(#loop_id, #location);
            }
        };
    }

    /// Transform infinite loop
    fn transform_loop(&mut self, expr: &mut Expr, loop_expr: &syn::ExprLoop) {
        let loop_id = self.gen_id();
        let location = Self::location_tokens(loop_expr.loop_token.span);
        let body = &loop_expr.body;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "loop", #location);
                let mut __iter_count = 0usize;
                loop {
                    borrowscope_runtime::track_loop_iteration(#loop_id, __iter_count, #location);
                    __iter_count += 1;
                    #body
                }
            }
        };
    }

    /// Transform try/? operator
    fn transform_try(&mut self, expr: &mut Expr, try_expr: &syn::ExprTry) {
        let try_id = self.gen_id();
        let location = Self::location_tokens(try_expr.question_token.span);
        let inner = &try_expr.expr;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_try(#try_id, #location);
                #inner?
            }
        };
    }

    /// Transform clone method call
    fn transform_clone(&mut self, expr: &mut Expr, method_call: &syn::ExprMethodCall) {
        let clone_id = self.gen_id();
        let location = Self::location_tokens(method_call.method.span());
        let receiver = &method_call.receiver;
        let var_name = Self::extract_receiver_name(receiver).unwrap_or_else(|| "expr".to_string());

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_clone(#clone_id, #var_name, #location);
                #receiver.clone()
            }
        };
    }

    /// Transform lock method calls (Mutex/RwLock)
    fn transform_lock(&mut self, expr: &mut Expr, method_call: &syn::ExprMethodCall, lock_type: &str) {
        let lock_id = self.gen_id();
        let location = Self::location_tokens(method_call.method.span());
        let receiver = &method_call.receiver;
        let var_name = Self::extract_receiver_name(receiver).unwrap_or_else(|| "lock".to_string());
        let method = &method_call.method;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_lock(#lock_id, #lock_type, #var_name, #location);
                #receiver.#method()
            }
        };
    }

    /// Transform unwrap method calls
    fn transform_unwrap(&mut self, expr: &mut Expr, method_call: &syn::ExprMethodCall) {
        let unwrap_id = self.gen_id();
        let location = Self::location_tokens(method_call.method.span());
        let receiver = &method_call.receiver;
        let var_name = Self::extract_receiver_name(receiver).unwrap_or_else(|| "expr".to_string());
        let method_name = method_call.method.to_string();
        let method = &method_call.method;
        let args = &method_call.args;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_unwrap(#unwrap_id, #method_name, #var_name, #location);
                #receiver.#method(#args)
            }
        };
    }

    /// Transform deref operation
    fn transform_deref(&mut self, expr: &mut Expr, unary: &syn::ExprUnary) {
        let deref_id = self.gen_id();
        let location = Self::location_tokens(unary.op.span());
        let inner = &unary.expr;
        let var_name = if let Expr::Path(path) = inner.as_ref() {
            quote::quote!(#path).to_string()
        } else {
            "expr".to_string()
        };

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_deref(#deref_id, #var_name, #location);
                *#inner
            }
        };
    }

    /// Transform match expression
    fn transform_match(&mut self, expr: &mut Expr, match_expr: &syn::ExprMatch) {
        let match_id = self.gen_id();
        let location = Self::location_tokens(match_expr.match_token.span);
        let scrutinee = &match_expr.expr;

        let mut new_arms: Vec<syn::Arm> = Vec::new();
        for (idx, arm) in match_expr.arms.iter().enumerate() {
            let pat = &arm.pat;
            let guard = &arm.guard;
            let body = &arm.body;
            let pat_str = quote::quote!(#pat).to_string();

            let new_body: Expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_match_arm(#match_id, #idx, #pat_str, #location);
                    #body
                }
            };

            let new_arm: syn::Arm = if let Some((if_token, guard_expr)) = guard {
                syn::parse_quote! { #pat #if_token #guard_expr => #new_body }
            } else {
                syn::parse_quote! { #pat => #new_body }
            };
            new_arms.push(new_arm);
        }

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_match_enter(#match_id, #location);
                let __match_result = match #scrutinee {
                    #(#new_arms),*
                };
                borrowscope_runtime::track_match_exit(#match_id, #location);
                __match_result
            }
        };
    }

    /// Transform if expression
    fn transform_if(&mut self, expr: &mut Expr, if_expr: &syn::ExprIf) {
        let branch_id = self.gen_id();
        let location = Self::location_tokens(if_expr.if_token.span);
        let cond = &if_expr.cond;
        let then_branch = &if_expr.then_branch;

        let new_then: Block = syn::parse_quote! {
            {
                borrowscope_runtime::track_branch(#branch_id, "then", #location);
                #then_branch
            }
        };

        if let Some((_else_token, else_branch)) = &if_expr.else_branch {
            let new_else: Expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_branch(#branch_id, "else", #location);
                    #else_branch
                }
            };
            *expr = syn::parse_quote! {
                if #cond #new_then else #new_else
            };
        } else {
            *expr = syn::parse_quote! {
                if #cond #new_then
            };
        }
    }

    /// Transform return expression
    fn transform_return(&mut self, expr: &mut Expr, return_expr: &syn::ExprReturn) {
        let return_id = self.gen_id();
        let location = Self::location_tokens(return_expr.return_token.span);
        let has_value = return_expr.expr.is_some();

        if let Some(value) = &return_expr.expr {
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_return(#return_id, #has_value, #location);
                    return #value
                }
            };
        } else {
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_return(#return_id, #has_value, #location);
                    return
                }
            };
        }
    }

    /// Transform index expression
    fn transform_index(&mut self, expr: &mut Expr, index_expr: &syn::ExprIndex) {
        let access_id = self.gen_id();
        let location = Self::location_tokens(index_expr.bracket_token.span.open());
        let base = &index_expr.expr;
        let index = &index_expr.index;
        let container = if let Expr::Path(path) = base.as_ref() {
            quote::quote!(#path).to_string()
        } else {
            "expr".to_string()
        };

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_index_access(#access_id, #container, #location);
                #base[#index]
            }
        };
    }

    /// Transform field access expression
    fn transform_field(&mut self, expr: &mut Expr, field_expr: &syn::ExprField) {
        let access_id = self.gen_id();
        let location = Self::location_tokens(field_expr.dot_token.span);
        let base_expr = &field_expr.base;
        let member = &field_expr.member;

        let base_name = if let Expr::Path(path) = base_expr.as_ref() {
            quote::quote!(#path).to_string()
        } else {
            "expr".to_string()
        };

        let field_name = match member {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(idx) => idx.index.to_string(),
        };

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_field_access(#access_id, #base_name, #field_name, #location);
                #base_expr.#member
            }
        };
    }

    /// Transform function call (generic, excluding special cases)
    /// Note: Currently disabled as it would be too noisy. Can be enabled via feature flag.
    #[allow(dead_code)]
    fn transform_fn_call(&mut self, expr: &mut Expr, call_expr: &syn::ExprCall) {
        let call_id = self.gen_id();
        let func = &call_expr.func;
        let args = &call_expr.args;

        let fn_name = if let Expr::Path(path) = func.as_ref() {
            quote::quote!(#path).to_string()
        } else {
            "fn".to_string()
        };

        // Skip if already handled (transmute, etc.)
        if fn_name.contains("transmute") || fn_name.contains("track_") {
            return;
        }

        let location = Self::location_tokens(
            if let Expr::Path(path) = func.as_ref() {
                path.path.segments.last().map(|s| s.ident.span()).unwrap_or_else(proc_macro2::Span::call_site)
            } else {
                proc_macro2::Span::call_site()
            }
        );

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_call(#call_id, #fn_name, #location);
                #func(#args)
            }
        };
    }

    // =========================================================================
    // Phase 6: Additional Transformations
    // =========================================================================

    /// Transform break statement
    fn transform_break(&mut self, expr: &mut Expr, break_expr: &syn::ExprBreak) {
        let break_id = self.gen_id();
        let location = Self::location_tokens(break_expr.break_token.span);
        let label = break_expr.label.as_ref().map(|l| l.ident.to_string());

        if let Some(ref lbl) = label {
            if let Some(value) = &break_expr.expr {
                *expr = syn::parse_quote! {
                    {
                        borrowscope_runtime::track_break(#break_id, Some(#lbl), #location);
                        break #value
                    }
                };
            } else {
                let label_lifetime = &break_expr.label;
                *expr = syn::parse_quote! {
                    {
                        borrowscope_runtime::track_break(#break_id, Some(#lbl), #location);
                        break #label_lifetime
                    }
                };
            }
        } else if let Some(value) = &break_expr.expr {
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_break(#break_id, None::<&str>, #location);
                    break #value
                }
            };
        } else {
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_break(#break_id, None::<&str>, #location);
                    break
                }
            };
        }
    }

    /// Transform continue statement
    fn transform_continue(&mut self, expr: &mut Expr, continue_expr: &syn::ExprContinue) {
        let continue_id = self.gen_id();
        let location = Self::location_tokens(continue_expr.continue_token.span);
        let label = continue_expr.label.as_ref().map(|l| l.ident.to_string());

        if let Some(ref lbl) = label {
            let label_lifetime = &continue_expr.label;
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_continue(#continue_id, Some(#lbl), #location);
                    continue #label_lifetime
                }
            };
        } else {
            *expr = syn::parse_quote! {
                {
                    borrowscope_runtime::track_continue(#continue_id, None::<&str>, #location);
                    continue
                }
            };
        }
    }

    /// Transform struct creation
    fn transform_struct(&mut self, expr: &mut Expr, struct_expr: &syn::ExprStruct) {
        let struct_id = self.gen_id();
        let location = Self::location_tokens(struct_expr.brace_token.span.open());
        let type_name = quote::quote!(#struct_expr).to_string().split('{').next().unwrap_or("").trim().to_string();
        let original = struct_expr.clone();

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_struct_create(#struct_id, #type_name, #location);
                #original
            }
        };
    }

    /// Transform tuple creation
    fn transform_tuple(&mut self, expr: &mut Expr, tuple_expr: &syn::ExprTuple) {
        let tuple_id = self.gen_id();
        let location = Self::location_tokens(tuple_expr.paren_token.span.open());
        let len = tuple_expr.elems.len();
        let original = tuple_expr.clone();

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_tuple_create(#tuple_id, #len, #location);
                #original
            }
        };
    }

    /// Transform range expression
    fn transform_range(&mut self, expr: &mut Expr, range_expr: &syn::ExprRange) {
        let range_id = self.gen_id();
        let location = Self::location_tokens(proc_macro2::Span::call_site());
        let range_type = match range_expr.limits {
            syn::RangeLimits::HalfOpen(_) => "half_open",
            syn::RangeLimits::Closed(_) => "closed",
        };
        let original = range_expr.clone();

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_range(#range_id, #range_type, #location);
                #original
            }
        };
    }

    /// Transform array creation
    fn transform_array(&mut self, expr: &mut Expr, array_expr: &syn::ExprArray) {
        let array_id = self.gen_id();
        let location = Self::location_tokens(array_expr.bracket_token.span.open());
        let len = array_expr.elems.len();
        let original = array_expr.clone();

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_array_create(#array_id, #len, #location);
                #original
            }
        };
    }

    /// Transform type cast (non-pointer)
    fn transform_cast(&mut self, expr: &mut Expr, cast_expr: &syn::ExprCast) {
        let cast_id = self.gen_id();
        let location = Self::location_tokens(cast_expr.as_token.span);
        let to_type = quote::quote!(#cast_expr.ty).to_string();
        let inner = &cast_expr.expr;
        let ty = &cast_expr.ty;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_type_cast(#cast_id, #to_type, #location);
                #inner as #ty
            }
        };
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

        // Handle async blocks before default traversal
        if let Expr::Async(async_expr) = expr {
            self.transform_async_block(async_expr);
            return;
        }

        // Handle await expressions - need to transform before visiting base
        if let Expr::Await(await_expr) = expr.clone() {
            self.transform_await(expr, &await_expr);
            return;
        }

        // Handle loops before default traversal
        if let Expr::ForLoop(for_loop) = expr.clone() {
            self.transform_for_loop(expr, &for_loop);
            return;
        }
        if let Expr::While(while_loop) = expr.clone() {
            self.transform_while_loop(expr, &while_loop);
            return;
        }
        if let Expr::Loop(loop_expr) = expr.clone() {
            self.transform_loop(expr, &loop_expr);
            return;
        }

        // Handle try/? operator
        if let Expr::Try(try_expr) = expr.clone() {
            self.transform_try(expr, &try_expr);
            return;
        }

        // Handle match expressions
        if let Expr::Match(match_expr) = expr.clone() {
            self.transform_match(expr, &match_expr);
            return;
        }

        // Handle if expressions
        if let Expr::If(if_expr) = expr.clone() {
            self.transform_if(expr, &if_expr);
            return;
        }

        // Handle return expressions
        if let Expr::Return(return_expr) = expr.clone() {
            self.transform_return(expr, &return_expr);
            return;
        }

        // Phase 6: Handle break expressions
        if let Expr::Break(break_expr) = expr.clone() {
            self.transform_break(expr, &break_expr);
            return;
        }

        // Phase 6: Handle continue expressions
        if let Expr::Continue(continue_expr) = expr.clone() {
            self.transform_continue(expr, &continue_expr);
            return;
        }

        // Phase 6: Handle struct creation
        if let Expr::Struct(struct_expr) = expr.clone() {
            self.transform_struct(expr, &struct_expr);
            return;
        }

        // Phase 6: Handle tuple creation (skip unit tuples)
        if let Expr::Tuple(tuple_expr) = expr.clone() {
            if !tuple_expr.elems.is_empty() {
                self.transform_tuple(expr, &tuple_expr);
                return;
            }
        }

        // Phase 6: Handle range expressions
        if let Expr::Range(range_expr) = expr.clone() {
            self.transform_range(expr, &range_expr);
            return;
        }

        // Phase 6: Handle array creation
        if let Expr::Array(array_expr) = expr.clone() {
            self.transform_array(expr, &array_expr);
            return;
        }

        // Handle method calls - check for RefCell/Cell methods first
        if let Expr::MethodCall(method_call) = expr {
            let method_name = method_call.method.to_string();
            
            // Check for clone
            if method_name == "clone" {
                let mc = method_call.clone();
                self.transform_clone(expr, &mc);
                return;
            }

            // Check for lock methods (Mutex/RwLock)
            match method_name.as_str() {
                "lock" | "try_lock" => {
                    let mc = method_call.clone();
                    self.transform_lock(expr, &mc, "mutex");
                    return;
                }
                "read" | "try_read" => {
                    let mc = method_call.clone();
                    self.transform_lock(expr, &mc, "rwlock_read");
                    return;
                }
                "write" | "try_write" => {
                    let mc = method_call.clone();
                    self.transform_lock(expr, &mc, "rwlock_write");
                    return;
                }
                _ => {}
            }

            // Check for unwrap methods
            match method_name.as_str() {
                "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" | "unwrap_or_default" => {
                    let mc = method_call.clone();
                    self.transform_unwrap(expr, &mc);
                    return;
                }
                _ => {}
            }
            
            // Check for RefCell/Cell specific methods that need wrapping
            if let Some(receiver_name) = Self::extract_receiver_name(&method_call.receiver) {
                let location = Self::location_tokens(method_call.method.span());
                let borrow_id = format!("borrow_{}", self.gen_id());
                let receiver_id = format!("refcell_{}", receiver_name);
                let receiver = method_call.receiver.clone();
                
                match method_name.as_str() {
                    "borrow" => {
                        // Transform cell.borrow() -> track_refcell_borrow(id, cell_id, loc, cell.borrow())
                        *expr = syn::parse_quote! {
                            borrowscope_runtime::track_refcell_borrow(#borrow_id, #receiver_id, #location, #receiver.borrow())
                        };
                        return;
                    }
                    "borrow_mut" => {
                        // Transform cell.borrow_mut() -> track_refcell_borrow_mut(id, cell_id, loc, cell.borrow_mut())
                        *expr = syn::parse_quote! {
                            borrowscope_runtime::track_refcell_borrow_mut(#borrow_id, #receiver_id, #location, #receiver.borrow_mut())
                        };
                        return;
                    }
                    "get" => {
                        // Transform cell.get() -> track_cell_get(cell_id, loc, cell.get())
                        let cell_id = format!("cell_{}", receiver_name);
                        *expr = syn::parse_quote! {
                            borrowscope_runtime::track_cell_get(#cell_id, #location, #receiver.get())
                        };
                        return;
                    }
                    "set" => {
                        // Transform cell.set(v) -> { track_cell_set(cell_id, loc); cell.set(v) }
                        let cell_id = format!("cell_{}", receiver_name);
                        // Visit arguments first
                        let args: Vec<_> = method_call.args.iter().cloned().collect();
                        if let Some(arg) = args.first() {
                            *expr = syn::parse_quote! {
                                {
                                    borrowscope_runtime::track_cell_set(#cell_id, #location);
                                    #receiver.set(#arg)
                                }
                            };
                        }
                        return;
                    }
                    _ => {}
                }
            }
            
            // Handle other method calls
            self.transform_method_call(method_call);
            return;
        }

        // Handle deref operations (*expr) - DISABLED: breaks assignment expressions
        // The transformation `*x = y` -> `{ track_deref(...); *x } = y` is invalid
        // Would need context-aware transformation to handle lvalue vs rvalue
        // if let Expr::Unary(unary) = expr.clone() {
        //     if matches!(unary.op, syn::UnOp::Deref(_)) {
        //         self.transform_deref(expr, &unary);
        //         return;
        //     }
        // }

        // Handle index access (arr[i]) - DISABLED: same issue as deref
        // if let Expr::Index(index_expr) = expr.clone() {
        //     self.transform_index(expr, &index_expr);
        //     return;
        // }

        // Handle field access (obj.field) - DISABLED: same issue as deref
        // if let Expr::Field(field_expr) = expr.clone() {
        //     self.transform_field(expr, &field_expr);
        //     return;
        // }

        // First recursively visit nested expressions
        visit_mut::visit_expr_mut(self, expr);

        // Then transform reference expressions at this level
        if let Expr::Reference(ref_expr) = expr.clone() {
            self.transform_reference(expr, &ref_expr);
        }

        // Transform unsafe blocks
        if let Expr::Unsafe(unsafe_expr) = expr {
            self.transform_unsafe_block(unsafe_expr);
        }

        // Transform raw pointer casts (takes precedence over general cast tracking)
        if let Expr::Cast(cast_expr) = expr.clone() {
            // Check if it's a pointer cast - those are handled specially
            if matches!(cast_expr.ty.as_ref(), syn::Type::Ptr(_)) {
                self.transform_ptr_cast(expr, &cast_expr);
            } else {
                // Non-pointer casts - Phase 6 tracking
                self.transform_cast(expr, &cast_expr);
            }
        }

        // Transform transmute calls
        if let Expr::Call(call_expr) = expr.clone() {
            self.transform_call_expr(expr, &call_expr);
        }

        // Note: The following cannot be detected without type information:
        // - Raw pointer dereferences (*ptr) - can't distinguish from regular deref
        // - FFI calls - can't know if function is extern "C"
        // - Union field access - can't know if type is union vs struct
        // - Unsafe fn calls - can't know if function is unsafe
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

    #[test]
    fn test_refcell_borrow_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            cell.borrow()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_refcell_borrow"));
    }

    #[test]
    fn test_refcell_borrow_mut_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            cell.borrow_mut()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_refcell_borrow_mut"));
    }

    #[test]
    fn test_cell_get_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            counter.get()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_cell_get"));
    }

    #[test]
    fn test_cell_set_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            counter.set(42)
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_cell_set"));
    }

    #[test]
    fn test_unsafe_block_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            unsafe { *ptr }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_unsafe_block_enter"));
        assert!(output.contains("track_unsafe_block_exit"));
    }

    #[test]
    fn test_raw_ptr_const_cast_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            &x as *const i32
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_raw_ptr"));
    }

    #[test]
    fn test_raw_ptr_mut_cast_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            &mut x as *mut i32
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_raw_ptr_mut"));
    }

    #[test]
    fn test_transmute_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            std::mem::transmute::<u32, f32>(x)
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_transmute"));
    }

    #[test]
    fn test_async_block_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            async { 42 }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_async_block_enter"));
        assert!(output.contains("track_async_block_exit"));
    }

    #[test]
    fn test_async_move_block_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            async move { x + 1 }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_async_block_enter"));
        assert!(output.contains("track_async_block_exit"));
    }

    #[test]
    fn test_await_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            my_future.await
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_await_start"));
        assert!(output.contains("track_await_end"));
        assert!(output.contains("my_future"));
    }

    #[test]
    fn test_await_method_call_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            fetch_data().await
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_await_start"));
        assert!(output.contains("fetch_data"));
    }

    // ========== Phase 5 Tests ==========

    #[test]
    fn test_for_loop_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            for i in 0..10 { println!("{}", i); }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_loop_enter"));
        assert!(output.contains("track_loop_iteration"));
        assert!(output.contains("track_loop_exit"));
        assert!(output.contains("\"for\""));
    }

    #[test]
    fn test_while_loop_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            while x > 0 { x -= 1; }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_loop_enter"));
        assert!(output.contains("track_loop_iteration"));
        assert!(output.contains("track_loop_exit"));
        assert!(output.contains("\"while\""));
    }

    #[test]
    fn test_loop_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            loop { break; }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_loop_enter"));
        assert!(output.contains("track_loop_iteration"));
        assert!(output.contains("\"loop\""));
    }

    #[test]
    fn test_try_operator_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            some_result?
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_try"));
    }

    #[test]
    fn test_clone_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            data.clone()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_clone"));
        assert!(output.contains("\"data\""));
    }

    #[test]
    fn test_mutex_lock_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            mutex.lock()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_lock"));
        assert!(output.contains("\"mutex\""));
    }

    #[test]
    fn test_rwlock_read_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            rwlock.read()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_lock"));
        assert!(output.contains("rwlock_read"));
    }

    #[test]
    fn test_unwrap_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            option.unwrap()
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_unwrap"));
        assert!(output.contains("\"unwrap\""));
    }

    #[test]
    fn test_expect_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            result.expect("error")
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_unwrap"));
        assert!(output.contains("\"expect\""));
    }

    #[test]
    fn test_match_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            match x { 1 => "one", _ => "other" }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_match_enter"));
        assert!(output.contains("track_match_arm"));
        assert!(output.contains("track_match_exit"));
    }

    #[test]
    fn test_if_else_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            if x > 0 { 1 } else { 0 }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_branch"));
        assert!(output.contains("\"then\""));
        assert!(output.contains("\"else\""));
    }

    #[test]
    fn test_return_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            return 42
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_return"));
    }

    // ========== Phase 6 Tests ==========

    #[test]
    fn test_break_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            break
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_break"));
    }

    #[test]
    fn test_break_with_label_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            break 'outer
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_break"));
        assert!(output.contains("outer"));
    }

    #[test]
    fn test_continue_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            continue
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_continue"));
    }

    #[test]
    fn test_struct_creation_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            Point { x: 1, y: 2 }
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_struct_create"));
        assert!(output.contains("Point"));
    }

    #[test]
    fn test_tuple_creation_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            (1, 2, 3)
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_tuple_create"));
        assert!(output.contains("3")); // arity
    }

    #[test]
    fn test_range_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            0..10
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_range"));
        assert!(output.contains("half_open"));
    }

    #[test]
    fn test_array_creation_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            [1, 2, 3, 4]
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_array_create"));
        assert!(output.contains("4")); // length
    }

    #[test]
    fn test_type_cast_transformation() {
        let mut visitor = OwnershipVisitor::new();

        let mut expr: Expr = parse_quote! {
            x as i64
        };

        visitor.visit_expr_mut(&mut expr);

        let output = expr.to_token_stream().to_string();
        assert!(output.contains("track_type_cast"));
    }
}
