use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// DFS Tests
// ============================================================================

#[test]
fn test_dfs_empty_graph() {
    let graph = OwnershipGraph::new();
    let visited = graph.dfs_from(1);
    assert_eq!(visited.len(), 0);
}

#[test]
fn test_dfs_single_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let visited = graph.dfs_from(1);
    assert_eq!(visited, vec![1]);
}

#[test]
fn test_dfs_linear_chain() {
    let mut graph = OwnershipGraph::new();

    // Create chain: 1 <- 2 <- 3 <- 4 <- 5
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

    for i in 2..=5 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let visited = graph.dfs_from(5);
    assert_eq!(visited.len(), 5);
    assert!(visited.contains(&1));
    assert!(visited.contains(&5));
}

#[test]
fn test_dfs_star_topology() {
    let mut graph = OwnershipGraph::new();

    // Center node
    graph.add_variable(Variable {
        id: 1,
        name: "center".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    // 10 nodes borrowing from center
    for i in 2..=11 {
        graph.add_variable(Variable {
            id: i,
            name: format!("spoke{}", i),
            type_name: "&i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
        graph.add_borrow(i, 1, false, i as u64 * 100);
    }

    let visited = graph.dfs_from(2);
    assert_eq!(visited.len(), 2);
    assert!(visited.contains(&1));
    assert!(visited.contains(&2));
}

#[test]
fn test_dfs_disconnected_component() {
    let mut graph = OwnershipGraph::new();

    // Component 1: 1 <- 2
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

    // Component 2: 3 <- 4 (disconnected)
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
        name: "s".into(),
        type_name: "&String".into(),
        created_at: 2100,
        dropped_at: None,
        scope_depth: 0,
    });
    graph.add_borrow(4, 3, false, 2100);

    let visited = graph.dfs_from(2);
    assert_eq!(visited.len(), 2);
    assert!(!visited.contains(&3));
    assert!(!visited.contains(&4));
}

// ============================================================================
// BFS Tests
// ============================================================================

#[test]
fn test_bfs_empty_graph() {
    let graph = OwnershipGraph::new();
    let visited = graph.bfs_from(1);
    assert_eq!(visited.len(), 0);
}

#[test]
fn test_bfs_single_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let visited = graph.bfs_from(1);
    assert_eq!(visited, vec![1]);
}

#[test]
fn test_bfs_level_order() {
    let mut graph = OwnershipGraph::new();

    // Tree structure:
    //       1
    //      / \
    //     2   3
    //    / \
    //   4   5

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

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(5, 2, false, 500);

    let visited = graph.bfs_from(4);
    assert_eq!(visited.len(), 3);
    assert!(visited.contains(&1));
    assert!(visited.contains(&2));
    assert!(visited.contains(&4));
    assert_eq!(visited[0], 4);
}

// ============================================================================
// Shortest Path Tests
// ============================================================================

#[test]
fn test_shortest_path_same_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let path = graph.shortest_path(1, 1);
    assert_eq!(path, Some(vec![1]));
}

#[test]
fn test_shortest_path_direct_edge() {
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

    let path = graph.shortest_path(2, 1);
    assert_eq!(path, Some(vec![2, 1]));
}

#[test]
fn test_shortest_path_chain() {
    let mut graph = OwnershipGraph::new();

    // Chain: 1 <- 2 <- 3 <- 4
    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=4 {
        graph.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let path = graph.shortest_path(4, 1);
    assert_eq!(path, Some(vec![4, 3, 2, 1]));
}

#[test]
fn test_shortest_path_no_path() {
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
        name: "y".into(),
        type_name: "String".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 0,
    });

    let path = graph.shortest_path(1, 2);
    assert_eq!(path, None);
}

#[test]
fn test_shortest_path_nonexistent_nodes() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.shortest_path(1, 2), None);
}

// ============================================================================
// Topological Sort Tests
// ============================================================================

#[test]
fn test_topological_order_empty() {
    let graph = OwnershipGraph::new();
    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 0);
}

#[test]
fn test_topological_order_single_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let order = graph.topological_order().unwrap();
    assert_eq!(order, vec![1]);
}

#[test]
fn test_topological_order_chain() {
    let mut graph = OwnershipGraph::new();

    // Chain: 1 <- 2 <- 3
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

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);

    let order = graph.topological_order().unwrap();
    assert_eq!(order.len(), 3);

    // 3 must come before 2, 2 must come before 1
    let pos_1 = order.iter().position(|&x| x == 1).unwrap();
    let pos_2 = order.iter().position(|&x| x == 2).unwrap();
    let pos_3 = order.iter().position(|&x| x == 3).unwrap();

    assert!(pos_3 < pos_2);
    assert!(pos_2 < pos_1);
}

#[test]
fn test_drop_order() {
    let mut graph = OwnershipGraph::new();

    // x <- r
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

    let order = graph.drop_order();
    assert_eq!(order.len(), 2);

    // r must drop before x
    let pos_1 = order.iter().position(|&x| x == 1).unwrap();
    let pos_2 = order.iter().position(|&x| x == 2).unwrap();
    assert!(pos_2 < pos_1);
}

// ============================================================================
// Connected Components Tests
// ============================================================================

#[test]
fn test_connected_components_empty() {
    let graph = OwnershipGraph::new();
    let components = graph.connected_components();
    assert_eq!(components.len(), 0);
}

#[test]
fn test_connected_components_single() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let components = graph.connected_components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0], vec![1]);
}

#[test]
fn test_connected_components_two_separate() {
    let mut graph = OwnershipGraph::new();

    // Component 1
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

    // Component 2
    graph.add_variable(Variable {
        id: 3,
        name: "y".into(),
        type_name: "String".into(),
        created_at: 2000,
        dropped_at: None,
        scope_depth: 0,
    });

    let components = graph.connected_components();
    assert_eq!(components.len(), 2);
}

// ============================================================================
// Reachability Tests
// ============================================================================

#[test]
fn test_can_reach_same_node() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert!(graph.can_reach(1, 1));
}

#[test]
fn test_can_reach_direct() {
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

    assert!(graph.can_reach(2, 1));
    assert!(!graph.can_reach(1, 2));
}

#[test]
fn test_can_reach_transitive() {
    let mut graph = OwnershipGraph::new();

    // Chain: 1 <- 2 <- 3
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

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);

    assert!(graph.can_reach(3, 1));
    assert!(graph.can_reach(3, 2));
    assert!(!graph.can_reach(1, 3));
}

#[test]
fn test_can_reach_nonexistent() {
    let graph = OwnershipGraph::new();
    assert!(!graph.can_reach(1, 2));
}

// ============================================================================
// Find All Borrowers Tests
// ============================================================================

#[test]
fn test_find_all_borrowers_none() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    let borrowers = graph.find_all_borrowers(1);
    assert_eq!(borrowers.len(), 0);
}

#[test]
fn test_find_all_borrowers_direct() {
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

    let borrowers = graph.find_all_borrowers(1);
    assert_eq!(borrowers, vec![2]);
}

#[test]
fn test_find_all_borrowers_transitive() {
    let mut graph = OwnershipGraph::new();

    // 1 <- 2 <- 3 <- 4
    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);

    let borrowers = graph.find_all_borrowers(1);
    assert_eq!(borrowers.len(), 3);
    assert!(borrowers.contains(&2));
    assert!(borrowers.contains(&3));
    assert!(borrowers.contains(&4));
}

// ============================================================================
// Borrow Depth Tests
// ============================================================================

#[test]
fn test_borrow_depth_leaf() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: None,
        scope_depth: 0,
    });

    assert_eq!(graph.borrow_depth(1), 0);
}

#[test]
fn test_borrow_depth_chain() {
    let mut graph = OwnershipGraph::new();

    // 1 <- 2 <- 3
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

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);

    assert_eq!(graph.borrow_depth(1), 2);
    assert_eq!(graph.borrow_depth(2), 1);
    assert_eq!(graph.borrow_depth(3), 0);
}

#[test]
fn test_borrow_depth_nonexistent() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.borrow_depth(999), 0);
}

// ============================================================================
// Borrow Chain Tests
// ============================================================================

#[test]
fn test_borrow_chain_direct() {
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
        type_name: "&i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_borrow(2, 1, false, 200);

    let chain = graph.borrow_chain(2, 1);
    assert_eq!(chain, Some(vec![2, 1]));
}

#[test]
fn test_borrow_chain_transitive() {
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

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);

    let chain = graph.borrow_chain(3, 1);
    assert_eq!(chain, Some(vec![3, 2, 1]));
}

#[test]
fn test_borrow_chain_no_path() {
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

    assert_eq!(graph.borrow_chain(1, 2), None);
}

#[test]
fn test_borrow_chain_same_variable() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let chain = graph.borrow_chain(1, 1);
    assert_eq!(chain, Some(vec![1]));
}

#[test]
fn test_borrow_chain_nonexistent() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.borrow_chain(1, 2), None);
}
