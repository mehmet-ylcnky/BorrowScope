//! Configuration options: quiet, verbose, skip, only
//!
//! Run with: cargo run --example config_options

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

// Default configuration - all standard tracking
#[trace_borrow]
fn default_config() {
    let x = vec![1, 2, 3];
    for i in &x {
        if *i > 1 {
            println!("i = {}", i);
        }
    }
}

// Quiet mode - only ownership tracking
#[trace_borrow(quiet)]
fn quiet_mode() {
    let x = vec![1, 2, 3];
    for i in &x {           // Loop NOT tracked
        if *i > 1 {         // Branch NOT tracked
            println!("i = {}", i);
        }
    }
}

// Skip specific features
#[trace_borrow(skip = "loops")]
fn skip_loops() {
    let x = vec![1, 2, 3];
    for i in &x {           // Loop NOT tracked
        if *i > 1 {         // Branch IS tracked
            println!("i = {}", i);
        }
    }
}

#[trace_borrow(skip = "branches")]
fn skip_branches() {
    let x = vec![1, 2, 3];
    for i in &x {           // Loop IS tracked
        if *i > 1 {         // Branch NOT tracked
            println!("i = {}", i);
        }
    }
}

#[trace_borrow(skip = "loops,branches")]
fn skip_loops_and_branches() {
    let x = vec![1, 2, 3];
    for i in &x {           // Loop NOT tracked
        if *i > 1 {         // Branch NOT tracked
            println!("i = {}", i);
        }
    }
}

// Only specific features
#[trace_borrow(only = "ownership")]
fn only_ownership() {
    let x = vec![1, 2, 3];  // New IS tracked
    let y = x;              // Move IS tracked
    for i in &y {           // Loop NOT tracked
        println!("i = {}", i);
    }
}                           // Drop IS tracked

#[trace_borrow(only = "loops")]
fn only_loops() {
    let x = vec![1, 2, 3];  // New NOT tracked
    for i in &x {           // Loop IS tracked
        println!("i = {}", i);
    }
}

#[trace_borrow(only = "ownership,loops")]
fn only_ownership_and_loops() {
    let x = vec![1, 2, 3];  // New IS tracked
    for i in &x {           // Loop IS tracked
        if *i > 1 {         // Branch NOT tracked
            println!("i = {}", i);
        }
    }
}

// Function tracking (disabled by default)
#[trace_borrow(only = "functions")]
fn only_functions() {
    // FnEnter tracked
    let x = 42;             // New NOT tracked
    println!("x = {}", x);
    // FnExit tracked
}

#[trace_borrow(only = "ownership,functions")]
fn ownership_with_functions() {
    // FnEnter tracked
    let x = String::from("hello");  // New IS tracked
    let y = x;                       // Move IS tracked
    println!("y = {}", y);
    // FnExit tracked, Drop tracked
}

// Smart pointers configuration
#[trace_borrow(only = "smart_pointers")]
fn only_smart_pointers() {
    use std::rc::Rc;
    
    let x = 42;                     // New NOT tracked
    let rc = Rc::new(x);            // RcNew IS tracked
    let rc2 = Rc::clone(&rc);       // RcClone IS tracked
    println!("rc: {}, rc2: {}", rc, rc2);
}

// Methods configuration
#[trace_borrow(only = "methods")]
fn only_methods() {
    let s = String::from("hello");
    let s2 = s.clone();             // Clone IS tracked
    
    let opt = Some(42);
    let v = opt.unwrap();           // Unwrap IS tracked
    
    println!("s2: {}, v: {}", s2, v);
}

// Try operator configuration
#[trace_borrow(only = "try")]
fn only_try() -> Option<i32> {
    let a = Some(10);
    let b = Some(20);
    
    let x = a?;                     // Try IS tracked
    let y = b?;                     // Try IS tracked
    
    Some(x + y)
}

// Unsafe configuration
#[trace_borrow(only = "unsafe")]
fn only_unsafe() {
    let x = 42;
    unsafe {                        // UnsafeBlockEnter IS tracked
        let ptr = &x as *const i32;
        println!("Value: {}", *ptr);
    }                               // UnsafeBlockExit IS tracked
}

// Expressions configuration
#[trace_borrow(only = "expressions")]
fn only_expressions() {
    let p = (1, 2, 3);              // TupleCreate IS tracked
    let arr = [1, 2, 3];            // ArrayCreate IS tracked
    let r = 0..10;                  // Range IS tracked
    let x = 42i32 as i64;           // TypeCast IS tracked
    
    println!("{:?}, {:?}, {:?}, {}", p, arr, r, x);
}

// Async configuration
#[trace_borrow(only = "async")]
async fn only_async() {
    // AsyncBlockEnter/Exit would be tracked for async blocks
    println!("Async function");
}

// Control flow configuration
#[trace_borrow(only = "control_flow")]
fn only_control_flow() {
    for i in 0..5 {
        if i == 2 {
            continue;               // Continue IS tracked
        }
        if i == 4 {
            break;                  // Break IS tracked
        }
        println!("i = {}", i);
    }
}

fn main() {
    println!("=== Default Config ===");
    reset();
    default_config();
    print_events("default_config");

    println!("\n=== Quiet Mode ===");
    reset();
    quiet_mode();
    print_events("quiet_mode");

    println!("\n=== Skip Loops ===");
    reset();
    skip_loops();
    print_events("skip_loops");

    println!("\n=== Skip Branches ===");
    reset();
    skip_branches();
    print_events("skip_branches");

    println!("\n=== Skip Loops and Branches ===");
    reset();
    skip_loops_and_branches();
    print_events("skip_loops_and_branches");

    println!("\n=== Only Ownership ===");
    reset();
    only_ownership();
    print_events("only_ownership");

    println!("\n=== Only Loops ===");
    reset();
    only_loops();
    print_events("only_loops");

    println!("\n=== Only Ownership and Loops ===");
    reset();
    only_ownership_and_loops();
    print_events("only_ownership_and_loops");

    println!("\n=== Only Functions ===");
    reset();
    only_functions();
    print_events("only_functions");

    println!("\n=== Ownership with Functions ===");
    reset();
    ownership_with_functions();
    print_events("ownership_with_functions");

    println!("\n=== Only Smart Pointers ===");
    reset();
    only_smart_pointers();
    print_events("only_smart_pointers");

    println!("\n=== Only Methods ===");
    reset();
    only_methods();
    print_events("only_methods");

    println!("\n=== Only Try ===");
    reset();
    let _ = only_try();
    print_events("only_try");

    println!("\n=== Only Unsafe ===");
    reset();
    only_unsafe();
    print_events("only_unsafe");

    println!("\n=== Only Expressions ===");
    reset();
    only_expressions();
    print_events("only_expressions");

    println!("\n=== Only Control Flow ===");
    reset();
    only_control_flow();
    print_events("only_control_flow");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
