use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_multiple_nested() {
    let a = 1;
    {
        let b = 2;
        {
            let c = 3;
            assert_eq!(c, 3);
        }
        let d = 4;
        assert_eq!(b, 2);
        assert_eq!(d, 4);
    }
    let e = 5;
    assert_eq!(a, 1);
    assert_eq!(e, 5);
}

fn main() {
    test_multiple_nested();
}
