//! Tests for medium priority tracking features
//!
//! Tests for:
//! - Lock guard acquire/drop tracking
//! - Closure create/capture tracking
//! - Box::into_raw/from_raw tracking

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset, Event};
use serial_test::serial;
use std::sync::{Mutex, RwLock};

/// Helper to check if events contain a specific event type
fn has_event_type(events: &[Event], type_name: &str) -> bool {
    events
        .iter()
        .any(|e| format!("{:?}", e).contains(type_name))
}

/// Helper to count events of a specific type
fn count_event_type(events: &[Event], type_name: &str) -> usize {
    events
        .iter()
        .filter(|e| format!("{:?}", e).contains(type_name))
        .count()
}

// ============================================================================
// Lock Guard Tests
// ============================================================================

#[test]
#[serial]
fn test_mutex_lock_guard_acquire() {
    reset();

    #[trace_borrow]
    fn lock_mutex() {
        let mutex = Mutex::new(42);
        let _guard = mutex.lock().unwrap();
    }

    lock_mutex();
    let events = get_events();

    assert!(
        has_event_type(&events, "LockGuardAcquire"),
        "Should track lock guard acquisition"
    );
    assert!(
        has_event_type(&events, "Lock"),
        "Should track lock operation"
    );
}

#[test]
#[serial]
fn test_rwlock_read_guard_acquire() {
    reset();

    #[trace_borrow]
    fn read_rwlock() {
        let rwlock = RwLock::new(100);
        let _guard = rwlock.read().unwrap();
    }

    read_rwlock();
    let events = get_events();

    assert!(
        has_event_type(&events, "LockGuardAcquire"),
        "Should track read guard acquisition"
    );

    // Check lock type is rwlock_read
    let guard_event = events
        .iter()
        .find(|e| format!("{:?}", e).contains("LockGuardAcquire"))
        .map(|e| format!("{:?}", e));
    assert!(
        guard_event.unwrap().contains("rwlock_read"),
        "Should be rwlock_read type"
    );
}

#[test]
#[serial]
fn test_rwlock_write_guard_acquire() {
    reset();

    #[trace_borrow]
    fn write_rwlock() {
        let rwlock = RwLock::new(100);
        let _guard = rwlock.write().unwrap();
    }

    write_rwlock();
    let events = get_events();

    assert!(
        has_event_type(&events, "LockGuardAcquire"),
        "Should track write guard acquisition"
    );

    // Check lock type is rwlock_write
    let guard_event = events
        .iter()
        .find(|e| format!("{:?}", e).contains("LockGuardAcquire"))
        .map(|e| format!("{:?}", e));
    assert!(
        guard_event.unwrap().contains("rwlock_write"),
        "Should be rwlock_write type"
    );
}

// ============================================================================
// Closure Tests
// ============================================================================

#[test]
#[serial]
fn test_closure_create_ref() {
    reset();

    #[trace_borrow]
    fn create_ref_closure() {
        let x = 10;
        let _closure = |a: i32| a + x;
    }

    create_ref_closure();
    let events = get_events();

    assert!(
        has_event_type(&events, "ClosureCreate"),
        "Should track closure creation"
    );

    // Check capture mode is ref
    let create_event = events
        .iter()
        .find(|e| format!("{:?}", e).contains("ClosureCreate"))
        .map(|e| format!("{:?}", e));
    assert!(
        create_event.unwrap().contains("\"ref\""),
        "Should be ref capture mode"
    );
}

#[test]
#[serial]
fn test_closure_create_move() {
    reset();

    #[trace_borrow]
    fn create_move_closure() {
        let data = vec![1, 2, 3];
        let _closure = move || data.len();
    }

    create_move_closure();
    let events = get_events();

    assert!(
        has_event_type(&events, "ClosureCreate"),
        "Should track closure creation"
    );

    // Check capture mode is move
    let create_event = events
        .iter()
        .find(|e| format!("{:?}", e).contains("ClosureCreate"))
        .map(|e| format!("{:?}", e));
    assert!(
        create_event.unwrap().contains("\"move\""),
        "Should be move capture mode"
    );
}

#[test]
#[serial]
fn test_closure_capture_tracking() {
    reset();

    #[trace_borrow]
    fn closure_with_captures() {
        let x = 10;
        let y = 20;
        let _closure = |a: i32| a + x + y;
    }

    closure_with_captures();
    let events = get_events();

    assert!(
        has_event_type(&events, "ClosureCapture"),
        "Should track closure captures"
    );
    assert!(
        count_event_type(&events, "ClosureCapture") >= 2,
        "Should capture at least 2 variables"
    );
}

#[test]
#[serial]
fn test_move_closure_capture_mode() {
    reset();

    #[trace_borrow]
    fn move_closure_captures() {
        let data = String::from("hello");
        let _closure = move || data.len();
    }

    move_closure_captures();
    let events = get_events();

    // Check capture mode is move for captured variable
    let capture_event = events
        .iter()
        .find(|e| format!("{:?}", e).contains("ClosureCapture"))
        .map(|e| format!("{:?}", e));
    assert!(capture_event.is_some(), "Should have capture event");
    assert!(
        capture_event.unwrap().contains("\"move\""),
        "Capture mode should be move"
    );
}

// ============================================================================
// Box Raw Pointer Tests
// ============================================================================

#[test]
#[serial]
fn test_box_new_tracking() {
    reset();

    #[trace_borrow]
    fn create_box() {
        let _boxed = Box::new(42);
    }

    create_box();
    let events = get_events();

    assert!(has_event_type(&events, "BoxNew"), "Should track Box::new");
}

#[test]
#[serial]
fn test_box_into_raw_tracking() {
    reset();

    #[trace_borrow]
    fn box_to_raw() {
        let boxed = Box::new(42);
        let _ptr = Box::into_raw(boxed);
    }

    box_to_raw();
    let events = get_events();

    assert!(has_event_type(&events, "BoxNew"), "Should track Box::new");
    assert!(
        has_event_type(&events, "BoxIntoRaw"),
        "Should track Box::into_raw"
    );
}

#[test]
#[serial]
fn test_box_from_raw_tracking() {
    reset();

    #[trace_borrow]
    fn box_from_raw() {
        let boxed = Box::new(42);
        let ptr = Box::into_raw(boxed);
        let _recovered = unsafe { Box::from_raw(ptr) };
    }

    box_from_raw();
    let events = get_events();

    assert!(has_event_type(&events, "BoxNew"), "Should track Box::new");
    assert!(
        has_event_type(&events, "BoxIntoRaw"),
        "Should track Box::into_raw"
    );
    assert!(
        has_event_type(&events, "BoxFromRaw"),
        "Should track Box::from_raw"
    );
}

#[test]
#[serial]
fn test_box_raw_roundtrip() {
    reset();

    #[trace_borrow]
    fn box_roundtrip() {
        let original = Box::new(String::from("test"));
        let ptr = Box::into_raw(original);
        let recovered = unsafe { Box::from_raw(ptr) };
        assert_eq!(*recovered, "test");
    }

    box_roundtrip();
    let events = get_events();

    // Should have all three events in order
    assert!(has_event_type(&events, "BoxNew"), "Should track Box::new");
    assert!(
        has_event_type(&events, "BoxIntoRaw"),
        "Should track Box::into_raw"
    );
    assert!(
        has_event_type(&events, "BoxFromRaw"),
        "Should track Box::from_raw"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[serial]
fn test_combined_medium_priority_features() {
    reset();

    #[trace_borrow]
    fn combined_test() {
        // Lock guard
        let mutex = Mutex::new(42);
        {
            let _guard = mutex.lock().unwrap();
        }

        // Closure
        let x = 10;
        let _closure = |a: i32| a + x;

        // Box raw
        let boxed = Box::new(100);
        let _ptr = Box::into_raw(boxed);
    }

    combined_test();
    let events = get_events();

    assert!(
        has_event_type(&events, "LockGuardAcquire"),
        "Should track lock guard"
    );
    assert!(
        has_event_type(&events, "ClosureCreate"),
        "Should track closure"
    );
    assert!(
        has_event_type(&events, "BoxIntoRaw"),
        "Should track Box::into_raw"
    );
}
