//! Tests for lifetime tracking challenges and edge cases

use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_borrow_of_borrow() {
    let _lock = TEST_LOCK.lock();

    // Simulate: let x = 42; let r1 = &x; let r2 = &r1;
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "r1_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 50,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // Should have 2 borrow relations
    assert_eq!(timeline.relations.len(), 2);

    // r2 borrows r1
    let r2_relation = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();
    assert_eq!(r2_relation.borrowed_id, "r1_0");

    // r1 borrows x
    let r1_relation = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    assert_eq!(r1_relation.borrowed_id, "x_0");

    // r2's lifetime is nested within r1's
    assert!(r2_relation.start_time > r1_relation.start_time);
    assert!(r2_relation.end_time.unwrap() < r1_relation.end_time.unwrap());
}

#[test]
fn test_multiple_independent_borrows() {
    let _lock = TEST_LOCK.lock();

    // Simulate: let x = 42; let r1 = &x; let r2 = &x;
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 15,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 25,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 40,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // Both r1 and r2 borrow x
    assert_eq!(timeline.relations.len(), 2);

    for relation in &timeline.relations {
        assert_eq!(relation.borrowed_id, "x_0");
    }

    // They overlap in time
    assert!(timeline.lifetimes_overlap("r1_0", "r2_0"));
}

#[test]
fn test_graph_lifetime_integration() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r".to_string(),
            borrower_id: "r_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 20,
            var_id: "r_0".to_string(),
        },
        Event::Drop {
            timestamp: 30,
            var_id: "x_0".to_string(),
        },
    ];

    let graph = build_graph(&events);

    // Get lifetime relations through graph
    let relations = graph.lifetime_relations(&events);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].borrower_id, "r_0");
    assert_eq!(relations[0].borrowed_id, "x_0");

    // Create timeline through graph
    let timeline = graph.create_timeline(&events);
    assert_eq!(timeline.relations.len(), 1);
    assert_eq!(timeline.min_time, 0);
    assert_eq!(timeline.max_time, 30);
}

#[test]
fn test_active_borrows_at_timestamp() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r1_0".to_string(),
        },
        Event::Borrow {
            timestamp: 35,
            borrower_name: "r3".to_string(),
            borrower_id: "r3_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r2_0".to_string(),
        },
    ];

    let graph = build_graph(&events);

    // At time 25: r1 and r2 are active
    let active_25 = graph.active_borrows_at(&events, 25);
    assert_eq!(active_25.len(), 2);

    // At time 32: only r2 is active (r1 dropped at 30)
    let active_32 = graph.active_borrows_at(&events, 32);
    assert_eq!(active_32.len(), 1);
    assert_eq!(active_32[0].borrower_id, "r2_0");

    // At time 37: r2 and r3 are active
    let active_37 = graph.active_borrows_at(&events, 37);
    assert_eq!(active_37.len(), 2);
}

#[test]
fn test_lifetimes_overlap_through_graph() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r2_0".to_string(),
        },
    ];

    let graph = build_graph(&events);

    // r1 and r2 overlap (r1: 10-30, r2: 20-40)
    assert!(graph.lifetimes_overlap(&events, "r1_0", "r2_0"));

    // Add a non-overlapping borrow
    let events_with_r3 = vec![
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 20,
            var_id: "r1_0".to_string(),
        },
        Event::Borrow {
            timestamp: 30,
            borrower_name: "r3".to_string(),
            borrower_id: "r3_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r3_0".to_string(),
        },
    ];

    let graph2 = build_graph(&events_with_r3);

    // r1 and r3 don't overlap (r1: 10-20, r3: 30-40)
    assert!(!graph2.lifetimes_overlap(&events_with_r3, "r1_0", "r3_0"));
}

#[test]
fn test_complex_nested_lifetimes() {
    let _lock = TEST_LOCK.lock();

    // Simulate complex nesting: x -> r1 -> r2 -> r3
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "r1_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 30,
            borrower_name: "r3".to_string(),
            borrower_id: "r3_0".to_string(),
            owner_id: "r2_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r3_0".to_string(),
        },
        Event::Drop {
            timestamp: 50,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 60,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 70,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // Should have 3 borrow relations
    assert_eq!(timeline.relations.len(), 3);

    // Verify the chain: r3 -> r2 -> r1 -> x
    let r3 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r3_0")
        .unwrap();
    assert_eq!(r3.borrowed_id, "r2_0");

    let r2 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();
    assert_eq!(r2.borrowed_id, "r1_0");

    let r1 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    assert_eq!(r1.borrowed_id, "x_0");

    // Verify nesting: r3 < r2 < r1
    assert!(r3.start_time > r2.start_time);
    assert!(r2.start_time > r1.start_time);
    assert!(r3.end_time.unwrap() < r2.end_time.unwrap());
    assert!(r2.end_time.unwrap() < r1.end_time.unwrap());
}

#[test]
fn test_mutable_and_immutable_borrows() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 20,
            var_id: "r1_0".to_string(),
        },
        Event::Borrow {
            timestamp: 30,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: true,
        },
        Event::Drop {
            timestamp: 40,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 50,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.relations.len(), 2);

    // Check immutable borrow
    let r1 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    assert!(!r1.is_mutable);

    // Check mutable borrow
    let r2 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();
    assert!(r2.is_mutable);

    // They don't overlap (sequential)
    assert!(!timeline.lifetimes_overlap("r1_0", "r2_0"));
}

#[test]
fn test_lifetime_with_early_drop() {
    let _lock = TEST_LOCK.lock();

    // Simulate explicit drop
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "String".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r".to_string(),
            borrower_id: "r_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 15,
            var_id: "r_0".to_string(),
        },
        Event::Drop {
            timestamp: 50,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    let r_relation = &timeline.relations[0];
    assert_eq!(r_relation.duration(), Some(5)); // 15 - 10 = 5

    // r's lifetime is much shorter than x's
    assert!(r_relation.end_time.unwrap() < 50);
}

#[test]
fn test_simultaneous_borrows_of_different_variables() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::New {
            timestamp: 5,
            var_name: "y".to_string(),
            var_id: "y_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r1".to_string(),
            borrower_id: "r1_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Borrow {
            timestamp: 15,
            borrower_name: "r2".to_string(),
            borrower_id: "r2_0".to_string(),
            owner_id: "y_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 35,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 40,
            var_id: "x_0".to_string(),
        },
        Event::Drop {
            timestamp: 45,
            var_id: "y_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.relations.len(), 2);

    // r1 borrows x
    let r1 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    assert_eq!(r1.borrowed_id, "x_0");

    // r2 borrows y
    let r2 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();
    assert_eq!(r2.borrowed_id, "y_0");

    // They overlap in time but borrow different variables
    assert!(r1.overlaps_with(r2));
}

#[test]
fn test_empty_events_timeline() {
    let _lock = TEST_LOCK.lock();

    let events: Vec<Event> = vec![];
    let graph = build_graph(&events);

    let relations = graph.lifetime_relations(&events);
    assert_eq!(relations.len(), 0);

    let timeline = graph.create_timeline(&events);
    assert_eq!(timeline.relations.len(), 0);
    assert_eq!(timeline.total_duration(), 0);
}

#[test]
fn test_borrow_without_drop() {
    let _lock = TEST_LOCK.lock();

    // Borrow that never gets dropped (still active)
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Borrow {
            timestamp: 10,
            borrower_name: "r".to_string(),
            borrower_id: "r_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
    ];

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.relations.len(), 1);

    let relation = &timeline.relations[0];
    assert!(relation.is_active());
    assert_eq!(relation.end_time, None);
    assert_eq!(relation.duration(), None);
}
