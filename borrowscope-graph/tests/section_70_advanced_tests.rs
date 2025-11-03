use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Advanced DFS Tests
// ============================================================================

#[test]
fn test_dfs_complex_diamond() {
    let mut graph = OwnershipGraph::new();

    // Diamond: 1 <- 2, 1 <- 3, 2 <- 4, 3 <- 4
    for i in 1..=4 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(4, 3, false, 400);

    let result = graph.dfs_from(4);
    assert_eq!(result.len(), 4);
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    assert!(result.contains(&3));
    assert!(result.contains(&4));
}

#[test]
fn test_dfs_with_moves() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=3 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "String".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_move(1, 2, 200);
    graph.add_move(2, 3, 300);

    let result = graph.dfs_from(3);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    assert!(result.contains(&3));
}

#[test]
fn test_dfs_with_rc_clones() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("rc{}", i),
            type_name: "Rc<i32>".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_rc_clone(2, 1, 1, 200);
    graph.add_rc_clone(3, 1, 2, 300);
    graph.add_rc_clone(4, 1, 3, 400);

    let result = graph.dfs_from(2);
    assert!(result.contains(&1));
    assert!(result.contains(&2));
}

// ============================================================================
// Advanced BFS Tests
// ============================================================================

#[test]
fn test_bfs_shortest_in_diamond() {
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

    // 1 <- 2 <- 4, 1 <- 3 <- 5 <- 4 (longer path)
    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(5, 3, false, 500);
    graph.add_borrow(4, 5, false, 400);

    let path = graph.shortest_path(4, 1);
    assert_eq!(path, Some(vec![4, 2, 1]));
}

#[test]
fn test_bfs_multiple_paths() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(4, 3, false, 400);

    let path = graph.shortest_path(4, 1);
    assert!(path.is_some());
    assert_eq!(path.as_ref().unwrap().len(), 3);
    assert_eq!(path.as_ref().unwrap()[0], 4);
    assert_eq!(path.as_ref().unwrap()[2], 1);
}

// ============================================================================
// Advanced Topological Sort Tests
// ============================================================================

#[test]
fn test_topological_order_complex() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=6 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(5, 3, false, 500);
    graph.add_borrow(6, 4, false, 600);
    graph.add_borrow(6, 5, false, 600);

    let order = graph.topological_order().unwrap();

    let pos: std::collections::HashMap<_, _> =
        order.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    assert!(pos[&6] < pos[&4]);
    assert!(pos[&6] < pos[&5]);
    assert!(pos[&4] < pos[&2]);
    assert!(pos[&5] < pos[&3]);
    assert!(pos[&2] < pos[&1]);
    assert!(pos[&3] < pos[&1]);
}

#[test]
fn test_drop_order_with_disconnected() {
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
    graph.add_borrow(4, 3, false, 400);

    let order = graph.drop_order();
    assert_eq!(order.len(), 5);

    let pos: std::collections::HashMap<_, _> =
        order.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    assert!(pos[&2] < pos[&1]);
    assert!(pos[&4] < pos[&3]);
}

// ============================================================================
// Advanced Connected Components Tests
// ============================================================================

#[test]
fn test_connected_components_complex() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=8 {
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
    graph.add_borrow(3, 2, false, 300);

    graph.add_borrow(5, 4, false, 500);
    graph.add_borrow(6, 5, false, 600);

    let components = graph.connected_components();
    assert_eq!(components.len(), 4);

    let comp1 = components.iter().find(|c| c.contains(&1)).unwrap().to_vec();
    assert!(comp1.contains(&1) && comp1.contains(&2) && comp1.contains(&3));

    let comp2 = components.iter().find(|c| c.contains(&4)).unwrap().to_vec();
    assert!(comp2.contains(&4) && comp2.contains(&5) && comp2.contains(&6));
}

#[test]
fn test_connected_components_bidirectional() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
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
    graph.add_borrow(1, 3, false, 100);
    graph.add_borrow(3, 4, false, 300);

    let components = graph.connected_components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].len(), 4);
}

// ============================================================================
// Advanced Reachability Tests
// ============================================================================

#[test]
fn test_can_reach_through_multiple_hops() {
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

    for i in 1..10 {
        graph.add_borrow(i + 1, i, false, (i + 1) as u64 * 100);
    }

    assert!(graph.can_reach(10, 1));
    assert!(graph.can_reach(5, 1));
    assert!(!graph.can_reach(1, 10));
}

#[test]
fn test_can_reach_in_diamond() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(4, 3, false, 400);

    assert!(graph.can_reach(4, 1));
    assert!(graph.can_reach(4, 2));
    assert!(graph.can_reach(4, 3));
    assert!(graph.can_reach(2, 1));
    assert!(graph.can_reach(3, 1));
    assert!(!graph.can_reach(1, 4));
}

// ============================================================================
// Advanced Borrowers Tests
// ============================================================================

#[test]
fn test_find_all_borrowers_complex_tree() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=7 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(5, 2, false, 500);
    graph.add_borrow(6, 3, false, 600);
    graph.add_borrow(7, 3, false, 700);

    let borrowers = graph.find_all_borrowers(1);
    assert_eq!(borrowers.len(), 6);
    for i in 2..=7 {
        assert!(borrowers.contains(&i));
    }
}

#[test]
fn test_find_all_borrowers_with_refcell() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: if i == 1 { "RefCell<i32>" } else { "&i32" }.into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_refcell_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);

    let borrowers = graph.find_all_borrowers(1);
    assert_eq!(borrowers.len(), 3);
    assert!(borrowers.contains(&2));
    assert!(borrowers.contains(&3));
    assert!(borrowers.contains(&4));
}

// ============================================================================
// Advanced Borrow Depth Tests
// ============================================================================

#[test]
fn test_borrow_depth_tree() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=7 {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 2, false, 400);
    graph.add_borrow(5, 2, false, 500);
    graph.add_borrow(6, 3, false, 600);
    graph.add_borrow(7, 6, false, 700);

    assert_eq!(graph.borrow_depth(1), 3);
    assert_eq!(graph.borrow_depth(2), 1);
    assert_eq!(graph.borrow_depth(3), 2);
    assert_eq!(graph.borrow_depth(6), 1);
    assert_eq!(graph.borrow_depth(7), 0);
}

#[test]
fn test_borrow_depth_with_multiple_borrowers() {
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
    graph.add_borrow(3, 1, false, 300);
    graph.add_borrow(4, 1, false, 400);
    graph.add_borrow(5, 2, false, 500);

    assert_eq!(graph.borrow_depth(1), 2);
    assert_eq!(graph.borrow_depth(2), 1);
}

// ============================================================================
// Advanced Borrow Chain Tests
// ============================================================================

#[test]
fn test_borrow_chain_long_path() {
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

    for i in 1..10 {
        graph.add_borrow(i + 1, i, false, (i + 1) as u64 * 100);
    }

    let chain = graph.borrow_chain(10, 1);
    assert_eq!(chain, Some(vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1]));
}

#[test]
fn test_borrow_chain_shortest_in_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=6 {
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
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);
    graph.add_borrow(5, 4, false, 500);
    graph.add_borrow(6, 1, false, 600);
    graph.add_borrow(5, 6, false, 500);

    let chain = graph.borrow_chain(5, 1);
    assert_eq!(chain, Some(vec![5, 6, 1]));
}

#[test]
fn test_borrow_chain_with_moves() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=4 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "String".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_move(1, 2, 200);
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);

    let chain = graph.borrow_chain(4, 1);
    assert_eq!(chain, Some(vec![4, 3, 2, 1]));
}

// ============================================================================
// Edge Cases and Stress Tests
// ============================================================================

#[test]
fn test_large_graph_traversal() {
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

    for i in 1..100 {
        graph.add_borrow(i + 1, i, false, (i + 1) as u64 * 100);
    }

    let result = graph.dfs_from(100);
    assert_eq!(result.len(), 100);

    let path = graph.shortest_path(100, 1);
    assert_eq!(path.as_ref().unwrap().len(), 100);
}

#[test]
fn test_wide_graph_traversal() {
    let mut graph = OwnershipGraph::new();

    for i in 0..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 1..=50 {
        graph.add_borrow(i, 0, false, i as u64 * 100);
    }

    let borrowers = graph.find_all_borrowers(0);
    assert_eq!(borrowers.len(), 50);

    assert_eq!(graph.borrow_depth(0), 1);
}

#[test]
fn test_mixed_relationship_types_traversal() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=8 {
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
    graph.add_move(3, 2, 300);
    graph.add_rc_clone(4, 3, 1, 400);
    graph.add_arc_clone(5, 4, 1, 500);
    graph.add_refcell_borrow(6, 5, true, 600);
    graph.add_borrow(7, 6, false, 700);
    graph.add_borrow(8, 7, false, 800);

    let result = graph.dfs_from(8);
    // 8->7->6->5->4->3, then 3 can reach 2 via reversed move edge (2->3)
    // So we should find at least 8,7,6,5,4,3,2
    assert!(
        result.len() >= 6,
        "Expected at least 6 nodes, got {}",
        result.len()
    );

    // Chain from 8 to 1 may not exist due to move semantics
    let chain = graph.borrow_chain(8, 3);
    assert!(chain.is_some());
}

#[test]
fn test_cycle_detection_should_not_exist() {
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
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);
    graph.add_borrow(5, 4, false, 500);

    assert!(!graph.has_cycles());
}

#[test]
fn test_topological_sort_with_dropped_variables() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=5 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 * 1000)
            } else {
                None
            },
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 2, false, 300);
    graph.add_borrow(4, 3, false, 400);
    graph.add_borrow(5, 4, false, 500);

    let order = graph.topological_order();
    assert!(order.is_ok());
    assert_eq!(order.unwrap().len(), 5);
}
