use borrowscope_macro::trace_borrow;

// Test 1: Simple function with no parameters
#[trace_borrow]
fn simple_function() {
    let x = 5;
    assert_eq!(x, 5);
}

#[test]
fn test_simple_function() {
    simple_function();
}

// Test 2: Function with parameters
#[trace_borrow]
fn function_with_params(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_function_with_params() {
    let result = function_with_params(2, 3);
    assert_eq!(result, 5);
}

// Test 3: Function with return type
#[trace_borrow]
fn function_with_return() -> String {
    String::from("hello")
}

#[test]
fn test_function_with_return() {
    let result = function_with_return();
    assert_eq!(result, "hello");
}

// Test 4: Function with generics
#[trace_borrow]
fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}

#[test]
fn test_generic_function() {
    let result = generic_function(42);
    assert_eq!(result, 42);
}

// Test 5: Function with lifetime parameters
#[trace_borrow]
fn function_with_lifetime<'a>(s: &'a str) -> &'a str {
    s
}

#[test]
fn test_function_with_lifetime() {
    let result = function_with_lifetime("test");
    assert_eq!(result, "test");
}

// Test 6: Function with borrows
#[trace_borrow]
fn function_with_borrows() {
    let x = String::from("hello");
    let r1 = &x;
    let r2 = &x;
    assert_eq!(r1, r2);
}

#[test]
fn test_function_with_borrows() {
    function_with_borrows();
}

// Test 7: Function with mutable borrows
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

// Test 8: Function with control flow
#[trace_borrow]
fn function_with_control_flow(condition: bool) -> i32 {
    if condition {
        let x = 10;
        x
    } else {
        let y = 20;
        y
    }
}

#[test]
fn test_function_with_control_flow() {
    assert_eq!(function_with_control_flow(true), 10);
    assert_eq!(function_with_control_flow(false), 20);
}

// Test 9: Function with loops
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

// Test 10: Async function
#[trace_borrow]
async fn async_function() -> i32 {
    42
}

#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, 42);
}
