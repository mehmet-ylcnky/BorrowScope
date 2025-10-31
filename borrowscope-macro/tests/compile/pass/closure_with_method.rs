use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_closure_with_method() {
    let s = String::from("hello");
    let closure = || s.len();
    let result = closure();
    assert_eq!(result, 5);
}

fn main() {
    test_closure_with_method();
}
