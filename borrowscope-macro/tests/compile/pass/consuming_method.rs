use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_consuming_method() {
    let s = String::from("hello");
    let bytes = s.into_bytes();
    assert_eq!(bytes.len(), 5);
}

fn main() {
    test_consuming_method();
}
