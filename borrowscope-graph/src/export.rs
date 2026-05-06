//! Serialization and multi-format export for ownership graphs.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeKind};
use crate::graph::OwnershipGraph;
use crate::node::{Node, NodeId};

// ═══════════════════════════════════════════════════════════════════════════
// Error types
// ═══════════════════════════════════════════════════════════════════════════

/// Errors during export operations.
#[derive(Debug)]
pub enum ExportError {
    SerializationError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

impl From<serde_json::Error> for ExportError {
    fn from(e: serde_json::Error) -> Self { Self::SerializationError(e.to_string()) }
}

impl From<rmp_serde::encode::Error> for ExportError {
    fn from(e: rmp_serde::encode::Error) -> Self { Self::SerializationError(e.to_string()) }
}

/// Errors during import operations.
#[derive(Debug)]
pub enum ImportError {
    ParseError(String),
    VersionMismatch { expected: String, found: String },
    IoError(std::io::Error),
}

impl From<std::io::Error> for ImportError {
    fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

impl From<serde_json::Error> for ImportError {
    fn from(e: serde_json::Error) -> Self { Self::ParseError(e.to_string()) }
}

impl From<rmp_serde::decode::Error> for ImportError {
    fn from(e: rmp_serde::decode::Error) -> Self { Self::ParseError(e.to_string()) }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.1 JSON export (full and compact)
// ═══════════════════════════════════════════════════════════════════════════

/// Metadata header for exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub version: String,
    pub node_count: usize,
    pub edge_count: usize,
}

/// Export graph to full JSON (all fields, pretty-printed).
pub fn to_json(graph: &OwnershipGraph) -> Result<String, ExportError> {
    Ok(serde_json::to_string_pretty(graph)?)
}

/// Export graph to compact JSON (single line, no extra whitespace).
pub fn to_json_compact(graph: &OwnershipGraph) -> Result<String, ExportError> {
    Ok(serde_json::to_string(graph)?)
}

/// Export graph to a JSON file (pretty-printed).
pub fn to_json_file(graph: &OwnershipGraph, path: &Path) -> Result<(), ExportError> {
    let json = to_json(graph)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Export graph to a compact JSON file.
pub fn to_json_compact_file(graph: &OwnershipGraph, path: &Path) -> Result<(), ExportError> {
    let json = to_json_compact(graph)?;
    std::fs::write(path, json)?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.2 Graphviz DOT export
// ═══════════════════════════════════════════════════════════════════════════

/// Layout direction for DOT export.
#[derive(Debug, Clone, Copy)]
pub enum DotDirection {
    TopBottom,
    LeftRight,
}

/// Color scheme for edge kinds.
#[derive(Debug, Clone)]
pub struct EdgeColorScheme {
    pub borrow_shared: String,
    pub borrow_mut: String,
    pub move_edge: String,
    pub rc_clone: String,
    pub arc_clone: String,
    pub capture: String,
    pub other: String,
}

impl Default for EdgeColorScheme {
    fn default() -> Self {
        Self {
            borrow_shared: "blue".to_string(),
            borrow_mut: "red".to_string(),
            move_edge: "darkgreen".to_string(),
            rc_clone: "purple".to_string(),
            arc_clone: "darkorange".to_string(),
            capture: "brown".to_string(),
            other: "gray".to_string(),
        }
    }
}

/// Options for DOT export.
#[derive(Debug, Clone)]
pub struct DotOptions {
    pub show_types: bool,
    pub show_timestamps: bool,
    pub colors: EdgeColorScheme,
    pub direction: DotDirection,
}

impl Default for DotOptions {
    fn default() -> Self {
        Self {
            show_types: true,
            show_timestamps: false,
            colors: EdgeColorScheme::default(),
            direction: DotDirection::TopBottom,
        }
    }
}

/// Export graph to Graphviz DOT format.
pub fn to_dot(graph: &OwnershipGraph, options: &DotOptions) -> String {
    let mut out = String::new();
    let rankdir = match options.direction {
        DotDirection::TopBottom => "TB",
        DotDirection::LeftRight => "LR",
    };

    out.push_str("digraph ownership {\n");
    out.push_str(&format!("    rankdir={};\n", rankdir));
    out.push_str("    node [fontname=\"monospace\"];\n\n");

    // Nodes
    for node in graph.nodes() {
        let (shape, label) = match node {
            Node::Variable(v) => {
                let label = if options.show_types {
                    format!("{}: {}", v.name, v.type_name)
                } else {
                    v.name.clone()
                };
                ("box", label)
            }
            Node::Scope(s) => {
                ("ellipse".to_string(), s.name.clone());
                ("ellipse", s.name.clone())
            }
        };
        let escaped_label = label.replace('"', "\\\"").replace('<', "\\<").replace('>', "\\>");
        out.push_str(&format!("    n{} [label=\"{}\", shape={}];\n", node.id().0, escaped_label, shape));
    }

    out.push('\n');

    // Edges
    for edge in graph.edges() {
        let (color, style, label) = edge_dot_attrs(edge, &options.colors, options.show_timestamps);
        out.push_str(&format!(
            "    n{} -> n{} [label=\"{}\", color={}, style={}];\n",
            edge.source.0, edge.target.0, label, color, style
        ));
    }

    out.push_str("}\n");
    out
}

/// Export to DOT file.
pub fn to_dot_file(graph: &OwnershipGraph, path: &Path, options: &DotOptions) -> Result<(), ExportError> {
    let dot = to_dot(graph, options);
    std::fs::write(path, dot)?;
    Ok(())
}

fn edge_dot_attrs(edge: &Edge, colors: &EdgeColorScheme, show_ts: bool) -> (String, String, String) {
    let (color, style, kind_label) = match &edge.kind {
        EdgeKind::BorrowShared => (colors.borrow_shared.clone(), "dashed", "&"),
        EdgeKind::BorrowMut => (colors.borrow_mut.clone(), "bold", "&mut"),
        EdgeKind::Move => (colors.move_edge.clone(), "solid", "move"),
        EdgeKind::RcClone { .. } => (colors.rc_clone.clone(), "dotted", "Rc::clone"),
        EdgeKind::ArcClone { .. } => (colors.arc_clone.clone(), "dotted", "Arc::clone"),
        EdgeKind::WeakDowngrade => (colors.other.clone(), "dashed", "downgrade"),
        EdgeKind::RefCellBorrow { mutable } => {
            if *mutable { (colors.borrow_mut.clone(), "bold", "borrow_mut") }
            else { (colors.borrow_shared.clone(), "dashed", "borrow") }
        }
        EdgeKind::LockAcquire { .. } => (colors.other.clone(), "bold", "lock"),
        EdgeKind::ClosureCapture { .. } => (colors.capture.clone(), "dotted", "capture"),
        EdgeKind::ScopeContains => (colors.other.clone(), "invis", ""),
        EdgeKind::ChannelSend => (colors.other.clone(), "solid", "send"),
    };

    let label = if show_ts {
        format!("{} @{}", kind_label, edge.created_at)
    } else {
        kind_label.to_string()
    };

    (color, style.to_string(), label)
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.3 MessagePack export
// ═══════════════════════════════════════════════════════════════════════════

/// Export graph to MessagePack bytes.
pub fn to_msgpack(graph: &OwnershipGraph) -> Result<Vec<u8>, ExportError> {
    Ok(rmp_serde::to_vec(graph)?)
}

/// Export graph to MessagePack file.
pub fn to_msgpack_file(graph: &OwnershipGraph, path: &Path) -> Result<(), ExportError> {
    let bytes = to_msgpack(graph)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Import graph from MessagePack bytes.
pub fn from_msgpack(data: &[u8]) -> Result<OwnershipGraph, ImportError> {
    let mut graph: OwnershipGraph = rmp_serde::from_slice(data)?;
    check_version(&graph)?;
    graph.rebuild_indices();
    Ok(graph)
}

/// Import graph from MessagePack file.
pub fn from_msgpack_file(path: &Path) -> Result<OwnershipGraph, ImportError> {
    let data = std::fs::read(path)?;
    from_msgpack(&data)
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.4 Delta export (incremental updates)
// ═══════════════════════════════════════════════════════════════════════════

/// Incremental graph update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDelta {
    pub sequence: u64,
    pub added_nodes: Vec<Node>,
    pub added_edges: Vec<Edge>,
    pub dropped_nodes: Vec<(NodeId, u64)>,
    pub ended_edges: Vec<(crate::edge::EdgeId, u64)>,
}

impl GraphDelta {
    /// Create an empty delta.
    pub fn empty(sequence: u64) -> Self {
        Self {
            sequence,
            added_nodes: Vec::new(),
            added_edges: Vec::new(),
            dropped_nodes: Vec::new(),
            ended_edges: Vec::new(),
        }
    }

    /// Whether this delta contains any changes.
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.dropped_nodes.is_empty()
            && self.ended_edges.is_empty()
    }
}

/// Compute the delta between two graph states.
/// Returns changes needed to transform `before` into `after`.
pub fn compute_delta(before: &OwnershipGraph, after: &OwnershipGraph, sequence: u64) -> GraphDelta {
    let before_node_ids: std::collections::HashSet<NodeId> =
        before.nodes().iter().map(|n| n.id()).collect();
    let _after_node_ids: std::collections::HashSet<NodeId> =
        after.nodes().iter().map(|n| n.id()).collect();

    let before_edge_ids: std::collections::HashSet<crate::edge::EdgeId> =
        before.edges().iter().map(|e| e.id).collect();
    let _after_edge_ids: std::collections::HashSet<crate::edge::EdgeId> =
        after.edges().iter().map(|e| e.id).collect();

    let added_nodes: Vec<Node> = after.nodes().iter()
        .filter(|n| !before_node_ids.contains(&n.id()))
        .cloned()
        .collect();

    let added_edges: Vec<Edge> = after.edges().iter()
        .filter(|e| !before_edge_ids.contains(&e.id))
        .cloned()
        .collect();

    // Nodes that gained an end_time
    let dropped_nodes: Vec<(NodeId, u64)> = after.nodes().iter()
        .filter_map(|n| {
            let had_end = before.get_node(n.id()).and_then(|bn| bn.end_time());
            let has_end = n.end_time();
            if had_end.is_none() && has_end.is_some() {
                Some((n.id(), has_end.unwrap()))
            } else {
                None
            }
        })
        .collect();

    // Edges that gained an ended_at
    let ended_edges: Vec<(crate::edge::EdgeId, u64)> = after.edges().iter()
        .filter_map(|e| {
            let had_end = before.get_edge(e.id).and_then(|be| be.ended_at);
            if had_end.is_none() && e.ended_at.is_some() {
                Some((e.id, e.ended_at.unwrap()))
            } else {
                None
            }
        })
        .collect();

    GraphDelta { sequence, added_nodes, added_edges, dropped_nodes, ended_edges }
}

/// Apply a delta to an existing graph.
pub fn apply_delta(graph: &mut OwnershipGraph, delta: &GraphDelta) {
    // Note: adding nodes/edges from a delta requires rebuilding indices
    // For simplicity, we serialize the delta info and rebuild
    for (node_id, timestamp) in &delta.dropped_nodes {
        graph.mark_dropped(*node_id, *timestamp);
    }
    for (edge_id, timestamp) in &delta.ended_edges {
        graph.end_edge(*edge_id, *timestamp);
    }
    // Added nodes/edges would need special handling since IDs must match
    // This is a simplified implementation for the common case (drops/ends)
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.5 Import from JSON
// ═══════════════════════════════════════════════════════════════════════════

/// Import graph from JSON string.
pub fn from_json(json: &str) -> Result<OwnershipGraph, ImportError> {
    let mut graph: OwnershipGraph = serde_json::from_str(json)?;
    check_version(&graph)?;
    graph.rebuild_indices();
    Ok(graph)
}

/// Import graph from JSON file.
pub fn from_json_file(path: &Path) -> Result<OwnershipGraph, ImportError> {
    let json = std::fs::read_to_string(path)?;
    from_json(&json)
}

fn check_version(graph: &OwnershipGraph) -> Result<(), ImportError> {
    let found = graph.version();
    let expected = crate::graph::GRAPH_VERSION;
    if found != expected {
        return Err(ImportError::VersionMismatch {
            expected: expected.to_string(),
            found: found.to_string(),
        });
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.6 D3.js-compatible JSON format
// ═══════════════════════════════════════════════════════════════════════════

/// D3.js force-directed graph format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D3Graph {
    pub nodes: Vec<D3Node>,
    pub links: Vec<D3Link>,
}

/// A node in D3.js format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D3Node {
    pub id: usize,
    pub name: String,
    pub group: u32,
    pub size: f64,
    pub type_name: String,
}

/// A link in D3.js format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D3Link {
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub kind: String,
    pub color: String,
}

/// Export graph in D3.js-compatible format.
pub fn to_d3(graph: &OwnershipGraph) -> D3Graph {
    let colors = EdgeColorScheme::default();

    let nodes: Vec<D3Node> = graph.nodes().iter().map(|n| {
        let (group, type_name) = match n {
            Node::Variable(v) => (node_group(&v.type_name), v.type_name.clone()),
            Node::Scope(s) => (10, format!("scope:{:?}", s.kind)),
        };
        let edge_count = graph.outgoing_edges(n.id()).len() + graph.incoming_edges(n.id()).len();
        D3Node {
            id: n.id().0,
            name: n.name().to_string(),
            group,
            size: (edge_count as f64 + 1.0).sqrt() * 5.0,
            type_name,
        }
    }).collect();

    let links: Vec<D3Link> = graph.edges().iter().map(|e| {
        let (kind, color, value) = match &e.kind {
            EdgeKind::BorrowShared => ("borrow", &colors.borrow_shared, 1.0),
            EdgeKind::BorrowMut => ("borrow_mut", &colors.borrow_mut, 1.5),
            EdgeKind::Move => ("move", &colors.move_edge, 2.0),
            EdgeKind::RcClone { .. } => ("rc_clone", &colors.rc_clone, 1.0),
            EdgeKind::ArcClone { .. } => ("arc_clone", &colors.arc_clone, 1.0),
            EdgeKind::WeakDowngrade => ("weak", &colors.other, 0.5),
            EdgeKind::RefCellBorrow { .. } => ("refcell", &colors.borrow_shared, 1.0),
            EdgeKind::LockAcquire { .. } => ("lock", &colors.other, 1.5),
            EdgeKind::ClosureCapture { .. } => ("capture", &colors.capture, 1.0),
            EdgeKind::ScopeContains => ("contains", &colors.other, 0.5),
            EdgeKind::ChannelSend => ("send", &colors.other, 2.0),
        };
        D3Link {
            source: e.source.0,
            target: e.target.0,
            value,
            kind: kind.to_string(),
            color: color.clone(),
        }
    }).collect();

    D3Graph { nodes, links }
}

/// Export D3 graph to JSON string.
pub fn to_d3_json(graph: &OwnershipGraph) -> Result<String, ExportError> {
    let d3 = to_d3(graph);
    Ok(serde_json::to_string_pretty(&d3)?)
}

fn node_group(type_name: &str) -> u32 {
    if type_name.contains("Rc") { 2 }
    else if type_name.contains("Arc") { 3 }
    else if type_name.contains("RefCell") || type_name.contains("Cell") { 4 }
    else if type_name.contains("Mutex") || type_name.contains("RwLock") { 5 }
    else if type_name.starts_with('&') { 1 }
    else { 0 }
}
