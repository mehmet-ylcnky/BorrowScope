use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example<T>(value: T) -> T {
    value
}

fn main() {
    let result = example(42);
    assert_eq!(result, 42);
}
