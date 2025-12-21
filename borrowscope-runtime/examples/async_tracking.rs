//! Async tracking example
//!
//! Demonstrates tracking async blocks and await expressions.
//!
//! Run with: cargo run --example async_tracking --features track

use borrowscope_runtime::*;

fn main() {
    reset();

    println!("=== Async Tracking Example ===\n");

    // Simulate async block tracking
    println!("Tracking async block...");
    track_async_block_enter(1, "main.rs:15:5");

    // Track variable inside async block
    let _data = track_new("data", vec![1, 2, 3]);

    // Simulate await tracking
    println!("Tracking await expression...");
    track_await_start(1, "fetch_data", "main.rs:20:9");
    // ... simulated await completion ...
    track_await_end(1, "main.rs:20:9");

    // Another await
    track_await_start(2, "process_result", "main.rs:25:9");
    track_await_end(2, "main.rs:25:9");

    track_drop("data");
    track_async_block_exit(1, "main.rs:30:5");

    // Print events
    println!("\n=== Recorded Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }

    // Export to JSON
    println!("\n=== JSON Export ===");
    println!("{}", serde_json::to_string_pretty(&get_events()).unwrap());
}
