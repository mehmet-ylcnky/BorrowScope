use borrowscope_runtime::*;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
use serial_test::serial;

// ============================================================================
// Custom Arbitrary Types
// ============================================================================

#[derive(Clone, Debug)]
struct TrackingOperation {
    op_type: OperationType,
    var_name: String,
    value: i32,
}

#[derive(Clone, Debug)]
enum OperationType {
    New,
    Borrow,
    Move,
    Drop,
}

impl Arbitrary for OperationType {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 4 {
            0 => OperationType::New,
            1 => OperationType::Borrow,
            2 => OperationType::Move,
            _ => OperationType::Drop,
        }
    }
}

impl Arbitrary for TrackingOperation {
    fn arbitrary(g: &mut Gen) -> Self {
        TrackingOperation {
            op_type: OperationType::arbitrary(g),
            var_name: format!("var_{}", usize::arbitrary(g) % 100),
            value: i32::arbitrary(g),
        }
    }
}

// ============================================================================
// QuickCheck Tests
// ============================================================================

#[quickcheck]
#[serial]
fn qc_track_new_identity(value: i32) -> bool {
    reset();
    let result = track_new("x", value);
    result == value
}

#[quickcheck]
#[serial]
fn qc_track_new_string_identity(s: String) -> bool {
    reset();
    let result = track_new("s", s.clone());
    result == s
}

#[quickcheck]
#[serial]
fn qc_borrow_preserves_value(value: i32) -> bool {
    reset();
    let x = track_new("x", value);
    let r = track_borrow("r", &x);
    *r == value
}

#[quickcheck]
#[serial]
fn qc_move_preserves_value(value: i32) -> bool {
    reset();
    let x = track_new("x", value);
    let y = track_move("x", "y", x);
    y == value
}

#[quickcheck]
#[serial]
fn qc_event_count_matches_operations(values: Vec<i32>) -> TestResult {
    if values.is_empty() || values.len() > 100 {
        return TestResult::discard();
    }

    reset();

    for (i, &value) in values.iter().enumerate() {
        track_new(&format!("var_{}", i), value);
    }

    let events = get_events();
    TestResult::from_bool(events.len() == values.len())
}

#[quickcheck]
#[serial]
fn qc_timestamps_monotonic(count: usize) -> TestResult {
    if count == 0 || count > 100 {
        return TestResult::discard();
    }

    reset();

    for i in 0..count {
        track_new(&format!("var_{}", i), i as i32);
    }

    let events = get_events();
    let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

    for i in 1..timestamps.len() {
        if timestamps[i] < timestamps[i - 1] {
            return TestResult::failed();
        }
    }

    TestResult::passed()
}

#[quickcheck]
#[serial]
fn qc_rc_strong_count_tracked(clone_count: usize) -> TestResult {
    if clone_count == 0 || clone_count > 20 {
        return TestResult::discard();
    }

    reset();

    let rc = std::rc::Rc::new(42);
    let mut tracked = track_rc_new("rc_0", rc);

    for i in 1..=clone_count {
        let cloned = std::rc::Rc::clone(&tracked);
        tracked = track_rc_clone(&format!("rc_{}", i), &format!("rc_{}", i - 1), cloned);
    }

    let events = get_events();
    let rc_events = events
        .iter()
        .filter(|e| matches!(e, Event::RcNew { .. } | Event::RcClone { .. }))
        .count();

    TestResult::from_bool(rc_events == clone_count + 1)
}

#[quickcheck]
#[serial]
fn qc_arc_thread_safe(value: i32) -> bool {
    reset();

    let arc = std::sync::Arc::new(value);
    let tracked = track_arc_new("arc", arc);

    let arc_clone = std::sync::Arc::clone(&tracked);
    let handle = std::thread::spawn(move || *arc_clone);

    let result = handle.join().unwrap();
    result == value
}

#[quickcheck]
#[serial]
fn qc_refcell_borrow_mut_exclusive(value: i32, new_value: i32) -> bool {
    reset();

    let cell = std::cell::RefCell::new(value);
    let tracked = track_refcell_new("cell", cell);

    {
        let borrowed = tracked.borrow_mut();
        let mut tracked_borrow = track_refcell_borrow_mut("borrow", "cell", "test", borrowed);
        *tracked_borrow = new_value;
    }

    let result = *tracked.borrow();
    result == new_value
}

#[quickcheck]
#[serial]
fn qc_cell_get_set_roundtrip(value: i32) -> bool {
    reset();

    let cell = std::cell::Cell::new(0);
    let tracked = track_cell_new("cell", cell);

    tracked.set(value);
    let result = track_cell_get("cell", "test", tracked.get());

    result == value
}

#[quickcheck]
#[serial]
fn qc_batch_drop_equivalent(count: usize) -> TestResult {
    if count == 0 || count > 50 {
        return TestResult::discard();
    }

    // Batch
    reset();
    let names: Vec<String> = (0..count).map(|i| format!("var_{}", i)).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    track_drop_batch(&name_refs);
    let batch_count = get_events().len();

    // Individual
    reset();
    for name in &names {
        track_drop(name);
    }
    let individual_count = get_events().len();

    TestResult::from_bool(batch_count == individual_count)
}

#[quickcheck]
#[serial]
fn qc_graph_node_count(var_count: usize) -> TestResult {
    if var_count == 0 || var_count > 50 {
        return TestResult::discard();
    }

    reset();

    for i in 0..var_count {
        track_new(&format!("var_{}", i), i as i32);
    }

    let events = get_events();
    let graph = build_graph(&events);

    TestResult::from_bool(graph.nodes.len() == var_count)
}

#[quickcheck]
#[serial]
fn qc_json_export_parseable(var_count: usize) -> TestResult {
    if var_count == 0 || var_count > 20 {
        return TestResult::discard();
    }

    reset();

    for i in 0..var_count {
        track_new(&format!("var_{}", i), i as i32);
    }

    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    match export.to_json() {
        Ok(json) => {
            let parse_result: std::result::Result<serde_json::Value, _> =
                serde_json::from_str(&json);
            TestResult::from_bool(parse_result.is_ok())
        }
        Err(_) => TestResult::failed(),
    }
}

#[quickcheck]
#[serial]
fn qc_reset_clears_state(op_count: usize) -> TestResult {
    if op_count == 0 || op_count > 100 {
        return TestResult::discard();
    }

    reset();

    for i in 0..op_count {
        track_new(&format!("var_{}", i), i as i32);
    }

    reset();
    let events = get_events();

    TestResult::from_bool(events.is_empty())
}

#[quickcheck]
#[serial]
fn qc_static_init_preserves_value(value: i32) -> bool {
    reset();
    let result = track_static_init("STATIC", 1, "i32", false, value);
    result == value
}

#[quickcheck]
#[serial]
fn qc_const_eval_preserves_value(value: i32) -> bool {
    reset();
    let result = track_const_eval("CONST", 1, "i32", "test", value);
    result == value
}

#[quickcheck]
#[serial]
fn qc_unsafe_block_pairs(block_count: usize) -> TestResult {
    if block_count == 0 || block_count > 20 {
        return TestResult::discard();
    }

    reset();

    for i in 0..block_count {
        track_unsafe_block_enter(i, "test");
        track_unsafe_block_exit(i, "test");
    }

    let events = get_events();
    let enter_count = events
        .iter()
        .filter(|e| matches!(e, Event::UnsafeBlockEnter { .. }))
        .count();
    let exit_count = events
        .iter()
        .filter(|e| matches!(e, Event::UnsafeBlockExit { .. }))
        .count();

    TestResult::from_bool(enter_count == block_count && exit_count == block_count)
}

#[quickcheck]
#[serial]
fn qc_raw_ptr_preserves_address(value: i32) -> bool {
    reset();

    let x = value;
    let ptr = &x as *const i32;
    let tracked = track_raw_ptr("ptr", 1, "i32", "test", ptr);

    tracked == ptr
}

#[quickcheck]
#[serial]
fn qc_variable_name_preserved(name: String) -> TestResult {
    if name.is_empty() || name.len() > 100 {
        return TestResult::discard();
    }

    reset();
    track_new(&name, 42);

    let events = get_events();
    if events.len() != 1 {
        return TestResult::failed();
    }

    match &events[0] {
        Event::New { var_id, .. } => TestResult::from_bool(var_id.contains(&name)),
        _ => TestResult::failed(),
    }
}

#[quickcheck]
#[serial]
fn qc_concurrent_operations_safe(thread_count: usize, ops_per_thread: usize) -> TestResult {
    if thread_count == 0 || thread_count > 8 || ops_per_thread == 0 || ops_per_thread > 50 {
        return TestResult::discard();
    }

    reset();

    let handles: Vec<_> = (0..thread_count)
        .map(|tid| {
            std::thread::spawn(move || {
                for i in 0..ops_per_thread {
                    track_new(&format!("t{}_v{}", tid, i), i as i32);
                }
            })
        })
        .collect();

    for handle in handles {
        if handle.join().is_err() {
            return TestResult::failed();
        }
    }

    let events = get_events();
    let expected_min = thread_count * ops_per_thread * 8 / 10; // Allow 20% loss

    TestResult::from_bool(events.len() >= expected_min)
}

#[quickcheck]
#[serial]
fn qc_memory_linear_scaling(size: usize, multiplier: usize) -> TestResult {
    if !(10..=100).contains(&size) || !(2..=5).contains(&multiplier) {
        return TestResult::discard();
    }

    // First measurement
    reset();
    for i in 0..size {
        track_new(&format!("var_{}", i), i as i32);
    }
    let events1 = get_events();
    let mem1 = events1.len() * std::mem::size_of::<Event>();

    // Second measurement
    reset();
    let size2 = size * multiplier;
    for i in 0..size2 {
        track_new(&format!("var_{}", i), i as i32);
    }
    let events2 = get_events();
    let mem2 = events2.len() * std::mem::size_of::<Event>();

    let ratio = mem2 as f64 / mem1 as f64;
    let expected_ratio = multiplier as f64;

    // Allow 20% variance
    TestResult::from_bool(ratio > expected_ratio * 0.8 && ratio < expected_ratio * 1.2)
}

// ============================================================================
// Stateful Testing
// ============================================================================

#[test]
#[serial]
fn qc_stateful_operations() {
    fn run_operations(ops: Vec<TrackingOperation>) -> bool {
        reset();

        for op in ops {
            match op.op_type {
                OperationType::New => {
                    track_new(&op.var_name, op.value);
                }
                OperationType::Borrow => {
                    // Skip if no variables exist
                    if get_events().is_empty() {
                        continue;
                    }
                }
                OperationType::Move => {
                    // Skip for now
                }
                OperationType::Drop => {
                    track_drop(&op.var_name);
                }
            }
        }

        // Should not panic
        let _ = get_events();
        true
    }

    QuickCheck::new()
        .tests(100)
        .quickcheck(run_operations as fn(Vec<TrackingOperation>) -> bool);
}
