//! Type information output structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type information for a single variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTypeInfo {
    pub name: String,
    pub ty: String,
    pub is_copy: bool,
    pub is_rc: bool,
    pub is_arc: bool,
    pub is_refcell: bool,
    pub is_cell: bool,
    pub is_mutex: bool,
    pub is_rwlock: bool,
    pub is_raw_ptr: bool,
    pub is_union: bool,
    pub is_static: bool,
    pub is_ffi: bool,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Type information for an entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeInfo {
    pub version: String,
    /// Map from file path to variables in that file
    pub files: HashMap<String, Vec<VariableTypeInfo>>,
}

impl ProjectTypeInfo {
    pub fn new() -> Self {
        Self {
            version: "0.1.0".to_string(),
            files: HashMap::new(),
        }
    }
}
