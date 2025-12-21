use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_if_else() {
    let condition = true;
    let result = if condition {
        let x = 1;
        x
    } else {
        let y = 2;
        y
    };
    assert_eq!(result, 1);
}

fn main() {
    test_if_else();
}
