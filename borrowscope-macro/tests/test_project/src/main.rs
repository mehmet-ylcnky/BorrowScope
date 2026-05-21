use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

mod copy_vs_move;
mod comprehensive_types;
mod all_test_patterns;

#[trace_borrow]
fn test_method_borrows() {
    // Test immutable borrow methods
    let vec_data = vec![1, 2, 3];
    let _len = vec_data.len();           // &self - immutable
    let _first = vec_data.first();       // &self - immutable
    let _is_empty = vec_data.is_empty(); // &self - immutable
    
    // Test mutable borrow methods
    let mut vec_mut = vec![1, 2, 3];
    vec_mut.push(4);                     // &mut self - mutable
    vec_mut.pop();                       // &mut self - mutable
    vec_mut.clear();                     // &mut self - mutable
    
    // Test consuming methods
    let vec_consume = vec![1, 2, 3];
    let _iter = vec_consume.into_iter(); // self - consuming
}

fn main() {
    reset();
    
    // Test method borrows
    test_method_borrows();
    
    // Test copy vs move
    copy_vs_move::test_copy_vs_move();
    
    // Test drop filtering
    copy_vs_move::test_drop_filtering();
    
    let events = get_events();
    println!("\nTotal events: {}", events.len());
    
    // Check for borrow tracking
    let borrow_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::Borrow { .. }))
        .collect();
    
    println!("Borrow events: {}", borrow_events.len());
    
    // Check for move tracking
    let move_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::Move { .. }))
        .collect();
    
    println!("Move events: {}", move_events.len());
    
    // Check for new events (includes copies)
    let new_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::New { .. } | Event::RcNew { .. } | Event::ArcNew { .. }))
        .collect();
    
    println!("New events: {}", new_events.len());
    
    // Check for drop events
    let drop_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::Drop { .. }))
        .collect();
    
    println!("Drop events: {}", drop_events.len());
    println!("\nDrop event details:");
    for event in &drop_events {
        println!("  {:?}", event);
    }
}
