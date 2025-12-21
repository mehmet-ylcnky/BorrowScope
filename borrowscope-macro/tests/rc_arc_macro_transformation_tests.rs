//! Macro transformation tests for Rc/Arc smart pointers
//!
//! These tests verify that the #[trace_borrow] macro correctly transforms
//! Rc::new, Rc::clone, Arc::new, and Arc::clone operations.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use serial_test::serial;
use std::rc::Rc;
use std::sync::Arc;

#[test]
#[serial]
#[serial]
fn test_rc_new_transformation() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let _x = Rc::new(42);
    }

    test_fn();

    let events = get_events();
    assert!(!events.is_empty(), "Should have tracked Rc::new");

    // Find RcNew event
    let rc_new_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_new_events.len(), 1, "Should have exactly one Rc event");
}

#[test]
#[serial]
fn test_rc_clone_transformation() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Rc::new(42);
        let _y = Rc::clone(&x);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert!(
        rc_events.len() >= 2,
        "Should have at least RcNew and RcClone events, got {}",
        rc_events.len()
    );
}

#[test]
#[serial]
fn test_rc_multiple_clones_transformation() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Rc::new(100);
        let _y = Rc::clone(&x);
        let _z = Rc::clone(&x);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(
        rc_events.len(),
        3,
        "Should have 1 RcNew and 2 RcClone events"
    );
}

#[test]
#[serial]
fn test_arc_new_transformation() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let _x = Arc::new(42);
    }

    test_fn();

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert_eq!(arc_events.len(), 1, "Should have exactly one Arc event");
}

#[test]
#[serial]
fn test_arc_clone_transformation() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Arc::new(42);
        let _y = Arc::clone(&x);
    }

    test_fn();

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert!(
        arc_events.len() >= 2,
        "Should have at least ArcNew and ArcClone events, got {}",
        arc_events.len()
    );
}

#[test]
#[serial]
fn test_rc_value_correctness() {
    reset();

    #[trace_borrow]
    fn test_fn() -> i32 {
        let x = Rc::new(42);
        *x
    }

    let result = test_fn();
    assert_eq!(result, 42, "Rc value should be preserved");
}

#[test]
#[serial]
fn test_arc_value_correctness() {
    reset();

    #[trace_borrow]
    fn test_fn() -> i32 {
        let x = Arc::new(99);
        *x
    }

    let result = test_fn();
    assert_eq!(result, 99, "Arc value should be preserved");
}

#[test]
#[serial]
fn test_rc_clone_value_correctness() {
    reset();

    #[trace_borrow]
    fn test_fn() -> (i32, i32) {
        let x = Rc::new(42);
        let y = Rc::clone(&x);
        (*x, *y)
    }

    let (val1, val2) = test_fn();
    assert_eq!(val1, 42);
    assert_eq!(val2, 42);
}

#[test]
#[serial]
fn test_rc_with_string() {
    reset();

    #[trace_borrow]
    fn test_fn() -> String {
        let x = Rc::new(String::from("hello"));
        (*x).clone()
    }

    let result = test_fn();
    assert_eq!(result, "hello");

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_events.len(), 1);
}

#[test]
#[serial]
fn test_arc_with_string() {
    reset();

    #[trace_borrow]
    fn test_fn() -> String {
        let x = Arc::new(String::from("world"));
        (*x).clone()
    }

    let result = test_fn();
    assert_eq!(result, "world");

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert_eq!(arc_events.len(), 1);
}

#[test]
#[serial]
fn test_rc_in_nested_scope() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Rc::new(42);
        {
            let _y = Rc::clone(&x);
        }
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_events.len(), 2);
}

#[test]
#[serial]
fn test_mixed_rc_arc() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let _rc = Rc::new(1);
        let _arc = Arc::new(2);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();

    assert_eq!(rc_events.len(), 1);
    assert_eq!(arc_events.len(), 1);
}

#[test]
#[serial]
fn test_rc_with_struct() {
    reset();

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[trace_borrow]
    fn test_fn() -> i32 {
        let p = Rc::new(Point { x: 10, y: 20 });
        p.x + p.y
    }

    let result = test_fn();
    assert_eq!(result, 30);

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_events.len(), 1);
}

#[test]
#[serial]
fn test_arc_with_vec() {
    reset();

    #[trace_borrow]
    fn test_fn() -> usize {
        let v = Arc::new(vec![1, 2, 3, 4, 5]);
        v.len()
    }

    let result = test_fn();
    assert_eq!(result, 5);

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert!(!arc_events.is_empty(), "Should have at least one Arc event");
}

#[test]
#[serial]
fn test_rc_clone_chain() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Rc::new(42);
        let y = Rc::clone(&x);
        let _z = Rc::clone(&y);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert!(
        rc_events.len() >= 3,
        "Should have at least 3 Rc events (1 new + 2 clones), got {}",
        rc_events.len()
    );
}

#[test]
#[serial]
fn test_arc_thread_send() {
    use std::sync::mpsc;
    use std::thread;

    reset();

    #[trace_borrow]
    fn create_and_clone_arc() -> (Arc<i32>, Arc<i32>) {
        let x = Arc::new(42);
        let x_clone = Arc::clone(&x);
        (x, x_clone)
    }

    let (arc1, arc2) = create_and_clone_arc();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(*arc2).unwrap();
    });

    let val = rx.recv().unwrap();
    assert_eq!(val, 42);
    assert_eq!(*arc1, 42);

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert!(
        arc_events.len() >= 2,
        "Should have at least 2 Arc events (new + clone), got {}",
        arc_events.len()
    );
}

#[test]
#[serial]
fn test_rc_strong_count_tracking() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Rc::new(42);
        let _y = Rc::clone(&x);
        let _z = Rc::clone(&x);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();

    // Verify strong counts are tracked
    for event in rc_events {
        assert!(event.strong_count().is_some(), "Should track strong count");
    }
}

#[test]
#[serial]
fn test_arc_strong_count_tracking() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let x = Arc::new(42);
        let _y = Arc::clone(&x);
    }

    test_fn();

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();

    for event in arc_events {
        assert!(event.strong_count().is_some(), "Should track strong count");
    }
}

#[test]
#[serial]
fn test_rc_with_option() {
    reset();

    #[trace_borrow]
    fn test_fn() -> Option<i32> {
        let x = Rc::new(Some(42));
        *x
    }

    let result = test_fn();
    assert_eq!(result, Some(42));
}

#[test]
#[serial]
fn test_rc_with_result() {
    reset();

    #[trace_borrow]
    fn test_fn() -> std::result::Result<i32, String> {
        let x = Rc::new(Ok::<i32, String>(42));
        (*x).clone()
    }

    let result = test_fn();
    assert_eq!(result, Ok(42));
}

#[test]
#[serial]
fn test_rc_full_path() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let _x = std::rc::Rc::new(42);
    }

    test_fn();

    let events = get_events();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_events.len(), 1);
}

#[test]
#[serial]
fn test_arc_full_path() {
    reset();

    #[trace_borrow]
    fn test_fn() {
        let _x = std::sync::Arc::new(42);
    }

    test_fn();

    let events = get_events();
    let arc_events: Vec<_> = events.iter().filter(|e| e.is_arc()).collect();
    assert_eq!(arc_events.len(), 1);
}
