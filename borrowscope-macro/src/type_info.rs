//! Type information loading and lookup for borrowscope-macro
//!
//! This module loads type-info.json (v2.1) produced by borrowscope-analyzer
//! and provides lookup by variable name (stable Rust compatible).
//!
//! ## Limitation
//! On stable Rust, proc_macro::Span doesn't expose file/line/column info.
//! We use variable name lookup instead, which works when names are unique.
//! If multiple variables share the same name, we check if they have identical
//! type info - if so, we use it; otherwise we fall back to heuristics.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static TYPE_INFO_CACHE: OnceLock<Option<TypeInfoCache>> = OnceLock::new();

/// Load type info from a specific project root path.
/// Used by tests to load analyzer data from a test project.
/// Must be called before any `get_type_info()` / `lookup_*` calls.
/// Returns true if successfully loaded.
#[doc(hidden)]
pub fn load_from_path(project_root: &std::path::Path) -> bool {
    TYPE_INFO_CACHE
        .get_or_init(|| TypeInfoCache::load(project_root))
        .is_some()
}

/// Deserialized variable type info (v2.1 schema)
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct VariableTypeInfo {
    pub name: String,
    pub ty: String,

    // Trait implementations
    pub is_copy: bool,
    pub is_clone: bool,
    pub is_send: bool,
    pub is_sync: bool,
    pub is_drop: bool,
    pub is_sized: bool,
    pub is_future: bool,
    pub is_iterator: bool,

    // Type structure
    pub is_primitive: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_raw_ptr: bool,
    pub is_mutable_raw_ptr: bool,
    pub is_slice: bool,
    pub is_str: bool,
    pub is_closure: bool,
    pub is_fn_ptr: bool,
    pub is_dyn_trait: bool,
    pub is_union: bool,

    // ADT classification
    pub is_rc: bool,
    pub is_arc: bool,
    pub is_box: bool,
    pub is_weak: bool,
    pub is_refcell: bool,
    pub is_cell: bool,
    pub is_mutex: bool,
    pub is_rwlock: bool,
    pub is_guard: bool,
    pub is_vec: bool,
    pub is_string: bool,
    pub is_option: bool,
    pub is_result: bool,
    pub is_pin: bool,
    pub is_cow: bool,
    pub is_once_cell: bool,
    pub is_maybe_uninit: bool,
    pub is_channel: bool,
    pub is_extern_type: bool,
    pub is_never: bool,
    pub is_ordering: bool,

    // Declaration type
    pub is_static: bool,
    pub is_const: bool,

    // Binding patterns
    pub is_tuple_binding: bool,
    pub is_mut_binding: bool,
    pub is_impl_trait: bool,
    
    // Copy semantics (true if assignment is copy, not move)
    #[serde(default)]
    pub copy_semantics: bool,
    
    // Initializer pattern (semantic)
    pub initializer_kind: Option<String>,
    
    // Function context (for disambiguation)
    pub function_name: Option<String>,
    pub decl_index: Option<u32>,
    
    // Method calls on this variable (semantic operation tracking)
    #[serde(default)]
    pub method_calls: Vec<MethodCallInfo>,

    // Closure captures (semantic - from analyzer)
    #[serde(default)]
    pub closure_captures: Vec<ClosureCaptureInfo>,

    // === Group 1: Wire to existing runtime functions (5 fields) ===

    // JoinHandle detection (replaces var-name-based HashSet)
    #[serde(default)]
    pub is_join_handle: bool,

    // Callable detection (Fn/FnMut/FnOnce impl)
    #[serde(default)]
    pub is_callable: bool,

    // Drop point from analyzer (semantic via enclosing block)
    #[serde(default)]
    pub drop_line: Option<u32>,
    #[serde(default)]
    pub drop_column: Option<u32>,

    // Scope nesting identifier
    #[serde(default)]
    pub scope_id: Option<u32>,

    // Struct fields (semantic via Type::fields)
    #[serde(default)]
    pub fields: Vec<FieldInfo>,

    // === Group 2: New runtime functions + macro wiring (6 fields) ===

    // Atomic type detection
    #[serde(default)]
    pub is_atomic: bool,

    // Time type detection
    #[serde(default)]
    pub is_duration: bool,
    #[serde(default)]
    pub is_instant: bool,

    // Expression adjustments (autoref/autoderef chain)
    #[serde(default)]
    pub adjustments: Vec<AdjustmentInfo>,

    // Autoderef sequence
    #[serde(default)]
    pub deref_chain: Vec<String>,

    // Variable usage sites
    #[serde(default)]
    pub usages: Vec<VariableUsageInfo>,

    // === Group 3: Deserialize-only metadata (16 fields) ===

    // Source location
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u32,
    #[serde(default)]
    pub column: u32,
    #[serde(default)]
    pub span_start: u32,
    #[serde(default)]
    pub span_end: u32,

    // Memory layout
    #[serde(default)]
    pub layout: Option<LayoutInfo>,

    // Generic type arguments (e.g., ["String"] for Rc<String>)
    #[serde(default)]
    pub type_arguments: Vec<String>,

    // Lifetime annotation (e.g., "'static", "'a")
    #[serde(default)]
    pub lifetime: Option<String>,

    // Binding mode: "move", "ref", "ref_mut"
    #[serde(default)]
    pub binding_mode: Option<String>,

    // Reference analysis
    #[serde(default)]
    pub contains_reference: bool,
    #[serde(default)]
    pub reference_mutability: Option<String>,

    // Ref binding pattern
    #[serde(default)]
    pub is_ref_binding: bool,

    // Pattern match adjustments
    #[serde(default)]
    pub pattern_adjustments: Vec<String>,

    // Async/iterator output types
    #[serde(default)]
    pub future_output_type: Option<String>,
    #[serde(default)]
    pub iterator_item_type: Option<String>,
}

/// Memory layout information from analyzer
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LayoutInfo {
    pub size: u64,
    pub align: u64,
}

/// Struct field information from analyzer
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

/// Adjustment (coercion) information from analyzer
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AdjustmentInfo {
    pub kind: String,
    pub target: String,
}

/// Variable usage site from analyzer
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VariableUsageInfo {
    pub line: u32,
    pub column: u32,
    pub kind: String,
}

/// Closure capture information from analyzer
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ClosureCaptureInfo {
    pub name: String,
    pub capture_kind: String,
    #[serde(default)]
    pub ty: Option<String>,
    #[serde(default)]
    pub line: u32,
    #[serde(default)]
    pub column: u32,
}

/// Method call information (compact - only fields macro needs)
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MethodCallInfo {
    pub method: String,
    pub line: u32,
    pub column: u32,
    pub operation: Option<String>,
    pub self_borrow: Option<String>,
    #[serde(default)]
    pub is_trait_method: Option<bool>,
    #[serde(default)]
    pub trait_name: Option<String>,
    #[serde(default)]
    pub receiver_type: Option<String>,
    #[serde(default)]
    pub result_type: Option<String>,
    #[serde(default)]
    pub is_unsafe: Option<bool>,
}

/// Standalone expression information (compact - only fields macro needs)
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExpressionInfo {
    pub line: u32,
    pub column: u32,
    pub path: Option<String>,
    pub operation: String,
    #[serde(default)]
    pub result_type: Option<String>,
    #[serde(default)]
    pub argument: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwaitPointInfo {
    pub line: u32,
    pub column: u32,
    pub awaited_type: String,
    #[serde(default)]
    pub result_type: Option<String>,
    #[serde(default)]
    pub live_variables: Vec<String>,
    #[serde(default)]
    pub poll_function: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BorrowSpanInfo {
    pub variable: String,
    pub kind: String,
    pub start_line: u32,
    pub start_column: u32,
    #[serde(default)]
    pub end_line: Option<u32>,
    #[serde(default)]
    pub end_column: Option<u32>,
    #[serde(default)]
    pub use_sites: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UnsafeOperationInfo {
    pub line: u32,
    pub column: u32,
    pub kind: String,
    pub inside_unsafe_block: bool,
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ClosureTraitInfo {
    pub line: u32,
    pub column: u32,
    pub fn_trait: String,
    #[serde(default)]
    pub captures: Vec<ClosureCaptureInfo>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FieldAccessInfo {
    pub line: u32,
    pub column: u32,
    pub variable: String,
    pub field: String,
    pub field_type: String,
    pub access_kind: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DestructuringInfo {
    pub line: u32,
    pub column: u32,
    pub kind: String,
    #[serde(default)]
    pub source_expr: Option<String>,
    pub bindings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MatchBindingInfo {
    pub line: u32,
    pub column: u32,
    pub pattern: String,
    pub bindings: Vec<PatternBindingInfo>,
    pub context: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PatternBindingInfo {
    pub name: String,
    #[serde(default)]
    pub binding_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VariantInfo {
    pub line: u32,
    pub column: u32,
    pub enum_type: String,
    pub variant_name: String,
    pub variant_kind: String,
    #[serde(default)]
    pub field_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LifetimeInfoEntry {
    pub line: u32,
    pub column: u32,
    pub name: String,
    pub context: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LabelInfoEntry {
    pub line: u32,
    pub column: u32,
    pub name: String,
    pub loop_kind: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ConstPatternInfo {
    pub line: u32,
    pub column: u32,
    pub const_name: String,
    pub const_type: String,
    #[serde(default)]
    pub const_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CallableInfoEntry {
    pub line: u32,
    pub column: u32,
    pub kind: String,
    #[serde(default)]
    pub param_types: Vec<String>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub is_callable: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RecordFieldExprInfo {
    pub line: u32,
    pub column: u32,
    pub parent_type: String,
    pub field_name: String,
    pub field_type: String,
    #[serde(default)]
    pub value_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RecordFieldPatInfo {
    pub line: u32,
    pub column: u32,
    pub parent_type: String,
    pub field_name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MethodBorrowInfo {
    pub method_name: String,
    pub receiver_var: String,
    pub borrow_kind: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FunctionCallInfoEntry {
    pub function_name: String,
    pub return_category: String,
    pub is_copy_return: bool,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TraitImplInfo {
    pub type_name: String,
    #[serde(default)]
    pub implements_deref: bool,
    #[serde(default)]
    pub implements_deref_mut: bool,
    #[serde(default)]
    pub implements_index: bool,
    #[serde(default)]
    pub implements_index_mut: bool,
    #[serde(default)]
    pub implements_from: bool,
    #[serde(default)]
    pub implements_into: bool,
    #[serde(default)]
    pub implements_as_ref: bool,
    #[serde(default)]
    pub implements_as_mut: bool,
    #[serde(default)]
    pub implements_borrow: bool,
    #[serde(default)]
    pub implements_borrow_mut: bool,
    #[serde(default)]
    pub implements_to_owned: bool,
    #[serde(default)]
    pub implements_partial_eq: bool,
    #[serde(default)]
    pub implements_eq: bool,
    #[serde(default)]
    pub implements_partial_ord: bool,
    #[serde(default)]
    pub implements_ord: bool,
    #[serde(default)]
    pub implements_add: bool,
    #[serde(default)]
    pub implements_sub: bool,
    #[serde(default)]
    pub implements_mul: bool,
    #[serde(default)]
    pub implements_div: bool,
    #[serde(default)]
    pub implements_rem: bool,
    #[serde(default)]
    pub implements_neg: bool,
    #[serde(default)]
    pub implements_add_assign: bool,
    #[serde(default)]
    pub implements_sub_assign: bool,
    #[serde(default)]
    pub implements_mul_assign: bool,
    #[serde(default)]
    pub implements_div_assign: bool,
    #[serde(default)]
    pub implements_rem_assign: bool,
    #[serde(default)]
    pub implements_bit_and: bool,
    #[serde(default)]
    pub implements_bit_or: bool,
    #[serde(default)]
    pub implements_bit_xor: bool,
    #[serde(default)]
    pub implements_shl: bool,
    #[serde(default)]
    pub implements_shr: bool,
    #[serde(default)]
    pub implements_not: bool,
    #[serde(default)]
    pub implements_bit_and_assign: bool,
    #[serde(default)]
    pub implements_bit_or_assign: bool,
    #[serde(default)]
    pub implements_bit_xor_assign: bool,
    #[serde(default)]
    pub implements_shl_assign: bool,
    #[serde(default)]
    pub implements_shr_assign: bool,
    #[serde(default)]
    pub implements_range_bounds: bool,
    #[serde(default)]
    pub implements_termination: bool,
    #[serde(default)]
    pub implements_unwind_safe: bool,
    #[serde(default)]
    pub implements_ref_unwind_safe: bool,
}

#[derive(Debug, Deserialize)]
struct ProjectTypeInfo {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    files: HashMap<String, Vec<VariableTypeInfo>>,
    #[serde(default)]
    by_name: HashMap<String, Vec<VariableTypeInfo>>,
    #[serde(default)]
    by_function: HashMap<String, HashMap<String, Vec<VariableTypeInfo>>>,
    #[serde(default)]
    expressions: HashMap<String, Vec<ExpressionInfo>>,
    #[serde(default)]
    await_points: HashMap<String, Vec<AwaitPointInfo>>,
    #[serde(default)]
    borrow_spans: HashMap<String, Vec<BorrowSpanInfo>>,
    #[serde(default)]
    unsafe_operations: HashMap<String, Vec<UnsafeOperationInfo>>,
    #[serde(default)]
    closure_traits: HashMap<String, Vec<ClosureTraitInfo>>,
    #[serde(default)]
    field_accesses: HashMap<String, Vec<FieldAccessInfo>>,
    #[serde(default)]
    destructuring: HashMap<String, Vec<DestructuringInfo>>,
    #[serde(default)]
    match_bindings: HashMap<String, Vec<MatchBindingInfo>>,
    #[serde(default)]
    variants: HashMap<String, Vec<VariantInfo>>,
    #[serde(default)]
    lifetimes: HashMap<String, Vec<LifetimeInfoEntry>>,
    #[serde(default)]
    labels: HashMap<String, Vec<LabelInfoEntry>>,
    #[serde(default)]
    const_patterns: HashMap<String, Vec<ConstPatternInfo>>,
    #[serde(default)]
    callables: HashMap<String, Vec<CallableInfoEntry>>,
    #[serde(default)]
    record_field_exprs: HashMap<String, Vec<RecordFieldExprInfo>>,
    #[serde(default)]
    record_field_pats: HashMap<String, Vec<RecordFieldPatInfo>>,
    #[serde(default)]
    method_borrows: HashMap<String, Vec<MethodBorrowInfo>>,
    #[serde(default)]
    function_calls: HashMap<String, Vec<FunctionCallInfoEntry>>,
    #[serde(default)]
    trait_impls: HashMap<String, TraitImplInfo>,
}

pub struct TypeInfoCache {
    by_name: HashMap<String, Vec<VariableTypeInfo>>,
    by_function: HashMap<String, HashMap<String, Vec<VariableTypeInfo>>>,
    expressions: HashMap<String, Vec<ExpressionInfo>>,
    await_points: HashMap<String, Vec<AwaitPointInfo>>,
    borrow_spans: HashMap<String, Vec<BorrowSpanInfo>>,
    unsafe_operations: HashMap<String, Vec<UnsafeOperationInfo>>,
    closure_traits: HashMap<String, Vec<ClosureTraitInfo>>,
    field_accesses: HashMap<String, Vec<FieldAccessInfo>>,
    destructuring: HashMap<String, Vec<DestructuringInfo>>,
    match_bindings: HashMap<String, Vec<MatchBindingInfo>>,
    variants: HashMap<String, Vec<VariantInfo>>,
    lifetimes: HashMap<String, Vec<LifetimeInfoEntry>>,
    labels: HashMap<String, Vec<LabelInfoEntry>>,
    const_patterns: HashMap<String, Vec<ConstPatternInfo>>,
    callables: HashMap<String, Vec<CallableInfoEntry>>,
    record_field_exprs: HashMap<String, Vec<RecordFieldExprInfo>>,
    record_field_pats: HashMap<String, Vec<RecordFieldPatInfo>>,
    method_borrows: HashMap<String, Vec<MethodBorrowInfo>>,
    function_calls: HashMap<String, Vec<FunctionCallInfoEntry>>,
    trait_impls: HashMap<String, TraitImplInfo>,
}

impl TypeInfoCache {
    fn load(project_root: &Path) -> Option<Self> {
        let json_path = project_root.join(".borrowscope/type-info.json");
        if !json_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&json_path).ok()?;
        let info: ProjectTypeInfo = serde_json::from_str(&content).ok()?;

        Some(Self { 
            by_name: info.by_name,
            by_function: info.by_function,
            expressions: info.expressions,
            await_points: info.await_points,
            borrow_spans: info.borrow_spans,
            unsafe_operations: info.unsafe_operations,
            closure_traits: info.closure_traits,
            field_accesses: info.field_accesses,
            destructuring: info.destructuring,
            match_bindings: info.match_bindings,
            variants: info.variants,
            lifetimes: info.lifetimes,
            labels: info.labels,
            const_patterns: info.const_patterns,
            callables: info.callables,
            record_field_exprs: info.record_field_exprs,
            record_field_pats: info.record_field_pats,
            method_borrows: info.method_borrows,
            function_calls: info.function_calls,
            trait_impls: info.trait_impls,
        })
    }

    /// Lookup by function name and variable name (preferred)
    pub fn lookup_in_function(&self, fn_name: &str, var_name: &str, decl_index: Option<u32>) -> Option<&VariableTypeInfo> {
        let fn_vars = self.by_function.get(fn_name)?;
        let entries = fn_vars.get(var_name)?;
        
        if entries.len() == 1 {
            return Some(&entries[0]);
        }
        
        // Multiple entries - try to match by decl_index
        if let Some(idx) = decl_index {
            if let Some(entry) = entries.iter().find(|e| e.decl_index == Some(idx)) {
                return Some(entry);
            }
        }
        
        // Fall back to same_classification check
        let first = entries.first()?;
        if entries.iter().skip(1).all(|e| Self::same_classification(first, e)) {
            Some(first)
        } else {
            None
        }
    }

    /// Lookup by variable name only (fallback)
    pub fn lookup(&self, var_name: &str) -> Option<&VariableTypeInfo> {
        let entries = self.by_name.get(var_name)?;
        if entries.is_empty() {
            return None;
        }
        if entries.len() == 1 {
            return Some(&entries[0]);
        }
        // Multiple entries - check if all have same type classification
        let first = &entries[0];
        if entries.iter().skip(1).all(|e| Self::same_classification(first, e)) {
            Some(first)
        } else {
            None // Ambiguous
        }
    }

    fn same_classification(a: &VariableTypeInfo, b: &VariableTypeInfo) -> bool {
        a.is_rc == b.is_rc && a.is_arc == b.is_arc && a.is_box == b.is_box
            && a.is_refcell == b.is_refcell && a.is_cell == b.is_cell
            && a.is_reference == b.is_reference && a.is_mutable_reference == b.is_mutable_reference
            && a.is_raw_ptr == b.is_raw_ptr && a.is_mutex == b.is_mutex
            && a.is_rwlock == b.is_rwlock && a.is_guard == b.is_guard
            && a.initializer_kind == b.initializer_kind
    }
}

fn find_project_root() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return Some(PathBuf::from(dir));
    }
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn get_type_info() -> Option<&'static TypeInfoCache> {
    TYPE_INFO_CACHE
        .get_or_init(|| find_project_root().and_then(|root| TypeInfoCache::load(&root)))
        .as_ref()
}

/// Lookup type info by function name and variable name (preferred)
pub fn lookup_in_function(fn_name: &str, var_name: &str, decl_index: Option<u32>) -> Option<&'static VariableTypeInfo> {
    get_type_info()?.lookup_in_function(fn_name, var_name, decl_index)
}

/// Lookup type info by variable name only (fallback)
pub fn lookup_by_name(var_name: &str) -> Option<&'static VariableTypeInfo> {
    get_type_info()?.lookup(var_name)
}

pub fn is_ffi(name: &str) -> bool {
    lookup_by_name(name).map_or(false, |v| v.is_extern_type)
}

pub fn is_static(name: &str) -> bool {
    lookup_by_name(name).map_or(false, |v| v.is_static)
}

pub fn is_union(name: &str) -> bool {
    lookup_by_name(name).map_or(false, |v| v.is_union)
}

/// Lookup method call info for a variable at a specific location
pub fn lookup_method_call(var_name: &str, line: u32, column: u32) -> Option<&'static MethodCallInfo> {
    let type_info = get_type_info()?;
    let var_info = type_info.lookup(var_name)?;
    var_info.method_calls.iter().find(|mc| mc.line == line && mc.column == column)
}

/// Lookup expression info at a specific location
pub fn lookup_expression(file: &str, line: u32, column: u32) -> Option<&'static ExpressionInfo> {
    let type_info = get_type_info()?;
    let exprs = type_info.expressions.get(file)?;
    exprs.iter().find(|e| e.line == line && e.column == column)
}

/// Check if a transmute call exists in analyzer expression data.
/// Returns Some(true) if transmute found, Some(false) if analyzer data exists but no transmute,
/// None if no analyzer data available (caller should fall back to heuristic).
pub fn has_transmute_expression() -> Option<bool> {
    let type_info = get_type_info()?;
    if type_info.expressions.is_empty() {
        return None; // No expression data — can't determine
    }
    Some(type_info.expressions.values()
        .flat_map(|v| v.iter())
        .any(|e| e.operation == "core::intrinsics::transmute"))
}

/// Find a transmute expression's type info from semantic data by line number.
/// Returns (argument_type, result_type) if a transmute at the given line is found.
pub fn find_transmute_types(line: u32) -> Option<(&'static str, &'static str)> {
    let type_info = get_type_info()?;
    for exprs in type_info.expressions.values() {
        for e in exprs {
            if e.operation == "core::intrinsics::transmute" && e.line == line {
                let from = e.argument.as_deref().unwrap_or("unknown");
                let to = e.result_type.as_deref().unwrap_or("unknown");
                return Some((from, to));
            }
        }
    }
    None
}

/// Lookup await point at a specific line
pub fn lookup_await_point(line: u32) -> Option<&'static AwaitPointInfo> {
    let ti = get_type_info()?;
    ti.await_points.values().flat_map(|v| v.iter()).find(|a| a.line == line)
}

/// Lookup borrow spans for a variable
pub fn lookup_borrow_spans(variable: &str) -> Vec<&'static BorrowSpanInfo> {
    get_type_info().map_or(vec![], |ti| {
        ti.borrow_spans.values().flat_map(|v| v.iter()).filter(|b| b.variable == variable).collect()
    })
}

/// Lookup unsafe operation at a specific line
pub fn lookup_unsafe_operation(line: u32) -> Option<&'static UnsafeOperationInfo> {
    let ti = get_type_info()?;
    ti.unsafe_operations.values().flat_map(|v| v.iter()).find(|u| u.line == line)
}

/// Lookup closure trait info at a specific line
pub fn lookup_closure_trait(line: u32) -> Option<&'static ClosureTraitInfo> {
    let ti = get_type_info()?;
    ti.closure_traits.values().flat_map(|v| v.iter()).find(|c| c.line == line)
}

/// Lookup field access at a specific line/column
pub fn lookup_field_access(line: u32, column: u32) -> Option<&'static FieldAccessInfo> {
    let ti = get_type_info()?;
    ti.field_accesses.values().flat_map(|v| v.iter()).find(|f| f.line == line && f.column == column)
}

/// Lookup destructuring at a specific line
pub fn lookup_destructuring(line: u32) -> Option<&'static DestructuringInfo> {
    let ti = get_type_info()?;
    ti.destructuring.values().flat_map(|v| v.iter()).find(|d| d.line == line)
}

/// Lookup match binding at a specific line
pub fn lookup_match_binding(line: u32) -> Option<&'static MatchBindingInfo> {
    let ti = get_type_info()?;
    ti.match_bindings.values().flat_map(|v| v.iter()).find(|m| m.line == line)
}

/// Lookup variant construction at a specific line
pub fn lookup_variant(line: u32, column: u32) -> Option<&'static VariantInfo> {
    let ti = get_type_info()?;
    ti.variants.values().flat_map(|v| v.iter()).find(|v| v.line == line && v.column == column)
}

/// Lookup lifetime at a specific line
#[allow(dead_code)]
pub fn lookup_lifetime(line: u32) -> Option<&'static LifetimeInfoEntry> {
    let ti = get_type_info()?;
    ti.lifetimes.values().flat_map(|v| v.iter()).find(|l| l.line == line)
}

/// Lookup label at a specific line
pub fn lookup_label(line: u32) -> Option<&'static LabelInfoEntry> {
    let ti = get_type_info()?;
    ti.labels.values().flat_map(|v| v.iter()).find(|l| l.line == line)
}

/// Lookup const pattern at a specific line
#[allow(dead_code)]
pub fn lookup_const_pattern(line: u32) -> Option<&'static ConstPatternInfo> {
    let ti = get_type_info()?;
    ti.const_patterns.values().flat_map(|v| v.iter()).find(|c| c.line == line)
}

/// Lookup callable at a specific line
pub fn lookup_callable(line: u32, column: u32) -> Option<&'static CallableInfoEntry> {
    let ti = get_type_info()?;
    ti.callables.values().flat_map(|v| v.iter()).find(|c| c.line == line && c.column == column)
}

/// Lookup record field expression at a specific line/column
pub fn lookup_record_field_expr(line: u32, column: u32) -> Option<&'static RecordFieldExprInfo> {
    let ti = get_type_info()?;
    ti.record_field_exprs.values().flat_map(|v| v.iter()).find(|r| r.line == line && r.column == column)
}

/// Lookup record field pattern at a specific line/column
#[allow(dead_code)]
pub fn lookup_record_field_pat(line: u32, column: u32) -> Option<&'static RecordFieldPatInfo> {
    let ti = get_type_info()?;
    ti.record_field_pats.values().flat_map(|v| v.iter()).find(|r| r.line == line && r.column == column)
}

/// Lookup method borrow at a specific line
pub fn lookup_method_borrow(line: u32, column: u32) -> Option<&'static MethodBorrowInfo> {
    let ti = get_type_info()?;
    ti.method_borrows.values().flat_map(|v| v.iter()).find(|m| m.line == line && m.column == column)
}

/// Lookup function call at a specific line
pub fn lookup_function_call(line: u32, column: u32) -> Option<&'static FunctionCallInfoEntry> {
    let ti = get_type_info()?;
    ti.function_calls.values().flat_map(|v| v.iter()).find(|f| f.line == line && f.column == column)
}

/// Lookup trait impl info for a type
#[allow(dead_code)]
pub fn lookup_trait_impl(type_name: &str) -> Option<&'static TraitImplInfo> {
    let ti = get_type_info()?;
    ti.trait_impls.get(type_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group3_deserialization() {
        let json = r#"{
            "name": "x", "ty": "Rc<String>",
            "is_copy": false, "is_clone": true, "is_send": false, "is_sync": false,
            "is_drop": true, "is_sized": true, "is_future": false, "is_iterator": false,
            "is_primitive": false, "is_reference": false, "is_mutable_reference": false,
            "is_raw_ptr": false, "is_mutable_raw_ptr": false, "is_slice": false,
            "is_str": false, "is_closure": false, "is_fn_ptr": false, "is_dyn_trait": false,
            "is_union": false, "is_rc": true, "is_arc": false, "is_box": false,
            "is_weak": false, "is_refcell": false, "is_cell": false, "is_mutex": false,
            "is_rwlock": false, "is_guard": false, "is_vec": false, "is_string": false,
            "is_option": false, "is_result": false, "is_pin": false, "is_cow": false,
            "is_once_cell": false, "is_maybe_uninit": false, "is_channel": false,
            "is_extern_type": false, "is_never": false, "is_ordering": false,
            "is_static": false, "is_const": false, "is_tuple_binding": false,
            "is_mut_binding": false, "is_impl_trait": false, "copy_semantics": false,
            "initializer_kind": "rc_new",
            "function_name": "test_fn", "decl_index": 0,
            "file": "src/main.rs", "line": 10, "column": 8,
            "span_start": 100, "span_end": 120,
            "layout": { "size": 8, "align": 8 },
            "type_arguments": ["String"],
            "lifetime": null,
            "binding_mode": "move",
            "contains_reference": false,
            "reference_mutability": null,
            "is_ref_binding": false,
            "pattern_adjustments": [],
            "future_output_type": null,
            "iterator_item_type": null
        }"#;
        let v: VariableTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(v.file, "src/main.rs");
        assert_eq!(v.line, 10);
        assert_eq!(v.column, 8);
        assert_eq!(v.span_start, 100);
        assert_eq!(v.span_end, 120);
        assert_eq!(v.layout.as_ref().unwrap().size, 8);
        assert_eq!(v.layout.as_ref().unwrap().align, 8);
        assert_eq!(v.type_arguments, vec!["String"]);
        assert_eq!(v.binding_mode.as_deref(), Some("move"));
        assert!(!v.contains_reference);
        assert!(v.reference_mutability.is_none());
        assert!(!v.is_ref_binding);
        assert!(v.pattern_adjustments.is_empty());
        assert!(v.future_output_type.is_none());
        assert!(v.iterator_item_type.is_none());
    }

    #[test]
    fn test_group3_defaults() {
        // Minimal JSON — all Group 3 fields should default
        let json = r#"{
            "name": "y", "ty": "i32",
            "is_copy": true, "is_clone": true, "is_send": true, "is_sync": true,
            "is_drop": false, "is_sized": true, "is_future": false, "is_iterator": false,
            "is_primitive": true, "is_reference": false, "is_mutable_reference": false,
            "is_raw_ptr": false, "is_mutable_raw_ptr": false, "is_slice": false,
            "is_str": false, "is_closure": false, "is_fn_ptr": false, "is_dyn_trait": false,
            "is_union": false, "is_rc": false, "is_arc": false, "is_box": false,
            "is_weak": false, "is_refcell": false, "is_cell": false, "is_mutex": false,
            "is_rwlock": false, "is_guard": false, "is_vec": false, "is_string": false,
            "is_option": false, "is_result": false, "is_pin": false, "is_cow": false,
            "is_once_cell": false, "is_maybe_uninit": false, "is_channel": false,
            "is_extern_type": false, "is_never": false, "is_ordering": false,
            "is_static": false, "is_const": false, "is_tuple_binding": false,
            "is_mut_binding": false, "is_impl_trait": false,
            "initializer_kind": null,
            "function_name": null, "decl_index": null
        }"#;
        let v: VariableTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(v.file, "");
        assert_eq!(v.line, 0);
        assert_eq!(v.column, 0);
        assert_eq!(v.span_start, 0);
        assert_eq!(v.span_end, 0);
        assert!(v.layout.is_none());
        assert!(v.type_arguments.is_empty());
        assert!(v.lifetime.is_none());
        assert!(v.binding_mode.is_none());
        assert!(!v.contains_reference);
        assert!(v.reference_mutability.is_none());
        assert!(!v.is_ref_binding);
        assert!(v.pattern_adjustments.is_empty());
        assert!(v.future_output_type.is_none());
        assert!(v.iterator_item_type.is_none());
    }

    #[test]
    fn test_group1_deserialization() {
        let json = r#"{
            "name": "handle", "ty": "JoinHandle<()>",
            "is_copy": false, "is_clone": false, "is_send": true, "is_sync": true,
            "is_drop": true, "is_sized": true, "is_future": false, "is_iterator": false,
            "is_primitive": false, "is_reference": false, "is_mutable_reference": false,
            "is_raw_ptr": false, "is_mutable_raw_ptr": false, "is_slice": false,
            "is_str": false, "is_closure": false, "is_fn_ptr": false, "is_dyn_trait": false,
            "is_union": false, "is_rc": false, "is_arc": false, "is_box": false,
            "is_weak": false, "is_refcell": false, "is_cell": false, "is_mutex": false,
            "is_rwlock": false, "is_guard": false, "is_vec": false, "is_string": false,
            "is_option": false, "is_result": false, "is_pin": false, "is_cow": false,
            "is_once_cell": false, "is_maybe_uninit": false, "is_channel": false,
            "is_extern_type": false, "is_never": false, "is_ordering": false,
            "is_static": false, "is_const": false, "is_tuple_binding": false,
            "is_mut_binding": false, "is_impl_trait": false,
            "initializer_kind": null,
            "function_name": "test_fn", "decl_index": 0,
            "is_join_handle": true,
            "is_callable": false,
            "drop_line": 42, "drop_column": 1,
            "scope_id": 5,
            "fields": [{"name": "inner", "ty": "Thread"}]
        }"#;
        let v: VariableTypeInfo = serde_json::from_str(json).unwrap();
        assert!(v.is_join_handle);
        assert!(!v.is_callable);
        assert_eq!(v.drop_line, Some(42));
        assert_eq!(v.drop_column, Some(1));
        assert_eq!(v.scope_id, Some(5));
        assert_eq!(v.fields.len(), 1);
        assert_eq!(v.fields[0].name, "inner");
        assert_eq!(v.fields[0].ty, "Thread");
    }

    #[test]
    fn test_group2_deserialization() {
        let json = r#"{
            "name": "flag", "ty": "AtomicBool",
            "is_copy": false, "is_clone": false, "is_send": true, "is_sync": true,
            "is_drop": false, "is_sized": true, "is_future": false, "is_iterator": false,
            "is_primitive": false, "is_reference": false, "is_mutable_reference": false,
            "is_raw_ptr": false, "is_mutable_raw_ptr": false, "is_slice": false,
            "is_str": false, "is_closure": false, "is_fn_ptr": false, "is_dyn_trait": false,
            "is_union": false, "is_rc": false, "is_arc": false, "is_box": false,
            "is_weak": false, "is_refcell": false, "is_cell": false, "is_mutex": false,
            "is_rwlock": false, "is_guard": false, "is_vec": false, "is_string": false,
            "is_option": false, "is_result": false, "is_pin": false, "is_cow": false,
            "is_once_cell": false, "is_maybe_uninit": false, "is_channel": false,
            "is_extern_type": false, "is_never": false, "is_ordering": false,
            "is_static": false, "is_const": false, "is_tuple_binding": false,
            "is_mut_binding": false, "is_impl_trait": false,
            "initializer_kind": "atomic_new",
            "function_name": "test_fn", "decl_index": 0,
            "is_atomic": true,
            "is_duration": false,
            "is_instant": false,
            "adjustments": [{"kind": "deref", "target": "bool"}],
            "deref_chain": ["bool"],
            "usages": [{"line": 10, "column": 4, "kind": "read"}]
        }"#;
        let v: VariableTypeInfo = serde_json::from_str(json).unwrap();
        assert!(v.is_atomic);
        assert!(!v.is_duration);
        assert!(!v.is_instant);
        assert_eq!(v.adjustments.len(), 1);
        assert_eq!(v.adjustments[0].kind, "deref");
        assert_eq!(v.adjustments[0].target, "bool");
        assert_eq!(v.deref_chain, vec!["bool"]);
        assert_eq!(v.usages.len(), 1);
        assert_eq!(v.usages[0].kind, "read");
        assert_eq!(v.usages[0].line, 10);
    }

    #[test]
    fn test_method_call_info_new_fields() {
        let json = r#"{
            "method": "lock", "line": 5, "column": 8,
            "operation": "std::sync::poison::mutex::lock",
            "self_borrow": "mutable",
            "receiver_type": "Mutex<i32>",
            "result_type": "Result<MutexGuard<'_, i32>, PoisonError<MutexGuard<'_, i32>>>",
            "is_unsafe": false
        }"#;
        let mc: MethodCallInfo = serde_json::from_str(json).unwrap();
        assert_eq!(mc.receiver_type.as_deref(), Some("Mutex<i32>"));
        assert!(mc.result_type.as_deref().unwrap().contains("MutexGuard"));
        assert_eq!(mc.is_unsafe, Some(false));
    }
}
