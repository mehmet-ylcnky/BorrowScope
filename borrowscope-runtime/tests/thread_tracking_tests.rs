//! Comprehensive tests for thread spawn/join tracking
//!
//! Tests cover:
//! - thread::spawn tracking
//! - JoinHandle::join tracking
//! - Thread lifecycle patterns
//! - Data passing between threads

use borrowscope_runtime::*;
use std::thread;
use std::time::Duration;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Basic Thread Spawn Tests
// =============================================================================

#[test]
fn test_thread_spawn_simple() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = track_thread_spawn("worker", "test:1", thread::spawn(|| 42));
    let result = handle.join().unwrap();

    assert_eq!(result, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_thread());
}

#[test]
fn test_thread_spawn_with_closure() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let value = 10;
    let handle = track_thread_spawn("compute", "test:1", thread::spawn(move || value * 2));

    let result = handle.join().unwrap();
    assert_eq!(result, 20);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_thread_spawn_returning_string() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = track_thread_spawn(
        "string_worker",
        "test:1",
        thread::spawn(|| String::from("hello from thread")),
    );

    let result = handle.join().unwrap();
    assert_eq!(result, "hello from thread");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Thread Join Tests
// =============================================================================

#[test]
fn test_thread_join_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = thread::spawn(|| 100);
    let result = track_thread_join("worker", "test:1", handle.join());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_thread());
}

#[test]
fn test_thread_join_with_panic() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = thread::spawn(|| {
        panic!("intentional panic");
    });

    let result = track_thread_join("panicking", "test:1", handle.join());
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Full Lifecycle Tests
// =============================================================================

#[test]
fn test_thread_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = track_thread_spawn(
        "lifecycle",
        "test:1",
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(10));
            "completed"
        }),
    );

    let result = track_thread_join("lifecycle", "test:2", handle.join());
    assert_eq!(result.unwrap(), "completed");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_thread()));
}

#[test]
fn test_multiple_threads() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let h1 = track_thread_spawn("t1", "test:1", thread::spawn(|| 1));
    let h2 = track_thread_spawn("t2", "test:2", thread::spawn(|| 2));
    let h3 = track_thread_spawn("t3", "test:3", thread::spawn(|| 3));

    let r1 = track_thread_join("t1", "test:4", h1.join());
    let r2 = track_thread_join("t2", "test:5", h2.join());
    let r3 = track_thread_join("t3", "test:6", h3.join());

    assert_eq!(r1.unwrap() + r2.unwrap() + r3.unwrap(), 6);

    let events = get_events();
    assert_eq!(events.len(), 6);
}

// =============================================================================
// Data Transfer Tests
// =============================================================================

#[test]
fn test_thread_move_data() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let data = vec![1, 2, 3, 4, 5];
    let handle = track_thread_spawn(
        "processor",
        "test:1",
        thread::spawn(move || data.iter().sum::<i32>()),
    );

    let result = track_thread_join("processor", "test:2", handle.join());
    assert_eq!(result.unwrap(), 15);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_thread_return_complex_data() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    #[derive(Debug, PartialEq)]
    struct Result {
        sum: i32,
        count: usize,
    }

    let handle = track_thread_spawn(
        "aggregator",
        "test:1",
        thread::spawn(|| {
            let data = vec![1, 2, 3, 4, 5];
            Result {
                sum: data.iter().sum(),
                count: data.len(),
            }
        }),
    );

    let result = track_thread_join("aggregator", "test:2", handle.join()).unwrap();
    assert_eq!(result.sum, 15);
    assert_eq!(result.count, 5);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// =============================================================================
// Concurrent Patterns
// =============================================================================

#[test]
fn test_thread_parallel_computation() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handles: Vec<_> = (0..4)
        .map(|i| {
            track_thread_spawn(
                &format!("worker_{}", i),
                &format!("test:{}", i),
                thread::spawn(move || i * 10),
            )
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            track_thread_join(&format!("worker_{}", i), &format!("join:{}", i), h.join()).unwrap()
        })
        .collect();

    assert_eq!(results, vec![0, 10, 20, 30]);

    let events = get_events();
    assert_eq!(events.len(), 8); // 4 spawns + 4 joins
}

#[test]
fn test_thread_with_sleep() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = track_thread_spawn(
        "sleeper",
        "test:1",
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(50));
            "woke up"
        }),
    );

    let result = track_thread_join("sleeper", "test:2", handle.join());
    assert_eq!(result.unwrap(), "woke up");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_thread_empty_closure() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let handle = track_thread_spawn("empty", "test:1", thread::spawn(|| {}));
    let result = track_thread_join("empty", "test:2", handle.join());

    assert!(result.is_ok());

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_thread_nested_spawn() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let outer = track_thread_spawn(
        "outer",
        "test:1",
        thread::spawn(|| {
            let inner = thread::spawn(|| 42);
            inner.join().unwrap()
        }),
    );

    let result = track_thread_join("outer", "test:2", outer.join());
    assert_eq!(result.unwrap(), 42);

    let events = get_events();
    assert_eq!(events.len(), 2);
}
