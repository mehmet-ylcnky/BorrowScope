//! Comprehensive tests for Weak reference tracking (Rc and Arc)
//!
//! Tests cover:
//! - Weak::new / Rc::downgrade / Arc::downgrade
//! - Weak::clone
//! - Weak::upgrade (success and failure cases)
//! - Weak count tracking
//! - Lifecycle patterns

use borrowscope_runtime::*;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Rc Weak Reference Tests
// =============================================================================

#[test]
fn test_rc_downgrade_creates_weak() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc = Rc::new(42);
    let weak = track_weak_new("weak", "rc", "test:1", Rc::downgrade(&rc));

    assert_eq!(Rc::strong_count(&rc), 1);
    assert_eq!(RcWeak::weak_count(&weak), 1);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_weak());
}

#[test]
fn test_rc_multiple_weak_references() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc = Rc::new("shared data");
    let weak1 = track_weak_new("weak1", "rc", "test:1", Rc::downgrade(&rc));
    let weak2 = track_weak_new("weak2", "rc", "test:2", Rc::downgrade(&rc));
    let weak3 = track_weak_new("weak3", "rc", "test:3", Rc::downgrade(&rc));

    assert_eq!(RcWeak::weak_count(&weak1), 3);
    assert_eq!(RcWeak::weak_count(&weak2), 3);
    assert_eq!(RcWeak::weak_count(&weak3), 3);

    let events = get_events();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|e| e.is_weak()));
}

#[test]
fn test_rc_weak_clone() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc = Rc::new(100);
    let weak1 = Rc::downgrade(&rc);
    let weak2 = track_weak_clone("weak2", "weak1", "test:1", weak1.clone());

    assert_eq!(RcWeak::weak_count(&weak1), 2);
    assert_eq!(RcWeak::weak_count(&weak2), 2);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_weak());
}

#[test]
fn test_rc_weak_upgrade_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc = Rc::new(vec![1, 2, 3]);
    let weak = Rc::downgrade(&rc);

    let upgraded = track_weak_upgrade("weak", "test:1", weak.upgrade());
    assert!(upgraded.is_some());
    assert_eq!(*upgraded.unwrap(), vec![1, 2, 3]);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_weak());
}

#[test]
fn test_rc_weak_upgrade_after_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let weak = {
        let rc = Rc::new("temporary");
        Rc::downgrade(&rc)
        // rc dropped here
    };

    let upgraded = track_weak_upgrade("weak", "test:1", weak.upgrade());
    assert!(upgraded.is_none());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_rc_weak_empty() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let weak: RcWeak<i32> = RcWeak::new();
    let upgraded = track_weak_upgrade("empty_weak", "test:1", weak.upgrade());

    assert!(upgraded.is_none());
    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_rc_weak_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create Rc and downgrade
    let rc = Rc::new(String::from("lifecycle test"));
    let weak1 = track_weak_new("weak1", "rc", "test:1", Rc::downgrade(&rc));

    // Clone weak
    let weak2 = track_weak_clone("weak2", "weak1", "test:2", weak1.clone());

    // Upgrade both
    let up1 = track_weak_upgrade("weak1", "test:3", weak1.upgrade());
    let up2 = track_weak_upgrade("weak2", "test:4", weak2.upgrade());

    assert!(up1.is_some());
    assert!(up2.is_some());

    let events = get_events();
    assert_eq!(events.len(), 4);
}

// =============================================================================
// Arc Weak Reference Tests (Thread-safe)
// =============================================================================

#[test]
fn test_arc_downgrade_creates_weak() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arc = Arc::new(42);
    let weak = track_weak_new_sync("weak", "arc", "test:1", Arc::downgrade(&arc));

    assert_eq!(Arc::strong_count(&arc), 1);
    assert_eq!(ArcWeak::weak_count(&weak), 1);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_weak());
}

#[test]
fn test_arc_multiple_weak_references() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arc = Arc::new(vec![1, 2, 3, 4, 5]);
    let weak1 = track_weak_new_sync("weak1", "arc", "test:1", Arc::downgrade(&arc));
    let weak2 = track_weak_new_sync("weak2", "arc", "test:2", Arc::downgrade(&arc));

    assert_eq!(ArcWeak::weak_count(&weak1), 2);
    assert_eq!(ArcWeak::weak_count(&weak2), 2);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_arc_weak_clone_sync() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arc = Arc::new("thread-safe data");
    let weak1 = Arc::downgrade(&arc);
    let weak2 = track_weak_clone_sync("weak2", "weak1", "test:1", weak1.clone());

    assert_eq!(ArcWeak::weak_count(&weak1), 2);
    assert_eq!(ArcWeak::weak_count(&weak2), 2);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_arc_weak_upgrade_sync_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arc = Arc::new(999);
    let weak = Arc::downgrade(&arc);

    let upgraded = track_weak_upgrade_sync("weak", "test:1", weak.upgrade());
    assert!(upgraded.is_some());
    assert_eq!(*upgraded.unwrap(), 999);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_arc_weak_upgrade_sync_after_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let weak = {
        let arc = Arc::new("will be dropped");
        Arc::downgrade(&arc)
    };

    let upgraded = track_weak_upgrade_sync("weak", "test:1", weak.upgrade());
    assert!(upgraded.is_none());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_arc_weak_cross_thread() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arc = Arc::new(42);
    let weak = track_weak_new_sync("weak", "arc", "main:1", Arc::downgrade(&arc));

    let handle = std::thread::spawn(move || {
        let upgraded = weak.upgrade();
        upgraded.map(|v| *v)
    });

    let result = handle.join().unwrap();
    assert_eq!(result, Some(42));

    let events = get_events();
    assert!(events.len() >= 1);
}

// =============================================================================
// Mixed Scenarios
// =============================================================================

#[test]
fn test_weak_with_rc_clone() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc1 = Rc::new(100);
    let rc2 = Rc::clone(&rc1);
    let weak = track_weak_new("weak", "rc1", "test:1", Rc::downgrade(&rc1));

    assert_eq!(Rc::strong_count(&rc1), 2);
    assert_eq!(RcWeak::weak_count(&weak), 1);

    drop(rc1);
    // rc2 still alive, weak should upgrade
    let upgraded = track_weak_upgrade("weak", "test:2", weak.upgrade());
    assert!(upgraded.is_some());
    drop(upgraded); // Must drop the upgraded Rc before dropping rc2

    drop(rc2);
    // Now weak should fail (no strong refs left)
    let upgraded2 = track_weak_upgrade("weak", "test:3", weak.upgrade());
    assert!(upgraded2.is_none());

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_weak_chain() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let rc = Rc::new("chain");
    let w1 = track_weak_new("w1", "rc", "test:1", Rc::downgrade(&rc));
    let w2 = track_weak_clone("w2", "w1", "test:2", w1.clone());
    let w3 = track_weak_clone("w3", "w2", "test:3", w2.clone());
    let w4 = track_weak_clone("w4", "w3", "test:4", w3.clone());

    assert_eq!(RcWeak::weak_count(&w1), 4);

    let events = get_events();
    assert_eq!(events.len(), 4);
    assert!(events.iter().all(|e| e.is_weak()));
}
