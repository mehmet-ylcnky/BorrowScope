use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_mixed() {
    let s1 = String::from("hello");
    let s2 = s1;
    let s3 = String::from("world");
    let s4 = s3;
    assert_eq!(s2, "hello");
    assert_eq!(s4, "world");
}

fn main() {
    test_mixed();
}
