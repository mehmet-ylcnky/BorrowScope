use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_mixed_pattern() {
    let (num, text) = (42, "hello");
    let simple = 100;
    assert_eq!(num, 42);
    assert_eq!(text, "hello");
    assert_eq!(simple, 100);
}

fn main() {
    test_mixed_pattern();
}
