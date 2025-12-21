use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_move_string() {
    let original = String::from("test");
    let moved = original;
    assert_eq!(moved.len(), 4);
}

fn main() {
    test_move_string();
}
