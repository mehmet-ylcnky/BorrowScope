//! Comprehensive tests for lifetime tracking and inference

use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_simple_lifetime_relation() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Simulate: let x = 42; let r = &x;
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

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.relations.len(), 1);
    assert_eq!(timeline.min_time, 0);
    assert_eq!(timeline.max_time, 30);

    let relation = &timeline.relations[0];
    assert_eq!(relation.borrower_id, "r_0");
    assert_eq!(relation.borrowed_id, "x_0");
    assert_eq!(relation.start_time, 10);
    assert_eq!(relation.end_time, Some(20));
    assert!(!relation.is_mutable);
}

#[test]
fn test_multiple_immutable_borrows() {
    let _lock = TEST_LOCK.lock();
    reset();

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

    assert_eq!(timeline.relations.len(), 2);

    // Both borrows should overlap
    assert!(timeline.lifetimes_overlap("r1_0", "r2_0"));

    // Check that both borrow from x
    let r1_relations = timeline.relations_for("r1_0");
    let r2_relations = timeline.relations_for("r2_0");

    assert_eq!(r1_relations.len(), 1);
    assert_eq!(r2_relations.len(), 1);
    assert_eq!(r1_relations[0].borrowed_id, "x_0");
    assert_eq!(r2_relations[0].borrowed_id, "x_0");
}

#[test]
fn test_nested_scopes() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Simulate:
    // let x = 42;
    // let r1 = &x;
    // { let r2 = &x; } // r2 dropped
    // // r1 still valid
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
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 50,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 60,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // r2's lifetime should be nested within r1's
    let r1 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    let r2 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();

    assert!(r1.start_time < r2.start_time);
    assert!(r1.end_time.unwrap() > r2.end_time.unwrap());

    // At time 25, both should be active
    let active_at_25 = timeline.active_at(25);
    assert_eq!(active_at_25.len(), 2);

    // At time 40, only r1 should be active
    let active_at_40 = timeline.active_at(40);
    assert_eq!(active_at_40.len(), 1);
    assert_eq!(active_at_40[0].borrower_id, "r1_0");
}

#[test]
fn test_mutable_borrow() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Simulate: let mut x = 42; let r = &mut x;
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
            mutable: true,
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

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.relations.len(), 1);

    let relation = &timeline.relations[0];
    assert!(relation.is_mutable);
    assert_eq!(relation.duration(), Some(10));
}

#[test]
fn test_lifetime_duration_calculation() {
    let _lock = TEST_LOCK.lock();

    let mut relation = LifetimeRelation::new("r".to_string(), "x".to_string(), 100, false);

    // Active lifetime has no duration
    assert_eq!(relation.duration(), None);
    assert!(relation.is_active());

    // Set end time
    relation.end_time = Some(250);
    assert_eq!(relation.duration(), Some(150));
    assert!(!relation.is_active());
}

#[test]
fn test_lifetime_overlap_detection() {
    let _lock = TEST_LOCK.lock();

    let r1 = LifetimeRelation {
        borrower_id: "r1".to_string(),
        borrowed_id: "x".to_string(),
        start_time: 100,
        end_time: Some(200),
        is_mutable: false,
    };

    let r2 = LifetimeRelation {
        borrower_id: "r2".to_string(),
        borrowed_id: "x".to_string(),
        start_time: 150,
        end_time: Some(250),
        is_mutable: false,
    };

    let r3 = LifetimeRelation {
        borrower_id: "r3".to_string(),
        borrowed_id: "x".to_string(),
        start_time: 300,
        end_time: Some(400),
        is_mutable: false,
    };

    // r1 and r2 overlap
    assert!(r1.overlaps_with(&r2));
    assert!(r2.overlaps_with(&r1));

    // r1 and r3 don't overlap
    assert!(!r1.overlaps_with(&r3));
    assert!(!r3.overlaps_with(&r1));

    // r2 and r3 don't overlap
    assert!(!r2.overlaps_with(&r3));
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
        Event::Borrow {
            timestamp: 30,
            borrower_name: "r3".to_string(),
            borrower_id: "r3_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 25,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 35,
            var_id: "r2_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // At time 15: only r1
    let active_15 = timeline.active_at(15);
    assert_eq!(active_15.len(), 1);
    assert_eq!(active_15[0].borrower_id, "r1_0");

    // At time 22: r1 and r2
    let active_22 = timeline.active_at(22);
    assert_eq!(active_22.len(), 2);

    // At time 32: r2 and r3
    let active_32 = timeline.active_at(32);
    assert_eq!(active_32.len(), 2);

    // At time 40: only r3 (still active, no drop event)
    let active_40 = timeline.active_at(40);
    assert_eq!(active_40.len(), 1);
    assert_eq!(active_40[0].borrower_id, "r3_0");
}

#[test]
fn test_timeline_total_duration() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
        Event::New {
            timestamp: 100,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        },
        Event::Drop {
            timestamp: 500,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    assert_eq!(timeline.min_time, 100);
    assert_eq!(timeline.max_time, 500);
    assert_eq!(timeline.total_duration(), 400);
}

#[test]
fn test_empty_timeline() {
    let _lock = TEST_LOCK.lock();

    let timeline = Timeline::from_events(&[]);

    assert_eq!(timeline.relations.len(), 0);
    assert_eq!(timeline.min_time, 0);
    assert_eq!(timeline.max_time, 0);
    assert_eq!(timeline.total_duration(), 0);
}

#[test]
fn test_relations_for_variable() {
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
            owner_id: "y_0".to_string(),
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

    let timeline = Timeline::from_events(&events);

    // Get relations for x_0
    let x_relations = timeline.relations_for("x_0");
    assert_eq!(x_relations.len(), 1);
    assert_eq!(x_relations[0].borrowed_id, "x_0");

    // Get relations for y_0
    let y_relations = timeline.relations_for("y_0");
    assert_eq!(y_relations.len(), 1);
    assert_eq!(y_relations[0].borrowed_id, "y_0");

    // Get relations for r1_0
    let r1_relations = timeline.relations_for("r1_0");
    assert_eq!(r1_relations.len(), 1);
    assert_eq!(r1_relations[0].borrower_id, "r1_0");
}

#[test]
fn test_complex_lifetime_scenario() {
    let _lock = TEST_LOCK.lock();

    // Simulate complex scenario with multiple variables and borrows
    let events = vec![
        Event::New {
            timestamp: 0,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "String".to_string(),
        },
        Event::New {
            timestamp: 5,
            var_name: "y".to_string(),
            var_id: "y_0".to_string(),
            type_name: "String".to_string(),
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
        Event::Borrow {
            timestamp: 20,
            borrower_name: "r3".to_string(),
            borrower_id: "r3_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        },
        Event::Drop {
            timestamp: 25,
            var_id: "r3_0".to_string(),
        },
        Event::Drop {
            timestamp: 30,
            var_id: "r2_0".to_string(),
        },
        Event::Drop {
            timestamp: 35,
            var_id: "r1_0".to_string(),
        },
        Event::Drop {
            timestamp: 40,
            var_id: "y_0".to_string(),
        },
        Event::Drop {
            timestamp: 45,
            var_id: "x_0".to_string(),
        },
    ];

    let timeline = Timeline::from_events(&events);

    // Should have 3 borrow relations
    assert_eq!(timeline.relations.len(), 3);

    // r1 and r3 both borrow from x, so they should overlap
    assert!(timeline.lifetimes_overlap("r1_0", "r3_0"));

    // r1 and r2 borrow from different variables but overlap in time
    let r1 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r1_0")
        .unwrap();
    let r2 = timeline
        .relations
        .iter()
        .find(|r| r.borrower_id == "r2_0")
        .unwrap();
    assert!(r1.overlaps_with(r2));

    // At time 22, r1, r2, and r3 should all be active
    let active_22 = timeline.active_at(22);
    assert_eq!(active_22.len(), 3);
}

#[test]
fn test_elision_rules() {
    let _lock = TEST_LOCK.lock();

    // Test that elision rules have descriptions
    assert!(!ElisionRule::EachInputOwn.description().is_empty());
    assert!(!ElisionRule::SingleInputToOutput.description().is_empty());
    assert!(!ElisionRule::SelfToOutput.description().is_empty());

    // Test equality
    assert_eq!(ElisionRule::EachInputOwn, ElisionRule::EachInputOwn);
    assert_ne!(ElisionRule::EachInputOwn, ElisionRule::SingleInputToOutput);
}

#[test]
fn test_still_active_borrows() {
    let _lock = TEST_LOCK.lock();

    // Simulate borrows that are never dropped (still active)
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
    ];

    let timeline = Timeline::from_events(&events);

    // Both borrows should be in the timeline
    assert_eq!(timeline.relations.len(), 2);

    // Both should be active (no end_time)
    for relation in &timeline.relations {
        assert!(relation.is_active());
        assert_eq!(relation.end_time, None);
        assert_eq!(relation.duration(), None);
    }
}

#[test]
fn test_serialization() {
    let _lock = TEST_LOCK.lock();

    let relation = LifetimeRelation::new("r".to_string(), "x".to_string(), 100, false);

    // Test that it can be serialized
    let json = serde_json::to_string(&relation).unwrap();
    assert!(!json.is_empty());

    // Test that it can be deserialized
    let deserialized: LifetimeRelation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, relation);
}

#[test]
fn test_timeline_serialization() {
    let _lock = TEST_LOCK.lock();

    let events = vec![
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
    ];

    let timeline = Timeline::from_events(&events);

    // Test serialization
    let json = serde_json::to_string(&timeline).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: Timeline = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.relations.len(), timeline.relations.len());
    assert_eq!(deserialized.min_time, timeline.min_time);
    assert_eq!(deserialized.max_time, timeline.max_time);
}
