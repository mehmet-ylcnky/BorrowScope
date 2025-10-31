use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_while_loop() {
    let mut count = 0;
    while count < 3 {
        let temp = count + 1;
        count = temp;
    }
    assert_eq!(count, 3);
}

fn main() {
    test_while_loop();
}
