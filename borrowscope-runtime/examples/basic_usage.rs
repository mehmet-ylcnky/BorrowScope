//! Basic usage example demonstrating variable tracking

use borrowscope_runtime::*;

fn main() {
    println!("=== BorrowScope Runtime - Basic Usage ===\n");

    // Reset tracker to start fresh
    reset();

    // Track a simple variable
    let x = track_new("x", 42);
    println!("Created variable x = {}", x);

    // Track a borrow
    let r = track_borrow("r", &x);
    println!("Borrowed x as r = {}", r);
    track_drop("r");

    // Track another variable
    let y = track_new("y", 100);
    println!("Created variable y = {}", y);

    // Clean up
    track_drop("y");
    track_drop("x");

    // Get events
    let events = get_events();
    println!("\nTracked {} events:", events.len());
    for (i, event) in events.iter().enumerate() {
        if event.is_new() {
            println!("  {}. New variable", i + 1);
        } else if event.is_borrow() {
            println!("  {}. Borrow", i + 1);
        } else if event.is_drop() {
            println!("  {}. Drop", i + 1);
        }
    }

    // Build graph
    let graph = get_graph();
    println!("\nOwnership graph:");
    println!("  Variables: {}", graph.nodes.len());
    println!("  Relationships: {}", graph.edges.len());

    // Export to JSON
    let temp_path = std::env::temp_dir().join("borrowscope_basic.json");
    match export_json(&temp_path) {
        Ok(()) => println!("\nExported to: {}", temp_path.display()),
        Err(e) => eprintln!("Export failed: {}", e),
    }
}
