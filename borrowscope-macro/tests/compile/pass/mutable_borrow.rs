use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_mutable_borrow() {
    let mut x = 42;
    let r = &mut x;
    *r = 100;
    assert_eq!(*r, 100);
}

fn main() {
    test_mutable_borrow();
}
