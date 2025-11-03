use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Cycle Detection Tests
// ============================================================================

#[test]
fn test_find_cycles_empty_graph() {
    let graph = OwnershipGraph::new();
    let cycles = graph.find_cycles();
    assert!(cycles.is_empty());
}

#[test]
fn test_find_cycles_no_cycles() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=5 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }
    let cycles = graph.find_cycles();
    assert!(cycles.is_empty());
}

// ============================================================================
// Root and Leaf Tests
// ============================================================================

#[test]
fn test_find_roots() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }
    let roots = graph.find_roots();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].id, 10);
}

#[test]
fn test_find_leaves() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }
    let leaves = graph.find_leaves();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].id, 1);
}

#[test]
fn test_roots_and_leaves_star_topology() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=11 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=11 {
        graph.add_borrow(i, 1, false, i as u64);
    }
    let roots = graph.find_roots();
    assert_eq!(roots.len(), 10);
    let leaves = graph.find_leaves();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].id, 1);
}

// ============================================================================
// Lifetime Range Tests
// ============================================================================

#[test]
fn test_find_by_lifetime_range() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(100 + i as u64 * 10),
            scope_depth: 0,
        });
    }
    let short = graph.find_by_lifetime_range(0, 50);
    assert_eq!(short.len(), 5);
    let medium = graph.find_by_lifetime_range(51, 80);
    assert_eq!(medium.len(), 3);
    let long = graph.find_by_lifetime_range(81, 200);
    assert_eq!(long.len(), 2);
}

#[test]
fn test_find_short_lived() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(1000 + i as u64),
            scope_depth: 0,
        });
    }
    let short = graph.find_short_lived(10);
    assert_eq!(short.len(), 10);
}

#[test]
fn test_find_long_lived() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(1000 + i as u64 * 100),
            scope_depth: 0,
        });
    }
    let long = graph.find_long_lived(5000);
    assert_eq!(long.len(), 51);
}

// ============================================================================
// Overlapping Lifetime Tests
// ============================================================================

#[test]
fn test_find_overlapping_lifetimes() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "target".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    for i in 2..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 50 + i as u64 * 20,
            dropped_at: Some(150 + i as u64 * 20),
            scope_depth: 0,
        });
    }
    let overlapping = graph.find_overlapping_lifetimes(1);
    assert!(overlapping.len() >= 5);
}

#[test]
fn test_overlapping_lifetimes_no_overlap() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "target".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "before".into(),
        type_name: "i32".into(),
        created_at: 0,
        dropped_at: Some(99),
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 3,
        name: "after".into(),
        type_name: "i32".into(),
        created_at: 201,
        dropped_at: Some(300),
        scope_depth: 0,
    });
    let overlapping = graph.find_overlapping_lifetimes(1);
    assert_eq!(overlapping.len(), 0);
}

#[test]
fn test_overlapping_with_never_dropped() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "target".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "eternal".into(),
        type_name: "i32".into(),
        created_at: 50,
        dropped_at: None,
        scope_depth: 0,
    });
    let overlapping = graph.find_overlapping_lifetimes(1);
    assert_eq!(overlapping.len(), 1);
}

// ============================================================================
// Borrow Count Tests
// ============================================================================

#[test]
fn test_find_by_borrow_count() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("owner{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 1..=10 {
        for j in 1..=i {
            let borrower_id = 100 + i * 10 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: "&i32".into(),
                created_at: borrower_id as u64,
                dropped_at: None,
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, false, borrower_id as u64);
        }
    }
    let with_5 = graph.find_by_borrow_count(5);
    assert_eq!(with_5.len(), 1);
    assert_eq!(with_5[0].id, 5);
}

#[test]
fn test_find_heavily_borrowed() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=20 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 1..=20 {
        for j in 1..=i {
            let borrower_id = 100 + i * 20 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: "&i32".into(),
                created_at: borrower_id as u64,
                dropped_at: None,
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, false, borrower_id as u64);
        }
    }
    let heavily = graph.find_heavily_borrowed(10);
    assert_eq!(heavily.len(), 11);
}

// ============================================================================
// Type Category Tests
// ============================================================================

#[test]
fn test_find_by_type_category_references() {
    let mut graph = OwnershipGraph::new();
    let types = ["i32", "&i32", "&mut i32", "String", "&str", "&mut String"];
    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    let refs = graph.find_by_type_category("reference");
    assert_eq!(refs.len(), 4);
    let mut_refs = graph.find_by_type_category("mutable_reference");
    assert_eq!(mut_refs.len(), 2);
}

#[test]
fn test_find_by_type_category_smart_pointers() {
    let mut graph = OwnershipGraph::new();
    let types = ["Box<i32>", "Rc<String>", "Arc<Vec<i32>>", "i32", "String"];
    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    let smart_ptrs = graph.find_by_type_category("smart_pointer");
    assert_eq!(smart_ptrs.len(), 3);
}

#[test]
fn test_find_by_type_category_collections() {
    let mut graph = OwnershipGraph::new();
    let types = [
        "Vec<i32>",
        "HashMap<String, i32>",
        "HashSet<String>",
        "i32",
        "String",
    ];
    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    let collections = graph.find_by_type_category("collection");
    assert_eq!(collections.len(), 3);
}

#[test]
fn test_find_by_type_category_invalid() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });
    let result = graph.find_by_type_category("invalid_category");
    assert!(result.is_empty());
}

// ============================================================================
// Transitive Borrower Tests
// ============================================================================

#[test]
fn test_find_transitive_borrowers() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }
    let transitive = graph.find_transitive_borrowers(1);
    assert_eq!(transitive.len(), 9);
}

#[test]
fn test_transitive_borrowers_diamond() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    graph.add_borrow(2, 1, false, 2);
    graph.add_borrow(3, 1, false, 3);
    graph.add_borrow(4, 2, false, 4);
    graph.add_borrow(4, 3, false, 4);
    graph.add_borrow(5, 4, false, 5);

    let transitive = graph.find_transitive_borrowers(1);
    assert_eq!(transitive.len(), 4);
}

#[test]
fn test_transitive_borrowers_nonexistent() {
    let graph = OwnershipGraph::new();
    let transitive = graph.find_transitive_borrowers(999);
    assert!(transitive.is_empty());
}

// ============================================================================
// Common Borrowers Tests
// ============================================================================

#[test]
fn test_find_common_borrowers() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("owner{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 10..=15 {
        graph.add_variable(Variable {
            id: i,
            name: format!("borrower{}", i),
            type_name: "&i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 1,
        });
    }
    graph.add_borrow(10, 1, false, 10);
    graph.add_borrow(10, 2, false, 10);
    graph.add_borrow(11, 1, false, 11);
    graph.add_borrow(11, 2, false, 11);
    graph.add_borrow(12, 1, false, 12);

    let common = graph.find_common_borrowers(&[1, 2]);
    assert_eq!(common.len(), 2);
}

#[test]
fn test_common_borrowers_empty_input() {
    let graph = OwnershipGraph::new();
    let common = graph.find_common_borrowers(&[]);
    assert!(common.is_empty());
}

#[test]
fn test_common_borrowers_no_common() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    graph.add_borrow(3, 1, false, 3);
    graph.add_borrow(4, 2, false, 4);

    let common = graph.find_common_borrowers(&[1, 2]);
    assert!(common.is_empty());
}

// ============================================================================
// Median and Percentile Tests
// ============================================================================

#[test]
fn test_median_lifetime_odd_count() {
    let mut graph = OwnershipGraph::new();
    let lifetimes = [10, 20, 30, 40, 50];
    for (i, &lifetime) in lifetimes.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(100 + lifetime),
            scope_depth: 0,
        });
    }
    let median = graph.median_lifetime();
    assert_eq!(median, Some(30.0));
}

#[test]
fn test_median_lifetime_even_count() {
    let mut graph = OwnershipGraph::new();
    let lifetimes = [10, 20, 30, 40];
    for (i, &lifetime) in lifetimes.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(100 + lifetime),
            scope_depth: 0,
        });
    }
    let median = graph.median_lifetime();
    assert_eq!(median, Some(25.0));
}

#[test]
fn test_median_lifetime_empty() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.median_lifetime(), None);
}

#[test]
fn test_lifetime_percentile() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(1000 + i as u64),
            scope_depth: 0,
        });
    }
    let p50 = graph.lifetime_percentile(50.0);
    assert_eq!(p50, Some(51));
    let p90 = graph.lifetime_percentile(90.0);
    assert_eq!(p90, Some(90));
    let p99 = graph.lifetime_percentile(99.0);
    assert_eq!(p99, Some(99));
}

#[test]
fn test_lifetime_percentile_invalid() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    assert_eq!(graph.lifetime_percentile(-1.0), None);
    assert_eq!(graph.lifetime_percentile(101.0), None);
}

#[test]
fn test_lifetime_percentile_empty() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.lifetime_percentile(50.0), None);
}
