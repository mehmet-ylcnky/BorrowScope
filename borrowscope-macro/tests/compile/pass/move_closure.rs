use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_move_closure() {
    let s = String::from("hello");
    let closure = move || s.len();
    let result = closure();
    assert_eq!(result, 5);
}

fn main() {
    test_move_closure();
}
