//! Basic ownership tracking: new, move, drop, borrow
//!
//! Run with: cargo run --example ownership

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn basic_new_and_drop() {
    let x = String::from("hello");
    let y = 42;
    let z = vec![1, 2, 3];
    // All variables dropped at end of scope
}

#[trace_borrow]
fn ownership_move() {
    let s1 = String::from("owned");
    let s2 = s1; // Move occurs here
    // s1 is no longer valid, s2 owns the data
    println!("s2 = {}", s2);
}

#[trace_borrow]
fn immutable_borrow() {
    let data = vec![1, 2, 3, 4, 5];
    let r1 = &data; // Immutable borrow
    let r2 = &data; // Multiple immutable borrows allowed
    println!("r1: {:?}, r2: {:?}", r1, r2);
}

#[trace_borrow]
fn mutable_borrow() {
    let mut data = vec![1, 2, 3];
    let r = &mut data; // Mutable borrow
    r.push(4);
    println!("Modified: {:?}", r);
}

#[trace_borrow]
fn borrow_then_move() {
    let s = String::from("hello");
    {
        let r = &s; // Borrow in inner scope
        println!("Borrowed: {}", r);
    } // Borrow ends here
    let s2 = s; // Now we can move
    println!("Moved: {}", s2);
}

#[trace_borrow]
fn nested_scopes() {
    let outer = String::from("outer");
    {
        let inner = String::from("inner");
        {
            let innermost = String::from("innermost");
            println!("{} {} {}", outer, inner, innermost);
        } // innermost dropped
    } // inner dropped
} // outer dropped

fn main() {
    println!("=== Basic New and Drop ===");
    reset();
    basic_new_and_drop();
    print_events("basic_new_and_drop");

    println!("\n=== Ownership Move ===");
    reset();
    ownership_move();
    print_events("ownership_move");

    println!("\n=== Immutable Borrow ===");
    reset();
    immutable_borrow();
    print_events("immutable_borrow");

    println!("\n=== Mutable Borrow ===");
    reset();
    mutable_borrow();
    print_events("mutable_borrow");

    println!("\n=== Borrow Then Move ===");
    reset();
    borrow_then_move();
    print_events("borrow_then_move");

    println!("\n=== Nested Scopes ===");
    reset();
    nested_scopes();
    print_events("nested_scopes");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
