//! Example: Extended Smart Pointer Tracking with Macro
//!
//! Demonstrates automatic instrumentation of Box, Pin, Cow, and Weak references.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};
use std::borrow::Cow;
use std::pin::Pin;
use std::rc::Rc;

fn main() {
    reset();
    println!("=== Extended Smart Pointer Tracking (Macro) ===\n");

    example_box();
    example_pin();
    example_cow();
    example_weak();

    println!("\n=== All Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}

#[trace_borrow]
fn example_box() {
    println!("--- Box Tracking ---");
    let boxed = Box::new(42);
    println!("Created boxed value: {}", *boxed);

    let boxed_string = Box::new(String::from("hello"));
    println!("Created boxed string: {}", *boxed_string);
}

#[trace_borrow]
fn example_pin() {
    println!("\n--- Pin Tracking ---");
    let pinned = Box::pin(100);
    println!("Created pinned value: {}", *pinned);

    let pinned_vec = Box::pin(vec![1, 2, 3]);
    println!("Created pinned vec: {:?}", &*pinned_vec);
}

#[trace_borrow]
fn example_cow() {
    println!("\n--- Cow Tracking ---");
    let borrowed: Cow<str> = Cow::Borrowed("static string");
    println!("Cow borrowed: {}", &*borrowed);

    let owned: Cow<str> = Cow::Owned(String::from("owned string"));
    println!("Cow owned: {}", &*owned);
}

#[trace_borrow]
fn example_weak() {
    println!("\n--- Weak Reference Tracking ---");
    let rc = Rc::new("shared data");
    println!("Created Rc: {:?}", &*rc);

    let weak = Rc::downgrade(&rc);
    println!("Created weak reference");

    if let Some(upgraded) = weak.upgrade() {
        println!("Upgraded weak: {:?}", &*upgraded);
    }
}
