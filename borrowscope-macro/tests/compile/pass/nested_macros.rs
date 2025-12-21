use borrowscope_macro::trace_borrow;

macro_rules! create_vec {
    ($($x:expr),*) => {
        vec![$($x),*]
    };
}

#[trace_borrow]
fn test_nested_macros() {
    let v = create_vec![1, 2, 3];
    assert_eq!(v.len(), 3);
}

fn main() {
    test_nested_macros();
}
