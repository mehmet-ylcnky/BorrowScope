use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_scope_with_moves() {
    let s1 = String::from("hello");
    {
        let s2 = s1;
        assert_eq!(s2, "hello");
    }
    let s3 = String::from("world");
    assert_eq!(s3, "world");
}

fn main() {
    test_scope_with_moves();
}
