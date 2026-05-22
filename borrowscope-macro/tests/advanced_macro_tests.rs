use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_generates_unique_ids() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = 1;
        let _y = 2;
        let _z = 3;
    }

    example_2();

    let events = get_events();
    assert!(
        events.len() >= 3,
        "Should have at least 3 events, got {}",
        events.len()
    );

    // Verify we have New events with IDs
    let new_count = events.iter().filter(|e| e.is_new()).count();
    assert!(new_count >= 3, "Should have at least 3 New events");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_tracks_move_with_ids() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_2() {
        let x = String::from("hello");
        let _y = x; // Move
    }

    example_3();

    let events = get_events();
    assert!(events.len() >= 2);

    // Should have a Move event with IDs
    let has_move = events.iter().any(|e| e.is_move());
    assert!(has_move, "Should track move with IDs");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_tracks_borrow_with_owner_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_3() {
        let x = 42;
        let _r = &x;
    }

    example_4();

    let events = get_events();
    assert!(events.len() >= 2);

    // Should have a Borrow event with owner_id
    match events.iter().find(|e| e.is_borrow()) {
        Some(Event::Borrow { owner_id, .. }) => {
            assert!(
                owner_id.contains("owner_"),
                "Should have owner ID: {}",
                owner_id
            );
        }
        _ => panic!("Should have Borrow event with owner ID"),
    }
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_tracks_mut_borrow_with_ids() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_4() {
        let mut x = vec![1, 2, 3];
        let _r = &mut x;
    }

    example_5();

    let events = get_events();
    assert!(events.len() >= 2);

    // Should have mutable Borrow event
    let has_mut_borrow = events.iter().any(|e| {
        if let Event::Borrow { mutable, .. } = e {
            *mutable
        } else {
            false
        }
    });
    assert!(has_mut_borrow, "Should track mutable borrow");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_location_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_5() {
        let _x = 42;
    }

    example_6();

    let events = get_events();
    assert!(!events.is_empty());

    // Should have at least one New event
    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should have New event");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_complex_ownership_chain() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_6() {
        let x = String::from("hello");
        let r1 = &x;
        let _r2 = &r1;
    }

    example_7();

    let events = get_events();
    assert!(events.len() >= 3);

    // Should track the borrow chain
    let borrow_count = events.iter().filter(|e| e.is_borrow()).count();
    assert!(borrow_count >= 2, "Should track borrow chain");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_multiple_variables_unique_ids() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_7() {
        let _a = 1;
        let _b = 2;
        let _c = 3;
        let _d = 4;
        let _e = 5;
    }

    example_8();

    let events = get_events();

    // All New events should have unique var_ids
    let var_ids: Vec<String> = events
        .iter()
        .filter_map(|e| {
            if let Event::New { var_id, .. } = e {
                Some(var_id.clone())
            } else {
                None
            }
        })
        .collect();

    // Check uniqueness
    let unique_count = var_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert!(
        unique_count >= 5,
        "Should have at least 5 unique var_ids, got {}",
        unique_count
    );
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_move_preserves_source_id() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_8() {
        let x = 42;
        let y = x;
        let _z = y;
    }

    example_9();

    let events = get_events();

    // Should have New and Move events
    let new_count = events.iter().filter(|e| e.is_new()).count();
    let move_count = events.iter().filter(|e| e.is_move()).count();

    assert!(new_count >= 1, "Should have New event");
    assert!(move_count >= 2, "Should have Move events");
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_borrow_and_move() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_9() {
        let x = String::from("test");
        let _r = &x;
        let _y = x;
    }

    example_10();

    let events = get_events();

    // Should have New, Borrow, and Move
    assert!(events.iter().any(|e| e.is_new()));
    assert!(events.iter().any(|e| e.is_borrow()));
    assert!(events.iter().any(|e| e.is_move()));
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_with_box() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_10() {
        let _x = Box::new(42);
    }

    example_11();

    let events = get_events();
    assert!(!events.is_empty());

    // Should track Box allocation (BoxNew event)
    assert!(events.iter().any(|e| e.is_box()));
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_macro_timestamp_ordering() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_11() {
        let _x = 1;
        let _y = 2;
        let _z = 3;
    }

    example_12();

    let events = get_events();

    // Timestamps should be monotonically increasing
    for i in 1..events.len() {
        assert!(
            events[i - 1].timestamp() < events[i].timestamp(),
            "Timestamps should be ordered"
        );
    }
}
