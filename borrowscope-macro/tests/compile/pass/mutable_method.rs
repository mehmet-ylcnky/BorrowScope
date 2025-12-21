use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_mutable_method() {
    let mut s = String::from("hello");
    s.push('!');
    assert_eq!(s, "hello!");
}

fn main() {
    test_mutable_method();
}
