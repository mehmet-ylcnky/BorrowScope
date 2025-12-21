use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_method_with_args() {
    let s = String::from("hello world");
    let contains = s.contains("world");
    assert!(contains);
}

fn main() {
    test_method_with_args();
}
