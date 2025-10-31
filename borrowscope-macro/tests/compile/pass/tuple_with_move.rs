use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_tuple_with_move() {
    let s1 = String::from("hello");
    let s2 = String::from("world");
    let (a, b) = (s1, s2);
    assert_eq!(a, "hello");
    assert_eq!(b, "world");
}

fn main() {
    test_tuple_with_move();
}
