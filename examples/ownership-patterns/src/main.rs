//! Ownership Patterns - Experience BorrowScope Runtime Tracking
//!
//! This example demonstrates Rust's ownership patterns with runtime tracking,
//! including RAII guards, filtering API, and pretty print summary.

use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    reset();
    println!("=== Ownership Patterns Demo ===\n");

    demo_basic_ownership();
    demo_raii_guards();
    demo_borrowing();
    demo_shared_ownership();
    demo_interior_mutability();
    demo_filtering_api();

    // Pretty print summary
    println!("\n");
    print_summary();

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

/// RAII guards for automatic drop tracking
fn demo_raii_guards() {
    println!("--- 2. RAII Guards (Auto Drop) ---");

    {
        // TrackGuard automatically calls track_drop when it goes out of scope
        let data = track_new_guard("guarded_data", vec![1, 2, 3]);
        println!("Created guarded_data: {:?}", *data);

        // BorrowGuard for automatic borrow tracking
        {
            let r = track_borrow_guard("guarded_ref", &*data);
            println!("Borrowed via guard: {:?}", *r);
        } // track_drop("guarded_ref") called automatically

        // BorrowMutGuard for mutable borrows
        // (can't use here since data is not mut, but shown for reference)
    } // track_drop("guarded_data") called automatically

    println!("Guards dropped automatically!\n");
}

/// Borrowing: immutable and mutable references
fn demo_borrowing() {
    println!("--- 3. Borrowing ---");

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
    println!("--- 4. Shared Ownership (Rc) ---");

    let rc1 = Rc::new(42);
    let rc1 = track_rc_new("rc1", rc1);
    println!("rc1 strong count: {}", Rc::strong_count(&rc1));

    let rc2 = Rc::clone(&rc1);
    let rc2 = track_rc_clone("rc2", "rc1", rc2);
    println!(
        "After clone - rc1 count: {}, rc2 count: {}",
        Rc::strong_count(&rc1),
        Rc::strong_count(&rc2)
    );

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
    println!("--- 5. Interior Mutability (RefCell) ---");

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

/// Filtering API demonstration
fn demo_filtering_api() {
    println!("--- 6. Filtering API ---");

    // Get counts
    let (new, borrow, mov, drop) = get_event_counts();
    println!("Event counts: {} new, {} borrow, {} move, {} drop", new, borrow, mov, drop);

    // Filter specific event types
    let borrows = get_borrow_events();
    println!("Total borrow events: {}", borrows.len());

    // Get events for specific variable
    let rc1_events = get_events_for_var("rc1");
    println!("Events for 'rc1': {}", rc1_events.len());

    // Custom filter
    let rc_events = get_events_filtered(|e| e.is_rc());
    println!("All Rc events: {}", rc_events.len());
    println!();
}
