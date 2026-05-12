//! Ownership analysis engine.
//!
//! Extracts exhaustive type information from the Salsa database
//! using all available hir::Type methods.

use ra_ap_hir::{self as hir, HirDisplay, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{ast, AstNode, Edition};
use ra_ap_syntax::ast::{HasName, HasArgList};
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════════════════
// Data structures
// ═══════════════════════════════════════════════════════════════════════════

/// Complete ownership information for a single variable.
#[derive(Debug, Clone, Serialize, Default)]
pub struct VariableOwnershipInfo {
    // Identity
    pub name: String,
    pub type_display: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function_name: Option<String>,

    // Boolean type properties (no db needed)
    pub is_unit: bool,
    pub is_bool: bool,
    pub is_str: bool,
    pub is_never: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_slice: bool,
    pub is_usize: bool,
    pub is_float: bool,
    pub is_char: bool,
    pub is_int_or_uint: bool,
    pub is_scalar: bool,
    pub is_tuple: bool,
    pub is_array: bool,
    pub is_closure: bool,
    pub is_fn: bool,
    pub is_raw_ptr: bool,
    pub is_unknown: bool,
    pub contains_unknown: bool,

    // Queries requiring db
    pub is_copy: bool,
    pub is_packed: bool,
    pub contains_reference: bool,
    pub impls_fnonce: bool,
    pub impls_iterator: bool,

    // Decomposition
    pub reference_inner: Option<TypeDecomposition>,
    pub adt_info: Option<AdtInfo>,
    pub builtin_type: Option<String>,
    pub dyn_trait: Option<String>,
    pub impl_traits: Vec<String>,
    pub type_arguments: Vec<String>,
    pub future_output: Option<String>,
    pub iterator_item: Option<String>,
    pub tuple_fields: Vec<String>,
    pub struct_fields: Vec<FieldInfo>,
    pub array_info: Option<ArrayInfo>,
    pub autoderef_chain: Vec<String>,
    pub callable_info: Option<CallableInfo>,

    // Layout
    pub layout_size: Option<u64>,
    pub layout_align: Option<u64>,
    pub has_drop_glue: bool,

    // ADT classification
    pub adt_canonical_path: Option<String>,
    pub ownership_category: OwnershipCategory,

    // Trait implementations
    pub trait_impls: TraitImplInfo,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TypeDecomposition {
    pub inner_type: String,
    pub mutability: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AdtInfo {
    pub kind: String,
    pub name: String,
    pub canonical_path: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ArrayInfo {
    pub element_type: String,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CallableInfo {
    pub params: Vec<String>,
    pub return_type: String,
    pub is_closure: bool,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum OwnershipCategory {
    Owned,
    SharedRef,
    MutableRef,
    SharedOwnership,
    InteriorMut,
    RawPointer,
    Copy,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TraitImplInfo {
    pub is_send: bool,
    pub is_sync: bool,
    pub is_clone: bool,
    pub is_drop: bool,
    pub is_sized: bool,
    pub is_debug: bool,
    pub is_display: bool,
    pub is_default: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Extraction
// ═══════════════════════════════════════════════════════════════════════════

/// Extract all ownership info for a single type.
pub fn extract_full_type_info(
    db: &RootDatabase,
    display_target: &hir::DisplayTarget,
    ty: &hir::Type<'_>,
    name: &str,
    file: &str,
    line: u32,
    column: u32,
    function_name: Option<&str>,
) -> VariableOwnershipInfo {
    let mut info = VariableOwnershipInfo {
        name: name.to_string(),
        type_display: ty.display(db, *display_target).to_string(),
        file: file.to_string(),
        line,
        column,
        function_name: function_name.map(|s| s.to_string()),
        ..Default::default()
    };

    // ── Boolean queries (no db) ──
    info.is_unit = ty.is_unit();
    info.is_bool = ty.is_bool();
    info.is_str = ty.is_str();
    info.is_never = ty.is_never();
    info.is_reference = ty.is_reference();
    info.is_mutable_reference = ty.is_mutable_reference();
    info.is_slice = ty.is_slice();
    info.is_usize = ty.is_usize();
    info.is_float = ty.is_float();
    info.is_char = ty.is_char();
    info.is_int_or_uint = ty.is_int_or_uint();
    info.is_scalar = ty.is_scalar();
    info.is_tuple = ty.is_tuple();
    info.is_array = ty.is_array();
    info.is_closure = ty.is_closure();
    info.is_fn = ty.is_fn();
    info.is_raw_ptr = ty.is_raw_ptr();
    info.is_unknown = ty.is_unknown();
    info.contains_unknown = ty.contains_unknown();

    // ── Queries requiring db ──
    info.is_copy = ty.is_copy(db);
    info.is_packed = ty.is_packed(db);
    info.contains_reference = ty.contains_reference(db);
    info.impls_fnonce = ty.impls_fnonce(db);
    info.impls_iterator = ty.clone().impls_iterator(db);

    // ── Reference decomposition ──
    if let Some((inner, mutability)) = ty.as_reference() {
        info.reference_inner = Some(TypeDecomposition {
            inner_type: inner.display(db, *display_target).to_string(),
            mutability: format!("{:?}", mutability),
        });
    }

    // ── ADT info ──
    if let Some(adt) = ty.as_adt() {
        let kind = match adt {
            hir::Adt::Struct(_) => "struct",
            hir::Adt::Enum(_) => "enum",
            hir::Adt::Union(_) => "union",
        };
        let module = adt.module(db);
        let canonical_path = module.path_to_root(db).iter().rev().filter_map(|m| m.name(db)).map(|n| n.display_no_db(Edition::Edition2021).to_string()).collect::<Vec<_>>().join("::");
        let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();

        info.adt_info = Some(AdtInfo {
            kind: kind.to_string(),
            name: adt_name.clone(),
            canonical_path: format!("{}::{}", canonical_path, adt_name),
        });
        info.adt_canonical_path = Some(format!("{}::{}", canonical_path, adt_name));
    }

    // ── Builtin type ──
    if let Some(builtin) = ty.as_builtin() {
        info.builtin_type = Some(builtin.name().display_no_db(Edition::Edition2021).to_string());
    }

    // ── Dyn trait ──
    if let Some(trait_) = ty.as_dyn_trait() {
        info.dyn_trait = Some(trait_.name(db).display_no_db(Edition::Edition2021).to_string());
    }

    // ── Impl traits ──
    if let Some(traits) = ty.as_impl_traits(db) {
        info.impl_traits = traits.map(|t| t.name(db).display_no_db(Edition::Edition2021).to_string()).collect();
    }

    // ── Type arguments ──
    info.type_arguments = ty.type_arguments().map(|t| t.display(db, *display_target).to_string()).collect();

    // ── Future output ──
    info.future_output = ty.clone().future_output(db).map(|t| t.display(db, *display_target).to_string());

    // ── Iterator item ──
    info.iterator_item = ty.clone().iterator_item(db).map(|t| t.display(db, *display_target).to_string());

    // ── Tuple fields ──
    info.tuple_fields = ty.tuple_fields(db).iter().map(|t| t.display(db, *display_target).to_string()).collect();

    // ── Struct fields ──
    info.struct_fields = ty
        .fields(db)
        .iter()
        .map(|(field, field_ty)| FieldInfo {
            name: field.name(db).display_no_db(Edition::Edition2021).to_string(),
            ty: field_ty.display(db, *display_target).to_string(),
        })
        .collect();

    // ── Array info ──
    if let Some((elem_ty, length)) = ty.as_array(db) {
        info.array_info = Some(ArrayInfo {
            element_type: elem_ty.display(db, *display_target).to_string(),
            length,
        });
    }

    // ── Autoderef chain ──
    info.autoderef_chain = ty
        .autoderef(db)
        .map(|t| t.display(db, *display_target).to_string())
        .collect();

    // ── Callable info ──
    if let Some(callable) = ty.as_callable(db) {
        info.callable_info = Some(CallableInfo {
            params: callable
                .params()
                .iter()
                .map(|p| p.ty().display(db, *display_target).to_string())
                .collect(),
            return_type: callable.return_type().display(db, *display_target).to_string(),
            is_closure: ty.is_closure(),
        });
    }

    // ── Layout ──
    if let Ok(layout) = ty.layout(db) {
        info.layout_size = Some(layout.size());
        info.layout_align = Some(layout.align());
    }

    // ── Drop glue ──
    info.has_drop_glue = !matches!(ty.drop_glue(db), hir::DropGlue::None);

    // ── Ownership category ──
    info.ownership_category = classify_ownership(db, ty, &info);

    // ── Trait impls ──
    info.trait_impls = check_traits(db, ty);

    info
}

// ═══════════════════════════════════════════════════════════════════════════
// Classification
// ═══════════════════════════════════════════════════════════════════════════

fn classify_ownership(
    db: &RootDatabase,
    ty: &hir::Type<'_>,
    info: &VariableOwnershipInfo,
) -> OwnershipCategory {
    if info.is_unknown {
        return OwnershipCategory::Unknown;
    }
    if info.is_raw_ptr {
        return OwnershipCategory::RawPointer;
    }
    if info.is_mutable_reference {
        return OwnershipCategory::MutableRef;
    }
    if info.is_reference {
        return OwnershipCategory::SharedRef;
    }
    if info.is_copy {
        return OwnershipCategory::Copy;
    }

    // Check ADT path for smart pointers
    if let Some(ref path) = info.adt_canonical_path {
        let p = path.to_lowercase();
        if p.contains("rc::rc") || p.contains("sync::arc") || p == "rc" || p.ends_with("::rc") {
            return OwnershipCategory::SharedOwnership;
        }
        if p.contains("cell::refcell")
            || p.contains("cell::cell")
            || p.contains("sync::mutex")
            || p.contains("sync::rwlock")
        {
            return OwnershipCategory::InteriorMut;
        }
    }

    OwnershipCategory::Owned
}

fn check_traits(db: &RootDatabase, ty: &hir::Type<'_>) -> TraitImplInfo {
    // Note: checking specific traits requires looking them up by name.
    // For now, we use the available methods. Full trait checking
    // will be expanded when we have trait lookup infrastructure.
    TraitImplInfo {
        is_send: false, // Requires trait lookup (Send is not a lang item)
        is_sync: false, // Requires trait lookup
        is_clone: false, // Requires trait lookup
        is_drop: false, // Requires trait lookup
        is_sized: true, // Most types are sized
        is_debug: false,
        is_display: false,
        is_default: false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.2 Method Call Resolution
// ═══════════════════════════════════════════════════════════════════════════

/// How a method borrows self.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SelfBorrow {
    Shared,    // &self
    Exclusive, // &mut self
    Owned,     // self (consuming)
}

/// Resolved method call information.
#[derive(Debug, Clone, Serialize)]
pub struct MethodCallResolution {
    pub method_name: String,
    pub line: u32,
    pub column: u32,
    pub canonical_path: String,
    pub self_borrow: SelfBorrow,
    pub receiver_type: String,
    pub return_type: String,
    pub is_trait_method: bool,
    pub trait_name: Option<String>,
    pub is_unsafe: bool,
}

/// Resolve all method calls in a function body.
pub fn resolve_method_calls(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    body: &ra_ap_syntax::ast::BlockExpr,
) -> Vec<MethodCallResolution> {
    use ra_ap_syntax::{ast, AstNode, TextSize};

    let mut results = Vec::new();

    for node in body.syntax().descendants() {
        let method_call = match ast::MethodCallExpr::cast(node) {
            Some(mc) => mc,
            None => continue,
        };

        let func = match sema.resolve_method_call(&method_call) {
            Some(f) => f,
            None => continue, // Unresolvable - skip without panic
        };

        // Method name
        let method_name = method_call
            .name_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();

        // Position
        let offset = method_call.syntax().text_range().start();
        let line = 0u32; // Would need line index to compute properly
        let column = u32::from(offset);

        // Self borrow type
        let self_borrow = match func.self_param(db) {
            Some(param) => match param.access(db) {
                hir::Access::Shared => SelfBorrow::Shared,
                hir::Access::Exclusive => SelfBorrow::Exclusive,
                hir::Access::Owned => SelfBorrow::Owned,
            },
            None => SelfBorrow::Owned,
        };

        // Canonical path
        let canonical_path = build_function_path(db, &func);

        // Receiver type
        let receiver_type = method_call
            .receiver()
            .and_then(|recv| sema.type_of_expr(&recv))
            .map(|ti| ti.original.display(db, *display_target).to_string())
            .unwrap_or_default();

        // Return type
        let return_type = func
            .ret_type(db)
            .display(db, *display_target)
            .to_string();

        // Trait method detection
        let (is_trait_method, trait_name) = resolve_trait_info(db, &func);

        // Unsafe
        let is_unsafe = func.is_unsafe_to_call(db, None, Edition::Edition2021);

        results.push(MethodCallResolution {
            method_name,
            line,
            column,
            canonical_path,
            self_borrow,
            receiver_type,
            return_type,
            is_trait_method,
            trait_name,
            is_unsafe,
        });
    }

    results
}

/// Build a canonical path for a function (module::name).
fn build_function_path(db: &RootDatabase, func: &hir::Function) -> String {
    let module = func.module(db);
    let module_path: String = module
        .path_to_root(db)
        .iter()
        .rev()
        .filter_map(|m| m.name(db))
        .map(|n| n.display_no_db(Edition::Edition2021).to_string())
        .collect::<Vec<_>>()
        .join("::");

    let func_name = func.name(db).display_no_db(Edition::Edition2021).to_string();

    if module_path.is_empty() {
        func_name
    } else {
        format!("{}::{}", module_path, func_name)
    }
}

/// Detect if a function is a trait method and get the trait name.
fn resolve_trait_info(db: &RootDatabase, func: &hir::Function) -> (bool, Option<String>) {
    let assoc_item = hir::AssocItem::Function(*func);
    match assoc_item.container(db) {
        hir::AssocItemContainer::Trait(t) => {
            let name = t.name(db).display_no_db(Edition::Edition2021).to_string();
            (true, Some(name))
        }
        hir::AssocItemContainer::Impl(i) => {
            if let Some(trait_ref) = i.trait_(db) {
                let name = trait_ref.name(db).display_no_db(Edition::Edition2021).to_string();
                (true, Some(name))
            } else {
                (false, None)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.3 Borrow Scope Computation
// ═══════════════════════════════════════════════════════════════════════════

/// Information about a borrow's active scope.
#[derive(Debug, Clone, Serialize)]
pub struct BorrowScopeInfo {
    pub borrower_name: String,
    pub target_name: String,
    pub is_mutable: bool,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Compute borrow scopes for all reference bindings in a function.
pub fn compute_borrow_scopes(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    function: &ra_ap_syntax::ast::Fn,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<BorrowScopeInfo> {
    use ra_ap_ide_db::defs::Definition;
    use ra_ap_syntax::{ast, AstNode, TextRange};
    use ra_ap_syntax::ast::HasName;

    let body = match function.body() {
        Some(b) => b,
        None => return vec![],
    };

    let mut scopes = Vec::new();

    // Find all let statements that bind references
    for node in body.syntax().descendants() {
        let let_stmt = match ast::LetStmt::cast(node) {
            Some(ls) => ls,
            None => continue,
        };

        // Get the pattern (variable name)
        let pat = match let_stmt.pat() {
            Some(p) => p,
            None => continue,
        };

        // Check if the initializer is a reference expression
        let initializer = match let_stmt.initializer() {
            Some(init) => init,
            None => continue,
        };

        // Determine if this is a borrow and its mutability
        let (is_borrow, is_mutable, target_name) = analyze_borrow_expr(sema, &initializer);
        if !is_borrow {
            continue;
        }

        // Get the borrower name
        let borrower_name = pat.syntax().text().to_string().trim().to_string();

        // Get start position (the let statement)
        let start_offset = let_stmt.syntax().text_range().start();
        let (start_line, start_col) = line_index(start_offset);

        // Find last use of this variable to determine scope end
        let end_offset = find_last_use(sema, &pat, &body);
        let (end_line, end_col) = line_index(end_offset);

        scopes.push(BorrowScopeInfo {
            borrower_name,
            target_name,
            is_mutable,
            start_line,
            start_col,
            end_line,
            end_col,
        });
    }

    scopes
}

/// Analyze if an expression is a borrow (&x or &mut x) and extract target name.
fn analyze_borrow_expr(
    sema: &hir::Semantics<'_, RootDatabase>,
    expr: &ra_ap_syntax::ast::Expr,
) -> (bool, bool, String) {
    use ra_ap_syntax::ast;

    match expr {
        ra_ap_syntax::ast::Expr::RefExpr(ref_expr) => {
            let is_mutable = ref_expr.mut_token().is_some();
            let target_name = ref_expr
                .expr()
                .map(|e| e.syntax().text().to_string().trim().to_string())
                .unwrap_or_default();
            (true, is_mutable, target_name)
        }
        _ => {
            // Check if the type is a reference (e.g., function returning &T)
            if let Some(ty) = sema.type_of_expr(expr) {
                if ty.original.is_reference() {
                    let is_mutable = ty.original.is_mutable_reference();
                    let target_name = expr.syntax().text().to_string().trim().to_string();
                    return (true, is_mutable, target_name);
                }
            }
            (false, false, String::new())
        }
    }
}

/// Find the last use of a pattern (variable) within a block.
/// Returns the text offset of the last usage, or the pattern's own offset if no uses found.
fn find_last_use(
    sema: &hir::Semantics<'_, RootDatabase>,
    pat: &ra_ap_syntax::ast::Pat,
    body: &ra_ap_syntax::ast::BlockExpr,
) -> ra_ap_syntax::TextSize {
    use ra_ap_ide_db::defs::Definition;
    use ra_ap_syntax::{ast, AstNode};

    let pat_offset = pat.syntax().text_range().end();

    // Try to resolve the pattern to a Local
    let ident_pat = match pat.syntax().descendants().find_map(ast::IdentPat::cast) {
        Some(ip) => ip,
        None => return pat_offset,
    };

    let local: hir::Local = match sema.to_def(&ident_pat) {
        Some(l) => l,
        None => return pat_offset,
    };

    // Find all usages
    let def = Definition::Local(local);
    let usages = def.usages(sema).all();

    // Find the maximum offset among all usages
    let mut last_offset = pat_offset;
    for (_file_id, refs) in usages.references {
        for reference in refs {
            let ref_end = reference.range.end();
            if ref_end > last_offset {
                last_offset = ref_end;
            }
        }
    }

    last_offset
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.4 Move Detection
// ═══════════════════════════════════════════════════════════════════════════

/// Where a value was moved to.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum MoveDestination {
    Variable(String),
    FunctionArg(String),
    Return,
    ClosureCapture(String),
}

/// Information about a detected move.
#[derive(Debug, Clone, Serialize)]
pub struct MoveInfo {
    pub source_name: String,
    pub destination: MoveDestination,
    pub line: u32,
    pub column: u32,
    pub source_type: String,
}

/// Detect all ownership transfers (moves) in a function.
pub fn detect_moves(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    function: &ast::Fn,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<MoveInfo> {
    let body = match function.body() {
        Some(b) => b,
        None => return vec![],
    };

    let mut moves = Vec::new();

    for node in body.syntax().descendants() {
        // Case 1: let b = a; (assignment move)
        if let Some(let_stmt) = ast::LetStmt::cast(node.clone()) {
            if let Some(move_info) = detect_let_move(db, sema, display_target, &let_stmt, line_index) {
                moves.push(move_info);
            }
            continue;
        }

        // Case 2: foo(a) (function argument move)
        if let Some(call_expr) = ast::CallExpr::cast(node.clone()) {
            moves.extend(detect_call_arg_moves(db, sema, display_target, &call_expr, line_index));
            continue;
        }

        // Case 3: return expr (return move)
        if let Some(return_expr) = ast::ReturnExpr::cast(node.clone()) {
            if let Some(move_info) = detect_return_move(db, sema, display_target, &return_expr, line_index) {
                moves.push(move_info);
            }
            continue;
        }

        // Case 4: move || { ... } (closure capture move)
        if let Some(closure) = ast::ClosureExpr::cast(node.clone()) {
            moves.extend(detect_closure_capture_moves(db, sema, display_target, &closure, line_index));
        }
    }

    moves
}

fn detect_let_move(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    let_stmt: &ast::LetStmt,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Option<MoveInfo> {
    let init = let_stmt.initializer()?;
    let pat = let_stmt.pat()?;

    // Only path expressions (variable references) can be moves
    let path_expr = match &init {
        ast::Expr::PathExpr(p) => p,
        _ => return None,
    };

    // Get the type of the initializer
    let ty_info = sema.type_of_expr(&init)?;
    let ty = ty_info.original;

    // Copy types don't move
    if ty.is_copy(db) {
        return None;
    }

    // References are not moves
    if ty.is_reference() {
        return None;
    }

    let source_name = path_expr.syntax().text().to_string().trim().to_string();
    let dest_name = pat.syntax().text().to_string().trim().to_string();
    let (line, column) = line_index(let_stmt.syntax().text_range().start());

    Some(MoveInfo {
        source_name,
        destination: MoveDestination::Variable(dest_name),
        line,
        column,
        source_type: ty.display(db, *display_target).to_string(),
    })
}

fn detect_call_arg_moves(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    call_expr: &ast::CallExpr,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<MoveInfo> {
    let mut moves = Vec::new();

    // Get function name
    let fn_name = call_expr
        .expr()
        .map(|e| e.syntax().text().to_string().trim().to_string())
        .unwrap_or_default();

    // Check each argument
    let arg_list = match call_expr.arg_list() {
        Some(al) => al,
        None => return moves,
    };

    for arg in arg_list.args() {
        // Only path expressions (variable references) can be moves
        if !matches!(&arg, ast::Expr::PathExpr(_)) {
            continue;
        }

        let ty_info = match sema.type_of_expr(&arg) {
            Some(ti) => ti,
            None => continue,
        };
        let ty = ty_info.original;

        // Copy types and references don't move
        if ty.is_copy(db) || ty.is_reference() {
            continue;
        }

        let source_name = arg.syntax().text().to_string().trim().to_string();
        let (line, column) = line_index(call_expr.syntax().text_range().start());

        moves.push(MoveInfo {
            source_name,
            destination: MoveDestination::FunctionArg(fn_name.clone()),
            line,
            column,
            source_type: ty.display(db, *display_target).to_string(),
        });
    }

    moves
}

fn detect_return_move(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    return_expr: &ast::ReturnExpr,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Option<MoveInfo> {
    let expr = return_expr.expr()?;

    // Only path expressions
    if !matches!(&expr, ast::Expr::PathExpr(_)) {
        return None;
    }

    let ty_info = sema.type_of_expr(&expr)?;
    let ty = ty_info.original;

    if ty.is_copy(db) || ty.is_reference() {
        return None;
    }

    let source_name = expr.syntax().text().to_string().trim().to_string();
    let (line, column) = line_index(return_expr.syntax().text_range().start());

    Some(MoveInfo {
        source_name,
        destination: MoveDestination::Return,
        line,
        column,
        source_type: ty.display(db, *display_target).to_string(),
    })
}

fn detect_closure_capture_moves(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    closure: &ast::ClosureExpr,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<MoveInfo> {
    let mut moves = Vec::new();

    // Only `move` closures capture by move
    if closure.move_token().is_none() {
        return moves;
    }

    // Get the closure type and check captures via hir
    let Some(ty_info) = sema.type_of_expr(&ast::Expr::ClosureExpr(closure.clone())) else {
        return moves;
    };

    let Some(closure_hir) = ty_info.original.as_closure() else {
        return moves;
    };

    let (line, column) = line_index(closure.syntax().text_range().start());

    for capture in closure_hir.captured_items(db) {
        let local = capture.local();
        let name = local.name(db).display_no_db(Edition::Edition2021).to_string();
        let ty = local.ty(db);

        // Only non-Copy, non-reference types are actual moves
        if ty.is_copy(db) || ty.is_reference() {
            continue;
        }

        moves.push(MoveInfo {
            source_name: name.clone(),
            destination: MoveDestination::ClosureCapture(name),
            line,
            column,
            source_type: ty.display(db, *display_target).to_string(),
        });
    }

    moves
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.5 Closure Capture Analysis
// ═══════════════════════════════════════════════════════════════════════════

/// Which Fn trait a closure implements.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum FnTrait {
    Fn,
    FnMut,
    FnOnce,
}

/// How a variable is captured by a closure.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum CaptureMode {
    BySharedRef,
    ByMutRef,
    ByMove,
}

/// A variable captured by a closure.
#[derive(Debug, Clone, Serialize)]
pub struct CapturedVariable {
    pub name: String,
    pub capture_mode: CaptureMode,
    pub variable_type: String,
}

/// Information about a closure's captures.
#[derive(Debug, Clone, Serialize)]
pub struct ClosureCaptureInfo {
    pub closure_line: u32,
    pub closure_column: u32,
    pub fn_trait: FnTrait,
    pub captures: Vec<CapturedVariable>,
}

/// Analyze all closures in a function.
pub fn analyze_closures(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    function: &ast::Fn,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<ClosureCaptureInfo> {
    use ra_ap_hir::CaptureKind;

    let body = match function.body() {
        Some(b) => b,
        None => return vec![],
    };

    let mut results = Vec::new();

    for node in body.syntax().descendants() {
        let closure_expr = match ast::ClosureExpr::cast(node) {
            Some(c) => c,
            None => continue,
        };

        // Get the closure's type
        let ty_info = match sema.type_of_expr(&ast::Expr::ClosureExpr(closure_expr.clone())) {
            Some(ti) => ti,
            None => continue,
        };

        let closure_hir = match ty_info.original.as_closure() {
            Some(c) => c,
            None => continue,
        };

        let (closure_line, closure_column) =
            line_index(closure_expr.syntax().text_range().start());

        // Determine Fn trait
        let fn_trait_str = closure_hir.fn_trait(db).to_string();
        let fn_trait = match fn_trait_str.as_str() {
            "Fn" => FnTrait::Fn,
            "FnMut" => FnTrait::FnMut,
            _ => FnTrait::FnOnce,
        };

        // Get captured variables
        let mut captures = Vec::new();
        for capture in closure_hir.captured_items(db) {
            let local = capture.local();
            let name = local.name(db).display_no_db(Edition::Edition2021).to_string();
            let ty = local.ty(db);

            let capture_mode = match capture.kind() {
                CaptureKind::SharedRef => CaptureMode::BySharedRef,
                CaptureKind::UniqueSharedRef => CaptureMode::BySharedRef,
                CaptureKind::MutableRef => CaptureMode::ByMutRef,
                CaptureKind::Move => CaptureMode::ByMove,
            };

            captures.push(CapturedVariable {
                name,
                capture_mode,
                variable_type: ty.display(db, *display_target).to_string(),
            });
        }

        results.push(ClosureCaptureInfo {
            closure_line,
            closure_column,
            fn_trait,
            captures,
        });
    }

    results
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.6 Rc/Arc Clone Tracking
// ═══════════════════════════════════════════════════════════════════════════

/// Whether a clone is Rc or Arc.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RcType {
    Rc,
    Arc,
}

/// Information about an Rc/Arc clone operation.
#[derive(Debug, Clone, Serialize)]
pub struct RcCloneInfo {
    pub clone_variable: String,
    pub source_variable: String,
    pub clone_type: RcType,
    pub line: u32,
    pub column: u32,
}

/// Track all Rc::clone() and Arc::clone() calls in a function.
pub fn track_rc_clones(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    function: &ast::Fn,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Vec<RcCloneInfo> {
    let body = match function.body() {
        Some(b) => b,
        None => return vec![],
    };

    let mut results = Vec::new();

    for node in body.syntax().descendants() {
        // Case 1: let b = a.clone() where a is Rc/Arc
        if let Some(let_stmt) = ast::LetStmt::cast(node.clone()) {
            if let Some(info) = detect_method_clone(db, sema, display_target, &let_stmt, line_index) {
                results.push(info);
                continue;
            }
            // Case 2: let b = Rc::clone(&a) or Arc::clone(&a)
            if let Some(info) = detect_explicit_clone(db, sema, display_target, &let_stmt, line_index) {
                results.push(info);
            }
        }
    }

    results
}

/// Detect `let b = a.clone()` where a is Rc<T> or Arc<T>.
fn detect_method_clone(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    let_stmt: &ast::LetStmt,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Option<RcCloneInfo> {
    let init = let_stmt.initializer()?;
    let pat = let_stmt.pat()?;

    // Must be a method call expression
    let method_call = match &init {
        ast::Expr::MethodCallExpr(mc) => mc,
        _ => return None,
    };

    // Method must be "clone"
    let method_name = method_call.name_ref()?;
    if method_name.text() != "clone" {
        return None;
    }

    // Get receiver type
    let receiver = method_call.receiver()?;
    let receiver_ty = sema.type_of_expr(&receiver)?.original;
    let receiver_type_str = receiver_ty.display(db, *display_target).to_string();

    // Check if receiver is Rc or Arc
    let clone_type = classify_rc_arc(&receiver_type_str, &receiver_ty, db)?;

    let clone_variable = pat.syntax().text().to_string().trim().to_string();
    let source_variable = receiver.syntax().text().to_string().trim().to_string();
    let (line, column) = line_index(let_stmt.syntax().text_range().start());

    Some(RcCloneInfo {
        clone_variable,
        source_variable,
        clone_type,
        line,
        column,
    })
}

/// Detect `let b = Rc::clone(&a)` or `Arc::clone(&a)`.
fn detect_explicit_clone(
    db: &RootDatabase,
    sema: &hir::Semantics<'_, RootDatabase>,
    display_target: &hir::DisplayTarget,
    let_stmt: &ast::LetStmt,
    line_index: &dyn Fn(ra_ap_syntax::TextSize) -> (u32, u32),
) -> Option<RcCloneInfo> {
    let init = let_stmt.initializer()?;
    let pat = let_stmt.pat()?;

    // Must be a call expression (not method call)
    let call_expr = match &init {
        ast::Expr::CallExpr(c) => c,
        _ => return None,
    };

    // Check if the call path contains "Rc::clone" or "Arc::clone"
    let callee = call_expr.expr()?;
    let callee_text = callee.syntax().text().to_string();

    let clone_type = if callee_text.contains("Rc::clone") {
        RcType::Rc
    } else if callee_text.contains("Arc::clone") {
        RcType::Arc
    } else {
        return None;
    };

    // Get the source variable from the argument
    let arg_list = call_expr.arg_list()?;
    let first_arg = arg_list.args().next()?;
    // Strip the & from &a
    let source_variable = first_arg.syntax().text().to_string().trim()
        .trim_start_matches('&').trim().to_string();

    let clone_variable = pat.syntax().text().to_string().trim().to_string();
    let (line, column) = line_index(let_stmt.syntax().text_range().start());

    Some(RcCloneInfo {
        clone_variable,
        source_variable,
        clone_type,
        line,
        column,
    })
}

/// Classify if a type is Rc or Arc based on its display string and ADT path.
fn classify_rc_arc(
    type_str: &str,
    ty: &hir::Type<'_>,
    db: &RootDatabase,
) -> Option<RcType> {
    // Check ADT canonical path
    if let Some(adt) = ty.as_adt() {
        let module = adt.module(db);
        let path: String = module
            .path_to_root(db)
            .iter()
            .rev()
            .filter_map(|m| m.name(db))
            .map(|n| n.display_no_db(Edition::Edition2021).to_string())
            .collect::<Vec<_>>()
            .join("::");
        let name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
        let full_path = format!("{}::{}", path, name).to_lowercase();

        if full_path.contains("rc::rc") || (name == "Rc" && path.contains("rc")) {
            return Some(RcType::Rc);
        }
        if full_path.contains("sync::arc") || (name == "Arc" && path.contains("sync")) {
            return Some(RcType::Arc);
        }
    }

    // Fallback: check display string
    if type_str.starts_with("Rc<") || type_str.contains("::Rc<") {
        return Some(RcType::Rc);
    }
    if type_str.starts_with("Arc<") || type_str.contains("::Arc<") {
        return Some(RcType::Arc);
    }

    None
}
