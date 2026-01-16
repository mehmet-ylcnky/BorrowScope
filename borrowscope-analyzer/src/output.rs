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

    // Callable/async types (semantic via Type methods and trait impl)
    pub is_closure: bool,
    pub is_fn_ptr: bool,
    pub is_future: bool,
    pub is_iterator: bool,
    
    // Trait objects (semantic via as_dyn_trait)
    pub is_dyn_trait: bool,

    // FFI and unsafe types (semantic via ADT)
    pub is_union: bool,
    pub is_extern_type: bool,

    // Static/const binding (semantic via syntax kind)
    pub is_static: bool,
    pub is_const: bool,

    // Binding patterns (semantic via AST pattern analysis)
    pub is_tuple_binding: bool,
    pub is_mut_binding: bool,
    pub is_impl_trait: bool,

    // Initializer kind for tracking strategy
    pub initializer_kind: Option<String>,

    /// Method calls made on this variable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_calls: Vec<MethodCallInfo>,

    /// Source location
    pub file: String,
    pub line: u32,
    pub column: u32,
    
    // Byte offsets for precise matching
    pub span_start: u32,
    pub span_end: u32,
    
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
            is_closure: false,
            is_fn_ptr: false,
            is_future: false,
            is_iterator: false,
            is_dyn_trait: false,
            is_union: false,
            is_extern_type: false,
            is_static: false,
            is_const: false,
            is_tuple_binding: false,
            is_mut_binding: false,
            is_impl_trait: false,
            initializer_kind: None,
            method_calls: Vec::new(),
            file,
            line,
            column,
            span_start: 0,
            span_end: 0,
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
            version: "2.4".to_string(),
            analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
            files: HashMap::new(),
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
