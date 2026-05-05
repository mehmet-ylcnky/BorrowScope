//! Comprehensive tests for Pin tracking
//!
//! Tests cover:
//! - Pin::new with references
//! - Box::pin for heap pinning
//! - Pin::into_inner extraction
//! - Pinned data access patterns

use borrowscope_runtime::*;
use std::pin::Pin;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Box::pin Tests (most common Pin usage)
// =============================================================================

#[test]
fn test_box_pin_primitive() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned = track_pin_new("pinned", "test:1", Box::pin(42));
    assert_eq!(*pinned, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_pin());
}

#[test]
fn test_box_pin_string() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned = track_pin_new("pinned", "test:1", Box::pin(String::from("pinned string")));
    assert_eq!(&*pinned, "pinned string");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_pin());
}

#[test]
fn test_box_pin_vec() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned = track_pin_new("pinned", "test:1", Box::pin(vec![1, 2, 3, 4, 5]));
    assert_eq!(pinned.len(), 5);
    assert_eq!(pinned[0], 1);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_box_pin_struct() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    #[derive(Debug)]
    struct Data {
        value: i32,
        name: String,
    }

    let pinned = track_pin_new(
        "pinned",
        "test:1",
        Box::pin(Data {
            value: 42,
            name: String::from("test"),
        }),
    );

    assert_eq!(pinned.value, 42);
    assert_eq!(pinned.name, "test");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Pin::new with References
// =============================================================================

#[test]
fn test_pin_new_with_box() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let boxed = Box::new(100);
    let pinned = track_pin_new("pinned", "test:1", Pin::new(boxed));
    assert_eq!(*pinned, 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_pin());
}

#[test]
fn test_pin_new_with_mut_ref() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mut value = 42;
    let pinned = track_pin_new("pinned", "test:1", Pin::new(&mut value));
    assert_eq!(*pinned, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Pin::into_inner Tests
// =============================================================================

#[test]
fn test_pin_into_inner_box() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned = Pin::new(Box::new(100));
    let boxed = track_pin_into_inner("pinned", "test:1", Pin::into_inner(pinned));
    assert_eq!(*boxed, 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_pin());
}

#[test]
fn test_pin_into_inner_string() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned = Pin::new(Box::new(String::from("extract me")));
    let boxed = track_pin_into_inner("pinned", "test:1", Pin::into_inner(pinned));
    assert_eq!(&*boxed, "extract me");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Pin Lifecycle Tests
// =============================================================================

#[test]
fn test_pin_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create pinned value
    let pinned = track_pin_new("data", "test:1", Box::pin(String::from("lifecycle")));
    assert_eq!(&*pinned, "lifecycle");

    // Extract value (consumes the Pin)
    let boxed = track_pin_into_inner("data", "test:2", Pin::into_inner(pinned));
    assert_eq!(&*boxed, "lifecycle");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_pin()));
}

#[test]
fn test_pin_multiple_values() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let p1 = track_pin_new("p1", "test:1", Box::pin(1));
    let p2 = track_pin_new("p2", "test:2", Box::pin(2));
    let p3 = track_pin_new("p3", "test:3", Box::pin(3));

    assert_eq!(*p1 + *p2 + *p3, 6);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

// =============================================================================
// Large Struct Tests
// =============================================================================

#[test]
fn test_box_pin_large_struct() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    #[derive(Debug)]
    struct LargeStruct {
        data: [u8; 1024],
        id: u64,
    }

    let pinned = track_pin_new(
        "large",
        "test:1",
        Box::pin(LargeStruct {
            data: [0u8; 1024],
            id: 12345,
        }),
    );
    assert_eq!(pinned.id, 12345);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Option and Result Tests
// =============================================================================

#[test]
fn test_pin_option() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned_some = track_pin_new("some", "test:1", Box::pin(Some(42)));
    let pinned_none: Pin<Box<Option<i32>>> = track_pin_new("none", "test:2", Box::pin(None));

    assert_eq!(*pinned_some, Some(42));
    assert_eq!(*pinned_none, None);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_pin_result() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pinned_ok: Pin<Box<Result<i32, &str>>> = track_pin_new("ok", "test:1", Box::pin(Ok(42)));
    let pinned_err: Pin<Box<Result<i32, &str>>> =
        track_pin_new("err", "test:2", Box::pin(Err("error")));

    assert_eq!(*pinned_ok, Ok(42));
    assert_eq!(*pinned_err, Err("error"));

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// =============================================================================
// Nested Pin Tests
// =============================================================================

#[test]
fn test_pin_nested_box() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let inner = Box::new(42);
    let outer = track_pin_new("outer", "test:1", Box::pin(inner));

    assert_eq!(**outer, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Collection Tests
// =============================================================================

#[test]
fn test_pin_in_vec() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let pins: Vec<Pin<Box<i32>>> = vec![
        track_pin_new("p0", "test:1", Box::pin(10)),
        track_pin_new("p1", "test:2", Box::pin(20)),
        track_pin_new("p2", "test:3", Box::pin(30)),
    ];

    let sum: i32 = pins.iter().map(|p| **p).sum();
    assert_eq!(sum, 60);

    let events = get_events();
    assert_eq!(events.len(), 3);
}
