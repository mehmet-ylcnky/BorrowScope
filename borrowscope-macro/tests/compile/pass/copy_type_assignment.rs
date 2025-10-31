use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_copy_type() {
    let x = 42;
    let y = x;
    // Both x and y are valid (Copy type)
    assert_eq!(x, 42);
    assert_eq!(y, 42);
}

fn main() {
    test_copy_type();
}
