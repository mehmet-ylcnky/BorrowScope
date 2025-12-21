use borrowscope_macro::trace_borrow;

// Test 1: Tuple destructuring
#[trace_borrow]
fn test_tuple_destructuring() {
    let (x, y) = (1, 2);
    assert_eq!(x, 1);
    assert_eq!(y, 2);
}

#[test]
fn test_tuple_destructuring_works() {
    test_tuple_destructuring();
}

// Test 2: Nested tuple
#[trace_borrow]
fn test_nested_tuple() {
    let (a, (b, c)) = (1, (2, 3));
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
}

#[test]
fn test_nested_tuple_works() {
    test_nested_tuple();
}

// Test 3: Tuple with different types
#[trace_borrow]
fn test_tuple_mixed_types() {
    let (num, text) = (42, "hello");
    assert_eq!(num, 42);
    assert_eq!(text, "hello");
}

#[test]
fn test_tuple_mixed_types_works() {
    test_tuple_mixed_types();
}

// Test 4: Simple pattern still works
#[trace_borrow]
fn test_simple_still_works() {
    let x = 5;
    assert_eq!(x, 5);
}

#[test]
fn test_simple_still_works_test() {
    test_simple_still_works();
}
