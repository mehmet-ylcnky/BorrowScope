use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_tuple_pattern() {
    let (x, y) = (1, 2);
    assert_eq!(x, 1);
    assert_eq!(y, 2);
}

fn main() {
    test_tuple_pattern();
}
