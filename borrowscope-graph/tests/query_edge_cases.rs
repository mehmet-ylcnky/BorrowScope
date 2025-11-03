use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Unicode and Special Character Tests
// ============================================================================

#[test]
fn test_query_unicode_names() {
    let mut graph = OwnershipGraph::new();

    let names = ["变量", "переменная", "μεταβλητή", "متغير"];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let result = graph.find_by_name("变量");
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, 1);
}

#[test]
fn test_query_emoji_names() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "🦀".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let result = graph.find_by_name("🦀");
    assert!(result.is_some());
}

#[test]
fn test_query_empty_name() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let result = graph.find_by_name("");
    assert!(result.is_some());
}

#[test]
fn test_query_special_characters_in_names() {
    let mut graph = OwnershipGraph::new();

    let names = [
        "var\"with\\quotes",
        "var\nwith\nnewlines",
        "var\twith\ttabs",
    ];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let result = graph.find_by_name("var\"with\\quotes");
    assert!(result.is_some());
}

// ============================================================================
// Extreme Timestamp Tests
// ============================================================================

#[test]
fn test_query_u64_max_timestamps() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });

    let alive = graph.alive_at(u64::MAX - 500);
    assert_eq!(alive.len(), 1);

    let created = graph.created_between(u64::MAX - 2000, u64::MAX);
    assert_eq!(created.len(), 1);
}

#[test]
fn test_query_zero_timestamps() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 0,
            dropped_at: Some(0),
            scope_depth: 0,
        });
    }

    let alive = graph.alive_at(0);
    assert_eq!(alive.len(), 0);

    let created = graph.created_between(0, 0);
    assert_eq!(created.len(), 10);
}

#[test]
fn test_query_timestamp_boundaries() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    let alive_at_100 = graph.alive_at(100);
    assert_eq!(alive_at_100.len(), 1);

    let alive_at_200 = graph.alive_at(200);
    assert_eq!(alive_at_200.len(), 0);

    let alive_at_199 = graph.alive_at(199);
    assert_eq!(alive_at_199.len(), 1);
}

// ============================================================================
// Scope Depth Edge Cases
// ============================================================================

#[test]
fn test_query_max_scope_depth() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: usize::MAX,
    });

    let results = graph.find_by_scope_depth(usize::MAX);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_query_1000_scope_depths() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: i,
        });
    }

    for depth in 1..=1000 {
        let results = graph.find_by_scope_depth(depth);
        assert_eq!(results.len(), 1);
    }
}

// ============================================================================
// Complex Type Pattern Tests
// ============================================================================

#[test]
fn test_query_generic_types() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Vec<i32>",
        "Vec<String>",
        "HashMap<String, i32>",
        "Option<Vec<i32>>",
        "Result<i32, Error>",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let vec_types = graph.find_by_type_pattern("Vec");
    assert_eq!(vec_types.len(), 3);

    let i32_types = graph.find_by_type_pattern("i32");
    assert_eq!(i32_types.len(), 4);
}

#[test]
fn test_query_lifetime_annotations() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "&'a i32",
        "&'static str",
        "Cow<'a, str>",
        "Box<dyn Trait + 'a>",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let with_lifetime = graph.find_by_type_pattern("'a");
    assert_eq!(with_lifetime.len(), 3);
}

// ============================================================================
// Centrality and Analysis Edge Cases
// ============================================================================

#[test]
fn test_degree_centrality_empty_graph() {
    let graph = OwnershipGraph::new();
    let centrality = graph.degree_centrality();
    assert!(centrality.is_empty());
}

#[test]
fn test_degree_centrality_single_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let centrality = graph.degree_centrality();
    assert!(centrality.is_empty());
}

#[test]
fn test_degree_centrality_star_topology() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let centrality = graph.degree_centrality();
    let center_centrality = centrality.get(&1).unwrap();
    assert!(*center_centrality > 0.9);
}

#[test]
fn test_longest_chain_disconnected_components() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=20 {
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

    for i in 12..=20 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let chain = graph.longest_borrow_chain();
    assert!(chain.len() >= 9);
}

// ============================================================================
// Lifetime Analysis Edge Cases
// ============================================================================

#[test]
fn test_average_lifetime_with_never_dropped() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "v1".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "v2".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let avg = graph.average_lifetime();
    assert_eq!(avg, Some(100.0));
}

#[test]
fn test_variables_by_lifetime_large_scale() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(100 + i as u64),
            scope_depth: 0,
        });
    }

    let sorted = graph.variables_by_lifetime();
    assert_eq!(sorted.len(), 1000);
    assert_eq!(sorted[0].0.id, 1000);
    assert_eq!(sorted[999].0.id, 1);
}

// ============================================================================
// Query Builder Edge Cases
// ============================================================================

#[test]
fn test_query_builder_no_results() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let results = graph.query().by_name("nonexistent").collect();
    assert!(results.is_empty());

    let count = graph.query().by_name("nonexistent").count();
    assert_eq!(count, 0);

    let first = graph.query().by_name("nonexistent").first();
    assert!(first.is_none());
}

#[test]
fn test_query_builder_all_filters_applied() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: if i == 50 { "target" } else { "other" }.into(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i == 50 { Some(10000) } else { None },
            scope_depth: if i == 50 { 5 } else { 0 },
        });
    }

    let results = graph
        .query()
        .by_name("target")
        .and_by_type("i32")
        .and_in_scope(5)
        .and_dropped()
        .and_created_after(4000)
        .and_created_before(6000)
        .collect();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 50);
}

#[test]
fn test_query_builder_contradictory_filters() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(200),
        scope_depth: 0,
    });

    let results = graph
        .query()
        .all()
        .and_dropped()
        .and_not_dropped()
        .collect();

    assert_eq!(results.len(), 0);
}

// ============================================================================
// Pattern Matching Edge Cases
// ============================================================================

#[test]
fn test_pattern_matching_case_sensitive() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "MyVar".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let result = graph.find_by_name_pattern("myvar");
    assert_eq!(result.len(), 0);

    let result = graph.find_by_name_pattern("MyVar");
    assert_eq!(result.len(), 1);
}

#[test]
fn test_pattern_matching_partial() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("prefix_{}_suffix", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let with_prefix = graph.find_by_name_pattern("prefix");
    assert_eq!(with_prefix.len(), 10);

    let with_suffix = graph.find_by_name_pattern("suffix");
    assert_eq!(with_suffix.len(), 10);

    let with_5 = graph.find_by_name_pattern("_5_");
    assert_eq!(with_5.len(), 1);
}

#[test]
fn test_type_pattern_complex_generics() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Vec<i32>",
        "Vec<String>",
        "HashMap<String, Vec<i32>>",
        "Option<Vec<i32>>",
        "Arc<Mutex<Vec<i32>>>",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let with_vec = graph.find_by_type_pattern("Vec<i32>");
    assert_eq!(with_vec.len(), 4);

    let with_string = graph.find_by_type_pattern("String");
    assert_eq!(with_string.len(), 2);
}

// ============================================================================
// Analysis Function Edge Cases
// ============================================================================

#[test]
fn test_most_borrowed_with_ties() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=3 {
        graph.add_variable(Variable {
            id: i,
            name: format!("owner{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..=3 {
        for j in 1..=5 {
            let borrower_id = i * 10 + j;
            graph.add_variable(Variable {
                id: borrower_id,
                name: format!("r{}_{}", i, j),
                type_name: "&i32".into(),
                created_at: borrower_id as u64 * 100,
                dropped_at: None,
                scope_depth: 1,
            });
            graph.add_borrow(borrower_id, i, false, borrower_id as u64 * 100);
        }
    }

    let most_borrowed = graph.most_borrowed_variable();
    assert!(most_borrowed.is_some());
}

#[test]
fn test_longest_chain_with_cycles_prevention() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let chain = graph.longest_borrow_chain();
    assert_eq!(chain.len(), 100);
}

#[test]
fn test_variables_with_no_borrowers_large_graph() {
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

    for i in 2..=500 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    let no_borrowers = graph.variables_with_no_borrowers();
    assert!(no_borrowers.len() >= 500);
}

// ============================================================================
// Performance Stress Tests
// ============================================================================

#[test]
fn test_query_performance_10k_variables() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i % 1000),
            type_name: if i % 2 == 0 { "i32" } else { "String" }.into(),
            created_at: i as u64,
            dropped_at: if i % 3 == 0 {
                Some(i as u64 + 1000)
            } else {
                None
            },
            scope_depth: i % 100,
        });
    }

    let by_type = graph.find_by_type("i32");
    assert_eq!(by_type.len(), 5000);

    let by_name = graph.find_all_by_name("var500");
    assert_eq!(by_name.len(), 10);

    let alive = graph.alive_at(5000);
    assert!(!alive.is_empty());
}

#[test]
fn test_complex_query_chain_performance() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5000 {
        graph.add_variable(Variable {
            id: i,
            name: if i % 10 == 0 { "target" } else { "other" }.into(),
            type_name: "i32".into(),
            created_at: i as u64 * 10,
            dropped_at: if i > 2500 { Some(i as u64 * 20) } else { None },
            scope_depth: i % 50,
        });
    }

    let results = graph
        .query()
        .by_name("target")
        .and_created_after(10000)
        .and_created_before(40000)
        .and_in_scope(0)
        .and_dropped()
        .collect();

    assert!(!results.is_empty());
}

#[test]
fn test_degree_centrality_dense_graph() {
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

    for i in 2..=100 {
        for j in 1..i {
            if (i * j) % 11 == 0 {
                graph.add_borrow(i, j, false, i as u64);
            }
        }
    }

    let centrality = graph.degree_centrality();
    assert_eq!(centrality.len(), 100);
}
