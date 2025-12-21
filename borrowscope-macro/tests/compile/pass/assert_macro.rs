use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_assert_macro() {
    let x = 42;
    let y = 42;
    assert_eq!(x, y);
    assert!(x > 0);
}

fn main() {
    test_assert_macro();
}
