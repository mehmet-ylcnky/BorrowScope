use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_nested_tuple() {
    let (a, (b, c)) = (1, (2, 3));
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
}

fn main() {
    test_nested_tuple();
}
