use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_match() {
    let value = Some(42);
    let result = match value {
        Some(x) => {
            let doubled = x * 2;
            doubled
        }
        None => 0,
    };
    assert_eq!(result, 84);
}

fn main() {
    test_match();
}
