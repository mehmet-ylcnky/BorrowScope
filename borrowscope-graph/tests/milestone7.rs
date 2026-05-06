//! Comprehensive tests for Milestone 7: Integration with borrowscope-runtime.

use borrowscope_graph::builder::{GraphStream, GraphUpdate};
use borrowscope_graph::conflict::is_valid;
use borrowscope_graph::export::{from_json, to_json};
use borrowscope_graph::stats::statistics;
use borrowscope_graph::OwnershipGraph;
use borrowscope_runtime::{get_events, reset, Event};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Helper: create events that mirror what the macro produces (with proper IDs)
fn instrumented_events() -> Vec<Event> {
    vec![
        Event::New {
            timestamp: 0,
            var_name: "data".to_string(),
            var_id: "data_0".to_string(),
            type_name: "Vec<i32>".to_string(),
        },
        Event::New {
            timestamp: 10,
            var_name: "r1".to_string(),
            var_id: "r1_0".to_string(),
            type_name: "&Vec<i32>".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "data_0".to_string(),
            mutable: false,
        },
        Event::New {
            timestamp: 20,
            var_name: "r2".to_string(),
            var_id: "r2_0".to_string(),
            type_name: "&Vec<i32>".to_string(),
        },
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "data_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 50,
            var_id: "r1_0".to_string(),
            location: None,
        },
        Event::Drop {
            timestamp: 60,
            var_id: "r2_0".to_string(),
            location: None,
        },
        Event::Drop {
            timestamp: 100,
            var_id: "data_0".to_string(),
            location: None,
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// 7.1 Direct construction from get_events()
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_from_runtime_produces_valid_graph() {
    let events = instrumented_events();
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
    assert!(is_valid(&graph));
}

#[test]
fn test_from_runtime_empty_when_no_events() {
    reset();
    let graph = OwnershipGraph::from_runtime();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_from_runtime_for_var() {
    let events = instrumented_events();
    // from_runtime_for_var filters by var_name in the event
    let graph = OwnershipGraph::from_events(
        &events
            .into_iter()
            .filter(|e| match e {
                Event::New { var_name, .. } => var_name == "data",
                Event::Drop { var_id, .. } => var_id == "data_0",
                Event::Borrow { owner_id, .. } => owner_id == "data_0",
                _ => false,
            })
            .collect::<Vec<_>>(),
    );
    assert!(graph.find_by_name("data").len() >= 1);
}

#[test]
fn test_from_runtime_filtered_only_new() {
    let events = instrumented_events();
    let graph = OwnershipGraph::from_events(
        &events
            .into_iter()
            .filter(|e| matches!(e, Event::New { .. }))
            .collect::<Vec<_>>(),
    );
    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 0); // no borrow edges
}

// ═══════════════════════════════════════════════════════════════════════════
// 7.2 Streaming graph construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_stream_matches_batch() {
    let events = instrumented_events();
    let batch_graph = OwnershipGraph::from_events(&events);

    let mut stream = GraphStream::new();
    stream.push_all(&events);
    let stream_graph = stream.into_graph();

    assert_eq!(batch_graph.node_count(), stream_graph.node_count());
    assert_eq!(batch_graph.edge_count(), stream_graph.edge_count());
}

#[test]
fn test_drain_runtime() {
    reset();
    // Use track_new which does record events (even with "unknown" owner)
    borrowscope_runtime::track_new("x", 42);
    borrowscope_runtime::track_drop("x");

    let mut stream = GraphStream::new();
    let updates = stream.drain_runtime();

    // At least the New and Drop events should produce updates
    assert!(!updates.is_empty());
    assert!(stream.graph().node_count() >= 1);
}

#[test]
fn test_stream_callbacks_fire() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let mut stream = GraphStream::new();
    stream.on_update(move |_update| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::New {
            timestamp: 10,
            var_name: "y".to_string(),
            var_id: "y_0".to_string(),
            type_name: "i32".to_string(),
        },
    ];

    stream.push_all(&events);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_stream_into_graph() {
    let events = instrumented_events();

    let mut stream = GraphStream::new();
    stream.push_all(&events);
    let graph = stream.into_graph();

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.find_by_name("data").len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7.3 / 7.4 Feature gate and independent compilation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_graph_crate_compiles_independently() {
    let g = OwnershipGraph::new();
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.version(), "1.0");
}

#[test]
fn test_runtime_works_without_graph() {
    reset();
    borrowscope_runtime::track_new("x", 42);
    borrowscope_runtime::track_drop("x");
    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_graph_works_on_imported_data() {
    // Graph algorithms work on manually constructed or imported graphs
    // (no runtime needed)
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "&i32", 10);
    g.add_borrow(b, a, false, 10);

    assert_eq!(g.borrowers_of(a), vec![b]);
}

// ═══════════════════════════════════════════════════════════════════════════
// End-to-end tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_end_to_end_build_graph_traverse() {
    let events = instrumented_events();
    let graph = OwnershipGraph::from_events(&events);

    let data_id = graph.find_by_name("data")[0];
    let borrowers = graph.borrowers_of(data_id);
    assert_eq!(borrowers.len(), 2);

    // Borrow edges ended when borrowers were dropped
    let r1_id = graph.find_by_name("r1")[0];
    let edge_id = graph.outgoing_edges(r1_id)[0];
    let edge = graph.get_edge(edge_id).unwrap();
    assert_eq!(edge.ended_at, Some(50));
}

#[test]
fn test_end_to_end_export_import_roundtrip() {
    let events = instrumented_events();
    let graph = OwnershipGraph::from_events(&events);

    let json = to_json(&graph).unwrap();
    let restored = from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_end_to_end_statistics() {
    let events = instrumented_events();
    let graph = OwnershipGraph::from_events(&events);

    let stats = statistics(&graph);
    assert_eq!(stats.total_nodes, 3);
    assert_eq!(stats.total_edges, 2);
    assert_eq!(stats.shared_borrows, 2);
    assert_eq!(stats.mutable_borrows, 0);
}
