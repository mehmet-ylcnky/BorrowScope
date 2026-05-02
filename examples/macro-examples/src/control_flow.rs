//! Control flow tracking: break, continue, return
//!
//! Run with: cargo run --example control_flow

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn return_early() {
    let x = 10;
    if x > 5 {
        println!("Early return");
        return;
    }
    println!("This won't print");
}

#[trace_borrow]
fn return_value() -> i32 {
    let x = 5;
    let y = 10;
    return x + y;
}

#[trace_borrow]
fn return_implicit() -> String {
    let s = String::from("implicit return");
    s // Implicit return
}

#[trace_borrow]
fn break_simple() {
    for i in 0..10 {
        if i == 5 {
            break;
        }
        println!("i = {}", i);
    }
}

#[trace_borrow]
fn break_with_value() -> i32 {
    let result = loop {
        let x = 42;
        break x;
    };
    result
}

#[trace_borrow]
fn break_labeled() {
    'outer: for i in 0..3 {
        for j in 0..3 {
            if i == 1 && j == 1 {
                break 'outer;
            }
            println!("({}, {})", i, j);
        }
    }
    println!("Exited outer loop");
}

#[trace_borrow]
fn continue_simple() {
    for i in 0..5 {
        if i == 2 {
            continue;
        }
        println!("Processing {}", i);
    }
}

#[trace_borrow]
fn continue_labeled() {
    'outer: for i in 0..3 {
        for j in 0..3 {
            if j == 1 {
                continue 'outer;
            }
            println!("({}, {})", i, j);
        }
    }
}

#[trace_borrow]
fn multiple_returns() -> i32 {
    let x = 10;
    
    if x < 0 {
        return -1;
    }
    
    if x == 0 {
        return 0;
    }
    
    if x < 10 {
        return 1;
    }
    
    return 2;
}

#[trace_borrow]
fn break_continue_in_while() {
    let mut i = 0;
    while i < 10 {
        i += 1;
        if i == 3 {
            continue;
        }
        if i == 7 {
            break;
        }
        println!("i = {}", i);
    }
}

#[trace_borrow]
fn nested_control_flow() {
    'outer: for i in 0..5 {
        for j in 0..5 {
            if j == 0 {
                continue; // Skip j=0
            }
            if i * j > 6 {
                break 'outer; // Exit both loops
            }
            println!("{} * {} = {}", i, j, i * j);
        }
    }
}

fn main() {
    println!("=== Return Early ===");
    reset();
    return_early();
    print_events("return_early");

    println!("\n=== Return Value ===");
    reset();
    let r = return_value();
    println!("Returned: {}", r);
    print_events("return_value");

    println!("\n=== Return Implicit ===");
    reset();
    let s = return_implicit();
    println!("Returned: {}", s);
    print_events("return_implicit");

    println!("\n=== Break Simple ===");
    reset();
    break_simple();
    print_events("break_simple");

    println!("\n=== Break with Value ===");
    reset();
    let v = break_with_value();
    println!("Break returned: {}", v);
    print_events("break_with_value");

    println!("\n=== Break Labeled ===");
    reset();
    break_labeled();
    print_events("break_labeled");

    println!("\n=== Continue Simple ===");
    reset();
    continue_simple();
    print_events("continue_simple");

    println!("\n=== Continue Labeled ===");
    reset();
    continue_labeled();
    print_events("continue_labeled");

    println!("\n=== Multiple Returns ===");
    reset();
    let r = multiple_returns();
    println!("Returned: {}", r);
    print_events("multiple_returns");

    println!("\n=== Break Continue in While ===");
    reset();
    break_continue_in_while();
    print_events("break_continue_in_while");

    println!("\n=== Nested Control Flow ===");
    reset();
    nested_control_flow();
    print_events("nested_control_flow");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
