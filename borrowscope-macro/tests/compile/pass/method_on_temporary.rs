use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_method_on_temporary() {
    let len = String::from("hello").len();
    assert_eq!(len, 5);
}

fn main() {
    test_method_on_temporary();
}
