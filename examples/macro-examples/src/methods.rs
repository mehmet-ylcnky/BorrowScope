//! Method tracking: clone, lock, unwrap
//!
//! Run with: cargo run --example methods

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::sync::{Arc, Mutex, RwLock};

#[trace_borrow]
fn clone_string() {
    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("s1: {}, s2: {}", s1, s2);
}

#[trace_borrow]
fn clone_vec() {
    let v1 = vec![1, 2, 3, 4, 5];
    let v2 = v1.clone();
    println!("v1: {:?}, v2: {:?}", v1, v2);
}

#[trace_borrow]
fn clone_in_loop() {
    let original = String::from("data");
    let mut clones = Vec::new();
    
    for i in 0..3 {
        let cloned = original.clone();
        clones.push(format!("{}-{}", cloned, i));
    }
    
    println!("Clones: {:?}", clones);
}

#[trace_borrow]
fn unwrap_option() {
    let opt = Some(42);
    let value = opt.unwrap();
    println!("Unwrapped: {}", value);
}

#[trace_borrow]
fn unwrap_result() {
    let result: Result<i32, &str> = Ok(100);
    let value = result.unwrap();
    println!("Unwrapped: {}", value);
}

#[trace_borrow]
fn expect_option() {
    let opt = Some("hello");
    let value = opt.expect("Should have a value");
    println!("Expected: {}", value);
}

#[trace_borrow]
fn unwrap_or_default() {
    let opt: Option<String> = None;
    let value = opt.unwrap_or_default();
    println!("Unwrap or default: '{}'", value);
}

#[trace_borrow]
fn unwrap_or_else() {
    let opt: Option<i32> = None;
    let value = opt.unwrap_or_else(|| {
        println!("Computing default...");
        42
    });
    println!("Unwrap or else: {}", value);
}

#[trace_borrow]
fn mutex_lock() {
    let mutex = Mutex::new(vec![1, 2, 3]);
    
    {
        let mut guard = mutex.lock().unwrap();
        guard.push(4);
        println!("Inside lock: {:?}", guard);
    }
    
    println!("After lock released");
}

// Note: Using match instead of if-let to avoid Rust 2021 temporary lifetime issues
// with try_lock() returning Result<MutexGuard, TryLockError<MutexGuard>>
#[trace_borrow]
fn mutex_try_lock() {
    let mutex = Mutex::new(42);
    
    // Explicitly handle the result to avoid temporary lifetime extension issues
    let result = mutex.try_lock();
    match result {
        Ok(guard) => println!("Got lock: {}", guard),
        Err(_) => println!("Couldn't get lock"),
    }
}

#[trace_borrow]
fn rwlock_read() {
    let rwlock = RwLock::new(vec![1, 2, 3]);
    
    // Multiple readers allowed
    let r1 = rwlock.read().unwrap();
    let r2 = rwlock.read().unwrap();
    println!("r1: {:?}, r2: {:?}", r1, r2);
}

#[trace_borrow]
fn rwlock_write() {
    let rwlock = RwLock::new(vec![1, 2, 3]);
    
    {
        let mut w = rwlock.write().unwrap();
        w.push(4);
        println!("After write: {:?}", w);
    }
    
    let r = rwlock.read().unwrap();
    println!("After release: {:?}", r);
}

#[trace_borrow]
fn arc_mutex_pattern() {
    let shared = Arc::new(Mutex::new(0));
    
    let clone1 = Arc::clone(&shared);
    let clone2 = Arc::clone(&shared);
    
    {
        let mut guard = clone1.lock().unwrap();
        *guard += 10;
    }
    
    {
        let mut guard = clone2.lock().unwrap();
        *guard += 20;
    }
    
    println!("Final value: {}", shared.lock().unwrap());
}

#[trace_borrow]
fn chained_methods() {
    let s = String::from("  hello world  ");
    let result = s.trim().to_uppercase().clone();
    println!("Result: {}", result);
}

fn main() {
    println!("=== Clone String ===");
    reset();
    clone_string();
    print_events("clone_string");

    println!("\n=== Clone Vec ===");
    reset();
    clone_vec();
    print_events("clone_vec");

    println!("\n=== Clone in Loop ===");
    reset();
    clone_in_loop();
    print_events("clone_in_loop");

    println!("\n=== Unwrap Option ===");
    reset();
    unwrap_option();
    print_events("unwrap_option");

    println!("\n=== Unwrap Result ===");
    reset();
    unwrap_result();
    print_events("unwrap_result");

    println!("\n=== Expect Option ===");
    reset();
    expect_option();
    print_events("expect_option");

    println!("\n=== Unwrap Or Default ===");
    reset();
    unwrap_or_default();
    print_events("unwrap_or_default");

    println!("\n=== Unwrap Or Else ===");
    reset();
    unwrap_or_else();
    print_events("unwrap_or_else");

    println!("\n=== Mutex Lock ===");
    reset();
    mutex_lock();
    print_events("mutex_lock");

    println!("\n=== Mutex Try Lock ===");
    reset();
    mutex_try_lock();
    print_events("mutex_try_lock");

    println!("\n=== RwLock Read ===");
    reset();
    rwlock_read();
    print_events("rwlock_read");

    println!("\n=== RwLock Write ===");
    reset();
    rwlock_write();
    print_events("rwlock_write");

    println!("\n=== Arc Mutex Pattern ===");
    reset();
    arc_mutex_pattern();
    print_events("arc_mutex_pattern");

    println!("\n=== Chained Methods ===");
    reset();
    chained_methods();
    print_events("chained_methods");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
