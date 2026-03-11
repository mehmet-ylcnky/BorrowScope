use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn test_semantic_vs_heuristic() {
    // "retain" is in the heuristic mutable list - both agree
    let mut v = vec![1, 2, 3, 4, 5];
    v.retain(|x| *x > 2);
    
    // "windows" is NOT in any heuristic list, defaults to immutable
    // Semantic: self_borrow="immutable" (correct)
    // Heuristic: defaults to immutable (happens to be correct)
    let _w = v.windows(2);
    
    // "split_off" is NOT in any heuristic list, defaults to immutable
    // Semantic: self_borrow="mutable" (correct - takes &mut self)
    // Heuristic: defaults to immutable (WRONG!)
    let mut v2 = vec![1, 2, 3, 4, 5];
    let _tail = v2.split_off(2);
}

fn main() {
    reset();
    test_semantic_vs_heuristic();
    
    let events = get_events();
    println!("Events:");
    for event in &events {
        match event {
            Event::Borrow { borrower_name, mutable, .. } => {
                println!("  Borrow: {} mutable={}", borrower_name, mutable);
            }
            Event::New { var_name, .. } => {
                println!("  New: {}", var_name);
            }
            Event::Drop { var_id, .. } => {
                println!("  Drop: {}", var_id);
            }
            _ => {}
        }
    }
    
    // Count mutable borrows - with semantic we should get 2 (retain + split_off)
    // With heuristic only we'd get 1 (retain only, split_off defaults to immutable)
    let mut_borrows = events.iter().filter(|e| matches!(e, Event::Borrow { mutable: true, .. })).count();
    let immut_borrows = events.iter().filter(|e| matches!(e, Event::Borrow { mutable: false, .. })).count();
    
    println!("\nMutable borrows: {}", mut_borrows);
    println!("Immutable borrows: {}", immut_borrows);
    
    // If semantic is working: 2 mutable (retain, split_off), 1 immutable (windows)
    // If heuristic only: 1 mutable (retain), 2 immutable (windows, split_off)
    if mut_borrows == 2 {
        println!("\n✅ SEMANTIC PATH IS ACTIVE - split_off correctly detected as mutable");
    } else {
        println!("\n❌ HEURISTIC FALLBACK - split_off incorrectly detected as immutable");
    }
}

// Test new type coverage: Ordering, PanicInfo, never type
fn type_coverage_extras() {
    use std::cmp::Ordering;
    let ord: Ordering = 1i32.cmp(&2);
    let _is_less = ord.is_lt();

    // fmt::Arguments is created by format_args! macro — can't easily bind to a variable

    // Never type: functions that return ! produce diverging control flow
    // We can't bind a variable of type ! but we can detect it in return types
}

fn field_access_test() {
    struct Point { x: i32, y: i32 }
    let mut p = Point { x: 10, y: 20 };
    let x_val = p.x;        // read
    p.y = 30;               // write
    let x_ref = &p.x;       // borrow_shared
    let y_ref = &mut p.y;   // borrow_mut
}

fn closure_capture_test() {
    let x = 42;
    let y = String::from("hello");
    
    // Closure that captures by reference
    let f1 = || println!("{} {}", x, y);
    f1();
    
    // Closure that captures by mutable reference
    let mut z = vec![1, 2, 3];
    let f2 = || z.push(4);
    f2();
    
    // Closure that captures by move
    let f3 = move || drop(y);
    f3();
}
