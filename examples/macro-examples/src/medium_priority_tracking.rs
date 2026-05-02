//! Example: Medium Priority Tracking Features
//!
//! Demonstrates automatic instrumentation of:
//! - Lock guard acquire/drop (Mutex, RwLock)
//! - Closure create/capture
//! - Box::into_raw/from_raw

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};
use std::sync::{Arc, Mutex, RwLock};

fn main() {
    println!("=== Medium Priority Tracking Features ===\n");

    example_lock_guards();
    example_closures();
    example_box_raw();

    println!("\n=== All Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}

#[trace_borrow]
fn example_lock_guards() {
    reset();
    println!("--- Lock Guard Tracking ---");

    // Mutex lock
    let mutex = Mutex::new(42);
    {
        let guard = mutex.lock().unwrap();
        println!("Mutex locked, value: {}", *guard);
        // guard drops here
    }
    println!("Mutex unlocked");

    // RwLock read
    let rwlock = RwLock::new(100);
    {
        let read_guard = rwlock.read().unwrap();
        println!("RwLock read locked, value: {}", *read_guard);
    }

    // RwLock write
    {
        let mut write_guard = rwlock.write().unwrap();
        *write_guard = 200;
        println!("RwLock write locked, new value: {}", *write_guard);
    }

    println!("\nLock events:");
    for event in get_events().iter().filter(|e| {
        let s = format!("{:?}", e);
        s.contains("Lock") || s.contains("Guard")
    }) {
        println!("  {:?}", event);
    }
}

#[trace_borrow]
fn example_closures() {
    reset();
    println!("\n--- Closure Tracking ---");

    let x = 10;
    let y = 20;

    // Non-move closure (borrows)
    let add = |a: i32| a + x + y;
    println!("Non-move closure result: {}", add(5));

    // Move closure
    let data = vec![1, 2, 3];
    let sum_closure = move || data.iter().sum::<i32>();
    println!("Move closure result: {}", sum_closure());

    // Closure with mutable capture
    let mut counter = 0;
    let mut increment = || {
        counter += 1;
        counter
    };
    println!("Counter after increment: {}", increment());
    println!("Counter after increment: {}", increment());

    println!("\nClosure events:");
    for event in get_events().iter().filter(|e| {
        let s = format!("{:?}", e);
        s.contains("Closure") || s.contains("Capture")
    }) {
        println!("  {:?}", event);
    }
}

#[trace_borrow]
fn example_box_raw() {
    reset();
    println!("\n--- Box Raw Pointer Tracking ---");

    // Create a Box
    let boxed = Box::new(42);
    println!("Created Box with value: {}", *boxed);

    // Convert to raw pointer
    let raw_ptr = Box::into_raw(boxed);
    println!("Converted to raw pointer: {:?}", raw_ptr);

    // Convert back to Box (unsafe)
    let recovered = unsafe { Box::from_raw(raw_ptr) };
    println!("Recovered Box with value: {}", *recovered);

    println!("\nBox raw events:");
    for event in get_events().iter().filter(|e| {
        let s = format!("{:?}", e);
        s.contains("Box") || s.contains("Raw")
    }) {
        println!("  {:?}", event);
    }
}
