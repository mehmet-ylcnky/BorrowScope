use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Batch Operations Tests
// ============================================================================

#[test]
fn test_add_variables_batch_empty() {
    let mut graph = OwnershipGraph::new();
    let nodes = graph.add_variables(vec![]);
    assert_eq!(nodes.len(), 0);
    assert_eq!(graph.node_count(), 0);
}

#[test]
fn test_add_variables_batch_single() {
    let mut graph = OwnershipGraph::new();

    let vars = vec![Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    }];

    let nodes = graph.add_variables(vars);
    assert_eq!(nodes.len(), 1);
    assert_eq!(graph.node_count(), 1);
}

#[test]
fn test_add_variables_batch_multiple() {
    let mut graph = OwnershipGraph::new();

    let vars: Vec<_> = (1..=100)
        .map(|i| Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        })
        .collect();

    let nodes = graph.add_variables(vars);
    assert_eq!(nodes.len(), 100);
    assert_eq!(graph.node_count(), 100);
}

#[test]
fn test_add_variables_batch_preserves_order() {
    let mut graph = OwnershipGraph::new();

    let vars: Vec<_> = (1..=10)
        .map(|i| Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        })
        .collect();

    graph.add_variables(vars);

    for i in 1..=10 {
        let var = graph.get_variable(i).unwrap();
        assert_eq!(var.name, format!("var{}", i));
    }
}

#[test]
fn test_mark_dropped_batch_empty() {
    let mut graph = OwnershipGraph::new();
    let count = graph.mark_dropped_batch(&[], 1000);
    assert_eq!(count, 0);
}

#[test]
fn test_mark_dropped_batch_single() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let count = graph.mark_dropped_batch(&[1], 2000);
    assert_eq!(count, 1);
    assert_eq!(graph.get_variable(1).unwrap().dropped_at, Some(2000));
}

#[test]
fn test_mark_dropped_batch_multiple() {
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

    let ids: Vec<_> = (1..=10).collect();
    let count = graph.mark_dropped_batch(&ids, 5000);
    assert_eq!(count, 10);

    for i in 1..=10 {
        assert_eq!(graph.get_variable(i).unwrap().dropped_at, Some(5000));
    }
}

#[test]
fn test_mark_dropped_batch_partial_success() {
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

    // Try to drop both existing and non-existing variables
    let ids = vec![1, 2, 3, 999, 888, 777];
    let count = graph.mark_dropped_batch(&ids, 5000);
    assert_eq!(count, 3); // Only 1, 2, 3 exist
}

#[test]
fn test_mark_dropped_batch_all_nonexistent() {
    let mut graph = OwnershipGraph::new();
    let count = graph.mark_dropped_batch(&[999, 888, 777], 5000);
    assert_eq!(count, 0);
}

#[test]
fn test_mark_dropped_batch_duplicates() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // Same ID multiple times
    let count = graph.mark_dropped_batch(&[1, 1, 1], 2000);
    assert_eq!(count, 3); // Each call succeeds
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validate_empty_graph() {
    let graph = OwnershipGraph::new();
    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_simple_valid_graph() {
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

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_dropped_before_creation() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 2000,
        dropped_at: Some(1000), // Dropped before created!
        scope_depth: 0,
    });

    let result = graph.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("dropped before creation"));
}

#[test]
fn test_validate_borrow_before_owner_creation() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1500); // Borrow at 1500, but owner created at 2000

    let result = graph.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("before owner"));
    assert!(errors[0].contains("creation"));
}

#[test]
fn test_validate_borrow_after_owner_dropped() {
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
        created_at: 2500,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 2500); // Borrow at 2500, but owner dropped at 2000

    let result = graph.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("after owner"));
    assert!(errors[0].contains("dropped"));
}

#[test]
fn test_validate_multiple_errors() {
    let mut graph = OwnershipGraph::new();

    // Error 1: Dropped before creation
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 2000,
        dropped_at: Some(1000),
        scope_depth: 0,
    });

    // Error 2: Another dropped before creation
    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 3000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let result = graph.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_validate_moves_not_checked() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "String".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "String".into(),
        created_at: 1500,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_move(1, 2, 1500);

    // Moves don't have the same validation rules as borrows
    assert!(graph.validate().is_ok());
}

#[test]
fn test_has_cycles_empty_graph() {
    let graph = OwnershipGraph::new();
    assert!(!graph.has_cycles());
}

#[test]
fn test_has_cycles_acyclic_graph() {
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

    // Create chain: 1 <- 2 <- 3 <- ... <- 10
    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    assert!(!graph.has_cycles());
}

// ============================================================================
// Statistics Tests
// ============================================================================

#[test]
fn test_statistics_empty_graph() {
    let graph = OwnershipGraph::new();
    let stats = graph.statistics();

    assert_eq!(stats.total_variables, 0);
    assert_eq!(stats.alive_variables, 0);
    assert_eq!(stats.total_edges, 0);
    assert_eq!(stats.immutable_borrows, 0);
    assert_eq!(stats.mutable_borrows, 0);
    assert_eq!(stats.moves, 0);
    assert_eq!(stats.rc_clones, 0);
    assert_eq!(stats.arc_clones, 0);
    assert_eq!(stats.refcell_borrows, 0);
}

#[test]
fn test_statistics_single_variable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 1);
    assert_eq!(stats.alive_variables, 1);
    assert_eq!(stats.total_edges, 0);
}

#[test]
fn test_statistics_alive_vs_dropped() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
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
            scope_depth: 0,
        });
    }

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 10);
    assert_eq!(stats.alive_variables, 5); // Odd numbers are alive
}

#[test]
fn test_statistics_all_relationship_types() {
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

    let stats = graph.statistics();
    assert_eq!(stats.total_edges, 6);
    assert_eq!(stats.immutable_borrows, 1);
    assert_eq!(stats.mutable_borrows, 1);
    assert_eq!(stats.moves, 1);
    assert_eq!(stats.rc_clones, 1);
    assert_eq!(stats.arc_clones, 1);
    assert_eq!(stats.refcell_borrows, 1);
}

#[test]
fn test_statistics_multiple_same_type() {
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
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let stats = graph.statistics();
    assert_eq!(stats.immutable_borrows, 10);
    assert_eq!(stats.mutable_borrows, 0);
}

#[test]
fn test_statistics_after_clear() {
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

    graph.clear();

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 0);
    assert_eq!(stats.alive_variables, 0);
    assert_eq!(stats.total_edges, 0);
}

#[test]
fn test_statistics_large_graph() {
    let mut graph = OwnershipGraph::with_capacity(1000, 2000);

    for i in 0..1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: if i < 500 { Some(i as u64 + 1000) } else { None },
            scope_depth: 0,
        });
    }

    for i in 1..1000 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 1000);
    assert_eq!(stats.alive_variables, 500);
    assert_eq!(stats.total_edges, 999);
    assert_eq!(stats.immutable_borrows, 999);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_batch_and_validate_integration() {
    let mut graph = OwnershipGraph::new();

    let vars: Vec<_> = (1..=5)
        .map(|i| Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        })
        .collect();

    graph.add_variables(vars);

    // Add some relationships
    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 300);

    // Validate
    assert!(graph.validate().is_ok());

    // Drop in batch
    let count = graph.mark_dropped_batch(&[1, 2, 3, 4, 5], 5000);
    assert_eq!(count, 5);

    // Still valid
    assert!(graph.validate().is_ok());

    // Check statistics
    let stats = graph.statistics();
    assert_eq!(stats.alive_variables, 0);
}
