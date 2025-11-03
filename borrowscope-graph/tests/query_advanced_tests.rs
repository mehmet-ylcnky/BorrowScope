use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Query Builder Tests
// ============================================================================

#[test]
fn test_query_builder_by_name() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let results = graph.query().by_name("x").collect();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_query_builder_chaining() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: if i % 2 == 0 { "even" } else { "odd" }.into(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i > 5 { Some(i as u64 * 200) } else { None },
            scope_depth: i % 3,
        });
    }

    let results = graph
        .query()
        .by_name("even")
        .and_in_scope(0)
        .and_dropped()
        .collect();

    assert!(!results.is_empty());
}

#[test]
fn test_query_builder_alive_at() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: Some(i as u64 * 200),
            scope_depth: 0,
        });
    }

    let results = graph.query().alive_at(500).collect();
    assert!(!results.is_empty());
}

#[test]
fn test_query_builder_count() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=20 {
        graph.add_variable(Variable {
            id: i,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let count = graph.query().by_name("x").count();
    assert_eq!(count, 20);
}

#[test]
fn test_query_builder_first() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let first = graph.query().by_name("x").first();
    assert!(first.is_some());
    assert_eq!(first.unwrap().id, 1);
}

#[test]
fn test_query_builder_ids() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let ids = graph.query().by_name("x").ids();
    assert_eq!(ids.len(), 5);
    assert!(ids.contains(&1));
    assert!(ids.contains(&5));
}

#[test]
fn test_query_builder_names() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=3 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let names = graph.query().all().names();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"var1"));
    assert!(names.contains(&"var3"));
}

#[test]
fn test_query_builder_types() {
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
        type_name: "String".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let types = graph.query().all().types();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"i32"));
    assert!(types.contains(&"String"));
}

// ============================================================================
// Complex Filter Tests
// ============================================================================

#[test]
fn test_query_created_after() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let results = graph.query().all().and_created_after(500).collect();
    assert_eq!(results.len(), 5);
}

#[test]
fn test_query_created_before() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let results = graph.query().all().and_created_before(500).collect();
    assert_eq!(results.len(), 4);
}

#[test]
fn test_query_dropped_filter() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
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

    let dropped = graph.query().all().and_dropped().collect();
    assert_eq!(dropped.len(), 5);

    let not_dropped = graph.query().all().and_not_dropped().collect();
    assert_eq!(not_dropped.len(), 5);
}

#[test]
fn test_query_multiple_filters() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=20 {
        graph.add_variable(Variable {
            id: i,
            name: if i % 2 == 0 { "even" } else { "odd" }.into(),
            type_name: if i % 3 == 0 { "i64" } else { "i32" }.into(),
            created_at: i as u64 * 100,
            dropped_at: if i > 10 { Some(i as u64 * 200) } else { None },
            scope_depth: i % 4,
        });
    }

    let results = graph
        .query()
        .by_name("even")
        .and_by_type("i32")
        .and_in_scope(0)
        .and_not_dropped()
        .collect();

    assert!(!results.is_empty());
    for var in results {
        assert_eq!(var.name, "even");
        assert_eq!(var.type_name, "i32");
        assert_eq!(var.scope_depth, 0);
        assert!(var.dropped_at.is_none());
    }
}

// ============================================================================
// Graph Analysis Tests
// ============================================================================

#[test]
fn test_degree_centrality() {
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
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let centrality = graph.degree_centrality();
    assert!(!centrality.is_empty());
    assert!(centrality.contains_key(&1));
}

#[test]
fn test_longest_borrow_chain() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=10 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let chain = graph.longest_borrow_chain();
    assert_eq!(chain.len(), 10);
}

#[test]
fn test_most_borrowed_variable() {
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
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let most_borrowed = graph.most_borrowed_variable();
    assert!(most_borrowed.is_some());
    assert_eq!(most_borrowed.unwrap().id, 1);
}

#[test]
fn test_least_borrowed_variable() {
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

    for i in 2..=4 {
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let least_borrowed = graph.least_borrowed_variable();
    assert!(least_borrowed.is_some());
}

#[test]
fn test_variables_with_no_borrowers() {
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

    graph.add_borrow(2, 1, false, 200);

    let no_borrowers = graph.variables_with_no_borrowers();
    assert_eq!(no_borrowers.len(), 4);
}

#[test]
fn test_variables_with_no_borrows() {
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

    graph.add_borrow(2, 1, false, 200);

    let no_borrows = graph.variables_with_no_borrows();
    assert_eq!(no_borrows.len(), 4);
}

// ============================================================================
// Empty Graph Tests
// ============================================================================

#[test]
fn test_queries_on_empty_graph() {
    let graph = OwnershipGraph::new();

    assert!(graph.find_by_name("x").is_none());
    assert!(graph.find_by_type("i32").is_empty());
    assert!(graph.find_references().is_empty());
    assert!(graph.alive_at(100).is_empty());
    assert!(graph.oldest_variable().is_none());
    assert!(graph.newest_variable().is_none());
    assert!(graph.average_lifetime().is_none());
    assert!(graph.most_borrowed_variable().is_none());
    assert!(graph.longest_borrow_chain().is_empty());
}

// ============================================================================
// Large Scale Query Tests
// ============================================================================

#[test]
fn test_query_performance_1000_variables() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i % 100),
            type_name: if i % 2 == 0 { "i32" } else { "String" }.into(),
            created_at: i as u64 * 100,
            dropped_at: if i % 3 == 0 {
                Some(i as u64 * 200)
            } else {
                None
            },
            scope_depth: i % 10,
        });
    }

    let results = graph.find_by_type("i32");
    assert_eq!(results.len(), 500);

    let alive = graph.alive_at(50000);
    assert!(!alive.is_empty());

    let in_scope = graph.find_by_scope_depth(5);
    assert_eq!(in_scope.len(), 100);
}

#[test]
fn test_complex_query_chain_large_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=500 {
        graph.add_variable(Variable {
            id: i,
            name: if i % 5 == 0 { "target" } else { "other" }.into(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i > 250 { Some(i as u64 * 200) } else { None },
            scope_depth: i % 10,
        });
    }

    let results = graph
        .query()
        .by_name("target")
        .and_created_after(10000)
        .and_created_before(40000)
        .and_dropped()
        .collect();

    assert!(!results.is_empty());
}
