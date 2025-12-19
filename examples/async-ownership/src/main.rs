//! Async Ownership - Comprehensive BorrowScope Runtime Demo
//!
//! Demonstrates ALL tracking features across sync and async contexts.

use borrowscope_runtime::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    reset();
    println!("=== Async Ownership - Full Runtime Demo ===\n");

    // 1. Basic ownership
    demo_basic_ownership();

    // 2. Borrowing patterns
    demo_borrowing();

    // 3. Smart pointers (Rc, Arc)
    demo_smart_pointers();

    // 4. Interior mutability (RefCell, Cell)
    demo_interior_mutability();

    // 5. Static and const
    demo_static_const();

    // 6. Unsafe operations
    demo_unsafe();

    // 7. Async ownership patterns
    demo_async().await;

    // Print results
    print_results();
}

// ============================================================================
// 1. Basic Ownership: track_new, track_move, track_drop
// ============================================================================
fn demo_basic_ownership() {
    println!("--- 1. Basic Ownership ---");

    let x = track_new("x", String::from("hello"));
    println!("Created x: {}", x);

    let y = track_move("x", "y", x);
    println!("Moved to y: {}", y);

    track_drop("y");
    println!();
}

// ============================================================================
// 2. Borrowing: track_borrow, track_borrow_mut
// ============================================================================
fn demo_borrowing() {
    println!("--- 2. Borrowing ---");

    let mut data = track_new("data", vec![1, 2, 3]);

    // Multiple immutable borrows
    {
        let r1 = track_borrow("r1", &data);
        let r2 = track_borrow("r2", &data);
        println!("Immutable: {:?}, {:?}", r1, r2);
        track_drop("r2");
        track_drop("r1");
    }

    // Mutable borrow
    {
        let r_mut = track_borrow_mut("r_mut", &mut data);
        r_mut.push(4);
        println!("Mutable: {:?}", r_mut);
        track_drop("r_mut");
    }

    track_drop("data");
    println!();
}

// ============================================================================
// 3. Smart Pointers: track_rc_*, track_arc_*
// ============================================================================
fn demo_smart_pointers() {
    println!("--- 3. Smart Pointers ---");

    // Rc - single-threaded reference counting
    let rc1 = track_rc_new("rc1", Rc::new(42));
    let rc2 = track_rc_clone("rc2", "rc1", Rc::clone(&rc1));
    println!("Rc: rc1={}, rc2={}, count={}", *rc1, *rc2, Rc::strong_count(&rc1));
    track_drop("rc2");
    track_drop("rc1");

    // Arc - thread-safe reference counting
    let arc1 = track_arc_new("arc1", Arc::new("shared"));
    let arc2 = track_arc_clone("arc2", "arc1", Arc::clone(&arc1));
    println!("Arc: arc1={}, arc2={}, count={}", *arc1, *arc2, Arc::strong_count(&arc1));
    track_drop("arc2");
    track_drop("arc1");

    println!();
}

// ============================================================================
// 4. Interior Mutability: track_refcell_*, track_cell_*
// ============================================================================
fn demo_interior_mutability() {
    println!("--- 4. Interior Mutability ---");

    // RefCell - runtime borrow checking
    let refcell = track_refcell_new("refcell", RefCell::new(vec![1, 2]));
    {
        let borrowed = refcell_borrow!("ref_imm", "refcell", refcell.borrow());
        println!("RefCell borrowed: {:?}", *borrowed);
        refcell_drop!("ref_imm");
    }
    {
        let mut borrowed = refcell_borrow_mut!("ref_mut", "refcell", refcell.borrow_mut());
        borrowed.push(3);
        println!("RefCell mutated: {:?}", *borrowed);
        refcell_drop!("ref_mut");
    }
    track_drop("refcell");

    // Cell - copy-type interior mutability
    let cell = track_cell_new("cell", Cell::new(10));
    let val = track_cell_get("cell", "main.rs:cell_get", cell.get());
    println!("Cell get: {}", val);
    track_cell_set("cell", "main.rs:cell_set");
    cell.set(20);
    println!("Cell after set: {}", cell.get());
    track_drop("cell");

    println!();
}

// ============================================================================
// 5. Static and Const: track_static_*, track_const_eval
// ============================================================================
static mut COUNTER: i32 = 0;
const MAX_VALUE: i32 = 100;

fn demo_static_const() {
    println!("--- 5. Static and Const ---");

    // Const evaluation
    let max = track_const_eval("MAX_VALUE", 0, "i32", "main.rs:const", MAX_VALUE);
    println!("Const MAX_VALUE: {}", max);

    // Static initialization and access
    let _ = track_static_init("COUNTER", 1, "i32", false, 0i32);
    
    unsafe {
        track_static_access(1, "COUNTER", false, "main.rs:read"); // read
        println!("Static COUNTER (read): {}", COUNTER);
        
        track_static_access(1, "COUNTER", true, "main.rs:write"); // write
        COUNTER += 1;
        println!("Static COUNTER (write): {}", COUNTER);
    }

    println!();
}

// ============================================================================
// 6. Unsafe: raw pointers, unsafe blocks, FFI, transmute, unions
// ============================================================================
#[repr(C)]
union MyUnion {
    i: i32,
    f: f32,
}

fn demo_unsafe() {
    println!("--- 6. Unsafe Operations ---");

    let mut value = 42i32;

    // Raw pointers
    let ptr = track_raw_ptr("ptr", 0, "*const i32", "main.rs:ptr", &value as *const i32);
    println!("Raw ptr created: {:?}", ptr);

    let ptr_mut = track_raw_ptr_mut("ptr_mut", 1, "*mut i32", "main.rs:ptr_mut", &mut value as *mut i32);
    println!("Raw mut ptr created: {:?}", ptr_mut);

    // Unsafe block
    track_unsafe_block_enter(0, "main.rs:unsafe_start");
    unsafe {
        track_raw_ptr_deref(0, "main.rs:deref_read", false); // read
        println!("Deref ptr: {}", *ptr);

        track_raw_ptr_deref(1, "main.rs:deref_write", true); // write
        *ptr_mut = 100;
        println!("Deref ptr_mut (write): {}", *ptr_mut);

        // Unsafe function call
        track_unsafe_fn_call("dangerous_operation", "main.rs:unsafe_fn");
        dangerous_operation();

        // Transmute
        track_transmute("i32", "[u8; 4]", "main.rs:transmute");
        let bytes: [u8; 4] = std::mem::transmute::<i32, [u8; 4]>(0x12345678i32);
        println!("Transmuted bytes: {:?}", bytes);

        // Union field access
        let u = MyUnion { i: 42 };
        track_union_field_access("MyUnion", "i", "main.rs:union");
        println!("Union field i: {}", u.i);
    }
    track_unsafe_block_exit(0, "main.rs:unsafe_end");

    // FFI call simulation
    track_ffi_call("libc::getpid", "main.rs:ffi");
    println!("FFI call tracked");

    println!();
}

unsafe fn dangerous_operation() {
    // Simulated unsafe operation
}

// ============================================================================
// 7. Async Patterns: ownership across await points
// ============================================================================
async fn demo_async() {
    println!("--- 7. Async Ownership ---");

    // Arc for sharing across tasks
    let shared = track_arc_new("shared_data", Arc::new(vec![1, 2, 3]));

    let arc_clone = track_arc_clone("task_data", "shared_data", Arc::clone(&shared));

    // Simulate async work
    let handle = tokio::spawn(async move {
        let borrowed = track_borrow("async_ref", &*arc_clone);
        println!("Async task sees: {:?}", borrowed);
        track_drop("async_ref");
        track_drop("task_data");
    });

    handle.await.unwrap();

    // Ownership after await
    let local = track_new("post_await", String::from("after await"));
    {
        let r = track_borrow("await_borrow", &local);
        println!("Post-await borrow: {}", r);
        track_drop("await_borrow");
    }
    track_drop("post_await");
    track_drop("shared_data");

    println!();
}

// ============================================================================
// Results
// ============================================================================
fn print_results() {
    let events = get_events();
    println!("=== Captured Events ({}) ===", events.len());
    for (i, e) in events.iter().enumerate() {
        println!("{:3}. {:?}", i + 1, e);
    }

    let graph = get_graph();
    let stats = graph.stats();
    println!("\n=== Graph Stats ===");
    println!("Variables: {}", stats.total_variables);
    println!("Relationships: {}", stats.total_relationships);
    println!("Immutable borrows: {}", stats.immutable_borrows);
    println!("Mutable borrows: {}", stats.mutable_borrows);

    let path = std::env::temp_dir().join("async-ownership.json");
    export_json(&path).unwrap();
    println!("\nExported to: {}", path.display());
}
