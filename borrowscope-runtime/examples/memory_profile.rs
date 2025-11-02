use borrowscope_runtime::*;
use std::time::Instant;

fn main() {
    println!("=== BorrowScope Memory Profiling ===\n");

    // Event size analysis
    println!("Event Size Analysis:");
    println!("  Event enum size: {} bytes", std::mem::size_of::<Event>());
    println!("  Event alignment: {} bytes", std::mem::align_of::<Event>());
    println!();

    // Small workload
    profile_workload("Small (100 ops)", 100);

    // Medium workload
    profile_workload("Medium (10,000 ops)", 10_000);

    // Large workload
    profile_workload("Large (100,000 ops)", 100_000);

    // Very large workload
    profile_workload("Very Large (1,000,000 ops)", 1_000_000);

    // Mixed operations
    profile_mixed_operations();

    // Smart pointer operations
    profile_smart_pointers();

    // Graph building
    profile_graph_building();
}

fn profile_workload(name: &str, count: usize) {
    println!("=== {} ===", name);

    reset();

    let start = Instant::now();
    for i in 0..count {
        track_new(&format!("var_{}", i), i);
    }
    let duration = start.elapsed();

    let events = get_events();
    let event_count = events.len();
    let memory_bytes = event_count * std::mem::size_of::<Event>();
    let memory_kb = memory_bytes / 1024;
    let memory_mb = memory_kb / 1024;

    println!("  Operations: {}", count);
    println!("  Events tracked: {}", event_count);
    println!("  Time: {:?}", duration);
    println!("  Avg per op: {:?}", duration / count as u32);
    println!(
        "  Memory (approx): {} bytes ({} KB, {} MB)",
        memory_bytes, memory_kb, memory_mb
    );
    println!("  Bytes per event: {}", memory_bytes / event_count);
    println!();
}

fn profile_mixed_operations() {
    println!("=== Mixed Operations (10,000 ops) ===");

    reset();

    let start = Instant::now();
    for i in 0..10_000 {
        let name = format!("var_{}", i);
        let x = track_new(&name, i);
        let _r = track_borrow("ref", &x);
        track_move(&name, &format!("moved_{}", i), x);
        track_drop(&format!("moved_{}", i));
    }
    let duration = start.elapsed();

    let events = get_events();
    let event_count = events.len();
    let memory_bytes = event_count * std::mem::size_of::<Event>();
    let memory_kb = memory_bytes / 1024;

    println!("  Operations: 40,000 (4 per iteration)");
    println!("  Events tracked: {}", event_count);
    println!("  Time: {:?}", duration);
    println!("  Avg per op: {:?}", duration / 40_000);
    println!("  Memory: {} bytes ({} KB)", memory_bytes, memory_kb);
    println!();
}

fn profile_smart_pointers() {
    println!("=== Smart Pointer Operations (1,000 ops) ===");

    reset();

    let start = Instant::now();
    for i in 0..1_000 {
        let rc = std::rc::Rc::new(i);
        let tracked_rc = track_rc_new(&format!("rc_{}", i), rc);

        let cloned = std::rc::Rc::clone(&tracked_rc);
        let _tracked_clone =
            track_rc_clone(&format!("rc_clone_{}", i), &format!("rc_{}", i), cloned);

        let arc = std::sync::Arc::new(i);
        let tracked_arc = track_arc_new(&format!("arc_{}", i), arc);

        let cloned_arc = std::sync::Arc::clone(&tracked_arc);
        let _tracked_arc_clone = track_arc_clone(
            &format!("arc_clone_{}", i),
            &format!("arc_{}", i),
            cloned_arc,
        );
    }
    let duration = start.elapsed();

    let events = get_events();
    let event_count = events.len();
    let memory_bytes = event_count * std::mem::size_of::<Event>();
    let memory_kb = memory_bytes / 1024;

    println!("  Rc::new operations: 1,000");
    println!("  Rc::clone operations: 1,000");
    println!("  Arc::new operations: 1,000");
    println!("  Arc::clone operations: 1,000");
    println!("  Total events: {}", event_count);
    println!("  Time: {:?}", duration);
    println!("  Avg per op: {:?}", duration / 4_000);
    println!("  Memory: {} bytes ({} KB)", memory_bytes, memory_kb);
    println!();
}

fn profile_graph_building() {
    println!("=== Graph Building Performance ===");

    for size in [100, 1_000, 10_000] {
        reset();

        // Generate events
        let gen_start = Instant::now();
        for i in 0..size {
            let name = format!("var_{}", i);
            let x = track_new(&name, i);
            let _r = track_borrow("ref", &x);
            track_drop(&name);
        }
        let gen_duration = gen_start.elapsed();

        let events = get_events();

        // Build graph
        let graph_start = Instant::now();
        let graph = build_graph(&events);
        let graph_duration = graph_start.elapsed();

        println!("  Size: {} operations", size);
        println!("    Event generation: {:?}", gen_duration);
        println!("    Graph building: {:?}", graph_duration);
        println!("    Graph nodes: {}", graph.nodes.len());
        println!("    Graph edges: {}", graph.edges.len());
        println!("    Total time: {:?}", gen_duration + graph_duration);
        println!();
    }
}
