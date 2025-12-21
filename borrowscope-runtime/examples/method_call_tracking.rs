//! Method call tracking example
//!
//! Demonstrates tracking clone, lock, and unwrap operations.
//!
//! Run with: cargo run --example method_call_tracking --features track

use borrowscope_runtime::*;

fn main() {
    reset();
    println!("=== Method Call Tracking Example ===\n");

    // Clone tracking
    println!("--- Clone Tracking ---");
    track_clone(1, "data", "main.rs:14");
    println!("  Tracked clone of 'data'");

    track_clone(2, "config", "main.rs:17");
    println!("  Tracked clone of 'config'");

    // Lock tracking (Mutex)
    println!("\n--- Mutex Lock Tracking ---");
    track_lock(1, "mutex", "counter", "main.rs:22");
    println!("  Tracked mutex.lock() on 'counter'");

    track_lock(2, "mutex_try", "state", "main.rs:25");
    println!("  Tracked mutex.try_lock() on 'state'");

    // Lock tracking (RwLock)
    println!("\n--- RwLock Tracking ---");
    track_lock(3, "rwlock_read", "cache", "main.rs:30");
    println!("  Tracked rwlock.read() on 'cache'");

    track_lock(4, "rwlock_write", "cache", "main.rs:33");
    println!("  Tracked rwlock.write() on 'cache'");

    // Unwrap tracking
    println!("\n--- Unwrap Tracking ---");
    track_unwrap(1, "unwrap", "option_value", "main.rs:38");
    println!("  Tracked option.unwrap()");

    track_unwrap(2, "expect", "result_value", "main.rs:41");
    println!("  Tracked result.expect()");

    track_unwrap(3, "unwrap_or", "fallback", "main.rs:44");
    println!("  Tracked option.unwrap_or()");

    track_unwrap(4, "unwrap_or_default", "default_val", "main.rs:47");
    println!("  Tracked option.unwrap_or_default()");

    // Print events
    println!("\n=== Recorded Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }

    // JSON export
    println!("\n=== JSON Export ===");
    println!("{}", serde_json::to_string_pretty(&get_events()).unwrap());
}
