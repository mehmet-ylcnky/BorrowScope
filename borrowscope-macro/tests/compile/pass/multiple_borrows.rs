use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_multiple_borrows() {
    let x = 42;
    let r1 = &x;
    let r2 = &x;
    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);
}

fn main() {
    test_multiple_borrows();
}
