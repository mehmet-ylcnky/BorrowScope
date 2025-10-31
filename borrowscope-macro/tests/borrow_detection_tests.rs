use borrowscope_macro::trace_borrow;

// Test 1: Borrow in function argument
#[trace_borrow]
fn test_function_arg_borrow() {
    let x = vec![1, 2, 3];
    let len = get_len(&x);
    assert_eq!(len, 3);
}

fn get_len(v: &[i32]) -> usize {
    v.len()
}

#[test]
fn test_function_arg_borrow_works() {
    test_function_arg_borrow();
}

// Test 2: Multiple borrows in function call
#[trace_borrow]
fn test_multiple_borrows() {
    let x = 10;
    let y = 20;
    let sum = add_refs(&x, &y);
    assert_eq!(sum, 30);
}

fn add_refs(a: &i32, b: &i32) -> i32 {
    a + b
}

#[test]
fn test_multiple_borrows_works() {
    test_multiple_borrows();
}

// Test 3: Mutable borrow in function argument
#[trace_borrow]
fn test_mut_borrow_arg() {
    let mut x = 5;
    increment(&mut x);
    assert_eq!(x, 6);
}

fn increment(n: &mut i32) {
    *n += 1;
}

#[test]
fn test_mut_borrow_arg_works() {
    test_mut_borrow_arg();
}

// Test 4: Borrow in method call
#[trace_borrow]
fn test_method_borrow() {
    let data = vec![1, 2, 3];
    let first = data.first();
    assert!(first.is_some());
}

#[test]
fn test_method_borrow_works() {
    test_method_borrow();
}
