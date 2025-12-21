use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_lifo_drops() {
    let first = String::from("first");
    let second = String::from("second");
    let third = String::from("third");
    assert_eq!(first, "first");
    assert_eq!(second, "second");
    assert_eq!(third, "third");
}

fn main() {
    test_lifo_drops();
}
