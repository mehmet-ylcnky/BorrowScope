//! Example: Cow (Clone-on-Write) Tracking
//!
//! Demonstrates tracking of Cow operations including Borrowed,
//! Owned, and the clone-on-write behavior with to_mut.

use borrowscope_runtime::*;
use std::borrow::Cow;

fn main() {
    reset();
    println!("=== Cow (Clone-on-Write) Tracking Example ===\n");

    // Cow::Borrowed - no allocation
    println!("--- Cow::Borrowed ---");
    let static_str = "I'm a static string";
    let cow1: Cow<str> = track_cow_borrowed("cow1", "main.rs:14", Cow::Borrowed(static_str));
    println!("cow1 (borrowed): {:?}", &*cow1);
    println!("Is borrowed: {}", matches!(cow1, Cow::Borrowed(_)));

    // Cow::Owned - owns the data
    println!("\n--- Cow::Owned ---");
    let cow2: Cow<str> =
        track_cow_owned("cow2", "main.rs:20", Cow::Owned(String::from("I'm owned")));
    println!("cow2 (owned): {:?}", &*cow2);
    println!("Is owned: {}", matches!(cow2, Cow::Owned(_)));

    // Clone-on-write with to_mut
    println!("\n--- Clone-on-Write (to_mut) ---");
    let original = "original data";
    let mut cow3: Cow<str> = Cow::Borrowed(original);
    println!(
        "Before to_mut - borrowed: {}",
        matches!(cow3, Cow::Borrowed(_))
    );

    // This triggers a clone because cow3 is borrowed
    track_cow_to_mut("cow3", true, "main.rs:31");
    cow3.to_mut().push_str(" + modified");
    println!("After to_mut - owned: {}", matches!(cow3, Cow::Owned(_)));
    println!("Modified value: {:?}", &*cow3);
    println!("Original unchanged: {:?}", original);

    // to_mut on already owned - no clone
    println!("\n--- to_mut on Owned (no clone) ---");
    let mut cow4: Cow<str> = Cow::Owned(String::from("already owned"));
    track_cow_to_mut("cow4", false, "main.rs:41");
    cow4.to_mut().push_str("!");
    println!("Modified owned: {:?}", &*cow4);

    // Real-world pattern: conditional modification
    println!("\n--- Real-world Pattern ---");
    fn process(input: &str) -> Cow<str> {
        if input.contains("bad") {
            track_cow_owned(
                "result",
                "process",
                Cow::Owned(input.replace("bad", "good")),
            )
        } else {
            track_cow_borrowed("result", "process", Cow::Borrowed(input))
        }
    }

    let clean = process("hello world");
    println!(
        "Clean input -> borrowed: {}",
        matches!(clean, Cow::Borrowed(_))
    );

    let dirty = process("hello bad world");
    println!(
        "Dirty input -> owned: {}, value: {:?}",
        matches!(dirty, Cow::Owned(_)),
        &*dirty
    );

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
