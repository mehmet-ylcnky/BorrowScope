use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Advanced Batch Operations Edge Cases
// ============================================================================

#[test]
fn test_add_variables_batch_with_duplicate_ids() {
    let mut graph = OwnershipGraph::new();

    let vars = vec![
        Variable {
            id: 1,
            name: "first".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        },
        Variable {
            id: 1,
            name: "second".into(),
            type_name: "String".into(),
            created_at: 2000,
            dropped_at: None,
            scope_depth: 1,
        },
    ];

    let nodes = graph.add_variables(vars);
    assert_eq!(nodes.len(), 2);
    assert_eq!(graph.node_count(), 2);

    // Last one with same ID should be retrievable
    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.name, "second");
}

#[test]
fn test_add_variables_batch_large_scale() {
    let mut graph = OwnershipGraph::with_capacity(10000, 20000);

    let vars: Vec<_> = (0..10000)
        .map(|i| Variable {
            id: i,
            name: format!("variable_with_long_name_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: i % 10,
        })
        .collect();

    let nodes = graph.add_variables(vars);
    assert_eq!(nodes.len(), 10000);
    assert_eq!(graph.node_count(), 10000);
}

#[test]
fn test_add_variables_batch_with_unicode() {
    let mut graph = OwnershipGraph::new();

    let vars = vec![
        Variable {
            id: 1,
            name: "变量".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        },
        Variable {
            id: 2,
            name: "🦀".into(),
            type_name: "String".into(),
            created_at: 2000,
            dropped_at: None,
            scope_depth: 0,
        },
    ];

    graph.add_variables(vars);
    assert_eq!(graph.get_variable(1).unwrap().name, "变量");
    assert_eq!(graph.get_variable(2).unwrap().name, "🦀");
}

#[test]
fn test_add_variables_batch_with_extreme_timestamps() {
    let mut graph = OwnershipGraph::new();

    let vars = vec![
        Variable {
            id: 1,
            name: "min".into(),
            type_name: "i32".into(),
            created_at: 0,
            dropped_at: Some(0),
            scope_depth: 0,
        },
        Variable {
            id: 2,
            name: "max".into(),
            type_name: "i32".into(),
            created_at: u64::MAX - 1,
            dropped_at: Some(u64::MAX),
            scope_depth: 0,
        },
    ];

    graph.add_variables(vars);
    assert_eq!(graph.node_count(), 2);
}

#[test]
fn test_add_variables_batch_with_extreme_scope_depth() {
    let mut graph = OwnershipGraph::new();

    let vars = vec![
        Variable {
            id: 1,
            name: "shallow".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        },
        Variable {
            id: 2,
            name: "deep".into(),
            type_name: "i32".into(),
            created_at: 2000,
            dropped_at: None,
            scope_depth: usize::MAX,
        },
    ];

    graph.add_variables(vars);
    assert_eq!(graph.get_variable(2).unwrap().scope_depth, usize::MAX);
}

#[test]
fn test_mark_dropped_batch_with_large_array() {
    let mut graph = OwnershipGraph::new();

    for i in 0..1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let ids: Vec<_> = (0..1000).collect();
    let count = graph.mark_dropped_batch(&ids, 10000);
    assert_eq!(count, 1000);
}

#[test]
fn test_mark_dropped_batch_with_mixed_valid_invalid() {
    let mut graph = OwnershipGraph::new();

    // Add only even IDs
    for i in (0..100).step_by(2) {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    // Try to drop all IDs (0-99)
    let ids: Vec<_> = (0..100).collect();
    let count = graph.mark_dropped_batch(&ids, 5000);
    assert_eq!(count, 50); // Only even IDs exist
}

#[test]
fn test_mark_dropped_batch_already_dropped() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    // Try to drop again
    let count = graph.mark_dropped_batch(&[1], 3000);
    assert_eq!(count, 1); // Still succeeds, updates timestamp
    assert_eq!(graph.get_variable(1).unwrap().dropped_at, Some(3000));
}

#[test]
fn test_mark_dropped_batch_reverse_time() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 5000,
        dropped_at: None,
        scope_depth: 0,
    });

    // Drop at time before creation (invalid but allowed by API)
    let count = graph.mark_dropped_batch(&[1], 1000);
    assert_eq!(count, 1);
    assert_eq!(graph.get_variable(1).unwrap().dropped_at, Some(1000));
}

#[test]
fn test_mark_dropped_batch_zero_timestamp() {
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

    let count = graph.mark_dropped_batch(&[1, 2, 3, 4, 5], 0);
    assert_eq!(count, 5);
}

// ============================================================================
// Advanced Validation Edge Cases
// ============================================================================

#[test]
fn test_validate_with_never_dropped_variables() {
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

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_borrow_at_exact_owner_creation() {
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
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1000); // Borrow at exact creation time

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_borrow_one_before_owner_drop() {
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
        created_at: 1500,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1999); // One before drop

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_refcell_borrow_validation() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 1000,
        dropped_at: Some(3000),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "Ref<i32>".into(),
        created_at: 1500,
        dropped_at: Some(2500),
        scope_depth: 0,
    });

    graph.add_refcell_borrow(2, 1, false, 1500);

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_refcell_borrow_after_owner_dropped() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "Ref<i32>".into(),
        created_at: 2500,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_refcell_borrow(2, 1, false, 2500);

    let result = graph.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_complex_valid_graph() {
    let mut graph = OwnershipGraph::new();

    // Create 10 variables with proper lifetimes
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 500),
            scope_depth: 0,
        });
    }

    // Add borrows with valid timing
    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_all_error_types() {
    let mut graph = OwnershipGraph::new();

    // Error 1: Dropped before creation
    graph.add_variable(Variable {
        id: 1,
        name: "x1".into(),
        type_name: "i32".into(),
        created_at: 2000,
        dropped_at: Some(1000),
        scope_depth: 0,
    });

    // Error 2: Borrow before owner creation
    graph.add_variable(Variable {
        id: 2,
        name: "x2".into(),
        type_name: "i32".into(),
        created_at: 3000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&i32".into(),
        created_at: 2500,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(3, 2, false, 2500);

    // Error 3: Borrow after owner dropped
    graph.add_variable(Variable {
        id: 4,
        name: "x3".into(),
        type_name: "i32".into(),
        created_at: 4000,
        dropped_at: Some(5000),
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 5,
        name: "r3".into(),
        type_name: "&i32".into(),
        created_at: 5500,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(5, 4, false, 5500);

    let result = graph.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 3);
}

#[test]
fn test_validate_empty_names_and_types() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: String::new(),
        type_name: String::new(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert!(graph.validate().is_ok());
}

#[test]
fn test_validate_after_clear() {
    let mut graph = OwnershipGraph::new();

    // Add invalid data
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 2000,
        dropped_at: Some(1000),
        scope_depth: 0,
    });

    graph.clear();

    // Should be valid after clear
    assert!(graph.validate().is_ok());
}

// ============================================================================
// Advanced Statistics Edge Cases
// ============================================================================

#[test]
fn test_statistics_all_dropped() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 200),
            scope_depth: 0,
        });
    }

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 100);
    assert_eq!(stats.alive_variables, 0);
}

#[test]
fn test_statistics_mixed_relationship_counts() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=21 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    // 5 immutable borrows
    for i in 2..=6 {
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    // 3 mutable borrows
    for i in 7..=9 {
        graph.add_borrow(i, 1, true, i as u64 * 100);
    }

    // 4 moves
    for i in 10..=13 {
        graph.add_move(i, 1, i as u64 * 100);
    }

    // 2 rc clones
    for i in 14..=15 {
        graph.add_rc_clone(i, 1, 2, i as u64 * 100);
    }

    // 3 arc clones
    for i in 16..=18 {
        graph.add_arc_clone(i, 1, 2, i as u64 * 100);
    }

    // 3 refcell borrows
    for i in 19..=21 {
        graph.add_refcell_borrow(i, 1, false, i as u64 * 100);
    }

    let stats = graph.statistics();
    assert_eq!(stats.immutable_borrows, 5);
    assert_eq!(stats.mutable_borrows, 3);
    assert_eq!(stats.moves, 4);
    assert_eq!(stats.rc_clones, 2);
    assert_eq!(stats.arc_clones, 3);
    assert_eq!(stats.refcell_borrows, 3);
    assert_eq!(stats.total_edges, 20);
}

#[test]
fn test_statistics_with_disconnected_components() {
    let mut graph = OwnershipGraph::new();

    // Component 1: 5 nodes
    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("c1_var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=5 {
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    // Component 2: 3 nodes
    for i in 6..=8 {
        graph.add_variable(Variable {
            id: i,
            name: format!("c2_var{}", i),
            type_name: "String".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 7..=8 {
        graph.add_borrow(i, 6, false, i as u64 * 100);
    }

    // Isolated nodes
    for i in 9..=12 {
        graph.add_variable(Variable {
            id: i,
            name: format!("isolated{}", i),
            type_name: "f64".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 12);
    assert_eq!(stats.total_edges, 6);
}

#[test]
fn test_statistics_extreme_values() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: usize::MAX,
        name: "max_id".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: usize::MAX,
    });

    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 1);
    assert_eq!(stats.alive_variables, 0);
}

#[test]
fn test_statistics_after_batch_operations() {
    let mut graph = OwnershipGraph::new();

    let vars: Vec<_> = (1..=50)
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

    let stats_before = graph.statistics();
    assert_eq!(stats_before.alive_variables, 50);

    let ids: Vec<_> = (1..=25).collect();
    graph.mark_dropped_batch(&ids, 10000);

    let stats_after = graph.statistics();
    assert_eq!(stats_after.alive_variables, 25);
}

// ============================================================================
// Integration and Stress Tests
// ============================================================================

#[test]
fn test_batch_validate_statistics_integration() {
    let mut graph = OwnershipGraph::new();

    // Add 100 variables in batch
    let vars: Vec<_> = (1..=100)
        .map(|i| Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: i % 5,
        })
        .collect();

    graph.add_variables(vars);

    // Add relationships
    for i in 2..=100 {
        graph.add_borrow(i, i - 1, i % 2 == 0, i as u64 * 100);
    }

    // Validate
    assert!(graph.validate().is_ok());

    // Check statistics
    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 100);
    assert_eq!(stats.alive_variables, 100);
    assert_eq!(stats.total_edges, 99);

    // Drop half in batch
    let ids: Vec<_> = (1..=50).collect();
    let count = graph.mark_dropped_batch(&ids, 20000);
    assert_eq!(count, 50);

    // Still valid
    assert!(graph.validate().is_ok());

    // Updated statistics
    let stats = graph.statistics();
    assert_eq!(stats.alive_variables, 50);
}

#[test]
fn test_stress_batch_operations() {
    let mut graph = OwnershipGraph::with_capacity(5000, 10000);

    // Add 5000 variables in batches of 500
    for batch in 0..10 {
        let vars: Vec<_> = (batch * 500..(batch + 1) * 500)
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
    }

    assert_eq!(graph.node_count(), 5000);

    // Drop in batches
    for batch in 0..10 {
        let ids: Vec<_> = (batch * 500..(batch + 1) * 500).collect();
        let count = graph.mark_dropped_batch(&ids, 100000);
        assert_eq!(count, 500);
    }

    let stats = graph.statistics();
    assert_eq!(stats.alive_variables, 0);
}

#[test]
fn test_validation_performance_large_graph() {
    let mut graph = OwnershipGraph::with_capacity(1000, 2000);

    for i in 0..1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 5000),
            scope_depth: 0,
        });
    }

    for i in 1..1000 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    // Should complete quickly even with 1000 nodes and 999 edges
    assert!(graph.validate().is_ok());
}

#[test]
fn test_statistics_consistency_after_modifications() {
    let mut graph = OwnershipGraph::new();

    // Initial state
    let vars: Vec<_> = (1..=10)
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

    let stats1 = graph.statistics();
    assert_eq!(stats1.total_variables, 10);

    // Add more variables
    for i in 11..=20 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let stats2 = graph.statistics();
    assert_eq!(stats2.total_variables, 20);

    // Drop some
    graph.mark_dropped_batch(&[1, 2, 3, 4, 5], 5000);

    let stats3 = graph.statistics();
    assert_eq!(stats3.total_variables, 20);
    assert_eq!(stats3.alive_variables, 15);
}
