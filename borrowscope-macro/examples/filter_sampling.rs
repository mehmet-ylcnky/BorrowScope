//! Example demonstrating filter and sampling features
//!
//! Filter allows tracking only variables matching a pattern.
//! Sampling allows probabilistic tracking for high-volume code.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};

// Filter: only track variables starting with "data"
#[trace_borrow(filter = "data*")]
fn process_with_filter() {
    let data_input = vec![1, 2, 3];      // Tracked (matches "data*")
    let data_output = vec![4, 5, 6];     // Tracked (matches "data*")
    let temp = 42;                        // NOT tracked (doesn't match)
    let result = temp + 1;                // NOT tracked
    
    println!("data_input: {:?}", data_input);
    println!("data_output: {:?}", data_output);
    println!("result: {}", result);
}

// Sample: track only ~10% of calls (useful for hot paths)
#[trace_borrow(sample = 0.1)]
fn process_with_sampling() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!("z = {}", z);
}

// Combine filter with other options
#[trace_borrow(debug_only, filter = "user*")]
fn process_user_data() {
    let user_id = 123;
    let user_name = "Alice";
    let internal_counter = 0;  // NOT tracked
    
    println!("User {} ({}), counter: {}", user_name, user_id, internal_counter);
}

fn main() {
    println!("=== Filter Example ===");
    reset();
    process_with_filter();
    let events = get_events();
    println!("Events captured: {} (expected 4: 2 New + 2 Drop for data_* vars)", events.len());
    for event in &events {
        println!("  {:?}", event);
    }
    
    println!("\n=== Sampling Example (10% rate) ===");
    println!("Running 100 iterations...");
    let mut total_events = 0;
    for _ in 0..100 {
        reset();
        process_with_sampling();
        total_events += get_events().len();
    }
    // With 10% sampling, 3 vars * 2 events (new+drop) * 100 iterations * 0.1 = ~60 events
    println!("Total events across 100 runs: {} (expected ~60 with 10% sampling)", total_events);
    
    println!("\n=== Combined Filter + Debug Only ===");
    reset();
    process_user_data();
    let events = get_events();
    println!("Events captured: {} (expected 4: 2 New + 2 Drop for user_* vars)", events.len());
    for event in &events {
        println!("  {:?}", event);
    }
}
