use borrowscope_macro::trace_borrow;

fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

#[trace_borrow]
fn test_closure_as_argument() {
    let multiplier = 3;
    let result = apply(|x| x * multiplier, 10);
    assert_eq!(result, 30);
}

fn main() {
    test_closure_as_argument();
}
