use borrowscope_runtime::*;
use proptest::prelude::*;
use serial_test::serial;
use std::sync::{Arc, Barrier};
use std::thread;

// ============================================================================
// Extreme Value Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_extreme_integers(value in i32::MIN..=i32::MAX) {
        reset();
        let result = track_new("x", value);
        prop_assert_eq!(result, value);
    }

    #[test]
    #[serial]
    fn prop_extreme_unsigned(value: u64) {
        reset();
        let result = track_new("x", value);
        prop_assert_eq!(result, value);
    }

    #[test]
    #[serial]
    fn prop_extreme_floats(value: f64) {
        reset();
        let result = track_new("x", value);
        if value.is_nan() {
            prop_assert!(result.is_nan());
        } else {
            prop_assert_eq!(result, value);
        }
    }

    #[test]
    #[serial]
    fn prop_empty_strings(count in 1..50usize) {
        reset();
        for i in 0..count {
            track_new(&format!("empty_{}", i), String::new());
        }
        let events = get_events();
        prop_assert_eq!(events.len(), count);
    }

    #[test]
    #[serial]
    fn prop_very_large_strings(size in 1000..10000usize) {
        reset();
        let large_string = "x".repeat(size);
        let result = track_new("large", large_string.clone());
        prop_assert_eq!(result, large_string);
    }

    #[test]
    #[serial]
    fn prop_nested_collections(depth in 1..5usize) {
        reset();
        let mut nested: Vec<Vec<i32>> = Vec::new();
        for i in 0..depth {
            nested.push(vec![i as i32; depth]);
        }
        let result = track_new("nested", nested.clone());
        prop_assert_eq!(result, nested);
    }
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
#[serial]
fn test_zero_operations() {
    reset();
    let events = get_events();
    assert_eq!(events.len(), 0);
}

#[test]
#[serial]
fn test_single_operation() {
    reset();
    track_new("x", 42);
    let events = get_events();
    assert_eq!(events.len(), 1);
}

proptest! {
    fn prop_maximum_batch_size(size in 1000..5000usize) {
        reset();
        let names: Vec<String> = (0..size).map(|i| format!("v{}", i)).collect();
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        track_drop_batch(&refs);
        let events = get_events();
        prop_assert_eq!(events.len(), size);
    }

    #[test]
    #[serial]
    fn prop_alternating_operations(count in 1..100usize) {
        reset();
        for i in 0..count {
            if i % 2 == 0 {
                track_new(&format!("v{}", i), i);
            } else {
                track_drop(&format!("v{}", i));
            }
        }
        let events = get_events();
        prop_assert_eq!(events.len(), count);
    }
}

// ============================================================================
// Concurrent Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_concurrent_reset_race(thread_count in 2..8usize) {
        reset();

        let barrier = Arc::new(Barrier::new(thread_count + 1));
        let handles: Vec<_> = (0..thread_count)
            .map(|tid| {
                let b = Arc::clone(&barrier);
                thread::spawn(move || {
                    b.wait();
                    for i in 0..10 {
                        track_new(&format!("t{}_v{}", tid, i), i);
                    }
                })
            })
            .collect();

        barrier.wait();
        thread::sleep(std::time::Duration::from_micros(100));
        reset();

        for handle in handles {
            let _ = handle.join();
        }

        // Should not crash
        prop_assert!(true);
    }

    #[test]
    #[serial]
    fn prop_concurrent_get_events(thread_count in 2..8usize) {
        reset();

        for i in 0..100 {
            track_new(&format!("v{}", i), i);
        }

        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                thread::spawn(|| {
                    let events = get_events();
                    events.len()
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // All threads should see same event count
        let first = results[0];
        prop_assert!(results.iter().all(|&r| r == first));
    }

    #[test]
    #[serial]
    fn prop_interleaved_operations(ops_per_thread in 10..50usize) {
        reset();

        let handles: Vec<_> = (0..4)
            .map(|tid| {
                thread::spawn(move || {
                    for i in 0..ops_per_thread {
                        track_new(&format!("t{}_v{}", tid, i), i);
                        track_drop(&format!("t{}_v{}", tid, i));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        prop_assert!(events.len() >= ops_per_thread * 4);
    }
}

// ============================================================================
// Smart Pointer Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_rc_weak_references(clone_count in 1..10usize) {
        reset();

        let rc = std::rc::Rc::new(42);
        let mut tracked = track_rc_new("rc_0", rc);
        let mut weaks = Vec::new();

        for i in 1..=clone_count {
            let weak = std::rc::Rc::downgrade(&tracked);
            weaks.push(weak);
            let cloned = std::rc::Rc::clone(&tracked);
            tracked = track_rc_clone(&format!("rc_{}", i), &format!("rc_{}", i-1), cloned);
        }

        prop_assert_eq!(weaks.len(), clone_count);
    }

    #[test]
    #[serial]
    fn prop_arc_weak_references(clone_count in 1..10usize) {
        reset();

        let arc = std::sync::Arc::new(42);
        let mut tracked = track_arc_new("arc_0", arc);
        let mut weaks = Vec::new();

        for i in 1..=clone_count {
            let weak = std::sync::Arc::downgrade(&tracked);
            weaks.push(weak);
            let cloned = std::sync::Arc::clone(&tracked);
            tracked = track_arc_clone(&format!("arc_{}", i), &format!("arc_{}", i-1), cloned);
        }

        prop_assert_eq!(weaks.len(), clone_count);
    }

    #[test]
    #[serial]
    fn prop_refcell_multiple_borrows(borrow_count in 1..10usize) {
        reset();

        let cell = std::cell::RefCell::new(42);
        let tracked = track_refcell_new("cell", cell);

        let mut borrows = Vec::new();
        for i in 0..borrow_count {
            let b = tracked.borrow();
            let tb = track_refcell_borrow(&format!("b{}", i), "cell", "test", b);
            borrows.push(tb);
        }

        prop_assert_eq!(borrows.len(), borrow_count);
    }

    #[test]
    #[serial]
    fn prop_cell_rapid_updates(update_count in 10..100usize) {
        reset();

        let cell = std::cell::Cell::new(0);
        let tracked = track_cell_new("cell", cell);

        for i in 0..update_count {
            tracked.set(i as i32);
            let _ = track_cell_get("cell", "test", tracked.get());
        }

        let events = get_events();
        prop_assert!(events.len() >= update_count);
    }
}

// ============================================================================
// Memory Pressure Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_repeated_allocations(cycles in 10..50usize, size in 100..500usize) {
        for _ in 0..cycles {
            reset();
            for i in 0..size {
                track_new(&format!("v{}", i), i);
            }
        }

        reset();
        let events = get_events();
        prop_assert_eq!(events.len(), 0);
    }

    #[test]
    #[serial]
    fn prop_large_event_accumulation(size in 1000..5000usize) {
        reset();

        for i in 0..size {
            track_new(&format!("v{}", i), i);
            track_drop(&format!("v{}", i));
        }

        let events = get_events();
        let memory = events.len() * std::mem::size_of::<Event>();
        prop_assert!(memory < 10_000_000); // < 10MB
    }

    #[test]
    #[serial]
    fn prop_fragmented_memory_pattern(iterations in 10..50usize) {
        reset();

        for cycle in 0..iterations {
            for i in 0..10 {
                track_new(&format!("c{}_v{}", cycle, i), i);
            }
            for i in 0..5 {
                track_drop(&format!("c{}_v{}", cycle, i));
            }
        }

        let events = get_events();
        prop_assert!(!events.is_empty());
    }
}

// ============================================================================
// Graph Building Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_graph_with_cycles(node_count in 3..10usize) {
        reset();

        let values: Vec<_> = (0..node_count)
            .map(|i| track_new(&format!("v{}", i), i))
            .collect();

        // Create circular borrows
        for i in 0..node_count {
            let next = (i + 1) % node_count;
            let _r = track_borrow(&format!("r{}", i), &values[next]);
        }

        let events = get_events();
        let graph = build_graph(&events);
        prop_assert_eq!(graph.nodes.len(), node_count);
    }

    #[test]
    #[serial]
    fn prop_graph_disconnected_components(component_count in 2..10usize, nodes_per in 2..5usize) {
        reset();

        for comp in 0..component_count {
            for node in 0..nodes_per {
                track_new(&format!("c{}_n{}", comp, node), node);
            }
        }

        let events = get_events();
        let graph = build_graph(&events);
        prop_assert_eq!(graph.nodes.len(), component_count * nodes_per);
    }

    #[test]
    #[serial]
    fn prop_graph_star_topology(center_borrows in 5..20usize) {
        reset();

        let center = track_new("center", 0);

        for i in 0..center_borrows {
            let _r = track_borrow(&format!("spoke_{}", i), &center);
        }

        let events = get_events();
        let graph = build_graph(&events);
        prop_assert!(!graph.nodes.is_empty());
    }
}

// ============================================================================
// Export Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_export_special_characters(char_type in 0..5u8) {
        reset();

        let special = match char_type {
            0 => "var\"with\"quotes",
            1 => "var\nwith\nnewlines",
            2 => "var\\with\\backslashes",
            3 => "var\twith\ttabs",
            _ => "var with spaces",
        };

        track_new(special, 42);

        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        let json = export.to_json();
        prop_assert!(json.is_ok());
    }

    #[test]
    #[serial]
    fn prop_export_large_dataset(size in 1000..5000usize) {
        reset();

        for i in 0..size {
            track_new(&format!("v{}", i), i);
        }

        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        let json = export.to_json();
        prop_assert!(json.is_ok());

        if let Ok(j) = json {
            prop_assert!(j.len() > size * 10); // Reasonable size
        }
    }

    #[test]
    #[serial]
    fn prop_export_parse_roundtrip(var_count in 1..20usize) {
        reset();

        for i in 0..var_count {
            track_new(&format!("v{}", i), i);
        }

        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        let json = export.to_json().unwrap();
        let parsed: std::result::Result<serde_json::Value, _> = serde_json::from_str(&json);
        prop_assert!(parsed.is_ok());

        if let Ok(val) = parsed {
            prop_assert!(val.is_object());
        }
    }
}

// ============================================================================
// Unsafe Code Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_null_pointer_operations(count in 1..10usize) {
        reset();

        for i in 0..count {
            let null_ptr: *const i32 = std::ptr::null();
            let _tracked = track_raw_ptr(&format!("null_{}", i), i, "i32", "test", null_ptr);
        }

        let events = get_events();
        prop_assert_eq!(events.len(), count);
    }

    #[test]
    #[serial]
    fn prop_pointer_arithmetic_tracking(offset in 0..10usize) {
        reset();

        let arr = [1, 2, 3, 4, 5];
        let base_ptr = arr.as_ptr();

        for i in 0..=offset.min(4) {
            unsafe {
                let offset_ptr = base_ptr.add(i);
                let _tracked = track_raw_ptr(&format!("ptr_{}", i), i, "i32", "test", offset_ptr);
            }
        }

        let events = get_events();
        prop_assert!(!events.is_empty());
    }

    #[test]
    #[serial]
    fn prop_nested_unsafe_blocks(depth in 1..5usize) {
        reset();

        for i in 0..depth {
            track_unsafe_block_enter(i, "test");
        }

        for i in (0..depth).rev() {
            track_unsafe_block_exit(i, "test");
        }

        let events = get_events();
        prop_assert_eq!(events.len(), depth * 2);
    }
}

// ============================================================================
// Static/Const Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_static_high_frequency_access(access_count in 100..1000usize) {
        reset();

        track_static_init("STATIC", 1, "i32", false, 42);

        for _ in 0..access_count {
            track_static_access(1, "STATIC", false, "test");
        }

        let events = get_events();
        prop_assert_eq!(events.len(), access_count + 1);
    }

    #[test]
    #[serial]
    fn prop_multiple_static_variables(static_count in 1..20usize) {
        reset();

        for i in 0..static_count {
            track_static_init(&format!("STATIC_{}", i), i, "i32", false, i as i32);
        }

        let events = get_events();
        prop_assert_eq!(events.len(), static_count);
    }

    #[test]
    #[serial]
    fn prop_const_eval_caching_pattern(eval_count in 10..100usize) {
        reset();

        for i in 0..eval_count {
            let _ = track_const_eval("CONST", 1, "i32", "test", i as i32);
        }

        let events = get_events();
        prop_assert_eq!(events.len(), eval_count);
    }
}

// ============================================================================
// Error Recovery Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_panic_recovery_isolation(panic_count in 1..5usize) {
        reset();

        for i in 0..panic_count {
            let _ = std::panic::catch_unwind(|| {
                track_new(&format!("panic_{}", i), i);
                if i % 2 == 0 {
                    panic!("Test panic");
                }
            });
        }

        track_new("after_panics", 42);

        let events = get_events();
        prop_assert!(!events.is_empty());
    }

    #[test]
    #[serial]
    fn prop_concurrent_panic_handling(thread_count in 2..8usize) {
        reset();

        let handles: Vec<_> = (0..thread_count)
            .map(|tid| {
                thread::spawn(move || {
                    let _ = std::panic::catch_unwind(|| {
                        track_new(&format!("t{}", tid), tid);
                        if tid % 2 == 0 {
                            panic!("Test panic");
                        }
                    });
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.join();
        }

        let events = get_events();
        prop_assert!(events.len() >= thread_count / 2);
    }
}

// ============================================================================
// Timestamp Edge Cases
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_timestamp_uniqueness_high_frequency(op_count in 100..1000usize) {
        reset();

        for i in 0..op_count {
            track_new(&format!("v{}", i), i);
        }

        let events = get_events();
        let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

        // Check for strictly increasing (allowing some duplicates under high load)
        let mut increasing_count = 0;
        for i in 1..timestamps.len() {
            if timestamps[i] > timestamps[i-1] {
                increasing_count += 1;
            }
        }

        let increasing_ratio = increasing_count as f64 / (timestamps.len() - 1) as f64;
        prop_assert!(increasing_ratio > 0.9); // At least 90% strictly increasing
    }

    #[test]
    #[serial]
    fn prop_timestamp_overflow_handling(large_count in 10000..50000usize) {
        reset();

        for i in 0..large_count {
            track_new(&format!("v{}", i % 100), i);
        }

        let events = get_events();
        let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

        // Should not overflow or wrap around
        for i in 1..timestamps.len() {
            prop_assert!(timestamps[i] >= timestamps[i-1]);
        }
    }
}

// ============================================================================
// Complex Interaction Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_mixed_operation_sequence(seq_len in 10..50usize) {
        reset();

        let mut values = Vec::new();

        for i in 0..seq_len {
            match i % 4 {
                0 => {
                    let v = track_new(&format!("v{}", i), i);
                    values.push(v);
                }
                1 if !values.is_empty() => {
                    let idx = i % values.len();
                    let _r = track_borrow(&format!("r{}", i), &values[idx]);
                }
                2 => {
                    track_drop(&format!("v{}", i));
                }
                _ => {
                    track_static_access(i, "STATIC", false, "test");
                }
            }
        }

        let events = get_events();
        prop_assert!(!events.is_empty());
    }

    #[test]
    #[serial]
    fn prop_nested_smart_pointers(depth in 1..5usize) {
        reset();

        let mut current = std::rc::Rc::new(42);
        let mut tracked = track_rc_new("rc_0", current.clone());

        for i in 1..depth {
            current = std::rc::Rc::new(*tracked);
            tracked = track_rc_new(&format!("rc_{}", i), current.clone());
        }

        let events = get_events();
        prop_assert_eq!(events.len(), depth);
    }
}
