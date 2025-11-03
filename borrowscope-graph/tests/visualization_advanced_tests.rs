use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Complex Graph Visualization Tests
// ============================================================================

#[test]
fn test_large_graph_visualization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 * 100 + 500)
            } else {
                None
            },
            scope_depth: i % 5,
        });
    }

    for i in 2..=100 {
        if i % 3 == 0 {
            graph.add_borrow(i, i - 1, false, i as u64 * 100);
        }
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 100);
    assert!(viz.elements.edges.len() > 30);
}

#[test]
fn test_diamond_pattern_visualization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(4, 3, false, 400);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 4);
    assert_eq!(viz.elements.edges.len(), 4);
}

#[test]
fn test_mixed_relationship_types() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=6 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: match i {
                1 => "i32",
                2 => "&i32",
                3 => "&mut i32",
                4 => "Rc<i32>",
                5 => "Arc<i32>",
                6 => "RefCell<i32>",
                _ => "i32",
            }
            .into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 300);
    graph.add_rc_clone(4, 1, 400, 2);
    graph.add_arc_clone(5, 1, 500, 2);
    graph.add_refcell_borrow(6, 1, false, 600);

    let viz = graph.export_for_visualization();

    let relationships: Vec<_> = viz
        .elements
        .edges
        .iter()
        .map(|e| e.data.relationship.as_str())
        .collect();

    assert!(relationships.contains(&"immutable_borrow"));
    assert!(relationships.contains(&"mutable_borrow"));
    assert!(relationships.contains(&"rc_clone"));
    assert!(relationships.contains(&"arc_clone"));
    assert!(relationships.contains(&"refcell_immut"));
}

// ============================================================================
// Timeline Advanced Tests
// ============================================================================

#[test]
fn test_timeline_complex_lifecycle() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(5000),
        scope_depth: 0,
    });

    for i in 2..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&i32".into(),
            created_at: 1000 + i as u64 * 500,
            dropped_at: Some(1000 + i as u64 * 500 + 300),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, 1000 + i as u64 * 500);
    }

    let timeline = graph.export_timeline();

    assert!(timeline.len() >= 10);

    let mid_frame = timeline.iter().find(|f| f.timestamp == 2500).unwrap();

    assert!(mid_frame.elements.nodes.iter().any(|n| n.data.id == "1"));
}

#[test]
fn test_timeline_overlapping_lifetimes() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 500),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();

    let mid_frame = timeline.iter().find(|f| f.timestamp == 500).unwrap();

    let alive_count = mid_frame
        .elements
        .nodes
        .iter()
        .filter(|n| n.data.is_alive)
        .count();

    assert!(alive_count >= 4);
}

#[test]
fn test_timeline_edge_visibility() {
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
        created_at: 2000,
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 2000);

    let timeline = graph.export_timeline();

    let before_borrow = timeline.iter().find(|f| f.timestamp == 1000).unwrap();
    assert_eq!(before_borrow.elements.edges.len(), 0);

    let after_borrow = timeline.iter().find(|f| f.timestamp == 2000).unwrap();
    assert_eq!(after_borrow.elements.edges.len(), 1);
}

// ============================================================================
// D3 Export Advanced Tests
// ============================================================================

#[test]
fn test_d3_export_with_metadata() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Vec<String>".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 3,
    });

    let d3 = graph.export_for_d3();
    let metadata = d3.nodes[0].metadata.as_ref().unwrap();

    assert_eq!(metadata["type"], "Vec<String>");
    assert_eq!(metadata["created_at"], 1000);
    assert_eq!(metadata["dropped_at"], 2000);
}

#[test]
fn test_d3_export_link_metadata() {
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
        created_at: 1500,
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 1500);

    let d3 = graph.export_for_d3();
    let link_metadata = d3.links[0].metadata.as_ref().unwrap();

    assert_eq!(link_metadata["type"], "mutable");
    assert_eq!(link_metadata["at"], 1500);
}

#[test]
fn test_d3_export_grouping() {
    let mut graph = OwnershipGraph::new();

    for depth in 0..5 {
        for i in 0..3 {
            let id = depth * 10 + i + 1;
            graph.add_variable(Variable {
                id,
                name: format!("v{}_{}", depth, i),
                type_name: "i32".into(),
                created_at: id as u64 * 100,
                dropped_at: None,
                scope_depth: depth,
            });
        }
    }

    let d3 = graph.export_for_d3();

    for depth in 0..5 {
        let count = d3.nodes.iter().filter(|n| n.group == depth).count();
        assert_eq!(count, 3);
    }
}

// ============================================================================
// Highlight Advanced Tests
// ============================================================================

#[test]
fn test_highlight_transitive_borrowers() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=5 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 1);
}

#[test]
fn test_highlight_multiple_borrowers() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 9);
}

// ============================================================================
// Time-Based Visualization Tests
// ============================================================================

#[test]
fn test_visualization_at_specific_time() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let viz_before = graph.export_for_visualization_at(Some(1500));
    assert!(viz_before.elements.nodes[0].data.is_alive);

    let viz_after = graph.export_for_visualization_at(Some(2500));
    assert!(!viz_after.elements.nodes[0].data.is_alive);
}

#[test]
fn test_visualization_boundary_times() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let viz_at_creation = graph.export_for_visualization_at(Some(1000));
    assert!(viz_at_creation.elements.nodes[0].data.is_alive);

    let viz_at_drop = graph.export_for_visualization_at(Some(2000));
    assert!(!viz_at_drop.elements.nodes[0].data.is_alive);
}

// ============================================================================
// Style Customization Tests
// ============================================================================

#[test]
fn test_all_node_styles_present() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    let selectors: Vec<_> = viz.style.iter().map(|s| s.selector.as_str()).collect();

    assert!(selectors.contains(&"node"));
    assert!(selectors.contains(&"node.dropped"));
    assert!(selectors.contains(&"node.mutable-ref"));
    assert!(selectors.contains(&"node.immutable-ref"));
    assert!(selectors.contains(&"node.rc"));
    assert!(selectors.contains(&"node.arc"));
    assert!(selectors.contains(&"node.box"));
}

#[test]
fn test_all_edge_styles_present() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    let selectors: Vec<_> = viz.style.iter().map(|s| s.selector.as_str()).collect();

    assert!(selectors.contains(&"edge"));
    assert!(selectors.contains(&"edge.immutable"));
    assert!(selectors.contains(&"edge.mutable"));
    assert!(selectors.contains(&"edge.move"));
}

#[test]
fn test_style_values_valid() {
    let graph = OwnershipGraph::new();
    let viz = graph.export_for_visualization();

    for style in &viz.style {
        let obj = style.style.as_object().unwrap();
        assert!(!obj.is_empty());
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_large_graph_export_performance() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 + 1000)
            } else {
                None
            },
            scope_depth: i % 10,
        });
    }

    for i in 2..=1000 {
        if i % 5 == 0 {
            graph.add_borrow(i, i - 1, false, i as u64);
        }
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 1000);
}

#[test]
fn test_timeline_export_performance() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 500),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();
    assert!(timeline.len() >= 100);
    assert!(timeline.len() <= 200);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_visualization_with_unicode_names() {
    let mut graph = OwnershipGraph::new();

    let names = ["变量", "переменная", "μεταβλητή", "🦀"];
    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 4);

    let json = serde_json::to_string(&viz).unwrap();
    assert!(json.contains("变量"));
    assert!(json.contains("🦀"));
}

#[test]
fn test_visualization_with_complex_types() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "HashMap<String, Vec<Rc<RefCell<i32>>>>",
        "Arc<Mutex<Vec<Box<dyn Trait>>>>",
        "&'static [u8]",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 3);
}

#[test]
fn test_visualization_u64_max_timestamps() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization_at(Some(u64::MAX - 500));
    assert!(viz.elements.nodes[0].data.is_alive);
}
