use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_large_graph() {
    let mut graph = OwnershipGraph::with_capacity(10000, 20000);

    for i in 0..10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 10000);
}

#[test]
fn test_large_graph_with_edges() {
    let mut graph = OwnershipGraph::with_capacity(1000, 2000);

    // Create chain: 0 <- 1 <- 2 <- ... <- 999
    for i in 0..1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..1000 {
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    assert_eq!(graph.node_count(), 1000);
    assert_eq!(graph.edge_count(), 999);
}

#[test]
fn test_zero_timestamps() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 0,
        dropped_at: Some(0),
        scope_depth: 0,
    });

    assert!(!graph.is_alive(1, 0));
}

#[test]
fn test_max_timestamps() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });

    assert!(graph.is_alive(1, u64::MAX - 1));
    assert!(!graph.is_alive(1, u64::MAX));
}

#[test]
fn test_empty_variable_name() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: String::new(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.name, "");
}

#[test]
fn test_long_variable_name() {
    let mut graph = OwnershipGraph::new();
    let long_name = "a".repeat(10000);

    graph.add_variable(Variable {
        id: 1,
        name: long_name.clone(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.name.len(), 10000);
}

#[test]
fn test_unicode_variable_names() {
    let mut graph = OwnershipGraph::new();

    let names = ["变量", "переменная", "μεταβλητή", "🦀", "x̃ỹz̃"];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 5);
}

#[test]
fn test_special_characters_in_names() {
    let mut graph = OwnershipGraph::new();

    let names = ["x\n", "y\t", "z\"", "a'b", "c\\d", "e/f"];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 6);
}

#[test]
fn test_complex_type_names() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Vec<HashMap<String, Arc<Mutex<RefCell<Box<dyn Trait>>>>>>",
        "&'static str",
        "impl Iterator<Item = Result<T, E>>",
        "fn(i32, i32) -> i32",
        "[u8; 1024]",
    ];

    for (i, type_name) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: type_name.to_string(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 5);
}

#[test]
fn test_max_scope_depth() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "deeply_nested".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: usize::MAX,
    });

    let var = graph.get_variable(1).unwrap();
    assert_eq!(var.scope_depth, usize::MAX);
}

#[test]
fn test_clear_and_reuse() {
    let mut graph = OwnershipGraph::new();

    // Add some data
    for i in 0..100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 100);

    // Clear
    graph.clear();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    // Reuse with same IDs
    for i in 0..50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("new_var_{}", i),
            type_name: "String".into(),
            created_at: i as u64 + 1000,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    assert_eq!(graph.node_count(), 50);
}

#[test]
fn test_multiple_edges_same_nodes() {
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

    // Add multiple borrows at different times
    graph.add_borrow(2, 1, false, 1100);
    graph.add_borrow(2, 1, false, 1200);
    graph.add_borrow(2, 1, false, 1300);

    // Should have 3 edges
    assert_eq!(graph.edge_count(), 3);
}

#[test]
fn test_disconnected_components() {
    let mut graph = OwnershipGraph::new();

    // Component 1: x -> r1
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

    // Component 2: y -> r2
    graph.add_variable(Variable {
        id: 3,
        name: "y".into(),
        type_name: "String".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_variable(Variable {
        id: 4,
        name: "r2".into(),
        type_name: "&String".into(),
        created_at: 2100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(4, 3, false, 2100);

    // Isolated node
    graph.add_variable(Variable {
        id: 5,
        name: "z".into(),
        type_name: "f64".into(),
        created_at: 3000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert_eq!(graph.node_count(), 5);
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn test_star_topology() {
    let mut graph = OwnershipGraph::new();

    // Central node
    graph.add_variable(Variable {
        id: 0,
        name: "center".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // 100 nodes borrowing from center
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("spoke_{}", i),
            type_name: "&i32".into(),
            created_at: 1000 + i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_borrow(i, 0, false, 1000 + i as u64);
    }

    assert_eq!(graph.node_count(), 101);
    assert_eq!(graph.edge_count(), 100);
    assert_eq!(graph.borrowers_of(0).len(), 100);
}

#[test]
fn test_linear_chain() {
    let mut graph = OwnershipGraph::new();

    // Create chain: 0 <- 1 <- 2 <- ... <- 99
    for i in 0..100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("node_{}", i),
            type_name: if i == 0 {
                "i32".into()
            } else {
                format!("&{}", "i32")
            },
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });

        if i > 0 {
            graph.add_borrow(i, i - 1, false, i as u64 * 100);
        }
    }

    assert_eq!(graph.node_count(), 100);
    assert_eq!(graph.edge_count(), 99);
}

#[test]
fn test_all_variables_iterator_empty() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.all_variables().count(), 0);
}

#[test]
fn test_all_variables_iterator_ordering() {
    let mut graph = OwnershipGraph::new();

    for i in 0..10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let vars: Vec<_> = graph.all_variables().collect();
    assert_eq!(vars.len(), 10);
}

#[test]
fn test_borrows_empty() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert_eq!(graph.borrows(1).len(), 0);
}

#[test]
fn test_borrows_single() {
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

    graph.add_borrow(2, 1, false, 1100);

    let borrows = graph.borrows(2);
    assert_eq!(borrows.len(), 1);
    assert_eq!(borrows[0].name, "x");
}

#[test]
fn test_borrowers_of_nonexistent() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.borrowers_of(999).len(), 0);
}

#[test]
fn test_get_variable_nonexistent() {
    let graph = OwnershipGraph::new();
    assert!(graph.get_variable(999).is_none());
}

#[test]
fn test_concurrent_lifetime_ranges() {
    let mut graph = OwnershipGraph::new();

    // Variable with specific lifetime
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    // Borrow that outlives owner (invalid but graph allows it)
    graph.add_variable(Variable {
        id: 2,
        name: "r".into(),
        type_name: "&i32".into(),
        created_at: 1500,
        dropped_at: Some(3000),
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 1500);

    assert!(graph.is_alive(1, 1500));
    assert!(!graph.is_alive(1, 2500));
    assert!(graph.is_alive(2, 2500));
}

#[test]
fn test_instant_lifetime() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "temp".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(1000),
        scope_depth: 0,
    });

    assert!(!graph.is_alive(1, 1000));
}
