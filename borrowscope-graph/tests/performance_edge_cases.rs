use borrowscope_graph::{
    BatchGraph, CachedOwnershipGraph, ConcurrentGraph, GraphMetrics, LazyGraph, OwnershipGraph,
    Variable,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ============================================================================
// Caching Edge Cases
// ============================================================================

#[test]
fn test_cache_with_empty_graph() {
    let graph = OwnershipGraph::new();
    let cached = CachedOwnershipGraph::new(graph);

    let borrowers = cached.borrowers_of(999);
    assert!(borrowers.is_empty());

    let stats = cached.stats();
    assert_eq!(stats.node_count, 0);
}

#[test]
fn test_cache_with_nonexistent_id() {
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

    let cached = CachedOwnershipGraph::new(graph);
    let borrowers = cached.borrowers_of(999);
    assert!(borrowers.is_empty());
}

#[test]
fn test_cache_multiple_invalidations() {
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

    for i in 2..=100 {
        let _ = cached.stats();
        cached.graph_mut().add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let final_stats = cached.stats();
    assert_eq!(final_stats.node_count, 100);
}

#[test]
fn test_cache_with_large_borrower_list() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    for i in 2..=10001 {
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
    let borrowers = cached.borrowers_of(1);
    assert_eq!(borrowers.len(), 10000);
}

#[test]
fn test_cache_stats_with_max_depth() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: i,
        });
    }

    let cached = CachedOwnershipGraph::new(graph);
    let stats = cached.stats();
    assert_eq!(stats.max_depth, 100);
}

// ============================================================================
// Concurrent Access Edge Cases
// ============================================================================

#[test]
fn test_concurrent_empty_graph() {
    let graph = ConcurrentGraph::new();
    let count = graph.node_count();
    assert_eq!(count, 0);
}

#[test]
fn test_concurrent_many_writers() {
    let graph = ConcurrentGraph::new();

    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let g = graph.clone_handle();
            thread::spawn(move || {
                for i in 0..100 {
                    let id = thread_id * 100 + i + 1;
                    g.write(|graph| {
                        graph.add_variable(Variable {
                            id,
                            name: format!("v{}", id),
                            type_name: "i32".into(),
                            created_at: id as u64,
                            dropped_at: None,
                            scope_depth: 0,
                        });
                    });
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(graph.node_count(), 1000);
}

#[test]
fn test_concurrent_read_during_write() {
    let graph = ConcurrentGraph::new();

    for i in 1..=100 {
        graph.write(|g| {
            g.add_variable(Variable {
                id: i,
                name: format!("v{}", i),
                type_name: "i32".into(),
                created_at: i as u64,
                dropped_at: None,
                scope_depth: 0,
            });
        });
    }

    let writer = {
        let g = graph.clone_handle();
        thread::spawn(move || {
            for i in 101..=200 {
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
                thread::sleep(Duration::from_micros(10));
            }
        })
    };

    let readers: Vec<_> = (0..5)
        .map(|_| {
            let g = graph.clone_handle();
            thread::spawn(move || {
                let mut counts = Vec::new();
                for _ in 0..50 {
                    counts.push(g.node_count());
                    thread::sleep(Duration::from_micros(10));
                }
                counts
            })
        })
        .collect();

    writer.join().unwrap();
    for reader in readers {
        let counts = reader.join().unwrap();
        assert!(counts.iter().all(|&c| (100..=200).contains(&c)));
    }
}

#[test]
fn test_concurrent_clone_handle_independence() {
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

    let handle1 = graph.clone_handle();
    let handle2 = graph.clone_handle();

    let count1 = handle1.node_count();
    let count2 = handle2.node_count();

    assert_eq!(count1, count2);
    assert_eq!(count1, 1);
}

// ============================================================================
// Batch Operations Edge Cases
// ============================================================================

#[test]
fn test_batch_empty_flush() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    let flushed = batch.flush();
    assert_eq!(flushed, 0);
}

#[test]
fn test_batch_multiple_flushes() {
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

    batch.flush();

    for i in 51..=100 {
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

    assert_eq!(batch.graph().node_count(), 100);
}

#[test]
fn test_batch_with_all_relationship_types() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    for i in 1..=7 {
        batch.queue_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    batch.queue_borrow(2, 1, false, 200);
    batch.queue_borrow(3, 1, true, 300);
    batch.queue_move(4, 1, 400);

    batch.flush();

    assert_eq!(batch.graph().node_count(), 7);
    assert_eq!(batch.graph().edge_count(), 3);
}

#[test]
fn test_batch_large_scale() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    for i in 1..=50000 {
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
    assert_eq!(batch.graph().node_count(), 50000);
}

#[test]
fn test_batch_pending_count_accuracy() {
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
        assert_eq!(batch.pending_count(), i);
    }

    batch.flush();
    assert_eq!(batch.pending_count(), 0);
}

// ============================================================================
// Lazy Serialization Edge Cases
// ============================================================================

#[test]
fn test_lazy_empty_graph() {
    let graph = OwnershipGraph::new();
    let lazy = LazyGraph::new(graph);

    let json = lazy.to_json().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn test_lazy_multiple_invalidations() {
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

    for i in 2..=100 {
        let _ = lazy.to_json().unwrap();
        lazy.graph_mut().add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let final_json = lazy.to_json().unwrap();
    // Just verify it contains some data
    assert!(!final_json.is_empty());
}

#[test]
fn test_lazy_large_graph_serialization() {
    let mut graph = OwnershipGraph::new();
    for i in 1..=10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let lazy = LazyGraph::new(graph);
    let json = lazy.to_json().unwrap();

    assert!(json.len() > 100000);
}

#[test]
fn test_lazy_cache_consistency() {
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
    let json3 = lazy.to_json().unwrap();

    assert_eq!(json1, json2);
    assert_eq!(json2, json3);
}

// ============================================================================
// Metrics Edge Cases
// ============================================================================

#[test]
fn test_metrics_zero_division() {
    let metrics = GraphMetrics::new();
    let hit_rate = metrics.get_cache_hit_rate();
    assert_eq!(hit_rate, 0.0);
}

#[test]
fn test_metrics_all_hits() {
    let metrics = GraphMetrics::new();

    for _ in 0..1000 {
        metrics.record_cache_hit();
    }

    let hit_rate = metrics.get_cache_hit_rate();
    assert_eq!(hit_rate, 1.0);
}

#[test]
fn test_metrics_all_misses() {
    let metrics = GraphMetrics::new();

    for _ in 0..1000 {
        metrics.record_cache_miss();
    }

    let hit_rate = metrics.get_cache_hit_rate();
    assert_eq!(hit_rate, 0.0);
}

#[test]
fn test_metrics_high_contention() {
    let metrics = Arc::new(GraphMetrics::new());

    let handles: Vec<_> = (0..100)
        .map(|_| {
            let m = Arc::clone(&metrics);
            thread::spawn(move || {
                for _ in 0..1000 {
                    m.increment_nodes();
                    m.increment_edges();
                    m.increment_queries();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(metrics.get_node_count(), 100000);
    assert_eq!(metrics.get_edge_count(), 100000);
    assert_eq!(metrics.get_query_count(), 100000);
}

#[test]
fn test_metrics_reset_under_load() {
    let metrics = Arc::new(GraphMetrics::new());

    let incrementer = {
        let m = Arc::clone(&metrics);
        thread::spawn(move || {
            for _ in 0..1000 {
                m.increment_nodes();
                thread::sleep(Duration::from_micros(10));
            }
        })
    };

    thread::sleep(Duration::from_millis(5));
    metrics.reset();

    incrementer.join().unwrap();

    let count = metrics.get_node_count();
    assert!(count < 1000);
}

// ============================================================================
// Memory Statistics Edge Cases
// ============================================================================

#[test]
fn test_memory_stats_unicode_strings() {
    let mut graph = OwnershipGraph::new();

    let names = ["变量", "переменная", "μεταβλητή", "🦀"];
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

    let stats = graph.memory_usage();
    assert!(stats.string_memory > 0);
}

#[test]
fn test_memory_stats_large_strings() {
    let mut graph = OwnershipGraph::new();

    let long_name = "a".repeat(10000);
    graph.add_variable(Variable {
        id: 1,
        name: long_name.clone(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let stats = graph.memory_usage();
    assert!(stats.string_memory >= 10000);
}

#[test]
fn test_memory_stats_empty_strings() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "".into(),
        type_name: "".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let stats = graph.memory_usage();
    assert!(stats.nodes_memory > 0);
    assert_eq!(stats.string_memory, 0);
}

// ============================================================================
// Combined Optimization Tests
// ============================================================================

#[test]
fn test_cached_concurrent_access() {
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

    let cached = Arc::new(CachedOwnershipGraph::new(graph));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let c = Arc::clone(&cached);
            thread::spawn(move || {
                for i in 1..=100 {
                    let _ = c.borrowers_of(i);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_batch_with_lazy_serialization() {
    let graph = OwnershipGraph::new();
    let mut batch = BatchGraph::new(graph);

    for i in 1..=1000 {
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
    let lazy = LazyGraph::new(final_graph);

    let json = lazy.to_json().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn test_metrics_with_all_optimizations() {
    let metrics = Arc::new(GraphMetrics::new());

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
        metrics.increment_nodes();
    }

    batch.flush();

    let cached = CachedOwnershipGraph::new(batch.into_graph());

    for i in 1..=100 {
        let _ = cached.borrowers_of(i);
        metrics.increment_queries();
        metrics.record_cache_hit();
    }

    assert_eq!(metrics.get_node_count(), 100);
    assert_eq!(metrics.get_query_count(), 100);
    assert_eq!(metrics.get_cache_hit_rate(), 1.0);
}
