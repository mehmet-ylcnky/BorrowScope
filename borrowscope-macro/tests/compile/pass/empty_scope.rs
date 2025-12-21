use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_empty_scope() {
    {
        // Empty scope
    }
    let x = 42;
    assert_eq!(x, 42);
}

fn main() {
    test_empty_scope();
}
