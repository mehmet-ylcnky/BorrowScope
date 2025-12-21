//! Tests for attribute parameter parsing and configuration

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

mod test_utils {
    use std::sync::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

use test_utils::TEST_LOCK;

#[test]
fn test_default_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        let _y = &x;
    }

    example();

    let events = get_events();
    // Should have: New, Borrow, Drop events
    assert!(events.len() >= 2, "Default should track new and borrow");
    assert!(events.iter().any(|e| e.is_new()));
    assert!(events.iter().any(|e| e.is_borrow()));
}

#[test]
fn test_quiet_mode() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow(quiet)]
    fn example() {
        let x = 42;
        let _y = &x;
        for _i in 0..2 {}
    }

    example();

    let events = get_events();
    // Quiet mode: ownership only (new, move, drop, borrow)
    // Should NOT have loop events
    assert!(events.iter().any(|e| e.is_new()));
    assert!(!events.iter().any(|e| matches!(e, Event::LoopEnter { .. })));
}

#[test]
fn test_skip_loops() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow(skip = "loops")]
    fn example() {
        let x = 42;
        for _i in 0..2 {
            let _ = x;
        }
    }

    example();

    let events = get_events();
    // Should have new but NOT loop events
    assert!(events.iter().any(|e| e.is_new()));
    assert!(!events.iter().any(|e| matches!(e, Event::LoopEnter { .. })));
}

#[test]
fn test_skip_branches() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow(skip = "branches")]
    fn example() {
        let x = 42;
        if x > 0 {
            let _ = 1;
        }
    }

    example();

    let events = get_events();
    // Should have new but NOT branch events
    assert!(events.iter().any(|e| e.is_new()));
    assert!(!events.iter().any(|e| matches!(e, Event::Branch { .. })));
}

#[test]
fn test_only_ownership() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow(only = "ownership")]
    fn example() {
        let x = 42;
        let _y = &x;
        for _i in 0..2 {}
        if x > 0 {}
    }

    example();

    let events = get_events();
    // Only ownership events (new, move, drop, borrow)
    for event in &events {
        assert!(
            event.is_new() || event.is_move() || event.is_drop() || event.is_borrow(),
            "Only ownership events expected, got: {:?}",
            event
        );
    }
}

#[test]
fn test_skip_multiple() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow(skip = "loops, branches, methods")]
    fn example() {
        let x = vec![1, 2, 3];
        let _y = x.clone();
        for _i in 0..2 {}
        if true {}
    }

    example();

    let events = get_events();
    // Should NOT have loop, branch, or clone events
    assert!(!events.iter().any(|e| matches!(e, Event::LoopEnter { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Branch { .. })));
    assert!(!events.iter().any(|e| matches!(e, Event::Clone { .. })));
}
