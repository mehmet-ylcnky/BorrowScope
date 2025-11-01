//! Tests for async function support
//!
//! These tests verify that the #[trace_borrow] macro works correctly
//! with async functions and async blocks.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[tokio::test]
async fn test_async_function_basic() {
    reset();

    #[trace_borrow]
    async fn async_example() -> i32 {
        let x = 42;
        x
    }

    let result = async_example().await;
    assert_eq!(result, 42);

    let events = get_events();
    assert!(
        !events.is_empty(),
        "Should track variable in async function"
    );
}

#[tokio::test]
async fn test_async_function_with_string() {
    reset();

    #[trace_borrow]
    async fn async_string() -> String {
        let s = String::from("hello");
        s
    }

    let result = async_string().await;
    assert_eq!(result, "hello");

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_await() {
    reset();

    #[trace_borrow]
    async fn helper() -> i32 {
        42
    }

    #[trace_borrow]
    async fn async_with_await() -> i32 {
        let x = helper().await;
        x
    }

    let result = async_with_await().await;
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_multiple_variables() {
    reset();

    #[trace_borrow]
    async fn async_multi() -> i32 {
        let x = 10;
        let y = 20;
        let z = 30;
        x + y + z
    }

    let result = async_multi().await;
    assert_eq!(result, 60);

    let events = get_events();
    assert!(events.len() >= 3, "Should track all variables");
}

#[tokio::test]
async fn test_async_function_with_borrow() {
    reset();

    #[trace_borrow]
    async fn async_borrow() -> usize {
        let s = String::from("hello");
        let r = &s;
        r.len()
    }

    let result = async_borrow().await;
    assert_eq!(result, 5);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_move() {
    reset();

    #[trace_borrow]
    async fn async_move() -> String {
        let s = String::from("hello");
        let t = s;
        t
    }

    let result = async_move().await;
    assert_eq!(result, "hello");

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_sleep() {
    reset();

    #[trace_borrow]
    async fn async_sleep() -> i32 {
        let x = 42;
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        x
    }

    let result = async_sleep().await;
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_return_type() {
    reset();

    #[trace_borrow]
    async fn async_result() -> std::result::Result<i32, String> {
        let x = 42;
        Ok(x)
    }

    let result = async_result().await;
    assert_eq!(result, Ok(42));

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_vec() {
    reset();

    #[trace_borrow]
    async fn async_vec() -> Vec<i32> {
        let v = vec![1, 2, 3];
        v
    }

    let result = async_vec().await;
    assert_eq!(result, vec![1, 2, 3]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_nested_calls() {
    reset();

    #[trace_borrow]
    async fn inner() -> i32 {
        let x = 10;
        x
    }

    #[trace_borrow]
    async fn outer() -> i32 {
        let a = inner().await;
        let b = inner().await;
        a + b
    }

    let result = outer().await;
    assert_eq!(result, 20);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_multi_threaded() {
    reset();

    #[trace_borrow]
    async fn async_task(id: i32) -> i32 {
        let x = id * 2;
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        x
    }

    let handle1 = tokio::spawn(async_task(1));
    let handle2 = tokio::spawn(async_task(2));

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert_eq!(result1, 2);
    assert_eq!(result2, 4);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_struct() {
    reset();

    #[derive(Debug, PartialEq)]
    struct Data {
        value: i32,
    }

    #[trace_borrow]
    async fn async_struct() -> Data {
        let d = Data { value: 42 };
        d
    }

    let result = async_struct().await;
    assert_eq!(result, Data { value: 42 });

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_with_option() {
    reset();

    #[trace_borrow]
    async fn async_option() -> Option<i32> {
        let x = Some(42);
        x
    }

    let result = async_option().await;
    assert_eq!(result, Some(42));

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_async_function_preserves_semantics() {
    reset();

    #[trace_borrow]
    async fn complex_async() -> (i32, String, Vec<i32>) {
        let num = 42;
        let text = String::from("test");
        let list = vec![1, 2, 3];
        (num, text, list)
    }

    let (n, t, l) = complex_async().await;
    assert_eq!(n, 42);
    assert_eq!(t, "test");
    assert_eq!(l, vec![1, 2, 3]);

    let events = get_events();
    assert!(events.len() >= 3);
}
