//! Control flow tracking example
//!
//! Demonstrates tracking loops, branches, match expressions, and returns.
//!
//! Run with: cargo run --example control_flow_tracking --features track

use borrowscope_runtime::*;

fn main() {
    reset();
    println!("=== Control Flow Tracking Example ===\n");

    // Loop tracking
    println!("--- Loop Tracking ---");
    track_loop_enter(1, "for", "main.rs:15");
    for i in 0..3 {
        track_loop_iteration(1, i, "main.rs:16");
        println!("  Iteration {}", i);
    }
    track_loop_exit(1, "main.rs:19");

    // Match tracking
    println!("\n--- Match Tracking ---");
    let value = 42;
    track_match_enter(1, "main.rs:24");
    let result = match value {
        0 => {
            track_match_arm(1, 0, "0", "main.rs:26");
            "zero"
        }
        1..=10 => {
            track_match_arm(1, 1, "1..=10", "main.rs:30");
            "small"
        }
        _ => {
            track_match_arm(1, 2, "_", "main.rs:34");
            "large"
        }
    };
    track_match_exit(1, "main.rs:38");
    println!("  Match result: {}", result);

    // Branch tracking
    println!("\n--- Branch Tracking ---");
    if value > 10 {
        track_branch(1, "then", "main.rs:44");
        println!("  Took 'then' branch");
    } else {
        track_branch(1, "else", "main.rs:47");
        println!("  Took 'else' branch");
    }

    // Return tracking
    println!("\n--- Return Tracking ---");
    track_return(1, true, "main.rs:53");
    println!("  Return with value tracked");

    // Try operator tracking
    println!("\n--- Try Operator Tracking ---");
    track_try(1, "main.rs:58");
    println!("  Try operator tracked");

    // Print events
    println!("\n=== Recorded Events ===");
    for event in get_events() {
        println!("{:?}", event);
    }
}
