use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_nested_borrows() {
    let x = 42;
    let r1 = &x;
    let r2 = &r1;
    assert_eq!(**r2, 42);
}

fn main() {
    test_nested_borrows();
}
