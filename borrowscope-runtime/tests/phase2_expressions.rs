//! Phase 2: Standalone Expression Tracking — Runtime Verification
//! Tests smart pointer creation, clone, weak refs, box raw ops, pin, cow, transmute, thread spawn

use borrowscope_runtime::*;

// === Smart pointer creation ===

#[test]
fn test_phase2_rc_new() {
    reset();
    let _rc = track_rc_new_with_id(1, "rc_data", "Rc<i32>", "test:1", std::rc::Rc::new(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RcNew { .. })));
}

#[test]
fn test_phase2_arc_new() {
    reset();
    let _arc = track_arc_new_with_id(1, "arc_data", "Arc<i32>", "test:1", std::sync::Arc::new(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::ArcNew { .. })));
}

#[test]
fn test_phase2_box_new() {
    reset();
    let _b = track_box_new("boxed", "test:1", Box::new(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::BoxNew { .. })));
}

// === Smart pointer clone ===

#[test]
fn test_phase2_rc_clone() {
    reset();
    let rc = std::rc::Rc::new(42);
    let _rc2 = track_rc_clone_with_id(2, 1, "rc2", "test:1", rc.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RcClone { .. })));
}

#[test]
fn test_phase2_arc_clone() {
    reset();
    let arc = std::sync::Arc::new(42);
    let _arc2 = track_arc_clone_with_id(2, 1, "arc2", "test:1", arc.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::ArcClone { .. })));
}

// === Weak references ===

#[test]
fn test_phase2_weak_new() {
    reset();
    let rc = std::rc::Rc::new(42);
    let _weak = track_weak_new("weak_w", "rc_data", "test:1", std::rc::Rc::downgrade(&rc));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::WeakNew { .. })));
}

#[test]
fn test_phase2_weak_upgrade() {
    reset();
    let rc = std::rc::Rc::new(42);
    let weak = std::rc::Rc::downgrade(&rc);
    let _upgraded = track_weak_upgrade("weak_w", "test:1", weak.upgrade());
    let events = get_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::WeakUpgrade { .. })));
}

#[test]
fn test_phase2_weak_clone() {
    reset();
    let rc = std::rc::Rc::new(42);
    let weak = std::rc::Rc::downgrade(&rc);
    let _weak2 = track_weak_clone("weak_clone_0", "weak_w", "test:1", weak.clone());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::WeakClone { .. })));
}

// === Box raw operations ===

#[test]
fn test_phase2_box_into_raw() {
    reset();
    let b = Box::new(42);
    let _ptr = track_box_into_raw("boxed", "test:1", Box::into_raw(b));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::BoxIntoRaw { .. })));
}

#[test]
fn test_phase2_box_from_raw() {
    reset();
    let b = Box::new(42);
    let ptr = Box::into_raw(b);
    let _b2 = track_box_from_raw("boxed2", "test:1", unsafe { Box::from_raw(ptr) });
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::BoxFromRaw { .. })));
}

// === Pin ===

#[test]
fn test_phase2_pin_new() {
    reset();
    let _pinned = track_pin_new("pinned", "test:1", Box::pin(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::PinNew { .. })));
}

// === Cow ===

#[test]
fn test_phase2_cow_borrowed() {
    reset();
    let _cow = track_cow_borrowed("cow_c", "test:1", std::borrow::Cow::Borrowed("hello"));
    let events = get_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::CowBorrowed { .. })));
}

#[test]
fn test_phase2_cow_owned() {
    reset();
    let _cow = track_cow_owned(
        "cow_c",
        "test:1",
        std::borrow::Cow::<str>::Owned(String::from("world")),
    );
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::CowOwned { .. })));
}

// === Transmute ===

#[test]
fn test_phase2_transmute() {
    reset();
    track_transmute("u32", "f32", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Transmute { .. })));
}

// === Thread spawn ===

#[test]
fn test_phase2_thread_spawn() {
    reset();
    let _h = track_thread_spawn("handle", "test:1", std::thread::spawn(|| 42));
    let events = get_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::ThreadSpawn { .. })));
}

// === Clone ===

#[test]
fn test_phase2_clone() {
    reset();
    let v = vec![1, 2, 3];
    track_clone(1, "v", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Clone { .. })));
}

// === New runtime functions (Group 2) ===

#[test]
fn test_phase2_atomic_new() {
    reset();
    let _a = track_atomic_new(
        "flag",
        "AtomicBool",
        "test:1",
        std::sync::atomic::AtomicBool::new(false),
    );
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
}

#[test]
fn test_phase2_duration_new() {
    reset();
    let _d = track_duration_new("dur", "test:1", std::time::Duration::from_secs(1));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
}

#[test]
fn test_phase2_instant_new() {
    reset();
    let _i = track_instant_new("now", "test:1", std::time::Instant::now());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
}

// === Enriched events (Section C) ===

#[test]
fn test_phase2_await_with_live_vars() {
    reset();
    track_await_start_with_live_vars(1, "my_future", "test:1", &["x", "y", "z"]);
    track_await_end(1, "test:1");
    let events = get_events();
    let await_start = events
        .iter()
        .find(|e| matches!(e, Event::AwaitStart { .. }));
    assert!(await_start.is_some());
    if let Some(Event::AwaitStart { live_variables, .. }) = await_start {
        assert_eq!(live_variables.len(), 3);
        assert_eq!(live_variables[0], "x");
    }
}

#[test]
fn test_phase2_unsafe_block_enriched() {
    reset();
    track_unsafe_block_enter_enriched(1, "test:1", "deref_raw_ptr", Some("*const i32"));
    track_unsafe_block_exit(1, "test:1");
    let events = get_events();
    let enter = events
        .iter()
        .find(|e| matches!(e, Event::UnsafeBlockEnter { .. }));
    assert!(enter.is_some());
    if let Some(Event::UnsafeBlockEnter {
        operation_kind,
        operation_context,
        ..
    }) = enter
    {
        assert_eq!(operation_kind.as_deref(), Some("deref_raw_ptr"));
        assert_eq!(operation_context.as_deref(), Some("*const i32"));
    }
}

#[test]
fn test_phase2_closure_with_trait() {
    reset();
    track_closure_create_with_trait(1, "move", "test:1", "FnOnce");
    let events = get_events();
    let create = events
        .iter()
        .find(|e| matches!(e, Event::ClosureCreate { .. }));
    assert!(create.is_some());
    if let Some(Event::ClosureCreate { fn_trait, .. }) = create {
        assert_eq!(fn_trait.as_deref(), Some("FnOnce"));
    }
}

#[test]
fn test_phase2_match_arm_with_bindings() {
    reset();
    track_match_enter(1, "test:1");
    track_match_arm_with_bindings(1, 0, "Some(x)", "test:1", &["x"]);
    track_match_exit(1, "test:1");
    let events = get_events();
    let arm = events.iter().find(|e| matches!(e, Event::MatchArm { .. }));
    assert!(arm.is_some());
    if let Some(Event::MatchArm { bindings, .. }) = arm {
        assert_eq!(bindings, &["x"]);
    }
}

#[test]
fn test_phase2_borrow_span() {
    reset();
    track_borrow_span("data", "shared", "10:4", "15:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
}

#[test]
fn test_phase2_destructure() {
    reset();
    track_destructure("tuple", &["a", "b", "c"], "test:1");
    let events = get_events();
    let new_count = events
        .iter()
        .filter(|e| matches!(e, Event::New { .. }))
        .count();
    assert_eq!(
        new_count, 3,
        "Destructure of 3 bindings should produce 3 New events"
    );
}

#[test]
fn test_phase2_variant_construct() {
    reset();
    track_variant_construct("Option", "Some", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::New { .. })));
}

// === Autoref/Autoderef/VarRead/VarWrite ===

#[test]
fn test_phase2_autoref() {
    reset();
    track_autoref("data", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
}

#[test]
fn test_phase2_autoderef() {
    reset();
    track_autoderef("ptr", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Deref { .. })));
}

#[test]
fn test_phase2_var_read_write() {
    reset();
    track_var_read("x", "10:4");
    track_var_write("x", "11:4");
    let events = get_events();
    assert!(
        events.len() >= 2,
        "var_read and var_write should produce events"
    );
}

// === Drop with location ===

#[test]
fn test_phase2_drop_at() {
    reset();
    track_drop_at("data", "25:1");
    let events = get_events();
    let drop_event = events.iter().find(|e| matches!(e, Event::Drop { .. }));
    assert!(drop_event.is_some());
    if let Some(Event::Drop { location, .. }) = drop_event {
        assert_eq!(location.as_deref(), Some("25:1"));
    }
}
