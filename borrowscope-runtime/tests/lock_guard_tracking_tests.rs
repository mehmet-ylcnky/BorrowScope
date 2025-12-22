//! Comprehensive tests for lock guard tracking
//!
//! Tests cover:
//! - Lock guard acquisition
//! - Lock guard drop
//! - Mutex patterns
//! - RwLock patterns
//! - Guard lifecycle

use borrowscope_runtime::*;
use std::sync::{Mutex, RwLock};

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Lock Guard Acquire Tests
// =============================================================================

#[test]
fn test_lock_guard_acquire_mutex() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_acquire("guard", "data_mutex", "Mutex", "test:1");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_lock_guard());
}

#[test]
fn test_lock_guard_acquire_rwlock_read() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_acquire("read_guard", "data_rwlock", "RwLock::read", "test:1");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_lock_guard());
}

#[test]
fn test_lock_guard_acquire_rwlock_write() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_acquire("write_guard", "data_rwlock", "RwLock::write", "test:1");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_lock_guard());
}

// =============================================================================
// Lock Guard Drop Tests
// =============================================================================

#[test]
fn test_lock_guard_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_drop("guard", "test:1");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_lock_guard());
}

// =============================================================================
// Full Lifecycle Tests
// =============================================================================

#[test]
fn test_lock_guard_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_acquire("guard", "mutex", "Mutex", "test:1");
    // ... use the guard ...
    track_lock_guard_drop("guard", "test:2");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_lock_guard()));
}

#[test]
fn test_multiple_guards_sequential() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // First guard
    track_lock_guard_acquire("g1", "mutex", "Mutex", "test:1");
    track_lock_guard_drop("g1", "test:2");

    // Second guard
    track_lock_guard_acquire("g2", "mutex", "Mutex", "test:3");
    track_lock_guard_drop("g2", "test:4");

    let events = get_events();
    assert_eq!(events.len(), 4);
}

#[test]
fn test_multiple_guards_nested() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Outer guard
    track_lock_guard_acquire("outer", "mutex1", "Mutex", "test:1");

    // Inner guard (different mutex)
    track_lock_guard_acquire("inner", "mutex2", "Mutex", "test:2");
    track_lock_guard_drop("inner", "test:3");

    track_lock_guard_drop("outer", "test:4");

    let events = get_events();
    assert_eq!(events.len(), 4);
}

// =============================================================================
// Real Mutex Usage Tests
// =============================================================================

#[test]
fn test_with_real_mutex() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mutex = Mutex::new(42);

    track_lock_guard_acquire("guard", "mutex", "Mutex", "test:1");
    {
        let guard = mutex.lock().unwrap();
        assert_eq!(*guard, 42);
    }
    track_lock_guard_drop("guard", "test:2");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_with_real_mutex_modify() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mutex = Mutex::new(0);

    track_lock_guard_acquire("guard", "counter", "Mutex", "test:1");
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    }
    track_lock_guard_drop("guard", "test:2");

    assert_eq!(*mutex.lock().unwrap(), 1);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// =============================================================================
// Real RwLock Usage Tests
// =============================================================================

#[test]
fn test_with_real_rwlock_read() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rwlock = RwLock::new(vec![1, 2, 3]);

    track_lock_guard_acquire("read_guard", "data", "RwLock::read", "test:1");
    {
        let guard = rwlock.read().unwrap();
        assert_eq!(guard.len(), 3);
    }
    track_lock_guard_drop("read_guard", "test:2");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_with_real_rwlock_write() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rwlock = RwLock::new(String::from("hello"));

    track_lock_guard_acquire("write_guard", "data", "RwLock::write", "test:1");
    {
        let mut guard = rwlock.write().unwrap();
        guard.push_str(" world");
    }
    track_lock_guard_drop("write_guard", "test:2");

    assert_eq!(*rwlock.read().unwrap(), "hello world");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_rwlock_multiple_readers() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rwlock = RwLock::new(100);

    // Multiple read guards can coexist
    track_lock_guard_acquire("r1", "data", "RwLock::read", "test:1");
    track_lock_guard_acquire("r2", "data", "RwLock::read", "test:2");
    track_lock_guard_acquire("r3", "data", "RwLock::read", "test:3");

    {
        let g1 = rwlock.read().unwrap();
        let g2 = rwlock.read().unwrap();
        let g3 = rwlock.read().unwrap();
        assert_eq!(*g1 + *g2 + *g3, 300);
    }

    track_lock_guard_drop("r1", "test:4");
    track_lock_guard_drop("r2", "test:5");
    track_lock_guard_drop("r3", "test:6");

    let events = get_events();
    assert_eq!(events.len(), 6);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_guard_with_try_lock() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mutex = Mutex::new(42);

    if let Ok(guard) = mutex.try_lock() {
        track_lock_guard_acquire("guard", "mutex", "Mutex::try_lock", "test:1");
        assert_eq!(*guard, 42);
        drop(guard);
        track_lock_guard_drop("guard", "test:2");
    }

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_guard_different_lock_types() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    track_lock_guard_acquire("mutex_guard", "m1", "Mutex", "test:1");
    track_lock_guard_acquire("rwlock_read", "rw1", "RwLock::read", "test:2");
    track_lock_guard_acquire("rwlock_write", "rw2", "RwLock::write", "test:3");

    track_lock_guard_drop("rwlock_write", "test:4");
    track_lock_guard_drop("rwlock_read", "test:5");
    track_lock_guard_drop("mutex_guard", "test:6");

    let events = get_events();
    assert_eq!(events.len(), 6);
}

#[test]
fn test_guard_reacquire_same_name() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    for i in 0..3 {
        track_lock_guard_acquire("guard", "mutex", "Mutex", &format!("acquire:{}", i));
        track_lock_guard_drop("guard", &format!("drop:{}", i));
    }

    let events = get_events();
    assert_eq!(events.len(), 6);
}
