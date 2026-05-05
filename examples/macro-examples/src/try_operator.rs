//! Try operator (?) tracking
//!
//! Run with: cargo run --example try_operator

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::fs::File;
use std::io::{self, Read};

#[trace_borrow]
fn try_option() -> Option<i32> {
    let a = Some(10);
    let b = Some(20);
    
    let x = a?;
    let y = b?;
    
    Some(x + y)
}

#[trace_borrow]
fn try_option_chain() -> Option<String> {
    let data = Some(vec![1, 2, 3]);
    
    let vec = data?;
    let first = vec.first()?;
    
    Some(format!("First element: {}", first))
}

#[trace_borrow]
fn try_result() -> Result<i32, &'static str> {
    let a: Result<i32, &str> = Ok(10);
    let b: Result<i32, &str> = Ok(20);
    
    let x = a?;
    let y = b?;
    
    Ok(x + y)
}

fn try_step1() -> Result<i32, &'static str> { Ok(1) }
fn try_step2(x: i32) -> Result<i32, &'static str> { Ok(x * 2) }
fn try_step3(x: i32) -> Result<String, &'static str> { Ok(format!("Result: {}", x)) }

#[trace_borrow]
fn try_result_chain() -> Result<String, &'static str> {
    let a = try_step1()?;
    let b = try_step2(a)?;
    let c = try_step3(b)?;
    
    Ok(c)
}

#[trace_borrow]
fn try_with_map() -> Option<i32> {
    let s = Some("42");
    let parsed = s?.parse::<i32>().ok()?;
    Some(parsed * 2)
}

#[trace_borrow]
fn try_in_loop() -> Option<i32> {
    let items = vec![Some(1), Some(2), None, Some(4)];
    let mut sum = 0;
    
    for item in items {
        // This will return None when hitting the None element
        if item.is_none() {
            return None;
        }
        sum += item?;
    }
    
    Some(sum)
}

fn try_nested() -> Result<i32, String> {
    fn inner() -> Result<i32, String> {
        let x: Result<i32, String> = Ok(5);
        Ok(x? * 2)
    }
    let a = inner()?;
    let b = inner()?;
    Ok(a + b)
}

#[trace_borrow]
fn try_with_conversion() -> Result<i32, Box<dyn std::error::Error>> {
    let s = "42";
    let n: i32 = s.parse()?; // ParseIntError -> Box<dyn Error>
    Ok(n)
}


fn multiple_try_same_line() -> Option<i32> {
    fn get_a() -> Option<i32> { Some(1) }
    fn get_b() -> Option<i32> { Some(2) }
    
    // Multiple ? on conceptually related operations
    let sum = get_a()? + get_b()?;
    Some(sum)
}

fn main() {
    println!("=== Try Option ===");
    reset();
    let r = try_option();
    println!("Result: {:?}", r);
    print_events("try_option");

    println!("\n=== Try Option Chain ===");
    reset();
    let r = try_option_chain();
    println!("Result: {:?}", r);
    print_events("try_option_chain");

    println!("\n=== Try Result ===");
    reset();
    let r = try_result();
    println!("Result: {:?}", r);
    print_events("try_result");

    println!("\n=== Try Result Chain ===");
    reset();
    let r = try_result_chain();
    println!("Result: {:?}", r);
    print_events("try_result_chain");

    println!("\n=== Try with Map ===");
    reset();
    let r = try_with_map();
    println!("Result: {:?}", r);
    print_events("try_with_map");

    println!("\n=== Try in Loop ===");
    reset();
    let r = try_in_loop();
    println!("Result: {:?}", r);
    print_events("try_in_loop");

    println!("\n=== Try Nested ===");
    reset();
    let r = try_nested();
    println!("Result: {:?}", r);
    print_events("try_nested");

    println!("\n=== Try with Conversion ===");
    reset();
    let r = try_with_conversion();
    println!("Result: {:?}", r);
    print_events("try_with_conversion");

    println!("\n=== Multiple Try Same Line ===");
    reset();
    let r = multiple_try_same_line();
    println!("Result: {:?}", r);
    print_events("multiple_try_same_line");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
