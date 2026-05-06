//! Comprehensive tests for Milestone 2: Graph Traversal Algorithms.

use borrowscope_graph::*;
use borrowscope_graph::traversal::*;

// ═══════════════════════════════════════════════════════════════════════════
// Test fixtures
// ═══════════════════════════════════════════════════════════════════════════

/// A -> B -> C (linear chain via borrows)
fn linear_graph() -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "&i32", 10);
    let c = g.add_variable("c", "&&i32", 20);
    g.add_borrow(b, a, false, 10);
    g.add_borrow(c, b, false, 20);
    g
}

/// A -> B, A -> C, B -> D, C -> D (diamond)
fn diamond_graph() -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "&i32", 10);
    let c = g.add_variable("c", "&i32", 10);
    let d = g.add_variable("d", "i32", 20);
    g.add_borrow(b, a, false, 10);
    g.add_borrow(c, a, false, 10);
    g.add_move(b, d, 20);
    g.add_move(c, d, 20);
    g
}

/// 3 isolated variables with no edges
fn disconnected_graph() -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    g.add_variable("x", "i32", 0);
    g.add_variable("y", "String", 10);
    g.add_variable("z", "Vec<i32>", 20);
    g
}

/// Owner with two borrowers
fn simple_borrow_graph() -> OwnershipGraph {
    let mut g = OwnershipGraph::new();
    let owner = g.add_variable("data", "Vec<i32>", 0);
    let r1 = g.add_variable("r1", "&Vec<i32>", 10);
    let r2 = g.add_variable("r2", "&Vec<i32>", 20);
    g.add_borrow(r1, owner, false, 10);
    g.add_borrow(r2, owner, false, 20);
    g
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.1 DFS tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dfs_linear_outgoing() {
    let g = linear_graph();
    let b = g.find_by_name("b")[0];
    // b has outgoing borrow to a
    let visited = dfs(&g, b, Direction::Outgoing);
    assert!(visited.contains(&b));
    let a = g.find_by_name("a")[0];
    assert!(visited.contains(&a));
}

#[test]
fn test_dfs_linear_incoming() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    // a has incoming borrows from b
    let visited = dfs(&g, a, Direction::Incoming);
    let b = g.find_by_name("b")[0];
    assert!(visited.contains(&a));
    assert!(visited.contains(&b));
}

#[test]
fn test_dfs_visits_each_node_once() {
    let g = diamond_graph();
    let a = g.find_by_name("a")[0];
    let visited = dfs(&g, a, Direction::Both);
    let unique: std::collections::HashSet<_> = visited.iter().collect();
    assert_eq!(visited.len(), unique.len());
}

#[test]
fn test_dfs_disconnected_only_visits_start() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];
    let visited = dfs(&g, x, Direction::Outgoing);
    assert_eq!(visited.len(), 1);
    assert_eq!(visited[0], x);
}

#[test]
fn test_dfs_both_direction_reaches_all_connected() {
    let g = simple_borrow_graph();
    let r1 = g.find_by_name("r1")[0];
    let visited = dfs(&g, r1, Direction::Both);
    // r1 connects to data, data connects to r2
    assert_eq!(visited.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.2 BFS tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_bfs_distances() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];
    let b = g.find_by_name("b")[0];
    let a = g.find_by_name("a")[0];

    let result = bfs(&g, c, Direction::Outgoing);
    // c -> b -> a
    let dist_map: std::collections::HashMap<_, _> = result.into_iter().collect();
    assert_eq!(dist_map[&c], 0);
    assert_eq!(dist_map[&b], 1);
    assert_eq!(dist_map[&a], 2);
}

#[test]
fn test_bfs_monotonic_distances() {
    let g = diamond_graph();
    let a = g.find_by_name("a")[0];
    let result = bfs(&g, a, Direction::Both);

    let mut prev_dist = 0;
    for (_, dist) in &result {
        assert!(*dist >= prev_dist);
        prev_dist = *dist;
    }
}

#[test]
fn test_bfs_disconnected() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];
    let result = bfs(&g, x, Direction::Both);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (x, 0));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.3 Shortest path tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_shortest_path_self() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    let path = shortest_path(&g, a, a);
    assert_eq!(path, Some(vec![a]));
}

#[test]
fn test_shortest_path_direct() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    let r1 = g.find_by_name("r1")[0];
    let path = shortest_path(&g, r1, data).unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(path[0], r1);
    assert_eq!(path[1], data);
}

#[test]
fn test_shortest_path_two_hops() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];
    let a = g.find_by_name("a")[0];
    let path = shortest_path(&g, c, a).unwrap();
    assert_eq!(path.len(), 3); // c -> b -> a
}

#[test]
fn test_shortest_path_disconnected_returns_none() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];
    let y = g.find_by_name("y")[0];
    assert_eq!(shortest_path(&g, x, y), None);
}

#[test]
fn test_shortest_path_diamond_is_length_2() {
    let g = diamond_graph();
    let a = g.find_by_name("a")[0];
    let d = g.find_by_name("d")[0];
    let path = shortest_path(&g, a, d).unwrap();
    // a -> b -> d or a -> c -> d (both length 3 nodes = 2 hops)
    assert_eq!(path.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.4 Topological ordering tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_topo_order_simple_borrow() {
    let g = simple_borrow_graph();
    let order = topological_order(&g).unwrap();

    let data = g.find_by_name("data")[0];
    let r1 = g.find_by_name("r1")[0];
    let r2 = g.find_by_name("r2")[0];

    let pos: std::collections::HashMap<_, _> = order.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    // Owner (data) comes first in topo order (no dependencies)
    // Borrowers depend on owner, so they come after
    assert!(pos[&r1] > pos[&data]);
    assert!(pos[&r2] > pos[&data]);
}

#[test]
fn test_drop_order_borrower_before_owner() {
    let g = simple_borrow_graph();
    let order = drop_order(&g).unwrap();

    let data = g.find_by_name("data")[0];
    let r1 = g.find_by_name("r1")[0];
    let r2 = g.find_by_name("r2")[0];

    let pos: std::collections::HashMap<_, _> = order.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    // In drop order: borrowers dropped before owner
    assert!(pos[&r1] < pos[&data]);
    assert!(pos[&r2] < pos[&data]);
}

#[test]
fn test_topo_order_disconnected() {
    let g = disconnected_graph();
    let order = topological_order(&g).unwrap();
    assert_eq!(order.len(), 3);
}

#[test]
fn test_topo_order_chain() {
    let g = linear_graph();
    let order = topological_order(&g).unwrap();
    let a = g.find_by_name("a")[0];
    let b = g.find_by_name("b")[0];
    let c = g.find_by_name("c")[0];

    let pos: std::collections::HashMap<_, _> = order.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    // a has no borrow deps, b depends on a, c depends on b
    assert!(pos[&b] > pos[&a]);
    assert!(pos[&c] > pos[&b]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.5 Reachability tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_can_reach_self() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    assert!(can_reach(&g, a, a));
}

#[test]
fn test_can_reach_direct() {
    let g = simple_borrow_graph();
    let r1 = g.find_by_name("r1")[0];
    let data = g.find_by_name("data")[0];
    assert!(can_reach(&g, r1, data));
}

#[test]
fn test_can_reach_transitive() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];
    let a = g.find_by_name("a")[0];
    assert!(can_reach(&g, c, a));
}

#[test]
fn test_cannot_reach_reverse() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    let c = g.find_by_name("c")[0];
    // a has no outgoing edges, cannot reach c
    assert!(!can_reach(&g, a, c));
}

#[test]
fn test_cannot_reach_disconnected() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];
    let y = g.find_by_name("y")[0];
    assert!(!can_reach(&g, x, y));
}

#[test]
fn test_are_connected_via_borrow() {
    let g = simple_borrow_graph();
    let r1 = g.find_by_name("r1")[0];
    let r2 = g.find_by_name("r2")[0];
    // r1 and r2 both connect to data (undirected)
    assert!(are_connected(&g, r1, r2));
}

#[test]
fn test_are_not_connected() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];
    let y = g.find_by_name("y")[0];
    assert!(!are_connected(&g, x, y));
}

#[test]
fn test_all_reachable() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];
    let reachable = all_reachable(&g, c, Direction::Outgoing);
    assert_eq!(reachable.len(), 3); // c, b, a
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.6 Connected components tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_components_disconnected() {
    let g = disconnected_graph();
    let components = connected_components(&g);
    assert_eq!(components.len(), 3);
}

#[test]
fn test_components_fully_connected() {
    let g = simple_borrow_graph();
    let components = connected_components(&g);
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].len(), 3);
}

#[test]
fn test_component_count() {
    let g = disconnected_graph();
    assert_eq!(component_count(&g), 3);

    let g2 = simple_borrow_graph();
    assert_eq!(component_count(&g2), 1);
}

#[test]
fn test_component_of() {
    let g = simple_borrow_graph();
    let r1 = g.find_by_name("r1")[0];
    let component = component_of(&g, r1);
    assert_eq!(component.len(), 3);
}

#[test]
fn test_components_mixed() {
    let mut g = OwnershipGraph::new();
    // Component 1: a -> b
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "&i32", 10);
    g.add_borrow(b, a, false, 10);
    // Component 2: isolated
    g.add_variable("c", "String", 20);

    let components = connected_components(&g);
    assert_eq!(components.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.7 Borrow chain and depth tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_borrow_chain_owner() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    let chain = borrow_chain(&g, data);
    assert_eq!(chain, vec![data]); // owner has no borrow chain
}

#[test]
fn test_borrow_chain_direct() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    let r1 = g.find_by_name("r1")[0];
    let chain = borrow_chain(&g, r1);
    assert_eq!(chain, vec![r1, data]);
}

#[test]
fn test_borrow_chain_nested() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    let b = g.find_by_name("b")[0];
    let c = g.find_by_name("c")[0];
    let chain = borrow_chain(&g, c);
    assert_eq!(chain, vec![c, b, a]);
}

#[test]
fn test_borrow_depth_owner() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    assert_eq!(borrow_depth(&g, data), 0);
}

#[test]
fn test_borrow_depth_direct() {
    let g = simple_borrow_graph();
    let r1 = g.find_by_name("r1")[0];
    assert_eq!(borrow_depth(&g, r1), 1);
}

#[test]
fn test_borrow_depth_nested() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];
    assert_eq!(borrow_depth(&g, c), 2);
}

#[test]
fn test_root_owner() {
    let g = linear_graph();
    let a = g.find_by_name("a")[0];
    let c = g.find_by_name("c")[0];
    assert_eq!(root_owner(&g, c), a);
    assert_eq!(root_owner(&g, a), a); // owner is its own root
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_graph_traversals() {
    let g = OwnershipGraph::new();
    assert_eq!(connected_components(&g).len(), 0);
    assert_eq!(component_count(&g), 0);
    assert_eq!(topological_order(&g).unwrap().len(), 0);
}

#[test]
fn test_topo_order_cycle_returns_error() {
    // Create a cycle via Rc clones: a -> b -> a
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "Rc<i32>", 0);
    let b = g.add_variable("b", "Rc<i32>", 10);
    // Simulate mutual borrows (cycle)
    g.add_borrow(a, b, false, 10);
    g.add_borrow(b, a, false, 20);

    let result = topological_order(&g);
    assert!(result.is_err());
    let cycle = result.unwrap_err().cycle;
    assert!(cycle.contains(&a));
    assert!(cycle.contains(&b));
}

#[test]
fn test_dfs_tree_topology() {
    //     root
    //    /    \
    //   l      r
    //  / \
    // ll  lr
    let mut g = OwnershipGraph::new();
    let root = g.add_variable("root", "i32", 0);
    let l = g.add_variable("l", "&i32", 10);
    let r = g.add_variable("r", "&i32", 10);
    let ll = g.add_variable("ll", "&&i32", 20);
    let lr = g.add_variable("lr", "&&i32", 20);
    g.add_borrow(l, root, false, 10);
    g.add_borrow(r, root, false, 10);
    g.add_borrow(ll, l, false, 20);
    g.add_borrow(lr, l, false, 20);

    // DFS from ll should reach root
    let visited = dfs(&g, ll, Direction::Outgoing);
    assert!(visited.contains(&ll));
    assert!(visited.contains(&l));
    assert!(visited.contains(&root));
    // ll cannot reach r directly via outgoing
    // (r borrows root, but ll -> l -> root, not root -> r)

    // BFS from root incoming should find all borrowers
    let result = bfs(&g, root, Direction::Incoming);
    assert_eq!(result.len(), 5); // root + l + r + ll + lr
}

#[test]
fn test_dfs_find_with_predicate() {
    let g = linear_graph();
    let c = g.find_by_name("c")[0];

    // Find a node whose name is "a"
    let path = dfs_find(&g, c, Direction::Outgoing, |n| {
        g.get_node(n).map_or(false, |node| node.name() == "a")
    });
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.last().copied(), Some(g.find_by_name("a")[0]));
}

#[test]
fn test_dfs_find_not_found() {
    let g = disconnected_graph();
    let x = g.find_by_name("x")[0];

    let path = dfs_find(&g, x, Direction::Outgoing, |n| {
        g.get_node(n).map_or(false, |node| node.name() == "y")
    });
    assert!(path.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// shortest_path_with_edges tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_shortest_path_with_edges_direct() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    let r1 = g.find_by_name("r1")[0];

    let path = shortest_path_with_edges(&g, r1, data).unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(path[0].0, r1);
    assert!(path[0].1.is_some()); // edge connecting r1 to data
    assert_eq!(path[1].0, data);
    assert!(path[1].1.is_none()); // last node has no next edge
}

#[test]
fn test_shortest_path_with_edges_self() {
    let g = simple_borrow_graph();
    let data = g.find_by_name("data")[0];
    let path = shortest_path_with_edges(&g, data, data).unwrap();
    assert_eq!(path.len(), 1);
    assert_eq!(path[0], (data, None));
}

// ═══════════════════════════════════════════════════════════════════════════
// Performance test
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_performance_large_graph() {
    let mut g = OwnershipGraph::new();
    let mut nodes = Vec::new();

    // Create 10,000 nodes
    for i in 0..10_000 {
        let id = g.add_variable(&format!("v{}", i), "i32", i as u64);
        nodes.push(id);
    }

    // Create a chain of borrows (each borrows from previous)
    for i in 1..10_000 {
        g.add_borrow(nodes[i], nodes[i - 1], false, i as u64);
    }

    let start = std::time::Instant::now();

    // DFS from last node should reach all
    let visited = dfs(&g, nodes[9999], Direction::Outgoing);
    assert_eq!(visited.len(), 10_000);

    // BFS from first node (incoming direction)
    let bfs_result = bfs(&g, nodes[0], Direction::Incoming);
    assert_eq!(bfs_result.len(), 10_000);

    // Connected components
    let components = connected_components(&g);
    assert_eq!(components.len(), 1);

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Traversals took too long: {:?}", elapsed);
}
