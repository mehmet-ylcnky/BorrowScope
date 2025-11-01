use borrowscope_runtime::*;
use std::rc::Rc;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

// ============================================================================
// Rc Tests
// ============================================================================

#[test]
fn test_rc_new_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_rc_new("x", Rc::new(42));

    let events = get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        Event::RcNew {
            var_name,
            strong_count,
            weak_count,
            ..
        } => {
            assert_eq!(var_name, "x");
            assert_eq!(*strong_count, 1);
            assert_eq!(*weak_count, 0);
        }
        _ => panic!("Expected RcNew event, got {:?}", events[0]),
    }
}

#[test]
fn test_rc_clone_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(42));
    let _y = track_rc_clone("y", "x", Rc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        Event::RcNew { strong_count, .. } => {
            assert_eq!(*strong_count, 1);
        }
        _ => panic!("Expected RcNew event"),
    }

    match &events[1] {
        Event::RcClone {
            var_name,
            source_id,
            strong_count,
            ..
        } => {
            assert_eq!(var_name, "y");
            assert_eq!(source_id, "x");
            assert_eq!(*strong_count, 2);
        }
        _ => panic!("Expected RcClone event"),
    }
}

#[test]
fn test_rc_multiple_clones() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(42));
    let _y = track_rc_clone("y", "x", Rc::clone(&x));
    let _z = track_rc_clone("z", "x", Rc::clone(&x));
    let _w = track_rc_clone("w", "x", Rc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 4);

    // Verify strong counts increase
    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));
    assert_eq!(events[3].strong_count(), Some(4));
}

#[test]
fn test_rc_value_correctness() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(42));
    assert_eq!(*x, 42);

    let y = track_rc_clone("y", "x", Rc::clone(&x));
    assert_eq!(*y, 42);

    let z = track_rc_clone("z", "x", Rc::clone(&x));
    assert_eq!(*z, 42);

    // All should point to same value
    assert_eq!(*x, *y);
    assert_eq!(*y, *z);
}

#[test]
fn test_rc_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(String::from("hello")));
    assert_eq!(*x, "hello");

    let y = track_rc_clone("y", "x", Rc::clone(&x));
    assert_eq!(*y, "hello");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_rc());
    assert!(events[1].is_rc());
}

#[test]
fn test_rc_with_vec() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(vec![1, 2, 3, 4, 5]));
    assert_eq!(x.len(), 5);

    let y = track_rc_clone("y", "x", Rc::clone(&x));
    assert_eq!(y.len(), 5);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_rc_nested() {
    let _lock = TEST_LOCK.lock();
    reset();

    let inner = Rc::new(42);
    let _outer = track_rc_new("outer", Rc::new(inner));

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_rc_clone_chain() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(42));
    let y = track_rc_clone("y", "x", Rc::clone(&x));
    let _z = track_rc_clone("z", "y", Rc::clone(&y));

    let events = get_events();
    assert_eq!(events.len(), 3);

    // All should have increasing strong counts
    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));
}

#[test]
fn test_rc_weak_count_zero() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_rc_new("x", Rc::new(42));

    let events = get_events();
    assert_eq!(events[0].weak_count(), Some(0));
}

// ============================================================================
// Arc Tests
// ============================================================================

#[test]
fn test_arc_new_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_arc_new("x", Arc::new(42));

    let events = get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        Event::ArcNew {
            var_name,
            strong_count,
            weak_count,
            ..
        } => {
            assert_eq!(var_name, "x");
            assert_eq!(*strong_count, 1);
            assert_eq!(*weak_count, 0);
        }
        _ => panic!("Expected ArcNew event, got {:?}", events[0]),
    }
}

#[test]
fn test_arc_clone_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(42));
    let _y = track_arc_clone("y", "x", Arc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 2);

    match &events[1] {
        Event::ArcClone {
            var_name,
            source_id,
            strong_count,
            ..
        } => {
            assert_eq!(var_name, "y");
            assert_eq!(source_id, "x");
            assert_eq!(*strong_count, 2);
        }
        _ => panic!("Expected ArcClone event"),
    }
}

#[test]
fn test_arc_multiple_clones() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(100));
    let _y = track_arc_clone("y", "x", Arc::clone(&x));
    let _z = track_arc_clone("z", "x", Arc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 3);

    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));
}

#[test]
fn test_arc_value_correctness() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(42));
    assert_eq!(*x, 42);

    let y = track_arc_clone("y", "x", Arc::clone(&x));
    assert_eq!(*y, 42);

    assert_eq!(*x, *y);
}

#[test]
fn test_arc_thread_safety() {
    use std::thread;

    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(42));
    let x_clone = track_arc_clone("x_clone", "x", Arc::clone(&x));

    let handle = thread::spawn(move || {
        assert_eq!(*x_clone, 42);
    });

    handle.join().unwrap();
    assert_eq!(*x, 42);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_arc_multiple_threads() {
    use std::thread;

    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(100));
    let mut handles = vec![];

    for i in 0..5 {
        let x_clone = track_arc_clone(&format!("clone_{}", i), "x", Arc::clone(&x));
        handles.push(thread::spawn(move || {
            assert_eq!(*x_clone, 100);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let events = get_events();
    assert_eq!(events.len(), 6); // 1 new + 5 clones
}

#[test]
fn test_arc_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(String::from("hello")));
    assert_eq!(*x, "hello");

    let y = track_arc_clone("y", "x", Arc::clone(&x));
    assert_eq!(*y, "hello");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_arc());
    assert!(events[1].is_arc());
}

// ============================================================================
// Mixed Rc/Arc Tests
// ============================================================================

#[test]
fn test_mixed_rc_arc() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _rc = track_rc_new("rc", Rc::new(42));
    let _arc = track_arc_new("arc", Arc::new(100));

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_rc());
    assert!(events[1].is_arc());
}

#[test]
fn test_event_type_helpers() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _rc = track_rc_new("rc", Rc::new(42));
    let _arc = track_arc_new("arc", Arc::new(100));

    let events = get_events();

    assert!(events[0].is_refcounted());
    assert!(events[1].is_refcounted());

    assert!(events[0].is_rc());
    assert!(!events[0].is_arc());

    assert!(events[1].is_arc());
    assert!(!events[1].is_rc());
}

#[test]
fn test_strong_count_accessor() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(42));
    let _y = track_rc_clone("y", "x", Rc::clone(&x));
    let _z = track_rc_clone("z", "x", Rc::clone(&x));

    let events = get_events();

    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));
}

#[test]
fn test_weak_count_accessor() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_rc_new("x", Rc::new(42));

    let events = get_events();
    assert_eq!(events[0].weak_count(), Some(0));
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_rc_complex_lifecycle() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_rc_new("x", Rc::new(vec![1, 2, 3]));
    let y = track_rc_clone("y", "x", Rc::clone(&x));
    let z = track_rc_clone("z", "x", Rc::clone(&x));

    assert_eq!(x.len(), 3);
    assert_eq!(y.len(), 3);
    assert_eq!(z.len(), 3);

    drop(y);
    drop(z);

    assert_eq!(x.len(), 3);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_arc_complex_lifecycle() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_arc_new("x", Arc::new(String::from("test")));
    let y = track_arc_clone("y", "x", Arc::clone(&x));

    assert_eq!(*x, "test");
    assert_eq!(*y, "test");

    drop(y);
    assert_eq!(*x, "test");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_rc_with_struct() {
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let _lock = TEST_LOCK.lock();
    reset();

    let p = track_rc_new("p", Rc::new(Point { x: 10, y: 20 }));
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    let q = track_rc_clone("q", "p", Rc::clone(&p));
    assert_eq!(*p, *q);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_arc_with_struct() {
    #[derive(Debug, PartialEq)]
    struct Data {
        value: i32,
    }

    let _lock = TEST_LOCK.lock();
    reset();

    let d = track_arc_new("d", Arc::new(Data { value: 100 }));
    assert_eq!(d.value, 100);

    let e = track_arc_clone("e", "d", Arc::clone(&d));
    assert_eq!(*d, *e);

    let events = get_events();
    assert_eq!(events.len(), 2);
}
