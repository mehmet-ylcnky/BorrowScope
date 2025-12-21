use borrowscope_macro::trace_borrow;

fn takes_ref(r: &i32) -> i32 {
    *r
}

#[trace_borrow]
fn test_borrow_in_call() {
    let x = 42;
    let result = takes_ref(&x);
    assert_eq!(result, 42);
}

fn main() {
    test_borrow_in_call();
}
