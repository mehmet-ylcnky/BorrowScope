use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Extreme Temporal Precision Tests
// ============================================================================

#[test]
fn test_conflict_nanosecond_precision() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000000000,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0u64..10 {
        let ts = 1000000000 + i;
        graph.add_variable(Variable {
            id: (i + 2) as usize,
            name: format!("r{}", i),
            type_name: "&mut i32".into(),
            created_at: ts,
            dropped_at: Some(ts + 5),
            scope_depth: 1,
        });
        graph.add_borrow((i + 2) as usize, 1, true, ts);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_at_timestamp_boundaries() {
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
        name: "r1".into(),
        type_name: "&mut i32".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 300);

    let conflict_at_299 = graph.check_conflicts_at(1, 299);
    assert!(conflict_at_299.is_none());

    let conflict_at_300 = graph.check_conflicts_at(1, 300);
    assert!(conflict_at_300.is_none());

    let conflict_at_301 = graph.check_conflicts_at(1, 301);
    assert!(conflict_at_301.is_none());
}

#[test]
fn test_conflict_with_instant_lifetime() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0usize..100 {
        let ts = 200 + i as u64;
        graph.add_variable(Variable {
            id: i + 2,
            name: format!("r{}", i),
            type_name: "&mut i32".into(),
            created_at: ts,
            dropped_at: Some(ts),
            scope_depth: 1,
        });
        graph.add_borrow(i + 2, 1, true, ts);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Massive Scale Conflict Tests
// ============================================================================

#[test]
fn test_conflict_1000_simultaneous_immutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=1001 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: 200,
            dropped_at: Some(300),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, 200);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_conflict_100_mutable_borrows_sequential() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=101 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&mut i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 50),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, true, i as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_conflict_50_overlapping_mutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=51 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&mut i32".into(),
            created_at: 200 + i as u64,
            dropped_at: Some(1000),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, true, 200 + i as u64);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 49);
}

// ============================================================================
// Complex Borrow Pattern Tests
// ============================================================================

#[test]
fn test_conflict_alternating_mutable_immutable_dense() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0..100 {
        let is_mut = i % 2 == 0;
        graph.add_variable(Variable {
            id: i + 2,
            name: format!("r{}", i),
            type_name: if is_mut { "&mut i32" } else { "&i32" }.into(),
            created_at: 200 + i as u64 * 10,
            dropped_at: Some(200 + i as u64 * 10 + 5),
            scope_depth: 1,
        });
        graph.add_borrow(i + 2, 1, is_mut, 200 + i as u64 * 10);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_conflict_wave_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for wave in 0..10 {
        for i in 0..5 {
            let id = wave * 5 + i + 2;
            graph.add_variable(Variable {
                id,
                name: format!("r_{}_{}", wave, i),
                type_name: "&mut i32".into(),
                created_at: wave as u64 * 1000 + i as u64 * 10,
                dropped_at: Some(wave as u64 * 1000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(id, 1, true, wave as u64 * 1000 + i as u64 * 10);
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 10);
}

#[test]
fn test_conflict_pyramid_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let mut id = 2;
    for level in 1..=10 {
        for i in 0..level {
            graph.add_variable(Variable {
                id,
                name: format!("r_{}_{}", level, i),
                type_name: "&mut i32".into(),
                created_at: level as u64 * 1000 + i as u64 * 10,
                dropped_at: Some(level as u64 * 1000 + 100),
                scope_depth: level,
            });
            graph.add_borrow(id, 1, true, level as u64 * 1000 + i as u64 * 10);
            id += 1;
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(!conflicts.is_empty());
}

// ============================================================================
// RefCell Complex Scenarios
// ============================================================================

#[test]
fn test_conflict_refcell_100_immutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<Vec<i32>>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=101 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "Ref<Vec<i32>>".into(),
            created_at: 200,
            dropped_at: Some(300),
            scope_depth: 1,
        });
        graph.add_refcell_borrow(i, 1, false, 200);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_conflict_refcell_interleaved_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0..50 {
        let is_mut = i % 5 == 0;
        graph.add_variable(Variable {
            id: i + 2,
            name: format!("r{}", i),
            type_name: if is_mut { "RefMut<i32>" } else { "Ref<i32>" }.into(),
            created_at: 200 + i as u64 * 20,
            dropped_at: Some(200 + i as u64 * 20 + 10),
            scope_depth: 1,
        });
        graph.add_refcell_borrow(i + 2, 1, is_mut, 200 + i as u64 * 20);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_conflict_refcell_nested_cells() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("cell{}", i),
            type_name: "RefCell<i32>".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let mut id = 11;
    for cell_id in 1..=10 {
        for j in 0..3 {
            graph.add_variable(Variable {
                id,
                name: format!("r_{}_{}", cell_id, j),
                type_name: "RefMut<i32>".into(),
                created_at: cell_id as u64 * 1000 + j as u64 * 10,
                dropped_at: Some(cell_id as u64 * 1000 + 100),
                scope_depth: 1,
            });
            graph.add_refcell_borrow(id, cell_id, true, cell_id as u64 * 1000 + j as u64 * 10);
            id += 1;
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 10);
}

// ============================================================================
// Multi-Owner Conflict Scenarios
// ============================================================================

#[test]
fn test_conflict_100_owners_with_conflicts() {
    let mut graph = OwnershipGraph::new();

    for owner_id in 1..=100 {
        graph.add_variable(Variable {
            id: owner_id,
            name: format!("x{}", owner_id),
            type_name: "i32".into(),
            created_at: owner_id as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for j in 1..=3 {
            let borrower_id = owner_id * 1000 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", owner_id, j),
                type_name: "&mut i32".into(),
                created_at: owner_id as u64 * 10000 + j as u64 * 10,
                dropped_at: Some(owner_id as u64 * 10000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(
                borrower_id,
                owner_id,
                true,
                owner_id as u64 * 10000 + j as u64 * 10,
            );
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 100);
}

#[test]
fn test_conflict_cross_owner_no_conflicts() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("x{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        graph.add_variable(Variable {
            id: i + 100,
            name: format!("r{}", i),
            type_name: "&mut i32".into(),
            created_at: i as u64 * 1000,
            dropped_at: Some(i as u64 * 1000 + 100),
            scope_depth: 1,
        });

        graph.add_borrow(i + 100, i, true, i as u64 * 1000);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Timeline and Reporting Edge Cases
// ============================================================================

#[test]
fn test_conflict_timeline_1000_events() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=1001 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: i as u64 * 10,
            dropped_at: Some(i as u64 * 10 + 5),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 10);
    }

    let timeline = graph.conflict_timeline(1);
    assert!(!timeline.is_empty());
}

#[test]
fn test_conflict_report_with_1000_conflicts() {
    let mut graph = OwnershipGraph::new();

    for owner_id in 1..=100 {
        graph.add_variable(Variable {
            id: owner_id,
            name: format!("x{}", owner_id),
            type_name: "i32".into(),
            created_at: owner_id as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for j in 1..=10 {
            let borrower_id = owner_id * 100 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", owner_id, j),
                type_name: "&mut i32".into(),
                created_at: owner_id as u64 * 10000 + j as u64,
                dropped_at: Some(owner_id as u64 * 10000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(
                borrower_id,
                owner_id,
                true,
                owner_id as u64 * 10000 + j as u64,
            );
        }
    }

    let report = graph.report_conflicts();
    assert!(report.contains("conflict"));
}

// ============================================================================
// Extreme Scope Depth Tests
// ============================================================================

#[test]
fn test_conflict_max_scope_depth() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&mut i32".into(),
            created_at: 200,
            dropped_at: Some(300),
            scope_depth: usize::MAX - i,
        });
        graph.add_borrow(i, 1, true, 200);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_deeply_nested_scopes() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for depth in 1..=1000 {
        graph.add_variable(Variable {
            id: depth + 1,
            name: format!("r{}", depth),
            type_name: "&mut i32".into(),
            created_at: depth as u64 * 1000,
            dropped_at: Some(depth as u64 * 1000 + 500),
            scope_depth: depth,
        });
        graph.add_borrow(depth + 1, 1, true, depth as u64 * 1000);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Performance Stress Tests
// ============================================================================

#[test]
fn test_conflict_detection_performance_stress() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=500 {
        graph.add_variable(Variable {
            id: i,
            name: format!("x{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for j in 1..=5 {
            let borrower_id = i * 10 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: if j % 2 == 0 { "&mut i32" } else { "&i32" }.into(),
                created_at: i as u64 * 1000 + j as u64 * 10,
                dropped_at: Some(i as u64 * 1000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, j % 2 == 0, i as u64 * 1000 + j as u64 * 10);
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(!conflicts.is_empty());
}

#[test]
fn test_conflict_optimized_vs_naive_large_scale() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("x{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for j in 1..=10 {
            let borrower_id = i * 100 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: if j <= 5 { "&i32" } else { "&mut i32" }.into(),
                created_at: i as u64 * 10000 + j as u64 * 10,
                dropped_at: Some(i as u64 * 10000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, j > 5, i as u64 * 10000 + j as u64 * 10);
        }
    }

    let conflicts1 = graph.find_conflicts();
    let conflicts2 = graph.find_conflicts_optimized();

    assert!(!conflicts1.is_empty());
    assert!(!conflicts2.is_empty());
}
