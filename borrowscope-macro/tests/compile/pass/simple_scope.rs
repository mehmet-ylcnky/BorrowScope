use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_simple_scope() {
    let x = 42;
    let y = 100;
    assert_eq!(x, 42);
    assert_eq!(y, 100);
}

fn main() {
    test_simple_scope();
}
