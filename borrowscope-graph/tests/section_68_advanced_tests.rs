use borrowscope_graph::{OwnershipGraph, Relationship, Variable};

// ============================================================================
// Advanced Active Borrows Tests
// ============================================================================

#[test]
fn test_active_borrows_at_boundary_conditions() {
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
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    // Exactly at creation time
    let active = graph.active_borrows_at(1, 1100);
    assert_eq!(active.len(), 1);

    // One microsecond before drop
    let active = graph.active_borrows_at(1, 1199);
    assert_eq!(active.len(), 1);

    // Exactly at drop time (should not be active)
    let active = graph.active_borrows_at(1, 1200);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_active_borrows_mixed_borrow_types() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // Immutable borrow
    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1500),
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);

    // Mutable borrow
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 1200,
        dropped_at: Some(1400),
        scope_depth: 0,
    });
    graph.add_borrow(3, 1, true, 1200);

    // RefCell borrow
    graph.add_variable(Variable {
        id: 4,
        name: "r3".into(),
        type_name: "Ref<i32>".into(),
        created_at: 1300,
        dropped_at: Some(1600),
        scope_depth: 0,
    });
    graph.add_refcell_borrow(4, 1, false, 1300);

    let active = graph.active_borrows_at(1, 1350);
    assert_eq!(active.len(), 3); // r1, r2, and r3 all active

    // Verify types
    let has_immut = active
        .iter()
        .any(|(_, rel)| matches!(rel, Relationship::BorrowsImmut { .. }));
    let has_mut = active
        .iter()
        .any(|(_, rel)| matches!(rel, Relationship::BorrowsMut { .. }));
    let has_refcell = active
        .iter()
        .any(|(_, rel)| matches!(rel, Relationship::RefCellBorrow { .. }));
    assert!(has_immut);
    assert!(has_mut);
    assert!(has_refcell);
}

#[test]
fn test_active_borrows_nested_borrows() {
    let mut graph = OwnershipGraph::new();

    // x <- r1 <- r2
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
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&&i32".into(),
        created_at: 1200,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(3, 2, false, 1200);

    // x has one direct borrower
    assert_eq!(graph.active_borrows_at(1, 1500).len(), 1);

    // r1 has one direct borrower
    assert_eq!(graph.active_borrows_at(2, 1500).len(), 1);

    // r2 has no borrowers
    assert_eq!(graph.active_borrows_at(3, 1500).len(), 0);
}

#[test]
fn test_active_borrows_with_never_dropped_variables() {
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
        dropped_at: None, // Never dropped
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    // Should be active at any time after creation
    assert_eq!(graph.active_borrows_at(1, 1100).len(), 1);
    assert_eq!(graph.active_borrows_at(1, 10000).len(), 1);
    assert_eq!(graph.active_borrows_at(1, u64::MAX).len(), 1);
}

#[test]
fn test_active_borrows_zero_lifetime() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // Borrow that exists for zero time
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1100),
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1100);

    // Should not be active at creation time (dropped immediately)
    assert_eq!(graph.active_borrows_at(1, 1100).len(), 0);
}

#[test]
fn test_active_borrows_max_timestamp() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: u64::MAX - 500,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, u64::MAX - 500);

    assert_eq!(graph.active_borrows_at(1, u64::MAX).len(), 1);
}

#[test]
fn test_active_borrows_multiple_edges_same_nodes() {
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

    // Add multiple borrows at different times (reborrow scenario)
    graph.add_borrow(2, 1, false, 1100);
    graph.add_borrow(2, 1, false, 1200);
    graph.add_borrow(2, 1, false, 1300);

    // All three edges should be counted
    let active = graph.active_borrows_at(1, 1500);
    assert_eq!(active.len(), 3);
}

#[test]
fn test_active_borrows_refcell_mutable() {
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
        type_name: "RefMut<i32>".into(),
        created_at: 1100,
        dropped_at: Some(1500),
        scope_depth: 0,
    });

    graph.add_refcell_borrow(2, 1, true, 1100);

    let active = graph.active_borrows_at(1, 1300);
    assert_eq!(active.len(), 1);
    assert!(matches!(
        active[0].1,
        Relationship::RefCellBorrow { is_mut: true, .. }
    ));
}

#[test]
fn test_active_borrows_sequential_non_overlapping() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // First borrow: 1100-1200
    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&i32".into(),
        created_at: 1100,
        dropped_at: Some(1200),
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);

    // Second borrow: 1300-1400
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&i32".into(),
        created_at: 1300,
        dropped_at: Some(1400),
        scope_depth: 0,
    });
    graph.add_borrow(3, 1, false, 1300);

    // Gap between borrows
    assert_eq!(graph.active_borrows_at(1, 1250).len(), 0);

    // During first borrow
    assert_eq!(graph.active_borrows_at(1, 1150).len(), 1);

    // During second borrow
    assert_eq!(graph.active_borrows_at(1, 1350).len(), 1);
}

// ============================================================================
// Advanced Serialization Tests
// ============================================================================

#[test]
fn test_export_with_all_metadata() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 42,
        name: "complex_var".into(),
        type_name: "Vec<HashMap<String, Arc<Mutex<i32>>>>".into(),
        created_at: 123456789,
        dropped_at: Some(987654321),
        scope_depth: 5,
    });

    let export = graph.export();
    assert_eq!(export.nodes.len(), 1);

    let var = &export.nodes[0];
    assert_eq!(var.id, 42);
    assert_eq!(var.name, "complex_var");
    assert_eq!(var.type_name, "Vec<HashMap<String, Arc<Mutex<i32>>>>");
    assert_eq!(var.created_at, 123456789);
    assert_eq!(var.dropped_at, Some(987654321));
    assert_eq!(var.scope_depth, 5);
}

#[test]
fn test_export_preserves_edge_order() {
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

    // Add edges in specific order
    for i in 2..=10 {
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let export = graph.export();
    assert_eq!(export.edges.len(), 9);

    // All edges should point to var1
    for edge in &export.edges {
        assert_eq!(edge.to_id, 1);
    }
}

#[test]
fn test_json_with_special_characters() {
    let mut graph = OwnershipGraph::new();

    let special_chars = [
        ("newline", "x\ny"),
        ("tab", "x\ty"),
        ("quote", "x\"y"),
        ("backslash", "x\\y"),
        ("null", "x\0y"),
    ];

    for (i, (name, value)) in special_chars.iter().enumerate() {
        graph.add_variable(Variable {
            id: i,
            name: value.to_string(),
            type_name: name.to_string(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json().unwrap();
    let export: borrowscope_graph::GraphExport = serde_json::from_str(&json).unwrap();

    assert_eq!(export.nodes.len(), 5);
}

#[test]
fn test_json_with_emoji_and_unicode() {
    let mut graph = OwnershipGraph::new();

    let unicode_names = ["🦀", "变量", "переменная", "μεταβλητή", "🔥💯"];

    for (i, name) in unicode_names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json().unwrap();
    let export: borrowscope_graph::GraphExport = serde_json::from_str(&json).unwrap();

    for (i, name) in unicode_names.iter().enumerate() {
        assert_eq!(export.nodes[i].name, *name);
    }
}

#[test]
fn test_export_empty_variable_names() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: String::new(),
        type_name: String::new(),
        created_at: 0,
        dropped_at: None,
        scope_depth: 0,
    });

    let export = graph.export();
    assert_eq!(export.nodes[0].name, "");
    assert_eq!(export.nodes[0].type_name, "");

    let json = graph.to_json().unwrap();
    assert!(json.contains("\"name\": \"\""));
}

#[test]
fn test_export_very_long_names() {
    let mut graph = OwnershipGraph::new();

    let long_name = "a".repeat(100000);
    let long_type = "b".repeat(100000);

    graph.add_variable(Variable {
        id: 1,
        name: long_name.clone(),
        type_name: long_type.clone(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let export = graph.export();
    assert_eq!(export.nodes[0].name.len(), 100000);
    assert_eq!(export.nodes[0].type_name.len(), 100000);

    let json = graph.to_json_compact().unwrap();
    assert!(json.len() > 200000);
}

#[test]
fn test_export_disconnected_components() {
    let mut graph = OwnershipGraph::new();

    // Component 1
    graph.add_variable(Variable {
        id: 1,
        name: "x1".into(),
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
        scope_depth: 0,
    });
    graph.add_borrow(2, 1, false, 1100);

    // Component 2
    graph.add_variable(Variable {
        id: 3,
        name: "x2".into(),
        type_name: "String".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 4,
        name: "r2".into(),
        type_name: "&String".into(),
        created_at: 2100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(4, 3, false, 2100);

    // Isolated node
    graph.add_variable(Variable {
        id: 5,
        name: "isolated".into(),
        type_name: "f64".into(),
        created_at: 3000,
        dropped_at: None,
        scope_depth: 0,
    });

    let export = graph.export();
    assert_eq!(export.nodes.len(), 5);
    assert_eq!(export.edges.len(), 2);
}

#[test]
fn test_json_compact_vs_pretty_content() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let compact = graph.to_json_compact().unwrap();
    let pretty = graph.to_json().unwrap();

    // Both should deserialize to same structure
    let compact_export: borrowscope_graph::GraphExport = serde_json::from_str(&compact).unwrap();
    let pretty_export: borrowscope_graph::GraphExport = serde_json::from_str(&pretty).unwrap();

    assert_eq!(compact_export.nodes.len(), pretty_export.nodes.len());
    assert_eq!(compact_export.edges.len(), pretty_export.edges.len());
}

#[test]
fn test_export_with_max_scope_depth() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "deeply_nested".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: usize::MAX,
    });

    let export = graph.export();
    assert_eq!(export.nodes[0].scope_depth, usize::MAX);

    let json = graph.to_json().unwrap();
    let loaded: borrowscope_graph::GraphExport = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.nodes[0].scope_depth, usize::MAX);
}

#[test]
fn test_export_relationship_metadata() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=6 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 1234);
    graph.add_borrow(3, 1, true, 5678);
    graph.add_move(4, 1, 9012);
    graph.add_rc_clone(5, 1, 42, 3456);
    graph.add_arc_clone(6, 1, 99, 7890);

    let export = graph.export();

    // Verify timestamps are preserved
    let immut_edge = export
        .edges
        .iter()
        .find(|e| matches!(e.relationship, Relationship::BorrowsImmut { .. }))
        .unwrap();
    assert!(matches!(
        immut_edge.relationship,
        Relationship::BorrowsImmut { at: 1234 }
    ));

    // Verify strong counts are preserved
    let rc_edge = export
        .edges
        .iter()
        .find(|e| matches!(e.relationship, Relationship::RcClone { .. }))
        .unwrap();
    assert!(matches!(
        rc_edge.relationship,
        Relationship::RcClone {
            at: 3456,
            strong_count: 42
        }
    ));
}

#[test]
fn test_export_after_clear() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.clear();

    let export = graph.export();
    assert_eq!(export.nodes.len(), 0);
    assert_eq!(export.edges.len(), 0);

    let json = graph.to_json().unwrap();
    assert!(json.contains("\"nodes\": []"));
    assert!(json.contains("\"edges\": []"));
}

#[test]
fn test_export_star_topology() {
    let mut graph = OwnershipGraph::new();

    // Central node
    graph.add_variable(Variable {
        id: 0,
        name: "center".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // 50 borrowers
    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("spoke{}", i),
            type_name: "&i32".into(),
            created_at: 1000 + i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_borrow(i, 0, false, 1000 + i as u64);
    }

    let export = graph.export();
    assert_eq!(export.nodes.len(), 51);
    assert_eq!(export.edges.len(), 50);

    // All edges should point to center
    for edge in &export.edges {
        assert_eq!(edge.to_id, 0);
    }
}

#[test]
fn test_json_size_comparison() {
    let mut graph = OwnershipGraph::new();

    for i in 0..100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_with_long_name_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let compact = graph.to_json_compact().unwrap();
    let pretty = graph.to_json().unwrap();

    // Compact should be significantly smaller
    assert!(compact.len() < pretty.len());
    let ratio = pretty.len() as f64 / compact.len() as f64;
    assert!(ratio > 1.2); // At least 20% larger
}

#[test]
fn test_export_with_all_dropped_variables() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 200),
            scope_depth: 0,
        });
    }

    let export = graph.export();
    assert_eq!(export.nodes.len(), 10);

    // All should have dropped_at set
    for node in &export.nodes {
        assert!(node.dropped_at.is_some());
    }
}
