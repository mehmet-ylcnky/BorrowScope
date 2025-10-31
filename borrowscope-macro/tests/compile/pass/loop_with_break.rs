use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_loop_with_break() {
    let mut counter = 0;
    let result = loop {
        let temp = counter + 1;
        counter = temp;
        if counter >= 5 {
            break counter;
        }
    };
    assert_eq!(result, 5);
}

fn main() {
    test_loop_with_break();
}
