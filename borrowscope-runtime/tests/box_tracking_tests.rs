//! Comprehensive tests for Box tracking
//!
//! Tests cover:
//! - Box::new creation
//! - Box::into_raw conversion
//! - Box::from_raw reconstruction
//! - Type information capture
//! - Ownership transfer patterns

use borrowscope_runtime::*;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Box Creation Tests
// =============================================================================

#[test]
fn test_box_new_primitive() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = track_box_new("boxed_int", "test:1", Box::new(42i32));
    assert_eq!(*boxed, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_new_string() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = track_box_new("boxed_str", "test:1", Box::new(String::from("hello")));
    assert_eq!(&*boxed, "hello");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_new_vector() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = track_box_new("boxed_vec", "test:1", Box::new(vec![1, 2, 3, 4, 5]));
    assert_eq!(boxed.len(), 5);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_new_struct() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    #[derive(Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }

    let boxed = track_box_new("point", "test:1", Box::new(Point { x: 1.0, y: 2.0 }));
    assert_eq!(boxed.x, 1.0);
    assert_eq!(boxed.y, 2.0);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_new_nested() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let inner = track_box_new("inner", "test:1", Box::new(42));
    let outer = track_box_new("outer", "test:2", Box::new(inner));

    assert_eq!(**outer, 42);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_box()));
}

// =============================================================================
// Box::into_raw Tests
// =============================================================================

#[test]
fn test_box_into_raw_basic() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = Box::new(100);
    let ptr = track_box_into_raw("boxed", "test:1", Box::into_raw(boxed));

    assert!(!ptr.is_null());

    // Clean up
    unsafe {
        let _ = Box::from_raw(ptr);
    }

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_into_raw_preserves_value() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = Box::new(String::from("preserved"));
    let ptr = track_box_into_raw("boxed", "test:1", Box::into_raw(boxed));

    // Verify value through raw pointer
    unsafe {
        assert_eq!(&*ptr, "preserved");
        let _ = Box::from_raw(ptr);
    }

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Box::from_raw Tests
// =============================================================================

#[test]
fn test_box_from_raw_basic() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let ptr = Box::into_raw(Box::new(42));
    let recovered = unsafe { track_box_from_raw("recovered", "test:1", Box::from_raw(ptr)) };

    assert_eq!(*recovered, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_box());
}

#[test]
fn test_box_from_raw_complex_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let original = vec![1, 2, 3, 4, 5];
    let ptr = Box::into_raw(Box::new(original));
    let recovered = unsafe { track_box_from_raw("recovered", "test:1", Box::from_raw(ptr)) };

    assert_eq!(*recovered, vec![1, 2, 3, 4, 5]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Full Lifecycle Tests
// =============================================================================

#[test]
fn test_box_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create
    let boxed = track_box_new("data", "test:1", Box::new(String::from("lifecycle")));

    // Convert to raw
    let ptr = track_box_into_raw("data", "test:2", Box::into_raw(boxed));

    // Recover
    let recovered = unsafe { track_box_from_raw("recovered", "test:3", Box::from_raw(ptr)) };

    assert_eq!(&*recovered, "lifecycle");

    let events = get_events();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|e| e.is_box()));
}

#[test]
fn test_box_multiple_into_from_raw() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = Box::new(1);
    let ptr1 = track_box_into_raw("b1", "test:1", Box::into_raw(boxed));
    let boxed2 = unsafe { track_box_from_raw("b2", "test:2", Box::from_raw(ptr1)) };

    let ptr2 = track_box_into_raw("b2", "test:3", Box::into_raw(boxed2));
    let boxed3 = unsafe { track_box_from_raw("b3", "test:4", Box::from_raw(ptr2)) };

    assert_eq!(*boxed3, 1);

    let events = get_events();
    assert_eq!(events.len(), 4);
}

#[test]
fn test_box_array_of_boxes() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let b1 = track_box_new("b1", "test:1", Box::new(1));
    let b2 = track_box_new("b2", "test:2", Box::new(2));
    let b3 = track_box_new("b3", "test:3", Box::new(3));

    let boxes = vec![b1, b2, b3];
    let sum: i32 = boxes.iter().map(|b| **b).sum();
    assert_eq!(sum, 6);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_box_zero_sized_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = track_box_new("unit", "test:1", Box::new(()));
    assert_eq!(*boxed, ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_box_large_allocation() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let large_vec: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB
    let boxed = track_box_new("large", "test:1", Box::new(large_vec));

    assert_eq!(boxed.len(), 1024 * 1024);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_box_trait_object() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    trait Animal {
        fn speak(&self) -> &str;
    }

    struct Dog;
    impl Animal for Dog {
        fn speak(&self) -> &str {
            "woof"
        }
    }

    let boxed: Box<dyn Animal> = track_box_new("animal", "test:1", Box::new(Dog));
    assert_eq!(boxed.speak(), "woof");

    let events = get_events();
    assert_eq!(events.len(), 1);
}
