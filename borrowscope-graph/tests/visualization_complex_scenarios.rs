use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Complex Graph Topology Tests
// ============================================================================

#[test]
fn test_visualization_complete_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=20 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=20 {
        for j in 1..i {
            graph.add_borrow(i, j, false, i as u64 * 100);
        }
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 20);
    assert_eq!(viz.elements.edges.len(), 190);
}

#[test]
fn test_visualization_star_topology() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "center".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=101 {
        graph.add_variable(Variable {
            id: i,
            name: format!("spoke{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 10,
            dropped_at: None,
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 10);
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 101);
    assert_eq!(viz.elements.edges.len(), 100);
}

#[test]
fn test_visualization_binary_tree() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=127 {
        graph.add_variable(Variable {
            id: i,
            name: format!("node{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: (i as f64).log2() as usize,
        });
    }

    for i in 2..=127 {
        graph.add_borrow(i, i / 2, false, i as u64 * 100);
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 127);
    assert_eq!(viz.elements.edges.len(), 126);
}

#[test]
fn test_visualization_disconnected_components() {
    let mut graph = OwnershipGraph::new();

    for component in 0..10 {
        for i in 0..10 {
            let id = component * 10 + i + 1;
            graph.add_variable(Variable {
                id,
                name: format!("c{}v{}", component, i),
                type_name: "i32".into(),
                created_at: id as u64 * 100,
                dropped_at: None,
                scope_depth: component,
            });
        }

        for i in 1..10 {
            let id = component * 10 + i + 1;
            graph.add_borrow(id, id - 1, false, id as u64 * 100);
        }
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 100);
    assert_eq!(viz.elements.edges.len(), 90);
}

// ============================================================================
// Mixed Relationship Complexity Tests
// ============================================================================

#[test]
fn test_visualization_all_relationship_types_on_one_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "owner".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "immut_ref".into(),
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(2, 1, false, 200);

    graph.add_variable(Variable {
        id: 3,
        name: "mut_ref".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 1,
    });
    graph.add_borrow(3, 1, true, 300);

    graph.add_variable(Variable {
        id: 4,
        name: "moved".into(),
        type_name: "i32".into(),
        created_at: 400,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_move(4, 1, 400);

    graph.add_variable(Variable {
        id: 5,
        name: "rc_clone".into(),
        type_name: "Rc<i32>".into(),
        created_at: 500,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_rc_clone(5, 1, 500, 2);

    graph.add_variable(Variable {
        id: 6,
        name: "arc_clone".into(),
        type_name: "Arc<i32>".into(),
        created_at: 600,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_arc_clone(6, 1, 600, 2);

    graph.add_variable(Variable {
        id: 7,
        name: "refcell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 700,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_refcell_borrow(7, 1, false, 700);

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.edges.len(), 6);

    let relationships: Vec<_> = viz
        .elements
        .edges
        .iter()
        .map(|e| e.data.relationship.as_str())
        .collect();

    assert!(relationships.contains(&"immutable_borrow"));
    assert!(relationships.contains(&"mutable_borrow"));
    assert!(relationships.contains(&"move"));
    assert!(relationships.contains(&"rc_clone"));
    assert!(relationships.contains(&"arc_clone"));
    assert!(relationships.contains(&"refcell_immut"));
}

#[test]
fn test_visualization_cascading_borrows() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: if i == 1 { "i32" } else { "&i32" }.into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: i - 1,
        });
    }

    for i in 2..=100 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 100);
    assert_eq!(viz.elements.edges.len(), 99);
}

#[test]
fn test_visualization_multiple_mutable_borrows_over_time() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=11 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&mut i32".into(),
            created_at: i as u64 * 1000,
            dropped_at: Some(i as u64 * 1000 + 500),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, true, i as u64 * 1000);
    }

    let timeline = graph.export_timeline();

    for frame in &timeline {
        let mut_borrows = frame
            .elements
            .edges
            .iter()
            .filter(|e| e.data.relationship == "mutable_borrow")
            .count();
        assert!(mut_borrows <= 1);
    }
}

// ============================================================================
// D3 Export Complex Tests
// ============================================================================

#[test]
fn test_d3_export_large_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: if i % 3 == 0 {
                Some(i as u64 + 1000)
            } else {
                None
            },
            scope_depth: i % 50,
        });
    }

    for i in 2..=1000 {
        if i % 5 == 0 {
            graph.add_borrow(i, i - 1, false, i as u64);
        }
    }

    let d3 = graph.export_for_d3();
    assert_eq!(d3.nodes.len(), 1000);
    assert!(d3.links.len() > 190);
}

#[test]
fn test_d3_export_all_metadata_fields() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "complex_var".into(),
        type_name: "HashMap<String, Vec<Rc<RefCell<i32>>>>".into(),
        created_at: 12345,
        dropped_at: Some(67890),
        scope_depth: 42,
    });

    let d3 = graph.export_for_d3();
    let metadata = d3.nodes[0].metadata.as_ref().unwrap();

    assert_eq!(metadata["type"], "HashMap<String, Vec<Rc<RefCell<i32>>>>");
    assert_eq!(metadata["created_at"], 12345);
    assert_eq!(metadata["dropped_at"], 67890);
}

#[test]
fn test_d3_export_link_types() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=7 {
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
    graph.add_borrow(3, 1, true, 300);
    graph.add_move(4, 1, 400);
    graph.add_rc_clone(5, 1, 500, 2);
    graph.add_arc_clone(6, 1, 600, 2);
    graph.add_refcell_borrow(7, 1, false, 700);

    let d3 = graph.export_for_d3();

    let types: Vec<_> = d3
        .links
        .iter()
        .filter_map(|l| l.metadata.as_ref())
        .map(|m| m["type"].as_str().unwrap())
        .collect();

    assert_eq!(types.len(), 6);
    assert!(types.contains(&"immutable"));
    assert!(types.contains(&"mutable"));
    assert!(types.contains(&"move"));
    assert!(types.contains(&"rc"));
    assert!(types.contains(&"arc"));
    assert!(types.contains(&"refcell"));
}

// ============================================================================
// Highlight Complex Tests
// ============================================================================

#[test]
fn test_highlight_deep_borrow_chain() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=1000 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 1);
}

#[test]
fn test_highlight_fan_out() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "center".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=501 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 10,
            dropped_at: None,
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 10);
    }

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 500);
}

#[test]
fn test_highlight_diamond_pattern() {
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

    let highlight = graph.highlight_borrowers(1).unwrap();
    assert_eq!(highlight.highlight_path.as_ref().unwrap().len(), 2);
}

// ============================================================================
// Style and Layout Edge Cases
// ============================================================================

#[test]
fn test_all_layout_algorithms_with_large_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let layouts = [
        borrowscope_graph::LayoutConfig::dagre(),
        borrowscope_graph::LayoutConfig::cola(),
        borrowscope_graph::LayoutConfig::circular(),
        borrowscope_graph::LayoutConfig::grid(),
        borrowscope_graph::LayoutConfig::breadthfirst(),
    ];

    for layout in layouts {
        assert!(layout.options.is_some());
    }
}

#[test]
fn test_node_classes_all_combinations() {
    let mut graph = OwnershipGraph::new();

    let test_cases = [
        ("i32", "alive owned"),
        ("&i32", "alive immutable-ref"),
        ("&mut i32", "alive mutable-ref"),
        ("Rc<i32>", "alive owned rc"),
        ("Arc<i32>", "alive owned arc"),
        ("Box<i32>", "alive owned box"),
        ("RefCell<i32>", "alive owned refcell"),
        ("Rc<RefCell<i32>>", "alive owned rc"), // Only first match
        ("Arc<Mutex<i32>>", "alive owned arc"),
    ];

    for (i, (ty, _expected_classes)) in test_cases.iter().enumerate() {
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

    for (i, (_, expected)) in test_cases.iter().enumerate() {
        let classes = viz.elements.nodes[i].classes.as_ref().unwrap();
        for class in expected.split_whitespace() {
            assert!(
                classes.contains(class),
                "Expected '{}' to contain '{}' but got '{}'",
                expected,
                class,
                classes
            );
        }
    }
}

// ============================================================================
// Serialization Stress Tests
// ============================================================================

#[test]
fn test_visualization_json_roundtrip_large() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_{}", i),
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

    let viz = graph.export_for_visualization();
    let json = serde_json::to_string(&viz).unwrap();
    let deserialized: borrowscope_graph::VisualizationExport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.elements.nodes.len(), 1000);
}

#[test]
fn test_d3_json_serialization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: i % 5,
        });
    }

    let d3 = graph.export_for_d3();
    let json = serde_json::to_string(&d3).unwrap();

    assert!(json.contains("\"nodes\""));
    assert!(json.contains("\"links\""));
    assert!(json.len() > 1000);
}

#[test]
fn test_timeline_json_serialization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
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
    let json = serde_json::to_string(&timeline).unwrap();

    assert!(json.len() > 1000);
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"elements\""));
}
