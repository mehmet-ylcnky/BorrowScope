use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Extreme Scale Tests
// ============================================================================

#[test]
fn test_transitive_borrowers_10k_chain() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=1000 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }
    let transitive = graph.find_transitive_borrowers(1);
    assert_eq!(transitive.len(), 999);
}

#[test]
fn test_overlapping_lifetimes_1000_vars() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: Some(i as u64 + 1000),
            scope_depth: 0,
        });
    }
    let overlapping = graph.find_overlapping_lifetimes(500);
    assert!(overlapping.len() > 500);
}

#[test]
fn test_heavily_borrowed_extreme() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "popular".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });
    for i in 2..=5001 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64);
    }
    let heavily = graph.find_heavily_borrowed(5000);
    assert_eq!(heavily.len(), 1);
    assert_eq!(heavily[0].id, 1);
}

#[test]
fn test_lifetime_percentile_large_dataset() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(1000 + i as u64),
            scope_depth: 0,
        });
    }
    let p1 = graph.lifetime_percentile(1.0).unwrap();
    let p50 = graph.lifetime_percentile(50.0).unwrap();
    let p99 = graph.lifetime_percentile(99.0).unwrap();
    assert!(p1 < p50);
    assert!(p50 < p99);
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_lifetime_range_zero_to_zero() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "instant".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(100),
        scope_depth: 0,
    });
    let result = graph.find_by_lifetime_range(0, 0);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_lifetime_range_u64_max() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "eternal".into(),
        type_name: "i32".into(),
        created_at: 0,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });
    let result = graph.find_by_lifetime_range(u64::MAX - 1000, u64::MAX);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_overlapping_lifetimes_u64_max() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "target".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 2,
        name: "overlapping".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 500,
        dropped_at: None,
        scope_depth: 0,
    });
    let overlapping = graph.find_overlapping_lifetimes(1);
    assert_eq!(overlapping.len(), 1);
}

#[test]
fn test_borrow_count_zero() {
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
    let with_zero = graph.find_by_borrow_count(0);
    assert_eq!(with_zero.len(), 10);
}

// ============================================================================
// Complex Graph Topology Tests
// ============================================================================

#[test]
fn test_transitive_borrowers_complete_graph() {
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
    for i in 2..=20 {
        for j in 1..i {
            graph.add_borrow(i, j, false, i as u64 * 100 + j as u64);
        }
    }
    let transitive = graph.find_transitive_borrowers(1);
    assert_eq!(transitive.len(), 19);
}

#[test]
fn test_common_borrowers_complex() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 51..=100 {
        for j in 1..=50 {
            graph.add_borrow(i, j, false, i as u64 * 100 + j as u64);
        }
    }
    let common = graph.find_common_borrowers(&[1, 2, 3, 4, 5]);
    assert_eq!(common.len(), 50);
}

#[test]
fn test_roots_and_leaves_balanced_tree() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=31 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 2..=31 {
        graph.add_borrow(i, i / 2, false, i as u64);
    }
    let roots = graph.find_roots();
    assert_eq!(roots.len(), 16);
    let leaves = graph.find_leaves();
    assert_eq!(leaves.len(), 1);
}

// ============================================================================
// Type Category Edge Cases
// ============================================================================

#[test]
fn test_type_category_nested_generics() {
    let mut graph = OwnershipGraph::new();
    let types = [
        "Box<Rc<Arc<Vec<i32>>>>",
        "Vec<Box<String>>",
        "HashMap<String, Vec<i32>>",
        "&Box<i32>",
        "&mut Rc<String>",
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
    let smart_ptrs = graph.find_by_type_category("smart_pointer");
    assert_eq!(smart_ptrs.len(), 1); // Only "Box<Rc<Arc<Vec<i32>>>>" starts with Box/Rc/Arc
    let collections = graph.find_by_type_category("collection");
    assert_eq!(collections.len(), 2);
    let refs = graph.find_by_type_category("reference");
    assert_eq!(refs.len(), 2);
}

#[test]
fn test_type_category_all_categories() {
    let mut graph = OwnershipGraph::new();
    let types = [
        "&i32",
        "&mut String",
        "Box<i32>",
        "Rc<String>",
        "Arc<i32>",
        "Vec<i32>",
        "HashMap<String, i32>",
        "HashSet<i32>",
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
    let refs = graph.find_by_type_category("reference");
    let mut_refs = graph.find_by_type_category("mutable_reference");
    let smart_ptrs = graph.find_by_type_category("smart_pointer");
    let collections = graph.find_by_type_category("collection");

    assert_eq!(refs.len(), 2);
    assert_eq!(mut_refs.len(), 1);
    assert_eq!(smart_ptrs.len(), 3);
    assert_eq!(collections.len(), 3);
}

// ============================================================================
// Statistical Edge Cases
// ============================================================================

#[test]
fn test_median_lifetime_single_value() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    let median = graph.median_lifetime();
    assert_eq!(median, Some(100.0));
}

#[test]
fn test_median_with_never_dropped() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: if i <= 3 {
                Some(100 + i as u64 * 10)
            } else {
                None
            },
            scope_depth: 0,
        });
    }
    let median = graph.median_lifetime();
    assert_eq!(median, Some(20.0));
}

#[test]
fn test_percentile_boundaries() {
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
    let p0 = graph.lifetime_percentile(0.0);
    let p100 = graph.lifetime_percentile(100.0);
    assert_eq!(p0, Some(1));
    assert_eq!(p100, Some(100));
}

// ============================================================================
// Lifetime Overlap Stress Tests
// ============================================================================

#[test]
fn test_overlapping_lifetimes_all_overlap() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(200),
            scope_depth: 0,
        });
    }
    let overlapping = graph.find_overlapping_lifetimes(1);
    assert_eq!(overlapping.len(), 99);
}

#[test]
fn test_overlapping_lifetimes_staggered() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 10,
            dropped_at: Some(i as u64 * 10 + 50),
            scope_depth: 0,
        });
    }
    let overlapping = graph.find_overlapping_lifetimes(50);
    assert!(overlapping.len() >= 5);
}

// ============================================================================
// Borrow Depth Tests
// ============================================================================

#[test]
fn test_borrow_depth_single_node() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });
    let depth = graph.borrow_depth(1);
    assert_eq!(depth, 0);
}

#[test]
fn test_borrow_depth_long_chain() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 1..100 {
        graph.add_borrow(i, i + 1, false, i as u64);
    }
    let depth = graph.borrow_depth(100);
    assert_eq!(depth, 99);
}

#[test]
fn test_borrow_depth_nonexistent() {
    let graph = OwnershipGraph::new();
    let depth = graph.borrow_depth(999);
    assert_eq!(depth, 0);
}

// ============================================================================
// Common Borrowers Stress Tests
// ============================================================================

#[test]
fn test_common_borrowers_many_owners() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("owner{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    for i in 101..=200 {
        graph.add_variable(Variable {
            id: i,
            name: format!("borrower{}", i),
            type_name: "&i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 1,
        });
        for j in 1..=100 {
            graph.add_borrow(i, j, false, i as u64);
        }
    }
    let owners: Vec<_> = (1..=100).collect();
    let common = graph.find_common_borrowers(&owners);
    assert_eq!(common.len(), 100);
}

#[test]
fn test_common_borrowers_single_owner() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "owner".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });
    for i in 2..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("r{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 1,
        });
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }
    let common = graph.find_common_borrowers(&[1]);
    assert_eq!(common.len(), 9);
}

// ============================================================================
// Lifetime Range Edge Cases
// ============================================================================

#[test]
fn test_lifetime_range_inverted() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });
    let result = graph.find_by_lifetime_range(200, 100);
    assert!(result.is_empty());
}

#[test]
fn test_short_lived_zero_threshold() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "instant".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(100),
        scope_depth: 0,
    });
    let result = graph.find_short_lived(0);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_long_lived_u64_max_threshold() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 0,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });
    let result = graph.find_long_lived(u64::MAX);
    assert_eq!(result.len(), 1);
}
