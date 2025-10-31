use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_pattern_in_scope() {
    {
        let (x, y) = (1, 2);
        assert_eq!(x, 1);
        assert_eq!(y, 2);
    }
    let z = 3;
    assert_eq!(z, 3);
}

fn main() {
    test_pattern_in_scope();
}
