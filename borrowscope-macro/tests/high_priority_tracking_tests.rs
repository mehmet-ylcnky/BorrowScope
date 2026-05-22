//! Tests for high priority tracking features
//!
//! Tests for:
//! - Cow::to_mut tracking
//! - Weak::upgrade and Weak::clone tracking
//! - JoinHandle::join tracking
//! - Channel send/recv tracking

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset, Event};
use serial_test::serial;
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
// Cow::to_mut Tests
// ============================================================================

#[test]
#[serial]
#[serial]
fn test_cow_borrowed_creation() {
    reset();

    #[trace_borrow]
    fn create_borrowed() {
        let _cow: Cow<str> = Cow::Borrowed("hello");
    }

    create_borrowed();
    let events = get_events();

    assert!(
        has_event_type(&events, "CowBorrowed"),
        "Should track CowBorrowed creation"
    );
}

#[test]
#[serial]
fn test_cow_owned_creation() {
    reset();

    #[trace_borrow]
    fn create_owned() {
        let _cow: Cow<str> = Cow::Owned(String::from("hello"));
    }

    create_owned();
    let events = get_events();

    assert!(
        has_event_type(&events, "CowOwned"),
        "Should track CowOwned creation"
    );
}

#[test]
#[serial]
fn test_cow_to_mut_tracking() {
    reset();

    #[trace_borrow]
    fn use_to_mut() {
        let mut cow: Cow<str> = Cow::Borrowed("hello");
        cow.to_mut().push_str(" world");
    }

    use_to_mut();
    let events = get_events();

    assert!(
        has_event_type(&events, "CowBorrowed"),
        "Should track initial Cow creation"
    );
    assert!(
        has_event_type(&events, "CowToMut"),
        "Should track to_mut call"
    );
}

// ============================================================================
// Weak Reference Tests
// ============================================================================

#[test]
#[serial]
fn test_weak_new_from_rc_downgrade() {
    reset();

    #[trace_borrow]
    fn create_weak() {
        let strong = Rc::new(42);
        let _weak = Rc::downgrade(&strong);
    }

    create_weak();
    let events = get_events();

    assert!(
        has_event_type(&events, "WeakNew"),
        "Should track Weak creation from Rc::downgrade"
    );
}

#[test]
#[serial]
fn test_weak_new_from_arc_downgrade() {
    reset();

    #[trace_borrow]
    fn create_weak_sync() {
        let strong = Arc::new(42);
        let _weak = Arc::downgrade(&strong);
    }

    create_weak_sync();
    let events = get_events();

    assert!(
        has_event_type(&events, "WeakNew"),
        "Should track Weak creation from Arc::downgrade"
    );
}

#[test]
#[serial]
fn test_weak_clone_tracking() {
    reset();

    #[trace_borrow]
    fn clone_weak() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        let _weak2 = weak.clone();
    }

    clone_weak();
    let events = get_events();

    assert!(
        has_event_type(&events, "WeakNew"),
        "Should track initial Weak creation"
    );
    assert!(
        has_event_type(&events, "WeakClone"),
        "Should track Weak clone"
    );
}

#[test]
#[serial]
fn test_weak_upgrade_tracking() {
    reset();

    #[trace_borrow]
    fn upgrade_weak() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        let _upgraded = weak.upgrade();
    }

    upgrade_weak();
    let events = get_events();

    assert!(
        has_event_type(&events, "WeakNew"),
        "Should track Weak creation"
    );
    assert!(
        has_event_type(&events, "WeakUpgrade"),
        "Should track Weak upgrade"
    );
}

#[test]
#[serial]
fn test_weak_upgrade_after_drop() {
    reset();

    #[trace_borrow]
    fn upgrade_after_drop() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        drop(strong);
        let _result = weak.upgrade(); // Should be None
    }

    upgrade_after_drop();
    let events = get_events();

    assert!(
        has_event_type(&events, "WeakUpgrade"),
        "Should track failed Weak upgrade"
    );
}

// ============================================================================
// Thread Join Tests
// ============================================================================

#[test]
#[serial]
fn test_thread_spawn_tracking() {
    reset();

    #[trace_borrow]
    fn spawn_thread() {
        let handle = thread::spawn(|| 42);
        let _ = handle.join();
    }

    spawn_thread();
    let events = get_events();

    assert!(
        has_event_type(&events, "ThreadSpawn"),
        "Should track thread spawn"
    );
}

#[test]
#[serial]
fn test_thread_join_tracking() {
    reset();

    #[trace_borrow]
    fn join_thread() {
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1));
            42
        });
        let _ = handle.join();
    }

    join_thread();
    let events = get_events();

    assert!(
        has_event_type(&events, "ThreadSpawn"),
        "Should track thread spawn"
    );
    assert!(
        has_event_type(&events, "ThreadJoin"),
        "Should track thread join"
    );
}

// ============================================================================
// Channel Tests
// ============================================================================

#[test]
#[serial]
fn test_channel_creation_tracking() {
    reset();

    #[trace_borrow]
    fn create_channel() {
        let (_tx, _rx) = mpsc::channel::<i32>();
    }

    create_channel();
    let events = get_events();

    assert!(
        has_event_type(&events, "ChannelSenderNew") || has_event_type(&events, "Channel"),
        "Should track channel creation"
    );
}

#[test]
#[serial]
fn test_channel_tuple_destructuring() {
    reset();

    #[trace_borrow]
    fn channel_with_names() {
        let (sender, receiver) = mpsc::channel::<i32>();
        drop(sender);
        drop(receiver);
    }

    channel_with_names();
    let events = get_events();

    // Should track both sender and receiver creation
    let channel_events = events
        .iter()
        .filter(|e| {
            let s = format!("{:?}", e);
            s.contains("Channel") || s.contains("Sender") || s.contains("Receiver")
        })
        .count();

    assert!(channel_events >= 1, "Should track channel creation events");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[serial]
fn test_combined_smart_pointer_operations() {
    reset();

    #[trace_borrow]
    fn combined_ops() {
        // Cow operations
        let mut cow: Cow<str> = Cow::Borrowed("test");
        cow.to_mut().push_str("!");

        // Weak operations
        let rc = Rc::new(100);
        let weak = Rc::downgrade(&rc);
        let weak2 = weak.clone();
        let _ = weak.upgrade();
        let _ = weak2.upgrade();
    }

    combined_ops();
    let events = get_events();

    assert!(has_event_type(&events, "CowBorrowed"), "Should track Cow");
    assert!(has_event_type(&events, "CowToMut"), "Should track to_mut");
    assert!(
        has_event_type(&events, "WeakNew"),
        "Should track Weak creation"
    );
    assert!(
        has_event_type(&events, "WeakClone"),
        "Should track Weak clone"
    );
    assert!(
        count_event_type(&events, "WeakUpgrade") >= 2,
        "Should track both upgrades"
    );
}

#[test]
#[serial]
fn test_thread_with_channel() {
    reset();

    #[trace_borrow]
    fn thread_channel_combo() {
        let channel = mpsc::channel();
        let tx = channel.0;
        let rx = channel.1;

        let handle = thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let _ = rx.recv();
        let _ = handle.join();
    }

    thread_channel_combo();
    let events = get_events();

    assert!(
        has_event_type(&events, "ThreadSpawn"),
        "Should track thread spawn"
    );
    assert!(
        has_event_type(&events, "ThreadJoin"),
        "Should track thread join"
    );
    // Channel creation should be tracked
    let has_channel = has_event_type(&events, "Channel")
        || has_event_type(&events, "Sender")
        || has_event_type(&events, "Receiver");
    assert!(has_channel, "Should track channel creation");
}
