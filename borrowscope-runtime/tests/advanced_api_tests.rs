//! Advanced API tests demonstrating ID-based tracking with full metadata
//!
//! This test suite shows how the advanced API would work with explicit IDs,
//! locations, and type information for production-grade tracking.

use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_id_based_tracking_concept() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Each variable gets a unique ID from the macro
    // ID 1: x
    let x = track_new("x", 42);

    // ID 2: y (borrows from ID 1)
    let _y = track_borrow("y", &x);

    // ID 3: z (moves from ID 1)
    let _z = track_move("x", "z", x);

    let events = get_events();
    assert!(events.len() >= 3);

    // Verify we can track the relationship chain
    assert!(events[0].is_new());
    assert!(events[1].is_borrow());
    assert!(events[2].is_move());
}

#[test]
fn test_location_tracking_concept() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track source locations for debugging
    // In advanced API: track_new(1, "x", "i32", "src/main.rs:10:5", 42)
    let _x = track_new("x @ src/main.rs:10:5", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);

    // Location info embedded in variable name for now
    if let Some(name) = events[0].var_name() {
        assert!(name.contains("@"));
    }
}

#[test]
fn test_rc_with_id_tracking_concept() {
    use std::rc::Rc;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track Rc with explicit IDs
    // ID 1: x = Rc::new(42)
    let x = track_rc_new("x", Rc::new(42));

    // ID 2: y = Rc::clone(&x), source is ID 1
    let _y = track_rc_clone("y", "x", Rc::clone(&x));

    // ID 3: z = Rc::clone(&x), source is ID 1
    let _z = track_rc_clone("z", "x", Rc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify reference counts
    assert_eq!(events[0].strong_count(), Some(1));
    assert_eq!(events[1].strong_count(), Some(2));
    assert_eq!(events[2].strong_count(), Some(3));

    // Verify we can trace the clone chain
    match &events[1] {
        Event::RcClone { source_id, .. } => {
            assert_eq!(source_id, "x");
        }
        _ => panic!("Expected RcClone"),
    }
}

#[test]
fn test_arc_with_id_tracking_concept() {
    use std::sync::Arc;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track Arc with explicit IDs for thread safety
    let x = track_arc_new("x", Arc::new(100));
    let _y = track_arc_clone("y", "x", Arc::clone(&x));

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Verify Arc-specific tracking
    assert!(events[0].is_arc());
    assert!(events[1].is_arc());
    assert_eq!(events[1].strong_count(), Some(2));
}

#[test]
fn test_complex_ownership_graph_concept() {
    use std::rc::Rc;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Build complex ownership graphs with ID-based tracking
    // ID 1: data = Rc::new(vec![1,2,3])
    let data = track_rc_new("data", Rc::new(vec![1, 2, 3]));

    // ID 2: reader1 = Rc::clone(&data)
    let _reader1 = track_rc_clone("reader1", "data", Rc::clone(&data));

    // ID 3: reader2 = Rc::clone(&data)
    let _reader2 = track_rc_clone("reader2", "data", Rc::clone(&data));

    // ID 4: processor = Rc::clone(&data)
    let _processor = track_rc_clone("processor", "data", Rc::clone(&data));

    let events = get_events();
    assert_eq!(events.len(), 4);

    // Verify we can build the ownership graph:
    // data (ID 1) is shared by reader1 (ID 2), reader2 (ID 3), processor (ID 4)
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    assert_eq!(rc_events.len(), 4);

    // Final strong count should be 4
    assert_eq!(events[3].strong_count(), Some(4));
}

#[test]
fn test_borrow_chain_tracking_concept() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track borrow chains with IDs
    // ID 1: x = String::from("hello")
    let x = track_new("x", String::from("hello"));

    // ID 2: r1 = &x (borrows from ID 1)
    let r1 = track_borrow("r1", &x);

    // ID 3: r2 = &r1 (borrows from ID 2, which borrows from ID 1)
    let _r2 = track_borrow("r2", &r1);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify borrow chain
    assert!(events[0].is_new());
    assert!(events[1].is_borrow());
    assert!(events[2].is_borrow());
}

#[test]
fn test_type_information_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track full type information
    let _x = track_new("x", vec![1, 2, 3, 4, 5]);

    let events = get_events();
    assert_eq!(events.len(), 1);

    // Type information is captured
    match &events[0] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("Vec") || type_name.contains("alloc"));
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_mutable_vs_immutable_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Distinguish mutable and immutable borrows
    let mut x = track_new("x", vec![1, 2, 3]);

    let _r1 = track_borrow("r1", &x);
    let _r2 = track_borrow_mut("r2", &mut x);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify mutability tracking
    match &events[1] {
        Event::Borrow { mutable, .. } => assert!(!mutable),
        _ => panic!("Expected Borrow"),
    }

    match &events[2] {
        Event::Borrow { mutable, .. } => assert!(*mutable),
        _ => panic!("Expected Borrow"),
    }
}

#[test]
fn test_drop_tracking_concept() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track when variables are dropped
    {
        let _x = track_new("x", 42);
        track_drop("x");
    }

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_new());
    assert!(events[1].is_drop());
}

#[test]
fn test_concurrent_tracking_concept() {
    use std::sync::Arc;
    use std::thread;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track Arc across threads
    let data = track_arc_new("data", Arc::new(42));
    let data_clone = track_arc_clone("data_clone", "data", Arc::clone(&data));

    let handle = thread::spawn(move || {
        assert_eq!(*data_clone, 42);
    });

    handle.join().unwrap();

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Both events should be Arc events
    assert!(events[0].is_arc());
    assert!(events[1].is_arc());
}

#[test]
fn test_weak_reference_tracking_concept() {
    use std::rc::Rc;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Track weak references (future enhancement)
    let x = track_rc_new("x", Rc::new(42));

    let events = get_events();
    assert_eq!(events.len(), 1);

    // Weak count should be 0 initially
    assert_eq!(events[0].weak_count(), Some(0));
}

#[test]
fn test_event_query_api() {
    use std::rc::Rc;

    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Query events by type
    let _x = track_new("x", 42);
    let _rc = track_rc_new("rc", Rc::new(100));
    let _y = track_borrow("y", &_x);

    let events = get_events();

    // Query by type
    let new_events: Vec<_> = events.iter().filter(|e| e.is_new()).collect();
    let rc_events: Vec<_> = events.iter().filter(|e| e.is_rc()).collect();
    let borrow_events: Vec<_> = events.iter().filter(|e| e.is_borrow()).collect();

    assert_eq!(new_events.len(), 1);
    assert_eq!(rc_events.len(), 1);
    assert_eq!(borrow_events.len(), 1);
}

#[test]
fn test_timestamp_ordering() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Events are ordered by timestamp
    let _x = track_new("x", 1);
    let _y = track_new("y", 2);
    let _z = track_new("z", 3);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Timestamps should be monotonically increasing
    assert!(events[0].timestamp() < events[1].timestamp());
    assert!(events[1].timestamp() < events[2].timestamp());
}

#[test]
fn test_graph_building_from_events() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Concept: Build ownership graph from events
    let x = track_new("x", 42);
    let _y = track_borrow("y", &x);
    track_drop("x");

    let graph = get_graph();

    // Graph should have nodes and edges
    assert!(!graph.nodes.is_empty());

    let stats = graph.stats();
    assert_eq!(stats.total_variables, 1);
}
