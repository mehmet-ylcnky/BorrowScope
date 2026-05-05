//! Phase 6 tracking example - additional expression tracking
//!
//! Demonstrates: break, continue, struct, tuple, range, array, cast, closure, region tracking

use borrowscope_runtime::*;

fn main() {
    reset();

    // Break and continue tracking
    let loop_id = 1;
    track_loop_enter(loop_id, "loop", "main.rs:10");
    for i in 0..3 {
        track_loop_iteration(loop_id, i, "main.rs:11");
        if i == 1 {
            track_continue(100, None, "main.rs:13");
            continue;
        }
        if i == 2 {
            track_break(101, None, "main.rs:16");
            break;
        }
    }
    track_loop_exit(loop_id, "main.rs:19");

    // Labeled break/continue
    track_break(102, Some("outer"), "main.rs:22");
    track_continue(103, Some("inner"), "main.rs:23");

    // Struct creation
    track_struct_create(200, "Point", "main.rs:26");
    track_struct_create(201, "Config", "main.rs:27");

    // Tuple creation
    track_tuple_create(300, 2, "main.rs:30");
    track_tuple_create(301, 3, "main.rs:31");

    // Range expressions
    track_range(400, "half_open", "main.rs:34"); // 0..10
    track_range(401, "closed", "main.rs:35"); // 0..=10

    // Array creation
    track_array_create(500, 4, "main.rs:38");
    track_array_create(501, 0, "main.rs:39");

    // Type casts
    track_type_cast(600, "i64", "main.rs:42");
    track_type_cast(601, "f64", "main.rs:43");

    // Closure creation
    track_closure_create(700, "move", "main.rs:46");
    track_closure_create(701, "ref", "main.rs:47");

    // Let-else pattern
    track_let_else(800, "Some(x)", "main.rs:50");

    // Binary operations
    track_binary_op(900, "+", "main.rs:53");
    track_binary_op(901, "&&", "main.rs:54");

    // Region/scope tracking
    track_region_enter(1000, "inner_scope", "main.rs:57");
    track_region_exit(1000, "main.rs:59");

    // Print summary
    let events = get_events();
    println!("Phase 6 Tracking Example");
    println!("========================");
    println!("Total events: {}\n", events.len());

    for event in &events {
        println!("{:?}", event);
    }
}
