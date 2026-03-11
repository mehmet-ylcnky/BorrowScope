use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn test_semantic() {
    let rc_data = std::rc::Rc::new(42);
    let arc_data = std::sync::Arc::new(100);
    let vec_data = vec![1, 2, 3];
}

fn main() {
    reset();
    test_semantic();
    let events = get_events();
    
    println!("Total events: {}", events.len());
    for event in &events {
        println!("{:?}", event);
    }
}
