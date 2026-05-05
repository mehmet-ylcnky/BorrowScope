//! Example: Box Tracking
//!
//! Demonstrates tracking of Box operations including creation,
//! into_raw, and from_raw for heap allocation management.

use borrowscope_runtime::*;

fn main() {
    reset();
    println!("=== Box Tracking Example ===\n");

    // Basic Box creation
    println!("--- Box Creation ---");
    let boxed_int = track_box_new("boxed_int", "main.rs:12", Box::new(42));
    println!("Created boxed_int: {}", *boxed_int);

    let boxed_string = track_box_new(
        "boxed_string",
        "main.rs:15",
        Box::new(String::from("hello")),
    );
    println!("Created boxed_string: {}", *boxed_string);

    let boxed_vec = track_box_new("boxed_vec", "main.rs:18", Box::new(vec![1, 2, 3]));
    println!("Created boxed_vec: {:?}", *boxed_vec);

    // Box into_raw and from_raw (unsafe pattern)
    println!("\n--- Box into_raw / from_raw ---");
    let data = Box::new(100);
    println!("Original boxed value: {}", *data);

    let ptr = track_box_into_raw("data", "main.rs:26", Box::into_raw(data));
    println!("Converted to raw pointer: {:?}", ptr);

    // Safety: ptr was just created from Box::into_raw
    let recovered = unsafe { track_box_from_raw("recovered", "main.rs:30", Box::from_raw(ptr)) };
    println!("Recovered from raw pointer: {}", *recovered);

    // Trait object in Box
    println!("\n--- Box with Trait Object ---");
    trait Speak {
        fn speak(&self) -> &str;
    }
    struct Dog;
    impl Speak for Dog {
        fn speak(&self) -> &str {
            "woof!"
        }
    }

    let animal: Box<dyn Speak> = track_box_new("animal", "main.rs:44", Box::new(Dog));
    println!("Animal says: {}", animal.speak());

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
