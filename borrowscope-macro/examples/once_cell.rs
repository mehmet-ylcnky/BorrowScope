//! Example demonstrating OnceCell and OnceLock tracking with the macro
//!
//! Run with: cargo run -p borrowscope-macro --example once_cell

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::cell::OnceCell;
use std::sync::OnceLock;

#[trace_borrow]
fn once_cell_demo() {
    // OnceCell::new is tracked
    let config: OnceCell<String> = OnceCell::new();
    
    // .get() is tracked (returns None before initialization)
    let _value = config.get();
    
    // .set() is tracked
    let _ = config.set(String::from("initialized"));
    
    // .get() is tracked (returns Some after initialization)
    let _value = config.get();
    
    // Second set fails (tracked)
    let _ = config.set(String::from("second"));
}

#[trace_borrow]
fn once_lock_demo() {
    // OnceLock::new is tracked (thread-safe variant)
    let counter: OnceLock<i32> = OnceLock::new();
    
    // Operations are tracked
    let _ = counter.set(42);
    let _value = counter.get();
}

#[trace_borrow]
fn get_or_init_demo() {
    let lazy: OnceCell<Vec<i32>> = OnceCell::new();
    
    // get_or_init tracks whether initialization happened
    let _data = lazy.get_or_init(|| vec![1, 2, 3]);
    
    // Second call doesn't reinitialize (tracked)
    let _data = lazy.get_or_init(|| vec![4, 5, 6]);
}

fn main() {
    reset();
    println!("=== OnceCell/OnceLock Macro Tracking Demo ===\n");
    
    println!("--- OnceCell Demo ---");
    once_cell_demo();
    
    println!("\n--- OnceLock Demo ---");
    once_lock_demo();
    
    println!("\n--- get_or_init Demo ---");
    get_or_init_demo();
    
    println!("\n=== Tracked Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }
    
    // Summary
    let events = get_events();
    let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
    println!("\n=== Summary ===");
    println!("Total events: {}", events.len());
    println!("OnceCell/OnceLock events: {}", once_cell_events.len());
}
