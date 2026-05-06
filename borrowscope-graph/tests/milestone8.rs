//! Comprehensive tests for Milestone 8: Integration with borrowscope-analyzer.

use borrowscope_graph::analyzer::*;
use borrowscope_graph::*;
use std::io::Write;
use std::path::Path;

// Path to real type-info.json fixture
fn fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.borrowscope/type-info.json"
    ))
}

// Create a minimal type-info.json for controlled testing
fn create_test_fixture(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("type-info.json");
    let json = r#"{
  "version": "3.0",
  "analyzer_version": "0.1.0",
  "files": {
    "src/main.rs": [
      {
        "name": "data",
        "ty": "Vec<i32>",
        "is_copy": false,
        "is_clone": true,
        "is_drop": true,
        "is_send": true,
        "is_sync": true,
        "is_sized": true,
        "is_primitive": false,
        "is_rc": false,
        "is_arc": false,
        "is_box": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "is_vec": true,
        "initializer_kind": "vec_new",
        "function_name": "main",
        "scope_id": 1,
        "line": 5,
        "column": 8,
        "drop_line": 20,
        "drop_column": 1,
        "method_calls": [
          {"method": "push", "self_borrow": "mutable", "line": 10},
          {"method": "len", "self_borrow": "immutable", "line": 15}
        ],
        "closure_captures": []
      },
      {
        "name": "rc",
        "ty": "Rc<String>",
        "is_copy": false,
        "is_clone": true,
        "is_drop": true,
        "is_send": false,
        "is_sync": false,
        "is_sized": true,
        "is_primitive": false,
        "is_rc": true,
        "is_arc": false,
        "is_box": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "initializer_kind": "rc_new",
        "function_name": "main",
        "scope_id": 1,
        "line": 6,
        "column": 8,
        "drop_line": 18,
        "drop_column": 1,
        "method_calls": [],
        "closure_captures": []
      },
      {
        "name": "rc_clone",
        "ty": "Rc<String>",
        "is_copy": false,
        "is_clone": true,
        "is_drop": true,
        "is_send": false,
        "is_sync": false,
        "is_sized": true,
        "is_primitive": false,
        "is_rc": true,
        "is_arc": false,
        "is_box": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "initializer_kind": "rc_clone",
        "function_name": "main",
        "scope_id": 1,
        "line": 7,
        "column": 8,
        "drop_line": 16,
        "drop_column": 1,
        "method_calls": [],
        "closure_captures": []
      },
      {
        "name": "callback",
        "ty": "impl Fn(i32)",
        "is_copy": false,
        "is_clone": false,
        "is_drop": false,
        "is_send": true,
        "is_sync": true,
        "is_sized": true,
        "is_primitive": false,
        "is_rc": false,
        "is_arc": false,
        "is_box": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "initializer_kind": "closure",
        "function_name": "main",
        "scope_id": 1,
        "line": 8,
        "column": 8,
        "drop_line": null,
        "drop_column": null,
        "method_calls": [],
        "closure_captures": [
          {"name": "data", "mode": "by_ref"}
        ]
      },
      {
        "name": "x",
        "ty": "i32",
        "is_copy": true,
        "is_clone": true,
        "is_drop": false,
        "is_send": true,
        "is_sync": true,
        "is_sized": true,
        "is_primitive": true,
        "is_rc": false,
        "is_arc": false,
        "is_box": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "initializer_kind": "literal",
        "function_name": "helper",
        "scope_id": 2,
        "line": 25,
        "column": 8,
        "drop_line": 30,
        "drop_column": 1,
        "method_calls": [],
        "closure_captures": []
      }
    ]
  },
  "closure_traits": {}
}"#;
    std::fs::write(&path, json).unwrap();
    path
}

fn test_fixture_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.1 Enrichment
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_enrich_nodes_gain_type_flags() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);
    graph.add_variable("rc", "Rc<String>", 10);

    let (enriched, count) = enrich_from_analyzer(&graph, &path).unwrap();
    assert!(count >= 2);

    let data_enriched = enriched
        .iter()
        .find(|e| graph.get_node(e.node).map(|n| n.name()) == Some("data"))
        .unwrap();
    assert!(!data_enriched.is_copy);
    assert!(!data_enriched.is_smart_pointer);
    assert!(data_enriched.traits.contains(&"Clone".to_string()));
    assert!(data_enriched.traits.contains(&"Send".to_string()));

    let rc_enriched = enriched
        .iter()
        .find(|e| graph.get_node(e.node).map(|n| n.name()) == Some("rc"))
        .unwrap();
    assert!(rc_enriched.is_smart_pointer);
    assert!(!rc_enriched.is_send);
}

#[test]
fn test_enrich_unmatched_nodes_unchanged() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);
    graph.add_variable("unknown_var", "Mystery", 10);

    let (enriched, count) = enrich_from_analyzer(&graph, &path).unwrap();
    // Only "data" matches, "unknown_var" does not
    assert_eq!(count, 1);
    assert!(enriched
        .iter()
        .all(|e| { graph.get_node(e.node).map(|n| n.name()) != Some("unknown_var") }));
}

#[test]
fn test_enrich_return_count() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);
    graph.add_variable("rc", "Rc<String>", 10);
    graph.add_variable("x", "i32", 20);

    let (_, count) = enrich_from_analyzer(&graph, &path).unwrap();
    assert_eq!(count, 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.2 Static graph construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_static_graph_contains_all_variables() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let graph = static_graph_from_analyzer(&path).unwrap();
    // 5 variables in fixture + synthetic borrow nodes from method_calls
    assert!(graph.node_count() >= 5);
    assert!(graph.find_by_name("data").len() >= 1);
    assert!(graph.find_by_name("rc").len() >= 1);
    assert!(graph.find_by_name("x").len() >= 1);
}

#[test]
fn test_static_graph_method_borrow_creates_edge() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let graph = static_graph_from_analyzer(&path).unwrap();
    // "data" has method_call with self_borrow: "mutable" → should create BorrowMut edge
    assert!(graph.edge_count() > 0);

    // Find a mutable borrow edge
    let has_mut_borrow = graph
        .edges()
        .iter()
        .any(|e| matches!(e.kind, EdgeKind::BorrowMut));
    assert!(has_mut_borrow);
}

#[test]
fn test_static_graph_closure_capture_creates_edge() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let graph = static_graph_from_analyzer(&path).unwrap();
    // "callback" captures "data" by_ref
    let has_capture = graph
        .edges()
        .iter()
        .any(|e| matches!(e.kind, EdgeKind::ClosureCapture { .. }));
    assert!(has_capture);
}

#[test]
fn test_static_graph_for_single_function() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let graph = static_graph_for_function(&path, "helper").unwrap();
    // Only "x" is in the "helper" function
    assert!(graph.find_by_name("x").len() >= 1);
    assert!(graph.find_by_name("data").is_empty());
}

#[test]
fn test_static_graph_rc_clone_creates_edge() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let graph = static_graph_from_analyzer(&path).unwrap();
    // "rc_clone" has initializer_kind "rc_clone" and "rc" has "rc_new"
    // Should create an RcClone edge from rc_clone to rc
    let has_rc_clone_edge = graph
        .edges()
        .iter()
        .any(|e| matches!(e.kind, EdgeKind::RcClone { .. }));
    assert!(
        has_rc_clone_edge,
        "Expected RcClone edge from rc_clone initializer"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.3 Scope hierarchy
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_scope_hierarchy_groups_by_function() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let vars = function_scope_tree(&path, "main").unwrap();
    assert!(vars.contains(&"data".to_string()));
    assert!(vars.contains(&"rc".to_string()));
    assert!(!vars.contains(&"x".to_string())); // x is in "helper"
}

#[test]
fn test_scope_hierarchy_builds_scope_nodes() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 5);

    build_scope_hierarchy(&mut graph, &path).unwrap();
    // Should have added scope nodes for "main" and "helper"
    assert!(graph.find_by_name("main").len() >= 1);
    assert!(graph.find_by_name("helper").len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.4 Source location mapping
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_locations_attached() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);
    graph.add_variable("rc", "Rc<String>", 10);

    let (locations, count) = attach_source_locations(&graph, &path).unwrap();
    assert_eq!(count, 2);

    let data_loc = locations
        .iter()
        .find(|(id, _)| graph.get_node(*id).map(|n| n.name()) == Some("data"))
        .unwrap();
    assert_eq!(data_loc.1.file, "src/main.rs");
    assert_eq!(data_loc.1.line, 5);
    assert_eq!(data_loc.1.column, 8);
}

#[test]
fn test_node_at_location() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);

    let nodes = node_at_location(&graph, &path, "src/main.rs", 5).unwrap();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_node_at_location_not_found() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);

    let nodes = node_at_location(&graph, &path, "src/main.rs", 999).unwrap();
    assert!(nodes.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 8.5 Drop locations and lifetime bounds
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_lifetime_returns_range() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let lifetime = source_lifetime(&path, "data").unwrap();
    assert_eq!(lifetime, Some((5, 20))); // line 5 to drop_line 20
}

#[test]
fn test_source_lifetime_no_drop() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    // "callback" has no drop_line
    let lifetime = source_lifetime(&path, "callback").unwrap();
    assert_eq!(lifetime, None);
}

#[test]
fn test_attach_drop_locations() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    let mut graph = OwnershipGraph::new();
    graph.add_variable("data", "Vec<i32>", 0);
    graph.add_variable("x", "i32", 10); // Copy type, still gets drop location

    let (drops, count) = attach_drop_locations(&graph, &path).unwrap();
    assert_eq!(count, 2);

    let data_drop = drops
        .iter()
        .find(|(id, _)| graph.get_node(*id).map(|n| n.name()) == Some("data"))
        .unwrap();
    assert_eq!(data_drop.1, 20);
}

// ═══════════════════════════════════════════════════════════════════════════
// Error handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_missing_file_returns_error() {
    let graph = OwnershipGraph::new();
    let result = enrich_from_analyzer(&graph, Path::new("/nonexistent/path.json"));
    assert!(result.is_err());
}

#[test]
fn test_malformed_json_returns_parse_error() {
    let dir = test_fixture_dir();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not valid json {{{").unwrap();

    let graph = OwnershipGraph::new();
    let result = enrich_from_analyzer(&graph, &path);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EnrichError::ParseError(_)));
}

#[test]
fn test_enrich_from_project_not_found() {
    let graph = OwnershipGraph::new();
    let result = enrich_from_project(&graph, Path::new("/nonexistent/project"));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EnrichError::NotFound(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Combined: runtime + analyzer
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_combined_runtime_graph_plus_enrichment() {
    let dir = test_fixture_dir();
    let path = create_test_fixture(dir.path());

    // Simulate a runtime graph
    let mut graph = OwnershipGraph::new();
    let data = graph.add_variable("data", "Vec<i32>", 0);
    let r = graph.add_variable("r", "&Vec<i32>", 10);
    graph.add_borrow(r, data, false, 10);

    // Enrich with analyzer data
    let (enriched, count) = enrich_from_analyzer(&graph, &path).unwrap();
    assert!(count >= 1); // at least "data" matches

    // The runtime graph has edges, the enrichment adds metadata
    assert_eq!(graph.edge_count(), 1);
    assert!(!enriched.is_empty());
}
