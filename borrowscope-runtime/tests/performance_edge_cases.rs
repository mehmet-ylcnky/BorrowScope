use borrowscope_runtime::*;
use serial_test::serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

// ============================================================================
// Zero-Size Type Performance
// ============================================================================

#[test]
#[serial]
fn test_zst_tracking_overhead() {
    const ITERATIONS: usize = 10_000;

    reset();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        track_new(&format!("zst_{}", i), ());
    }
    let duration = start.elapsed();

    // ZST tracking should still be fast
    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(avg_ns < 1000, "ZST tracking too slow: {}ns", avg_ns);

    let events = get_events();
    assert_eq!(events.len(), ITERATIONS);
}

#[test]
#[serial]
fn test_large_type_tracking() {
    #[allow(dead_code)]
    #[derive(Clone)]
    struct LargeType([u8; 1024]);

    const ITERATIONS: usize = 1000;

    reset();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let large = LargeType([i as u8; 1024]);
        let _x = track_new(&format!("large_{}", i), large);
    }
    let duration = start.elapsed();

    // Large type tracking overhead should be similar (tracking is by reference)
    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(avg_ns < 2000, "Large type tracking too slow: {}ns", avg_ns);
}

// ============================================================================
// Extreme Concurrency Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_high_thread_contention() {
    const NUM_THREADS: usize = 32;
    const OPS_PER_THREAD: usize = 100;

    reset();

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait(); // Synchronize start
                for i in 0..OPS_PER_THREAD {
                    track_new(&format!("t{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();

    let events = get_events();
    assert!(
        events.len() >= NUM_THREADS * OPS_PER_THREAD * 9 / 10,
        "Lost events under high contention: {} < {}",
        events.len(),
        NUM_THREADS * OPS_PER_THREAD
    );

    // Should complete in reasonable time even with high contention
    assert!(
        duration.as_secs() < 5,
        "High contention took too long: {:?}",
        duration
    );
}

#[test]
#[serial]
fn test_rapid_thread_spawn_despawn() {
    const NUM_ITERATIONS: usize = 100;

    reset();

    let start = Instant::now();
    for i in 0..NUM_ITERATIONS {
        let handle = thread::spawn(move || {
            track_new(&format!("rapid_{}", i), i);
        });
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    let events = get_events();
    assert!(events.len() >= NUM_ITERATIONS * 9 / 10);
    assert!(duration.as_secs() < 2);
}

#[test]
#[serial]
fn test_concurrent_reset_safety() {
    const NUM_THREADS: usize = 8;

    reset();

    let barrier = Arc::new(Barrier::new(NUM_THREADS + 1));
    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                for i in 0..100 {
                    track_new(&format!("t{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();

    barrier.wait();
    thread::sleep(std::time::Duration::from_millis(10));
    reset(); // Reset while threads are tracking

    for handle in handles {
        handle.join().unwrap();
    }

    // Should not crash or deadlock
    let events = get_events();
    assert!(events.len() < NUM_THREADS * 100); // Some events lost due to reset
}

// ============================================================================
// Memory Pressure Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_memory_growth_pattern() {
    const BATCH_SIZE: usize = 1000;
    const NUM_BATCHES: usize = 10;

    reset();

    let mut memory_sizes = Vec::new();

    for batch in 0..NUM_BATCHES {
        for i in 0..BATCH_SIZE {
            track_new(&format!("batch_{}_{}", batch, i), i);
        }

        let events = get_events();
        let memory = events.len() * std::mem::size_of::<Event>();
        memory_sizes.push(memory);
    }

    // Check overall linear growth trend
    let first_size = memory_sizes[0] as f64;
    let last_size = memory_sizes[memory_sizes.len() - 1] as f64;
    let expected_ratio = NUM_BATCHES as f64;
    let actual_ratio = last_size / first_size;

    // Allow 20% variance from perfect linear growth
    assert!(
        actual_ratio > expected_ratio * 0.8 && actual_ratio < expected_ratio * 1.2,
        "Memory growth not linear: expected ~{:.1}x, got {:.1}x",
        expected_ratio,
        actual_ratio
    );
}

#[test]
#[serial]
fn test_repeated_reset_memory() {
    const ITERATIONS: usize = 100;
    const OPS_PER_ITERATION: usize = 1000;

    for _ in 0..ITERATIONS {
        reset();
        for i in 0..OPS_PER_ITERATION {
            track_new(&format!("var_{}", i), i);
        }
    }

    reset();

    // Memory should be freed after reset
    let events = get_events();
    assert_eq!(events.len(), 0);
}

#[test]
#[serial]
fn test_fragmented_operations() {
    const CYCLES: usize = 100;

    reset();

    for cycle in 0..CYCLES {
        for i in 0..10 {
            let name = format!("c{}_v{}", cycle, i);
            let x = track_new(&name, i);
            let _r = track_borrow("ref", &x);
            track_drop(&name);
        }
    }

    let events = get_events();
    assert_eq!(events.len(), CYCLES * 10 * 3); // new + borrow + drop
}

// ============================================================================
// String Allocation Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_long_variable_names() {
    const NAME_LENGTH: usize = 1000;
    const ITERATIONS: usize = 100;

    reset();

    let long_name = "x".repeat(NAME_LENGTH);

    let start = Instant::now();
    for i in 0..ITERATIONS {
        track_new(&format!("{}_{}", long_name, i), i);
    }
    let duration = start.elapsed();

    // Should handle long names without excessive overhead
    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(avg_ns < 5000, "Long name tracking too slow: {}ns", avg_ns);
}

#[test]
#[serial]
fn test_unicode_variable_names() {
    reset();

    let names = [
        "变量",       // Chinese
        "переменная", // Russian
        "変数",       // Japanese
        "متغير",      // Arabic
        "🦀_rust_🔥", // Emoji
    ];

    for (i, name) in names.iter().enumerate() {
        track_new(name, i);
    }

    let events = get_events();
    assert_eq!(events.len(), names.len());
}

#[test]
#[serial]
fn test_repeated_string_allocation() {
    const ITERATIONS: usize = 10_000;
    const UNIQUE_NAMES: usize = 10;

    reset();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let name = format!("var_{}", i % UNIQUE_NAMES);
        track_new(&name, i);
    }
    let duration = start.elapsed();

    // Repeated strings should not cause excessive allocation overhead
    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(
        avg_ns < 1500,
        "Repeated string allocation too slow: {}ns",
        avg_ns
    );
}

// ============================================================================
// Graph Building Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_graph_with_no_relationships() {
    const SIZE: usize = 1000;

    reset();

    for i in 0..SIZE {
        track_new(&format!("isolated_{}", i), i);
    }

    let events = get_events();
    let start = Instant::now();
    let graph = build_graph(&events);
    let duration = start.elapsed();

    assert_eq!(graph.nodes.len(), SIZE);
    assert_eq!(graph.edges.len(), 0);
    assert!(duration.as_millis() < 100);
}

#[test]
#[serial]
fn test_graph_with_dense_relationships() {
    const SIZE: usize = 100;

    reset();

    let values: Vec<_> = (0..SIZE)
        .map(|i| track_new(&format!("v{}", i), i))
        .collect();

    // Create dense borrow graph
    for i in 0..SIZE {
        for (j, value) in values.iter().enumerate() {
            if i != j {
                let _r = track_borrow(&format!("r_{}_{}", i, j), value);
            }
        }
    }

    let events = get_events();
    let start = Instant::now();
    let graph = build_graph(&events);
    let duration = start.elapsed();

    // Should have all nodes and complete quickly
    assert_eq!(graph.nodes.len(), SIZE);
    assert!(
        duration.as_millis() < 500,
        "Dense graph building too slow: {:?}",
        duration
    );

    // Verify we tracked all the borrow events (SIZE new + SIZE*(SIZE-1) borrows)
    let expected_events = SIZE + SIZE * (SIZE - 1);
    assert!(
        events.len() >= expected_events,
        "Expected at least {} events, got {}",
        expected_events,
        events.len()
    );
}

#[test]
#[serial]
fn test_graph_with_cycles() {
    reset();

    let a = track_new("a", 1);
    let b = track_new("b", 2);
    let c = track_new("c", 3);

    let _ra = track_borrow("ra", &a);
    let _rb = track_borrow("rb", &b);
    let _rc = track_borrow("rc", &c);

    let events = get_events();
    let graph = build_graph(&events);

    // Should handle potential cycles without hanging
    assert!(graph.nodes.len() >= 3);
}

// ============================================================================
// Export Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_export_empty_graph() {
    reset();

    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    let result = export.to_json();
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("nodes"));
    assert!(json.contains("events"));
}

#[test]
#[serial]
fn test_export_with_special_characters() {
    reset();

    track_new("var\"with\"quotes", 1);
    track_new("var\nwith\nnewlines", 2);
    track_new("var\\with\\backslashes", 3);

    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    let result = export.to_json();
    assert!(result.is_ok());

    let json = result.unwrap();
    // Should properly escape special characters
    assert!(!json.contains("\n\n")); // No unescaped newlines
}

#[test]
#[serial]
fn test_export_very_large_dataset() {
    const SIZE: usize = 10_000;

    reset();

    for i in 0..SIZE {
        let name = format!("var_{}", i);
        let x = track_new(&name, i);
        let _r = track_borrow("ref", &x);
        track_drop(&name);
    }

    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    let start = Instant::now();
    let result = export.to_json();
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
        duration.as_secs() < 5,
        "Export took too long: {:?}",
        duration
    );

    let json = result.unwrap();
    assert!(json.len() > 100_000); // Should be substantial
}

// ============================================================================
// Timestamp Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_timestamp_monotonicity() {
    const ITERATIONS: usize = 10_000;

    reset();

    for i in 0..ITERATIONS {
        track_new(&format!("var_{}", i), i);
    }

    let events = get_events();

    // Timestamps should be monotonically increasing
    for i in 1..events.len() {
        let prev_ts = events[i - 1].timestamp();
        let curr_ts = events[i].timestamp();
        assert!(
            curr_ts >= prev_ts,
            "Timestamp not monotonic: {} -> {}",
            prev_ts,
            curr_ts
        );
    }
}

#[test]
#[serial]
fn test_concurrent_timestamp_uniqueness() {
    const NUM_THREADS: usize = 8;
    const OPS_PER_THREAD: usize = 1000;

    reset();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    track_new(&format!("t{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    let mut timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();
    timestamps.sort_unstable();

    // Check for duplicate timestamps (should be rare but possible)
    let unique_count = timestamps
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    let duplicate_ratio = 1.0 - (unique_count as f64 / timestamps.len() as f64);

    // Allow up to 1% duplicates under high concurrency
    assert!(
        duplicate_ratio < 0.01,
        "Too many duplicate timestamps: {:.2}%",
        duplicate_ratio * 100.0
    );
}

// ============================================================================
// Smart Pointer Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_rc_clone_chain() {
    const CHAIN_LENGTH: usize = 100;

    reset();

    let rc = std::rc::Rc::new(42);
    let mut tracked = track_rc_new("rc_0", rc);

    for i in 1..CHAIN_LENGTH {
        let cloned = std::rc::Rc::clone(&tracked);
        tracked = track_rc_clone(&format!("rc_{}", i), &format!("rc_{}", i - 1), cloned);
    }

    let events = get_events();
    assert_eq!(events.len(), CHAIN_LENGTH);
}

#[test]
#[serial]
fn test_arc_cross_thread_tracking() {
    const NUM_THREADS: usize = 4;

    reset();

    let arc = std::sync::Arc::new(42);
    let tracked = track_arc_new("arc_main", arc);

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|i| {
            let arc_clone = std::sync::Arc::clone(&tracked);
            thread::spawn(move || {
                let _tracked = track_arc_clone(&format!("arc_{}", i), "arc_main", arc_clone);
                thread::sleep(std::time::Duration::from_millis(10));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    assert!(events.len() > NUM_THREADS);
}

#[test]
#[serial]
fn test_refcell_borrow_drop_cycle() {
    const CYCLES: usize = 1000;

    reset();

    let cell = std::cell::RefCell::new(42);
    let tracked_cell = track_refcell_new("cell", cell);

    for i in 0..CYCLES {
        let borrowed = tracked_cell.borrow();
        let _tracked = track_refcell_borrow(&format!("borrow_{}", i), "cell", "test", borrowed);
    }

    let events = get_events();
    assert!(events.len() >= CYCLES);
}

// ============================================================================
// Unsafe Code Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_null_pointer_tracking() {
    reset();

    let null_ptr: *const i32 = std::ptr::null();
    let _tracked = track_raw_ptr("null_ptr", 1, "i32", "test", null_ptr);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_dangling_pointer_tracking() {
    reset();

    let ptr = {
        let x = 42;
        &x as *const i32
    };

    // Pointer is now dangling but tracking should still work
    let _tracked = track_raw_ptr("dangling", 1, "i32", "test", ptr);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_fat_pointer_tracking() {
    reset();

    let slice: &[i32] = &[1, 2, 3, 4, 5];
    let ptr = slice as *const [i32];
    let _tracked = track_raw_ptr("fat_ptr", 1, "[i32]", "test", ptr);

    let trait_obj: &dyn std::fmt::Debug = &42;
    let ptr2 = trait_obj as *const dyn std::fmt::Debug;
    let _tracked2 = track_raw_ptr("trait_ptr", 2, "dyn Debug", "test", ptr2);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// ============================================================================
// Static/Const Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_multiple_static_init() {
    reset();

    for i in 0..100 {
        let _x = track_static_init(&format!("STATIC_{}", i), i, "i32", false, 42);
    }

    let events = get_events();
    assert_eq!(events.len(), 100);
}

#[test]
#[serial]
fn test_static_access_frequency() {
    const ACCESSES: usize = 10_000;

    reset();

    let start = Instant::now();
    for _ in 0..ACCESSES {
        track_static_access(1, "STATIC", false, "test");
    }
    let duration = start.elapsed();

    let avg_ns = duration.as_nanos() / ACCESSES as u128;
    assert!(
        avg_ns < 500,
        "Static access tracking too slow: {}ns",
        avg_ns
    );
}

#[test]
#[serial]
fn test_const_eval_caching() {
    const ITERATIONS: usize = 1000;

    reset();

    for _ in 0..ITERATIONS {
        let _x = track_const_eval("CONST", 1, "i32", "test", 42);
    }

    let events = get_events();
    assert_eq!(events.len(), ITERATIONS);
}

// ============================================================================
// Batch Operation Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_empty_batch_drop() {
    reset();

    track_drop_batch(&[]);

    let events = get_events();
    assert_eq!(events.len(), 0);
}

#[test]
#[serial]
fn test_single_item_batch_drop() {
    reset();

    track_drop_batch(&["x"]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_very_large_batch_drop() {
    const BATCH_SIZE: usize = 10_000;

    reset();

    let names: Vec<String> = (0..BATCH_SIZE).map(|i| format!("var_{}", i)).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    track_drop_batch(&name_refs);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "Large batch drop too slow: {:?}",
        duration
    );

    let events = get_events();
    assert_eq!(events.len(), BATCH_SIZE);
}

#[test]
#[serial]
fn test_duplicate_names_in_batch() {
    reset();

    track_drop_batch(&["x", "y", "x", "z", "y"]);

    let events = get_events();
    assert_eq!(events.len(), 5); // All drops recorded, even duplicates
}

// ============================================================================
// Error Recovery Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_tracking_after_panic_recovery() {
    reset();

    track_new("before_panic", 1);

    let result = std::panic::catch_unwind(|| {
        track_new("during_panic", 2);
        panic!("Test panic");
    });

    assert!(result.is_err());

    track_new("after_panic", 3);

    let events = get_events();
    assert!(events.len() >= 2); // before and after should be tracked
}

#[test]
#[serial]
fn test_concurrent_panic_isolation() {
    const NUM_THREADS: usize = 8;

    reset();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|i| {
            thread::spawn(move || {
                if i % 2 == 0 {
                    track_new(&format!("thread_{}", i), i);
                } else {
                    let _ = std::panic::catch_unwind(|| {
                        track_new(&format!("panic_{}", i), i);
                        panic!("Test panic");
                    });
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }

    let events = get_events();
    assert!(events.len() >= NUM_THREADS / 2);
}
