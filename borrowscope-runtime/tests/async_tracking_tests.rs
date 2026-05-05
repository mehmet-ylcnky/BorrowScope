//! Integration tests for async tracking
//!
//! These tests verify that async operations are properly tracked including
//! async blocks and await expressions.

use borrowscope_runtime::*;
use serial_test::serial;

// ============================================================================
// ASYNC BLOCK TESTS
// ============================================================================

#[test]
#[serial]
fn test_async_block_enter_exit() {
    reset();

    track_async_block_enter(1, "test.rs:10:5");
    track_async_block_exit(1, "test.rs:12:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        Event::AsyncBlockEnter {
            block_id, location, ..
        } => {
            assert_eq!(block_id, "1");
            assert_eq!(location, "test.rs:10:5");
        }
        _ => panic!("Expected AsyncBlockEnter event"),
    }

    match &events[1] {
        Event::AsyncBlockExit {
            block_id, location, ..
        } => {
            assert_eq!(block_id, "1");
            assert_eq!(location, "test.rs:12:5");
        }
        _ => panic!("Expected AsyncBlockExit event"),
    }
}

#[test]
#[serial]
fn test_nested_async_blocks() {
    reset();

    track_async_block_enter(1, "outer:1");
    track_async_block_enter(2, "inner:1");
    track_async_block_exit(2, "inner:2");
    track_async_block_exit(1, "outer:2");

    let events = get_events();
    assert_eq!(events.len(), 4);

    // Verify nesting order
    match &events[0] {
        Event::AsyncBlockEnter { block_id, .. } => assert_eq!(block_id, "1"),
        _ => panic!("Expected outer enter"),
    }
    match &events[1] {
        Event::AsyncBlockEnter { block_id, .. } => assert_eq!(block_id, "2"),
        _ => panic!("Expected inner enter"),
    }
    match &events[2] {
        Event::AsyncBlockExit { block_id, .. } => assert_eq!(block_id, "2"),
        _ => panic!("Expected inner exit"),
    }
    match &events[3] {
        Event::AsyncBlockExit { block_id, .. } => assert_eq!(block_id, "1"),
        _ => panic!("Expected outer exit"),
    }
}

// ============================================================================
// AWAIT TESTS
// ============================================================================

#[test]
#[serial]
fn test_await_start_end() {
    reset();

    track_await_start(1, "my_future", "test.rs:20:5");
    track_await_end(1, "test.rs:20:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        Event::AwaitStart {
            await_id,
            future_name,
            location,
            ..
        } => {
            assert_eq!(await_id, "1");
            assert_eq!(future_name, "my_future");
            assert_eq!(location, "test.rs:20:5");
        }
        _ => panic!("Expected AwaitStart event"),
    }

    match &events[1] {
        Event::AwaitEnd {
            await_id, location, ..
        } => {
            assert_eq!(await_id, "1");
            assert_eq!(location, "test.rs:20:5");
        }
        _ => panic!("Expected AwaitEnd event"),
    }
}

#[test]
#[serial]
fn test_multiple_awaits() {
    reset();

    track_await_start(1, "future_a", "line:1");
    track_await_end(1, "line:1");
    track_await_start(2, "future_b", "line:2");
    track_await_end(2, "line:2");
    track_await_start(3, "future_c", "line:3");
    track_await_end(3, "line:3");

    let events = get_events();
    assert_eq!(events.len(), 6);

    // Verify sequential await tracking
    let await_starts: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::AwaitStart { future_name, .. } => Some(future_name.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(await_starts, vec!["future_a", "future_b", "future_c"]);
}

// ============================================================================
// COMBINED ASYNC TRACKING TESTS
// ============================================================================

#[test]
#[serial]
fn test_async_block_with_await() {
    reset();

    // Simulate: async { some_future.await }
    track_async_block_enter(1, "async_block:1");
    track_await_start(1, "some_future", "await:1");
    track_await_end(1, "await:1");
    track_async_block_exit(1, "async_block:2");

    let events = get_events();
    assert_eq!(events.len(), 4);

    assert!(matches!(events[0], Event::AsyncBlockEnter { .. }));
    assert!(matches!(events[1], Event::AwaitStart { .. }));
    assert!(matches!(events[2], Event::AwaitEnd { .. }));
    assert!(matches!(events[3], Event::AsyncBlockExit { .. }));
}

#[test]
#[serial]
fn test_async_with_ownership_tracking() {
    reset();

    // Simulate async function with variable tracking
    track_async_block_enter(1, "fn:1");

    let _data = track_new("data", vec![1, 2, 3]);
    track_await_start(1, "fetch_data", "await:1");
    track_await_end(1, "await:1");

    let _ref = track_borrow("ref", &[1, 2, 3]);
    track_drop("ref");
    track_drop("data");

    track_async_block_exit(1, "fn:2");

    let events = get_events();

    // Should have: async_enter, new, await_start, await_end, borrow, drop, drop, async_exit
    assert_eq!(events.len(), 8);
    assert!(matches!(events[0], Event::AsyncBlockEnter { .. }));
    assert!(matches!(events[1], Event::New { .. }));
    assert!(matches!(events[2], Event::AwaitStart { .. }));
    assert!(matches!(events[3], Event::AwaitEnd { .. }));
    assert!(matches!(events[4], Event::Borrow { .. }));
    assert!(matches!(events[5], Event::Drop { .. }));
    assert!(matches!(events[6], Event::Drop { .. }));
    assert!(matches!(events[7], Event::AsyncBlockExit { .. }));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
#[serial]
fn test_async_events_serialize() {
    reset();

    track_async_block_enter(42, "test.rs:1:1");
    track_await_start(99, "my_async_fn", "test.rs:2:5");
    track_await_end(99, "test.rs:2:5");
    track_async_block_exit(42, "test.rs:3:1");

    let events = get_events();
    let json = serde_json::to_string(&events).expect("Should serialize");

    assert!(json.contains("AsyncBlockEnter"));
    assert!(json.contains("AsyncBlockExit"));
    assert!(json.contains("AwaitStart"));
    assert!(json.contains("AwaitEnd"));
    assert!(json.contains("my_async_fn"));
}
