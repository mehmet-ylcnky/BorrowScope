use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_immutable_borrow() {
    let x = 42;
    let r = &x;
    assert_eq!(*r, 42);
}

fn main() {
    test_immutable_borrow();
}
