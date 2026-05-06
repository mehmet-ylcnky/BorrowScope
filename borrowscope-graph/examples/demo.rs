use borrowscope_graph::export::*;
use borrowscope_graph::stats::*;
use borrowscope_graph::*;

fn main() {
    let mut g = OwnershipGraph::new();

    let data = g.add_variable("data", "Vec<i32>", 0);
    let rc = g.add_variable("rc", "Rc<Vec<i32>>", 10);
    let rc_clone1 = g.add_variable("rc_clone1", "Rc<Vec<i32>>", 20);
    let rc_clone2 = g.add_variable("rc_clone2", "Rc<Vec<i32>>", 30);
    let borrow_ref = g.add_variable("r", "&Vec<i32>", 40);
    let mut_ref = g.add_variable("m", "&mut Vec<i32>", 70);
    let moved = g.add_variable("moved_data", "Vec<i32>", 100);

    g.add_rc_clone(rc_clone1, rc, 2, 20);
    g.add_rc_clone(rc_clone2, rc, 3, 30);
    let eid = g.add_borrow(borrow_ref, data, false, 40);
    g.end_edge(eid, 60);
    let eid2 = g.add_borrow(mut_ref, data, true, 70);
    g.end_edge(eid2, 90);
    g.add_move(data, moved, 100);

    g.mark_dropped(borrow_ref, 60);
    g.mark_dropped(mut_ref, 90);
    g.mark_dropped(rc_clone2, 80);
    g.mark_dropped(rc_clone1, 110);
    g.mark_dropped(rc, 120);
    g.mark_dropped(moved, 130);

    // DOT output
    println!("=== GRAPHVIZ DOT ===\n");
    println!("{}", to_dot(&g, &DotOptions::default()));

    // D3 JSON
    println!("=== D3.js JSON ===\n");
    println!("{}", to_d3_json(&g).unwrap());

    // Stats
    let stats = statistics(&g);
    println!("\n=== STATISTICS ===");
    println!(
        "Nodes: {} ({} variables)",
        stats.total_nodes, stats.variable_nodes
    );
    println!("Edges: {} total", stats.total_edges);
    println!("  Shared borrows: {}", stats.shared_borrows);
    println!("  Mutable borrows: {}", stats.mutable_borrows);
    println!("  Moves: {}", stats.moves);
    println!("  Rc clones: {}", stats.rc_clones);
}
