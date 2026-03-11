//! Type information output structures
//!
//! These structures are serialized to JSON and read by the proc macro.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a method call on a tracked variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCallInfo {
    /// Method name (e.g., "set", "send", "join")
    pub method: String,
    /// Line number where the call occurs
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Semantic operation classification (e.g., "cell_set", "channel_send")
    /// None if the method doesn't map to a tracked operation
    pub operation: Option<String>,
    /// How the method borrows self: "immutable", "mutable", or "consuming"
    pub self_borrow: Option<String>,
    /// Fully qualified receiver type
    pub receiver_type: String,
    /// Fully qualified result type (if any)
    pub result_type: Option<String>,
    /// Whether this is a trait method (vs inherent method)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_trait_method: Option<bool>,
    /// Trait name if this is a trait method (e.g., "Clone", "Iterator")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trait_name: Option<String>,
    /// Whether this method call is unsafe
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,
}

/// Information about a variable usage (read or write)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableUsageInfo {
    /// Line number of the usage
    pub line: u32,
    /// Column number of the usage
    pub column: u32,
    /// Kind of usage: "read", "write", "read_write"
    pub kind: String,
}

/// Information about a closure capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureCaptureInfo {
    /// Name of the captured variable
    pub name: String,
    /// Capture mode: "shared_ref", "unique_shared_ref", "mutable_ref", "move"
    pub capture_kind: String,
    /// Type of the captured variable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
}

/// Information about an await point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwaitPointInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Type of the awaited expression (the Future type)
    pub awaited_type: String,
    /// Result type after awaiting (semantic via Type::future_output)
    pub result_type: Option<String>,
    /// Variables that are live across this await point
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub live_variables: Vec<String>,
    /// Resolved Poll function (semantic via sema.resolve_await_to_poll)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_function: Option<String>,
}

/// Information about an enum variant construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Enum type name
    pub enum_type: String,
    /// Variant name
    pub variant_name: String,
    /// Variant kind: "unit", "tuple", "struct"
    pub variant_kind: String,
    /// Field types (for tuple/struct variants)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_types: Vec<String>,
}

/// Information about a lifetime parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Lifetime name (e.g., "'a", "'static")
    pub name: String,
    /// Context: "function", "struct", "impl", "trait"
    pub context: String,
}

/// Information about a loop label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Label name (e.g., "'outer")
    pub name: String,
    /// Loop kind: "loop", "while", "for"
    pub loop_kind: String,
}

/// Information about a const pattern binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstPatternInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Const name
    pub const_name: String,
    /// Const type
    pub const_type: String,
    /// Const value (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub const_value: Option<String>,
}

/// Memory layout information for a type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    /// Size in bytes
    pub size: u64,
    /// Alignment in bytes
    pub align: u64,
}

/// Information about a callable (function pointer, closure, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallableInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Callable kind: "fn_ptr", "closure", "fn_def", "fn_trait"
    pub kind: String,
    /// Parameter types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub param_types: Vec<String>,
    /// Return type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    /// Whether it implements FnOnce (semantic via Type::impls_fnonce)
    #[serde(default)]
    pub is_callable: bool,
}

/// Information about a record field access in expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordFieldExprInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Struct/enum type
    pub parent_type: String,
    /// Field name
    pub field_name: String,
    /// Field type
    pub field_type: String,
    /// Expression being assigned (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
}

/// Information about a record field in pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordFieldPatInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Struct/enum type
    pub parent_type: String,
    /// Field name
    pub field_name: String,
    /// Field type
    pub field_type: String,
}

/// Information about a borrow span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowSpanInfo {
    /// Variable being borrowed
    pub variable: String,
    /// Borrow kind: "shared", "mutable"
    pub kind: String,
    /// Line where borrow starts
    pub start_line: u32,
    /// Column where borrow starts
    pub start_column: u32,
    /// Line where borrow ends (last use)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// Column where borrow ends
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
    /// Lines where borrow is used
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub use_sites: Vec<(u32, u32)>,
}

/// Information about a destructuring binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructuringInfo {
    /// Line number of the destructuring pattern
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Kind of destructuring: "tuple", "struct", "slice", "tuple_struct"
    pub kind: String,
    /// Source expression being destructured (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_expr: Option<String>,
    /// Bindings created by this destructuring
    pub bindings: Vec<String>,
}

/// Information about a match arm binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchBindingInfo {
    /// Line number of the match arm
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Pattern text
    pub pattern: String,
    /// Bindings created in this arm
    pub bindings: Vec<PatternBindingInfo>,
    /// Whether this is from match, if-let, or while-let
    pub context: String,
}

/// Information about a single binding in a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBindingInfo {
    /// Binding name
    pub name: String,
    /// Binding mode: "move", "ref", "ref_mut"
    pub mode: String,
    /// Type of the binding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
}

/// Information about an unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeOperationInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Kind of unsafe operation: "deref_raw_ptr", "call_unsafe_fn", "access_mutable_static", "call_unsafe_method", "ffi_call", "unsafe_ident_pat"
    pub kind: String,
    /// Whether this operation is inside an unsafe block
    pub inside_unsafe_block: bool,
    /// Additional context (e.g., function name for calls, static name for access)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Information about a field access (for partial borrow tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Variable being accessed
    pub variable: String,
    /// Field name or tuple index
    pub field: String,
    /// Field type
    pub field_type: String,
    /// Access kind: "read", "write", "borrow_shared", "borrow_mut"
    pub access_kind: String,
}

/// Information about a closure's Fn trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureTraitInfo {
    /// Line number where closure is defined
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Fn trait: "Fn", "FnMut", "FnOnce"
    pub fn_trait: String,
    /// Captures with their modes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub captures: Vec<ClosureCaptureInfo>,
}

/// Information about a standalone expression (not a method call on a variable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionInfo {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Expression kind: "function_call", "macro_call"
    pub kind: String,
    /// Function/macro path (e.g., "std::thread::spawn", "std::mem::drop")
    pub path: Option<String>,
    /// Semantic operation classification
    pub operation: String,
    /// Argument variable name (if applicable, e.g., for drop(x))
    pub argument: Option<String>,
    /// Result type (if applicable)
    pub result_type: Option<String>,
    /// Whether this function call is unsafe
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_unsafe: Option<bool>,
    /// Closure captures with their capture modes (for spawn, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closure_captures: Vec<ClosureCaptureInfo>,
}

/// Information about a field in a struct/enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: String,
}

/// Information about an adjustment (coercion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentInfo {
    /// Adjustment kind: "deref", "borrow_shared", "borrow_mut", "deref_mut", "unsize", "pointer_cast"
    pub kind: String,
    /// Target type after adjustment
    pub target: String,
}

/// Type information for a single variable binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTypeInfo {
    /// Variable name as it appears in source
    pub name: String,
    /// Fully resolved type string (e.g., "Rc<String>", "Vec<i32>")
    pub ty: String,
    
    // Core trait implementations (semantic via impls_trait)
    pub is_copy: bool,
    pub is_clone: bool,
    pub is_drop: bool,
    pub is_send: bool,  // Cannot be detected - Send is not a lang item
    pub is_sync: bool,
    pub is_sized: bool,

    // Primitive types (semantic via as_builtin)
    pub is_primitive: bool,

    // Smart pointers (semantic via ADT canonical path)
    pub is_rc: bool,
    pub is_arc: bool,
    pub is_box: bool,
    pub is_weak: bool,

    // Interior mutability (semantic via ADT canonical path)
    pub is_refcell: bool,
    pub is_cell: bool,
    pub is_mutex: bool,
    pub is_rwlock: bool,

    // Guards (semantic via ADT canonical path)
    pub is_guard: bool,

    // Collections (semantic via ADT canonical path)
    pub is_vec: bool,
    pub is_string: bool,

    // References and pointers (semantic via Type methods)
    pub is_raw_ptr: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_slice: bool,
    pub is_str: bool,

    // Wrapper types (semantic via ADT canonical path)
    pub is_pin: bool,
    pub is_cow: bool,
    pub is_option: bool,
    pub is_result: bool,
    pub is_once_cell: bool,
    pub is_maybe_uninit: bool,
    pub is_channel: bool,

    // Atomics (semantic via ADT)
    pub is_atomic: bool,
    
    // Comparison (semantic via ADT)
    pub is_ordering: bool,
    
    // Threading (semantic via ADT)
    pub is_join_handle: bool,
    
    // Time (semantic via ADT)
    pub is_duration: bool,
    pub is_instant: bool,

    // Callable/async types (semantic via Type methods and trait impl)
    pub is_closure: bool,
    pub is_fn_ptr: bool,
    pub is_future: bool,
    pub is_iterator: bool,
    
    // Callable check (semantic via Type::impls_fnonce)
    #[serde(default)]
    pub is_callable: bool,
    
    // Future output type (semantic via Type::future_output)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub future_output_type: Option<String>,
    
    // Iterator item type (semantic via Type::iterator_item)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterator_item_type: Option<String>,
    
    // Memory layout (semantic via Adt::layout)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutInfo>,
    
    // Trait objects (semantic via as_dyn_trait)
    pub is_dyn_trait: bool,

    // FFI and unsafe types (semantic via ADT)
    pub is_union: bool,
    pub is_extern_type: bool,
    pub is_never: bool,

    // Static/const binding (semantic via syntax kind)
    pub is_static: bool,
    pub is_const: bool,

    // Binding patterns (semantic via AST pattern analysis)
    pub is_tuple_binding: bool,
    pub is_mut_binding: bool,
    pub is_impl_trait: bool,

    // Explicit lifetime annotation from source (e.g., "'static", "'a")
    // Extracted from type annotation AST, not from inferred type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<String>,

    // Binding mode (semantic via sema.binding_mode_of_pat)
    // Values: "move", "ref", "ref_mut"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_mode: Option<String>,
    
    // Reference analysis (semantic via Type methods)
    #[serde(default)]
    pub contains_reference: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_mutability: Option<String>, // "shared" or "mutable" if is_reference
    
    // Binding flags (semantic via Local methods)
    #[serde(default)]
    pub is_ref_binding: bool, // ref or ref mut pattern
    
    // Deref chain (semantic via Type::autoderef)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deref_chain: Vec<String>,
    
    // Struct fields (semantic via Type::fields)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    
    // Expression adjustments (semantic via sema.expr_adjustments)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustments: Vec<AdjustmentInfo>,
    
    // Pattern adjustments (semantic via sema.pattern_adjustments)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pattern_adjustments: Vec<String>,

    // Generic type arguments (e.g., ["String", "i32"] for HashMap<String, i32>)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_arguments: Vec<String>,

    // Initializer kind for tracking strategy
    pub initializer_kind: Option<String>,

    /// Closure captures with their capture modes (for closure variables)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closure_captures: Vec<ClosureCaptureInfo>,

    /// Method calls made on this variable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_calls: Vec<MethodCallInfo>,

    /// Variable usages (reads and writes)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub usages: Vec<VariableUsageInfo>,

    /// Source location
    pub file: String,
    pub line: u32,
    pub column: u32,
    
    // Byte offsets for precise matching
    pub span_start: u32,
    pub span_end: u32,
    
    // Drop point - where the variable goes out of scope (semantic via enclosing block)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_column: Option<u32>,
    
    // Scope tracking
    pub scope_id: Option<u32>,
    
    /// Containing function name (for disambiguation)
    pub function_name: Option<String>,
    
    /// Declaration index within function (0-based, for disambiguation)
    pub decl_index: Option<u32>,
}

impl VariableTypeInfo {
    pub fn new(name: String, file: String, line: u32, column: u32) -> Self {
        Self {
            name,
            ty: "unknown".to_string(),
            is_copy: false,
            is_clone: false,
            is_drop: false,
            is_send: false,
            is_sync: false,
            is_sized: false,
            is_primitive: false,
            is_rc: false,
            is_arc: false,
            is_box: false,
            is_weak: false,
            is_refcell: false,
            is_cell: false,
            is_mutex: false,
            is_rwlock: false,
            is_guard: false,
            is_vec: false,
            is_string: false,
            is_raw_ptr: false,
            is_reference: false,
            is_mutable_reference: false,
            is_slice: false,
            is_str: false,
            is_pin: false,
            is_cow: false,
            is_option: false,
            is_result: false,
            is_once_cell: false,
            is_maybe_uninit: false,
            is_channel: false,
            is_atomic: false,
            is_ordering: false,
            is_join_handle: false,
            is_duration: false,
            is_instant: false,
            is_closure: false,
            is_fn_ptr: false,
            is_future: false,
            is_iterator: false,
            is_callable: false,
            future_output_type: None,
            iterator_item_type: None,
            layout: None,
            is_dyn_trait: false,
            is_union: false,
            is_extern_type: false,
            is_never: false,
            is_static: false,
            is_const: false,
            is_tuple_binding: false,
            is_mut_binding: false,
            is_impl_trait: false,
            lifetime: None,
            binding_mode: None,
            contains_reference: false,
            reference_mutability: None,
            is_ref_binding: false,
            deref_chain: Vec::new(),
            fields: Vec::new(),
            adjustments: Vec::new(),
            pattern_adjustments: Vec::new(),
            type_arguments: Vec::new(),
            initializer_kind: None,
            closure_captures: Vec::new(),
            method_calls: Vec::new(),
            usages: Vec::new(),
            file,
            line,
            column,
            span_start: 0,
            span_end: 0,
            drop_line: None,
            drop_column: None,
            scope_id: None,
            function_name: None,
            decl_index: None,
        }
    }
}

/// Type information for an entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeInfo {
    /// Schema version for forward compatibility
    pub version: String,
    /// Analyzer version that generated this file
    pub analyzer_version: String,
    /// Map from relative file path to variables in that file
    pub files: HashMap<String, Vec<VariableTypeInfo>>,
    /// Standalone expressions by file (thread::spawn, transmute, drop, etc.)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub expressions: HashMap<String, Vec<ExpressionInfo>>,
    /// Await points by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub await_points: HashMap<String, Vec<AwaitPointInfo>>,
    /// Unsafe operations by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub unsafe_operations: HashMap<String, Vec<UnsafeOperationInfo>>,
    /// Borrow spans by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub borrow_spans: HashMap<String, Vec<BorrowSpanInfo>>,
    /// Destructuring patterns by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub destructuring: HashMap<String, Vec<DestructuringInfo>>,
    /// Match bindings by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub match_bindings: HashMap<String, Vec<MatchBindingInfo>>,
    /// Field accesses by file (for partial borrow tracking)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_accesses: HashMap<String, Vec<FieldAccessInfo>>,
    /// Closure trait info by file
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub closure_traits: HashMap<String, Vec<ClosureTraitInfo>>,
    /// Enum variant constructions by file (semantic via sema.resolve_variant)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variants: HashMap<String, Vec<VariantInfo>>,
    /// Lifetime parameters by file (semantic via sema.resolve_lifetime_param)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub lifetimes: HashMap<String, Vec<LifetimeInfo>>,
    /// Loop labels by file (semantic via sema.resolve_label)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, Vec<LabelInfo>>,
    /// Const pattern bindings by file (semantic via sema.resolve_bind_pat_to_const)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub const_patterns: HashMap<String, Vec<ConstPatternInfo>>,
    /// Callable expressions by file (semantic via Type::as_callable, impls_fnonce)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub callables: HashMap<String, Vec<CallableInfo>>,
    /// Record field expressions by file (semantic via sema.resolve_record_field)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub record_field_exprs: HashMap<String, Vec<RecordFieldExprInfo>>,
    /// Record field patterns by file (semantic via sema.resolve_record_pat_field)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub record_field_pats: HashMap<String, Vec<RecordFieldPatInfo>>,
    /// Index by variable name for macro lookup (stable Rust compatible)
    #[serde(default)]
    pub by_name: HashMap<String, Vec<VariableTypeInfo>>,
    /// Index by (function_name, var_name) for precise lookup
    #[serde(default)]
    pub by_function: HashMap<String, HashMap<String, Vec<VariableTypeInfo>>>,
}

impl ProjectTypeInfo {
    pub fn new() -> Self {
        Self {
            version: "3.0".to_string(),
            analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
            files: HashMap::new(),
            expressions: HashMap::new(),
            await_points: HashMap::new(),
            unsafe_operations: HashMap::new(),
            borrow_spans: HashMap::new(),
            destructuring: HashMap::new(),
            match_bindings: HashMap::new(),
            field_accesses: HashMap::new(),
            closure_traits: HashMap::new(),
            variants: HashMap::new(),
            lifetimes: HashMap::new(),
            labels: HashMap::new(),
            const_patterns: HashMap::new(),
            callables: HashMap::new(),
            record_field_exprs: HashMap::new(),
            record_field_pats: HashMap::new(),
            by_name: HashMap::new(),
            by_function: HashMap::new(),
        }
    }
    
    /// Build the by_name and by_function indices from files data
    pub fn build_name_index(&mut self) {
        self.by_name.clear();
        self.by_function.clear();
        for vars in self.files.values() {
            for var in vars {
                // by_name index
                self.by_name.entry(var.name.clone()).or_default().push(var.clone());
                
                // by_function index
                if let Some(ref fn_name) = var.function_name {
                    self.by_function
                        .entry(fn_name.clone())
                        .or_default()
                        .entry(var.name.clone())
                        .or_default()
                        .push(var.clone());
                }
            }
        }
    }
}

impl Default for ProjectTypeInfo {
    fn default() -> Self {
        Self::new()
    }
}
