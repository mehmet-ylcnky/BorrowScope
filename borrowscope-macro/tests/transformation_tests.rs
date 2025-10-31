use borrowscope_macro::trace_borrow;

// Test 1: Simple variable declaration transformation
#[trace_borrow]
fn test_simple_variable() {
    let x = 5;
    assert_eq!(x, 5);
}

#[test]
fn test_simple_variable_works() {
    test_simple_variable();
}

// Test 2: String allocation transformation
#[trace_borrow]
fn test_string_allocation() -> String {
    let s = String::from("hello");
    s
}

#[test]
fn test_string_allocation_works() {
    let result = test_string_allocation();
    assert_eq!(result, "hello");
}

// Test 3: Immutable borrow transformation
#[trace_borrow]
fn test_immutable_borrow() {
    let x = vec![1, 2, 3];
    let r = &x;
    assert_eq!(r.len(), 3);
}

#[test]
fn test_immutable_borrow_works() {
    test_immutable_borrow();
}

// Test 4: Mutable borrow transformation
#[trace_borrow]
fn test_mutable_borrow() {
    let mut x = 5;
    let r = &mut x;
    *r += 1;
    assert_eq!(x, 6);
}

#[test]
fn test_mutable_borrow_works() {
    test_mutable_borrow();
}

// Test 5: Multiple variables
#[trace_borrow]
fn test_multiple_variables() {
    let x = 10;
    let y = 20;
    let sum = x + y;
    assert_eq!(sum, 30);
}

#[test]
fn test_multiple_variables_works() {
    test_multiple_variables();
}

// Test 6: Borrow chain
#[trace_borrow]
fn test_borrow_chain() {
    let s = String::from("test");
    let r1 = &s;
    let r2 = &s;
    assert_eq!(r1, r2);
}

#[test]
fn test_borrow_chain_works() {
    test_borrow_chain();
}

// Test 7: Function with parameters
#[trace_borrow]
fn test_with_params(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}

#[test]
fn test_with_params_works() {
    let result = test_with_params(5, 10);
    assert_eq!(result, 15);
}

// Test 8: Complex expression
#[trace_borrow]
fn test_complex_expression() {
    let x = vec![1, 2, 3];
    let doubled = x.iter().map(|n| n * 2).collect::<Vec<_>>();
    assert_eq!(doubled, vec![2, 4, 6]);
}

#[test]
fn test_complex_expression_works() {
    test_complex_expression();
}
