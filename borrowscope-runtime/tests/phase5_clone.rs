//! Phase 5: Clone Trait Verification — Runtime Verification
//! Tests that Clone::clone, Rc::clone, Arc::clone produce distinct event types

use borrowscope_runtime::*;

// === Generic Clone::clone ===

#[test]
fn test_phase5_generic_clone() {
    reset();
    let v = vec![1, 2, 3];
    track_clone(1, "v", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Clone { .. })));
}

#[test]
fn test_phase5_string_clone() {
    reset();
    let s = String::from("hello");
    track_clone(1, "s", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Clone { .. })));
}

// === Rc::clone produces RcClone, NOT Clone ===

#[test]
fn test_phase5_rc_clone_distinct() {
    reset();
    let rc = std::rc::Rc::new(42);
    let _rc2 = track_rc_clone_with_id(2, 1, "rc2", "test:1", rc.clone());
    let events = get_events();
    let has_rc_clone = events.iter().any(|e| matches!(e, Event::RcClone { .. }));
    let has_generic_clone = events.iter().any(|e| matches!(e, Event::Clone { .. }));
    assert!(has_rc_clone, "Should produce RcClone");
    assert!(!has_generic_clone, "Should NOT produce generic Clone");
}

// === Arc::clone produces ArcClone, NOT Clone ===

#[test]
fn test_phase5_arc_clone_distinct() {
    reset();
    let arc = std::sync::Arc::new(42);
    let _arc2 = track_arc_clone_with_id(2, 1, "arc2", "test:1", arc.clone());
    let events = get_events();
    let has_arc_clone = events.iter().any(|e| matches!(e, Event::ArcClone { .. }));
    let has_generic_clone = events.iter().any(|e| matches!(e, Event::Clone { .. }));
    assert!(has_arc_clone, "Should produce ArcClone");
    assert!(!has_generic_clone, "Should NOT produce generic Clone");
}

// === Weak::clone produces WeakClone ===

#[test]
fn test_phase5_weak_clone_distinct() {
    reset();
    let rc = std::rc::Rc::new(42);
    let weak = std::rc::Rc::downgrade(&rc);
    let _weak2 = track_weak_clone("weak_clone_0", "weak_w", "test:1", weak.clone());
    let events = get_events();
    let has_weak_clone = events.iter().any(|e| matches!(e, Event::WeakClone { .. }));
    let has_generic_clone = events.iter().any(|e| matches!(e, Event::Clone { .. }));
    assert!(has_weak_clone, "Should produce WeakClone");
    assert!(!has_generic_clone, "Should NOT produce generic Clone");
}

// === Multiple different clone types in one function ===

#[test]
fn test_phase5_mixed_clone_types() {
    reset();
    let v = vec![1, 2, 3];
    let rc = std::rc::Rc::new(42);
    let arc = std::sync::Arc::new(100);

    track_clone(1, "v", "test:1");
    let _rc2 = track_rc_clone_with_id(2, 0, "rc2", "test:2", rc.clone());
    let _arc2 = track_arc_clone_with_id(3, 0, "arc2", "test:3", arc.clone());

    let events = get_events();
    let generic_count = events.iter().filter(|e| matches!(e, Event::Clone { .. })).count();
    let rc_count = events.iter().filter(|e| matches!(e, Event::RcClone { .. })).count();
    let arc_count = events.iter().filter(|e| matches!(e, Event::ArcClone { .. })).count();

    assert_eq!(generic_count, 1, "One generic Clone");
    assert_eq!(rc_count, 1, "One RcClone");
    assert_eq!(arc_count, 1, "One ArcClone");
}

// === Clone preserves metadata ===

#[test]
fn test_phase5_clone_preserves_var_name() {
    reset();
    let v = vec![1, 2, 3];
    track_clone(1, "my_vector", "file.rs:10:4");
    let events = get_events();
    if let Some(Event::Clone { var_name, location, .. }) = events.iter().find(|e| matches!(e, Event::Clone { .. })) {
        assert_eq!(var_name, "my_vector");
        assert_eq!(location, "file.rs:10:4");
    } else {
        panic!("Expected Clone event");
    }
}

// === Edge case: clone of empty collections ===

#[test]
fn test_phase5_clone_empty_vec() {
    reset();
    let v: Vec<i32> = vec![];
    track_clone(1, "v", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Clone { .. })));
}

// === Edge case: clone of nested Rc ===

#[test]
fn test_phase5_nested_rc_clone() {
    reset();
    let inner = std::rc::Rc::new(42);
    let outer = std::rc::Rc::new(inner);
    let _outer2 = track_rc_clone_with_id(2, 1, "outer2", "test:1", outer.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RcClone { .. })));
}
