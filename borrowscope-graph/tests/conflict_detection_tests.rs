use borrowscope_graph::{ConflictType, OwnershipGraph, Variable};

// ============================================================================
// Basic Conflict Detection Tests
// ============================================================================

#[test]
fn test_no_conflicts_valid_code() {
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
        created_at: 400,
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 400);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_multiple_immutable_borrows_no_conflict() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 50),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_multiple_mutable_borrows_conflict() {
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
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::MultipleMutableBorrows
    );
    assert_eq!(conflicts[0].owner_id, 1);
    assert_eq!(conflicts[0].borrowers.len(), 2);
}

#[test]
fn test_mutable_with_immutable_conflict() {
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
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::MutableWithImmutable
    );
    assert_eq!(conflicts[0].owner_id, 1);
    assert_eq!(conflicts[0].borrowers.len(), 2);
}

// ============================================================================
// Time-Based Conflict Tests
// ============================================================================

#[test]
fn test_conflict_at_specific_time() {
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
        created_at: 300,
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 300);

    let conflict = graph.check_conflicts_at(1, 350);
    assert!(conflict.is_some());
    assert_eq!(
        conflict.unwrap().conflict_type,
        ConflictType::MultipleMutableBorrows
    );

    let no_conflict = graph.check_conflicts_at(1, 150);
    assert!(no_conflict.is_none());
}

#[test]
fn test_overlapping_borrow_intervals() {
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
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: Some(600),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 300);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].time_range.0, 300);
    assert_eq!(conflicts[0].time_range.1, 500);
}

#[test]
fn test_non_overlapping_borrows_no_conflict() {
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
        created_at: 400,
        dropped_at: Some(500),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 400);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

// ============================================================================
// Complex Conflict Scenarios
// ============================================================================

#[test]
fn test_three_way_mutable_conflict() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i - 1),
            type_name: "&mut i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 200),
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, true, i as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert!(!conflicts.is_empty());
    assert!(conflicts
        .iter()
        .all(|c| c.conflict_type == ConflictType::MultipleMutableBorrows));
}

#[test]
fn test_mixed_conflict_scenario() {
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
        type_name: "&i32".into(),
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
    graph.add_borrow(3, 1, false, 250);
    graph.add_borrow(4, 1, true, 300);

    let conflicts = graph.find_conflicts_optimized();
    assert!(conflicts.len() >= 2);
    assert!(conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::MutableWithImmutable));
}

#[test]
fn test_nested_scope_conflicts() {
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
        name: "outer".into(),
        type_name: "&mut i32".into(),
        created_at: 200,
        dropped_at: Some(600),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "inner".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: Some(400),
        scope_depth: 2,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 300);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
}

// ============================================================================
// RefCell Conflict Tests
// ============================================================================

#[test]
fn test_refcell_multiple_mutable_borrows() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "RefMut<i32>".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "RefMut<i32>".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_refcell_borrow(2, 1, true, 200);
    graph.add_refcell_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::MultipleMutableBorrows
    );
}

#[test]
fn test_refcell_mutable_with_immutable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "cell".into(),
        type_name: "RefCell<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "r1".into(),
        type_name: "Ref<i32>".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "RefMut<i32>".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_refcell_borrow(2, 1, false, 200);
    graph.add_refcell_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(
        conflicts[0].conflict_type,
        ConflictType::MutableWithImmutable
    );
}

// ============================================================================
// Conflict Timeline Tests
// ============================================================================

#[test]
fn test_conflict_timeline() {
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
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 250);

    let timeline = graph.conflict_timeline(1);
    assert!(!timeline.is_empty());
    assert!(timeline.iter().any(|(_, borrows)| borrows.len() > 1));
}

#[test]
fn test_empty_conflict_timeline() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let timeline = graph.conflict_timeline(1);
    assert!(timeline.is_empty());
}

// ============================================================================
// Conflict Reporting Tests
// ============================================================================

#[test]
fn test_conflict_format() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "data".into(),
        type_name: "Vec<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "ref1".into(),
        type_name: "&mut Vec<i32>".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "ref2".into(),
        type_name: "&mut Vec<i32>".into(),
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);

    let formatted = conflicts[0].format(&graph);
    assert!(formatted.contains("data"));
    assert!(formatted.contains("ref1") || formatted.contains("ref2"));
}

#[test]
fn test_report_no_conflicts() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let report = graph.report_conflicts();
    assert!(report.contains("No borrow conflicts"));
}

#[test]
fn test_report_with_conflicts() {
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
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let report = graph.report_conflicts();
    assert!(report.contains("Found 1 conflict"));
    assert!(report.contains("Multiple mutable borrows"));
}

// ============================================================================
// Edge Cases and Performance Tests
// ============================================================================

#[test]
fn test_conflict_with_never_dropped_variables() {
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
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 250,
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 1);
}

#[test]
fn test_active_borrows_at_boundary() {
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

    let at_start = graph.active_borrows_at_time(1, 200);
    assert_eq!(at_start.len(), 1);

    let at_end = graph.active_borrows_at_time(1, 300);
    assert_eq!(at_end.len(), 0);

    let before = graph.active_borrows_at_time(1, 199);
    assert_eq!(before.len(), 0);
}

#[test]
fn test_multiple_owners_with_conflicts() {
    let mut graph = OwnershipGraph::new();

    for owner_id in 1..=3 {
        graph.add_variable(Variable {
            id: owner_id,
            name: format!("x{}", owner_id),
            type_name: "i32".into(),
            created_at: owner_id as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        for borrower_offset in 1..=2 {
            let borrower_id = owner_id * 10 + borrower_offset;
            let base_time = owner_id as u64 * 1000;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", owner_id, borrower_offset),
                type_name: "&mut i32".into(),
                created_at: base_time + borrower_offset as u64 * 10,
                dropped_at: Some(base_time + 100),
                scope_depth: 1,
            });
            graph.add_borrow(
                borrower_id,
                owner_id,
                true,
                base_time + borrower_offset as u64 * 10,
            );
        }
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 3);
}

#[test]
fn test_conflict_detection_performance() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 51..=100 {
        let owner_id = (i - 50) % 50 + 1;
        graph.add_variable(Variable {
            id: i,
            name: format!("ref{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 100 + 50),
            scope_depth: 1,
        });
        graph.add_borrow(i, owner_id, false, i as u64 * 100);
    }

    let conflicts = graph.find_conflicts_optimized();
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_find_conflicts_vs_optimized() {
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
        created_at: 250,
        dropped_at: Some(350),
        scope_depth: 1,
    });

    graph.add_borrow(2, 1, true, 200);
    graph.add_borrow(3, 1, true, 250);

    let conflicts1 = graph.find_conflicts();
    let conflicts2 = graph.find_conflicts_optimized();

    assert_eq!(conflicts1.len(), conflicts2.len());
}
