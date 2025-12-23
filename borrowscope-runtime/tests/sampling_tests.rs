//! Tests for sampling functionality in borrowscope-runtime
//!
//! Note: These tests share global state and should be run with --test-threads=1
//! or use the serial_test crate for proper isolation.

use borrowscope_runtime::*;

#[test]
fn test_should_sample_always_true_at_rate_1() {
    // At rate 1.0, should always sample
    for _ in 0..100 {
        assert!(should_sample(1.0));
    }
}

#[test]
fn test_should_sample_always_false_at_rate_0() {
    // At rate 0.0, should never sample
    for _ in 0..100 {
        assert!(!should_sample(0.0));
    }
}

#[test]
fn test_should_sample_probabilistic() {
    // At rate 0.5, should sample roughly half the time
    // Use a large sample size for statistical significance
    let mut sampled = 0;
    let iterations = 10000;
    
    for _ in 0..iterations {
        if should_sample(0.5) {
            sampled += 1;
        }
    }
    
    // Allow 20% tolerance (40-60% range)
    let ratio = sampled as f64 / iterations as f64;
    assert!(ratio > 0.3 && ratio < 0.7, 
        "Expected ~50% sampling, got {}%", ratio * 100.0);
}

#[test]
fn test_track_new_sampled_returns_value() {
    reset();
    
    // Value should always be returned regardless of sampling
    let x = track_new_sampled("x", 42, 0.0);
    assert_eq!(x, 42);
    
    let y = track_new_sampled("y", "hello", 1.0);
    assert_eq!(y, "hello");
    
    let v = track_new_sampled("v", vec![1, 2, 3], 0.5);
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn test_track_new_sampled_at_rate_0_no_events() {
    reset();
    
    // At rate 0.0, no events should be recorded
    for i in 0..100 {
        let _ = track_new_sampled(&format!("x{}", i), i, 0.0);
    }
    
    let events = get_events();
    assert_eq!(events.len(), 0, "Expected no events at sample rate 0.0");
}

#[test]
fn test_track_new_sampled_at_rate_1_all_events() {
    reset();
    
    // At rate 1.0, all events should be recorded
    for i in 0..10 {
        let _ = track_new_sampled(&format!("x{}", i), i, 1.0);
    }
    
    let events = get_events();
    assert_eq!(events.len(), 10, "Expected all 10 events at sample rate 1.0");
}

#[test]
fn test_track_new_sampled_probabilistic() {
    reset();
    
    // At rate 0.1, should record roughly 10% of events
    let iterations = 1000;
    for i in 0..iterations {
        let _ = track_new_sampled(&format!("x{}", i), i, 0.1);
    }
    
    let events = get_events();
    let ratio = events.len() as f64 / iterations as f64;
    
    // Allow wide tolerance for probabilistic test
    assert!(ratio > 0.02 && ratio < 0.25, 
        "Expected ~10% events, got {}% ({} events)", ratio * 100.0, events.len());
}

#[test]
fn test_track_borrow_sampled() {
    reset();
    
    let data = vec![1, 2, 3];
    
    // At rate 0.0, no borrow events
    for _ in 0..10 {
        let _ = track_borrow_sampled("r", &data, 0.0);
    }
    assert_eq!(get_events().len(), 0);
    
    reset();
    
    // At rate 1.0, all borrow events
    for _ in 0..10 {
        let _ = track_borrow_sampled("r", &data, 1.0);
    }
    assert_eq!(get_events().len(), 10);
}

#[test]
fn test_track_borrow_mut_sampled() {
    reset();
    
    let mut data = vec![1, 2, 3];
    
    // At rate 1.0, should record
    let r = track_borrow_mut_sampled("r", &mut data, 1.0);
    r.push(4);
    
    let events = get_events();
    // Should have exactly 1 borrow event
    let borrow_events: Vec<_> = events.iter().filter(|e| e.is_borrow()).collect();
    assert_eq!(borrow_events.len(), 1, "Expected 1 borrow event, got {}", borrow_events.len());
}

#[test]
fn test_track_drop_sampled() {
    reset();
    
    // At rate 0.0, no drop events
    for _ in 0..10 {
        track_drop_sampled("x", 0.0);
    }
    assert_eq!(get_events().len(), 0);
    
    reset();
    
    // At rate 1.0, all drop events
    for _ in 0..10 {
        track_drop_sampled("x", 1.0);
    }
    assert_eq!(get_events().len(), 10);
}

#[test]
fn test_track_move_sampled() {
    reset();
    
    // At rate 0.0, no move events but value is returned
    let x = track_move_sampled("a", "b", 42, 0.0);
    assert_eq!(x, 42);
    assert_eq!(get_events().len(), 0);
    
    reset();
    
    // At rate 1.0, move event is recorded
    let y = track_move_sampled("a", "b", 42, 1.0);
    assert_eq!(y, 42);
    assert_eq!(get_events().len(), 1);
}

#[test]
fn test_track_new_with_id_sampled() {
    reset();
    
    // At rate 0.0, no events
    let _ = track_new_with_id_sampled(1, "x", "test.rs:1", 42, 0.0);
    assert_eq!(get_events().len(), 0);
    
    reset();
    
    // At rate 1.0, event is recorded with location
    let _ = track_new_with_id_sampled(1, "x", "test.rs:1", 42, 1.0);
    let events = get_events();
    assert_eq!(events.len(), 1);
    
    if let Event::New { var_name, type_name, .. } = &events[0] {
        assert_eq!(var_name, "x");
        assert!(type_name.contains("test.rs:1"));
    } else {
        panic!("Expected New event");
    }
}

#[test]
fn test_sampling_mixed_with_regular_tracking() {
    reset();
    
    // Mix sampled and regular tracking
    let _ = track_new("a", 1);                           // Always tracked
    let _ = track_new_sampled("b", 2, 0.0);              // Never tracked
    let _ = track_new("c", 3);                           // Always tracked
    let _ = track_new_sampled("d", 4, 1.0);              // Always tracked
    
    let events = get_events();
    assert_eq!(events.len(), 3); // a, c, d
    
    let names: Vec<_> = events.iter().filter_map(|e| {
        if let Event::New { var_name, .. } = e {
            Some(var_name.as_str())
        } else {
            None
        }
    }).collect();
    
    assert!(names.contains(&"a"));
    assert!(!names.contains(&"b"));
    assert!(names.contains(&"c"));
    assert!(names.contains(&"d"));
}

#[test]
fn test_sampling_thread_safety() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    reset();
    
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    // Spawn multiple threads doing sampled tracking
    for t in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let name = format!("t{}_{}", t, i);
                let _ = track_new_sampled(&name, i, 0.5);
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    
    for h in handles {
        h.join().unwrap();
    }
    
    // All 400 calls completed
    assert_eq!(counter.load(Ordering::Relaxed), 400);
    
    // Some events were recorded (probabilistic)
    let events = get_events();
    assert!(events.len() > 50 && events.len() < 350,
        "Expected ~200 events (50%), got {}", events.len());
}
