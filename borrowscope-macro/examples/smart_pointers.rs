//! Smart pointer tracking: Rc, Arc, RefCell, Cell
//!
//! Run with: cargo run --example smart_pointers

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

#[trace_borrow]
fn rc_basic() {
    let rc1 = Rc::new(vec![1, 2, 3]);
    let rc2 = Rc::clone(&rc1); // Clone increases ref count
    let rc3 = Rc::clone(&rc1);
    println!("Strong count: {}", Rc::strong_count(&rc1));
}

#[trace_borrow]
fn rc_shared_ownership() {
    let data = Rc::new(String::from("shared"));
    let clone1 = Rc::clone(&data);
    let clone2 = Rc::clone(&data);
    
    // All three can read the data
    println!("data: {}, clone1: {}, clone2: {}", data, clone1, clone2);
}

#[trace_borrow]
fn arc_basic() {
    let arc1 = Arc::new(42);
    let arc2 = Arc::clone(&arc1);
    println!("Arc value: {}, count: {}", arc1, Arc::strong_count(&arc1));
}

#[trace_borrow]
fn refcell_basic() {
    let cell = RefCell::new(vec![1, 2, 3]);
    
    // Immutable borrow
    {
        let borrowed = cell.borrow();
        println!("Borrowed: {:?}", borrowed);
    }
    
    // Mutable borrow
    {
        let mut borrowed_mut = cell.borrow_mut();
        borrowed_mut.push(4);
        println!("After mutation: {:?}", borrowed_mut);
    }
}

#[trace_borrow]
fn refcell_multiple_borrows() {
    let cell = RefCell::new(100);
    
    // Multiple immutable borrows are OK
    let b1 = cell.borrow();
    let b2 = cell.borrow();
    println!("b1: {}, b2: {}", b1, b2);
    drop(b1);
    drop(b2);
    
    // Now we can mutably borrow
    *cell.borrow_mut() = 200;
    println!("After mutation: {}", cell.borrow());
}

#[trace_borrow]
fn cell_basic() {
    let cell = Cell::new(10);
    
    // Get value (copies it)
    let value = cell.get();
    println!("Initial value: {}", value);
    
    // Set new value
    cell.set(20);
    println!("After set: {}", cell.get());
    
    // Can set again
    cell.set(30);
    println!("After second set: {}", cell.get());
}

#[trace_borrow]
fn rc_with_refcell() {
    // Common pattern: Rc<RefCell<T>> for shared mutable state
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    
    let clone1 = Rc::clone(&shared);
    let clone2 = Rc::clone(&shared);
    
    // Mutate through clone1
    clone1.borrow_mut().push(4);
    
    // Read through clone2
    println!("Data: {:?}", clone2.borrow());
}

fn main() {
    println!("=== Rc Basic ===");
    reset();
    rc_basic();
    print_events("rc_basic");

    println!("\n=== Rc Shared Ownership ===");
    reset();
    rc_shared_ownership();
    print_events("rc_shared_ownership");

    println!("\n=== Arc Basic ===");
    reset();
    arc_basic();
    print_events("arc_basic");

    println!("\n=== RefCell Basic ===");
    reset();
    refcell_basic();
    print_events("refcell_basic");

    println!("\n=== RefCell Multiple Borrows ===");
    reset();
    refcell_multiple_borrows();
    print_events("refcell_multiple_borrows");

    println!("\n=== Cell Basic ===");
    reset();
    cell_basic();
    print_events("cell_basic");

    println!("\n=== Rc with RefCell ===");
    reset();
    rc_with_refcell();
    print_events("rc_with_refcell");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
