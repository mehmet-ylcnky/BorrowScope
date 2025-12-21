use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_if() {
    let condition = true;
    if condition {
        let x = 42;
        assert_eq!(x, 42);
    }
}

fn main() {
    test_if();
}
