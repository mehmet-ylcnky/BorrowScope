//! Type information output structures
//!
//! These structures are serialized to JSON and read by the proc macro.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type information for a single variable binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTypeInfo {
    /// Variable name as it appears in source
    pub name: String,
    /// Fully resolved type string (e.g., "Rc<String>", "Vec<i32>")
    pub ty: String,
    /// Whether the type implements Copy
    pub is_copy: bool,

    // Smart pointers
    pub is_rc: bool,
    pub is_arc: bool,
    pub is_box: bool,
    pub is_weak: bool,

    // Interior mutability
    pub is_refcell: bool,
    pub is_cell: bool,
    pub is_mutex: bool,
    pub is_rwlock: bool,

    // Guards (borrow scope tracking)
    pub is_guard: bool,

    // Collections
    pub is_vec: bool,
    pub is_string: bool,

    // References and pointers
    pub is_raw_ptr: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_slice: bool,
    pub is_str: bool,

    // Wrapper types
    pub is_pin: bool,
    pub is_cow: bool,
    pub is_option: bool,
    pub is_result: bool,

    // Callable/async types
    pub is_closure: bool,
    pub is_future: bool,
    pub is_iterator: bool,

    // Inner type for nested smart pointers (e.g., "String" for Rc<String>)
    pub inner_type: Option<String>,

    /// Source location
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl VariableTypeInfo {
    pub fn new(name: String, file: String, line: u32, column: u32) -> Self {
        Self {
            name,
            ty: "unknown".to_string(),
            is_copy: false,
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
            is_closure: false,
            is_future: false,
            is_iterator: false,
            inner_type: None,
            file,
            line,
            column,
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
}

impl ProjectTypeInfo {
    pub fn new() -> Self {
        Self {
            version: "1.1".to_string(),
            analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
            files: HashMap::new(),
        }
    }
}

impl Default for ProjectTypeInfo {
    fn default() -> Self {
        Self::new()
    }
}
