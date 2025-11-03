use borrowscope_graph::{
    BatchGraph, CachedOwnershipGraph, ConcurrentGraph, GraphMetrics, LazyGraph, OwnershipGraph,
    Variable,
};
use std::thread;
use std::time::Instant;

// ============================================================================
// Caching Tests
// ============================================================================

#[test]
fn test_cached_graph_basic() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let cached = CachedOwnershipGraph::new(graph);
    let borrowers = cached.borrowers_of(1);
    assert!(borrowers.is_empty());
}

#[test]
fn test_cached_graph_with_borrowers() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
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

    let cached = CachedOwnershipGraph::new(graph);

    let borrowers1 = cached.borrowers_of(1);
    assert_eq!(borrowers1.len(), 9);

    let borrowers2 = cached.borrowers_of(1);
    assert_eq!(borrowers2.len(), 9);
}

#[test]
fn test_cached_graph_stats() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: i % 10,
        });
    }

    let cached = CachedOwnershipGraph::new(graph);
    let stats = cached.stats();

    assert_eq!(stats.node_count, 100);
    assert_eq!(stats.edge_count, 0);
    assert_eq!(stats.max_depth, 9);
}

#[test]
fn test_cached_graph_invalidation() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let mut cached = CachedOwnershipGraph::new(graph);

    let _ = cached.borrowers_of(1);
    let _ = cached.stats();

    cached.graph_mut().add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let stats = cached.stats();
    assert_eq!(stats.node_count, 2);
}

#[test]
fn test_cache_performance_improvement() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=100 {
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

    let cached = CachedOwnershipGraph::new(graph);

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = cached.borrowers_of(1);
    }
    let cached_time = start.elapsed();

    assert!(cached_time.as_millis() < 100);
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_concurrent_graph_basic() {
    let graph = ConcurrentGraph::new();

    graph.write(|g| {
        g.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 100,
            dropped_at: None,
            scope_depth: 0,
        });
    });

    let count = graph.read(|g| g.node_count());
    assert_eq!(count, 1);
}

#[test]
fn test_concurrent_graph_multiple_readers() {
    let graph = ConcurrentGraph::new();

    graph.write(|g| {
        for i in 1..=100 {
            g.add_variable(Variable {
                id: i,
                name: format!("v{}", i),
                type_name: "i32".into(),
                created_at: i as u64,
                dropped_at: None,
                scope_depth: 0,
            });
        }
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let g = graph.clone_handle();
            thread::spawn(move || g.read(|graph| graph.node_count()))
        })
        .collect();

    for handle in handles {
        let count = handle.join().unwrap();
        assert_eq!(count, 100);
    }
}

#[test]
fn test_concurrent_graph_read_write() {
    let graph = ConcurrentGraph::new();

    let writer = {
        let g = graph.clone_handle();
        thread::spawn(move || {
            for i in 1..=100 {
                g.write(|graph| {
                    graph.add_variable(Variable {
                        id: i,
                        name: format!("v{}", i),
                        type_name: "i32".into(),
                        created_at: i as u64,
                        dropped_at: None,
                        scope_depth: 0,
                    });
                });
            }
        })
    };

    let reader = {
        let g = graph.clone_handle();
        thread::spawn(move || {
            let mut counts = Vec::new();
            for _ in 0..10 {
                counts.push(g.read(|graph| graph.node_count()));
                thread::sleep(std::time::Duration::from_millis(1));
            }
            counts
        })
    };

    writer.join().unwrap();
    let counts = reader.join().unwrap();

    assert!(counts.iter().all(|&c| c <= 100));
}

// ============================================================================
// Batch Operations Tests
// ============================================================================

#[test]
fn test_batch_graph_basic() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    batch.queue_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    assert_eq!(batch.pending_count(), 1);

    let flushed = batch.flush();
    assert_eq!(flushed, 1);
    assert_eq!(batch.pending_count(), 0);
    assert_eq!(batch.graph().node_count(), 1);
}

#[test]
fn test_batch_graph_multiple_operations() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    for i in 1..=100 {
        batch.queue_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        batch.queue_borrow(i, i - 1, false, i as u64);
    }

    assert_eq!(batch.pending_count(), 199);

    batch.flush();
    assert_eq!(batch.graph().node_count(), 100);
    assert_eq!(batch.graph().edge_count(), 99);
}

#[test]
fn test_batch_graph_into_graph() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    for i in 1..=50 {
        batch.queue_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let final_graph = batch.into_graph();
    assert_eq!(final_graph.node_count(), 50);
}

#[test]
fn test_batch_graph_performance() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    let start = Instant::now();
    for i in 1..=10000 {
        batch.queue_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }
    batch.flush();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 1000);
    assert_eq!(batch.graph().node_count(), 10000);
}

// ============================================================================
// Memory Statistics Tests
// ============================================================================

#[test]
fn test_memory_stats_empty() {
    let graph = OwnershipGraph::new();
    let stats = graph.memory_usage();

    assert_eq!(stats.nodes_memory, 0);
    assert_eq!(stats.edges_memory, 0);
    assert_eq!(stats.string_memory, 0);
    assert_eq!(stats.total, 0);
}

#[test]
fn test_memory_stats_with_nodes() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let stats = graph.memory_usage();

    assert!(stats.nodes_memory > 0);
    assert!(stats.string_memory > 0);
    assert_eq!(
        stats.total,
        stats.nodes_memory + stats.edges_memory + stats.string_memory
    );
}

#[test]
fn test_memory_stats_with_edges() {
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
        graph.add_borrow(i, i - 1, false, i as u64);
    }

    let stats = graph.memory_usage();

    assert!(stats.edges_memory > 0);
    assert!(stats.total > stats.nodes_memory);
}

// ============================================================================
// Lazy Serialization Tests
// ============================================================================

#[test]
fn test_lazy_graph_basic() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let lazy = LazyGraph::new(graph);
    let json1 = lazy.to_json().unwrap();
    let json2 = lazy.to_json().unwrap();

    assert_eq!(json1, json2);
}

#[test]
fn test_lazy_graph_cache_invalidation() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let mut lazy = LazyGraph::new(graph);
    let json1 = lazy.to_json().unwrap();

    lazy.graph_mut().add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let json2 = lazy.to_json().unwrap();
    assert_ne!(json1, json2);
}

#[test]
fn test_lazy_graph_performance() {
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

    let lazy = LazyGraph::new(graph);

    let start = Instant::now();
    let _ = lazy.to_json().unwrap();
    let first_call = start.elapsed();

    let start = Instant::now();
    for _ in 0..100 {
        let _ = lazy.to_json().unwrap();
    }
    let cached_calls = start.elapsed();

    assert!(cached_calls < first_call * 10);
}

// ============================================================================
// Metrics Tests
// ============================================================================

#[test]
fn test_metrics_basic() {
    let metrics = GraphMetrics::new();

    assert_eq!(metrics.get_node_count(), 0);
    assert_eq!(metrics.get_edge_count(), 0);
    assert_eq!(metrics.get_query_count(), 0);
}

#[test]
fn test_metrics_increment() {
    let metrics = GraphMetrics::new();

    for _ in 0..100 {
        metrics.increment_nodes();
    }

    for _ in 0..50 {
        metrics.increment_edges();
    }

    for _ in 0..200 {
        metrics.increment_queries();
    }

    assert_eq!(metrics.get_node_count(), 100);
    assert_eq!(metrics.get_edge_count(), 50);
    assert_eq!(metrics.get_query_count(), 200);
}

#[test]
fn test_metrics_cache_hit_rate() {
    let metrics = GraphMetrics::new();

    for _ in 0..80 {
        metrics.record_cache_hit();
    }

    for _ in 0..20 {
        metrics.record_cache_miss();
    }

    let hit_rate = metrics.get_cache_hit_rate();
    assert!((hit_rate - 0.8).abs() < 0.01);
}

#[test]
fn test_metrics_reset() {
    let metrics = GraphMetrics::new();

    metrics.increment_nodes();
    metrics.increment_edges();
    metrics.increment_queries();

    metrics.reset();

    assert_eq!(metrics.get_node_count(), 0);
    assert_eq!(metrics.get_edge_count(), 0);
    assert_eq!(metrics.get_query_count(), 0);
}

#[test]
fn test_metrics_concurrent() {
    let metrics = std::sync::Arc::new(GraphMetrics::new());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let m = std::sync::Arc::clone(&metrics);
            thread::spawn(move || {
                for _ in 0..100 {
                    m.increment_nodes();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(metrics.get_node_count(), 1000);
}
