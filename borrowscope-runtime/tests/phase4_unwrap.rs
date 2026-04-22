//! Phase 4: Unwrap Method Tracking — Runtime Verification
//! Tests all 5 unwrap variants and edge cases

use borrowscope_runtime::*;

#[test]
fn test_phase4_unwrap() {
    reset();
    track_unwrap(1, "unwrap", "opt_a", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Unwrap { .. })));
}

#[test]
fn test_phase4_expect() {
    reset();
    track_unwrap(1, "expect", "opt_b", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Unwrap { .. })));
}

#[test]
fn test_phase4_unwrap_or() {
    reset();
    track_unwrap(1, "unwrap_or", "opt_c", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Unwrap { .. })));
}

#[test]
fn test_phase4_unwrap_or_else() {
    reset();
    track_unwrap(1, "unwrap_or_else", "opt_d", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Unwrap { .. })));
}

#[test]
fn test_phase4_unwrap_or_default() {
    reset();
    track_unwrap(1, "unwrap_or_default", "opt_e", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Unwrap { .. })));
}

#[test]
fn test_phase4_all_five_counted() {
    reset();
    track_unwrap(1, "unwrap", "a", "test:1");
    track_unwrap(2, "expect", "b", "test:2");
    track_unwrap(3, "unwrap_or", "c", "test:3");
    track_unwrap(4, "unwrap_or_else", "d", "test:4");
    track_unwrap(5, "unwrap_or_default", "e", "test:5");
    let events = get_events();
    let count = events.iter().filter(|e| matches!(e, Event::Unwrap { .. })).count();
    assert_eq!(count, 5, "All 5 unwrap variants should be tracked");
}

#[test]
fn test_phase4_unwrap_preserves_method_name() {
    reset();
    track_unwrap(1, "expect", "opt", "test:1");
    let events = get_events();
    if let Some(Event::Unwrap { method, .. }) = events.iter().find(|e| matches!(e, Event::Unwrap { .. })) {
        assert_eq!(method, "expect");
    } else {
        panic!("Expected Unwrap event");
    }
}

#[test]
fn test_phase4_unwrap_preserves_var_name() {
    reset();
    track_unwrap(1, "unwrap", "my_option", "test:1");
    let events = get_events();
    if let Some(Event::Unwrap { var_name, .. }) = events.iter().find(|e| matches!(e, Event::Unwrap { .. })) {
        assert_eq!(var_name, "my_option");
    } else {
        panic!("Expected Unwrap event");
    }
}

#[test]
fn test_phase4_unwrap_preserves_location() {
    reset();
    track_unwrap(1, "unwrap", "opt", "file.rs:42:8");
    let events = get_events();
    if let Some(Event::Unwrap { location, .. }) = events.iter().find(|e| matches!(e, Event::Unwrap { .. })) {
        assert_eq!(location, "file.rs:42:8");
    } else {
        panic!("Expected Unwrap event");
    }
}

// === Edge case: multiple unwraps in sequence ===

#[test]
fn test_phase4_sequential_unwraps() {
    reset();
    for i in 0..10 {
        track_unwrap(i, "unwrap", &format!("opt_{}", i), &format!("test:{}", i));
    }
    let events = get_events();
    let count = events.iter().filter(|e| matches!(e, Event::Unwrap { .. })).count();
    assert_eq!(count, 10);
}
