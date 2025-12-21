use borrowscope_macro::trace_borrow;

struct Point {
    x: i32,
    y: i32,
}

#[trace_borrow]
fn test_struct_pattern() {
    let p = Point { x: 10, y: 20 };
    let Point { x, y } = p;
    assert_eq!(x, 10);
    assert_eq!(y, 20);
}

fn main() {
    test_struct_pattern();
}
