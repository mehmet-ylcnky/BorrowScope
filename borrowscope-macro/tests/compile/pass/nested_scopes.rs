use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_nested_scopes() {
    let x = 42;
    {
        let y = 100;
        assert_eq!(y, 100);
    }
    let z = 200;
    assert_eq!(x, 42);
    assert_eq!(z, 200);
}

fn main() {
    test_nested_scopes();
}
