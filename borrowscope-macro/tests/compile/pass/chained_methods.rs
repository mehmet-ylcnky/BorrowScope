use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_chained_methods() {
    let s = String::from("  hello  ");
    let result = s.trim().to_uppercase();
    assert_eq!(result, "HELLO");
}

fn main() {
    test_chained_methods();
}
