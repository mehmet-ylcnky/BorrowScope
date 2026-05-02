//! Example: High Priority Tracking Features
//!
//! Demonstrates automatic instrumentation of:
//! - Cow::to_mut
//! - Weak::upgrade, Weak::clone
//! - JoinHandle::join
//! - Sender::send, Receiver::recv, Receiver::try_recv

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};
use std::borrow::Cow;
use std::rc::{Rc, Weak};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== High Priority Tracking Features ===\n");

    example_cow_to_mut();
    example_weak_operations();
    example_thread_join();
    example_channel_operations();

    println!("\n=== All Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}

#[trace_borrow]
fn example_cow_to_mut() {
    reset();
    println!("--- Cow::to_mut Tracking ---");

    // Borrowed Cow - to_mut will clone
    let mut borrowed: Cow<str> = Cow::Borrowed("hello");
    println!("Created borrowed Cow: {:?}", borrowed);

    borrowed.to_mut().push_str(" world");
    println!("After to_mut (cloned): {:?}", borrowed);

    // Owned Cow - to_mut won't clone
    let mut owned: Cow<str> = Cow::Owned(String::from("owned"));
    println!("Created owned Cow: {:?}", owned);

    owned.to_mut().push_str(" data");
    println!("After to_mut (no clone): {:?}", owned);

    println!("\nCow events:");
    for event in get_events().iter().filter(|e| format!("{:?}", e).contains("Cow")) {
        println!("  {:?}", event);
    }
}

#[trace_borrow]
fn example_weak_operations() {
    reset();
    println!("\n--- Weak Operations Tracking ---");

    // Create Rc and downgrade to Weak
    let strong = Rc::new(42);
    println!("Created Rc with value: {}", *strong);

    let weak = Rc::downgrade(&strong);
    println!("Created Weak reference");

    // Clone the weak reference
    let weak2 = weak.clone();
    println!("Cloned Weak reference");

    // Upgrade weak to strong
    if let Some(upgraded) = weak.upgrade() {
        println!("Upgraded weak to strong: {}", *upgraded);
    }

    // Upgrade weak2
    if let Some(upgraded2) = weak2.upgrade() {
        println!("Upgraded weak2 to strong: {}", *upgraded2);
    }

    // Drop strong and try upgrade (should fail)
    drop(strong);
    println!("Dropped strong reference");

    if weak.upgrade().is_none() {
        println!("Upgrade failed (as expected) - strong was dropped");
    }

    println!("\nWeak events:");
    for event in get_events().iter().filter(|e| format!("{:?}", e).contains("Weak")) {
        println!("  {:?}", event);
    }
}

#[trace_borrow]
fn example_thread_join() {
    reset();
    println!("\n--- Thread Join Tracking ---");

    let handle = thread::spawn(|| {
        println!("  [thread] Computing...");
        thread::sleep(Duration::from_millis(10));
        100
    });

    println!("Spawned thread, calling join()...");
    let result = handle.join();
    match result {
        Ok(value) => println!("Thread joined successfully with value: {}", value),
        Err(_) => println!("Thread panicked!"),
    }

    println!("\nThread events:");
    for event in get_events().iter().filter(|e| format!("{:?}", e).contains("Thread")) {
        println!("  {:?}", event);
    }
}

#[trace_borrow]
fn example_channel_operations() {
    reset();
    println!("\n--- Channel Operations Tracking ---");

    let (tx, rx) = mpsc::channel();

    // Spawn sender thread
    let sender_handle = thread::spawn(move || {
        for i in 1..=3 {
            let result = tx.send(i * 10);
            match result {
                Ok(()) => println!("  [sender] Sent: {}", i * 10),
                Err(e) => println!("  [sender] Send failed: {}", e),
            }
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Receive with recv()
    for _ in 0..2 {
        match rx.recv() {
            Ok(value) => println!("  [receiver] recv() got: {}", value),
            Err(e) => println!("  [receiver] recv() error: {}", e),
        }
    }

    // Try try_recv()
    thread::sleep(Duration::from_millis(20));
    match rx.try_recv() {
        Ok(value) => println!("  [receiver] try_recv() got: {}", value),
        Err(e) => println!("  [receiver] try_recv() result: {:?}", e),
    }

    // Try again when empty
    match rx.try_recv() {
        Ok(value) => println!("  [receiver] try_recv() got: {}", value),
        Err(e) => println!("  [receiver] try_recv() result: {:?}", e),
    }

    sender_handle.join().unwrap();

    println!("\nChannel events:");
    for event in get_events().iter().filter(|e| {
        let s = format!("{:?}", e);
        s.contains("Channel") || s.contains("Send") || s.contains("Recv")
    }) {
        println!("  {:?}", event);
    }
}
