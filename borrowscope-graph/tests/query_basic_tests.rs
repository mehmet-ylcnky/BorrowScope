use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Basic Query Tests
// ============================================================================

#[test]
fn test_find_by_name_single() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let result = graph.find_by_name("x");
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, 1);
}

#[test]
fn test_find_by_name_not_found() {
    let graph = OwnershipGraph::new();
    let result = graph.find_by_name("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_find_all_by_name_multiple() {
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

    let results = graph.find_all_by_name("x");
    assert_eq!(results.len(), 5);
}

#[test]
fn test_find_by_type() {
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

    graph.add_variable(Variable {
        id: 3,
        name: "z".into(),
        type_name: "i32".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 0,
    });

    let results = graph.find_by_type("i32");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_find_references() {
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
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 1,
    });

    let refs = graph.find_references();
    assert_eq!(refs.len(), 2);
}

#[test]
fn test_find_mutable_references() {
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
        dropped_at: None,
        scope_depth: 1,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "r2".into(),
        type_name: "&mut i32".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 1,
    });

    let mut_refs = graph.find_mutable_references();
    assert_eq!(mut_refs.len(), 1);
    assert_eq!(mut_refs[0].id, 3);
}

// ============================================================================
// Temporal Query Tests
// ============================================================================

#[test]
fn test_alive_at() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(300),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: Some(400),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "z".into(),
        type_name: "i32".into(),
        created_at: 350,
        dropped_at: None,
        scope_depth: 0,
    });

    let alive_at_250 = graph.alive_at(250);
    assert_eq!(alive_at_250.len(), 2);

    let alive_at_350 = graph.alive_at(350);
    assert_eq!(alive_at_350.len(), 2);

    let alive_at_500 = graph.alive_at(500);
    assert_eq!(alive_at_500.len(), 1);
}

#[test]
fn test_created_between() {
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

    let results = graph.created_between(200, 500);
    assert_eq!(results.len(), 4);
}

#[test]
fn test_dropped_between() {
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

    let results = graph.dropped_between(400, 800);
    assert_eq!(results.len(), 3);
}

// ============================================================================
// Scope Query Tests
// ============================================================================

#[test]
fn test_find_by_scope_depth() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: i % 3,
        });
    }

    let depth_0 = graph.find_by_scope_depth(0);
    assert_eq!(depth_0.len(), 3);

    let depth_1 = graph.find_by_scope_depth(1);
    assert_eq!(depth_1.len(), 4);

    let depth_2 = graph.find_by_scope_depth(2);
    assert_eq!(depth_2.len(), 3);
}

// ============================================================================
// Pattern Matching Tests
// ============================================================================

#[test]
fn test_find_by_name_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "my_var".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "your_var".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "other".into(),
        type_name: "i32".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 0,
    });

    let results = graph.find_by_name_pattern("var");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_find_by_type_pattern() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Vec<i32>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "Vec<String>".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "z".into(),
        type_name: "HashMap<String, i32>".into(),
        created_at: 300,
        dropped_at: None,
        scope_depth: 0,
    });

    let results = graph.find_by_type_pattern("Vec");
    assert_eq!(results.len(), 2);
}

// ============================================================================
// Extrema Query Tests
// ============================================================================

#[test]
fn test_oldest_newest_variable() {
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

    let oldest = graph.oldest_variable();
    assert_eq!(oldest.unwrap().id, 1);

    let newest = graph.newest_variable();
    assert_eq!(newest.unwrap().id, 10);
}

#[test]
fn test_longest_shortest_lived() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "short".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(110),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "long".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: Some(1000),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 3,
        name: "never_dropped".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let longest = graph.longest_lived_variable();
    assert_eq!(longest.unwrap().id, 2);

    let shortest = graph.shortest_lived_variable();
    assert_eq!(shortest.unwrap().id, 1);
}

#[test]
fn test_average_lifetime() {
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
        dropped_at: Some(300),
        scope_depth: 0,
    });

    let avg = graph.average_lifetime();
    assert_eq!(avg, Some(150.0));
}

#[test]
fn test_variables_by_lifetime() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: Some(100 + i as u64 * 100),
            scope_depth: 0,
        });
    }

    let sorted = graph.variables_by_lifetime();
    assert_eq!(sorted.len(), 5);
    assert_eq!(sorted[0].0.id, 5);
    assert_eq!(sorted[4].0.id, 1);
}
