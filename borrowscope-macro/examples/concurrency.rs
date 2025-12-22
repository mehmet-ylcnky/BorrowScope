//! Example: Concurrency Tracking with Macro
//!
//! Demonstrates automatic instrumentation of thread::spawn and mpsc channels.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    reset();
    println!("=== Concurrency Tracking (Macro) ===\n");

    example_thread_spawn();
    example_channel();

    println!("\n=== All Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}

#[trace_borrow]
fn example_thread_spawn() {
    println!("--- Thread Spawn Tracking ---");

    let handle = thread::spawn(|| {
        println!("  [thread] Hello from spawned thread!");
        thread::sleep(Duration::from_millis(10));
        42
    });

    println!("Spawned thread, waiting for result...");
    let result = handle.join().unwrap();
    println!("Thread returned: {}", result);
}

#[trace_borrow]
fn example_channel() {
    println!("\n--- Channel Tracking ---");

    let (tx, rx) = mpsc::channel();

    // Spawn sender thread
    let sender = thread::spawn(move || {
        for i in 1..=3 {
            tx.send(i * 10).unwrap();
            println!("  [sender] Sent: {}", i * 10);
        }
    });

    // Receive in main thread
    for _ in 0..3 {
        let value = rx.recv().unwrap();
        println!("  [receiver] Received: {}", value);
    }

    sender.join().unwrap();
    println!("Channel communication complete");
}
