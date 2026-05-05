use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn test_semantic() {
    let rc_data = std::rc::Rc::new(42);
    let arc_data = std::sync::Arc::new(100);
    let vec_data = vec![1, 2, 3];
}

#[test]
fn test_macro_uses_semantic_data() {
    reset();
    test_semantic();
    let events = get_events();

    // Should have tracked the 3 variables
    assert!(
        events.len() >= 3,
        "Expected at least 3 events, got {}",
        events.len()
    );

    // Print events for inspection
    for event in &events {
        println!("{:?}", event);
    }
}
