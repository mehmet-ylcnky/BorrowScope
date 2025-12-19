//! Borrow Conflicts - Comprehensive conflict detection with BorrowScope
//!
//! Demonstrates valid patterns, compile-time conflicts, runtime conflicts,
//! and complex scenarios like nested borrows and struct field borrowing.

use borrowscope_graph::{OwnershipGraph, Variable};
use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("=== Borrow Conflicts - Comprehensive Demo ===\n");

    demo_valid_patterns();
    demo_compile_time_conflicts();
    demo_nested_borrows();
    demo_struct_field_borrowing();
    demo_refcell_scenarios();
    demo_rc_refcell_shared_mutation();
    demo_complex_lifetimes();

    println!("=== Demo Complete ===");
}

// ============================================================================
// 1. Valid Borrow Patterns
// ============================================================================
fn demo_valid_patterns() {
    println!("━━━ 1. Valid Borrow Patterns ━━━\n");
    reset();

    let mut data = track_new("data", vec![1, 2, 3, 4, 5]);

    // A: Multiple immutable borrows
    println!("A) Multiple immutable borrows (OK)");
    {
        let r1 = track_borrow("r1", &data);
        let r2 = track_borrow("r2", &data);
        let r3 = track_borrow("r3", &data);
        println!("   Sum via r1: {}", r1.iter().sum::<i32>());
        println!("   Len via r2: {}", r2.len());
        println!("   First via r3: {}", r3[0]);
        track_drop("r3");
        track_drop("r2");
        track_drop("r1");
    }
    println!("   ✓ Multiple readers allowed simultaneously\n");

    // B: Non-lexical lifetimes (NLL)
    println!("B) Non-lexical lifetimes - borrow ends at last use (OK)");
    {
        let r = track_borrow("r", &data);
        let first = r[0]; // Last use of r
        track_drop("r");
        
        let m = track_borrow_mut("m", &mut data);
        m.push(first + 10);
        track_drop("m");
    }
    println!("   ✓ Immutable borrow ended before mutable began\n");

    // C: Reborrowing
    println!("C) Reborrowing - borrow from a borrow (OK)");
    {
        let m = track_borrow_mut("m", &mut data);
        {
            let reborrow = track_borrow("reborrow", &*m);
            println!("   Reborrowed view: {:?}", reborrow);
            track_drop("reborrow");
        }
        m.push(100);
        track_drop("m");
    }
    println!("   ✓ Reborrow ended before original mutable borrow used again\n");

    // D: Disjoint borrows (different indices)
    println!("D) Split borrows via split_at_mut (OK)");
    {
        let (left, right) = data.split_at_mut(3);
        let l = track_borrow_mut("left", left);
        let r = track_borrow_mut("right", right);
        l[0] = 999;
        r[0] = 888;
        println!("   Left: {:?}, Right: {:?}", l, r);
        track_drop("right");
        track_drop("left");
    }
    println!("   ✓ Disjoint mutable borrows allowed\n");

    track_drop("data");
    println!("   Events captured: {}\n", get_events().len());
}

// ============================================================================
// 2. Compile-Time Conflicts (Simulated)
// ============================================================================
fn demo_compile_time_conflicts() {
    println!("━━━ 2. Compile-Time Conflicts (Simulated) ━━━\n");

    // Conflict A: Use after move
    println!("A) Use after move");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "s1", "String", 0, Some(10)));
    graph.add_variable(var(2, "s2", "String", 10, Some(20)));
    graph.add_move(1, 2, 10);
    println!("   s1 created at t=0, moved to s2 at t=10");
    println!("   // let s1 = String::from(\"hello\");");
    println!("   // let s2 = s1;  // s1 moved");
    println!("   // println!(\"{{}}\", s1);  // ERROR: use after move");
    println!("   ✗ Compiler prevents use of moved value\n");

    // Conflict B: Mutable + Immutable overlap
    println!("B) Mutable borrow while immutable exists");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "data", "Vec<i32>", 0, Some(100)));
    graph.add_variable(var(2, "r", "&Vec<i32>", 10, Some(50)));
    graph.add_variable(var(3, "m", "&mut Vec<i32>", 30, Some(60)));
    graph.add_borrow(2, 1, false, 10);
    graph.add_borrow(3, 1, true, 30);
    
    print_conflicts(&graph, "data");
    println!("   // let r = &data;        // t=10");
    println!("   // let m = &mut data;    // t=30 ERROR!");
    println!("   // println!(\"{{}}\", r);   // r still in use");
    println!();

    // Conflict C: Multiple mutable borrows
    println!("C) Two simultaneous mutable borrows");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "data", "Vec<i32>", 0, Some(100)));
    graph.add_variable(var(2, "m1", "&mut Vec<i32>", 10, Some(50)));
    graph.add_variable(var(3, "m2", "&mut Vec<i32>", 20, Some(40)));
    graph.add_borrow(2, 1, true, 10);
    graph.add_borrow(3, 1, true, 20);
    
    print_conflicts(&graph, "data");
    println!("   // let m1 = &mut data;   // t=10");
    println!("   // let m2 = &mut data;   // t=20 ERROR!");
    println!();

    // Conflict D: Borrow outlives owner
    println!("D) Reference outlives owner");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "data", "Vec<i32>", 0, Some(30)));
    graph.add_variable(var(2, "r", "&Vec<i32>", 10, Some(50))); // r lives longer!
    graph.add_borrow(2, 1, false, 10);
    
    let errors = graph.validate();
    if let Err(errs) = errors {
        println!("   ✗ Validation errors:");
        for e in errs {
            println!("     - {}", e);
        }
    }
    println!("   // {{ let data = vec![1,2,3]; r = &data; }}");
    println!("   // println!(\"{{}}\", r);  // ERROR: data dropped");
    println!();
}

// ============================================================================
// 3. Nested Borrows
// ============================================================================
fn demo_nested_borrows() {
    println!("━━━ 3. Nested Borrow Scenarios ━━━\n");
    reset();

    let mut outer = track_new("outer", vec![vec![1, 2], vec![3, 4]]);

    // Valid: Borrow chain
    println!("A) Valid borrow chain");
    {
        let outer_ref = track_borrow("outer_ref", &outer);
        let inner_ref = track_borrow("inner_ref", &outer_ref[0]);
        println!("   Inner: {:?}", inner_ref);
        track_drop("inner_ref");
        track_drop("outer_ref");
    }
    println!("   ✓ Nested immutable borrows OK\n");

    // Valid: Mutable through chain
    println!("B) Mutable access through chain");
    {
        let outer_mut = track_borrow_mut("outer_mut", &mut outer);
        let inner_mut = track_borrow_mut("inner_mut", &mut outer_mut[0]);
        inner_mut.push(99);
        println!("   Modified inner: {:?}", inner_mut);
        track_drop("inner_mut");
        track_drop("outer_mut");
    }
    println!("   ✓ Nested mutable borrows OK (one at a time)\n");

    // Simulated conflict: Borrow outer while inner mut exists
    println!("C) Conflict: Access outer while inner mutably borrowed");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "outer", "Vec<Vec<i32>>", 0, Some(100)));
    graph.add_variable(var(2, "outer_mut", "&mut Vec<Vec<i32>>", 10, Some(80)));
    graph.add_variable(var(3, "inner_mut", "&mut Vec<i32>", 20, Some(60)));
    graph.add_variable(var(4, "outer_ref", "&Vec<Vec<i32>>", 30, Some(50))); // Conflict!
    graph.add_borrow(2, 1, true, 10);
    graph.add_borrow(3, 2, true, 20);
    graph.add_borrow(4, 1, false, 30); // Can't borrow outer while inner_mut exists
    
    print_conflicts(&graph, "outer");
    println!();

    track_drop("outer");
}

// ============================================================================
// 4. Struct Field Borrowing
// ============================================================================
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    scores: Vec<i32>,
}

fn demo_struct_field_borrowing() {
    println!("━━━ 4. Struct Field Borrowing ━━━\n");
    reset();

    let mut person = track_new("person", Person {
        name: String::from("Alice"),
        age: 30,
        scores: vec![85, 90, 78],
    });

    // Valid: Borrow different fields
    println!("A) Borrow different fields simultaneously (OK)");
    {
        let name = track_borrow("name", &person.name);
        let age = track_borrow("age", &person.age);
        println!("   {} is {} years old", name, age);
        track_drop("age");
        track_drop("name");
    }
    println!("   ✓ Different fields can be borrowed independently\n");

    // Valid: Mutable borrow one field, immutable another
    println!("B) Mutable borrow one field, read another (OK)");
    {
        let scores = track_borrow_mut("scores", &mut person.scores);
        // Note: In real Rust, this works because fields are disjoint
        // let name = &person.name;  // Would work!
        scores.push(95);
        println!("   Scores updated: {:?}", scores);
        track_drop("scores");
    }
    println!("   ✓ Disjoint field borrows allowed\n");

    // Simulated conflict: Borrow whole struct while field borrowed
    println!("C) Conflict: Borrow whole struct while field mutably borrowed");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "person", "Person", 0, Some(100)));
    graph.add_variable(var(2, "scores", "&mut Vec<i32>", 10, Some(50)));
    graph.add_variable(var(3, "person_ref", "&Person", 20, Some(40))); // Conflict!
    graph.add_borrow(2, 1, true, 10);
    graph.add_borrow(3, 1, false, 20);
    
    print_conflicts(&graph, "person");
    println!();

    track_drop("person");
}

// ============================================================================
// 5. RefCell Scenarios
// ============================================================================
fn demo_refcell_scenarios() {
    println!("━━━ 5. RefCell Runtime Borrow Checking ━━━\n");
    reset();

    let cell = track_refcell_new("cell", RefCell::new(vec![1, 2, 3]));

    // Valid: Sequential access
    println!("A) Sequential borrow then borrow_mut (OK)");
    {
        let r = refcell_borrow!("r", "cell", cell.borrow());
        println!("   Read: {:?}", *r);
        refcell_drop!("r");
    }
    {
        let mut m = refcell_borrow_mut!("m", "cell", cell.borrow_mut());
        m.push(4);
        println!("   After mutation: {:?}", *m);
        refcell_drop!("m");
    }
    println!("   ✓ No panic\n");

    // Valid: Multiple immutable
    println!("B) Multiple simultaneous borrow() (OK)");
    {
        let r1 = refcell_borrow!("r1", "cell", cell.borrow());
        let r2 = refcell_borrow!("r2", "cell", cell.borrow());
        let r3 = refcell_borrow!("r3", "cell", cell.borrow());
        println!("   r1={:?}, r2={:?}, r3={:?}", *r1, *r2, *r3);
        refcell_drop!("r3");
        refcell_drop!("r2");
        refcell_drop!("r1");
    }
    println!("   ✓ Multiple readers OK\n");

    // try_borrow patterns
    println!("C) Using try_borrow to avoid panics");
    {
        let m = refcell_borrow_mut!("m", "cell", cell.borrow_mut());
        
        // try_borrow returns None instead of panicking
        match cell.try_borrow() {
            Ok(_) => println!("   Got borrow (unexpected)"),
            Err(_) => println!("   try_borrow() returned Err - already mutably borrowed"),
        }
        
        match cell.try_borrow_mut() {
            Ok(_) => println!("   Got borrow_mut (unexpected)"),
            Err(_) => println!("   try_borrow_mut() returned Err - already borrowed"),
        }
        
        refcell_drop!("m");
    }
    println!("   ✓ try_* methods allow graceful handling\n");

    track_drop("cell");
}

// ============================================================================
// 6. Rc<RefCell<T>> Shared Mutation
// ============================================================================
fn demo_rc_refcell_shared_mutation() {
    println!("━━━ 6. Rc<RefCell<T>> Shared Mutation ━━━\n");
    reset();

    let shared = track_rc_new("shared", Rc::new(RefCell::new(vec![1, 2, 3])));
    let clone1 = track_rc_clone("clone1", "shared", Rc::clone(&shared));
    let clone2 = track_rc_clone("clone2", "shared", Rc::clone(&shared));

    println!("Three Rc handles to same RefCell<Vec>");
    println!("   Rc count: {}\n", Rc::strong_count(&shared));

    // Mutate through different handles
    println!("A) Mutate through clone1");
    {
        let mut m = refcell_borrow_mut!("m1", "clone1", clone1.borrow_mut());
        m.push(4);
        println!("   After push via clone1: {:?}", *m);
        refcell_drop!("m1");
    }

    println!("B) Mutate through clone2");
    {
        let mut m = refcell_borrow_mut!("m2", "clone2", clone2.borrow_mut());
        m.push(5);
        println!("   After push via clone2: {:?}", *m);
        refcell_drop!("m2");
    }

    println!("C) Read through original");
    {
        let r = refcell_borrow!("r", "shared", shared.borrow());
        println!("   Final state: {:?}", *r);
        refcell_drop!("r");
    }
    println!("   ✓ All handles see same data\n");

    // Conflict scenario
    println!("D) Conflict: borrow_mut through one while borrowed through another");
    println!("   // let r = clone1.borrow();");
    println!("   // let m = clone2.borrow_mut();  // PANIC!");
    println!("   ✗ RefCell tracks borrows across all Rc handles\n");

    track_drop("clone2");
    track_drop("clone1");
    track_drop("shared");
}

// ============================================================================
// 7. Complex Lifetime Scenarios
// ============================================================================
fn demo_complex_lifetimes() {
    println!("━━━ 7. Complex Lifetime Scenarios ━━━\n");

    // Scenario A: Interleaved borrows
    println!("A) Interleaved borrow lifetimes");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "a", "i32", 0, Some(100)));
    graph.add_variable(var(2, "b", "i32", 0, Some(100)));
    graph.add_variable(var(3, "r_a", "&i32", 10, Some(60)));
    graph.add_variable(var(4, "r_b", "&i32", 20, Some(70)));
    graph.add_variable(var(5, "m_a", "&mut i32", 40, Some(80))); // Conflict with r_a!
    graph.add_borrow(3, 1, false, 10);
    graph.add_borrow(4, 2, false, 20);
    graph.add_borrow(5, 1, true, 40);
    
    println!("   Timeline:");
    println!("   t=10: r_a borrows a (immut)");
    println!("   t=20: r_b borrows b (immut)");
    println!("   t=40: m_a borrows a (mut) - CONFLICT with r_a!");
    println!("   t=60: r_a dropped");
    println!("   t=70: r_b dropped");
    println!("   t=80: m_a dropped");
    print_conflicts(&graph, "a");
    println!();

    // Scenario B: Borrow chain depth
    println!("B) Deep borrow chain");
    let mut graph = OwnershipGraph::new();
    graph.add_variable(var(1, "root", "Data", 0, Some(100)));
    graph.add_variable(var(2, "level1", "&Data", 10, Some(90)));
    graph.add_variable(var(3, "level2", "&&Data", 20, Some(80)));
    graph.add_variable(var(4, "level3", "&&&Data", 30, Some(70)));
    graph.add_borrow(2, 1, false, 10);
    graph.add_borrow(3, 2, false, 20);
    graph.add_borrow(4, 3, false, 30);
    
    println!("   Borrow depth: {}", graph.borrow_depth(4));
    println!("   Chain: root -> level1 -> level2 -> level3");
    println!("   ✓ Deep immutable chains are valid\n");

    // Scenario C: Graph connectivity
    println!("C) Connected components in borrow graph");
    let mut graph = OwnershipGraph::new();
    // Component 1
    graph.add_variable(var(1, "a", "i32", 0, Some(100)));
    graph.add_variable(var(2, "r_a", "&i32", 10, Some(50)));
    graph.add_borrow(2, 1, false, 10);
    // Component 2 (separate)
    graph.add_variable(var(3, "b", "i32", 0, Some(100)));
    graph.add_variable(var(4, "r_b", "&i32", 10, Some(50)));
    graph.add_borrow(4, 3, false, 10);
    
    let components = graph.connected_components();
    println!("   Found {} connected components", components.len());
    for (i, comp) in components.iter().enumerate() {
        let names: Vec<_> = comp.iter()
            .filter_map(|id| graph.get_variable(*id))
            .map(|v| v.name.as_str())
            .collect();
        println!("   Component {}: {:?}", i + 1, names);
    }
    println!();

    // Print final event count
    let events = get_events();
    println!("━━━ Summary ━━━");
    println!("Total events captured: {}", events.len());
    
    let path = std::env::temp_dir().join("borrow-conflicts.json");
    export_json(&path).unwrap();
    println!("Exported to: {}\n", path.display());
}

// ============================================================================
// Helper Functions
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

fn print_conflicts(graph: &OwnershipGraph, owner: &str) {
    let conflicts = graph.find_conflicts_optimized();
    if conflicts.is_empty() {
        println!("   ✓ No conflicts detected");
    } else {
        for c in &conflicts {
            println!("   ✗ CONFLICT: {}", c.format(graph));
            println!("     Time range: {} - {}", c.time_range.0, c.time_range.1);
        }
    }
    
    // Show timeline
    if let Some(owner_var) = graph.all_variables().find(|v| v.name == owner) {
        let timeline = graph.conflict_timeline(owner_var.id);
        if !timeline.is_empty() {
            println!("   Timeline for '{}':", owner);
            for (time, borrows) in timeline {
                let strs: Vec<_> = borrows.iter()
                    .filter_map(|(id, is_mut)| {
                        graph.get_variable(*id).map(|v| {
                            if *is_mut { format!("{} (mut)", v.name) } 
                            else { format!("{} (immut)", v.name) }
                        })
                    })
                    .collect();
                println!("     t={}: [{}]", time, strs.join(", "));
            }
        }
    }
}
