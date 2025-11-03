use borrowscope_graph::{OwnershipGraph, Relationship, Variable};

// ============================================================================
// Active Borrows Tests
// ============================================================================

#[test]
fn test_active_borrows_at_no_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_active_borrows_at_single_immutable() {
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
        dropped_at: Some(1900),
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    // Before borrow
    assert_eq!(graph.active_borrows_at(1, 1050).len(), 0);

    // During borrow
    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].0.name, "r");
    assert!(matches!(active[0].1, Relationship::BorrowsImmut { .. }));

    // After borrow dropped
    assert_eq!(graph.active_borrows_at(1, 2000).len(), 0);
}

#[test]
fn test_active_borrows_at_multiple_immutable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: 1000 + i as u64 * 100,
            dropped_at: Some(2000),
            scope_depth: 0,
        });
        graph.add_borrow(i, 1, false, 1000 + i as u64 * 100);
    }

    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 4);
}

#[test]
fn test_active_borrows_at_mutable() {
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
        dropped_at: Some(1900),
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, true, 1100);

    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 1);
    assert!(matches!(active[0].1, Relationship::BorrowsMut { .. }));
}

#[test]
fn test_active_borrows_at_refcell() {
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
        name: "r".into(),
        type_name: "Ref<i32>".into(),
        created_at: 1100,
        dropped_at: Some(1900),
        scope_depth: 0,
    });

    graph.add_refcell_borrow(2, 1, false, 1100);

    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 1);
    assert!(matches!(
        active[0].1,
        Relationship::RefCellBorrow { is_mut: false, .. }
    ));
}

#[test]
fn test_active_borrows_at_overlapping_lifetimes() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // r1: 1100-1500
    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1500),
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);

    // r2: 1200-1600
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&i32".into(),
        created_at: 1200,
        dropped_at: Some(1600),
        scope_depth: 0,
    });
    graph.add_borrow(3, 1, false, 1200);

    // r3: 1300-1700
    graph.add_variable(Variable {
        id: 4,
        name: "r3".into(),
        type_name: "&i32".into(),
        created_at: 1300,
        dropped_at: Some(1700),
        scope_depth: 0,
    });
    graph.add_borrow(4, 1, false, 1300);

    // At 1050: none active
    assert_eq!(graph.active_borrows_at(1, 1050).len(), 0);

    // At 1150: r1 active
    assert_eq!(graph.active_borrows_at(1, 1150).len(), 1);

    // At 1250: r1, r2 active
    assert_eq!(graph.active_borrows_at(1, 1250).len(), 2);

    // At 1350: r1, r2, r3 active
    assert_eq!(graph.active_borrows_at(1, 1350).len(), 3);

    // At 1550: r2, r3 active
    assert_eq!(graph.active_borrows_at(1, 1550).len(), 2);

    // At 1650: r3 active
    assert_eq!(graph.active_borrows_at(1, 1650).len(), 1);

    // At 1750: none active
    assert_eq!(graph.active_borrows_at(1, 1750).len(), 0);
}

#[test]
fn test_active_borrows_at_ignores_moves() {
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

    graph.add_move(1, 2, 1100);

    // Moves should not appear in active_borrows_at
    assert_eq!(graph.active_borrows_at(1, 1500).len(), 0);
}

#[test]
fn test_active_borrows_at_ignores_rc_arc() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Rc<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "Rc<i32>".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_rc_clone(2, 1, 2, 1100);

    // Rc/Arc clones should not appear in active_borrows_at
    assert_eq!(graph.active_borrows_at(1, 1500).len(), 0);
}

#[test]
fn test_active_borrows_at_nonexistent_variable() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.active_borrows_at(999, 1500).len(), 0);
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_export_empty_graph() {
    let graph = OwnershipGraph::new();
    let export = graph.export();

    assert_eq!(export.nodes.len(), 0);
    assert_eq!(export.edges.len(), 0);
}

#[test]
fn test_export_single_variable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let export = graph.export();
    assert_eq!(export.nodes.len(), 1);
    assert_eq!(export.edges.len(), 0);
    assert_eq!(export.nodes[0].name, "x");
}

#[test]
fn test_export_with_borrow() {
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
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    let export = graph.export();
    assert_eq!(export.nodes.len(), 2);
    assert_eq!(export.edges.len(), 1);
    assert_eq!(export.edges[0].from_id, 2);
    assert_eq!(export.edges[0].to_id, 1);
    assert!(matches!(
        export.edges[0].relationship,
        Relationship::BorrowsImmut { .. }
    ));
}

#[test]
fn test_export_complex_graph() {
    let mut graph = OwnershipGraph::new();

    // Create multiple variables with different relationships
    for i in 1..=5 {
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
            scope_depth: i % 3,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 300);
    graph.add_move(4, 5, 400);

    let export = graph.export();
    assert_eq!(export.nodes.len(), 5);
    assert_eq!(export.edges.len(), 3);
}

#[test]
fn test_to_json() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    assert!(json.contains("\"name\": \"x\""));
    assert!(json.contains("\"type_name\": \"i32\""));
    assert!(json.contains("\"created_at\": 1000"));
}

#[test]
fn test_to_json_compact() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let compact = graph.to_json_compact().unwrap();
    let pretty = graph.to_json().unwrap();

    // Compact should be shorter (no whitespace)
    assert!(compact.len() < pretty.len());
    assert!(!compact.contains('\n'));
}

#[test]
fn test_json_roundtrip() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1900),
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    let export = graph.export();
    let json = serde_json::to_string(&export).unwrap();
    let loaded: borrowscope_graph::GraphExport = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.nodes.len(), 2);
    assert_eq!(loaded.edges.len(), 1);
    assert_eq!(loaded.nodes[0].name, "x");
    assert_eq!(loaded.nodes[1].name, "r");
}

#[test]
fn test_export_preserves_all_relationship_types() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=7 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
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
    graph.add_arc_clone(6, 1, 2, 600);
    graph.add_refcell_borrow(7, 1, false, 700);

    let export = graph.export();
    assert_eq!(export.edges.len(), 6);

    // Verify all relationship types are present
    let has_immut = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::BorrowsImmut { .. }));
    let has_mut = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::BorrowsMut { .. }));
    let has_move = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::Moves { .. }));
    let has_rc = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::RcClone { .. }));
    let has_arc = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::ArcClone { .. }));
    let has_refcell = export
        .edges
        .iter()
        .any(|e| matches!(e.relationship, Relationship::RefCellBorrow { .. }));

    assert!(has_immut);
    assert!(has_mut);
    assert!(has_move);
    assert!(has_rc);
    assert!(has_arc);
    assert!(has_refcell);
}

#[test]
fn test_export_with_unicode() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "变量".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    assert!(json.contains("变量"));

    let export: borrowscope_graph::GraphExport = serde_json::from_str(&json).unwrap();
    assert_eq!(export.nodes[0].name, "变量");
}

#[test]
fn test_export_large_graph() {
    let mut graph = OwnershipGraph::with_capacity(1000, 2000);

    for i in 0..1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..1000 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    let export = graph.export();
    assert_eq!(export.nodes.len(), 1000);
    assert_eq!(export.edges.len(), 999);

    // Verify JSON serialization works
    let json = graph.to_json_compact().unwrap();
    assert!(!json.is_empty());
}
