//! Example demonstrating OnceCell and OnceLock tracking
//!
//! Run with: cargo run -p borrowscope-runtime --features track --example once_cell_tracking

use borrowscope_runtime::*;
use std::cell::OnceCell;
use std::sync::OnceLock;

fn main() {
    reset();
    println!("=== OnceCell/OnceLock Tracking Demo ===\n");

    // OnceCell (single-threaded)
    println!("--- OnceCell (single-threaded) ---");
    let cell: OnceCell<String> = track_once_cell_new("config", "main:1", OnceCell::new());

    // Try to get before initialization
    let value = track_once_cell_get("config", "main:2", cell.get());
    println!("Before init: {:?}", value);

    // Set the value
    let result = track_once_cell_set("config", "main:3", cell.set(String::from("initialized")));
    println!("Set result: {:?}", result);

    // Get after initialization
    let value = track_once_cell_get("config", "main:4", cell.get());
    println!("After init: {:?}", value);

    // Try to set again (will fail)
    let result = track_once_cell_set("config", "main:5", cell.set(String::from("second")));
    println!("Second set result: {:?}", result);

    // OnceLock (thread-safe)
    println!("\n--- OnceLock (thread-safe) ---");
    let lock: OnceLock<i32> = track_once_lock_new("counter", "main:6", OnceLock::new());

    let _ = track_once_cell_set("counter", "main:7", lock.set(42));
    let value = track_once_cell_get("counter", "main:8", lock.get());
    println!("OnceLock value: {:?}", value);

    // get_or_init pattern
    println!("\n--- get_or_init pattern ---");
    let lazy: OnceCell<Vec<i32>> = OnceCell::new();
    let was_init = lazy.get().is_some();
    let data = lazy.get_or_init(|| vec![1, 2, 3]);
    let _ = track_once_cell_get_or_init("lazy_data", was_init, "main:9", data);
    println!("Lazy initialized: {:?}", data);

    // Print events
    println!("\n=== Tracked Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }
}
