//! Example: Weak Reference Tracking
//!
//! Demonstrates tracking of Weak references from Rc and Arc,
//! including downgrade, clone, and upgrade operations.

use borrowscope_runtime::*;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    reset();
    println!("=== Weak Reference Tracking Example ===\n");

    // Rc weak references
    println!("--- Rc Weak References ---");
    let rc = Rc::new("shared data");
    println!("Created Rc with value: {:?}", rc);

    let weak1 = track_weak_new("weak1", "rc", "main.rs:15", Rc::downgrade(&rc));
    println!(
        "Created weak1 from rc (weak_count: {})",
        Rc::weak_count(&rc)
    );

    let weak2 = track_weak_clone("weak2", "weak1", "main.rs:18", weak1.clone());
    println!(
        "Cloned weak2 from weak1 (weak_count: {})",
        Rc::weak_count(&rc)
    );

    // Upgrade while Rc is alive
    let upgraded = track_weak_upgrade("weak1", "main.rs:22", weak1.upgrade());
    println!("Upgraded weak1: {:?}", upgraded.as_ref().map(|r| &**r));

    drop(upgraded);
    drop(rc);

    // Upgrade after Rc is dropped
    let failed = track_weak_upgrade("weak2", "main.rs:29", weak2.upgrade());
    println!("Upgraded weak2 after drop: {:?}", failed);

    // Arc weak references (thread-safe)
    println!("\n--- Arc Weak References ---");
    let arc = Arc::new(42);
    let weak_arc = track_weak_new_sync("weak_arc", "arc", "main.rs:35", Arc::downgrade(&arc));
    println!("Created weak_arc from Arc (value: {})", *arc);

    let upgraded_arc = track_weak_upgrade_sync("weak_arc", "main.rs:38", weak_arc.upgrade());
    println!("Upgraded weak_arc: {:?}", upgraded_arc);

    // Print events
    println!("\n=== Tracked Events ===");
    for (i, event) in get_events().iter().enumerate() {
        println!("{:3}: {:?}", i, event);
    }
}
