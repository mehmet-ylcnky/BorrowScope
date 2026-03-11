use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

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
    test_method_borrows();
    
    let events = get_events();
    println!("Total events: {}", events.len());
    
    // Check for borrow tracking
    let borrow_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::Borrow { .. }))
        .collect();
    
    println!("Borrow events: {}", borrow_events.len());
    
    for event in borrow_events {
        println!("{:?}", event);
    }
}
