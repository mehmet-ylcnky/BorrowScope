use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_nested_closures() {
    let x = 10;
    let outer = |y| {
        let inner = |z| x + y + z;
        inner(5)
    };
    let result = outer(20);
    assert_eq!(result, 35);
}

fn main() {
    test_nested_closures();
}
