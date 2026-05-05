//! Example: Thread Tracking
//!
//! Demonstrates tracking of thread spawn and join operations
//! for concurrent programming patterns.

use borrowscope_runtime::*;
use std::thread;
use std::time::Duration;

fn main() {
    reset();
    println!("=== Thread Tracking Example ===\n");

    // Basic thread spawn and join
    println!("--- Basic Thread ---");
    let handle = track_thread_spawn(
        "worker",
        "main.rs:14",
        thread::spawn(|| {
            println!("  [worker] Hello from spawned thread!");
            42
        }),
    );

    let result = track_thread_join("worker", "main.rs:19", handle.join());
    println!("Worker returned: {:?}", result);

    // Thread with moved data
    println!("\n--- Thread with Moved Data ---");
    let data = vec![1, 2, 3, 4, 5];
    let handle = track_thread_spawn(
        "processor",
        "main.rs:25",
        thread::spawn(move || {
            println!("  [processor] Processing {:?}", data);
            data.iter().sum::<i32>()
        }),
    );

    let sum = track_thread_join("processor", "main.rs:30", handle.join());
    println!("Sum computed by thread: {:?}", sum);

    // Multiple parallel threads
    println!("\n--- Parallel Threads ---");
    let handles: Vec<_> = (0..4)
        .map(|i| {
            track_thread_spawn(
                &format!("parallel_{}", i),
                &format!("main.rs:38:{}", i),
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(50));
                    println!("  [parallel_{}] Done!", i);
                    i * 10
                }),
            )
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            track_thread_join(
                &format!("parallel_{}", i),
                &format!("main.rs:50:{}", i),
                h.join(),
            )
        })
        .collect();

    println!("Parallel results: {:?}", results);

    // Thread returning complex data
    println!("\n--- Thread Returning Struct ---");
    #[derive(Debug)]
    struct ComputeResult {
        input: i32,
        output: i32,
    }

    let handle = track_thread_spawn(
        "compute",
        "main.rs:63",
        thread::spawn(|| ComputeResult {
            input: 5,
            output: 5 * 5,
        }),
    );

    let result = track_thread_join("compute", "main.rs:70", handle.join());
    println!("Compute result: {:?}", result);

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
