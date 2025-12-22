//! Example demonstrating MaybeUninit tracking
//!
//! Run with: cargo run -p borrowscope-runtime --features track --example maybe_uninit_tracking

use borrowscope_runtime::*;
use std::mem::MaybeUninit;

fn main() {
    reset();
    println!("=== MaybeUninit Tracking Demo ===\n");

    // Create uninitialized memory
    println!("--- Creating uninitialized memory ---");
    let mut uninit: MaybeUninit<i32> = track_maybe_uninit_uninit("data", "main:1", MaybeUninit::uninit());
    println!("Created uninitialized MaybeUninit<i32>");

    // Write a value
    println!("\n--- Writing value ---");
    let written = track_maybe_uninit_write("data", "main:2", uninit.write(42));
    println!("Wrote value: {}", written);

    // Assume init and use
    println!("\n--- Assuming initialized ---");
    let value = track_maybe_uninit_assume_init("data", "main:3", unsafe { uninit.assume_init() });
    println!("Value after assume_init: {}", value);

    // Create with initial value
    println!("\n--- Creating with initial value ---");
    let init: MaybeUninit<String> = track_maybe_uninit_new(
        "message",
        "main:4",
        MaybeUninit::new(String::from("Hello, World!")),
    );
    let msg = track_maybe_uninit_assume_init("message", "main:5", unsafe { init.assume_init() });
    println!("Message: {}", msg);

    // Array initialization pattern
    println!("\n--- Array initialization pattern ---");
    let mut arr: [MaybeUninit<i32>; 3] = unsafe { MaybeUninit::uninit().assume_init() };
    for (i, elem) in arr.iter_mut().enumerate() {
        let _ = track_maybe_uninit_write(&format!("arr[{}]", i), "main:6", elem.write((i * 10) as i32));
    }
    println!("Initialized array elements");

    // Read array values
    let values: Vec<i32> = arr.iter().map(|e| unsafe { e.assume_init_read() }).collect();
    println!("Array values: {:?}", values);

    // Dropping initialized value
    println!("\n--- Dropping initialized value ---");
    let mut to_drop: MaybeUninit<Vec<i32>> = MaybeUninit::new(vec![1, 2, 3, 4, 5]);
    unsafe { to_drop.assume_init_drop() };
    track_maybe_uninit_assume_init_drop("to_drop", "main:7");
    println!("Dropped Vec");

    // Print events
    println!("\n=== Tracked Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }
}
