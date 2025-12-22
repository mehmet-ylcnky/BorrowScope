//! Comprehensive tests for OnceCell and OnceLock tracking

use borrowscope_runtime::*;
use std::cell::OnceCell;
use std::sync::OnceLock;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// OnceCell Tests
// =============================================================================

#[test]
fn test_once_cell_new() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<i32> = track_once_cell_new("cell", "test:1", OnceCell::new());
    assert!(cell.get().is_none());

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_once_cell());
}

#[test]
fn test_once_cell_set_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<i32> = OnceCell::new();
    let result = track_once_cell_set("cell", "test:1", cell.set(42));

    assert!(result.is_ok());

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_once_cell());
}

#[test]
fn test_once_cell_set_failure() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<i32> = OnceCell::new();
    cell.set(42).unwrap();

    let result = track_once_cell_set("cell", "test:1", cell.set(100));
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_once_cell_get_initialized() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<i32> = OnceCell::new();
    cell.set(42).unwrap();

    let value = track_once_cell_get("cell", "test:1", cell.get());
    assert_eq!(value, Some(&42));

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_once_cell());
}

#[test]
fn test_once_cell_get_uninitialized() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<i32> = OnceCell::new();
    let value = track_once_cell_get("cell", "test:1", cell.get());

    assert!(value.is_none());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_once_cell_get_or_init_first_time() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<String> = OnceCell::new();
    let was_init = cell.get().is_some();

    let value = cell.get_or_init(|| String::from("initialized"));
    let _ = track_once_cell_get_or_init("cell", was_init, "test:1", value);

    assert_eq!(value, "initialized");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_once_cell_get_or_init_already_initialized() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cell: OnceCell<String> = OnceCell::new();
    cell.set(String::from("first")).unwrap();

    let was_init = cell.get().is_some();
    let value = cell.get_or_init(|| String::from("second"));
    let _ = track_once_cell_get_or_init("cell", was_init, "test:1", value);

    assert_eq!(value, "first");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// OnceLock Tests (Thread-safe)
// =============================================================================

#[test]
fn test_once_lock_new() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let lock: OnceLock<i32> = track_once_lock_new("lock", "test:1", OnceLock::new());
    assert!(lock.get().is_none());

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_once_cell());
}

#[test]
fn test_once_lock_set_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let lock: OnceLock<i32> = OnceLock::new();
    let result = track_once_cell_set("lock", "test:1", lock.set(42));

    assert!(result.is_ok());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_once_lock_get_initialized() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let lock: OnceLock<i32> = OnceLock::new();
    lock.set(42).unwrap();

    let value = track_once_cell_get("lock", "test:1", lock.get());
    assert_eq!(value, Some(&42));

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Lifecycle Tests
// =============================================================================

#[test]
fn test_once_cell_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create
    let cell: OnceCell<String> = track_once_cell_new("data", "test:1", OnceCell::new());

    // Try to get (uninitialized)
    let _ = track_once_cell_get("data", "test:2", cell.get());

    // Set
    let _ = track_once_cell_set("data", "test:3", cell.set(String::from("value")));

    // Get (initialized)
    let _ = track_once_cell_get("data", "test:4", cell.get());

    let events = get_events();
    assert_eq!(events.len(), 4);
    assert!(events.iter().all(|e| e.is_once_cell()));
}

#[test]
fn test_once_lock_cross_thread() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    use std::sync::Arc;
    use std::thread;

    let lock: Arc<OnceLock<i32>> = Arc::new(OnceLock::new());
    let lock2 = Arc::clone(&lock);

    let handle = thread::spawn(move || {
        lock2.set(42).ok();
    });

    handle.join().unwrap();

    let value = track_once_cell_get("lock", "test:1", lock.get());
    assert_eq!(value, Some(&42));

    let events = get_events();
    assert!(events.len() >= 1);
}
