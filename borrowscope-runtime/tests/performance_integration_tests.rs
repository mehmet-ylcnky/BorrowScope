use borrowscope_runtime::*;
use serial_test::serial;
use std::time::Instant;

// ============================================================================
// Overhead Measurement Tests
// ============================================================================

#[test]
#[serial]
fn test_track_new_overhead_acceptable() {
    const ITERATIONS: usize = 10_000;
    const MAX_OVERHEAD_NS: u128 = 900; // Adjusted for cross-platform compatibility

    reset();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _x = track_new(&format!("var_{}", i), i);
    }
    let duration = start.elapsed();

    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(
        avg_ns < MAX_OVERHEAD_NS,
        "Average overhead {}ns exceeds maximum {}ns",
        avg_ns,
        MAX_OVERHEAD_NS
    );
}

#[test]
#[serial]
fn test_track_borrow_overhead_acceptable() {
    const ITERATIONS: usize = 10_000;
    const MAX_OVERHEAD_NS: u128 = 1400;

    reset();
    let values: Vec<i32> = (0..ITERATIONS as i32).collect();

    let start = Instant::now();
    for (i, val) in values.iter().enumerate() {
        let _r = track_borrow(&format!("ref_{}", i), val);
    }
    let duration = start.elapsed();

    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(
        avg_ns < MAX_OVERHEAD_NS,
        "Average overhead {}ns exceeds maximum {}ns",
        avg_ns,
        MAX_OVERHEAD_NS
    );
}

#[test]
#[serial]
fn test_track_drop_overhead_acceptable() {
    const ITERATIONS: usize = 10_000;
    const MAX_OVERHEAD_NS: u128 = 500;

    reset();

    let start = Instant::now();
    for i in 0..ITERATIONS {
        track_drop(&format!("var_{}", i));
    }
    let duration = start.elapsed();

    let avg_ns = duration.as_nanos() / ITERATIONS as u128;
    assert!(
        avg_ns < MAX_OVERHEAD_NS,
        "Average overhead {}ns exceeds maximum {}ns",
        avg_ns,
        MAX_OVERHEAD_NS
    );
}

// ============================================================================
// Batch Operation Performance Tests
// ============================================================================

#[test]
#[serial]
fn test_batch_drop_performance() {
    const COUNT: usize = 1000;

    // Individual drops
    reset();
    let start = Instant::now();
    for i in 0..COUNT {
        track_drop(&format!("var_{}", i));
    }
    let individual_duration = start.elapsed();

    // Batch drop
    reset();
    let names: Vec<String> = (0..COUNT).map(|i| format!("var_{}", i)).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    track_drop_batch(&name_refs);
    let batch_duration = start.elapsed();

    // Batch should be at least as fast or faster (but timing can vary)
    // Main goal is to verify both work correctly
    println!(
        "Individual: {:?}, Batch: {:?}",
        individual_duration, batch_duration
    );
    assert!(
        batch_duration < individual_duration * 2,
        "Batch drop should not be significantly slower than individual drops"
    );
}

#[test]
#[serial]
fn test_batch_drop_correctness() {
    reset();

    let names = vec!["x", "y", "z"];
    track_drop_batch(&names);

    let events = get_events();
    assert_eq!(events.len(), 3);

    for event in events {
        match event {
            Event::Drop { var_id, .. } => {
                assert!(names.contains(&var_id.as_str()));
            }
            _ => panic!("Expected Drop event"),
        }
    }
}

// ============================================================================
// Memory Usage Tests
// ============================================================================

#[test]
#[serial]
fn test_event_size_reasonable() {
    let event_size = std::mem::size_of::<Event>();

    // Event should be reasonably sized (< 200 bytes)
    assert!(
        event_size < 200,
        "Event size {} bytes is too large",
        event_size
    );
}

#[test]
#[serial]
fn test_memory_usage_scales_linearly() {
    let sizes = [100, 1000, 10000];
    let mut ratios = Vec::new();

    for &size in &sizes {
        reset();
        for i in 0..size {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        let memory = events.len() * std::mem::size_of::<Event>();
        let ratio = memory as f64 / size as f64;
        ratios.push(ratio);
    }

    // Check that memory usage scales linearly (ratios should be similar)
    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    for ratio in ratios {
        let deviation = (ratio - avg_ratio).abs() / avg_ratio;
        assert!(deviation < 0.1, "Memory scaling is not linear");
    }
}

#[test]
#[serial]
fn test_large_workload_memory() {
    const LARGE_SIZE: usize = 100_000;
    const MAX_MEMORY_MB: usize = 100; // 100MB limit

    reset();
    for i in 0..LARGE_SIZE {
        track_new(&format!("var_{}", i), i);
    }

    let events = get_events();
    let memory_bytes = events.len() * std::mem::size_of::<Event>();
    let memory_mb = memory_bytes / 1024 / 1024;

    assert!(
        memory_mb < MAX_MEMORY_MB,
        "Memory usage {} MB exceeds limit {} MB",
        memory_mb,
        MAX_MEMORY_MB
    );
}

// ============================================================================
// Smart Pointer Performance Tests
// ============================================================================

#[test]
#[serial]
fn test_rc_tracking_overhead() {
    const ITERATIONS: usize = 1000;
    const MAX_OVERHEAD_RATIO: f64 = 20.0; // Max 20x overhead (realistic for debug builds)

    // Baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _rc = std::rc::Rc::new(i);
    }
    let baseline = start.elapsed();

    // Tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let rc = std::rc::Rc::new(i);
        let _tracked = track_rc_new(&format!("rc_{}", i), rc);
    }
    let tracked = start.elapsed();

    let ratio = tracked.as_nanos() as f64 / baseline.as_nanos().max(1) as f64;
    assert!(
        ratio < MAX_OVERHEAD_RATIO,
        "Rc tracking overhead ratio {:.2} exceeds maximum {:.2}",
        ratio,
        MAX_OVERHEAD_RATIO
    );
}

#[test]
#[serial]
fn test_arc_tracking_overhead() {
    const ITERATIONS: usize = 1000;
    const MAX_OVERHEAD_RATIO: f64 = 20.0;

    // Baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _arc = std::sync::Arc::new(i);
    }
    let baseline = start.elapsed();

    // Tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let arc = std::sync::Arc::new(i);
        let _tracked = track_arc_new(&format!("arc_{}", i), arc);
    }
    let tracked = start.elapsed();

    let ratio = tracked.as_nanos() as f64 / baseline.as_nanos().max(1) as f64;
    assert!(
        ratio < MAX_OVERHEAD_RATIO,
        "Arc tracking overhead ratio {:.2} exceeds maximum {:.2}",
        ratio,
        MAX_OVERHEAD_RATIO
    );
}

#[test]
#[serial]
fn test_refcell_tracking_overhead() {
    const ITERATIONS: usize = 1000;
    const MAX_OVERHEAD_RATIO: f64 = 100.0; // RefCell is very fast, so tracking overhead is proportionally higher

    // Baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _cell = std::cell::RefCell::new(i);
    }
    let baseline = start.elapsed();

    // Tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let cell = std::cell::RefCell::new(i);
        let _tracked = track_refcell_new(&format!("cell_{}", i), cell);
    }
    let tracked = start.elapsed();

    let ratio = tracked.as_nanos() as f64 / baseline.as_nanos().max(1) as f64;
    assert!(
        ratio < MAX_OVERHEAD_RATIO,
        "RefCell tracking overhead ratio {:.2} exceeds maximum {:.2}",
        ratio,
        MAX_OVERHEAD_RATIO
    );
}

// ============================================================================
// Concurrent Performance Tests
// ============================================================================

#[test]
#[serial]
fn test_concurrent_tracking_correctness() {
    const OPS_PER_THREAD: usize = 100;
    const NUM_THREADS: usize = 4;

    reset();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            std::thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    track_new(&format!("var_{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    assert_eq!(events.len(), NUM_THREADS * OPS_PER_THREAD);
}

#[test]
#[serial]
fn test_concurrent_tracking_no_data_races() {
    const OPS_PER_THREAD: usize = 1000;
    const NUM_THREADS: usize = 8;

    reset();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            std::thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let name = format!("var_{}_{}", thread_id, i);
                    let x = track_new(&name, i);
                    let _r = track_borrow("ref", &x);
                    track_drop(&name);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    // Each thread does 3 operations per iteration
    // But we need to account for the actual events generated
    let expected_min = NUM_THREADS * OPS_PER_THREAD * 3;
    assert!(
        events.len() >= expected_min,
        "Expected at least {} events, got {}",
        expected_min,
        events.len()
    );
}

// ============================================================================
// Graph Building Performance Tests
// ============================================================================

#[test]
#[serial]
fn test_graph_building_performance() {
    const SIZE: usize = 1000;
    const MAX_BUILD_TIME_MS: u128 = 100; // 100ms

    reset();
    for i in 0..SIZE {
        let name = format!("var_{}", i);
        let x = track_new(&name, i);
        let _r = track_borrow("ref", &x);
        track_drop(&name);
    }

    let events = get_events();

    let start = Instant::now();
    let graph = build_graph(&events);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < MAX_BUILD_TIME_MS,
        "Graph building took {}ms, exceeds {}ms",
        duration.as_millis(),
        MAX_BUILD_TIME_MS
    );
    assert!(!graph.nodes.is_empty());
}

#[test]
#[serial]
fn test_graph_building_scales() {
    let sizes = [100, 500, 1000];
    let mut times = Vec::new();

    for &size in &sizes {
        reset();
        for i in 0..size {
            let name = format!("var_{}", i);
            let x = track_new(&name, i);
            let _r = track_borrow("ref", &x);
            track_drop(&name);
        }

        let events = get_events();

        let start = Instant::now();
        let _graph = build_graph(&events);
        let duration = start.elapsed();

        times.push(duration.as_nanos());
    }

    // Check that time scales sub-quadratically
    // time[2] / time[1] should be less than (size[2] / size[1])^2
    let ratio1 = times[1] as f64 / times[0] as f64;
    let size_ratio1 = sizes[1] as f64 / sizes[0] as f64;
    assert!(
        ratio1 < size_ratio1 * size_ratio1,
        "Graph building does not scale well"
    );
}

// ============================================================================
// Export Performance Tests
// ============================================================================

#[test]
#[serial]
fn test_json_export_performance() {
    const SIZE: usize = 1000;
    const MAX_EXPORT_TIME_MS: u128 = 200; // 200ms

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
    let json = export.to_json().unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < MAX_EXPORT_TIME_MS,
        "JSON export took {}ms, exceeds {}ms",
        duration.as_millis(),
        MAX_EXPORT_TIME_MS
    );
    assert!(!json.is_empty());
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
#[serial]
fn test_stress_many_operations() {
    const OPERATIONS: usize = 50_000;
    const MAX_TIME_MS: u128 = 5000; // 5 seconds

    reset();

    let start = Instant::now();
    for i in 0..OPERATIONS {
        let name = format!("var_{}", i);
        let x = track_new(&name, i);
        let _r = track_borrow("ref", &x);
        track_move(&name, &format!("moved_{}", i), x);
        track_drop(&format!("moved_{}", i));
    }
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < MAX_TIME_MS,
        "Stress test took {}ms, exceeds {}ms",
        duration.as_millis(),
        MAX_TIME_MS
    );

    let events = get_events();
    assert_eq!(events.len(), OPERATIONS * 4);
}

#[test]
#[serial]
fn test_stress_deep_borrow_chain() {
    const DEPTH: usize = 100;

    reset();

    let x = track_new("x", 42);
    let mut refs = vec![&x];

    for i in 0..DEPTH {
        let r = track_borrow(&format!("ref_{}", i), refs.last().unwrap());
        refs.push(r);
    }

    let events = get_events();
    assert_eq!(events.len(), 1 + DEPTH); // 1 New + DEPTH Borrows
}

#[test]
#[serial]
fn test_stress_wide_borrow_tree() {
    const WIDTH: usize = 100;

    reset();

    let x = track_new("x", 42);

    for i in 0..WIDTH {
        let _r = track_borrow(&format!("ref_{}", i), &x);
    }

    let events = get_events();
    assert_eq!(events.len(), 1 + WIDTH); // 1 New + WIDTH Borrows
}

// ============================================================================
// Feature Flag Tests
// ============================================================================

#[test]
#[cfg(not(feature = "track"))]
fn test_zero_overhead_when_disabled() {
    // When tracking is disabled, operations should have zero overhead
    const ITERATIONS: usize = 100_000;

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _x = track_new(&format!("var_{}", i), i);
    }
    let duration = start.elapsed();

    // Should be extremely fast (< 1ms) when tracking is disabled
    assert!(
        duration.as_millis() < 1,
        "Operations should be near-instant when tracking is disabled"
    );
}

#[test]
#[cfg(feature = "track")]
fn test_tracking_enabled() {
    reset();

    track_new("x", 42);

    let events = get_events();
    assert!(
        !events.is_empty(),
        "Expected at least 1 event, got {}",
        events.len()
    );
}
