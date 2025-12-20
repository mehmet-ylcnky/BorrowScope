//! Borrow Conflicts - Comprehensive conflict detection with BorrowScope
//!
//! Demonstrates valid patterns, compile-time conflicts, runtime conflicts,
//! and complex scenarios like nested borrows and struct field borrowing.
//! Now with RAII guards and filtering API!

use borrowscope_runtime::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("=== Borrow Conflicts - Comprehensive Demo ===\n");
    reset(); // Single reset at start

    demo_valid_patterns();
    demo_raii_guards_for_borrows();
    demo_nested_borrows();
    demo_struct_field_borrowing();
    demo_refcell_scenarios();
    demo_rc_refcell_shared_mutation();

    // Use pretty print summary
    println!("\n");
    print_summary();

    // Show borrow-specific stats using filtering API
    let borrows = get_borrow_events();
    let mutable_borrows = get_events_filtered(|e| {
        matches!(e, Event::Borrow { mutable: true, .. })
    });
    println!("\nBorrow Analysis:");
    println!("  Total borrows: {}", borrows.len());
    println!("  Mutable borrows: {}", mutable_borrows.len());
    println!("  Immutable borrows: {}", borrows.len() - mutable_borrows.len());

    println!("\n=== Demo Complete ===");
}

// ============================================================================
// RAII Guards for Borrow Tracking
// ============================================================================
fn demo_raii_guards_for_borrows() {
    println!("━━━ RAII Guards for Automatic Borrow Tracking ━━━\n");

    let data = track_new_guard("data", vec![1, 2, 3, 4, 5]);

    // BorrowGuard automatically tracks when borrow ends
    {
        let r1 = track_borrow_guard("r1", &*data);
        let r2 = track_borrow_guard("r2", &*data);
        println!("   Borrowed via guards: {:?}, {:?}", *r1, *r2);
        // track_drop called automatically for r1 and r2
    }

    println!("   ✓ Guards automatically track borrow lifetimes\n");
}

// ============================================================================
// 1. Valid Borrow Patterns
// ============================================================================
fn demo_valid_patterns() {
    println!("━━━ 1. Valid Borrow Patterns ━━━\n");

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
        l[0] = 10;
        r[0] = 40;
        println!("   Left: {:?}, Right: {:?}", l, r);
        track_drop("right");
        track_drop("left");
    }
    println!("   ✓ Disjoint mutable borrows allowed\n");

    track_drop("data");
    println!("   Events captured: {}\n", get_events().len());
}

// ============================================================================
// 2. Nested Borrows
// ============================================================================
fn demo_nested_borrows() {
    println!("━━━ 2. Nested Borrow Scenarios ━━━\n");

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

    track_drop("outer");
}

// ============================================================================
// 3. Struct Field Borrowing
// ============================================================================
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    scores: Vec<i32>,
}

fn demo_struct_field_borrowing() {
    println!("━━━ 3. Struct Field Borrowing ━━━\n");

    let mut person = track_new(
        "person",
        Person {
            name: String::from("Alice"),
            age: 30,
            scores: vec![85, 90, 78],
        },
    );

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
        scores.push(95);
        println!("   Scores updated: {:?}", scores);
        track_drop("scores");
    }
    println!("   ✓ Disjoint field borrows allowed\n");

    track_drop("person");
}

// ============================================================================
// 4. RefCell Scenarios
// ============================================================================
fn demo_refcell_scenarios() {
    println!("━━━ 4. RefCell Runtime Borrow Checking ━━━\n");

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
        let _m = refcell_borrow_mut!("m", "cell", cell.borrow_mut());

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
// 5. Rc<RefCell<T>> Shared Mutation
// ============================================================================
fn demo_rc_refcell_shared_mutation() {
    println!("━━━ 5. Rc<RefCell<T>> Shared Mutation ━━━\n");

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

    // Export events
    let path = std::env::temp_dir().join("borrow-conflicts.json");
    export_json(&path).unwrap();
    println!("Exported to: {}\n", path.display());
}
