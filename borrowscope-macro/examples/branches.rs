//! Branch tracking: if/else, match expressions
//!
//! Run with: cargo run --example branches

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn if_basic() {
    let x = 10;
    if x > 5 {
        println!("x is greater than 5");
    }
}

#[trace_borrow]
fn if_else() {
    let x = 3;
    if x > 5 {
        println!("x is greater than 5");
    } else {
        println!("x is not greater than 5");
    }
}

#[trace_borrow]
fn if_else_if() {
    let x = 5;
    if x > 10 {
        println!("x > 10");
    } else if x > 5 {
        println!("x > 5");
    } else if x == 5 {
        println!("x == 5");
    } else {
        println!("x < 5");
    }
}

#[trace_borrow]
fn if_let() {
    let opt = Some(42);
    if let Some(value) = opt {
        println!("Got value: {}", value);
    } else {
        println!("No value");
    }
}

#[trace_borrow]
fn match_basic() {
    let x = 2;
    match x {
        1 => println!("one"),
        2 => println!("two"),
        3 => println!("three"),
        _ => println!("other"),
    }
}

#[trace_borrow]
fn match_with_guards() {
    let x = 5;
    match x {
        n if n < 0 => println!("negative"),
        n if n == 0 => println!("zero"),
        n if n < 10 => println!("small positive: {}", n),
        _ => println!("large"),
    }
}

#[trace_borrow]
fn match_option() {
    let opt: Option<i32> = Some(42);
    match opt {
        Some(x) if x > 50 => println!("Large: {}", x),
        Some(x) => println!("Value: {}", x),
        None => println!("No value"),
    }
}

#[trace_borrow]
fn match_result() {
    let result: Result<i32, &str> = Ok(100);
    match result {
        Ok(v) => println!("Success: {}", v),
        Err(e) => println!("Error: {}", e),
    }
}

#[trace_borrow]
fn match_tuple() {
    let pair = (0, -2);
    match pair {
        (0, y) => println!("First is zero, y = {}", y),
        (x, 0) => println!("Second is zero, x = {}", x),
        _ => println!("Neither is zero"),
    }
}

#[trace_borrow]
fn match_struct() {
    struct Point { x: i32, y: i32 }
    
    let p = Point { x: 0, y: 7 };
    match p {
        Point { x: 0, y } => println!("On y-axis at {}", y),
        Point { x, y: 0 } => println!("On x-axis at {}", x),
        Point { x, y } => println!("At ({}, {})", x, y),
    }
}

#[trace_borrow]
fn match_enum() {
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
    }
    
    let msg = Message::Move { x: 10, y: 20 };
    match msg {
        Message::Quit => println!("Quit"),
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Write(text) => println!("Write: {}", text),
    }
}

#[trace_borrow]
fn nested_if_match() {
    let x = Some(5);
    if let Some(n) = x {
        match n {
            1..=5 => println!("Small: {}", n),
            _ => println!("Large: {}", n),
        }
    }
}

fn main() {
    println!("=== If Basic ===");
    reset();
    if_basic();
    print_events("if_basic");

    println!("\n=== If Else ===");
    reset();
    if_else();
    print_events("if_else");

    println!("\n=== If Else If ===");
    reset();
    if_else_if();
    print_events("if_else_if");

    println!("\n=== If Let ===");
    reset();
    if_let();
    print_events("if_let");

    println!("\n=== Match Basic ===");
    reset();
    match_basic();
    print_events("match_basic");

    println!("\n=== Match with Guards ===");
    reset();
    match_with_guards();
    print_events("match_with_guards");

    println!("\n=== Match Option ===");
    reset();
    match_option();
    print_events("match_option");

    println!("\n=== Match Result ===");
    reset();
    match_result();
    print_events("match_result");

    println!("\n=== Match Tuple ===");
    reset();
    match_tuple();
    print_events("match_tuple");

    println!("\n=== Match Struct ===");
    reset();
    match_struct();
    print_events("match_struct");

    println!("\n=== Match Enum ===");
    reset();
    match_enum();
    print_events("match_enum");

    println!("\n=== Nested If Match ===");
    reset();
    nested_if_match();
    print_events("nested_if_match");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
