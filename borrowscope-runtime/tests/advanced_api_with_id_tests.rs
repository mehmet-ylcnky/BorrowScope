//! Tests for advanced API with explicit IDs and locations

use borrowscope_runtime::*;
use std::rc::Rc;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_track_new_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_new_with_id(1, "x", "i32", "test.rs:10:5", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        Event::New {
            var_id, type_name, ..
        } => {
            assert_eq!(var_id, "x_1");
            assert!(type_name.contains("test.rs:10:5"));
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_track_borrow_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_new_with_id(1, "x", "i32", "test.rs:10:5", 42);
    let _r = track_borrow_with_id(2, 1, "r", "test.rs:11:5", false, &x);

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::Borrow {
            borrower_id,
            owner_id,
            mutable,
            ..
        } => {
            assert_eq!(borrower_id, "r_2");
            assert_eq!(owner_id, "owner_1");
            assert!(!mutable);
        }
        _ => panic!("Expected Borrow event"),
    }
}

#[test]
fn test_track_borrow_mut_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let mut x = track_new_with_id(1, "x", "Vec<i32>", "test.rs:10:5", vec![1, 2, 3]);
    let _r = track_borrow_mut_with_id(2, 1, "r", "test.rs:11:5", &mut x);

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::Borrow { mutable, .. } => {
            assert!(mutable);
        }
        _ => panic!("Expected Borrow event"),
    }
}

#[test]
fn test_track_move_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_new_with_id(1, "x", "String", "test.rs:10:5", String::from("hello"));
    let _y = track_move_with_id(1, 2, "y", "test.rs:11:5", x);

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::Move { from_id, to_id, .. } => {
            assert_eq!(from_id, "var_1");
            assert_eq!(to_id, "y_2");
        }
        _ => panic!("Expected Move event"),
    }
}

#[test]
fn test_track_drop_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    {
        let _x = track_new_with_id(1, "x", "i32", "test.rs:10:5", 42);
        track_drop_with_id(1, "test.rs:12:1");
    }

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::Drop { var_id, .. } => {
            assert!(var_id.contains("var_1"));
            assert!(var_id.contains("test.rs:12:1"));
        }
        _ => panic!("Expected Drop event"),
    }
}

#[test]
fn test_track_rc_new_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_rc_new_with_id(1, "x", "Rc<i32>", "test.rs:10:5", Rc::new(42));

    let events = get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        Event::RcNew {
            var_id,
            type_name,
            strong_count,
            ..
        } => {
            assert_eq!(var_id, "x_1");
            assert!(type_name.contains("Rc<i32>"));
            assert!(type_name.contains("test.rs:10:5"));
            assert_eq!(*strong_count, 1);
        }
        _ => panic!("Expected RcNew event"),
    }
}

#[test]
fn test_track_rc_clone_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new_with_id(1, "x", "Rc<i32>", "test.rs:10:5", Rc::new(42));
    let _y = track_rc_clone_with_id(2, 1, "y", "test.rs:11:5", Rc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::RcClone {
            var_id,
            source_id,
            strong_count,
            ..
        } => {
            assert_eq!(var_id, "y_2");
            assert_eq!(source_id, "var_1");
            assert_eq!(*strong_count, 2);
        }
        _ => panic!("Expected RcClone event"),
    }
}

#[test]
fn test_track_arc_new_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_arc_new_with_id(1, "x", "Arc<i32>", "test.rs:10:5", Arc::new(100));

    let events = get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        Event::ArcNew {
            var_id,
            type_name,
            strong_count,
            ..
        } => {
            assert_eq!(var_id, "x_1");
            assert!(type_name.contains("Arc<i32>"));
            assert!(type_name.contains("test.rs:10:5"));
            assert_eq!(*strong_count, 1);
        }
        _ => panic!("Expected ArcNew event"),
    }
}

#[test]
fn test_track_arc_clone_with_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new_with_id(1, "x", "Arc<i32>", "test.rs:10:5", Arc::new(100));
    let _y = track_arc_clone_with_id(2, 1, "y", "test.rs:11:5", Arc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::ArcClone {
            var_id,
            source_id,
            strong_count,
            ..
        } => {
            assert_eq!(var_id, "y_2");
            assert_eq!(source_id, "var_1");
            assert_eq!(*strong_count, 2);
        }
        _ => panic!("Expected ArcClone event"),
    }
}

#[test]
fn test_complex_id_based_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    // ID 1: x = Rc::new(vec![1,2,3])
    let x = track_rc_new_with_id(
        1,
        "x",
        "Rc<Vec<i32>>",
        "test.rs:10:5",
        Rc::new(vec![1, 2, 3]),
    );

    // ID 2: y = Rc::clone(&x)
    let _y = track_rc_clone_with_id(2, 1, "y", "test.rs:11:5", Rc::clone(&x));

    // ID 3: z = Rc::clone(&x)
    let _z = track_rc_clone_with_id(3, 1, "z", "test.rs:12:5", Rc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify ID-based relationships
    match &events[1] {
        Event::RcClone { source_id, .. } => assert_eq!(source_id, "var_1"),
        _ => panic!("Expected RcClone"),
    }

    match &events[2] {
        Event::RcClone { source_id, .. } => assert_eq!(source_id, "var_1"),
        _ => panic!("Expected RcClone"),
    }

    // Verify strong counts
    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));
}

#[test]
fn test_borrow_chain_with_ids() {
    let _lock = TEST_LOCK.lock();
    reset();

    // ID 1: x = String::from("hello")
    let x = track_new_with_id(1, "x", "String", "test.rs:10:5", String::from("hello"));

    // ID 2: r1 = &x (borrows from ID 1)
    let r1 = track_borrow_with_id(2, 1, "r1", "test.rs:11:5", false, &x);

    // ID 3: r2 = &r1 (borrows from ID 2)
    let _r2 = track_borrow_with_id(3, 2, "r2", "test.rs:12:5", false, &r1);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify borrow chain through IDs
    match &events[1] {
        Event::Borrow { owner_id, .. } => assert_eq!(owner_id, "owner_1"),
        _ => panic!("Expected Borrow"),
    }

    match &events[2] {
        Event::Borrow { owner_id, .. } => assert_eq!(owner_id, "owner_2"),
        _ => panic!("Expected Borrow"),
    }
}

#[test]
fn test_location_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_new_with_id(1, "x", "i32", "src/main.rs:42:9", 42);
    let _y = track_new_with_id(2, "y", "i32", "src/main.rs:43:9", 100);

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Verify locations are captured
    match &events[0] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("src/main.rs:42:9"));
        }
        _ => panic!("Expected New event"),
    }

    match &events[1] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("src/main.rs:43:9"));
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_unique_id_generation() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Simulate macro generating unique IDs
    let _x = track_new_with_id(1, "x", "i32", "test.rs:10:5", 1);
    let _y = track_new_with_id(2, "y", "i32", "test.rs:11:5", 2);
    let _z = track_new_with_id(3, "z", "i32", "test.rs:12:5", 3);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify unique IDs
    match &events[0] {
        Event::New { var_id, .. } => assert_eq!(var_id, "x_1"),
        _ => panic!("Expected New"),
    }

    match &events[1] {
        Event::New { var_id, .. } => assert_eq!(var_id, "y_2"),
        _ => panic!("Expected New"),
    }

    match &events[2] {
        Event::New { var_id, .. } => assert_eq!(var_id, "z_3"),
        _ => panic!("Expected New"),
    }
}
