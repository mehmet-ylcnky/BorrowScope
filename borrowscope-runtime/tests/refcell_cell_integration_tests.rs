//! Integration tests for RefCell and Cell tracking
//!
//! These tests verify that RefCell and Cell operations are properly tracked
//! including borrow checking, interior mutability, and runtime violations.

use borrowscope_runtime::*;
use std::cell::{Cell, RefCell};
use std::sync::Mutex;

// Global lock for test isolation
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_refcell_new() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_refcell_new("x", RefCell::new(42));

    let events = get_events();
    assert!(!events.is_empty());

    let refcell_events: Vec<_> = events.iter().filter(|e| e.is_refcell()).collect();
    assert_eq!(refcell_events.len(), 1);
}

#[test]
fn test_refcell_immutable_borrow() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(42));
    let r = track_refcell_borrow("borrow_1", "refcell_x", "test.rs:1:1", x.borrow());

    assert_eq!(*r, 42);

    drop(r);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();
    let refcell_events: Vec<_> = events.iter().filter(|e| e.is_refcell()).collect();
    assert_eq!(refcell_events.len(), 3); // New, Borrow, Drop
}

#[test]
fn test_refcell_mutable_borrow() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(42));
    let mut m = track_refcell_borrow_mut("borrow_1", "refcell_x", "test.rs:1:1", x.borrow_mut());

    *m = 100;
    assert_eq!(*m, 100);

    drop(m);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();
    let refcell_events: Vec<_> = events.iter().filter(|e| e.is_refcell()).collect();
    assert_eq!(refcell_events.len(), 3);
}

#[test]
fn test_refcell_multiple_immutable_borrows() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(42));
    let r1 = track_refcell_borrow("borrow_1", "refcell_x", "test.rs:1:1", x.borrow());
    let r2 = track_refcell_borrow("borrow_2", "refcell_x", "test.rs:2:1", x.borrow());

    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);

    drop(r1);
    track_refcell_drop("borrow_1", "test.rs:3:1");
    drop(r2);
    track_refcell_drop("borrow_2", "test.rs:4:1");

    let events = get_events();
    let borrow_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::RefCellBorrow { .. }))
        .collect();
    assert_eq!(borrow_events.len(), 2);
}

#[test]
fn test_refcell_value_mutation() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(42));

    {
        let mut m =
            track_refcell_borrow_mut("borrow_1", "refcell_x", "test.rs:1:1", x.borrow_mut());
        *m = 100;
        drop(m);
        track_refcell_drop("borrow_1", "test.rs:2:1");
    }

    {
        let r = track_refcell_borrow("borrow_2", "refcell_x", "test.rs:3:1", x.borrow());
        assert_eq!(*r, 100);
        drop(r);
        track_refcell_drop("borrow_2", "test.rs:4:1");
    }

    let events = get_events();
    assert!(events.len() >= 5);
}

#[test]
#[should_panic(expected = "already borrowed")]
fn test_refcell_panic_mutable_while_borrowed() {
    let x = RefCell::new(42);
    let _r = x.borrow();
    let _m = x.borrow_mut(); // Should panic
}

#[test]
#[should_panic(expected = "already mutably borrowed")]
fn test_refcell_panic_immutable_while_mutably_borrowed() {
    let x = RefCell::new(42);
    let _m = x.borrow_mut();
    let _r = x.borrow(); // Should panic
}

#[test]
fn test_refcell_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(String::from("hello")));
    let r = track_refcell_borrow("borrow_1", "refcell_x", "test.rs:1:1", x.borrow());

    assert_eq!(*r, "hello");

    drop(r);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();
    let refcell_events: Vec<_> = events.iter().filter(|e| e.is_refcell()).collect();
    assert!(!refcell_events.is_empty());
}

#[test]
fn test_refcell_with_vec() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(vec![1, 2, 3]));
    let mut m = track_refcell_borrow_mut("borrow_1", "refcell_x", "test.rs:1:1", x.borrow_mut());

    m.push(4);
    assert_eq!(m.len(), 4);

    drop(m);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();
    assert!(events.iter().any(|e| e.is_refcell()));
}

#[test]
fn test_refcell_nested_in_struct() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[allow(dead_code)]
    struct Counter {
        value: RefCell<i32>,
    }

    let counter = Counter {
        value: track_refcell_new("value", RefCell::new(0)),
    };

    {
        let mut m = track_refcell_borrow_mut(
            "borrow_1",
            "refcell_value",
            "test.rs:1:1",
            counter.value.borrow_mut(),
        );
        *m += 1;
        drop(m);
        track_refcell_drop("borrow_1", "test.rs:2:1");
    }

    {
        let r = track_refcell_borrow(
            "borrow_2",
            "refcell_value",
            "test.rs:3:1",
            counter.value.borrow(),
        );
        assert_eq!(*r, 1);
        drop(r);
        track_refcell_drop("borrow_2", "test.rs:4:1");
    }

    let events = get_events();
    assert!(events.iter().any(|e| e.is_refcell()));
}

#[test]
fn test_cell_new() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _x = track_cell_new("x", Cell::new(42));

    let events = get_events();
    let cell_events: Vec<_> = events.iter().filter(|e| e.is_cell()).collect();
    assert_eq!(cell_events.len(), 1);
}

#[test]
fn test_cell_get() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("x", Cell::new(42));
    let val = track_cell_get("cell_x", "test.rs:1:1", x.get());

    assert_eq!(val, 42);

    let events = get_events();
    let get_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::CellGet { .. }))
        .collect();
    assert_eq!(get_events.len(), 1);
}

#[test]
fn test_cell_set() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("x", Cell::new(42));

    x.set(100);
    track_cell_set("cell_x", "test.rs:1:1");

    let val = track_cell_get("cell_x", "test.rs:2:1", x.get());
    assert_eq!(val, 100);

    let events = get_events();
    let set_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::CellSet { .. }))
        .collect();
    assert_eq!(set_events.len(), 1);
}

#[test]
fn test_cell_multiple_operations() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("x", Cell::new(0));

    for i in 1..=5 {
        x.set(i);
        track_cell_set("cell_x", &format!("test.rs:{}:1", i));
    }

    let val = track_cell_get("cell_x", "test.rs:6:1", x.get());
    assert_eq!(val, 5);

    let events = get_events();
    let set_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::CellSet { .. }))
        .collect();
    assert_eq!(set_events.len(), 5);
}

#[test]
fn test_cell_with_bool() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("flag", Cell::new(false));

    x.set(true);
    track_cell_set("cell_flag", "test.rs:1:1");

    let val = track_cell_get("cell_flag", "test.rs:2:1", x.get());
    assert!(val);

    let events = get_events();
    assert!(events.iter().any(|e| e.is_cell()));
}

#[test]
fn test_cell_no_borrow_checking() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("x", Cell::new(42));

    // Cell allows multiple "borrows" through get/set
    let _v1 = track_cell_get("cell_x", "test.rs:1:1", x.get());
    let _v2 = track_cell_get("cell_x", "test.rs:2:1", x.get());
    x.set(100);
    track_cell_set("cell_x", "test.rs:3:1");
    let _v3 = track_cell_get("cell_x", "test.rs:4:1", x.get());

    let events = get_events();
    let cell_ops: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::CellGet { .. } | Event::CellSet { .. }))
        .collect();
    assert_eq!(cell_ops.len(), 4); // 3 gets + 1 set
}

#[test]
fn test_refcell_in_rc() {
    let _lock = TEST_LOCK.lock();
    reset();

    use std::rc::Rc;

    let x = Rc::new(track_refcell_new("x", RefCell::new(42)));
    let r = track_refcell_borrow("borrow_1", "refcell_x", "test.rs:1:1", x.borrow());

    assert_eq!(*r, 42);

    drop(r);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();
    assert!(events.iter().any(|e| e.is_refcell()));
}

#[test]
fn test_cell_in_rc() {
    let _lock = TEST_LOCK.lock();
    reset();

    use std::rc::Rc;

    let x = Rc::new(track_cell_new("x", Cell::new(42)));
    let val = track_cell_get("cell_x", "test.rs:1:1", x.get());

    assert_eq!(val, 42);

    x.set(100);
    track_cell_set("cell_x", "test.rs:2:1");

    let events = get_events();
    assert!(events.iter().any(|e| e.is_cell()));
}

#[test]
fn test_refcell_event_ordering() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_refcell_new("x", RefCell::new(42));
    let r = track_refcell_borrow("borrow_1", "refcell_x", "test.rs:1:1", x.borrow());
    drop(r);
    track_refcell_drop("borrow_1", "test.rs:2:1");

    let events = get_events();

    // Verify ordering: New -> Borrow -> Drop
    let refcell_events: Vec<_> = events.iter().filter(|e| e.is_refcell()).collect();
    assert_eq!(refcell_events.len(), 3);

    assert!(matches!(refcell_events[0], Event::RefCellNew { .. }));
    assert!(matches!(refcell_events[1], Event::RefCellBorrow { .. }));
    assert!(matches!(refcell_events[2], Event::RefCellDrop { .. }));
}

#[test]
fn test_cell_event_ordering() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_cell_new("x", Cell::new(42));
    let _val = track_cell_get("cell_x", "test.rs:1:1", x.get());
    x.set(100);
    track_cell_set("cell_x", "test.rs:2:1");

    let events = get_events();

    // Verify ordering: New -> Get -> Set
    let cell_events: Vec<_> = events.iter().filter(|e| e.is_cell()).collect();
    assert_eq!(cell_events.len(), 3);

    assert!(matches!(cell_events[0], Event::CellNew { .. }));
    assert!(matches!(cell_events[1], Event::CellGet { .. }));
    assert!(matches!(cell_events[2], Event::CellSet { .. }));
}

#[test]
fn test_interior_mutability_helper() {
    let _lock = TEST_LOCK.lock();
    reset();

    let _refcell = track_refcell_new("x", RefCell::new(42));
    let _cell = track_cell_new("y", Cell::new(100));

    let events = get_events();
    let interior_mut_events: Vec<_> = events
        .iter()
        .filter(|e| e.is_interior_mutability())
        .collect();
    assert_eq!(interior_mut_events.len(), 2);
}
