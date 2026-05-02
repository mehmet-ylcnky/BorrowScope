//! Loop tracking: for, while, loop with iteration counts
//!
//! Run with: cargo run --example loops

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn for_loop_basic() {
    let items = vec![1, 2, 3, 4, 5];
    for item in &items {
        println!("Item: {}", item);
    }
}

#[trace_borrow]
fn for_loop_range() {
    for i in 0..5 {
        println!("Index: {}", i);
    }
}

#[trace_borrow]
fn for_loop_enumerate() {
    let words = vec!["hello", "world", "rust"];
    for (i, word) in words.iter().enumerate() {
        println!("{}: {}", i, word);
    }
}

#[trace_borrow]
fn while_loop_basic() {
    let mut count = 0;
    while count < 3 {
        println!("Count: {}", count);
        count += 1;
    }
}

#[trace_borrow]
fn while_let_loop() {
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("Popped: {}", top);
    }
}

#[trace_borrow]
fn infinite_loop_with_break() {
    let mut counter = 0;
    loop {
        counter += 1;
        println!("Loop iteration: {}", counter);
        if counter >= 3 {
            break;
        }
    }
}

#[trace_borrow]
fn loop_with_return_value() {
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2;
        }
    };
    println!("Loop returned: {}", result);
}

#[trace_borrow]
fn nested_loops() {
    for i in 0..2 {
        for j in 0..2 {
            println!("({}, {})", i, j);
        }
    }
}

#[trace_borrow]
fn labeled_loops() {
    'outer: for i in 0..3 {
        for j in 0..3 {
            if i == 1 && j == 1 {
                println!("Breaking outer at ({}, {})", i, j);
                break 'outer;
            }
            println!("({}, {})", i, j);
        }
    }
}

#[trace_borrow]
fn loop_with_continue() {
    for i in 0..5 {
        if i == 2 {
            continue;
        }
        println!("Processing: {}", i);
    }
}

fn main() {
    println!("=== For Loop Basic ===");
    reset();
    for_loop_basic();
    print_events("for_loop_basic");

    println!("\n=== For Loop Range ===");
    reset();
    for_loop_range();
    print_events("for_loop_range");

    println!("\n=== For Loop Enumerate ===");
    reset();
    for_loop_enumerate();
    print_events("for_loop_enumerate");

    println!("\n=== While Loop Basic ===");
    reset();
    while_loop_basic();
    print_events("while_loop_basic");

    println!("\n=== While Let Loop ===");
    reset();
    while_let_loop();
    print_events("while_let_loop");

    println!("\n=== Infinite Loop with Break ===");
    reset();
    infinite_loop_with_break();
    print_events("infinite_loop_with_break");

    println!("\n=== Loop with Return Value ===");
    reset();
    loop_with_return_value();
    print_events("loop_with_return_value");

    println!("\n=== Nested Loops ===");
    reset();
    nested_loops();
    print_events("nested_loops");

    println!("\n=== Labeled Loops ===");
    reset();
    labeled_loops();
    print_events("labeled_loops");

    println!("\n=== Loop with Continue ===");
    reset();
    loop_with_continue();
    print_events("loop_with_continue");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
