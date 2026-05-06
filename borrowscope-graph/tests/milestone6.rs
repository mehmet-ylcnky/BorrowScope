//! Comprehensive tests for Milestone 6: Serialization and Export.

use borrowscope_graph::*;
use borrowscope_graph::export::*;

fn sample_graph() -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "Vec<i32>", 0);
    let r = g.add_variable("r", "&Vec<i32>", 10);
    let m = g.add_variable("m", "&mut Vec<i32>", 50);
    let eid1 = g.add_borrow(r, x, false, 10);
    g.end_edge(eid1, 40);
    let eid2 = g.add_borrow(m, x, true, 50);
    g.end_edge(eid2, 80);
    g.mark_dropped(r, 40);
    g.mark_dropped(m, 80);
    g.mark_dropped(x, 100);
    g
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.1 JSON export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_roundtrip() {
    let g = sample_graph();
    let json = to_json(&g).unwrap();
    let restored = from_json(&json).unwrap();

    assert_eq!(restored.node_count(), g.node_count());
    assert_eq!(restored.edge_count(), g.edge_count());
    assert_eq!(restored.find_by_name("x").len(), 1);
    assert_eq!(restored.find_by_name("r").len(), 1);
}

#[test]
fn test_json_full_is_pretty() {
    let g = sample_graph();
    let json = to_json(&g).unwrap();
    assert!(json.contains('\n')); // pretty-printed has newlines
    assert!(json.contains("  ")); // has indentation
}

#[test]
fn test_json_compact_is_smaller() {
    let g = sample_graph();
    let full = to_json(&g).unwrap();
    let compact = to_json_compact(&g).unwrap();

    assert!(compact.len() < full.len());
    assert!(!compact.contains('\n')); // no newlines in compact
}

#[test]
fn test_json_compact_still_parseable() {
    let g = sample_graph();
    let compact = to_json_compact(&g).unwrap();
    let restored = from_json(&compact).unwrap();
    assert_eq!(restored.node_count(), g.node_count());
}

#[test]
fn test_json_compact_size_reduction() {
    let g = sample_graph();
    let full = to_json(&g).unwrap();
    let compact = to_json_compact(&g).unwrap();
    let reduction = 1.0 - (compact.len() as f64 / full.len() as f64);
    assert!(reduction > 0.30, "Compact should be at least 30% smaller, got {:.0}%", reduction * 100.0);
}

#[test]
fn test_json_file_roundtrip() {
    let g = sample_graph();
    let path = std::path::Path::new("/tmp/borrowscope_test_export.json");
    to_json_file(&g, path).unwrap();
    let restored = from_json_file(path).unwrap();
    assert_eq!(restored.node_count(), g.node_count());
    std::fs::remove_file(path).ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.2 DOT export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dot_valid_syntax() {
    let g = sample_graph();
    let dot = to_dot(&g, &DotOptions::default());
    assert!(dot.starts_with("digraph ownership {"));
    assert!(dot.ends_with("}\n"));
    assert!(dot.contains("rankdir=TB"));
}

#[test]
fn test_dot_contains_node_labels() {
    let g = sample_graph();
    let dot = to_dot(&g, &DotOptions::default());
    assert!(dot.contains("x: Vec\\<i32\\>"));
    assert!(dot.contains("r: &Vec\\<i32\\>"));
}

#[test]
fn test_dot_contains_edge_labels() {
    let g = sample_graph();
    let dot = to_dot(&g, &DotOptions::default());
    assert!(dot.contains("label=\"&\"")); // shared borrow
    assert!(dot.contains("label=\"&mut\"")); // mutable borrow
}

#[test]
fn test_dot_show_types_false() {
    let g = sample_graph();
    let opts = DotOptions { show_types: false, ..Default::default() };
    let dot = to_dot(&g, &opts);
    assert!(!dot.contains("Vec\\<i32\\>"));
    assert!(dot.contains("\"x\"")); // just the name
}

#[test]
fn test_dot_show_timestamps() {
    let g = sample_graph();
    let opts = DotOptions { show_timestamps: true, ..Default::default() };
    let dot = to_dot(&g, &opts);
    assert!(dot.contains("@10")); // timestamp on edge
}

#[test]
fn test_dot_direction_lr() {
    let g = sample_graph();
    let opts = DotOptions { direction: DotDirection::LeftRight, ..Default::default() };
    let dot = to_dot(&g, &opts);
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn test_dot_edge_colors() {
    let g = sample_graph();
    let dot = to_dot(&g, &DotOptions::default());
    assert!(dot.contains("color=blue")); // shared borrow
    assert!(dot.contains("color=red")); // mutable borrow
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.3 MessagePack export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_msgpack_roundtrip() {
    let g = sample_graph();
    let bytes = to_msgpack(&g).unwrap();
    let restored = from_msgpack(&bytes).unwrap();

    assert_eq!(restored.node_count(), g.node_count());
    assert_eq!(restored.edge_count(), g.edge_count());
    assert_eq!(restored.find_by_name("x").len(), 1);
}

#[test]
fn test_msgpack_smaller_than_json() {
    let g = sample_graph();
    let json = to_json_compact(&g).unwrap();
    let msgpack = to_msgpack(&g).unwrap();

    assert!(msgpack.len() < json.len(),
        "MessagePack ({} bytes) should be smaller than JSON ({} bytes)",
        msgpack.len(), json.len());
}

#[test]
fn test_msgpack_file_roundtrip() {
    let g = sample_graph();
    let path = std::path::Path::new("/tmp/borrowscope_test_export.msgpack");
    to_msgpack_file(&g, path).unwrap();
    let restored = from_msgpack_file(path).unwrap();
    assert_eq!(restored.node_count(), g.node_count());
    std::fs::remove_file(path).ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.4 Delta export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_delta_empty_when_no_changes() {
    let g = sample_graph();
    let delta = compute_delta(&g, &g, 1);
    assert!(delta.is_empty());
}

#[test]
fn test_delta_detects_added_node() {
    let g1 = sample_graph();
    let mut g2 = g1.clone();
    g2.add_variable("new_var", "String", 200);

    let delta = compute_delta(&g1, &g2, 1);
    assert_eq!(delta.added_nodes.len(), 1);
    assert_eq!(delta.sequence, 1);
}

#[test]
fn test_delta_detects_dropped_node() {
    let mut g1 = OwnershipGraph::new();
    let x = g1.add_variable("x", "i32", 0);

    let mut g2 = g1.clone();
    g2.rebuild_indices();
    g2.mark_dropped(x, 50);

    let delta = compute_delta(&g1, &g2, 2);
    assert_eq!(delta.dropped_nodes.len(), 1);
    assert_eq!(delta.dropped_nodes[0], (x, 50));
}

#[test]
fn test_delta_detects_ended_edge() {
    let mut g1 = OwnershipGraph::new();
    let x = g1.add_variable("x", "i32", 0);
    let r = g1.add_variable("r", "&i32", 10);
    let eid = g1.add_borrow(r, x, false, 10);

    let mut g2 = g1.clone();
    g2.rebuild_indices();
    g2.end_edge(eid, 50);

    let delta = compute_delta(&g1, &g2, 3);
    assert_eq!(delta.ended_edges.len(), 1);
    assert_eq!(delta.ended_edges[0], (eid, 50));
}

#[test]
fn test_delta_sequence_numbers() {
    let g = sample_graph();
    let d1 = compute_delta(&g, &g, 1);
    let d2 = compute_delta(&g, &g, 2);
    assert_eq!(d1.sequence, 1);
    assert_eq!(d2.sequence, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.5 Import
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_import_malformed_json_error() {
    let result = from_json("not valid json {{{");
    assert!(result.is_err());
    match result.unwrap_err() {
        ImportError::ParseError(msg) => assert!(!msg.is_empty()),
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_import_preserves_indices() {
    let g = sample_graph();
    let json = to_json(&g).unwrap();
    let restored = from_json(&json).unwrap();

    // Verify indices work after import
    let x_nodes = restored.find_by_name("x");
    assert_eq!(x_nodes.len(), 1);
    let x = x_nodes[0];
    assert_eq!(restored.borrowers_of(x).len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.6 D3.js format
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_d3_has_nodes_and_links() {
    let g = sample_graph();
    let d3 = to_d3(&g);

    assert_eq!(d3.nodes.len(), g.node_count());
    assert_eq!(d3.links.len(), g.edge_count());
}

#[test]
fn test_d3_node_ids_valid() {
    let g = sample_graph();
    let d3 = to_d3(&g);

    let node_ids: std::collections::HashSet<usize> = d3.nodes.iter().map(|n| n.id).collect();
    for link in &d3.links {
        assert!(node_ids.contains(&link.source), "Link source {} not in nodes", link.source);
        assert!(node_ids.contains(&link.target), "Link target {} not in nodes", link.target);
    }
}

#[test]
fn test_d3_groups_consistent() {
    let mut g = OwnershipGraph::new();
    g.add_variable("rc1", "Rc<i32>", 0);
    g.add_variable("rc2", "Rc<i32>", 10);
    g.add_variable("x", "i32", 20);

    let d3 = to_d3(&g);
    let rc_groups: Vec<u32> = d3.nodes.iter()
        .filter(|n| n.name.starts_with("rc"))
        .map(|n| n.group)
        .collect();
    // Both Rc nodes should have the same group
    assert_eq!(rc_groups[0], rc_groups[1]);
    // Plain i32 should have a different group
    let x_group = d3.nodes.iter().find(|n| n.name == "x").unwrap().group;
    assert_ne!(x_group, rc_groups[0]);
}

#[test]
fn test_d3_json_parseable() {
    let g = sample_graph();
    let json = to_d3_json(&g).unwrap();

    // Should be valid JSON with nodes and links
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["nodes"].is_array());
    assert!(parsed["links"].is_array());
}

#[test]
fn test_d3_link_kinds() {
    let g = sample_graph();
    let d3 = to_d3(&g);

    let kinds: Vec<&str> = d3.links.iter().map(|l| l.kind.as_str()).collect();
    assert!(kinds.contains(&"borrow"));
    assert!(kinds.contains(&"borrow_mut"));
}

#[test]
fn test_d3_node_size_proportional() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let y = g.add_variable("y", "i32", 0);
    // x gets 5 borrows, y gets 0
    for i in 0..5 {
        let r = g.add_variable(&format!("r{}", i), "&i32", 10);
        g.add_borrow(r, x, false, 10);
    }

    let d3 = to_d3(&g);
    let x_size = d3.nodes.iter().find(|n| n.name == "x").unwrap().size;
    let y_size = d3.nodes.iter().find(|n| n.name == "y").unwrap().size;
    assert!(x_size > y_size);
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_export_empty_graph() {
    let g = OwnershipGraph::new();
    let json = to_json(&g).unwrap();
    let dot = to_dot(&g, &DotOptions::default());
    let msgpack = to_msgpack(&g).unwrap();
    let d3 = to_d3(&g);

    assert!(!json.is_empty());
    assert!(dot.contains("digraph"));
    assert!(!msgpack.is_empty());
    assert!(d3.nodes.is_empty());
    assert!(d3.links.is_empty());
}

#[test]
fn test_imported_graph_passes_validate() {
    let g = sample_graph();
    let json = to_json(&g).unwrap();
    let restored = from_json(&json).unwrap();
    assert!(borrowscope_graph::conflict::is_valid(&restored));
}

#[test]
fn test_apply_delta_drops_node() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);

    let delta = GraphDelta {
        sequence: 1,
        added_nodes: vec![],
        added_edges: vec![],
        dropped_nodes: vec![(x, 50)],
        ended_edges: vec![],
    };

    apply_delta(&mut g, &delta);
    assert_eq!(g.get_node(x).unwrap().end_time(), Some(50));
}

#[test]
fn test_msgpack_size_reduction_30_percent() {
    // Build a larger graph for meaningful size comparison
    let mut g = OwnershipGraph::new();
    let owner = g.add_variable("data", "Vec<i32>", 0);
    for i in 0..20u64 {
        let r = g.add_variable(&format!("ref_{}", i), "&Vec<i32>", i * 10);
        g.add_borrow(r, owner, false, i * 10);
    }

    let json = to_json_compact(&g).unwrap();
    let msgpack = to_msgpack(&g).unwrap();
    let reduction = 1.0 - (msgpack.len() as f64 / json.len() as f64);
    assert!(reduction > 0.30,
        "MessagePack should be at least 30% smaller than JSON. Got {:.0}% reduction (json={}, msgpack={})",
        reduction * 100.0, json.len(), msgpack.len());
}
