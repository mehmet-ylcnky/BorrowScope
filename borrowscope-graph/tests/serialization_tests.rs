use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// JSON Serialization Tests
// ============================================================================

#[test]
fn test_json_roundtrip_empty_graph() {
    let graph = OwnershipGraph::new();
    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_json_roundtrip_single_variable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    let var = restored.get_variable(1).unwrap();
    assert_eq!(var.name, "x");
    assert_eq!(var.type_name, "i32");
    assert_eq!(var.created_at, 100);
    assert_eq!(var.dropped_at, Some(200));
}

#[test]
fn test_json_roundtrip_with_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());

    let borrowers = restored.borrowers_of(1);
    assert_eq!(borrowers.len(), 1);
    assert_eq!(borrowers[0].id, 2);
}

#[test]
fn test_json_roundtrip_all_relationship_types() {
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
    graph.add_rc_clone(5, 1, 2, 500);
    graph.add_arc_clone(6, 1, 3, 600);
    graph.add_refcell_borrow(7, 1, true, 700);

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_json_pretty_vs_compact() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let pretty = graph.to_json_pretty().unwrap();
    let compact = graph.to_json_compact().unwrap();

    assert!(pretty.len() > compact.len());
    assert!(pretty.contains('\n'));
    assert!(!compact.contains('\n'));
}

// ============================================================================
// MessagePack Serialization Tests
// ============================================================================

#[test]
fn test_messagepack_roundtrip_empty_graph() {
    let graph = OwnershipGraph::new();
    let data = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&data).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_messagepack_roundtrip_complex_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=20 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "Vec<String>".into(),
            created_at: i as u64 * 100,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 * 200)
            } else {
                None
            },
            scope_depth: i % 5,
        });
    }

    for i in 2..=20 {
        graph.add_borrow(i, i - 1, i % 3 == 0, i as u64 * 100);
    }

    let data = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&data).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_messagepack_smaller_than_json() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json_compact().unwrap();
    let msgpack = graph.to_messagepack().unwrap();

    assert!(msgpack.len() < json.len());
}

// ============================================================================
// DOT Format Tests
// ============================================================================

#[test]
fn test_dot_format_empty_graph() {
    let graph = OwnershipGraph::new();
    let dot = graph.to_dot();

    assert!(dot.contains("digraph OwnershipGraph"));
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn test_dot_format_single_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let dot = graph.to_dot();

    assert!(dot.contains("n1"));
    assert!(dot.contains("x"));
    assert!(dot.contains("i32"));
    assert!(dot.contains("@100"));
}

#[test]
fn test_dot_format_with_edges() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);

    let dot = graph.to_dot();

    assert!(dot.contains("n1 -> n2") || dot.contains("n2 -> n1"));
    assert!(dot.contains("&@200"));
}

#[test]
fn test_dot_format_all_relationship_types() {
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
    graph.add_rc_clone(5, 1, 2, 500);
    graph.add_arc_clone(6, 1, 3, 600);
    graph.add_refcell_borrow(7, 1, true, 700);

    let dot = graph.to_dot();

    assert!(dot.contains("&@200"));
    assert!(dot.contains("&mut@300"));
    assert!(dot.contains("move@400"));
    assert!(dot.contains("Rc(2)@500"));
    assert!(dot.contains("Arc(3)@600"));
    assert!(dot.contains("RefMut@700"));
}

#[test]
fn test_dot_format_dropped_nodes() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let dot = graph.to_dot();

    assert!(dot.contains("lightgray"));
    assert!(dot.contains("lightblue"));
}

// ============================================================================
// Special Characters and Edge Cases
// ============================================================================

#[test]
fn test_json_with_unicode_names() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "变量".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "переменная".into(),
        type_name: "String".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(restored.get_variable(1).unwrap().name, "变量");
    assert_eq!(restored.get_variable(2).unwrap().name, "переменная");
}

#[test]
fn test_json_with_special_characters() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "var\"with\\quotes".into(),
        type_name: "Type<T>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(restored.get_variable(1).unwrap().name, "var\"with\\quotes");
}

#[test]
fn test_json_with_empty_names() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "".into(),
        type_name: "".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(restored.get_variable(1).unwrap().name, "");
}

#[test]
fn test_json_with_very_long_names() {
    let mut graph = OwnershipGraph::new();

    let long_name = "a".repeat(10000);
    graph.add_variable(Variable {
        id: 1,
        name: long_name.clone(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(restored.get_variable(1).unwrap().name, long_name);
}
