use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_mixed_borrows() {
    let mut x = 42;
    {
        let r1 = &x;
        let r2 = &x;
        assert_eq!(*r1, 42);
        assert_eq!(*r2, 42);
    }
    let r3 = &mut x;
    *r3 = 100;
    assert_eq!(*r3, 100);
}

fn main() {
    test_mixed_borrows();
}
