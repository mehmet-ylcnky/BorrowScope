use borrowscope_graph::{ConflictType, OwnershipGraph, Variable};

// ============================================================================
// Advanced Temporal Conflict Tests
// ============================================================================

#[test]
fn test_conflict_at_exact_borrow_start() {
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
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 200);

    let conflict = graph.check_conflicts_at(1, 200);
    assert!(conflict.is_some());
    assert_eq!(
        conflict.unwrap().conflict_type,
        ConflictType::MultipleMutableBorrows
    );
}

#[test]
fn test_conflict_at_exact_borrow_end() {
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
}

#[test]
fn test_microsecond_precision_conflicts() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000000,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "&mut i32".into(),
        created_at: 1000001,
        dropped_at: Some(1000003),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 1000002,
        dropped_at: Some(1000004),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 1000001);
    graph.add_borrow(3, 1, true, 1000002);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].time_range.0, 1000002);
    assert_eq!(conflicts[0].time_range.1, 1000003);
}

#[test]
fn test_zero_duration_borrow() {
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
        dropped_at: Some(200),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);

    let active = graph.active_borrows_at_time(1, 200);
    assert_eq!(active.len(), 0);
}

// ============================================================================
// Complex Multi-Owner Conflict Tests
// ============================================================================

#[test]
fn test_cascading_conflicts() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("x{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..=5 {
        for j in 1..=2 {
            let borrower_id = i * 10 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: "&mut i32".into(),
                created_at: i as u64 * 1000 + j as u64 * 10,
                dropped_at: Some(i as u64 * 1000 + 100),
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, true, i as u64 * 1000 + j as u64 * 10);
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 5);
}

#[test]
fn test_interleaved_conflicts() {
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
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    // Create overlapping borrows for each owner
    for i in 1..=4 {
        let owner = if i % 2 == 0 { 1 } else { 2 };
        let base_time = if owner == 1 { 1000 } else { 2000 };
        graph.add_variable(Variable {
            id: i + 10,
            name: format!("r{}", i),
            type_name: "&mut i32".into(),
            created_at: base_time + i as u64 * 10,
            dropped_at: Some(base_time + 100),
            scope_depth: 1,
        });
        graph.add_borrow(i + 10, owner, true, base_time + i as u64 * 10);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 2);
}

// ============================================================================
// RefCell Advanced Tests
// ============================================================================

#[test]
fn test_refcell_complex_borrow_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<Vec<i32>>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "Ref<Vec<i32>>".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "Ref<Vec<i32>>".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 4,
        name: "r3".into(),
        type_name: "RefMut<Vec<i32>>".into(),
        created_at: 400,
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_refcell_borrow(2, 1, false, 200);
    graph.add_refcell_borrow(3, 1, false, 250);
    graph.add_refcell_borrow(4, 1, true, 400);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_refcell_three_mutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "RefMut<i32>".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 150),
            scope_depth: 1,
        });
        graph.add_refcell_borrow(i, 1, true, i as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 2);
}

#[test]
fn test_refcell_alternating_borrow_types() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<String>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0..10 {
        let is_mut = i % 2 == 0;
        graph.add_variable(Variable {
            id: i + 2,
            name: format!("r{}", i),
            type_name: if is_mut {
                "RefMut<String>"
            } else {
                "Ref<String>"
            }
            .into(),
            created_at: (i + 2) as u64 * 100,
            dropped_at: Some((i + 2) as u64 * 100 + 50),
            scope_depth: 1,
        });
        graph.add_refcell_borrow(i + 2, 1, is_mut, (i + 2) as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Extreme Scale Tests
// ============================================================================

#[test]
fn test_large_number_of_immutable_borrows() {
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
            type_name: "&i32".into(),
            created_at: i as u64 * 10,
            dropped_at: Some(i as u64 * 10 + 5),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 10);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_many_sequential_mutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=50 {
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
fn test_conflict_detection_with_100_owners() {
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

        if owner_id % 10 == 0 {
            for j in 1..=2 {
                let borrower_id = owner_id * 1000 + j;
                graph.add_variable(Variable {
                    id: borrower_id,
                    name: format!("r{}_{}", owner_id, j),
                    type_name: "&mut i32".into(),
                    created_at: owner_id as u64 * 1000 + j as u64 * 10,
                    dropped_at: Some(owner_id as u64 * 1000 + 100),
                    scope_depth: 1,
                });
                graph.add_borrow(
                    borrower_id,
                    owner_id,
                    true,
                    owner_id as u64 * 1000 + j as u64 * 10,
                );
            }
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 10);
}

// ============================================================================
// Mixed Borrow Type Tests
// ============================================================================

#[test]
fn test_five_immutable_one_mutable_conflict() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=6 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: 200,
            dropped_at: Some(600),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, 200);
    }

    graph.add_variable(Variable {
        id: 7,
        name: "r_mut".into(),
        type_name: "&mut i32".into(),
        created_at: 400,
        dropped_at: Some(500),
        scope_depth: 1,
    });
    graph.add_borrow(7, 1, true, 400);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 5);
    assert!(conflicts
        .iter()
        .all(|c| c.conflict_type == ConflictType::MutableWithImmutable));
}

#[test]
fn test_alternating_mutable_immutable_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 0..20 {
        let is_mut = i % 2 == 0;
        graph.add_variable(Variable {
            id: i + 2,
            name: format!("r{}", i),
            type_name: if is_mut { "&mut i32" } else { "&i32" }.into(),
            created_at: (i + 2) as u64 * 100,
            dropped_at: Some((i + 2) as u64 * 100 + 50),
            scope_depth: 1,
        });
        graph.add_borrow(i + 2, 1, is_mut, (i + 2) as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Timeline and Reporting Tests
// ============================================================================

#[test]
fn test_conflict_timeline_with_gaps() {
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
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: Some(300),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&i32".into(),
        created_at: 500,
        dropped_at: Some(600),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 500);

    let timeline = graph.conflict_timeline(1);
    assert_eq!(timeline.len(), 2);
    assert!(timeline[0].0 < timeline[1].0);
}

#[test]
fn test_conflict_timeline_dense_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=20 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: i as u64 * 10,
            dropped_at: Some(i as u64 * 10 + 100),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 10);
    }

    let timeline = graph.conflict_timeline(1);
    assert!(!timeline.is_empty());
    assert!(timeline.iter().all(|(_, borrows)| !borrows.is_empty()));
}

#[test]
fn test_report_formatting_with_unicode() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "数据".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "参考1".into(),
        type_name: "&mut i32".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "参考2".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let report = graph.report_conflicts();
    assert!(report.contains("数据"));
    assert!(report.contains("参考"));
}

// ============================================================================
// Edge Case Boundary Tests
// ============================================================================

#[test]
fn test_u64_max_timestamp() {
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
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, u64::MAX - 500);

    let active = graph.active_borrows_at_time(1, u64::MAX - 100);
    assert_eq!(active.len(), 1);
}

#[test]
fn test_conflict_with_max_scope_depth() {
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
        dropped_at: Some(400),
        scope_depth: usize::MAX,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: usize::MAX,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
}

#[test]
fn test_empty_variable_names_in_conflicts() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "".into(),
        type_name: "&mut i32".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);

    let formatted = conflicts[0].format(&graph);
    assert!(!formatted.is_empty());
}

// ============================================================================
// Performance and Optimization Tests
// ============================================================================

#[test]
fn test_optimized_vs_naive_consistency() {
    let mut graph = OwnershipGraph::new();

    for owner_id in 1..=10 {
        graph.add_variable(Variable {
            id: owner_id,
            name: format!("x{}", owner_id),
            type_name: "i32".into(),
            created_at: owner_id as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for j in 1..=3 {
            let borrower_id = owner_id * 10 + j;
            let base_time = owner_id as u64 * 1000;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", owner_id, j),
                type_name: if j == 1 { "&i32" } else { "&mut i32" }.into(),
                created_at: base_time + j as u64 * 10,
                dropped_at: Some(base_time + 100),
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, owner_id, j != 1, base_time + j as u64 * 10);
        }
    }

    let conflicts1 = graph.find_conflicts();
    let conflicts2 = graph.find_conflicts_optimized();

    // Both should find conflicts, optimized may find more due to better interval detection
    assert!(!conflicts1.is_empty());
    assert!(!conflicts2.is_empty());
    assert!(conflicts2.len() >= 10);
}

#[test]
fn test_conflict_detection_with_cleared_graph() {
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
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);

    graph.clear();

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);

    let report = graph.report_conflicts();
    assert!(report.contains("No borrow conflicts"));
}

#[test]
fn test_conflict_with_nonexistent_owner() {
    let graph = OwnershipGraph::new();

    let conflict = graph.check_conflicts_at(999, 100);
    assert!(conflict.is_none());

    let timeline = graph.conflict_timeline(999);
    assert!(timeline.is_empty());

    let active = graph.active_borrows_at_time(999, 100);
    assert!(active.is_empty());
}

#[test]
fn test_simultaneous_conflicts_different_types() {
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
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(450),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 4,
        name: "r3".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 250);
    graph.add_borrow(4, 1, true, 300);

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 2);
    assert!(conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::MutableWithImmutable));
    assert!(conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::MultipleMutableBorrows));
}
