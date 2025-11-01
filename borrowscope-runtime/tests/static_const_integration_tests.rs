//! Integration tests for static and const tracking
//!
//! These tests verify that static and const variables are properly tracked
//! including initialization, access patterns, and mutability.

use borrowscope_runtime::*;
use serial_test::serial;

#[test]
#[serial]
fn test_static_init() {
    reset();

    // Simulate static initialization
    let _value = track_static_init("GLOBAL_CONFIG", 1, "Config", false, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_CONFIG"));
}

#[test]
#[serial]
fn test_static_mut_init() {
    reset();

    // Simulate static mut initialization
    let _value = track_static_init("COUNTER", 2, "i32", true, 0);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("COUNTER"));
}

#[test]
#[serial]
fn test_static_read_access() {
    reset();

    // Simulate static read
    track_static_access(1, "GLOBAL_CONFIG", false, "test.rs:10:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_CONFIG"));
}

#[test]
#[serial]
fn test_static_write_access() {
    reset();

    // Simulate static mut write
    track_static_access(2, "COUNTER", true, "test.rs:15:9");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("COUNTER"));
}

#[test]
#[serial]
fn test_static_multiple_accesses() {
    reset();

    // Simulate multiple accesses to static mut
    track_static_init("COUNTER", 1, "i32", true, 0);
    track_static_access(1, "COUNTER", true, "test.rs:20:9");
    track_static_access(1, "COUNTER", false, "test.rs:21:9");
    track_static_access(1, "COUNTER", true, "test.rs:22:9");

    let events = get_events();
    assert_eq!(events.len(), 4);

    // First event is init
    assert!(events[0].is_static());

    // Rest are accesses
    for event in &events[1..] {
        assert!(event.is_static());
        assert_eq!(event.var_name(), Some("COUNTER"));
    }
}

#[test]
#[serial]
fn test_const_eval() {
    reset();

    // Simulate const evaluation
    let _value = track_const_eval("MAX_SIZE", 1, "usize", "test.rs:5:1", 100);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_const());
    assert_eq!(event.var_name(), Some("MAX_SIZE"));
}

#[test]
#[serial]
fn test_const_multiple_uses() {
    reset();

    // Simulate const being used multiple times
    let _v1 = track_const_eval("PI", 1, "f64", "test.rs:10:5", std::f64::consts::PI);
    let _v2 = track_const_eval("PI", 1, "f64", "test.rs:11:5", std::f64::consts::PI);
    let _v3 = track_const_eval("PI", 1, "f64", "test.rs:12:5", std::f64::consts::PI);

    let events = get_events();
    assert_eq!(events.len(), 3);

    for event in &events {
        assert!(event.is_const());
        assert_eq!(event.var_name(), Some("PI"));
    }
}

#[test]
#[serial]
fn test_global_event_helpers() {
    reset();

    track_static_init("STATIC_VAR", 1, "i32", false, 42);
    track_const_eval("CONST_VAR", 2, "i32", "test.rs:1:1", 100);

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Both should be global events
    assert!(events[0].is_global());
    assert!(events[1].is_global());

    // But only one is static, one is const
    assert!(events[0].is_static());
    assert!(!events[0].is_const());
    assert!(!events[1].is_static());
    assert!(events[1].is_const());
}

#[test]
#[serial]
fn test_static_with_complex_type() {
    reset();

    // Simulate static with complex type
    track_static_init("GLOBAL_MAP", 1, "HashMap<String, Vec<i32>>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_MAP"));
}

#[test]
#[serial]
fn test_lazy_static_pattern() {
    reset();

    // Simulate lazy_static initialization on first access
    track_static_init(
        "LAZY_VALUE",
        1,
        "String",
        false,
        String::from("initialized"),
    );
    track_static_access(1, "LAZY_VALUE", false, "test.rs:30:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_static());
    assert!(events[1].is_static());
}

#[test]
#[serial]
fn test_static_atomic_pattern() {
    reset();

    // Simulate atomic static
    track_static_init("ATOMIC_COUNTER", 1, "AtomicUsize", false, ());
    track_static_access(1, "ATOMIC_COUNTER", false, "test.rs:40:5");
    track_static_access(1, "ATOMIC_COUNTER", true, "test.rs:41:5");
    track_static_access(1, "ATOMIC_COUNTER", false, "test.rs:42:5");

    let events = get_events();
    assert_eq!(events.len(), 4);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_const_in_pattern_matching() {
    reset();

    // Simulate const used in pattern matching
    let _max = track_const_eval("MAX_RETRIES", 1, "u32", "test.rs:50:5", 3);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_const_array_size() {
    reset();

    // Simulate const used as array size
    let _size = track_const_eval("BUFFER_SIZE", 1, "usize", "test.rs:60:5", 1024);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_ref_pattern() {
    reset();

    // Simulate static reference
    track_static_init("EMPTY_STRING", 1, "&'static str", false, "");
    track_static_access(1, "EMPTY_STRING", false, "test.rs:70:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_mixed_global_and_local() {
    reset();

    // Mix of global and local variables
    track_const_eval("MAX", 1, "i32", "test.rs:1:1", 100);
    track_new("local_var", 2);
    track_static_access(3, "GLOBAL", false, "test.rs:5:5");
    track_borrow("borrowed", "local_var");

    let events = get_events();
    assert_eq!(events.len(), 4);

    assert!(events[0].is_const());
    assert!(events[1].is_new());
    assert!(events[2].is_static());
    assert!(events[3].is_borrow());
}

#[test]
#[serial]
fn test_static_initialization_order() {
    reset();

    // Simulate multiple static initializations
    track_static_init("FIRST", 1, "i32", false, 1);
    track_static_init("SECOND", 2, "i32", false, 2);
    track_static_init("THIRD", 3, "i32", false, 3);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify order is preserved
    assert_eq!(events[0].var_name(), Some("FIRST"));
    assert_eq!(events[1].var_name(), Some("SECOND"));
    assert_eq!(events[2].var_name(), Some("THIRD"));
}

#[test]
#[serial]
fn test_const_fn_evaluation() {
    reset();

    // Simulate const fn evaluation
    let _result = track_const_eval("COMPUTED_VALUE", 1, "i32", "test.rs:100:5", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_mut_unsafe_pattern() {
    reset();

    // Simulate unsafe static mut access pattern
    track_static_init("UNSAFE_COUNTER", 1, "i32", true, 0);

    // Multiple unsafe accesses
    for i in 0..5 {
        track_static_access(1, "UNSAFE_COUNTER", true, &format!("test.rs:{}:9", 110 + i));
    }

    let events = get_events();
    assert_eq!(events.len(), 6); // 1 init + 5 accesses

    assert!(events[0].is_static());
    for event in &events[1..] {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_thread_local_static() {
    reset();

    // Simulate thread_local static
    track_static_init("THREAD_LOCAL_VAR", 1, "RefCell<i32>", false, ());
    track_static_access(1, "THREAD_LOCAL_VAR", false, "test.rs:120:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_const_generic_parameter() {
    reset();

    // Simulate const generic parameter
    let _n = track_const_eval("N", 1, "usize", "test.rs:130:5", 10);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_lifetime_bound() {
    reset();

    // Simulate static lifetime bound
    track_static_init("STATIC_DATA", 1, "&'static [u8]", false, ());
    track_static_access(1, "STATIC_DATA", false, "test.rs:140:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}
