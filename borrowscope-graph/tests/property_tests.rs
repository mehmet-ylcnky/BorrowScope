//! Property-based tests for graph invariants.

use borrowscope_graph::conflict::is_valid;
use borrowscope_graph::traversal::*;
use borrowscope_graph::*;
use proptest::prelude::*;
use std::collections::HashSet;

// Strategy: generate a random graph with N nodes and M edges
fn arb_graph(max_nodes: usize, max_edges: usize) -> impl Strategy<Value = OwnershipGraph> {
    (1..=max_nodes, 0..=max_edges)
        .prop_flat_map(|(n, e)| {
            let nodes = prop::collection::vec(("[a-z]{1,5}", 0..1000u64), n);
            let edges =
                prop::collection::vec((0..n, 0..n, prop::bool::ANY, 0..1000u64), e.min(n * n));
            (nodes, edges)
        })
        .prop_map(|(nodes, edges)| {
            let mut g = OwnershipGraph::new();
            let mut ids = Vec::new();
            for (name, ts) in &nodes {
                ids.push(g.add_variable(name, "i32", *ts));
            }
            for &(src, tgt, mutable, ts) in &edges {
                if src != tgt && src < ids.len() && tgt < ids.len() {
                    g.add_borrow(ids[src], ids[tgt], mutable, ts);
                }
            }
            g
        })
}

proptest! {
    /// Every edge references valid node IDs.
    #[test]
    fn prop_edges_reference_valid_nodes(graph in arb_graph(20, 30)) {
        let node_ids: HashSet<NodeId> = graph.nodes().iter().map(|n| n.id()).collect();
        for edge in graph.edges() {
            prop_assert!(node_ids.contains(&edge.source));
            prop_assert!(node_ids.contains(&edge.target));
        }
    }

    /// DFS visits each reachable node exactly once.
    #[test]
    fn prop_dfs_visits_once(graph in arb_graph(20, 30)) {
        if graph.node_count() == 0 {
            return Ok(());
        }
        let start = graph.nodes()[0].id();
        let visited = dfs(&graph, start, Direction::Both);
        let unique: HashSet<_> = visited.iter().collect();
        prop_assert_eq!(visited.len(), unique.len());
    }

    /// Node count never decreases after adding nodes.
    #[test]
    fn prop_node_count_monotonic(names in prop::collection::vec("[a-z]{1,3}", 1..20)) {
        let mut g = OwnershipGraph::new();
        let mut prev_count = 0;
        for name in &names {
            g.add_variable(name, "i32", 0);
            prop_assert!(g.node_count() > prev_count);
            prev_count = g.node_count();
        }
    }

    /// JSON round-trip preserves node and edge counts.
    #[test]
    fn prop_json_roundtrip(graph in arb_graph(10, 15)) {
        let json = borrowscope_graph::export::to_json(&graph).unwrap();
        let restored = borrowscope_graph::export::from_json(&json).unwrap();
        prop_assert_eq!(graph.node_count(), restored.node_count());
        prop_assert_eq!(graph.edge_count(), restored.edge_count());
    }

    /// Connected components partition all nodes (no node left out).
    #[test]
    fn prop_components_cover_all_nodes(graph in arb_graph(15, 20)) {
        let components = connected_components(&graph);
        let total: usize = components.iter().map(|c| c.len()).sum();
        prop_assert_eq!(total, graph.node_count());
    }

    /// BFS distances are monotonically non-decreasing.
    #[test]
    fn prop_bfs_distances_monotonic(graph in arb_graph(15, 20)) {
        if graph.node_count() == 0 {
            return Ok(());
        }
        let start = graph.nodes()[0].id();
        let result = bfs(&graph, start, Direction::Both);
        let mut prev_dist = 0;
        for (_, dist) in &result {
            prop_assert!(*dist >= prev_dist);
            prev_dist = *dist;
        }
    }
}
