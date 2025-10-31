use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_multiple_captures() {
    let x = 10;
    let y = 20;
    let z = 30;
    let closure = |a| x + y + z + a;
    let result = closure(5);
    assert_eq!(result, 65);
}

fn main() {
    test_multiple_captures();
}
