use borrowscope_runtime::*;
use proptest::prelude::*;
use serial_test::serial;

// ============================================================================
// Basic Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_track_new_preserves_value(value: i32) {
        reset();
        let result = track_new("x", value);
        prop_assert_eq!(result, value);
    }

    #[test]
    #[serial]
    fn prop_track_new_preserves_string(s in "\\PC*") {
        reset();
        let result = track_new("s", s.clone());
        prop_assert_eq!(result, s);
    }

    #[test]
    #[serial]
    fn prop_track_borrow_preserves_reference(value: i32) {
        reset();
        let x = track_new("x", value);
        let r = track_borrow("r", &x);
        prop_assert_eq!(*r, value);
    }

    #[test]
    #[serial]
    fn prop_track_move_preserves_value(value: i32) {
        reset();
        let x = track_new("x", value);
        let y = track_move("x", "y", x);
        prop_assert_eq!(y, value);
    }

    #[test]
    #[serial]
    fn prop_multiple_variables_tracked(values: Vec<i32>) {
        reset();

        for (i, &value) in values.iter().enumerate() {
            track_new(&format!("var_{}", i), value);
        }

        let events = get_events();
        prop_assert_eq!(events.len(), values.len());
    }

    #[test]
    #[serial]
    fn prop_borrow_count_matches(borrow_count in 1..20usize) {
        reset();

        let x = track_new("x", 42);

        for i in 0..borrow_count {
            let _r = track_borrow(&format!("r_{}", i), &x);
        }

        let events = get_events();
        let borrow_events = events.iter()
            .filter(|e| e.is_borrow())
            .count();

        prop_assert_eq!(borrow_events, borrow_count);
    }

    #[test]
    #[serial]
    fn prop_drop_count_matches(drop_count in 1..20usize) {
        reset();

        for i in 0..drop_count {
            track_drop(&format!("var_{}", i));
        }

        let events = get_events();
        prop_assert_eq!(events.len(), drop_count);
    }

    #[test]
    #[serial]
    fn prop_timestamp_monotonic(op_count in 1..100usize) {
        reset();

        for i in 0..op_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

        for i in 1..timestamps.len() {
            prop_assert!(timestamps[i] >= timestamps[i-1],
                "Timestamps not monotonic: {} < {}", timestamps[i], timestamps[i-1]);
        }
    }
}

// ============================================================================
// Smart Pointer Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_rc_preserves_value(value: i32) {
        reset();
        let rc = std::rc::Rc::new(value);
        let tracked = track_rc_new("rc", rc);
        prop_assert_eq!(*tracked, value);
    }

    #[test]
    #[serial]
    fn prop_rc_clone_count(clone_count in 1..10usize) {
        reset();

        let rc = std::rc::Rc::new(42);
        let mut tracked = track_rc_new("rc_0", rc);

        for i in 1..=clone_count {
            let cloned = std::rc::Rc::clone(&tracked);
            tracked = track_rc_clone(&format!("rc_{}", i), &format!("rc_{}", i-1), cloned);
        }

        let events = get_events();
        let rc_events = events.iter()
            .filter(|e| matches!(e, Event::RcNew { .. } | Event::RcClone { .. }))
            .count();

        prop_assert_eq!(rc_events, clone_count + 1); // +1 for initial new
    }

    #[test]
    #[serial]
    fn prop_arc_preserves_value(value: i32) {
        reset();
        let arc = std::sync::Arc::new(value);
        let tracked = track_arc_new("arc", arc);
        prop_assert_eq!(*tracked, value);
    }

    #[test]
    #[serial]
    fn prop_refcell_preserves_value(value: i32) {
        reset();
        let cell = std::cell::RefCell::new(value);
        let tracked = track_refcell_new("cell", cell);
        prop_assert_eq!(*tracked.borrow(), value);
    }

    #[test]
    #[serial]
    fn prop_cell_get_set(value: i32) {
        reset();
        let cell = std::cell::Cell::new(0);
        let tracked = track_cell_new("cell", cell);

        tracked.set(value);
        let result = track_cell_get("cell", "test", tracked.get());

        prop_assert_eq!(result, value);
    }
}

// ============================================================================
// Unsafe Code Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_raw_ptr_address_preserved(value: i32) {
        reset();

        let x = value;
        let ptr = &x as *const i32;
        let tracked = track_raw_ptr("ptr", 1, "i32", "test", ptr);

        prop_assert_eq!(tracked, ptr);
    }

    #[test]
    #[serial]
    fn prop_unsafe_block_tracking(block_count in 1..10usize) {
        reset();

        for i in 0..block_count {
            track_unsafe_block_enter(i, "test");
            track_unsafe_block_exit(i, "test");
        }

        let events = get_events();
        let unsafe_events = events.iter()
            .filter(|e| matches!(e, Event::UnsafeBlockEnter { .. } | Event::UnsafeBlockExit { .. }))
            .count();

        prop_assert_eq!(unsafe_events, block_count * 2);
    }
}

// ============================================================================
// Static/Const Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_static_init_preserves_value(value: i32) {
        reset();
        let result = track_static_init("STATIC", 1, "i32", false, value);
        prop_assert_eq!(result, value);
    }

    #[test]
    #[serial]
    fn prop_const_eval_preserves_value(value: i32) {
        reset();
        let result = track_const_eval("CONST", 1, "i32", "test", value);
        prop_assert_eq!(result, value);
    }

    #[test]
    #[serial]
    fn prop_static_access_count(access_count in 1..50usize) {
        reset();

        for _ in 0..access_count {
            track_static_access(1, "STATIC", false, "test");
        }

        let events = get_events();
        let access_events = events.iter()
            .filter(|e| matches!(e, Event::StaticAccess { .. }))
            .count();

        prop_assert_eq!(access_events, access_count);
    }
}

// ============================================================================
// Graph Building Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_graph_node_count(var_count in 1..50usize) {
        reset();

        for i in 0..var_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        let graph = build_graph(&events);

        prop_assert_eq!(graph.nodes.len(), var_count);
    }

    #[test]
    #[serial]
    fn prop_graph_with_borrows(var_count in 1..20usize, borrow_count in 1..10usize) {
        reset();

        let values: Vec<_> = (0..var_count)
            .map(|i| track_new(&format!("var_{}", i), i))
            .collect();

        for (i, value) in values.iter().enumerate().take(borrow_count.min(var_count)) {
            let _r = track_borrow(&format!("ref_{}", i), value);
        }

        let events = get_events();
        let graph = build_graph(&events);

        prop_assert_eq!(graph.nodes.len(), var_count);
        prop_assert!(events.len() >= var_count + borrow_count.min(var_count));
    }
}

// ============================================================================
// Batch Operation Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_batch_drop_count(drop_count in 1..100usize) {
        reset();

        let names: Vec<String> = (0..drop_count)
            .map(|i| format!("var_{}", i))
            .collect();
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

        track_drop_batch(&name_refs);

        let events = get_events();
        prop_assert_eq!(events.len(), drop_count);
    }

    #[test]
    #[serial]
    fn prop_batch_vs_individual_equivalence(drop_count in 1..50usize) {
        // Batch drops
        reset();
        let names: Vec<String> = (0..drop_count)
            .map(|i| format!("var_{}", i))
            .collect();
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        track_drop_batch(&name_refs);
        let batch_events = get_events();

        // Individual drops
        reset();
        for name in &names {
            track_drop(name);
        }
        let individual_events = get_events();

        prop_assert_eq!(batch_events.len(), individual_events.len());
    }
}

// ============================================================================
// String Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_variable_name_preserved(name in "[a-zA-Z_][a-zA-Z0-9_]{0,50}") {
        reset();
        track_new(&name, 42);

        let events = get_events();
        prop_assert_eq!(events.len(), 1);

        match &events[0] {
            Event::New { var_id, .. } => {
                prop_assert!(var_id.contains(&name));
            }
            _ => prop_assert!(false, "Expected New event"),
        }
    }

    #[test]
    #[serial]
    fn prop_unicode_names_supported(name in "[\\p{L}_][\\p{L}\\p{N}_]{0,20}") {
        reset();
        track_new(&name, 42);

        let events = get_events();
        prop_assert_eq!(events.len(), 1);
    }

    #[test]
    #[serial]
    fn prop_long_names_handled(name_len in 1..500usize) {
        reset();
        let name = "x".repeat(name_len);
        track_new(&name, 42);

        let events = get_events();
        prop_assert_eq!(events.len(), 1);
    }
}

// ============================================================================
// Concurrent Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_concurrent_operations_safe(thread_count in 1..8usize, ops_per_thread in 1..50usize) {
        reset();

        let handles: Vec<_> = (0..thread_count)
            .map(|tid| {
                std::thread::spawn(move || {
                    for i in 0..ops_per_thread {
                        track_new(&format!("t{}_v{}", tid, i), i);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        let expected_min = thread_count * ops_per_thread * 8 / 10; // Allow 20% loss
        prop_assert!(events.len() >= expected_min,
            "Expected at least {} events, got {}", expected_min, events.len());
    }
}

// ============================================================================
// Export Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_json_export_valid(var_count in 1..20usize) {
        reset();

        for i in 0..var_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        let json_result = export.to_json();
        prop_assert!(json_result.is_ok());

        let json = json_result.unwrap();
        prop_assert!(json.contains("nodes"));
        prop_assert!(json.contains("events"));
    }

    #[test]
    #[serial]
    fn prop_json_parseable(var_count in 1..10usize) {
        reset();

        for i in 0..var_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        let json = export.to_json().unwrap();
        let parse_result: std::result::Result<serde_json::Value, _> = serde_json::from_str(&json);
        prop_assert!(parse_result.is_ok());
    }
}

// ============================================================================
// Memory Property Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_memory_scales_linearly(size1 in 100..500usize, multiplier in 2..5usize) {
        // First measurement
        reset();
        for i in 0..size1 {
            track_new(&format!("var_{}", i), i);
        }
        let events1 = get_events();
        let mem1 = events1.len() * std::mem::size_of::<Event>();

        // Second measurement
        reset();
        let size2 = size1 * multiplier;
        for i in 0..size2 {
            track_new(&format!("var_{}", i), i);
        }
        let events2 = get_events();
        let mem2 = events2.len() * std::mem::size_of::<Event>();

        let ratio = mem2 as f64 / mem1 as f64;
        let expected_ratio = multiplier as f64;

        // Allow 20% variance
        prop_assert!(ratio > expected_ratio * 0.8 && ratio < expected_ratio * 1.2,
            "Memory scaling not linear: expected ~{:.1}x, got {:.1}x", expected_ratio, ratio);
    }
}

// ============================================================================
// Invariant Tests
// ============================================================================

proptest! {
    #[test]
    #[serial]
    fn prop_no_events_lost(op_count in 1..100usize) {
        reset();

        for i in 0..op_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();
        prop_assert_eq!(events.len(), op_count, "Events were lost");
    }

    #[test]
    #[serial]
    fn prop_reset_clears_all(op_count in 1..100usize) {
        reset();

        for i in 0..op_count {
            track_new(&format!("var_{}", i), i);
        }

        reset();
        let events = get_events();
        prop_assert_eq!(events.len(), 0, "Reset did not clear events");
    }

    #[test]
    #[serial]
    fn prop_event_order_preserved(op_count in 2..50usize) {
        reset();

        for i in 0..op_count {
            track_new(&format!("var_{}", i), i);
        }

        let events = get_events();

        // Check that events are in order by timestamp
        for i in 1..events.len() {
            prop_assert!(events[i].timestamp() >= events[i-1].timestamp(),
                "Event order not preserved");
        }
    }
}
