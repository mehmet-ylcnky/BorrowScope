//! Example demonstrating macro integration with analyzer type info
//!
//! Run: borrowscope-analyzer . && cargo run

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

#[trace_borrow]
fn demo_smart_pointers() {
    // These should be detected via type-info.json lookup
    let rc_data = Rc::new(vec![1, 2, 3]);
    let arc_data = Arc::new("shared".to_string());
    let refcell_data = RefCell::new(42);
    
    // Borrows
    let borrowed = &rc_data;
    let mut guard = refcell_data.borrow_mut();
    *guard = 100;
    
    println!("Rc: {:?}", borrowed);
    println!("Arc: {}", arc_data);
}

#[trace_borrow]
fn demo_primitives() {
    let counter = 0i32;
    let name = String::from("test");
    let flag = true;
    
    println!("{} {} {}", counter, name, flag);
}

fn main() {
    reset();
    
    println!("=== Macro Integration Demo ===\n");
    
    demo_smart_pointers();
    demo_primitives();
    
    println!("\n=== Events ===");
    print_summary();
    
    // Export for inspection
    let events = get_events();
    let json = serde_json::to_string_pretty(&events).unwrap();
    std::fs::write("events.json", &json).unwrap();
    println!("\nExported to events.json");
}
