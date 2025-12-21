use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_multiple_macros() {
    let v = vec![1, 2, 3];
    let s = format!("Length: {}", v.len());
    println!("{}", s);
    assert_eq!(s, "Length: 3");
}

fn main() {
    test_multiple_macros();
}
