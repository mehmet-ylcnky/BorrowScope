//! Graph Visualization - ASCII visualization of BorrowScope graph structures
//!
//! Demonstrates all graph APIs with visual ASCII output.

use borrowscope_graph::{OwnershipGraph, Variable, Relationship};
use borrowscope_runtime::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     BorrowScope Graph Visualization - ASCII Demo             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    demo_basic_graph();
    demo_borrow_relationships();
    demo_rc_arc_graph();
    demo_graph_traversal();
    demo_conflict_detection();
    demo_timeline_visualization();
    demo_graph_statistics();
    demo_serialization();
}

// ============================================================================
// 1. Basic Graph Construction
// ============================================================================
fn demo_basic_graph() {
    println!("━━━ 1. Basic Graph Construction ━━━\n");

    let mut graph = OwnershipGraph::new();

    // Add variables
    graph.add_variable(var(1, "x", "i32", 0, None));
    graph.add_variable(var(2, "y", "String", 10, None));
    graph.add_variable(var(3, "z", "Vec<i32>", 20, Some(100)));

    println!("Graph with 3 variables:");
    println!();
    print_nodes_ascii(&graph);
    println!();

    println!("API: graph.add_variable(Variable {{ id, name, type_name, created_at, dropped_at }})");
    println!("     graph.node_count() = {}", graph.node_count());
    println!();
}

// ============================================================================
// 2. Borrow Relationships
// ============================================================================
fn demo_borrow_relationships() {
    println!("━━━ 2. Borrow Relationships ━━━\n");

    let mut graph = OwnershipGraph::new();

    // Owner and borrowers
    graph.add_variable(var(1, "data", "Vec<i32>", 0, Some(100)));
    graph.add_variable(var(2, "r1", "&Vec<i32>", 10, Some(50)));
    graph.add_variable(var(3, "r2", "&Vec<i32>", 20, Some(60)));
    graph.add_variable(var(4, "m", "&mut Vec<i32>", 70, Some(90)));

    graph.add_borrow(2, 1, false, 10); // r1 borrows data (immut)
    graph.add_borrow(3, 1, false, 20); // r2 borrows data (immut)
    graph.add_borrow(4, 1, true, 70);  // m borrows data (mut)

    println!("Ownership graph with borrows:");
    println!();
    print_borrow_graph_ascii(&graph, 1);
    println!();

    println!("API: graph.add_borrow(borrower_id, owner_id, is_mut, timestamp)");
    println!("     graph.borrowers_of(1) = {:?}", 
             graph.borrowers_of(1).iter().map(|v| &v.name).collect::<Vec<_>>());
    println!();
}

// ============================================================================
// 3. Rc/Arc Clone Relationships
// ============================================================================
fn demo_rc_arc_graph() {
    println!("━━━ 3. Reference Counting Graph ━━━\n");

    let mut graph = OwnershipGraph::new();

    // Rc chain
    graph.add_variable(var(1, "rc1", "Rc<Data>", 0, Some(100)));
    graph.add_variable(var(2, "rc2", "Rc<Data>", 10, Some(80)));
    graph.add_variable(var(3, "rc3", "Rc<Data>", 20, Some(60)));

    graph.add_rc_clone(2, 1, 2, 10);
    graph.add_rc_clone(3, 1, 3, 20);

    // Arc chain
    graph.add_variable(var(4, "arc1", "Arc<Data>", 0, Some(100)));
    graph.add_variable(var(5, "arc2", "Arc<Data>", 10, Some(90)));

    graph.add_arc_clone(5, 4, 2, 10);

    println!("Reference counting relationships:");
    println!();
    print_rc_graph_ascii();
    println!();

    println!("API: graph.add_rc_clone(clone_id, original_id, strong_count, timestamp)");
    println!("     graph.add_arc_clone(clone_id, original_id, strong_count, timestamp)");
    println!();
}

// ============================================================================
// 4. Graph Traversal Algorithms
// ============================================================================
fn demo_graph_traversal() {
    println!("━━━ 4. Graph Traversal Algorithms ━━━\n");

    let mut graph = OwnershipGraph::new();

    // Build a tree-like structure
    graph.add_variable(var(1, "root", "Node", 0, None));
    graph.add_variable(var(2, "child1", "Node", 10, None));
    graph.add_variable(var(3, "child2", "Node", 20, None));
    graph.add_variable(var(4, "leaf1", "Node", 30, None));
    graph.add_variable(var(5, "leaf2", "Node", 40, None));

    graph.add_borrow(2, 1, false, 10);
    graph.add_borrow(3, 1, false, 20);
    graph.add_borrow(4, 2, false, 30);
    graph.add_borrow(5, 3, false, 40);

    println!("Tree structure:");
    print_tree_ascii();
    println!();

    // DFS
    let dfs = graph.dfs_from(1);
    println!("DFS from root: {:?}", ids_to_names(&graph, &dfs));

    // BFS
    let bfs = graph.bfs_from(1);
    println!("BFS from root: {:?}", ids_to_names(&graph, &bfs));

    // Shortest path
    if let Some(path) = graph.shortest_path(1, 5) {
        println!("Shortest path root→leaf2: {:?}", ids_to_names(&graph, &path));
    }

    // Topological order
    if let Ok(topo) = graph.topological_order() {
        println!("Topological order: {:?}", ids_to_names(&graph, &topo));
    }

    // Can reach
    println!("Can root reach leaf2? {}", graph.can_reach(1, 5));
    println!("Can leaf1 reach leaf2? {}", graph.can_reach(4, 5));

    println!();
    println!("API: dfs_from(id), bfs_from(id), shortest_path(from, to)");
    println!("     topological_order(), can_reach(from, to)");
    println!();
}

// ============================================================================
// 5. Conflict Detection
// ============================================================================
fn demo_conflict_detection() {
    println!("━━━ 5. Conflict Detection ━━━\n");

    let mut graph = OwnershipGraph::new();

    graph.add_variable(var(1, "data", "Vec<i32>", 0, Some(100)));
    graph.add_variable(var(2, "r", "&Vec<i32>", 10, Some(60)));
    graph.add_variable(var(3, "m", "&mut Vec<i32>", 30, Some(80)));

    graph.add_borrow(2, 1, false, 10);
    graph.add_borrow(3, 1, true, 30);

    println!("Conflict scenario:");
    print_conflict_ascii();
    println!();

    // Find conflicts
    let conflicts = graph.find_conflicts_optimized();
    println!("Conflicts found: {}", conflicts.len());
    for c in &conflicts {
        println!("  ✗ {}", c.format(&graph));
        println!("    Time: {} - {}", c.time_range.0, c.time_range.1);
    }

    // Active borrows at time
    println!();
    println!("Active borrows at different times:");
    for t in [15, 35, 55, 75] {
        let active = graph.active_borrows_at_time(1, t);
        let names: Vec<_> = active.iter()
            .filter_map(|(id, is_mut)| {
                graph.get_variable(*id).map(|v| {
                    format!("{}({})", v.name, if *is_mut { "mut" } else { "imm" })
                })
            })
            .collect();
        println!("  t={}: [{}]", t, names.join(", "));
    }

    println!();
    println!("API: find_conflicts_optimized(), active_borrows_at_time(owner_id, time)");
    println!("     check_conflicts_at(owner_id, time), conflict_timeline(owner_id)");
    println!();
}

// ============================================================================
// 6. Timeline Visualization
// ============================================================================
fn demo_timeline_visualization() {
    println!("━━━ 6. Timeline Visualization ━━━\n");

    let mut graph = OwnershipGraph::new();

    graph.add_variable(var(1, "a", "i32", 0, Some(80)));
    graph.add_variable(var(2, "b", "i32", 20, Some(100)));
    graph.add_variable(var(3, "r_a", "&i32", 10, Some(50)));
    graph.add_variable(var(4, "r_b", "&i32", 30, Some(70)));
    graph.add_variable(var(5, "m_a", "&mut i32", 60, Some(75)));

    graph.add_borrow(3, 1, false, 10);
    graph.add_borrow(4, 2, false, 30);
    graph.add_borrow(5, 1, true, 60);

    print_timeline_ascii(&graph);
    println!();

    println!("API: graph.is_alive(id, timestamp)");
    println!("     variable.created_at, variable.dropped_at");
    println!();
}

// ============================================================================
// 7. Graph Statistics
// ============================================================================
fn demo_graph_statistics() {
    println!("━━━ 7. Graph Statistics ━━━\n");

    let mut graph = OwnershipGraph::new();

    // Build a complex graph
    for i in 1..=5 {
        graph.add_variable(var(i, &format!("v{}", i), "T", i as u64 * 10, None));
    }
    graph.add_borrow(2, 1, false, 20);
    graph.add_borrow(3, 1, false, 30);
    graph.add_borrow(4, 2, true, 40);
    graph.add_rc_clone(5, 1, 2, 50);

    let stats = graph.statistics();

    println!("┌─────────────────────────────────┐");
    println!("│       Graph Statistics          │");
    println!("├─────────────────────────────────┤");
    println!("│ Total variables:    {:>10} │", stats.total_variables);
    println!("│ Alive variables:    {:>10} │", stats.alive_variables);
    println!("│ Total edges:        {:>10} │", stats.total_edges);
    println!("│ Immutable borrows:  {:>10} │", stats.immutable_borrows);
    println!("│ Mutable borrows:    {:>10} │", stats.mutable_borrows);
    println!("│ Moves:              {:>10} │", stats.moves);
    println!("│ Rc clones:          {:>10} │", stats.rc_clones);
    println!("│ Arc clones:         {:>10} │", stats.arc_clones);
    println!("│ RefCell borrows:    {:>10} │", stats.refcell_borrows);
    println!("└─────────────────────────────────┘");
    println!();

    // Connected components
    let components = graph.connected_components();
    println!("Connected components: {}", components.len());
    for (i, comp) in components.iter().enumerate() {
        println!("  Component {}: {:?}", i + 1, ids_to_names(&graph, comp));
    }

    // Validation
    match graph.validate() {
        Ok(()) => println!("\n✓ Graph validation passed"),
        Err(errors) => {
            println!("\n✗ Validation errors:");
            for e in errors {
                println!("  - {}", e);
            }
        }
    }

    println!();
    println!("API: statistics(), connected_components(), validate(), has_cycles()");
    println!();
}

// ============================================================================
// 8. Serialization
// ============================================================================
fn demo_serialization() {
    println!("━━━ 8. Serialization Formats ━━━\n");

    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "x", "i32", 0, Some(50)));
    graph.add_variable(var(2, "r", "&i32", 10, Some(40)));
    graph.add_borrow(2, 1, false, 10);

    // JSON
    println!("JSON Export:");
    println!("─────────────");
    if let Ok(json) = graph.to_json() {
        // Print first 500 chars
        let preview: String = json.chars().take(500).collect();
        println!("{}", preview);
        if json.len() > 500 {
            println!("... ({} more chars)", json.len() - 500);
        }
    }
    println!();

    // DOT format
    println!("DOT (Graphviz) Export:");
    println!("──────────────────────");
    let dot = graph.to_dot();
    println!("{}", dot);

    println!("API: to_json(), to_json_compact(), to_dot(), to_messagepack()");
    println!("     from_json(str), export(), export_with_metadata()");
    println!();

    // Export runtime events
    reset();
    let x = track_new("demo", 42);
    let r = track_borrow("ref", &x);
    track_drop("ref");
    track_drop("demo");
    drop(r);
    drop(x);

    let path = std::env::temp_dir().join("graph-visualization.json");
    export_json(&path).unwrap();
    println!("Runtime events exported to: {}", path.display());
}

// ============================================================================
// ASCII Drawing Helpers
// ============================================================================

fn var(id: usize, name: &str, type_name: &str, created: u64, dropped: Option<u64>) -> Variable {
    Variable {
        id,
        name: name.into(),
        type_name: type_name.into(),
        created_at: created,
        dropped_at: dropped,
        scope_depth: 0,
    }
}

fn ids_to_names(graph: &OwnershipGraph, ids: &[usize]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| graph.get_variable(*id).map(|v| v.name.clone()))
        .collect()
}

fn print_nodes_ascii(graph: &OwnershipGraph) {
    println!("    ┌─────────────────────────────────────────┐");
    println!("    │              Variables                  │");
    println!("    ├─────────────────────────────────────────┤");
    for v in graph.all_variables() {
        let status = if v.dropped_at.is_some() { "dropped" } else { "alive" };
        println!("    │  [{}] {} : {} ({})  ", v.id, v.name, v.type_name, status);
    }
    println!("    └─────────────────────────────────────────┘");
}

fn print_borrow_graph_ascii(graph: &OwnershipGraph, owner_id: usize) {
    let owner = graph.get_variable(owner_id).unwrap();
    let borrowers = graph.borrowers_of(owner_id);

    println!("                    ┌──────────────┐");
    println!("                    │  {}  │", owner.name);
    println!("                    │  {}   │", owner.type_name);
    println!("                    └──────┬───────┘");
    println!("                           │");
    println!("           ┌───────────────┼───────────────┐");

    for (i, b) in borrowers.iter().enumerate() {
        let prefix = if i == 0 { "     " } else { "          " };
        let borrow_type = if b.type_name.contains("mut") { "&mut" } else { "&" };
        print!("{}┌────────┐", prefix);
    }
    println!();

    for b in &borrowers {
        print!("     │ {:^6} │", b.name);
    }
    println!();

    for b in &borrowers {
        let borrow_type = if b.type_name.contains("mut") { "(mut)" } else { "(imm)" };
        print!("     │ {:^6} │", borrow_type);
    }
    println!();

    for _ in &borrowers {
        print!("     └────────┘");
    }
    println!();
}

fn print_rc_graph_ascii() {
    println!("    Rc<Data> sharing:");
    println!();
    println!("         ┌────────┐");
    println!("         │  rc1   │ ◄─── Original (count=1)");
    println!("         └────┬───┘");
    println!("              │ clone");
    println!("         ┌────┴───┐");
    println!("         │  rc2   │ ◄─── Clone (count=2)");
    println!("         └────┬───┘");
    println!("              │ clone");
    println!("         ┌────┴───┐");
    println!("         │  rc3   │ ◄─── Clone (count=3)");
    println!("         └────────┘");
    println!();
    println!("    Arc<Data> sharing:");
    println!();
    println!("         ┌────────┐      ┌────────┐");
    println!("         │  arc1  │──────│  arc2  │");
    println!("         └────────┘      └────────┘");
    println!("              Thread-safe clones");
}

fn print_tree_ascii() {
    println!("              ┌────────┐");
    println!("              │  root  │");
    println!("              └───┬────┘");
    println!("           ┌──────┴──────┐");
    println!("      ┌────┴───┐    ┌────┴───┐");
    println!("      │ child1 │    │ child2 │");
    println!("      └───┬────┘    └───┬────┘");
    println!("     ┌────┴───┐    ┌────┴───┐");
    println!("     │ leaf1  │    │ leaf2  │");
    println!("     └────────┘    └────────┘");
}

fn print_conflict_ascii() {
    println!("    Timeline:");
    println!("    ─────────────────────────────────────────────────────────────");
    println!("    t=0        t=10       t=30       t=60       t=80      t=100");
    println!("    │          │          │          │          │          │");
    println!("    ├──────────┴──────────┴──────────┴──────────┴──────────┤ data");
    println!("    │          ├──────────────────────┤                    │ r (imm)");
    println!("    │                     ├──────────────────────┤         │ m (mut)");
    println!("    │                     │◄─ CONFLICT ─►│                 │");
    println!("    ─────────────────────────────────────────────────────────────");
}

fn print_timeline_ascii(graph: &OwnershipGraph) {
    println!("    Variable Lifetimes:");
    println!("    ════════════════════════════════════════════════════════════");
    println!("    t=0   10   20   30   40   50   60   70   80   90   100");
    println!("    │     │    │    │    │    │    │    │    │    │    │");

    for v in graph.all_variables() {
        let start = v.created_at as usize / 10;
        let end = v.dropped_at.unwrap_or(100) as usize / 10;

        let mut line = String::from("    ");
        for i in 0..=10 {
            if i < start {
                line.push_str("     ");
            } else if i == start {
                line.push_str("├");
            } else if i < end {
                line.push_str("─────");
            } else if i == end {
                line.push_str("┤    ");
            } else {
                line.push_str("     ");
            }
        }
        println!("{} {}", line, v.name);
    }
    println!("    ════════════════════════════════════════════════════════════");
}
