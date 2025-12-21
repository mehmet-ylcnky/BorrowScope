use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_move_chain() {
    let s1 = String::from("hello");
    let s2 = s1;
    let s3 = s2;
    assert_eq!(s3, "hello");
}

fn main() {
    test_move_chain();
}
