//! Example: Pin Tracking
//!
//! Demonstrates tracking of Pin operations for memory pinning,
//! commonly used with async/await and self-referential structs.

use borrowscope_runtime::*;
use std::pin::Pin;

fn main() {
    reset();
    println!("=== Pin Tracking Example ===\n");

    // Box::pin - most common way to pin heap data
    println!("--- Box::pin ---");
    let pinned_int = track_pin_new("pinned_int", "main.rs:13", Box::pin(42));
    println!("Pinned integer: {}", *pinned_int);

    let pinned_string = track_pin_new(
        "pinned_string",
        "main.rs:16",
        Box::pin(String::from("pinned!")),
    );
    println!("Pinned string: {}", &*pinned_string);

    // Pin with struct
    println!("\n--- Pinned Struct ---");
    #[derive(Debug)]
    struct Data {
        value: i32,
        name: String,
    }

    let pinned_struct = track_pin_new(
        "pinned_struct",
        "main.rs:27",
        Box::pin(Data {
            value: 100,
            name: String::from("important"),
        }),
    );
    println!("Pinned struct: {:?}", &*pinned_struct);

    // Pin::new with Box
    println!("\n--- Pin::new with Box ---");
    let boxed = Box::new(vec![1, 2, 3]);
    let pinned_box = track_pin_new("pinned_box", "main.rs:39", Pin::new(boxed));
    println!("Pinned box contents: {:?}", &*pinned_box);

    // Pin::into_inner - extract the inner value
    println!("\n--- Pin::into_inner ---");
    let pinned = Box::pin(String::from("extract me"));
    let pinned = track_pin_new("to_extract", "main.rs:45", pinned);
    println!("Before extraction: {}", &*pinned);

    let extracted = track_pin_into_inner("to_extract", "main.rs:48", Pin::into_inner(pinned));
    println!("After extraction: {}", &*extracted);

    // Multiple pinned values
    println!("\n--- Multiple Pinned Values ---");
    let pins: Vec<Pin<Box<i32>>> = (1..=5)
        .map(|i| track_pin_new(&format!("pin_{}", i), "main.rs:54", Box::pin(i * 10)))
        .collect();

    let sum: i32 = pins.iter().map(|p| **p).sum();
    println!("Sum of pinned values: {}", sum);

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
