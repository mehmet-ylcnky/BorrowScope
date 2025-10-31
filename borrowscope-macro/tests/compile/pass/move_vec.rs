use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_move_vec() {
    let v1 = vec![1, 2, 3];
    let v2 = v1;
    assert_eq!(v2.len(), 3);
}

fn main() {
    test_move_vec();
}
