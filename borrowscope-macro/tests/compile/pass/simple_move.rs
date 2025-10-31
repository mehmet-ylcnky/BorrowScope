use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_simple_move() {
    let s1 = String::from("hello");
    let s2 = s1;
    assert_eq!(s2, "hello");
}

fn main() {
    test_simple_move();
}
