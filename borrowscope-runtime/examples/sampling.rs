//! Example demonstrating sampling functionality in borrowscope-runtime
//!
//! Sampling allows you to track only a percentage of events, which is useful
//! for high-volume code paths where tracking everything would be too expensive.

use borrowscope_runtime::*;

fn main() {
    println!("=== BorrowScope Sampling Demo ===\n");

    // Demo 1: Basic sampling comparison
    println!("1. Basic Sampling Comparison");
    println!("   Running 1000 iterations with different sample rates...\n");

    let rates = [0.0, 0.1, 0.5, 1.0];
    
    for rate in rates {
        reset();
        
        for i in 0..1000 {
            let _ = track_new_sampled(&format!("x{}", i), i, rate);
        }
        
        let events = get_events();
        let percentage = (events.len() as f64 / 1000.0) * 100.0;
        println!("   Sample rate {:.0}%: {} events recorded ({:.1}%)", 
            rate * 100.0, events.len(), percentage);
    }

    // Demo 2: Mixed tracking
    println!("\n2. Mixed Tracking (sampled + regular)");
    reset();
    
    // Critical variables - always track
    let config = track_new("config", "production");
    let user_id = track_new("user_id", 12345);
    
    // High-volume loop - sample at 10%
    for i in 0..100 {
        let _ = track_new_sampled(&format!("item_{}", i), i * 2, 0.1);
    }
    
    // Critical result - always track
    let result = track_new("result", "success");
    
    let events = get_events();
    println!("   Total events: {} (3 critical + ~10 sampled)", events.len());
    println!("   Config: {}, User: {}, Result: {}", config, user_id, result);

    // Demo 3: Borrow sampling
    println!("\n3. Borrow Sampling");
    reset();
    
    let data = vec![1, 2, 3, 4, 5];
    
    // Simulate hot path with many borrows
    for _ in 0..100 {
        let r = track_borrow_sampled("data_ref", &data, 0.05);
        let _ = r.len(); // Use the borrow
    }
    
    let events = get_events();
    println!("   100 borrows at 5% rate: {} events recorded", events.len());

    // Demo 4: Performance comparison
    println!("\n4. Performance Comparison");
    
    let iterations = 100_000;
    
    // Full tracking
    reset();
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _ = track_new(&format!("x{}", i), i);
    }
    let full_time = start.elapsed();
    let full_events = get_events().len();
    
    // 1% sampling
    reset();
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _ = track_new_sampled(&format!("x{}", i), i, 0.01);
    }
    let sampled_time = start.elapsed();
    let sampled_events = get_events().len();
    
    println!("   {} iterations:", iterations);
    println!("   Full tracking:  {:?}, {} events", full_time, full_events);
    println!("   1% sampling:    {:?}, {} events", sampled_time, sampled_events);
    println!("   Speedup: {:.1}x", full_time.as_nanos() as f64 / sampled_time.as_nanos() as f64);

    // Demo 5: should_sample function
    println!("\n5. Direct should_sample() Usage");
    
    let mut samples = [0usize; 5];
    let rates_to_test = [0.1, 0.25, 0.5, 0.75, 0.9];
    
    for _ in 0..10000 {
        for (i, &rate) in rates_to_test.iter().enumerate() {
            if should_sample(rate) {
                samples[i] += 1;
            }
        }
    }
    
    println!("   10000 calls per rate:");
    for (i, &rate) in rates_to_test.iter().enumerate() {
        let actual = samples[i] as f64 / 10000.0 * 100.0;
        println!("   Rate {:.0}%: {:.1}% actual", rate * 100.0, actual);
    }

    println!("\n=== Demo Complete ===");
}
