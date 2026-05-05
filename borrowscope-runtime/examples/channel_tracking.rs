//! Example: Channel Tracking
//!
//! Demonstrates tracking of mpsc channel operations including
//! creation, send, recv, and try_recv.

use borrowscope_runtime::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    reset();
    println!("=== Channel Tracking Example ===\n");

    // Basic channel usage
    println!("--- Basic Channel ---");
    let (tx, rx) = mpsc::channel();
    let (tx, rx) = track_channel("basic", "main.rs:16", tx, rx);

    let _ = track_channel_send("basic_tx", "main.rs:18", tx.send("hello"));
    let _ = track_channel_send("basic_tx", "main.rs:19", tx.send("world"));

    let msg1 = track_channel_recv("basic_rx", "main.rs:21", rx.recv());
    let msg2 = track_channel_recv("basic_rx", "main.rs:22", rx.recv());
    println!("Received: {:?}, {:?}", msg1, msg2);

    // Channel with try_recv
    println!("\n--- try_recv ---");
    let (tx, rx) = mpsc::channel();
    tx.send(42).unwrap();

    let result = track_channel_try_recv("rx", "main.rs:30", rx.try_recv());
    println!("try_recv with data: {:?}", result);

    let empty = track_channel_try_recv("rx", "main.rs:33", rx.try_recv());
    println!("try_recv empty: {:?}", empty);

    // Cross-thread communication
    println!("\n--- Cross-Thread Channel ---");
    let (tx, rx) = mpsc::channel();
    let (tx, rx) = track_channel("cross_thread", "main.rs:39", tx, rx);

    let sender = thread::spawn(move || {
        for i in 1..=3 {
            let _ = track_channel_send("cross_thread_tx", &format!("send:{}", i), tx.send(i * 10));
            thread::sleep(Duration::from_millis(10));
        }
        println!("  [sender] Done sending");
    });

    let receiver = thread::spawn(move || {
        let mut received = Vec::new();
        for i in 1..=3 {
            if let Ok(val) =
                track_channel_recv("cross_thread_rx", &format!("recv:{}", i), rx.recv())
            {
                received.push(val);
            }
        }
        println!("  [receiver] Received: {:?}", received);
        received
    });

    sender.join().unwrap();
    let results = receiver.join().unwrap();
    println!("Final results: {:?}", results);

    // Multi-producer pattern
    println!("\n--- Multi-Producer ---");
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    let tx2 = tx.clone();
    drop(tx); // Drop original

    let _ = track_channel_send("producer1", "main.rs:70", tx1.send("from producer 1"));
    let _ = track_channel_send("producer2", "main.rs:71", tx2.send("from producer 2"));

    drop(tx1);
    drop(tx2);

    while let Ok(msg) = rx.recv() {
        println!("Multi-producer received: {}", msg);
    }

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
