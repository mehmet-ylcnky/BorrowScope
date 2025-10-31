use borrowscope_macro::trace_borrow;

// Test 1: Simple function with variable declaration
#[trace_borrow]
fn simple_function() {
    let x = 5;
    assert_eq!(x, 5);
}

#[test]
fn test_simple_function() {
    simple_function();
}

// Test 2: Function with parameters and return value
#[trace_borrow]
fn function_with_params(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_function_with_params() {
    let result = function_with_params(2, 3);
    assert_eq!(result, 5);
}

// Test 3: Function returning owned value
#[trace_borrow]
fn function_with_return() -> String {
    String::from("hello")
}

#[test]
fn test_function_with_return() {
    let result = function_with_return();
    assert_eq!(result, "hello");
}

// Test 4: Generic function (tests type parameter preservation)
#[trace_borrow]
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}

#[test]
fn test_generic_function() {
    let result = generic_function(42);
    assert_eq!(result, 42);

    let string_result = generic_function(String::from("test"));
    assert_eq!(string_result, "test");
}

// Test 5: Function with lifetime parameters (tests borrow preservation)
#[trace_borrow]
fn function_with_lifetime(s: &str) -> &str {
    s
}

#[test]
fn test_function_with_lifetime() {
    let result = function_with_lifetime("test");
    assert_eq!(result, "test");
}

// Test 6: Function with multiple immutable borrows (core use case)
#[trace_borrow]
fn function_with_borrows() {
    let x = String::from("hello");
    let r1 = &x;
    let r2 = &x;
    assert_eq!(r1, r2);
    assert_eq!(r1.len(), 5);
}

#[test]
fn test_function_with_borrows() {
    function_with_borrows();
}

// Test 7: Function with mutable borrow (tests exclusive access)
#[trace_borrow]
fn function_with_mut_borrow() {
    let mut x = 5;
    let r = &mut x;
    *r += 1;
    assert_eq!(x, 6);
}

#[test]
fn test_function_with_mut_borrow() {
    function_with_mut_borrow();
}

// Test 8: Function with control flow (tests scope handling)
#[trace_borrow]
fn function_with_control_flow(condition: bool) -> i32 {
    if condition {
        
        10
    } else {
        
        20
    }
}

#[test]
fn test_function_with_control_flow() {
    assert_eq!(function_with_control_flow(true), 10);
    assert_eq!(function_with_control_flow(false), 20);
}

// Test 9: Function with loop (tests iteration and mutable state)
#[trace_borrow]
fn function_with_loop() -> i32 {
    let mut sum = 0;
    for i in 0..5 {
        sum += i;
    }
    sum
}

#[test]
fn test_function_with_loop() {
    assert_eq!(function_with_loop(), 10);
}

// Test 10: Function with move semantics
#[trace_borrow]
fn function_with_move() -> String {
    let s = String::from("moved");
     // Move occurs here
    s
}

#[test]
fn test_function_with_move() {
    let result = function_with_move();
    assert_eq!(result, "moved");
}
