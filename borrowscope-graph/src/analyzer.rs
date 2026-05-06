#![allow(dead_code)]
//! Integration with borrowscope-analyzer's type-info.json output.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::edge::CaptureMode;
use crate::graph::OwnershipGraph;
use crate::node::{NodeId, ScopeKind};

// ═══════════════════════════════════════════════════════════════════════════
// Error type
// ═══════════════════════════════════════════════════════════════════════════

/// Errors during analyzer integration.
#[derive(Debug)]
pub enum EnrichError {
    IoError(std::io::Error),
    ParseError(String),
    NotFound(String),
}

impl From<std::io::Error> for EnrichError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_json::Error> for EnrichError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Type info JSON structures (subset of analyzer output we need)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
struct TypeInfoFile {
    version: String,
    files: HashMap<String, Vec<VariableInfo>>,
    #[serde(default)]
    closure_traits: HashMap<String, Vec<ClosureTraitEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct VariableInfo {
    name: String,
    ty: String,
    #[serde(default)]
    is_copy: bool,
    #[serde(default)]
    is_clone: bool,
    #[serde(default)]
    is_send: bool,
    #[serde(default)]
    is_sync: bool,
    #[serde(default)]
    is_drop: bool,
    #[serde(default)]
    is_rc: bool,
    #[serde(default)]
    is_arc: bool,
    #[serde(default)]
    is_box: bool,
    #[serde(default)]
    is_refcell: bool,
    #[serde(default)]
    is_cell: bool,
    #[serde(default)]
    is_mutex: bool,
    #[serde(default)]
    is_rwlock: bool,
    #[serde(default)]
    initializer_kind: Option<String>,
    #[serde(default)]
    function_name: Option<String>,
    #[serde(default)]
    scope_id: Option<u32>,
    #[serde(default)]
    line: u32,
    #[serde(default)]
    column: u32,
    #[serde(default)]
    drop_line: Option<u32>,
    #[serde(default)]
    drop_column: Option<u32>,
    #[serde(default)]
    method_calls: Vec<MethodCallEntry>,
    #[serde(default)]
    closure_captures: Vec<ClosureCaptureEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct MethodCallEntry {
    #[serde(default)]
    method: String,
    #[serde(default)]
    self_borrow: Option<String>,
    #[serde(default)]
    line: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ClosureCaptureEntry {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClosureTraitEntry {
    #[serde(default)]
    line: u32,
    #[serde(default)]
    column: u32,
    #[serde(default)]
    fn_trait: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.1 Loading type info into graph nodes
// ═══════════════════════════════════════════════════════════════════════════

/// Enriched metadata attached to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedNode {
    pub node: NodeId,
    pub is_copy: bool,
    pub is_smart_pointer: bool,
    pub is_interior_mutable: bool,
    pub is_sync: bool,
    pub is_send: bool,
    pub traits: Vec<String>,
    pub initializer_kind: Option<String>,
}

/// Source location for a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Enrich graph nodes with analyzer data. Returns count of enriched nodes.
pub fn enrich_from_analyzer(
    graph: &OwnershipGraph,
    type_info_path: &Path,
) -> Result<(Vec<EnrichedNode>, usize), EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;
    enrich_from_type_info(graph, &type_info)
}

/// Enrich graph nodes by auto-discovering .borrowscope/type-info.json.
pub fn enrich_from_project(
    graph: &OwnershipGraph,
    project_root: &Path,
) -> Result<(Vec<EnrichedNode>, usize), EnrichError> {
    let path = project_root.join(".borrowscope").join("type-info.json");
    if !path.exists() {
        return Err(EnrichError::NotFound(format!(
            "type-info.json not found at {}",
            path.display()
        )));
    }
    enrich_from_analyzer(graph, &path)
}

fn enrich_from_type_info(
    graph: &OwnershipGraph,
    type_info: &TypeInfoFile,
) -> Result<(Vec<EnrichedNode>, usize), EnrichError> {
    let mut enriched = Vec::new();
    let mut count = 0;

    // Build lookup from variable name to analyzer info
    let mut var_lookup: HashMap<&str, &VariableInfo> = HashMap::new();
    for vars in type_info.files.values() {
        for v in vars {
            var_lookup.insert(&v.name, v);
        }
    }

    for node in graph.nodes() {
        if let Some(info) = var_lookup.get(node.name()) {
            let mut traits = Vec::new();
            if info.is_copy {
                traits.push("Copy".to_string());
            }
            if info.is_clone {
                traits.push("Clone".to_string());
            }
            if info.is_drop {
                traits.push("Drop".to_string());
            }
            if info.is_send {
                traits.push("Send".to_string());
            }
            if info.is_sync {
                traits.push("Sync".to_string());
            }

            enriched.push(EnrichedNode {
                node: node.id(),
                is_copy: info.is_copy,
                is_smart_pointer: info.is_rc || info.is_arc || info.is_box,
                is_interior_mutable: info.is_refcell || info.is_cell || info.is_mutex,
                is_sync: info.is_sync,
                is_send: info.is_send,
                traits,
                initializer_kind: info.initializer_kind.clone(),
            });
            count += 1;
        }
    }

    Ok((enriched, count))
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.2 Static graph construction (without runtime)
// ═══════════════════════════════════════════════════════════════════════════

/// Build a static ownership graph from analyzer data alone.
pub fn static_graph_from_analyzer(type_info_path: &Path) -> Result<OwnershipGraph, EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;
    Ok(build_static_graph(&type_info, None))
}

/// Build static graph for a single function.
pub fn static_graph_for_function(
    type_info_path: &Path,
    function_name: &str,
) -> Result<OwnershipGraph, EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;
    Ok(build_static_graph(&type_info, Some(function_name)))
}

fn build_static_graph(type_info: &TypeInfoFile, filter_fn: Option<&str>) -> OwnershipGraph {
    let mut graph = OwnershipGraph::new();
    let mut node_map: HashMap<String, NodeId> = HashMap::new();

    // Create nodes for all variables
    for vars in type_info.files.values() {
        for v in vars {
            if let Some(fn_filter) = filter_fn {
                if v.function_name.as_deref() != Some(fn_filter) {
                    continue;
                }
            }

            let id = graph.add_variable(&v.name, &v.ty, v.line as u64);
            let key = format!("{}:{}", v.function_name.as_deref().unwrap_or(""), v.name);
            node_map.insert(key.clone(), id);
            // Also store by name alone for simpler lookups
            node_map.entry(v.name.clone()).or_insert(id);

            // Infer edges from initializer_kind
            if let Some(ref init) = v.initializer_kind {
                match init.as_str() {
                    "rc_clone" => {
                        // Find another Rc variable in the same function as potential source
                        if let Some(source_id) = find_rc_source(&node_map, &type_info, v, false) {
                            graph.add_rc_clone(id, source_id, 0, v.line as u64);
                        }
                    }
                    "arc_clone" => {
                        if let Some(source_id) = find_rc_source(&node_map, &type_info, v, true) {
                            graph.add_arc_clone(id, source_id, 0, v.line as u64);
                        }
                    }
                    _ => {}
                }
            }

            // Infer edges from method_calls
            for mc in &v.method_calls {
                if let Some(ref borrow_type) = mc.self_borrow {
                    match borrow_type.as_str() {
                        "mutable" => {
                            // Create a synthetic borrow edge
                            let borrower = graph.add_variable(
                                &format!("&mut_{}", v.name),
                                "&mut",
                                mc.line as u64,
                            );
                            graph.add_borrow(borrower, id, true, mc.line as u64);
                        }
                        "immutable" => {
                            let borrower =
                                graph.add_variable(&format!("&_{}", v.name), "&", mc.line as u64);
                            graph.add_borrow(borrower, id, false, mc.line as u64);
                        }
                        _ => {}
                    }
                }
            }

            // Infer edges from closure_captures
            for cap in &v.closure_captures {
                if let Some(ref cap_name) = cap.name {
                    if let Some(&captured_id) = node_map.get(cap_name.as_str()) {
                        let mode = match cap.mode.as_deref() {
                            Some("by_mut_ref") => CaptureMode::ByMutRef,
                            Some("by_move") => CaptureMode::ByMove,
                            _ => CaptureMode::ByRef,
                        };
                        graph.add_capture(id, captured_id, mode, v.line as u64);
                    }
                }
            }
        }
    }

    graph
}

/// Find a potential Rc/Arc source variable (same type, earlier declaration, same function).
fn find_rc_source(
    node_map: &HashMap<String, NodeId>,
    type_info: &TypeInfoFile,
    clone_var: &VariableInfo,
    is_arc: bool,
) -> Option<NodeId> {
    for vars in type_info.files.values() {
        for v in vars {
            if v.name == clone_var.name {
                continue;
            }
            if v.function_name != clone_var.function_name {
                continue;
            }
            if v.line >= clone_var.line {
                continue;
            }
            // Check if it's the same Rc/Arc type
            let is_source = if is_arc { v.is_arc } else { v.is_rc };
            if !is_source {
                continue;
            }
            // Check if initializer is rc_new/arc_new (original, not another clone)
            let is_origin = match v.initializer_kind.as_deref() {
                Some("rc_new") | Some("arc_new") => true,
                _ => false,
            };
            if is_origin {
                let key = format!("{}:{}", v.function_name.as_deref().unwrap_or(""), v.name);
                if let Some(&id) = node_map.get(&key).or_else(|| node_map.get(&v.name)) {
                    return Some(id);
                }
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.3 Scope hierarchy from analyzer
// ═══════════════════════════════════════════════════════════════════════════

/// Build scope hierarchy from analyzer data and attach to graph.
pub fn build_scope_hierarchy(
    graph: &mut OwnershipGraph,
    type_info_path: &Path,
) -> Result<(), EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    // Group variables by function_name
    let mut functions: HashMap<String, Vec<&VariableInfo>> = HashMap::new();
    for vars in type_info.files.values() {
        for v in vars {
            if let Some(ref fn_name) = v.function_name {
                functions.entry(fn_name.clone()).or_default().push(v);
            }
        }
    }

    // Create scope nodes for each function
    for (fn_name, vars) in &functions {
        let min_line = vars.iter().map(|v| v.line).min().unwrap_or(0);
        let max_line = vars
            .iter()
            .filter_map(|v| v.drop_line)
            .max()
            .unwrap_or(min_line);
        let scope_id = graph.add_scope(fn_name, ScopeKind::Function, min_line as u64);
        graph.mark_dropped(scope_id, max_line as u64);
    }

    Ok(())
}

/// Get the scope tree for a specific function.
pub fn function_scope_tree(
    type_info_path: &Path,
    function_name: &str,
) -> Result<Vec<String>, EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    let mut vars_in_fn = Vec::new();
    for vars in type_info.files.values() {
        for v in vars {
            if v.function_name.as_deref() == Some(function_name) {
                vars_in_fn.push(v.name.clone());
            }
        }
    }

    Ok(vars_in_fn)
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.4 Source location mapping
// ═══════════════════════════════════════════════════════════════════════════

/// Attach source locations to graph nodes from analyzer data.
/// Returns count of nodes with locations attached.
pub fn attach_source_locations(
    graph: &OwnershipGraph,
    type_info_path: &Path,
) -> Result<(Vec<(NodeId, SourceLocation)>, usize), EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    let mut locations = Vec::new();
    let mut count = 0;

    // Build lookup
    let mut var_locations: HashMap<&str, (&str, u32, u32)> = HashMap::new();
    for (file, vars) in &type_info.files {
        for v in vars {
            var_locations.insert(&v.name, (file.as_str(), v.line, v.column));
        }
    }

    for node in graph.nodes() {
        if let Some(&(file, line, column)) = var_locations.get(node.name()) {
            locations.push((
                node.id(),
                SourceLocation {
                    file: file.to_string(),
                    line,
                    column,
                },
            ));
            count += 1;
        }
    }

    Ok((locations, count))
}

/// Find graph nodes at a given source location.
pub fn node_at_location(
    graph: &OwnershipGraph,
    type_info_path: &Path,
    file: &str,
    line: u32,
) -> Result<Vec<NodeId>, EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    // Find variable names at this location
    let mut names_at_line: Vec<&str> = Vec::new();
    if let Some(vars) = type_info.files.get(file) {
        for v in vars {
            if v.line == line {
                names_at_line.push(&v.name);
            }
        }
    }

    let mut result = Vec::new();
    for name in names_at_line {
        for &id in graph.find_by_name(name) {
            result.push(id);
        }
    }

    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.5 Drop location and lifetime bounds
// ═══════════════════════════════════════════════════════════════════════════

/// Attach drop locations from analyzer data.
/// Returns count of nodes with drop locations.
pub fn attach_drop_locations(
    graph: &OwnershipGraph,
    type_info_path: &Path,
) -> Result<(Vec<(NodeId, u32)>, usize), EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    let mut drops = Vec::new();
    let mut count = 0;

    let mut var_drops: HashMap<&str, u32> = HashMap::new();
    for vars in type_info.files.values() {
        for v in vars {
            if let Some(drop_line) = v.drop_line {
                var_drops.insert(&v.name, drop_line);
            }
        }
    }

    for node in graph.nodes() {
        if let Some(&drop_line) = var_drops.get(node.name()) {
            drops.push((node.id(), drop_line));
            count += 1;
        }
    }

    Ok((drops, count))
}

/// Get the source-level lifetime of a variable (declaration line to drop line).
pub fn source_lifetime(
    type_info_path: &Path,
    var_name: &str,
) -> Result<Option<(u32, u32)>, EnrichError> {
    let json = std::fs::read_to_string(type_info_path)?;
    let type_info: TypeInfoFile = serde_json::from_str(&json)?;

    for vars in type_info.files.values() {
        for v in vars {
            if v.name == var_name {
                if let Some(drop_line) = v.drop_line {
                    return Ok(Some((v.line, drop_line)));
                }
                return Ok(None);
            }
        }
    }

    Ok(None)
}
