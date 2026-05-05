//! Example demonstrating MaybeUninit tracking with the macro
//!
//! Run with: cargo run -p borrowscope-macro --example maybe_uninit

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::mem::MaybeUninit;

#[trace_borrow]
fn uninit_demo() {
    // MaybeUninit::uninit is tracked
    let mut data: MaybeUninit<i32> = MaybeUninit::uninit();
    
    // .write() is tracked
    let written = data.write(42);
    println!("Written value: {}", written);
}

#[trace_borrow]
fn new_demo() {
    // MaybeUninit::new is tracked (initialized)
    let init: MaybeUninit<String> = MaybeUninit::new(String::from("hello"));
    
    // assume_init is tracked (unsafe)
    let value = unsafe { init.assume_init() };
    println!("Value: {}", value);
}

#[trace_borrow]
fn assume_init_read_demo() {
    let data: MaybeUninit<i32> = MaybeUninit::new(100);
    
    // assume_init_read is tracked (doesn't take ownership)
    let value = unsafe { data.assume_init_read() };
    println!("Read value: {}", value);
}


fn assume_init_drop_demo() {
    let mut data: MaybeUninit<Vec<i32>> = MaybeUninit::new(vec![1, 2, 3]);
    
    // assume_init_drop is tracked (drops the value)
    unsafe { data.assume_init_drop() };
    println!("Value dropped");
}

fn main() {
    reset();
    println!("=== MaybeUninit Macro Tracking Demo ===\n");
    
    println!("--- Uninit Demo ---");
    uninit_demo();
    
    println!("\n--- New Demo ---");
    new_demo();
    
    println!("\n--- assume_init_read Demo ---");
    assume_init_read_demo();
    
    println!("\n--- assume_init_drop Demo ---");
    assume_init_drop_demo();
    
    println!("\n=== Tracked Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }
    
    // Summary
    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    println!("\n=== Summary ===");
    println!("Total events: {}", events.len());
    println!("MaybeUninit events: {}", maybe_uninit_events.len());
}
