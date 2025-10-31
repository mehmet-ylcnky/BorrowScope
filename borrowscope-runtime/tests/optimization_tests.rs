use borrowscope_runtime::{get_events, reset, track_drop, track_drop_batch, track_new};
use std::time::Instant;

lazy_static::lazy_static! {
    /// Global test lock to ensure tests run serially when accessing shared tracker
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_track_new_zero_overhead_without_feature() {
    let _lock = TEST_LOCK.lock();
    // When compiled without "track" feature, should have minimal overhead
    reset();

    let start = Instant::now();
    for i in 0..10_000 {
        let x = track_new("x", i);
        std::hint::black_box(x);
    }
    let duration = start.elapsed();

    #[cfg(not(feature = "track"))]
    {
        // Without tracking, events should be empty
        let events = get_events();
        assert_eq!(events.len(), 0);
    }

    #[cfg(feature = "track")]
    {
        // With tracking, events should be recorded
        let events = get_events();
        assert_eq!(events.len(), 10_000);
    }

    println!("10K operations took: {:?}", duration);
}

#[test]
fn test_batch_drop_optimization() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Test individual drops
    let start = Instant::now();
    for _ in 0..1_000 {
        track_drop("x");
        track_drop("y");
        track_drop("z");
    }
    let individual_duration = start.elapsed();

    reset();

    // Test batch drops
    let start = Instant::now();
    for _ in 0..1_000 {
        track_drop_batch(&["x", "y", "z"]);
    }
    let batch_duration = start.elapsed();

    println!("Individual drops: {:?}", individual_duration);
    println!("Batch drops: {:?}", batch_duration);

    #[cfg(feature = "track")]
    {
        // Batch should be faster or similar
        assert!(
            batch_duration <= individual_duration * 2,
            "Batch drops should not be significantly slower"
        );
    }
}

#[test]
fn test_inline_optimization() {
    let _lock = TEST_LOCK.lock();
    // Test that inline(always) allows compiler optimization
    reset();

    let x = track_new("x", 42);
    assert_eq!(x, 42);

    let y = track_new("y", "hello");
    assert_eq!(y, "hello");

    let z = track_new("z", vec![1, 2, 3]);
    assert_eq!(z, vec![1, 2, 3]);
}

#[test]
fn test_memory_usage() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Track memory usage for large number of events
    let count = 10_000;
    for i in 0..count {
        track_new(&format!("var_{}", i), i);
    }

    let events = get_events();

    #[cfg(feature = "track")]
    {
        assert_eq!(events.len(), count);
        let memory_bytes = events.len() * std::mem::size_of_val(&events[0]);
        println!("Memory for {} events: {} bytes", count, memory_bytes);
        println!("Per event: {} bytes", memory_bytes / count);
    }

    #[cfg(not(feature = "track"))]
    {
        assert_eq!(events.len(), 0);
    }
}

#[test]
fn test_string_allocation_efficiency() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Test with static strings (should be efficient)
    let x = track_new("x", 42);
    let y = track_new("y", 43);
    let z = track_new("z", 44);

    std::hint::black_box((x, y, z));

    #[cfg(feature = "track")]
    {
        let events = get_events();
        assert_eq!(events.len(), 3);
    }
}

#[test]
fn test_per_operation_overhead() {
    let _lock = TEST_LOCK.lock();
    reset();

    let iterations = 1_000_000;

    // Measure baseline
    let start = Instant::now();
    for i in 0..iterations {
        let x = i;
        std::hint::black_box(x);
    }
    let baseline = start.elapsed();

    reset();

    // Measure with tracking
    let start = Instant::now();
    for i in 0..iterations {
        let x = track_new("x", i);
        std::hint::black_box(x);
    }
    let with_tracking = start.elapsed();

    let overhead = with_tracking.saturating_sub(baseline);
    let per_op = overhead.as_nanos() / iterations as u128;

    println!("Baseline: {:?}", baseline);
    println!("With tracking: {:?}", with_tracking);
    println!("Overhead: {:?}", overhead);
    println!("Per operation: {}ns", per_op);

    #[cfg(feature = "track")]
    {
        // Goal: <50ns per operation when tracking is enabled
        // This is a soft target and may vary by system
        println!("Overhead per operation: {}ns (target: <50ns)", per_op);
    }

    #[cfg(not(feature = "track"))]
    {
        // Without tracking, overhead should be minimal
        assert!(
            per_op < 5,
            "Without tracking, overhead should be <5ns, got {}ns",
            per_op
        );
    }
}

#[test]
fn test_conditional_compilation() {
    let _lock = TEST_LOCK.lock();
    reset();

    let x = track_new("x", 42);
    track_drop("x");

    let events = get_events();

    #[cfg(feature = "track")]
    {
        assert!(
            !events.is_empty(),
            "Events should be recorded with track feature"
        );
    }

    #[cfg(not(feature = "track"))]
    {
        assert!(
            events.is_empty(),
            "Events should not be recorded without track feature"
        );
    }

    std::hint::black_box(x);
}

#[test]
fn test_type_name_efficiency() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Test with various types
    let _a = track_new("a", 42i32);
    let _b = track_new("b", "hello");
    let _c = track_new("c", vec![1, 2, 3]);
    let _d = track_new("d", Some(42));
    let _e = track_new("e", (1, 2, 3));

    #[cfg(feature = "track")]
    {
        let events = get_events();
        assert_eq!(events.len(), 5);
    }
}

#[test]
fn test_batch_vs_individual_correctness() {
    let _lock = TEST_LOCK.lock();
    reset();

    // Individual drops
    track_drop("x");
    track_drop("y");
    track_drop("z");

    #[cfg(feature = "track")]
    {
        let events1 = get_events();
        reset();

        // Batch drops
        track_drop_batch(&["x", "y", "z"]);
        let events2 = get_events();

        // Both should produce same number of events
        assert_eq!(events1.len(), events2.len());
    }
}

#[test]
fn test_no_code_bloat() {
    let _lock = TEST_LOCK.lock();
    // Ensure tracking calls don't prevent optimization
    reset();

    let x = track_new("x", 42);
    let y = track_new("y", x + 1);
    let z = track_new("z", y * 2);

    assert_eq!(z, 86);

    // Compiler should be able to optimize this
    let result = track_new("result", 2 + 2);
    assert_eq!(result, 4);
}
