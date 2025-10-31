use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_for_loop() {
    let vec = vec![1, 2, 3];
    let mut sum = 0;
    for item in vec {
        let doubled = item * 2;
        sum += doubled;
    }
    assert_eq!(sum, 12);
}

fn main() {
    test_for_loop();
}
