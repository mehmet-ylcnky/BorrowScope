use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Large Graph Serialization Tests
// ============================================================================

#[test]
fn test_large_graph_json_serialization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 * 200)
            } else {
                None
            },
            scope_depth: i % 10,
        });
    }

    for i in 2..=1000 {
        if i % 3 == 0 {
            graph.add_borrow(i, i - 1, i % 5 == 0, i as u64 * 100);
        }
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_large_graph_messagepack_serialization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let data = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&data).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
}

#[test]
fn test_dense_graph_serialization() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        for j in 1..i {
            if (i + j) % 7 == 0 {
                graph.add_borrow(i, j, false, i as u64 * 100);
            }
        }
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

// ============================================================================
// Delta Serialization Tests
// ============================================================================

#[test]
fn test_delta_empty_to_empty() {
    let graph1 = OwnershipGraph::new();
    let graph2 = OwnershipGraph::new();

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert!(delta.is_empty());
}

#[test]
fn test_delta_add_nodes() {
    let graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    graph2.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph2.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 2);
    assert_eq!(delta.removed_nodes.len(), 0);
    assert_eq!(delta.added_edges.len(), 0);
}

#[test]
fn test_delta_remove_nodes() {
    let mut graph1 = OwnershipGraph::new();
    let graph2 = OwnershipGraph::new();

    graph1.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 0);
    assert_eq!(delta.removed_nodes.len(), 1);
    assert_eq!(delta.removed_nodes[0], 1);
}

#[test]
fn test_delta_modify_nodes() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    graph1.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph2.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.modified_nodes.len(), 1);
    assert_eq!(delta.modified_nodes[0].id, 1);
    assert_eq!(delta.modified_nodes[0].dropped_at, Some(200));
}

#[test]
fn test_delta_add_edges() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    for i in 1..=2 {
        let var = Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        };
        graph1.add_variable(var.clone());
        graph2.add_variable(var);
    }

    graph2.add_borrow(2, 1, false, 200);

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_edges.len(), 1);
    assert_eq!(delta.removed_edges.len(), 0);
}

#[test]
fn test_delta_complex_changes() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    for i in 1..=5 {
        graph1.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..=3 {
        graph2.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i == 2 { Some(250) } else { None },
            scope_depth: 0,
        });
    }

    graph2.add_variable(Variable {
        id: 6,
        name: "v6".into(),
        type_name: "i32".into(),
        created_at: 600,
        dropped_at: None,
        scope_depth: 0,
    });

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 1);
    assert_eq!(delta.removed_nodes.len(), 2);
    assert_eq!(delta.modified_nodes.len(), 1);
}

// ============================================================================
// Format Comparison Tests
// ============================================================================

#[test]
fn test_json_vs_messagepack_size() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=500 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_with_long_name_{}", i),
            type_name: "Vec<HashMap<String, Vec<i32>>>".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: i % 10,
        });
    }

    let json_compact = graph.to_json_compact().unwrap();
    let msgpack = graph.to_messagepack().unwrap();

    assert!(msgpack.len() < json_compact.len());
    let ratio = json_compact.len() as f64 / msgpack.len() as f64;
    assert!(ratio > 1.2);
}

#[test]
fn test_json_pretty_readability() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let pretty = graph.to_json_pretty().unwrap();

    assert!(pretty.contains("  "));
    assert!(pretty.contains("{\n"));
    assert!(pretty.contains("\n}"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_json_parsing() {
    let invalid_json = "{invalid json}";
    let result = OwnershipGraph::from_json(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_incomplete_json_parsing() {
    let incomplete_json =
        r#"{"metadata":{"version":"0.1.0","node_count":1,"edge_count":0},"nodes":[{"id":1"#;
    let result = OwnershipGraph::from_json(incomplete_json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_messagepack_parsing() {
    let invalid_data = vec![0xFF, 0xFF, 0xFF];
    let result = OwnershipGraph::from_messagepack(&invalid_data);
    assert!(result.is_err());
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_metadata_includes_version() {
    let graph = OwnershipGraph::new();
    let export = graph.export_with_metadata();

    assert!(!export.metadata.version.is_empty());
    assert_eq!(export.metadata.node_count, 0);
    assert_eq!(export.metadata.edge_count, 0);
}

#[test]
fn test_metadata_counts_accurate() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=50 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let export = graph.export_with_metadata();

    assert_eq!(export.metadata.node_count, 50);
    assert_eq!(export.metadata.edge_count, 49);
}

// ============================================================================
// DOT Format Advanced Tests
// ============================================================================

#[test]
fn test_dot_format_large_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let dot = graph.to_dot();

    assert!(dot.contains("digraph OwnershipGraph"));
    assert!(dot.matches("->").count() >= 99);
}

#[test]
fn test_dot_format_with_unicode() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "变量".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let dot = graph.to_dot();

    assert!(dot.contains("变量"));
}

#[test]
fn test_dot_format_escaping() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "var\"with\\quotes".into(),
        type_name: "Type<T>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let dot = graph.to_dot();

    assert!(dot.contains("n1"));
}

// ============================================================================
// Roundtrip Consistency Tests
// ============================================================================

#[test]
fn test_multiple_roundtrips_json() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json1 = graph.to_json().unwrap();
    let restored1 = OwnershipGraph::from_json(&json1).unwrap();
    let json2 = restored1.to_json().unwrap();
    let restored2 = OwnershipGraph::from_json(&json2).unwrap();

    assert_eq!(graph.node_count(), restored2.node_count());
    assert_eq!(json1, json2);
}

#[test]
fn test_multiple_roundtrips_messagepack() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let data1 = graph.to_messagepack().unwrap();
    let restored1 = OwnershipGraph::from_messagepack(&data1).unwrap();
    let data2 = restored1.to_messagepack().unwrap();
    let restored2 = OwnershipGraph::from_messagepack(&data2).unwrap();

    assert_eq!(graph.node_count(), restored2.node_count());
}

#[test]
fn test_cross_format_consistency() {
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

    let json = graph.to_json().unwrap();
    let msgpack = graph.to_messagepack().unwrap();

    let from_json = OwnershipGraph::from_json(&json).unwrap();
    let from_msgpack = OwnershipGraph::from_messagepack(&msgpack).unwrap();

    assert_eq!(from_json.node_count(), from_msgpack.node_count());
    assert_eq!(from_json.edge_count(), from_msgpack.edge_count());
}
