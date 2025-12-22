//! Example: Lock Guard Tracking
//!
//! Demonstrates tracking of Mutex and RwLock guard acquisition
//! and release for synchronization primitives.

use borrowscope_runtime::*;
use std::sync::{Mutex, RwLock};

fn main() {
    reset();
    println!("=== Lock Guard Tracking Example ===\n");

    // Mutex guard tracking
    println!("--- Mutex Guard ---");
    let mutex = Mutex::new(0);

    track_lock_guard_acquire("guard1", "counter", "Mutex", "main.rs:15");
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
        println!("Incremented counter to: {}", *guard);
    }
    track_lock_guard_drop("guard1", "main.rs:21");

    track_lock_guard_acquire("guard2", "counter", "Mutex", "main.rs:23");
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
        println!("Incremented counter to: {}", *guard);
    }
    track_lock_guard_drop("guard2", "main.rs:29");

    // RwLock read guards
    println!("\n--- RwLock Read Guards ---");
    let rwlock = RwLock::new(vec![1, 2, 3]);

    track_lock_guard_acquire("reader1", "data", "RwLock::read", "main.rs:35");
    track_lock_guard_acquire("reader2", "data", "RwLock::read", "main.rs:36");
    {
        let r1 = rwlock.read().unwrap();
        let r2 = rwlock.read().unwrap();
        println!("Reader 1 sees: {:?}", &*r1);
        println!("Reader 2 sees: {:?}", &*r2);
    }
    track_lock_guard_drop("reader1", "main.rs:43");
    track_lock_guard_drop("reader2", "main.rs:44");

    // RwLock write guard
    println!("\n--- RwLock Write Guard ---");
    track_lock_guard_acquire("writer", "data", "RwLock::write", "main.rs:48");
    {
        let mut w = rwlock.write().unwrap();
        w.push(4);
        println!("After write: {:?}", &*w);
    }
    track_lock_guard_drop("writer", "main.rs:54");

    // Nested locks (different mutexes)
    println!("\n--- Nested Locks ---");
    let mutex_a = Mutex::new("A");
    let mutex_b = Mutex::new("B");

    track_lock_guard_acquire("outer", "mutex_a", "Mutex", "main.rs:61");
    {
        let _a = mutex_a.lock().unwrap();
        println!("Acquired outer lock (mutex_a)");

        track_lock_guard_acquire("inner", "mutex_b", "Mutex", "main.rs:66");
        {
            let _b = mutex_b.lock().unwrap();
            println!("Acquired inner lock (mutex_b)");
        }
        track_lock_guard_drop("inner", "main.rs:71");
        println!("Released inner lock");
    }
    track_lock_guard_drop("outer", "main.rs:74");
    println!("Released outer lock");

    // try_lock pattern
    println!("\n--- try_lock Pattern ---");
    let mutex = Mutex::new(42);

    if let Ok(guard) = mutex.try_lock() {
        track_lock_guard_acquire("try_guard", "mutex", "Mutex::try_lock", "main.rs:82");
        println!("try_lock succeeded: {}", *guard);
        drop(guard);
        track_lock_guard_drop("try_guard", "main.rs:85");
    }

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
