//! Comprehensive example combining all features
//!
//! Run with: cargo run --example comprehensive

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// A simple data structure to demonstrate ownership patterns
#[derive(Debug, Clone)]
struct Data {
    id: u32,
    name: String,
    values: Vec<i32>,
}

impl Data {
    fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            values: Vec::new(),
        }
    }
    
    fn add_value(&mut self, value: i32) {
        self.values.push(value);
    }
}


fn ownership_patterns() {
    // Create owned data
    let mut data = Data::new(1, "original");
    data.add_value(10);
    data.add_value(20);
    
    // Borrow immutably
    let borrowed = &data;
    println!("Borrowed: {:?}", borrowed);
    
    // Borrow mutably (after immutable borrow ends)
    let borrowed_mut = &mut data;
    borrowed_mut.add_value(30);
    
    // Move ownership
    let moved_data = data;
    println!("Moved: {:?}", moved_data);
    
    // Clone to get a copy
    let cloned_data = moved_data.clone();
    println!("Cloned: {:?}", cloned_data);
}

#[trace_borrow]
fn smart_pointer_patterns() {
    // Rc for shared ownership
    let shared = Rc::new(Data::new(2, "shared"));
    let clone1 = Rc::clone(&shared);
    let clone2 = Rc::clone(&shared);
    
    println!("Shared data accessed by {} owners", Rc::strong_count(&shared));
    
    // Rc<RefCell<T>> for shared mutable state
    let mutable_shared = Rc::new(RefCell::new(Data::new(3, "mutable_shared")));
    
    {
        let mut borrowed = mutable_shared.borrow_mut();
        borrowed.add_value(100);
    }
    
    println!("After mutation: {:?}", mutable_shared.borrow());
    
    // Arc<Mutex<T>> for thread-safe shared state
    let thread_safe = Arc::new(Mutex::new(Data::new(4, "thread_safe")));
    let arc_clone = Arc::clone(&thread_safe);
    
    {
        let mut guard = arc_clone.lock().unwrap();
        guard.add_value(200);
    }
    
    println!("Thread-safe data: {:?}", thread_safe.lock().unwrap());
}

#[trace_borrow]
fn control_flow_patterns() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // For loop with continue and break
    'outer: for chunk in items.chunks(3) {
        for &item in chunk {
            if item == 2 {
                continue; // Skip 2
            }
            if item == 8 {
                break 'outer; // Exit both loops at 8
            }
            println!("Processing: {}", item);
        }
    }
    
    // While loop
    let mut count = 0;
    while count < 3 {
        count += 1;
        println!("Count: {}", count);
    }
    
    // Match expression
    let result: Result<i32, &str> = Ok(42);
    match result {
        Ok(v) if v > 50 => println!("Large value: {}", v),
        Ok(v) => println!("Value: {}", v),
        Err(e) => println!("Error: {}", e),
    }
}


fn error_handling_patterns() -> Result<String, &'static str> {
    fn step1() -> Result<i32, &'static str> {
        Ok(10)
    }
    
    fn step2(x: i32) -> Result<i32, &'static str> {
        if x > 0 { Ok(x * 2) } else { Err("negative") }
    }
    
    fn step3(x: i32) -> Result<String, &'static str> {
        Ok(format!("Result: {}", x))
    }
    
    // Chain with ? operator
    let a = step1()?;
    let b = step2(a)?;
    let c = step3(b)?;
    
    Ok(c)
}

#[trace_borrow]
fn expression_patterns() {
    // Struct creation
    let point = (10, 20);
    
    // Array creation
    let arr = [1, 2, 3, 4, 5];
    
    // Range expressions
    let range = 0..10;
    let range_inclusive = 0..=10;
    
    // Type casts
    let x: i32 = 42;
    let y = x as f64;
    
    // Closures
    let add = |a, b| a + b;
    let multiply = move |a: i32| a * x;
    
    println!("Point: {:?}", point);
    println!("Array: {:?}", arr);
    println!("Range: {:?}", range.collect::<Vec<_>>());
    println!("Add: {}", add(1, 2));
    println!("Multiply: {}", multiply(5));
}

#[trace_borrow]
fn unsafe_patterns() {
    let data = vec![1, 2, 3, 4, 5];
    let ptr = data.as_ptr();
    
    unsafe {
        // Raw pointer operations
        for i in 0..data.len() {
            let value = *ptr.add(i);
            println!("data[{}] = {}", i, value);
        }
    }
    
    // Transmute example
    let x: u32 = 0x41424344;
    unsafe {
        let bytes: [u8; 4] = std::mem::transmute(x);
        println!("Bytes: {:?}", bytes);
    }
}

#[trace_borrow]
fn method_patterns() {
    // Clone
    let s1 = String::from("hello");
    let s2 = s1.clone();
    
    // Unwrap variants
    let opt = Some(42);
    let value = opt.unwrap();
    
    let opt2: Option<i32> = None;
    let default = opt2.unwrap_or(0);
    
    // Lock
    let mutex = Mutex::new(vec![1, 2, 3]);
    {
        let mut guard = mutex.lock().unwrap();
        guard.push(4);
    }
    
    println!("s1: {}, s2: {}", s1, s2);
    println!("value: {}, default: {}", value, default);
    println!("mutex: {:?}", mutex.lock().unwrap());
}

#[trace_borrow(only = "ownership,functions")]
fn focused_tracking() {
    // Only ownership and function entry/exit tracked
    let data = vec![1, 2, 3];
    
    for i in &data {        // Loop NOT tracked
        if *i > 1 {         // Branch NOT tracked
            println!("{}", i);
        }
    }
    
    let moved = data;       // Move IS tracked
    println!("{:?}", moved);
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         BorrowScope Macro - Comprehensive Example            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. OWNERSHIP PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    ownership_patterns();
    print_summary("ownership_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2. SMART POINTER PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    smart_pointer_patterns();
    print_summary("smart_pointer_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. CONTROL FLOW PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    control_flow_patterns();
    print_summary("control_flow_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4. ERROR HANDLING PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    let result = error_handling_patterns();
    println!("Result: {:?}", result);
    print_summary("error_handling_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5. EXPRESSION PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    expression_patterns();
    print_summary("expression_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("6. UNSAFE PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    unsafe_patterns();
    print_summary("unsafe_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("7. METHOD PATTERNS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    method_patterns();
    print_summary("method_patterns");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("8. FOCUSED TRACKING (only ownership + functions)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    reset();
    focused_tracking();
    print_summary("focused_tracking");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Example Complete!                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn print_summary(name: &str) {
    let events = get_events();
    println!("\n📊 {} - {} events captured:", name, events.len());
    
    // Count event types
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for event in &events {
        let type_name = format!("{:?}", event).split_whitespace().next().unwrap_or("Unknown").to_string();
        *counts.entry(type_name).or_insert(0) += 1;
    }
    
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    for (event_type, count) in sorted {
        println!("   • {}: {}", event_type, count);
    }
}
