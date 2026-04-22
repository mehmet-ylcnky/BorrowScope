//! Phase 3: Self-Borrow Inference — Runtime Verification
//! Tests track_borrow/track_borrow_mut for method receiver wrapping

use borrowscope_runtime::*;

// === Immutable borrow tracking ===

#[test]
fn test_phase3_immutable_borrow() {
    reset();
    let v = vec![1, 2, 3];
    let _r = track_borrow("method_borrow", &v);
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
}

#[test]
fn test_phase3_immutable_borrow_with_id() {
    reset();
    let v = vec![1, 2, 3];
    let _r = track_borrow_with_id(1, 0, "method_borrow", "test:1", false, &v);
    let events = get_events();
    let borrow = events.iter().find(|e| matches!(e, Event::Borrow { .. }));
    assert!(borrow.is_some());
}

// === Mutable borrow tracking ===

#[test]
fn test_phase3_mutable_borrow() {
    reset();
    let mut v = vec![1, 2, 3];
    let _r = track_borrow_mut("method_borrow", &mut v);
    let events = get_events();
    // BorrowMut is the event for mutable borrows
    let has_borrow = events.iter().any(|e| matches!(e, Event::Borrow { .. }));
    assert!(has_borrow, "Mutable borrow should produce a borrow event");
}

#[test]
fn test_phase3_mutable_borrow_with_id() {
    reset();
    let mut v = vec![1, 2, 3];
    let _r = track_borrow_mut_with_id(1, 0, "method_borrow", "test:1", &mut v);
    let events = get_events();
    assert!(!events.is_empty(), "Mutable borrow with ID should produce events");
}

// === Multiple borrows on same variable ===

#[test]
fn test_phase3_multiple_immutable_borrows() {
    reset();
    let v = vec![1, 2, 3];
    let _r1 = track_borrow("borrow_1", &v);
    let _r2 = track_borrow("borrow_2", &v);
    let _r3 = track_borrow("borrow_3", &v);
    let events = get_events();
    let borrow_count = events.iter().filter(|e| matches!(e, Event::Borrow { .. })).count();
    assert_eq!(borrow_count, 3, "Three borrows should produce 3 events");
}

// === Borrow then drop ===

#[test]
fn test_phase3_borrow_lifecycle() {
    reset();
    let v = vec![1, 2, 3];
    {
        let _r = track_borrow("borrow_0", &v);
        // borrow active here
    }
    // borrow dropped
    track_drop("v");
    let events = get_events();
    let has_borrow = events.iter().any(|e| matches!(e, Event::Borrow { .. }));
    let has_drop = events.iter().any(|e| matches!(e, Event::Drop { .. }));
    assert!(has_borrow && has_drop, "Should have both borrow and drop events");
}

// === Interleaved mutable and immutable ===

#[test]
fn test_phase3_interleaved_borrows() {
    reset();
    let v = vec![1, 2, 3];
    let _r1 = track_borrow("read_1", &v);
    drop(_r1);
    // Now safe to take mutable borrow
    let mut v = v;
    let _r2 = track_borrow_mut("write_1", &mut v);
    let events = get_events();
    assert!(events.len() >= 2, "Should have at least 2 borrow events");
}

// === Edge case: borrow of reference ===

#[test]
fn test_phase3_borrow_of_ref() {
    reset();
    let data = vec![1, 2, 3];
    let r = &data;
    let _r2 = track_borrow("borrow_ref", r);
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
}

// === Clone tracking (Phase 5 overlap) ===

#[test]
fn test_phase3_clone_produces_clone_event() {
    reset();
    let v = vec![1, 2, 3];
    track_clone(1, "v", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Clone { .. })));
}

#[test]
fn test_phase3_multiple_clones() {
    reset();
    let v = vec![1, 2, 3];
    track_clone(1, "v", "test:1");
    track_clone(2, "v", "test:2");
    track_clone(3, "v", "test:3");
    let events = get_events();
    let clone_count = events.iter().filter(|e| matches!(e, Event::Clone { .. })).count();
    assert_eq!(clone_count, 3);
}

// === Rc/Arc clone should NOT be generic Clone ===

#[test]
fn test_phase3_rc_clone_is_rc_clone_not_generic() {
    reset();
    let rc = std::rc::Rc::new(42);
    let _rc2 = track_rc_clone_with_id(2, 1, "rc2", "test:1", rc.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RcClone { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Clone { .. })), "Rc clone should NOT produce generic Clone");
}

#[test]
fn test_phase3_arc_clone_is_arc_clone_not_generic() {
    reset();
    let arc = std::sync::Arc::new(42);
    let _arc2 = track_arc_clone_with_id(2, 1, "arc2", "test:1", arc.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::ArcClone { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Clone { .. })), "Arc clone should NOT produce generic Clone");
}
