//! Smart Pointers - Deep dive into Box, Rc, Arc, RefCell, and Weak
//!
//! Demonstrates complex ownership patterns with reference counting.

use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::thread;

fn main() {
    reset();
    println!("=== Smart Pointers Demo ===\n");

    demo_box();
    demo_rc_basic();
    demo_rc_tree();
    demo_rc_refcell();
    demo_weak_references();
    demo_arc_threads();

    print_results();
}

// ============================================================================
// Box<T> - Heap allocation with single ownership
// ============================================================================
fn demo_box() {
    println!("--- 1. Box<T> - Heap Allocation ---");

    // Box moves data to heap
    let boxed = track_new("boxed", Box::new([0u8; 1000]));
    println!("Boxed array on heap, len: {}", boxed.len());

    // Borrow from box
    let slice = track_borrow("slice", &boxed[..10]);
    println!("First 10 bytes: {:?}", slice);
    track_drop("slice");

    // Move box
    let moved = track_move("boxed", "moved", boxed);
    println!("Box moved, len still: {}", moved.len());

    track_drop("moved");
    println!();
}

// ============================================================================
// Rc<T> - Single-threaded reference counting
// ============================================================================
fn demo_rc_basic() {
    println!("--- 2. Rc<T> - Reference Counting ---");

    let rc1 = track_rc_new("rc1", Rc::new(String::from("shared data")));
    println!("rc1 created, count: {}", Rc::strong_count(&rc1));

    let rc2 = track_rc_clone("rc2", "rc1", Rc::clone(&rc1));
    println!("rc2 cloned, count: {}", Rc::strong_count(&rc1));

    let rc3 = track_rc_clone("rc3", "rc1", Rc::clone(&rc1));
    println!("rc3 cloned, count: {}", Rc::strong_count(&rc1));

    // Drop one reference
    track_drop("rc3");
    drop(rc3);
    println!("rc3 dropped, count: {}", Rc::strong_count(&rc1));

    // Borrow the shared data
    {
        let borrowed = track_borrow("borrowed", &*rc1);
        println!("Borrowed: {}", borrowed);
        track_drop("borrowed");
    }

    track_drop("rc2");
    track_drop("rc1");
    println!();
}

// ============================================================================
// Rc<T> Tree - Shared ownership in data structures
// ============================================================================
#[derive(Debug)]
struct TreeNode {
    value: i32,
    children: Vec<Rc<TreeNode>>,
}

fn demo_rc_tree() {
    println!("--- 3. Rc<T> Tree - Shared Nodes ---");

    // Create leaf nodes
    let leaf1 = track_rc_new("leaf1", Rc::new(TreeNode { value: 1, children: vec![] }));
    let leaf2 = track_rc_new("leaf2", Rc::new(TreeNode { value: 2, children: vec![] }));

    // Shared leaf - referenced by multiple parents
    let shared_leaf = track_rc_new("shared", Rc::new(TreeNode { value: 99, children: vec![] }));
    println!("Shared leaf count: {}", Rc::strong_count(&shared_leaf));

    // Parent 1 references leaf1 and shared
    let parent1 = track_rc_new("parent1", Rc::new(TreeNode {
        value: 10,
        children: vec![
            track_rc_clone("p1_leaf1", "leaf1", Rc::clone(&leaf1)),
            track_rc_clone("p1_shared", "shared", Rc::clone(&shared_leaf)),
        ],
    }));

    // Parent 2 references leaf2 and shared
    let parent2 = track_rc_new("parent2", Rc::new(TreeNode {
        value: 20,
        children: vec![
            track_rc_clone("p2_leaf2", "leaf2", Rc::clone(&leaf2)),
            track_rc_clone("p2_shared", "shared", Rc::clone(&shared_leaf)),
        ],
    }));

    println!("Shared leaf count after parents: {}", Rc::strong_count(&shared_leaf));
    println!("Parent1: {:?}", parent1.value);
    println!("Parent2: {:?}", parent2.value);

    // Drop parents - shared leaf survives
    track_drop("parent2");
    drop(parent2);
    println!("After parent2 drop, shared count: {}", Rc::strong_count(&shared_leaf));

    track_drop("parent1");
    drop(parent1);
    println!("After parent1 drop, shared count: {}", Rc::strong_count(&shared_leaf));

    track_drop("shared");
    track_drop("leaf2");
    track_drop("leaf1");
    println!();
}

// ============================================================================
// Rc<RefCell<T>> - Shared mutable state
// ============================================================================
fn demo_rc_refcell() {
    println!("--- 4. Rc<RefCell<T>> - Shared Mutable State ---");

    let shared_state = track_rc_new("state", Rc::new(RefCell::new(vec![1, 2, 3])));
    println!("Initial: {:?}", shared_state.borrow());

    // Clone for "another owner"
    let state_clone = track_rc_clone("state_clone", "state", Rc::clone(&shared_state));

    // Mutate through first reference
    {
        let mut borrowed = refcell_borrow_mut!("mut1", "state", shared_state.borrow_mut());
        borrowed.push(4);
        println!("After mut1 push: {:?}", *borrowed);
        refcell_drop!("mut1");
    }

    // Mutate through cloned reference
    {
        let mut borrowed = refcell_borrow_mut!("mut2", "state_clone", state_clone.borrow_mut());
        borrowed.push(5);
        println!("After mut2 push: {:?}", *borrowed);
        refcell_drop!("mut2");
    }

    // Read through original
    {
        let borrowed = refcell_borrow!("read", "state", shared_state.borrow());
        println!("Final state: {:?}", *borrowed);
        refcell_drop!("read");
    }

    track_drop("state_clone");
    track_drop("state");
    println!();
}

// ============================================================================
// Weak<T> - Non-owning references to prevent cycles
// ============================================================================
#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn demo_weak_references() {
    println!("--- 5. Weak<T> - Breaking Reference Cycles ---");

    let parent = track_rc_new("parent", Rc::new(Node {
        value: 100,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    }));

    let child = track_rc_new("child", Rc::new(Node {
        value: 200,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    }));

    // Set up parent-child relationship
    // Child has Weak reference to parent (no ownership)
    *child.parent.borrow_mut() = Rc::downgrade(&parent);
    
    // Parent has strong reference to child
    parent.children.borrow_mut().push(track_rc_clone("child_ref", "child", Rc::clone(&child)));

    println!("Parent strong: {}, weak: {}", 
             Rc::strong_count(&parent), Rc::weak_count(&parent));
    println!("Child strong: {}, weak: {}", 
             Rc::strong_count(&child), Rc::weak_count(&child));

    // Access parent from child via Weak
    if let Some(p) = child.parent.borrow().upgrade() {
        println!("Child's parent value: {}", p.value);
    }

    // Drop child's direct reference
    track_drop("child");
    drop(child);
    println!("After child drop, parent still has child in children");

    // Parent drop will clean up everything
    track_drop("parent");
    println!();
}

// ============================================================================
// Arc<T> - Thread-safe reference counting
// ============================================================================
fn demo_arc_threads() {
    println!("--- 6. Arc<T> - Thread-Safe Sharing ---");

    let data = track_arc_new("data", Arc::new(vec![1, 2, 3, 4, 5]));
    println!("Main thread, count: {}", Arc::strong_count(&data));

    let mut handles = vec![];

    for i in 0..3 {
        let name = format!("thread_{}", i);
        let arc_clone = track_arc_clone(&name, "data", Arc::clone(&data));
        println!("Cloned for {}, count: {}", name, Arc::strong_count(&data));

        let handle = thread::spawn(move || {
            let sum: i32 = arc_clone.iter().sum();
            println!("Thread {} sum: {}", i, sum);
            track_drop(&format!("thread_{}", i));
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads done, count: {}", Arc::strong_count(&data));
    track_drop("data");
    println!();
}

// ============================================================================
// Results
// ============================================================================
fn print_results() {
    println!("=== Results ===\n");

    // Use pretty print summary
    print_summary();

    // Use filtering API
    println!("\n--- Filtering API Demo ---");
    let rc_events = get_events_filtered(|e| e.is_rc());
    let arc_events = get_events_filtered(|e| e.is_arc());
    let refcell_events = get_events_filtered(|e| e.is_refcell());

    println!("Rc events: {}", rc_events.len());
    println!("Arc events: {}", arc_events.len());
    println!("RefCell events: {}", refcell_events.len());

    // Get summary struct for programmatic access
    let summary = get_summary();
    println!(
        "\nSummary struct: {} vars created, {} Rc ops, {} Arc ops",
        summary.variables_created, summary.rc_operations, summary.arc_operations
    );

    let path = std::env::temp_dir().join("smart-pointers.json");
    export_json(&path).unwrap();
    println!("\nExported to: {}", path.display());
}
