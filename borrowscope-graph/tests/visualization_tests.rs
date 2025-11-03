use borrowscope_graph::{LayoutConfig, OwnershipGraph, Variable};

// ============================================================================
// Basic Visualization Export Tests
// ============================================================================

#[test]
fn test_empty_graph_visualization() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    assert!(viz.elements.nodes.is_empty());
    assert!(viz.elements.edges.is_empty());
    assert!(!viz.style.is_empty());
    assert_eq!(viz.layout.name, "dagre");
}

#[test]
fn test_single_node_visualization() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 1);
    assert_eq!(viz.elements.nodes[0].data.label, "x");
    assert_eq!(viz.elements.nodes[0].data.type_name, "i32");
    assert!(viz.elements.nodes[0].data.is_alive);
}

#[test]
fn test_node_with_borrow_visualization() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1200),
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 1100);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 2);
    assert_eq!(viz.elements.edges.len(), 1);
    assert_eq!(viz.elements.edges[0].data.relationship, "immutable_borrow");
    assert_eq!(viz.elements.edges[0].data.source, "2");
    assert_eq!(viz.elements.edges[0].data.target, "1");
}

// ============================================================================
// Node Classification Tests
// ============================================================================

#[test]
fn test_node_classes_alive() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("alive"));
    assert!(classes.contains("owned"));
}

#[test]
fn test_node_classes_dropped() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization_at(Some(2500));
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("dropped"));
}

#[test]
fn test_node_classes_mutable_ref() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "r".into(),
        type_name: "&mut i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("mutable-ref"));
}

#[test]
fn test_node_classes_immutable_ref() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("immutable-ref"));
}

#[test]
fn test_node_classes_rc() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "rc".into(),
        type_name: "Rc<String>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("rc"));
}

#[test]
fn test_node_classes_arc() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "arc".into(),
        type_name: "Arc<Vec<i32>>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("arc"));
}

#[test]
fn test_node_classes_refcell() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("refcell"));
}

#[test]
fn test_node_classes_box() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "boxed".into(),
        type_name: "Box<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let classes = viz.elements.nodes[0].classes.as_ref().unwrap();
    assert!(classes.contains("box"));
}

// ============================================================================
// Edge Classification Tests
// ============================================================================

#[test]
fn test_edge_immutable_borrow() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 1100);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "immutable_borrow");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "immutable");
}

#[test]
fn test_edge_mutable_borrow() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&mut i32".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, true, 1100);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "mutable_borrow");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "mutable");
}

#[test]
fn test_edge_move() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "String".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "String".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_move(2, 1, 1100);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "move");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "move");
}

#[test]
fn test_edge_rc_clone() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "rc1".into(),
        type_name: "Rc<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "rc2".into(),
        type_name: "Rc<i32>".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_rc_clone(2, 1, 1100, 2);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "rc_clone");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "rc");
    assert!(viz.elements.edges[0].data.extra.is_some());
}

#[test]
fn test_edge_arc_clone() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "arc1".into(),
        type_name: "Arc<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "arc2".into(),
        type_name: "Arc<i32>".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_arc_clone(2, 1, 1100, 2);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "arc_clone");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "arc");
}

#[test]
fn test_edge_refcell_borrow() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "ref".into(),
        type_name: "Ref<i32>".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_refcell_borrow(2, 1, false, 1100);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges[0].data.relationship, "refcell_immut");
    assert_eq!(viz.elements.edges[0].classes.as_ref().unwrap(), "refcell");
}

// ============================================================================
// Style Tests
// ============================================================================

#[test]
fn test_default_styles_present() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    assert!(!viz.style.is_empty());

    let selectors: Vec<_> = viz.style.iter().map(|s| s.selector.as_str()).collect();
    assert!(selectors.contains(&"node"));
    assert!(selectors.contains(&"edge"));
    assert!(selectors.contains(&"node.dropped"));
    assert!(selectors.contains(&"node.mutable-ref"));
    assert!(selectors.contains(&"edge.immutable"));
    assert!(selectors.contains(&"edge.mutable"));
}

#[test]
fn test_style_structure() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    for style in &viz.style {
        assert!(!style.selector.is_empty());
        assert!(style.style.is_object());
    }
}

// ============================================================================
// Layout Configuration Tests
// ============================================================================

#[test]
fn test_dagre_layout() {
    let layout = LayoutConfig::dagre();
    assert_eq!(layout.name, "dagre");
    assert!(layout.options.is_some());
}

#[test]
fn test_cola_layout() {
    let layout = LayoutConfig::cola();
    assert_eq!(layout.name, "cola");
    assert!(layout.options.is_some());
}

#[test]
fn test_circular_layout() {
    let layout = LayoutConfig::circular();
    assert_eq!(layout.name, "circle");
    assert!(layout.options.is_some());
}

#[test]
fn test_grid_layout() {
    let layout = LayoutConfig::grid();
    assert_eq!(layout.name, "grid");
    assert!(layout.options.is_some());
}

#[test]
fn test_breadthfirst_layout() {
    let layout = LayoutConfig::breadthfirst();
    assert_eq!(layout.name, "breadthfirst");
    assert!(layout.options.is_some());
}

// ============================================================================
// Tooltip Tests
// ============================================================================

#[test]
fn test_tooltip_alive_variable() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 2,
    });

    let viz = graph.export_for_visualization();
    let tooltip = viz.elements.nodes[0].data.tooltip();

    assert_eq!(tooltip.title, "x");
    assert!(tooltip
        .details
        .iter()
        .any(|(k, v)| k == "Type" && v == "i32"));
    assert!(tooltip
        .details
        .iter()
        .any(|(k, v)| k == "Status" && v == "Alive"));
    assert!(tooltip
        .details
        .iter()
        .any(|(k, v)| k == "Scope Depth" && v == "2"));
}

#[test]
fn test_tooltip_dropped_variable() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "String".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let tooltip = viz.elements.nodes[0].data.tooltip();

    assert!(tooltip.details.iter().any(|(k, _)| k == "Dropped"));
    assert!(tooltip.details.iter().any(|(k, _)| k == "Lifetime"));
}

// ============================================================================
// Timeline Export Tests
// ============================================================================

#[test]
fn test_timeline_empty_graph() {
    let graph = OwnershipGraph::new();
    let timeline = graph.export_timeline();
    assert!(timeline.is_empty());
}

#[test]
fn test_timeline_single_variable() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let timeline = graph.export_timeline();
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].timestamp, 1000);
    assert_eq!(timeline[1].timestamp, 2000);
}

#[test]
fn test_timeline_with_borrows() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1500,
        dropped_at: Some(2000),
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 1500);

    let timeline = graph.export_timeline();
    assert!(timeline.len() >= 3);
    assert!(timeline.iter().any(|f| f.timestamp == 1000));
    assert!(timeline.iter().any(|f| f.timestamp == 1500));
    assert!(timeline.iter().any(|f| f.timestamp == 2000));
}

#[test]
fn test_timeline_frame_content() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let timeline = graph.export_timeline();

    assert_eq!(timeline[0].elements.nodes.len(), 1);
    assert!(timeline[0].elements.nodes[0].data.is_alive);

    assert_eq!(timeline[1].elements.nodes.len(), 1);
    assert!(!timeline[1].elements.nodes[0].data.is_alive);
}

// ============================================================================
// D3.js Export Tests
// ============================================================================

#[test]
fn test_d3_export_empty() {
    let graph = OwnershipGraph::new();
    let d3 = graph.export_for_d3();

    assert!(d3.nodes.is_empty());
    assert!(d3.links.is_empty());
}

#[test]
fn test_d3_export_nodes() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 2,
    });

    let d3 = graph.export_for_d3();
    assert_eq!(d3.nodes.len(), 1);
    assert_eq!(d3.nodes[0].id, "1");
    assert_eq!(d3.nodes[0].label, "x");
    assert_eq!(d3.nodes[0].group, 2);
    assert!(d3.nodes[0].metadata.is_some());
}

#[test]
fn test_d3_export_links() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 1100);

    let d3 = graph.export_for_d3();
    assert_eq!(d3.links.len(), 1);
    assert_eq!(d3.links[0].source, "2");
    assert_eq!(d3.links[0].target, "1");
    assert_eq!(d3.links[0].value, 1);
    assert!(d3.links[0].metadata.is_some());
}

// ============================================================================
// Highlight Tests
// ============================================================================

#[test]
fn test_highlight_borrowers() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&i32".into(),
        created_at: 1200,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 1100);
    graph.add_borrow(3, 1, false, 1200);

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.node_id, "1");
    assert!(highlight.highlight_neighbors);
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 2);
}

#[test]
fn test_highlight_nonexistent() {
    let graph = OwnershipGraph::new();
    let highlight = graph.highlight_borrowers(999);
    assert!(highlight.is_none());
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_visualization_json_serialization() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    let json = serde_json::to_string(&viz).unwrap();

    assert!(json.contains("\"label\":\"x\""));
    assert!(json.contains("\"type\":\"i32\""));
}

#[test]
fn test_d3_json_serialization() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let d3 = graph.export_for_d3();
    let json = serde_json::to_string(&d3).unwrap();

    assert!(json.contains("\"nodes\""));
    assert!(json.contains("\"links\""));
}
