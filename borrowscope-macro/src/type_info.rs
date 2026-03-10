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

    // Declaration type
    pub is_static: bool,
    pub is_const: bool,

    // Binding patterns
    pub is_tuple_binding: bool,
    pub is_mut_binding: bool,
    pub is_impl_trait: bool,
    
    // Initializer pattern (semantic)
    pub initializer_kind: Option<String>,
    
    // Function context (for disambiguation)
    pub function_name: Option<String>,
    pub decl_index: Option<u32>,
    
    // Method calls on this variable (semantic operation tracking)
    #[serde(default)]
    pub method_calls: Vec<MethodCallInfo>,
}

/// Method call information (compact - only fields macro needs)
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MethodCallInfo {
    pub method: String,
    pub line: u32,
    pub column: u32,
    pub operation: Option<String>,
    pub self_borrow: Option<String>,
}

/// Standalone expression information (compact - only fields macro needs)
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExpressionInfo {
    pub line: u32,
    pub column: u32,
    pub path: Option<String>,
    pub operation: String,
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
}

pub struct TypeInfoCache {
    by_name: HashMap<String, Vec<VariableTypeInfo>>,
    by_function: HashMap<String, HashMap<String, Vec<VariableTypeInfo>>>,
    expressions: HashMap<String, Vec<ExpressionInfo>>,
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
