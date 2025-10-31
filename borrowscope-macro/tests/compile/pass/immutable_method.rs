use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_immutable_method() {
    let s = String::from("hello");
    let len = s.len();
    assert_eq!(len, 5);
}

fn main() {
    test_immutable_method();
}
