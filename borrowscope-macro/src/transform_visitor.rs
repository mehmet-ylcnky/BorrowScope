//! AST transformation visitor using VisitMut
//!
//! This module implements the OwnershipVisitor that transforms Rust code
//! to inject runtime tracking calls.

use crate::config::TraceConfig;
use crate::smart_pointer::{
    detect_box_pin, detect_box_raw_op, detect_concurrency_op, detect_cow_creation,
    detect_downgrade, detect_maybe_uninit_new,
    detect_once_cell_new, detect_pin_operation, detect_rc_clone,
    detect_smart_pointer_new, ConcurrencyOp, SmartPointerOp, SmartPointerType,
};
use crate::type_info;
use quote::quote;
use std::collections::{HashMap, HashSet};
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
    /// Configuration for what to track
    config: TraceConfig,
    /// Variables that are references (to avoid double-borrowing in method calls)
    ref_vars: HashSet<String>,
    /// Variables that are OnceCell/OnceLock
    once_cell_vars: HashSet<String>,
    /// Variables that are MaybeUninit
    maybe_uninit_vars: HashSet<String>,
    /// Variables that are Cow
    cow_vars: HashSet<String>,
    /// Variables that are Weak (Rc or Arc)
    weak_vars: HashMap<String, SmartPointerType>,
    /// Variables that are channel senders
    sender_vars: HashSet<String>,
    /// Variables that are channel receivers
    receiver_vars: HashSet<String>,
    /// Variables that are JoinHandles
    join_handle_vars: HashSet<String>,
    /// Variables that are Box (for into_raw tracking)
    box_vars: HashSet<String>,
    /// Variables that are lock guards (for drop tracking)
    lock_guard_vars: HashMap<String, String>, // guard_name -> lock_type
    /// Collected warnings to include in output
    warnings: Vec<proc_macro2::TokenStream>,
    /// Current function name (for analyzer lookup)
    current_function: Option<String>,
    /// Current let-binding name (for closure capture lookup)
    current_let_binding: Option<String>,
    /// Declaration count per variable name in current function (for disambiguation)
    decl_counts: HashMap<String, u32>,
}

impl OwnershipVisitor {
    /// Create a new visitor with default config
    pub fn new() -> Self {
        Self::with_config(TraceConfig::standard())
    }

    /// Create a new visitor with custom config
    pub fn with_config(config: TraceConfig) -> Self {
        Self {
            scope_depth: 0,
            var_ids: HashMap::new(),
            next_id: 1,
            scope_stack: vec![Vec::new()], // Start with root scope
            current_stmt_index: 0,
            pending_inserts: Vec::new(),
            config,
            ref_vars: HashSet::new(),
            once_cell_vars: HashSet::new(),
            maybe_uninit_vars: HashSet::new(),
            cow_vars: HashSet::new(),
            weak_vars: HashMap::new(),
            sender_vars: HashSet::new(),
            receiver_vars: HashSet::new(),
            join_handle_vars: HashSet::new(),
            box_vars: HashSet::new(),
            lock_guard_vars: HashMap::new(),
            warnings: Vec::new(),
            current_function: None,
            current_let_binding: None,
            decl_counts: HashMap::new(),
        }
    }

    /// Get collected warnings
    pub fn take_warnings(&mut self) -> Vec<proc_macro2::TokenStream> {
        std::mem::take(&mut self.warnings)
    }

    /// Add a warning
    fn add_warning(&mut self, warning: proc_macro2::TokenStream) {
        self.warnings.push(warning);
    }

    /// Lookup type info from analyzer for a variable
    /// Returns the declaration index and increments the counter
    fn lookup_type_info(&mut self, var_name: &str) -> Option<&'static type_info::VariableTypeInfo> {
        let decl_idx = {
            let count = self.decl_counts.entry(var_name.to_string()).or_insert(0);
            let idx = *count;
            *count += 1;
            idx
        };

        // Try function-scoped lookup first
        if let Some(fn_name) = &self.current_function {
            if let Some(info) = type_info::lookup_in_function(fn_name, var_name, Some(decl_idx)) {
                return Some(info);
            }
        }

        // Fall back to name-only lookup
        type_info::lookup_by_name(var_name)
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

    /// Check if variable name matches the filter pattern
    /// Supports glob-style patterns: * matches any chars, ? matches single char
    fn matches_filter(&self, var_name: &str) -> bool {
        match &self.config.filter_pattern {
            None => true, // No filter = track everything
            Some(pattern) => {
                // Parse filter: "name:pattern" or just "pattern"
                let pattern = if let Some((_prefix, pat)) = pattern.split_once(':') {
                    pat
                } else {
                    pattern.as_str()
                };
                Self::glob_match(pattern, var_name)
            }
        }
    }

    /// Simple glob matching: * = any chars, ? = single char
    fn glob_match(pattern: &str, text: &str) -> bool {
        let mut p_chars = pattern.chars().peekable();
        let mut t_chars = text.chars().peekable();

        while let Some(p) = p_chars.next() {
            match p {
                '*' => {
                    // * matches zero or more characters
                    if p_chars.peek().is_none() {
                        return true; // Trailing * matches everything
                    }
                    // Try matching rest of pattern at each position
                    let rest: String = p_chars.collect();
                    let mut remaining: String = t_chars.collect();
                    while !remaining.is_empty() {
                        if Self::glob_match(&rest, &remaining) {
                            return true;
                        }
                        remaining = remaining.chars().skip(1).collect();
                    }
                    return Self::glob_match(&rest, "");
                }
                '?' => {
                    // ? matches exactly one character
                    if t_chars.next().is_none() {
                        return false;
                    }
                }
                c => {
                    // Literal character must match
                    if t_chars.next() != Some(c) {
                        return false;
                    }
                }
            }
        }
        t_chars.next().is_none() // Pattern consumed, text should be too
    }

    /// Wrap tracking call with sampling check if sample rate is set
    fn wrap_with_sampling(&self, tracking_call: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self.config.sample_rate {
            None => tracking_call,
            Some(rate) => {
                quote! {
                    if borrowscope_runtime::should_sample(#rate) {
                        #tracking_call
                    }
                }
            }
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
                        // Simple binding - generate track_new if it matches filter
                        let access_expr = Self::build_access_expr(source, &new_indices, fields);

                        self.var_ids.insert(var_name.clone(), self.next_id);
                        if let Some(current_scope) = self.scope_stack.last_mut() {
                            current_scope.push(var_name.clone());
                        }

                        let stmt: Stmt = if self.matches_filter(&var_name) {
                            syn::parse_quote! {
                                let #elem_pat = borrowscope_runtime::track_new(#var_name, #access_expr);
                            }
                        } else {
                            syn::parse_quote! {
                                let #elem_pat = #access_expr;
                            }
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
                        let stmt: Stmt = if self.matches_filter(&var_name) {
                            syn::parse_quote! {
                                let #pat = borrowscope_runtime::track_new(#var_name, #access_expr);
                            }
                        } else {
                            syn::parse_quote! {
                                let #pat = #access_expr;
                            }
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
            let original_expr = init.expr.clone();
            let original_pat = local.pat.clone();

            // Special handling for mpsc::channel() which returns (Sender, Receiver)
            if self.config.track_smart_pointers {
                if let Some(ConcurrencyOp::ChannelNew) = detect_concurrency_op(&original_expr) {
                    // For tuple pattern like (tx, rx), extract the names
                    if let Pat::Tuple(tuple_pat) = &original_pat {
                        if tuple_pat.elems.len() == 2 {
                            let tx_name = Self::extract_pattern_name(&tuple_pat.elems[0]);
                            let rx_name = Self::extract_pattern_name(&tuple_pat.elems[1]);
                            let location = Self::location_tokens(local.pat.span());
                            let channel_id = format!("{}_{}", tx_name, rx_name);

                            // Transform to: let (tx, rx) = { let (__tx, __rx) = channel(); track_channel(...) }
                            let new_expr: Expr = syn::parse_quote! {
                                {
                                    let (__borrowscope_tx, __borrowscope_rx) = #original_expr;
                                    let __borrowscope_tx_id = format!("{}_tx", #channel_id);
                                    let __borrowscope_rx_id = format!("{}_rx", #channel_id);
                                    borrowscope_runtime::track_channel(#channel_id, #location, __borrowscope_tx, __borrowscope_rx)
                                }
                            };
                            *init.expr = new_expr;

            // Track the variables
                            let tx_id = self.gen_id();
                            let rx_id = self.gen_id();
                            self.var_ids.insert(tx_name.clone(), tx_id);
                            self.var_ids.insert(rx_name.clone(), rx_id);
                            // Track as sender/receiver for send/recv detection
                            self.sender_vars.insert(tx_name.clone());
                            self.receiver_vars.insert(rx_name.clone());

                            if let Some(current_scope) = self.scope_stack.last_mut() {
                                current_scope.push(tx_name);
                                current_scope.push(rx_name);
                            }

                            return;
                        }
                    }
                }
            }

            let temp_name = format!("__pattern_temp_{}", self.next_id);
            let temp_ident: Ident = syn::parse_str(&temp_name).unwrap();

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

    /// Infer self borrow type from method name using semantic lookup with heuristic fallback
    fn infer_self_borrow_type(method_name: &str, receiver_name: Option<&str>) -> SelfBorrowType {
        // Try semantic lookup first if we have receiver name and type info
        if let Some(var_name) = receiver_name {
            if let Some(type_info) = crate::type_info::lookup_by_name(var_name) {
                // Find method call by name (we can't use line/column on stable Rust)
                // If there are multiple calls to the same method, they should have the same self_borrow
                if let Some(method_call) = type_info.method_calls.iter().find(|mc| mc.method == method_name) {
                    if let Some(ref self_borrow) = method_call.self_borrow {
                        return match self_borrow.as_str() {
                            "immutable" => SelfBorrowType::Immutable,
                            "mutable" => SelfBorrowType::Mutable,
                            "consuming" => SelfBorrowType::Consuming,
                            _ => SelfBorrowType::Immutable, // fallback
                        };
                    }
                }
            }
        }
        
        // Fallback to heuristics if semantic lookup fails
        Self::infer_self_borrow_type_heuristic(method_name)
    }
    
    /// Infer self borrow type from method name using heuristics (fallback)
    fn infer_self_borrow_type_heuristic(method_name: &str) -> SelfBorrowType {
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
            || method_name.starts_with("add")
            || method_name.starts_with("set")
            || method_name.starts_with("update")
            || method_name.starts_with("modify")
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
        
        // Skip wrapping for methods that return guards/borrows - wrapping creates
        // temporaries that cause lifetime issues (e.g., if let Ok(g) = mutex.try_lock())
        let guard_methods = [
            "lock", "try_lock", "read", "try_read", "write", "try_write",
            "borrow", "borrow_mut", "get_mut",
        ];
        if guard_methods.contains(&method_name.as_str()) {
            self.visit_expr_mut(&mut method_call.receiver);
            for arg in &mut method_call.args {
                self.visit_expr_mut(arg);
            }
            return;
        }
        
        // Extract receiver name for semantic lookup
        let receiver_name = Self::extract_receiver_name(&method_call.receiver);
        let borrow_type = Self::infer_self_borrow_type(&method_name, receiver_name.as_deref());

        // For consuming methods, just visit normally (move tracking happens at assignment level)
        if borrow_type == SelfBorrowType::Consuming {
            self.visit_expr_mut(&mut method_call.receiver);
            for arg in &mut method_call.args {
                self.visit_expr_mut(arg);
            }
            return;
        }

        // Extract receiver name for tracking
        if let Some(receiver_name) = Self::extract_receiver_name(&method_call.receiver) {
            // Skip wrapping if receiver is already a reference variable
            // (e.g., let r = &mut data; r.push(4) - r is already &mut)
            if self.ref_vars.contains(&receiver_name) {
                // Just visit arguments, don't wrap the receiver
                for arg in &mut method_call.args {
                    self.visit_expr_mut(arg);
                }
                return;
            }

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
    fn transform_closure(&mut self, expr: &mut Expr, closure: &ExprClosure) {
        let closure_id = self.gen_id();
        let location = Self::location_tokens(closure.or1_token.span);
        
        // Determine capture mode - semantic: check analyzer data, fallback: move keyword
        let binding_name = self.current_let_binding.clone();
        let semantic_captures = binding_name.as_ref()
            .and_then(|name| self.lookup_type_info(name))
            .filter(|ti| !ti.closure_captures.is_empty())
            .map(|ti| ti.closure_captures.clone());

        let capture_mode = if let Some(ref caps) = semantic_captures {
            if caps.iter().any(|c| c.capture_kind == "move") { "move" } else { "ref" }
        } else if closure.capture.is_some() {
            "move"
        } else {
            "ref"
        };
        
        // Build capture tracking statements - semantic or fallback
        let capture_stmts: Vec<proc_macro2::TokenStream> = if let Some(ref caps) = semantic_captures {
            caps.iter().map(|cap| {
                let var = &cap.name;
                let kind = match cap.capture_kind.as_str() {
                    "move" => "move",
                    "mutable_ref" => "borrow_mut",
                    _ => "borrow", // shared_ref, unique_shared_ref
                };
                quote::quote! {
                    borrowscope_runtime::track_closure_capture(#closure_id, #var, #kind, #location);
                }
            }).collect()
        } else {
            // Fallback: AST walking
            let mut captured_vars = Vec::new();
            self.extract_captured_vars(&closure.body, &mut captured_vars);
            captured_vars.iter().map(|var| {
                let var_capture_mode = if closure.capture.is_some() { "move" } else { "borrow" };
                quote::quote! {
                    borrowscope_runtime::track_closure_capture(#closure_id, #var, #var_capture_mode, #location);
                }
            }).collect()
        };
        
        // Clone the closure for transformation
        let mut new_closure = closure.clone();
        
        // Visit the closure body to transform nested expressions
        self.visit_expr_mut(&mut new_closure.body);
        
        // Wrap the closure with tracking
        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_closure_create(#closure_id, #capture_mode, #location);
                #(#capture_stmts)*
                #new_closure
            }
        };
    }

    /// Check if expression is a potential move (simple variable path)
    fn is_potential_move(expr: &Expr) -> bool {
        matches!(expr, Expr::Path(_))
    }

    /// Transform a let statement to inject track_new_with_id
    fn transform_local(&mut self, local: &mut Local) {
        // Only transform if there's an initializer and tracking is enabled
        if local.init.is_none() {
            return;
        }

        // Skip if ownership tracking is disabled
        if !self.config.track_new && !self.config.track_move {
            visit_mut::visit_local_mut(self, local);
            return;
        }

        if let Some(init) = &mut local.init {
            // Check if this is a complex pattern
            if Self::is_complex_pattern(&local.pat) {
                self.transform_complex_pattern(local);
                return;
            }

            let var_name = Self::extract_pattern_name(&local.pat);
            
            // Set current let binding for closure capture lookup
            self.current_let_binding = Some(var_name.clone());

            // Skip if variable doesn't match filter
            if !self.matches_filter(&var_name) {
                visit_mut::visit_local_mut(self, local);
                return;
            }
            
            let var_id = self.gen_id();
            let location = Self::location_tokens(local.pat.span());

            // Check if the initializer is a reference expression and mark the variable
            if matches!(init.expr.as_ref(), Expr::Reference(_)) {
                self.ref_vars.insert(var_name.clone());
            }

            // Store variable ID for later reference
            self.var_ids.insert(var_name.clone(), var_id);

            // Add to current scope for drop tracking
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(var_name.clone());
            }

            let original_expr = &init.expr;

            // Check analyzer type info first (semantic), then fall back to syntactic detection
            if self.config.track_smart_pointers {
                // Try analyzer-based detection first
                if let Some(type_info) = self.lookup_type_info(&var_name) {
                    if let Some(ref init_kind) = type_info.initializer_kind {
                        if let Some(new_expr) = self.transform_by_initializer_kind(
                            init_kind, &var_name, var_id, &location, original_expr, type_info
                        ) {
                            *init.expr = new_expr;
                            visit_mut::visit_local_mut(self, local);
                            return;
                        }
                    }
                }

                // Fall back to syntactic detection
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
                                borrowscope_runtime::track_refcell_new(#var_name, #original_expr)
                            }
                        }
                        SmartPointerType::Cell => {
                            syn::parse_quote! {
                                borrowscope_runtime::track_cell_new(#var_name, #original_expr)
                            }
                        }
                        SmartPointerType::Box => {
                            // Track as Box variable for into_raw detection
                            self.box_vars.insert(var_name.clone());
                            syn::parse_quote! {
                                borrowscope_runtime::track_box_new(#var_name, #location, #original_expr)
                            }
                        }
                        _ => {
                            // Others use regular tracking
                            syn::parse_quote! {
                                borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                            }
                        }
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
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
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if detect_box_pin(original_expr) {
                    // Box::pin -> track_pin_new
                    let new_expr: Expr = syn::parse_quote! {
                        borrowscope_runtime::track_pin_new(#var_name, #location, #original_expr)
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if let Some(op) = detect_box_raw_op(original_expr) {
                    // Box::into_raw or Box::from_raw
                    let new_expr: Expr = match op {
                        SmartPointerOp::BoxIntoRaw => {
                            // Extract the box being converted
                            let box_name = self.extract_box_from_into_raw(original_expr);
                            syn::parse_quote! {
                                borrowscope_runtime::track_box_into_raw(#box_name, #location, #original_expr)
                            }
                        },
                        SmartPointerOp::BoxFromRaw => {
                            // Track the new Box created from raw pointer
                            self.box_vars.insert(var_name.clone());
                            syn::parse_quote! {
                                borrowscope_runtime::track_box_from_raw(#var_name, #location, #original_expr)
                            }
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                // Check for Box::from_raw inside unsafe block
                } else if let Expr::Unsafe(unsafe_expr) = original_expr.as_ref() {
                    // Check if the unsafe block contains a single Box::from_raw call
                    if let Some(stmt) = unsafe_expr.block.stmts.last() {
                        let inner_expr = match stmt {
                            syn::Stmt::Expr(e, _) => Some(e),
                            _ => None,
                        };
                        if let Some(inner) = inner_expr {
                            if let Some(SmartPointerOp::BoxFromRaw) = detect_box_raw_op(inner) {
                                self.box_vars.insert(var_name.clone());
                                let new_expr: Expr = syn::parse_quote! {
                                    borrowscope_runtime::track_box_from_raw(#var_name, #location, #original_expr)
                                };
                                *init.expr = new_expr;
                                visit_mut::visit_local_mut(self, local);
                                return;
                            }
                        }
                    }
                } else if let Some(op) = detect_pin_operation(original_expr) {
                    let new_expr: Expr = match op {
                        SmartPointerOp::PinNew => syn::parse_quote! {
                            borrowscope_runtime::track_pin_new(#var_name, #location, #original_expr)
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if let Some(op) = detect_cow_creation(original_expr) {
                    // Track this as a Cow variable for to_mut detection
                    self.cow_vars.insert(var_name.clone());
                    let new_expr: Expr = match op {
                        SmartPointerOp::CowBorrowed => syn::parse_quote! {
                            borrowscope_runtime::track_cow_borrowed(#var_name, #location, #original_expr)
                        },
                        SmartPointerOp::CowOwned => syn::parse_quote! {
                            borrowscope_runtime::track_cow_owned(#var_name, #location, #original_expr)
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if let Some(weak_type) = detect_downgrade(original_expr) {
                    // Rc::downgrade or Arc::downgrade - track as Weak variable
                    self.weak_vars.insert(var_name.clone(), weak_type);
                    let source_name = self.extract_downgrade_source(original_expr);
                    let new_expr: Expr = match weak_type {
                        SmartPointerType::WeakRc => syn::parse_quote! {
                            borrowscope_runtime::track_weak_new(#var_name, #source_name, #location, #original_expr)
                        },
                        SmartPointerType::WeakArc => syn::parse_quote! {
                            borrowscope_runtime::track_weak_new_sync(#var_name, #source_name, #location, #original_expr)
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                // Check if assigning from a Weak clone (let weak2 = weak.clone())
                } else if let Expr::MethodCall(method_call) = original_expr.as_ref() {
                    let method_name = method_call.method.to_string();
                    if method_name == "clone" {
                        if let Some(receiver_name) = Self::extract_receiver_name(&method_call.receiver) {
                            if let Some(weak_type) = self.weak_vars.get(&receiver_name).copied() {
                                // This is a Weak clone - track the new variable as Weak too
                                self.weak_vars.insert(var_name.clone(), weak_type);
                                let weak_id = format!("weak_{}", receiver_name);
                                let clone_id = format!("weak_clone_{}", self.gen_id());
                                let receiver = &method_call.receiver;
                                let new_expr: Expr = match weak_type {
                                    SmartPointerType::WeakRc => syn::parse_quote! {
                                        borrowscope_runtime::track_weak_clone(#clone_id, #weak_id, #location, #receiver.clone())
                                    },
                                    SmartPointerType::WeakArc => syn::parse_quote! {
                                        borrowscope_runtime::track_weak_clone_sync(#clone_id, #weak_id, #location, #receiver.clone())
                                    },
                                    _ => syn::parse_quote! {
                                        borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                                    },
                                };
                                *init.expr = new_expr;
                                visit_mut::visit_local_mut(self, local);
                                return;
                            }
                        }
                    }
                    // Check if assigning from a Weak upgrade (let strong = weak.upgrade())
                    if method_name == "upgrade" {
                        if let Some(receiver_name) = Self::extract_receiver_name(&method_call.receiver) {
                            if let Some(weak_type) = self.weak_vars.get(&receiver_name).copied() {
                                let weak_id = format!("weak_{}", receiver_name);
                                let receiver = &method_call.receiver;
                                let new_expr: Expr = match weak_type {
                                    SmartPointerType::WeakRc => syn::parse_quote! {
                                        borrowscope_runtime::track_weak_upgrade(#weak_id, #location, #receiver.upgrade())
                                    },
                                    SmartPointerType::WeakArc => syn::parse_quote! {
                                        borrowscope_runtime::track_weak_upgrade_sync(#weak_id, #location, #receiver.upgrade())
                                    },
                                    _ => syn::parse_quote! { #receiver.upgrade() },
                                };
                                *init.expr = new_expr;
                                visit_mut::visit_local_mut(self, local);
                                return;
                            }
                        }
                    }
                } else if let Some(op) = detect_concurrency_op(original_expr) {
                    let new_expr: Expr = match op {
                        ConcurrencyOp::ThreadSpawn => {
                            // Track as JoinHandle for join() detection
                            self.join_handle_vars.insert(var_name.clone());
                            syn::parse_quote! {
                                borrowscope_runtime::track_thread_spawn(#var_name, #location, #original_expr)
                            }
                        },
                        ConcurrencyOp::ChannelNew => {
                            // mpsc::channel returns a tuple, handle specially
                            syn::parse_quote! {
                                {
                                    let (__tx, __rx) = #original_expr;
                                    borrowscope_runtime::track_channel(#var_name, #location, __tx, __rx)
                                }
                            }
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if let Some(is_sync) = detect_once_cell_new(original_expr) {
                    // OnceCell::new or OnceLock::new - register the variable
                    self.once_cell_vars.insert(var_name.clone());
                    let new_expr: Expr = if is_sync {
                        syn::parse_quote! {
                            borrowscope_runtime::track_once_lock_new(#var_name, #location, #original_expr)
                        }
                    } else {
                        syn::parse_quote! {
                            borrowscope_runtime::track_once_cell_new(#var_name, #location, #original_expr)
                        }
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                } else if let Some(op) = detect_maybe_uninit_new(original_expr) {
                    // MaybeUninit::uninit or MaybeUninit::new - register the variable
                    self.maybe_uninit_vars.insert(var_name.clone());
                    let new_expr: Expr = match op {
                        SmartPointerOp::MaybeUninitUninit => syn::parse_quote! {
                            borrowscope_runtime::track_maybe_uninit_uninit(#var_name, #location, #original_expr)
                        },
                        SmartPointerOp::MaybeUninitNew => syn::parse_quote! {
                            borrowscope_runtime::track_maybe_uninit_new(#var_name, #location, #original_expr)
                        },
                        _ => syn::parse_quote! {
                            borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                        },
                    };
                    *init.expr = new_expr;
                    visit_mut::visit_local_mut(self, local);
                    return;
                }
            }

            if self.config.track_move && Self::is_potential_move(original_expr) {
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
                        visit_mut::visit_local_mut(self, local);
                        return;
                    }
                }
            }

            if self.config.track_new {
                // Regular variable creation - use helper function
                // Use sampled version if sample_rate is set
                let new_expr: Expr = if let Some(rate) = self.config.sample_rate {
                    syn::parse_quote! {
                        borrowscope_runtime::track_new_with_id_sampled(#var_id, #var_name, #location, #original_expr, #rate)
                    }
                } else {
                    syn::parse_quote! {
                        borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
                    }
                };
                *init.expr = new_expr;
            }
        }

        // Continue visiting nested expressions
        visit_mut::visit_local_mut(self, local);
    }

    /// Transform expression based on analyzer's initializer_kind
    /// Returns None if no transformation applies (fall back to syntactic detection)
    fn transform_by_initializer_kind(
        &mut self,
        init_kind: &str,
        var_name: &str,
        var_id: usize,
        location: &proc_macro2::TokenStream,
        original_expr: &Expr,
        type_info: &type_info::VariableTypeInfo,
    ) -> Option<Expr> {
        match init_kind {
            // Smart pointer creation
            "rc_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_rc_new_with_id(#var_id, #var_name, "Rc<T>", #location, #original_expr)
            }),
            "arc_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_arc_new_with_id(#var_id, #var_name, "Arc<T>", #location, #original_expr)
            }),
            "box_new" => {
                self.box_vars.insert(var_name.to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_box_new(#var_name, #location, #original_expr)
                })
            }

            // Smart pointer cloning
            "rc_clone" => {
                let source_id = self.extract_clone_source_id(original_expr);
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_rc_clone_with_id(#var_id, #source_id, #var_name, #location, #original_expr)
                })
            }
            "arc_clone" => {
                let source_id = self.extract_clone_source_id(original_expr);
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_arc_clone_with_id(#var_id, #source_id, #var_name, #location, #original_expr)
                })
            }

            // Interior mutability
            "refcell_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_refcell_new(#var_name, #original_expr)
            }),
            "cell_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_cell_new(#var_name, #original_expr)
            }),
            "mutex_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_mutex_new(#var_name, #location, #original_expr)
            }),
            "rwlock_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_rwlock_new(#var_name, #location, #original_expr)
            }),

            // Weak references
            "weak_new" | "weak_downgrade" => {
                // Track as weak variable
                if type_info.is_arc {
                    self.weak_vars.insert(var_name.to_string(), SmartPointerType::WeakArc);
                } else {
                    self.weak_vars.insert(var_name.to_string(), SmartPointerType::WeakRc);
                }
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_weak_new(#var_name, #location, #original_expr)
                })
            }

            // OnceCell/OnceLock
            "once_cell_new" | "once_lock_new" => {
                self.once_cell_vars.insert(var_name.to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_once_cell_new(#var_name, #location, #original_expr)
                })
            }

            // MaybeUninit
            "maybe_uninit_new" | "maybe_uninit" => {
                self.maybe_uninit_vars.insert(var_name.to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_maybe_uninit_new(#var_name, #location, #original_expr)
                })
            }

            // Cow
            "cow_new" | "cow_variant" => {
                self.cow_vars.insert(var_name.to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_cow_new(#var_name, #location, #original_expr)
                })
            }

            // Pin
            "pin_new" => Some(syn::parse_quote! {
                borrowscope_runtime::track_pin_new(#var_name, #location, #original_expr)
            }),

            // Channels
            "channel_new" | "sync_channel_new" => {
                // This is typically a tuple (sender, receiver)
                None // Let default handling work
            }

            // Guards - track for drop
            "mutex_guard" | "mutex_lock" | "mutex_try_lock" | "mutex_guard_mapped" => {
                self.lock_guard_vars.insert(var_name.to_string(), "Mutex".to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_mutex_lock(#var_name, #location, #original_expr)
                })
            }
            "rwlock_read_guard" | "rwlock_read" | "rwlock_read_guard_mapped" => {
                self.lock_guard_vars.insert(var_name.to_string(), "RwLock".to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_rwlock_read(#var_name, #location, #original_expr)
                })
            }
            "rwlock_write_guard" | "rwlock_write" | "rwlock_write_guard_mapped" => {
                self.lock_guard_vars.insert(var_name.to_string(), "RwLock".to_string());
                Some(syn::parse_quote! {
                    borrowscope_runtime::track_rwlock_write(#var_name, #location, #original_expr)
                })
            }
            "refcell_borrow" | "ref_guard" => Some(syn::parse_quote! {
                borrowscope_runtime::track_refcell_borrow(#var_id, 0, #location, #original_expr)
            }),
            "refcell_borrow_mut" | "refmut_guard" => Some(syn::parse_quote! {
                borrowscope_runtime::track_refcell_borrow_mut(#var_id, 0, #location, #original_expr)
            }),

            // Collections - use generic tracking
            "vec_new" | "vec_macro" | "string_new" | "hashmap_new" | "hashset_new" 
            | "btreemap_new" | "btreeset_new" | "vecdeque_new" | "linkedlist_new" 
            | "binaryheap_new" => Some(syn::parse_quote! {
                borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
            }),

            // User-defined types - use generic tracking
            "user_struct" | "user_enum" | "user_union" => Some(syn::parse_quote! {
                borrowscope_runtime::__track_new_with_id_helper(#var_id, #var_name, #location, #original_expr)
            }),

            // Primitives, references, closures - no special tracking needed
            "primitive" | "ref" | "ref_mut" | "closure" | "tuple" | "array" => None,

            // Unknown - fall back to syntactic detection
            _ => None,
        }
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

    /// Extract source variable name from Rc::downgrade(&x) or Arc::downgrade(&x)
    fn extract_downgrade_source(&self, expr: &Expr) -> String {
        if let Expr::Call(call) = expr {
            if let Some(Expr::Reference(ref_expr)) = call.args.first() {
                if let Expr::Path(path) = ref_expr.expr.as_ref() {
                    if let Some(ident) = path.path.get_ident() {
                        return ident.to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Extract box variable name from Box::into_raw(box_var)
    fn extract_box_from_into_raw(&self, expr: &Expr) -> String {
        if let Expr::Call(call) = expr {
            if let Some(arg) = call.args.first() {
                if let Expr::Path(path) = arg {
                    if let Some(ident) = path.path.get_ident() {
                        return ident.to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Transform reference expressions to inject track_borrow_with_id
    fn transform_reference(&mut self, expr: &mut Expr, ref_expr: &ExprReference) {
        // Skip if borrow tracking is disabled
        if !self.config.track_borrow {
            return;
        }

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
        
        // Visit the inner block first to transform any expressions inside
        self.visit_block_mut(&mut unsafe_expr.block);
        
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
            // Check for transmute - semantic: verify via expressions[] data, fallback: path match
            let path_looks_like_transmute = path.path.segments.last()
                .map(|s| s.ident == "transmute")
                .unwrap_or(false);
            let is_transmute = if path_looks_like_transmute {
                crate::type_info::has_transmute_expression()
                    .unwrap_or(true) // no analyzer data → assume yes
            } else {
                false
            };
            if is_transmute {
                let span = path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.span())
                    .unwrap_or_else(proc_macro2::Span::call_site);
                let location = Self::location_tokens(span);
                let args = &call_expr.args;
                let func = &call_expr.func;

                // Try to get type info from semantic data
                let (from_type, to_type) = crate::type_info::find_transmute_types()
                    .unwrap_or(("unknown", "unknown"));

                // Add warning about transmute type info
                if self.config.warn_ambiguous {
                    let warning = crate::diagnostics::create_ambiguous_warning(
                        crate::diagnostics::AmbiguousPattern::Transmute,
                        "transmute",
                        span,
                    );
                    self.add_warning(warning);
                }

                *expr = syn::parse_quote! {
                    {
                        borrowscope_runtime::track_transmute(#from_type, #to_type, #location);
                        #func(#args)
                    }
                };
                return;
            }

            // Check for potential FFI calls
            if self.config.warn_ambiguous {
                // Extract function name from path
                if let Some(last_segment) = path.path.segments.last() {
                    let fn_name = last_segment.ident.to_string();

                    // Skip if it's a known FFI function (user declared)
                    if !self.config.known_ffi.iter().any(|f| f == &fn_name) {
                        // Check if it looks like FFI
                        if crate::diagnostics::looks_like_ffi(&fn_name) {
                            let warning = crate::diagnostics::create_ambiguous_warning(
                                crate::diagnostics::AmbiguousPattern::PossibleFfi,
                                &fn_name,
                                last_segment.ident.span(),
                            );
                            self.add_warning(warning);
                        }
                    }
                }
            }
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
        let label = &for_loop.label;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "for", #location);
                let mut __iter_count = 0usize;
                #label for #pat in #iter_expr {
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
        let label = &while_loop.label;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "while", #location);
                let mut __iter_count = 0usize;
                #label while #cond {
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
        let label = &loop_expr.label;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_loop_enter(#loop_id, "loop", #location);
                let mut __iter_count = 0usize;
                #label loop {
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
        let guard_id = format!("guard_{}", lock_id);
        let location = Self::location_tokens(method_call.method.span());
        let receiver = &method_call.receiver;
        let var_name = Self::extract_receiver_name(receiver).unwrap_or_else(|| "lock".to_string());
        let lock_id_str = format!("lock_{}", var_name);
        let method = &method_call.method;

        *expr = syn::parse_quote! {
            {
                borrowscope_runtime::track_lock_guard_acquire(#guard_id, #lock_id_str, #lock_type, #location);
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

        // Check if receiver is a lock method call (e.g., mutex.lock().unwrap())
        if let Expr::MethodCall(inner_method) = receiver.as_ref() {
            let inner_method_name = inner_method.method.to_string();
            let lock_type = match inner_method_name.as_str() {
                "lock" | "try_lock" => Some("mutex"),
                "read" | "try_read" => Some("rwlock_read"),
                "write" | "try_write" => Some("rwlock_write"),
                _ => None,
            };
            
            if let Some(lock_type) = lock_type {
                let lock_id = self.gen_id();
                let guard_id = format!("guard_{}", lock_id);
                let lock_var_name = Self::extract_receiver_name(&inner_method.receiver)
                    .unwrap_or_else(|| "lock".to_string());
                let lock_id_str = format!("lock_{}", lock_var_name);
                let inner_receiver = &inner_method.receiver;
                let inner_method_ident = &inner_method.method;
                
                *expr = syn::parse_quote! {
                    {
                        borrowscope_runtime::track_lock_guard_acquire(#guard_id, #lock_id_str, #lock_type, #location);
                        borrowscope_runtime::track_lock(#lock_id, #lock_type, #lock_var_name, #location);
                        borrowscope_runtime::track_unwrap(#unwrap_id, #method_name, #var_name, #location);
                        #inner_receiver.#inner_method_ident().#method(#args)
                    }
                };
                return;
            }
        }

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
        
        // Visit the condition first to transform any method calls (like weak.upgrade())
        let mut cond = if_expr.cond.clone();
        self.visit_expr_mut(&mut cond);
        
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
        let ty = &cast_expr.ty;
        let to_type = quote::quote!(#ty).to_string();
        let inner = &cast_expr.expr;

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
        let fn_name = func.sig.ident.to_string();
        let fn_id = self.gen_id();

        // Set current function context for analyzer lookup
        let prev_function = self.current_function.take();
        let prev_decl_counts = std::mem::take(&mut self.decl_counts);
        self.current_function = Some(fn_name.clone());

        // Visit the function body first
        self.visit_block_mut(&mut func.block);

        // Restore previous context
        self.current_function = prev_function;
        self.decl_counts = prev_decl_counts;

        // Wrap body with fn_enter/fn_exit if functions tracking is enabled
        if self.config.track_functions {
            let fn_name_lit = syn::LitStr::new(&fn_name, proc_macro2::Span::call_site());
            let location = Self::location_tokens(func.sig.ident.span());

            let original_stmts = std::mem::take(&mut func.block.stmts);

            // Check if last statement is an expression (return value)
            let (body_stmts, return_expr) = if let Some(Stmt::Expr(expr, None)) = original_stmts.last() {
                let body: Vec<_> = original_stmts[..original_stmts.len() - 1].to_vec();
                (body, Some(expr.clone()))
            } else {
                (original_stmts, None)
            };

            let enter_stmt: Stmt = syn::parse_quote! {
                borrowscope_runtime::track_fn_enter(#fn_id, #fn_name_lit, #location);
            };

            func.block.stmts = vec![enter_stmt];
            func.block.stmts.extend(body_stmts);

            if let Some(ret_expr) = return_expr {
                let exit_with_return: Stmt = syn::parse_quote! {
                    {
                        let __fn_result = #ret_expr;
                        borrowscope_runtime::track_fn_exit(#fn_id, #fn_name_lit, #location);
                        __fn_result
                    }
                };
                func.block.stmts.push(exit_with_return);
            } else {
                let exit_stmt: Stmt = syn::parse_quote! {
                    borrowscope_runtime::track_fn_exit(#fn_id, #fn_name_lit, #location);
                };
                func.block.stmts.push(exit_stmt);
            }
        }
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

        // Pop scope and insert drops in LIFO order (only if track_drop is enabled)
        if let Some(scope_vars) = self.scope_stack.pop() {
            if self.config.track_drop && !scope_vars.is_empty() {
                // Check if the last statement is an expression without semicolon (implicit return)
                let has_trailing_expr = block
                    .stmts
                    .last()
                    .map(|stmt| matches!(stmt, Stmt::Expr(_, None)))
                    .unwrap_or(false);

                let sample_rate = self.config.sample_rate;

                if has_trailing_expr {
                    // Insert drops before the last expression
                    let last_stmt = block.stmts.pop();
                    for var_name in scope_vars.into_iter().rev() {
                        let drop_stmt: Stmt = if let Some(rate) = sample_rate {
                            syn::parse_quote! {
                                borrowscope_runtime::track_drop_sampled(#var_name, #rate);
                            }
                        } else {
                            syn::parse_quote! {
                                borrowscope_runtime::track_drop(#var_name);
                            }
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
                        let drop_stmt: Stmt = if let Some(rate) = sample_rate {
                            syn::parse_quote! {
                                borrowscope_runtime::track_drop_sampled(#var_name, #rate);
                            }
                        } else {
                            syn::parse_quote! {
                                borrowscope_runtime::track_drop(#var_name);
                            }
                        };
                        block.stmts.push(drop_stmt);
                    }
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
        if let Expr::Closure(closure) = expr.clone() {
            self.transform_closure(expr, &closure);
            return;
        }

        // Handle async blocks before default traversal
        if self.config.track_async {
            if let Expr::Async(async_expr) = expr {
                self.transform_async_block(async_expr);
                return;
            }
        }

        // Handle await expressions - need to transform before visiting base
        if self.config.track_async {
            if let Expr::Await(await_expr) = expr.clone() {
                self.transform_await(expr, &await_expr);
                return;
            }
        }

        // Handle loops before default traversal
        if self.config.track_loops {
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
        }

        // Handle try/? operator
        if self.config.track_try {
            if let Expr::Try(try_expr) = expr.clone() {
                self.transform_try(expr, &try_expr);
                return;
            }
        }

        // Handle match expressions
        if self.config.track_branches {
            if let Expr::Match(match_expr) = expr.clone() {
                self.transform_match(expr, &match_expr);
                return;
            }
        }

        // Handle if expressions
        if self.config.track_branches {
            if let Expr::If(if_expr) = expr.clone() {
                self.transform_if(expr, &if_expr);
                return;
            }
        }

        // Handle return expressions
        if self.config.track_control_flow {
            if let Expr::Return(return_expr) = expr.clone() {
                self.transform_return(expr, &return_expr);
                return;
            }
        }

        // Phase 6: Handle break expressions
        if self.config.track_control_flow {
            if let Expr::Break(break_expr) = expr.clone() {
                self.transform_break(expr, &break_expr);
                return;
            }
        }

        // Phase 6: Handle continue expressions
        if self.config.track_control_flow {
            if let Expr::Continue(continue_expr) = expr.clone() {
                self.transform_continue(expr, &continue_expr);
                return;
            }
        }

        // Phase 6: Handle struct creation
        if self.config.track_expressions {
            if let Expr::Struct(struct_expr) = expr.clone() {
                self.transform_struct(expr, &struct_expr);
                return;
            }
        }

        // Phase 6: Handle tuple creation (skip unit tuples)
        if self.config.track_expressions {
            if let Expr::Tuple(tuple_expr) = expr.clone() {
                if !tuple_expr.elems.is_empty() {
                    self.transform_tuple(expr, &tuple_expr);
                    return;
                }
            }
        }

        // Phase 6: Handle range expressions
        if self.config.track_expressions {
            if let Expr::Range(range_expr) = expr.clone() {
                self.transform_range(expr, &range_expr);
                return;
            }
        }

        // Phase 6: Handle array creation
        if self.config.track_expressions {
            if let Expr::Array(array_expr) = expr.clone() {
                self.transform_array(expr, &array_expr);
                return;
            }
        }

        // Handle method calls - check for RefCell/Cell methods first
        if let Expr::MethodCall(method_call) = expr {
            let method_name = method_call.method.to_string();
            let receiver_name = Self::extract_receiver_name(&method_call.receiver);
            
            // Semantic operation lookup: resolve the canonical operation from analyzer data
            let semantic_op = receiver_name.as_ref().and_then(|name| {
                let type_info = crate::type_info::lookup_by_name(name)?;
                type_info.method_calls.iter()
                    .find(|mc| mc.method == method_name)
                    .and_then(|mc| mc.operation.clone())
            });
            
            // Check for Weak::clone first (before generic clone handling)
            if self.config.track_smart_pointers && method_name == "clone" {
                if let Some(ref rname) = receiver_name {
                    let is_weak = semantic_op.as_ref()
                        .map(|op| op.contains("Weak") && op.contains("clone"))
                        .unwrap_or_else(|| self.weak_vars.contains_key(rname.as_str()));
                    if is_weak {
                        if let Some(weak_type) = self.weak_vars.get(rname.as_str()).copied() {
                            let location = Self::location_tokens(method_call.method.span());
                            let receiver = method_call.receiver.clone();
                            let weak_id = format!("weak_{}", rname);
                            let clone_id = format!("weak_clone_{}", self.gen_id());
                            *expr = match weak_type {
                                SmartPointerType::WeakRc => syn::parse_quote! {
                                    borrowscope_runtime::track_weak_clone(#clone_id, #weak_id, #location, #receiver.clone())
                                },
                                SmartPointerType::WeakArc => syn::parse_quote! {
                                    borrowscope_runtime::track_weak_clone_sync(#clone_id, #weak_id, #location, #receiver.clone())
                                },
                                _ => syn::parse_quote! { #receiver.clone() },
                            };
                            return;
                        }
                    }
                }
            }
            
            // Check for generic clone
            // Semantic: verify Clone::clone trait via is_trait_method/trait_name
            if self.config.track_methods && method_name == "clone" {
                let is_clone_trait = receiver_name.as_ref().and_then(|name| {
                    let type_info = crate::type_info::lookup_by_name(name)?;
                    type_info.method_calls.iter()
                        .find(|mc| mc.method == "clone")
                        .and_then(|mc| mc.is_trait_method)
                });
                // Semantic path: only track if confirmed Clone::clone (or no analyzer data)
                if is_clone_trait.unwrap_or(true) {
                    let mc = method_call.clone();
                    self.transform_clone(expr, &mc);
                    return;
                }
                // Not Clone::clone — fall through to transform_method_call for self_borrow handling
            }

            // Check for lock methods (Mutex/RwLock)
            // Semantic: use operation to disambiguate "write" (RwLock vs MaybeUninit vs io::Write)
            if self.config.track_methods {
                let is_maybe_uninit = receiver_name.as_ref()
                    .map(|n| self.maybe_uninit_vars.contains(n))
                    .unwrap_or(false);
                
                // Semantic disambiguation for "lock", "read", "write"
                let is_lock_op = semantic_op.as_ref()
                    .map(|op| op.contains("Mutex::lock") || op.contains("RwLock::read") || op.contains("RwLock::write"))
                    .unwrap_or(false);
                
                if is_lock_op || !is_maybe_uninit {
                    match method_name.as_str() {
                        "lock" => {
                            let mc = method_call.clone();
                            self.transform_lock(expr, &mc, "mutex");
                            return;
                        }
                        "read" => {
                            let mc = method_call.clone();
                            self.transform_lock(expr, &mc, "rwlock_read");
                            return;
                        }
                        "write" => {
                            let mc = method_call.clone();
                            self.transform_lock(expr, &mc, "rwlock_write");
                            return;
                        }
                        _ => {}
                    }
                }
            }

            // Check for unwrap methods — semantic: verify Option/Result via operation
            if self.config.track_methods {
                match method_name.as_str() {
                    "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" | "unwrap_or_default" => {
                        let is_option_result = semantic_op.as_ref()
                            .map(|op| op.contains("option") || op.contains("result"))
                            .unwrap_or(true); // fallback: assume yes if no analyzer data
                        if is_option_result {
                            let mc = method_call.clone();
                            self.transform_unwrap(expr, &mc);
                            return;
                        }
                    }
                    _ => {}
                }
            }

            // Check for Cow::to_mut, Weak::upgrade/clone, JoinHandle::join, channel send/recv
            if self.config.track_smart_pointers {
                if let Some(ref receiver_name) = receiver_name {
                    let location = Self::location_tokens(method_call.method.span());
                    let receiver = method_call.receiver.clone();

                    // Cow::to_mut - semantic: check operation, fallback: check cow_vars
                    let is_cow_to_mut = semantic_op.as_ref()
                        .map(|op| op.contains("Cow") && op.contains("to_mut"))
                        .unwrap_or_else(|| self.cow_vars.contains(receiver_name) && method_name == "to_mut");
                    if is_cow_to_mut && method_name == "to_mut" {
                        let cow_id = format!("cow_{}", receiver_name);
                        // We can't easily wrap to_mut since it returns &mut T
                        // Track that to_mut was called - assume it might clone (conservative)
                        *expr = syn::parse_quote! {
                            {
                                borrowscope_runtime::track_cow_to_mut(#cow_id, true, #location);
                                #receiver.to_mut()
                            }
                        };
                        return;
                    }

                    // Weak::upgrade - semantic: check operation, fallback: check weak_vars
                    let is_weak_upgrade = semantic_op.as_ref()
                        .map(|op| op.contains("Weak") && op.contains("upgrade"))
                        .unwrap_or_else(|| self.weak_vars.contains_key(receiver_name.as_str()) && method_name == "upgrade");
                    if is_weak_upgrade {
                        if let Some(weak_type) = self.weak_vars.get(receiver_name.as_str()).copied() {
                            let weak_id = format!("weak_{}", receiver_name);
                            *expr = match weak_type {
                                SmartPointerType::WeakRc => syn::parse_quote! {
                                    borrowscope_runtime::track_weak_upgrade(#weak_id, #location, #receiver.upgrade())
                                },
                                SmartPointerType::WeakArc => syn::parse_quote! {
                                    borrowscope_runtime::track_weak_upgrade_sync(#weak_id, #location, #receiver.upgrade())
                                },
                                _ => syn::parse_quote! { #receiver.upgrade() },
                            };
                            return;
                        }
                        // Weak::clone is handled earlier in the method call processing
                    }

                    // JoinHandle::join - semantic: check operation, fallback: check join_handle_vars
                    let is_join = semantic_op.as_ref()
                        .map(|op| op.contains("JoinHandle") && op.contains("join"))
                        .unwrap_or_else(|| self.join_handle_vars.contains(receiver_name) && method_name == "join");
                    if is_join && method_name == "join" {
                        let handle_id = format!("thread_{}", receiver_name);
                        *expr = syn::parse_quote! {
                            borrowscope_runtime::track_thread_join(#handle_id, #location, #receiver.join())
                        };
                        return;
                    }

                    // Sender::send - semantic: check operation, fallback: check sender_vars
                    let is_send = semantic_op.as_ref()
                        .map(|op| op.contains("Sender") && op.contains("send"))
                        .unwrap_or_else(|| self.sender_vars.contains(receiver_name) && method_name == "send");
                    if is_send && method_name == "send" {
                        let sender_id = format!("sender_{}", receiver_name);
                        let args: Vec<_> = method_call.args.iter().cloned().collect();
                        if let Some(arg) = args.first() {
                            *expr = syn::parse_quote! {
                                borrowscope_runtime::track_channel_send(#sender_id, #location, #receiver.send(#arg))
                            };
                            return;
                        }
                    }

                    // Receiver::recv/try_recv - semantic: check operation, fallback: check receiver_vars
                    let is_recv = semantic_op.as_ref()
                        .map(|op| op.contains("Receiver") && (op.contains("recv") || op.contains("try_recv")))
                        .unwrap_or_else(|| self.receiver_vars.contains(receiver_name));
                    if is_recv {
                        match method_name.as_str() {
                            "recv" => {
                                let receiver_id = format!("receiver_{}", receiver_name);
                                *expr = syn::parse_quote! {
                                    borrowscope_runtime::track_channel_recv(#receiver_id, #location, #receiver.recv())
                                };
                                return;
                            }
                            "try_recv" => {
                                let receiver_id = format!("receiver_{}", receiver_name);
                                *expr = syn::parse_quote! {
                                    borrowscope_runtime::track_channel_try_recv(#receiver_id, #location, #receiver.try_recv())
                                };
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Check for RefCell/Cell specific methods that need wrapping
            // Semantic: use operation to disambiguate "borrow" (RefCell vs Borrow trait),
            // "get" (Cell vs HashMap/Vec), "set" (Cell vs OnceCell)
            if self.config.track_smart_pointers {
                if let Some(ref receiver_name) = receiver_name {
                    let location = Self::location_tokens(method_call.method.span());
                    let borrow_id = format!("borrow_{}", self.gen_id());
                    let receiver_id = format!("refcell_{}", receiver_name);
                    let receiver = method_call.receiver.clone();
                    
                    // RefCell::borrow - semantic: check operation, fallback: match name
                    let is_refcell_borrow = semantic_op.as_ref()
                        .map(|op| op.contains("RefCell") && op.contains("borrow"))
                        .unwrap_or(false);
                    
                    match method_name.as_str() {
                        "borrow" if is_refcell_borrow || semantic_op.is_none() => {
                            // Transform cell.borrow() -> track_refcell_borrow(id, cell_id, loc, cell.borrow())
                            *expr = syn::parse_quote! {
                                borrowscope_runtime::track_refcell_borrow(#borrow_id, #receiver_id, #location, #receiver.borrow())
                            };
                            return;
                        }
                        "borrow_mut" if is_refcell_borrow || semantic_op.is_none() => {
                            // Transform cell.borrow_mut() -> track_refcell_borrow_mut(id, cell_id, loc, cell.borrow_mut())
                            *expr = syn::parse_quote! {
                                borrowscope_runtime::track_refcell_borrow_mut(#borrow_id, #receiver_id, #location, #receiver.borrow_mut())
                            };
                            return;
                        }
                        _ => {}
                    }
                    
                    // Check for OnceCell/OnceLock methods
                    // Semantic: check operation, fallback: check once_cell_vars
                    let is_once_cell = semantic_op.as_ref()
                        .map(|op| op.contains("OnceCell") || op.contains("OnceLock"))
                        .unwrap_or_else(|| self.once_cell_vars.contains(receiver_name.as_str()));
                    if is_once_cell {
                        // Semantic: match operation path, fallback: match method name
                        let once_method = semantic_op.as_ref().and_then(|op| {
                            if op.contains("::set") { Some("set") }
                            else if op.contains("::get_or_init") { Some("get_or_init") }
                            else if op.contains("::get") { Some("get") }
                            else { None }
                        }).unwrap_or(method_name.as_str());
                        let cell_id = format!("once_{}", receiver_name);
                        match once_method {
                            "set" => {
                                let args: Vec<_> = method_call.args.iter().cloned().collect();
                                if let Some(arg) = args.first() {
                                    *expr = syn::parse_quote! {
                                        borrowscope_runtime::track_once_cell_set(#cell_id, #location, #receiver.set(#arg))
                                    };
                                }
                                return;
                            }
                            "get" => {
                                *expr = syn::parse_quote! {
                                    borrowscope_runtime::track_once_cell_get(#cell_id, #location, #receiver.get())
                                };
                                return;
                            }
                            "get_or_init" => {
                                let args: Vec<_> = method_call.args.iter().cloned().collect();
                                if let Some(init_fn) = args.first() {
                                    *expr = syn::parse_quote! {
                                        {
                                            let __was_init = #receiver.get().is_some();
                                            let __result = #receiver.get_or_init(#init_fn);
                                            borrowscope_runtime::track_once_cell_get_or_init(#cell_id, __was_init, #location, __result)
                                        }
                                    };
                                }
                                return;
                            }
                            _ => {}
                        }
                    }
                    
                    // Check for MaybeUninit methods
                    // Semantic: match operation path, fallback: match method name
                    let is_maybe_uninit_op = semantic_op.as_ref()
                        .map(|op| op.contains("MaybeUninit"))
                        .unwrap_or_else(|| self.maybe_uninit_vars.contains(receiver_name.as_str()));
                    if is_maybe_uninit_op {
                        let uninit_method = semantic_op.as_ref().and_then(|op| {
                            if op.contains("::assume_init_read") { Some("assume_init_read") }
                            else if op.contains("::assume_init_drop") { Some("assume_init_drop") }
                            else if op.contains("::assume_init") { Some("assume_init") }
                            else if op.contains("::write") { Some("write") }
                            else { None }
                        }).unwrap_or(method_name.as_str());
                        let uninit_id = format!("uninit_{}", receiver_name);
                        match uninit_method {
                            "write" => {
                                let args: Vec<_> = method_call.args.iter().cloned().collect();
                                if let Some(arg) = args.first() {
                                    *expr = syn::parse_quote! {
                                        borrowscope_runtime::track_maybe_uninit_write(#uninit_id, #location, #receiver.write(#arg))
                                    };
                                }
                                return;
                            }
                            "assume_init" => {
                                *expr = syn::parse_quote! {
                                    borrowscope_runtime::track_maybe_uninit_assume_init(#uninit_id, #location, #receiver.assume_init())
                                };
                                return;
                            }
                            "assume_init_read" => {
                                *expr = syn::parse_quote! {
                                    borrowscope_runtime::track_maybe_uninit_assume_init_read(#uninit_id, #location, #receiver.assume_init_read())
                                };
                                return;
                            }
                            "assume_init_drop" => {
                                *expr = syn::parse_quote! {
                                    {
                                        #receiver.assume_init_drop();
                                        borrowscope_runtime::track_maybe_uninit_assume_init_drop(#uninit_id, #location)
                                    }
                                };
                                return;
                            }
                            _ => {}
                        }
                    }
                    
                    // Cell get/set methods
                    // Semantic: check operation to disambiguate "get" (Cell vs HashMap/Vec)
                    // and "set" (Cell vs OnceCell vs user types), fallback: non-OnceCell heuristic
                    let is_cell_op = semantic_op.as_ref()
                        .map(|op| op.contains("Cell") && (op.contains("get") || op.contains("set")) && !op.contains("OnceCell"))
                        .unwrap_or_else(|| !self.once_cell_vars.contains(receiver_name.as_str()));
                    if is_cell_op {
                        match method_name.as_str() {
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

        // Handle non-pointer casts BEFORE visiting nested expressions
        // This prevents issues with chained casts like `n as u8 as char`
        if self.config.track_expressions {
            if let Expr::Cast(cast_expr) = expr {
                // Skip pointer casts - they're handled separately for unsafe tracking
                if !matches!(cast_expr.ty.as_ref(), syn::Type::Ptr(_)) {
                    let cast_id = self.gen_id();
                    let span = cast_expr.as_token.span;
                    let location = Self::location_tokens(span);
                    let ty = cast_expr.ty.clone();
                    let to_type_str = quote::quote!(#ty).to_string();
                    
                    // First, recursively visit the inner expression
                    self.visit_expr_mut(&mut cast_expr.expr);
                    
                    // Get the transformed inner expression
                    let transformed_inner = cast_expr.expr.clone();
                    
                    // If the inner expression is a block (from a previous cast transformation),
                    // we need to wrap it in parentheses for the cast to be valid
                    let cast_expr_final: Expr = if matches!(*transformed_inner, Expr::Block(_)) {
                        syn::parse_quote! { (#transformed_inner) as #ty }
                    } else {
                        syn::parse_quote! { #transformed_inner as #ty }
                    };
                    
                    *expr = syn::parse_quote! {
                        {
                            borrowscope_runtime::track_type_cast(#cast_id, #to_type_str, #location);
                            #cast_expr_final
                        }
                    };
                    
                    // Don't visit nested expressions again - we've already handled them
                    return;
                }
            }
        }

        // First recursively visit nested expressions
        visit_mut::visit_expr_mut(self, expr);

        // Then transform reference expressions at this level
        if let Expr::Reference(ref_expr) = expr.clone() {
            self.transform_reference(expr, &ref_expr);
        }

        // Transform unsafe blocks
        if self.config.track_unsafe {
            if let Expr::Unsafe(unsafe_expr) = expr {
                self.transform_unsafe_block(unsafe_expr);
            }
        }

        // Transform pointer casts (for unsafe tracking)
        if self.config.track_unsafe {
            if let Expr::Cast(cast_expr) = expr.clone() {
                if matches!(cast_expr.ty.as_ref(), syn::Type::Ptr(_)) {
                    self.transform_ptr_cast(expr, &cast_expr);
                }
            }
        }

        // Transform transmute calls
        if self.config.track_unsafe {
            if let Expr::Call(call_expr) = expr.clone() {
                self.transform_call_expr(expr, &call_expr);
            }
        }

        // Warn about potential static variable access
        if self.config.warn_ambiguous {
            if let Expr::Path(path_expr) = expr {
                if let Some(ident) = path_expr.path.get_ident() {
                    let name = ident.to_string();
                    // Skip if it's a known static (user declared)
                    if !self.config.known_statics.iter().any(|s| s == &name) {
                        // Check if it looks like a static (SCREAMING_SNAKE_CASE)
                        if crate::diagnostics::looks_like_static(&name) {
                            let warning = crate::diagnostics::create_ambiguous_warning(
                                crate::diagnostics::AmbiguousPattern::PossibleStatic,
                                &name,
                                ident.span(),
                            );
                            self.add_warning(warning);
                        }
                    }
                }
            }

            // Warn about potential union field access
            if let Expr::Field(field_expr) = expr {
                // Try to get the base type name
                if let Expr::Path(base_path) = field_expr.base.as_ref() {
                    if let Some(ident) = base_path.path.get_ident() {
                        let base_name = ident.to_string();
                        // Skip if it's a known union (user declared)
                        if !self.config.known_unions.iter().any(|u| u == &base_name) {
                            // Check if it looks like a union
                            if crate::diagnostics::looks_like_union(&base_name) {
                                if let syn::Member::Named(field_ident) = &field_expr.member {
                                    let warning = crate::diagnostics::create_ambiguous_warning(
                                        crate::diagnostics::AmbiguousPattern::PossibleUnion,
                                        &base_name,
                                        field_ident.span(),
                                    );
                                    self.add_warning(warning);
                                }
                            }
                        }
                    }
                }
            }
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

    #[test]
    fn test_glob_match_exact() {
        assert!(OwnershipVisitor::glob_match("hello", "hello"));
        assert!(!OwnershipVisitor::glob_match("hello", "world"));
    }

    #[test]
    fn test_glob_match_star_suffix() {
        assert!(OwnershipVisitor::glob_match("data*", "data"));
        assert!(OwnershipVisitor::glob_match("data*", "data_input"));
        assert!(OwnershipVisitor::glob_match("data*", "database"));
        assert!(!OwnershipVisitor::glob_match("data*", "mydata"));
    }

    #[test]
    fn test_glob_match_star_prefix() {
        assert!(OwnershipVisitor::glob_match("*_count", "item_count"));
        assert!(OwnershipVisitor::glob_match("*_count", "user_count"));
        assert!(OwnershipVisitor::glob_match("*_count", "_count"));
        assert!(!OwnershipVisitor::glob_match("*_count", "count"));
    }

    #[test]
    fn test_glob_match_star_middle() {
        assert!(OwnershipVisitor::glob_match("user*data", "userdata"));
        assert!(OwnershipVisitor::glob_match("user*data", "user_data"));
        assert!(OwnershipVisitor::glob_match("user*data", "user123data"));
        assert!(!OwnershipVisitor::glob_match("user*data", "userdat"));
    }

    #[test]
    fn test_glob_match_question_mark() {
        assert!(OwnershipVisitor::glob_match("x?", "x1"));
        assert!(OwnershipVisitor::glob_match("x?", "xa"));
        assert!(!OwnershipVisitor::glob_match("x?", "x"));
        assert!(!OwnershipVisitor::glob_match("x?", "x12"));
    }

    #[test]
    fn test_glob_match_combined() {
        assert!(OwnershipVisitor::glob_match("*_?", "data_1"));
        assert!(OwnershipVisitor::glob_match("*_?", "x_a"));
        assert!(!OwnershipVisitor::glob_match("*_?", "data_12"));
    }

    #[test]
    fn test_matches_filter_no_filter() {
        let visitor = OwnershipVisitor::new();
        assert!(visitor.matches_filter("anything"));
        assert!(visitor.matches_filter("data"));
        assert!(visitor.matches_filter("x"));
    }

    #[test]
    fn test_matches_filter_with_pattern() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("data*".to_string());
        let visitor = OwnershipVisitor::with_config(config);
        
        assert!(visitor.matches_filter("data"));
        assert!(visitor.matches_filter("data_input"));
        assert!(!visitor.matches_filter("temp"));
        assert!(!visitor.matches_filter("mydata"));
    }

    #[test]
    fn test_matches_filter_with_prefix() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("name:user*".to_string());
        let visitor = OwnershipVisitor::with_config(config);
        
        assert!(visitor.matches_filter("user_id"));
        assert!(visitor.matches_filter("username"));
        assert!(!visitor.matches_filter("admin"));
    }

    #[test]
    fn test_filter_skips_non_matching_vars() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("data*".to_string());
        let mut visitor = OwnershipVisitor::with_config(config);

        let mut stmt: Stmt = parse_quote! {
            let temp = 42;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        // Should NOT contain track_new because "temp" doesn't match "data*"
        assert!(!output.contains("track_new"), "temp should not be tracked with filter 'data*'");
    }

    #[test]
    fn test_filter_tracks_matching_vars() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("data*".to_string());
        let mut visitor = OwnershipVisitor::with_config(config);

        let mut stmt: Stmt = parse_quote! {
            let data_input = 42;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        // Should contain track_new because "data_input" matches "data*"
        assert!(output.contains("track_new"), "data_input should be tracked with filter 'data*'");
    }

    #[test]
    fn test_sample_rate_generates_sampled_call() {
        let mut config = TraceConfig::standard();
        config.sample_rate = Some(0.1);
        let mut visitor = OwnershipVisitor::with_config(config);

        let mut stmt: Stmt = parse_quote! {
            let x = 42;
        };

        visitor.visit_stmt_mut(&mut stmt);

        let output = stmt.to_token_stream().to_string();
        // Should use sampled version
        assert!(output.contains("track_new_with_id_sampled"), "Should use sampled tracking");
        assert!(output.contains("0.1"), "Should include sample rate");
    }
}
