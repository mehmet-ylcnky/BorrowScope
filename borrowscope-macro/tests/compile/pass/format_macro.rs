use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_format_macro() {
    let x = 42;
    let s = format!("Value: {}", x);
    assert_eq!(s, "Value: 42");
}

fn main() {
    test_format_macro();
}
