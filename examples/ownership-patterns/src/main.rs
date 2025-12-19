//! Ownership Patterns - Experience BorrowScope Runtime Tracking
//!
//! This example demonstrates Rust's ownership patterns with runtime tracking.

use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    reset();
    println!("=== Ownership Patterns Demo ===\n");

    demo_basic_ownership();
    demo_borrowing();
    demo_shared_ownership();
    demo_interior_mutability();

    // Print all captured events
    let events = get_events();
    println!("\n=== Captured Events ({}) ===", events.len());
    for (i, e) in events.iter().enumerate() {
        println!("{:3}. {:?}", i + 1, e);
    }

    // Show graph stats
    let graph = get_graph();
    let stats = graph.stats();
    println!("\n=== Graph Stats ===");
    println!("Variables: {}", stats.total_variables);
    println!("Relationships: {}", stats.total_relationships);
    println!("Immutable borrows: {}", stats.immutable_borrows);
    println!("Mutable borrows: {}", stats.mutable_borrows);

    // Export to JSON
    let path = std::env::temp_dir().join("ownership-patterns.json");
    export_json(&path).unwrap();
    println!("\nExported to: {}", path.display());
}

/// Basic ownership: create, use, drop
fn demo_basic_ownership() {
    println!("--- 1. Basic Ownership ---");
    
    let s1 = track_new("s1", String::from("hello"));
    println!("Created s1: {}", s1);
    
    // Move ownership
    let s2 = track_move("s1", "s2", s1);
    println!("Moved to s2: {}", s2);
    
    track_drop("s2");
    println!();
}

/// Borrowing: immutable and mutable references
fn demo_borrowing() {
    println!("--- 2. Borrowing ---");
    
    let mut data = track_new("data", vec![1, 2, 3]);
    
    // Multiple immutable borrows
    {
        let r1 = track_borrow("r1", &data);
        let r2 = track_borrow("r2", &data);
        println!("Immutable borrows: {:?}, {:?}", r1, r2);
        track_drop("r2");
        track_drop("r1");
    }
    
    // Mutable borrow
    {
        let r_mut = track_borrow_mut("r_mut", &mut data);
        r_mut.push(4);
        println!("After mutable borrow: {:?}", r_mut);
        track_drop("r_mut");
    }
    
    track_drop("data");
    println!();
}

/// Shared ownership with Rc
fn demo_shared_ownership() {
    println!("--- 3. Shared Ownership (Rc) ---");
    
    let rc1 = Rc::new(42);
    let rc1 = track_rc_new("rc1", rc1);
    println!("rc1 strong count: {}", Rc::strong_count(&rc1));
    
    let rc2 = Rc::clone(&rc1);
    let rc2 = track_rc_clone("rc2", "rc1", rc2);
    println!("After clone - rc1 count: {}, rc2 count: {}", 
             Rc::strong_count(&rc1), Rc::strong_count(&rc2));
    
    let rc3 = Rc::clone(&rc1);
    let _rc3 = track_rc_clone("rc3", "rc1", rc3);
    println!("After another clone - count: {}", Rc::strong_count(&rc1));
    
    track_drop("rc3");
    track_drop("rc2");
    track_drop("rc1");
    println!();
}

/// Interior mutability with RefCell
fn demo_interior_mutability() {
    println!("--- 4. Interior Mutability (RefCell) ---");
    
    let cell = RefCell::new(vec![1, 2, 3]);
    let cell = track_refcell_new("cell", cell);
    
    // Immutable borrow through RefCell
    {
        let borrowed = refcell_borrow!("cell_ref", "cell", cell.borrow());
        println!("RefCell borrowed: {:?}", *borrowed);
        refcell_drop!("cell_ref");
    }
    
    // Mutable borrow through RefCell
    {
        let mut borrowed = refcell_borrow_mut!("cell_mut", "cell", cell.borrow_mut());
        borrowed.push(4);
        println!("RefCell mutably borrowed: {:?}", *borrowed);
        refcell_drop!("cell_mut");
    }
    
    track_drop("cell");
    println!();
}
