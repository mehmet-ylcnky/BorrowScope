//! Advanced usage example with complex tracking scenarios

use borrowscope_runtime::*;

fn main() {
    println!("=== BorrowScope Runtime - Advanced Usage ===\n");

    reset();

    // Scenario 1: Multiple borrows
    println!("Scenario 1: Multiple immutable borrows");
    let data = track_new("data", vec![1, 2, 3, 4, 5]);
    let r1 = track_borrow("r1", &data);
    let r2 = track_borrow("r2", &data);
    println!("  data.len() = {}", data.len());
    println!("  r1.len() = {}", r1.len());
    println!("  r2.len() = {}", r2.len());
    track_drop("r2");
    track_drop("r1");
    track_drop("data");

    // Scenario 2: Mutable borrow
    println!("\nScenario 2: Mutable borrow and modification");
    let mut counter = track_new("counter", 0);
    let r_mut = track_borrow_mut("r_mut", &mut counter);
    *r_mut += 10;
    println!("  counter after modification = {}", r_mut);
    track_drop("r_mut");
    track_drop("counter");

    // Scenario 3: Move semantics
    println!("\nScenario 3: Ownership move");
    let s1 = track_new("s1", String::from("Hello"));
    let s2 = track_move("s1", "s2", s1);
    println!("  s2 = {}", s2);
    track_drop("s2");

    // Scenario 4: Nested scopes
    println!("\nScenario 4: Nested scopes");
    let outer = track_new("outer", 42);
    {
        let inner = track_borrow("inner", &outer);
        println!("  inner scope: {}", inner);
        track_drop("inner");
    }
    println!("  outer scope: {}", outer);
    track_drop("outer");

    // Generate report
    let events = get_events();
    let graph = get_graph();
    let stats = graph.stats();

    println!("\n=== Tracking Summary ===");
    println!("Total events: {}", events.len());
    println!("Variables tracked: {}", stats.total_variables);
    println!("Relationships: {}", stats.total_relationships);
    println!("Immutable borrows: {}", stats.immutable_borrows);
    println!("Mutable borrows: {}", stats.mutable_borrows);

    // Export detailed report
    let export_path = std::env::temp_dir().join("borrowscope_advanced.json");
    match export_json(&export_path) {
        Ok(()) => println!("\nDetailed report exported to: {}", export_path.display()),
        Err(e) => eprintln!("Export failed: {}", e),
    }
}
