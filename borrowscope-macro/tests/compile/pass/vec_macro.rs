use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_vec_macro() {
    let v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
}

fn main() {
    test_vec_macro();
}
