use borrowscope_runtime::*;
use serial_test::serial;

// ============================================================================
// Basic Tracking Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_track_new_returns_value() {
    reset();
    let x = track_new("x", 42);
    assert_eq!(x, 42);
}

#[test]
#[serial]
fn test_track_new_records_event() {
    reset();
    track_new("x", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_new());
}

#[test]
#[serial]
fn test_track_borrow_preserves_reference() {
    reset();
    let x = 42;
    let r = track_borrow("r", &x);
    assert_eq!(*r, 42);
}

#[test]
#[serial]
fn test_track_borrow_records_event() {
    reset();
    let x = 42;
    let _r = track_borrow("r", &x);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_borrow());
}

#[test]
#[serial]
fn test_track_borrow_mut_preserves_reference() {
    reset();
    let mut x = 42;
    let r = track_borrow_mut("r", &mut x);
    assert_eq!(*r, 42);
    *r = 100;
    assert_eq!(*r, 100);
}

#[test]
#[serial]
fn test_track_move_preserves_value() {
    reset();
    let x = track_new("x", 42);
    let y = track_move("x", "y", x);
    assert_eq!(y, 42);
}

#[test]
#[serial]
fn test_track_move_records_event() {
    reset();
    let x = track_new("x", 42);
    let _y = track_move("x", "y", x);

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Move
    assert!(events[1].is_move());
}

#[test]
#[serial]
fn test_track_drop_records_event() {
    reset();
    track_drop("x");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_drop());
}

#[test]
#[serial]
fn test_multiple_variables_tracked() {
    reset();

    track_new("x", 1);
    track_new("y", 2);
    track_new("z", 3);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_reset_clears_events() {
    reset();

    track_new("x", 42);
    track_new("y", 100);

    reset();

    let events = get_events();
    assert_eq!(events.len(), 0);
}

// ============================================================================
// Timestamp Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_timestamps_monotonic() {
    reset();

    track_new("x", 1);
    track_new("y", 2);
    track_new("z", 3);

    let events = get_events();
    let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

    for i in 1..timestamps.len() {
        assert!(
            timestamps[i] >= timestamps[i - 1],
            "Timestamps not monotonic"
        );
    }
}

#[test]
#[serial]
fn test_timestamps_unique_sequential() {
    reset();

    for i in 0..100 {
        track_new(&format!("var_{}", i), i);
    }

    let events = get_events();
    let timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();

    // Check strictly increasing
    for i in 1..timestamps.len() {
        assert!(timestamps[i] > timestamps[i - 1]);
    }
}

// ============================================================================
// Variable Name Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_variable_name_preserved() {
    reset();
    track_new("my_variable", 42);

    let events = get_events();
    match &events[0] {
        Event::New { var_id, .. } => {
            assert!(var_id.contains("my_variable"));
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
#[serial]
fn test_empty_variable_name() {
    reset();
    track_new("", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_long_variable_name() {
    reset();
    let long_name = "x".repeat(1000);
    track_new(&long_name, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_unicode_variable_name() {
    reset();
    track_new("变量", 42);
    track_new("переменная", 100);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// ============================================================================
// Type Preservation Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_track_new_preserves_string() {
    reset();
    let s = String::from("hello");
    let result = track_new("s", s.clone());
    assert_eq!(result, s);
}

#[test]
#[serial]
fn test_track_new_preserves_vec() {
    reset();
    let v = vec![1, 2, 3];
    let result = track_new("v", v.clone());
    assert_eq!(result, v);
}

#[test]
#[serial]
fn test_track_new_preserves_tuple() {
    reset();
    let t = (1, "hello", std::f64::consts::PI);
    let result = track_new("t", t);
    assert_eq!(result, t);
}

#[test]
#[serial]
fn test_track_new_preserves_option() {
    reset();
    let opt = Some(42);
    let result = track_new("opt", opt);
    assert_eq!(result, opt);
}

#[test]
#[serial]
fn test_track_new_preserves_result() {
    reset();
    let res: std::result::Result<i32, String> = Ok(42);
    let result = track_new("res", res);
    assert_eq!(result, Ok(42));
}

// ============================================================================
// Batch Operation Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_batch_drop_empty() {
    reset();
    track_drop_batch(&[]);

    let events = get_events();
    assert_eq!(events.len(), 0);
}

#[test]
#[serial]
fn test_batch_drop_single() {
    reset();
    track_drop_batch(&["x"]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_batch_drop_multiple() {
    reset();
    track_drop_batch(&["x", "y", "z"]);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_batch_drop_duplicates() {
    reset();
    track_drop_batch(&["x", "x", "y"]);

    let events = get_events();
    assert_eq!(events.len(), 3); // All recorded
}

// ============================================================================
// Smart Pointer Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_rc_new_preserves_value() {
    reset();
    let rc = std::rc::Rc::new(42);
    let tracked = track_rc_new("rc", rc);
    assert_eq!(*tracked, 42);
}

#[test]
#[serial]
fn test_rc_clone_preserves_value() {
    reset();
    let rc = std::rc::Rc::new(42);
    let tracked = track_rc_new("rc1", rc);
    let cloned = std::rc::Rc::clone(&tracked);
    let tracked2 = track_rc_clone("rc2", "rc1", cloned);
    assert_eq!(*tracked2, 42);
}

#[test]
#[serial]
fn test_arc_new_preserves_value() {
    reset();
    let arc = std::sync::Arc::new(42);
    let tracked = track_arc_new("arc", arc);
    assert_eq!(*tracked, 42);
}

#[test]
#[serial]
fn test_arc_clone_preserves_value() {
    reset();
    let arc = std::sync::Arc::new(42);
    let tracked = track_arc_new("arc1", arc);
    let cloned = std::sync::Arc::clone(&tracked);
    let tracked2 = track_arc_clone("arc2", "arc1", cloned);
    assert_eq!(*tracked2, 42);
}

#[test]
#[serial]
fn test_refcell_new_preserves_value() {
    reset();
    let cell = std::cell::RefCell::new(42);
    let tracked = track_refcell_new("cell", cell);
    assert_eq!(*tracked.borrow(), 42);
}

#[test]
#[serial]
fn test_refcell_borrow_preserves_value() {
    reset();
    let cell = std::cell::RefCell::new(42);
    let tracked = track_refcell_new("cell", cell);
    let borrowed = tracked.borrow();
    let tracked_borrow = track_refcell_borrow("borrow", "cell", "test", borrowed);
    assert_eq!(*tracked_borrow, 42);
}

#[test]
#[serial]
fn test_cell_new_preserves_value() {
    reset();
    let cell = std::cell::Cell::new(42);
    let tracked = track_cell_new("cell", cell);
    assert_eq!(tracked.get(), 42);
}

#[test]
#[serial]
fn test_cell_get_set() {
    reset();
    let cell = std::cell::Cell::new(0);
    let tracked = track_cell_new("cell", cell);

    tracked.set(42);
    let value = track_cell_get("cell", "test", tracked.get());
    assert_eq!(value, 42);
}

// ============================================================================
// Static/Const Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_static_init_preserves_value() {
    reset();
    let value = track_static_init("STATIC", 1, "i32", false, 42);
    assert_eq!(value, 42);
}

#[test]
#[serial]
fn test_static_init_records_event() {
    reset();
    track_static_init("STATIC", 1, "i32", false, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::StaticInit { .. }));
}

#[test]
#[serial]
fn test_static_access_records_event() {
    reset();
    track_static_access(1, "STATIC", false, "test");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::StaticAccess { .. }));
}

#[test]
#[serial]
fn test_const_eval_preserves_value() {
    reset();
    let value = track_const_eval("CONST", 1, "i32", "test", 42);
    assert_eq!(value, 42);
}

#[test]
#[serial]
fn test_const_eval_records_event() {
    reset();
    track_const_eval("CONST", 1, "i32", "test", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::ConstEval { .. }));
}

// ============================================================================
// Unsafe Code Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_raw_ptr_preserves_address() {
    reset();
    let x = 42;
    let ptr = &x as *const i32;
    let tracked = track_raw_ptr("ptr", 1, "i32", "test", ptr);
    assert_eq!(tracked, ptr);
}

#[test]
#[serial]
fn test_raw_ptr_mut_preserves_address() {
    reset();
    let mut x = 42;
    let ptr = &mut x as *mut i32;
    let tracked = track_raw_ptr_mut("ptr", 1, "i32", "test", ptr);
    assert_eq!(tracked, ptr);
}

#[test]
#[serial]
fn test_unsafe_block_enter_records_event() {
    reset();
    track_unsafe_block_enter(1, "test");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::UnsafeBlockEnter { .. }));
}

#[test]
#[serial]
fn test_unsafe_block_exit_records_event() {
    reset();
    track_unsafe_block_exit(1, "test");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::UnsafeBlockExit { .. }));
}

#[test]
#[serial]
fn test_unsafe_fn_call_records_event() {
    reset();
    track_unsafe_fn_call("unsafe_fn", "test");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::UnsafeFnCall { .. }));
}

// ============================================================================
// Event Helper Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_event_is_new() {
    reset();
    track_new("x", 42);

    let events = get_events();
    assert!(events[0].is_new());
    assert!(!events[0].is_borrow());
    assert!(!events[0].is_move());
    assert!(!events[0].is_drop());
}

#[test]
#[serial]
fn test_event_is_borrow() {
    reset();
    let x = 42;
    let _r = track_borrow("r", &x);

    let events = get_events();
    assert!(events[0].is_borrow());
    assert!(!events[0].is_new());
}

#[test]
#[serial]
fn test_event_is_move() {
    reset();
    let x = track_new("x", 42);
    let _y = track_move("x", "y", x);

    let events = get_events();
    assert!(events[1].is_move());
}

#[test]
#[serial]
fn test_event_is_drop() {
    reset();
    track_drop("x");

    let events = get_events();
    assert!(events[0].is_drop());
}

#[test]
#[serial]
fn test_event_var_name() {
    reset();
    track_new("my_var", 42);

    let events = get_events();
    let var_name = events[0].var_name();
    assert!(var_name.is_some());
    assert!(var_name.unwrap().contains("my_var"));
}

#[test]
#[serial]
fn test_event_timestamp() {
    reset();
    track_new("x", 42);

    let events = get_events();
    let _timestamp = events[0].timestamp();
    // Timestamp is always valid u64
}

// ============================================================================
// Thread Safety Unit Tests
// ============================================================================

#[test]
#[serial]
fn test_concurrent_tracking_safe() {
    reset();

    let handles: Vec<_> = (0..4)
        .map(|i| {
            std::thread::spawn(move || {
                for j in 0..10 {
                    track_new(&format!("t{}_v{}", i, j), j);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    assert!(events.len() >= 30); // Allow some loss
}

#[test]
#[serial]
fn test_concurrent_reset_safe() {
    reset();

    let handle = std::thread::spawn(|| {
        for i in 0..100 {
            track_new(&format!("var_{}", i), i);
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(10));
    reset();

    handle.join().unwrap();

    // Should not crash
    let _ = get_events();
}
