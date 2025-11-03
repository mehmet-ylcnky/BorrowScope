use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Basic Construction Tests
// ============================================================================

#[test]
fn test_empty_graph_properties() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.all_variables().next().is_none());
}

#[test]
fn test_graph_with_capacity() {
    let graph = OwnershipGraph::with_capacity(1000, 2000);
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_single_variable() {
    let mut graph = OwnershipGraph::new();
    let var = Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    };

    graph.add_variable(var.clone());

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.get_variable(1), Some(&var));
}

#[test]
fn test_duplicate_variable_id_overwrites() {
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
        id: 1,
        name: "y".into(),
        type_name: "String".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 1,
    });

    // Should have 2 nodes but same ID maps to latest
    assert_eq!(graph.node_count(), 2);
    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.name, "y");
}

// ============================================================================
// Borrow Tests
// ============================================================================

#[test]
fn test_simple_immutable_borrow() {
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

    let edge = graph.add_borrow(2, 1, false, 1100);
    assert!(edge.is_some());
    assert_eq!(graph.edge_count(), 1);

    let borrowers = graph.borrowers_of(1);
    assert_eq!(borrowers.len(), 1);
    assert_eq!(borrowers[0].name, "r");
}

#[test]
fn test_simple_mutable_borrow() {
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
        scope_depth: 0,
    });

    let edge = graph.add_borrow(2, 1, true, 1100);
    assert!(edge.is_some());
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_multiple_immutable_borrows() {
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
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: 1000 + i as u64 * 10,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_borrow(i, 1, false, 1000 + i as u64 * 10);
    }

    assert_eq!(graph.node_count(), 10);
    assert_eq!(graph.edge_count(), 9);

    let borrowers = graph.borrowers_of(1);
    assert_eq!(borrowers.len(), 9);
}

#[test]
fn test_nested_borrows() {
    let mut graph = OwnershipGraph::new();

    // x -> r1 -> r2 -> r3
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

    graph.add_variable(Variable {
        id: 4,
        name: "r3".into(),
        type_name: "&&&i32".into(),
        created_at: 1300,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(4, 3, false, 1300);

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 3);

    assert_eq!(graph.borrowers_of(1).len(), 1);
    assert_eq!(graph.borrowers_of(2).len(), 1);
    assert_eq!(graph.borrowers_of(3).len(), 1);
}

#[test]
fn test_borrow_with_missing_borrower() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_borrow(999, 1, false, 1100);
    assert!(edge.is_none());
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_borrow_with_missing_owner() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_borrow(1, 999, false, 1100);
    assert!(edge.is_none());
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_self_borrow_allowed() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // Self-borrow (edge case, shouldn't happen in real code but graph allows it)
    let edge = graph.add_borrow(1, 1, false, 1100);
    assert!(edge.is_some());
}

// ============================================================================
// Move Tests
// ============================================================================

#[test]
fn test_simple_move() {
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

    let edge = graph.add_move(1, 2, 1100);
    assert!(edge.is_some());
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_move_chain() {
    let mut graph = OwnershipGraph::new();

    // x -> y -> z (ownership transfers)
    for i in 1..=3 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "String".into(),
            created_at: 1000 + i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_move(1, 2, 1200);
    graph.add_move(2, 3, 1300);

    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn test_move_with_missing_variable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "String".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_move(1, 999, 1100);
    assert!(edge.is_none());
}

// ============================================================================
// Smart Pointer Tests
// ============================================================================

#[test]
fn test_rc_clone() {
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

    let edge = graph.add_rc_clone(2, 1, 2, 1100);
    assert!(edge.is_some());
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn test_multiple_rc_clones() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "original".into(),
        type_name: "Rc<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("clone{}", i - 1),
            type_name: "Rc<i32>".into(),
            created_at: 1000 + i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_rc_clone(i, 1, i, 1000 + i as u64 * 100);
    }

    assert_eq!(graph.node_count(), 5);
    assert_eq!(graph.edge_count(), 4);
    assert_eq!(graph.borrowers_of(1).len(), 4);
}

#[test]
fn test_arc_clone() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Arc<i32>".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "Arc<i32>".into(),
        created_at: 1100,
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_arc_clone(2, 1, 2, 1100);
    assert!(edge.is_some());
}

#[test]
fn test_refcell_immutable_borrow() {
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
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_refcell_borrow(2, 1, false, 1100);
    assert!(edge.is_some());
}

#[test]
fn test_refcell_mutable_borrow() {
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
        dropped_at: None,
        scope_depth: 0,
    });

    let edge = graph.add_refcell_borrow(2, 1, true, 1100);
    assert!(edge.is_some());
}

// ============================================================================
// Lifetime Tests
// ============================================================================

#[test]
fn test_mark_dropped() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert!(graph.mark_dropped(1, 2000));

    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.dropped_at, Some(2000));
}

#[test]
fn test_mark_dropped_nonexistent() {
    let mut graph = OwnershipGraph::new();
    assert!(!graph.mark_dropped(999, 2000));
}

#[test]
fn test_mark_dropped_multiple_times() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert!(graph.mark_dropped(1, 2000));
    assert!(graph.mark_dropped(1, 3000));

    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.dropped_at, Some(3000));
}

#[test]
fn test_is_alive_before_creation() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    assert!(!graph.is_alive(1, 500));
}

#[test]
fn test_is_alive_during_lifetime() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    assert!(graph.is_alive(1, 1500));
}

#[test]
fn test_is_alive_after_drop() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    assert!(!graph.is_alive(1, 2500));
}

#[test]
fn test_is_alive_never_dropped() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert!(graph.is_alive(1, 1000));
    assert!(graph.is_alive(1, 10000));
}

#[test]
fn test_is_alive_nonexistent() {
    let graph = OwnershipGraph::new();
    assert!(!graph.is_alive(999, 1000));
}
