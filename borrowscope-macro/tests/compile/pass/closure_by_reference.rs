use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_closure_by_reference() {
    let x = 42;
    let closure = |y| x + y;
    let result = closure(10);
    assert_eq!(result, 52);
}

fn main() {
    test_closure_by_reference();
}
