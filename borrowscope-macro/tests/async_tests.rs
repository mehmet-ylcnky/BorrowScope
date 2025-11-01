//! Tests for async function support
//!
//! These tests verify that the #[trace_borrow] macro works correctly
//! with async functions and async blocks.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use serial_test::serial;

#[tokio::test]
#[serial]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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

// Advanced async patterns

#[tokio::test]
#[serial]
async fn test_async_with_channels() {
    reset();

    use tokio::sync::mpsc;

    #[trace_borrow]
    async fn create_value() -> i32 {
        let value = 42;
        value
    }

    let (tx, mut rx) = mpsc::channel(1);

    let value = create_value().await;
    tx.send(value).await.unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(received, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_select() {
    reset();

    #[trace_borrow]
    async fn task1() -> i32 {
        let x = 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        x
    }

    #[trace_borrow]
    async fn task2() -> i32 {
        let y = 2;
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        y
    }

    let result = tokio::select! {
        v1 = task1() => v1,
        v2 = task2() => v2,
    };

    assert!(result == 1 || result == 2);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_join() {
    reset();

    #[trace_borrow]
    async fn task1() -> i32 {
        let x = 10;
        x
    }

    #[trace_borrow]
    async fn task2() -> i32 {
        let y = 20;
        y
    }

    let (r1, r2) = tokio::join!(task1(), task2());

    assert_eq!(r1, 10);
    assert_eq!(r2, 20);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_timeout() {
    reset();

    #[trace_borrow]
    async fn slow_task() -> i32 {
        let x = 42;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        x
    }

    let result = tokio::time::timeout(tokio::time::Duration::from_millis(10), slow_task()).await;

    assert!(result.is_err()); // Should timeout

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_arc_mutex() {
    reset();

    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[trace_borrow]
    async fn increment(mut counter: Arc<Mutex<i32>>) {
        let mut guard = counter.lock().await;
        let value = *guard;
        *guard = value + 1;
    }

    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counter_clone = Arc::clone(&counter);
            tokio::spawn(increment(counter_clone))
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let final_value = *counter.lock().await;
    assert_eq!(final_value, 10);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_error_propagation() {
    reset();

    #[trace_borrow]
    async fn may_fail(should_fail: bool) -> std::result::Result<i32, String> {
        let x = 42;
        if should_fail {
            return Err("failed".to_string());
        }
        Ok(x)
    }

    let result_ok = may_fail(false).await;
    assert_eq!(result_ok, Ok(42));

    let result_err = may_fail(true).await;
    assert!(result_err.is_err());

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_try_join() {
    reset();

    #[trace_borrow]
    async fn task1() -> std::result::Result<i32, String> {
        let x = 10;
        Ok(x)
    }

    #[trace_borrow]
    async fn task2() -> std::result::Result<i32, String> {
        let y = 20;
        Ok(y)
    }

    let result = tokio::try_join!(task1(), task2());

    assert_eq!(result, Ok((10, 20)));

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_spawn_blocking() {
    reset();

    #[trace_borrow]
    async fn async_wrapper() -> i32 {
        let x = 42;
        let result = tokio::task::spawn_blocking(move || {
            // CPU-intensive work
            x * 2
        })
        .await
        .unwrap();
        result
    }

    let result = async_wrapper().await;
    assert_eq!(result, 84);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_interval() {
    reset();

    #[trace_borrow]
    async fn tick_task() -> i32 {
        let mut count = 0;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1));

        for _ in 0..3 {
            interval.tick().await;
            count += 1;
        }

        count
    }

    let result = tick_task().await;
    assert_eq!(result, 3);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_oneshot() {
    reset();

    use tokio::sync::oneshot;

    #[trace_borrow]
    async fn create_value() -> i32 {
        let value = 42;
        value
    }

    let (tx, rx) = oneshot::channel();

    let value = create_value().await;
    tx.send(value).unwrap();

    let result = rx.await.unwrap();

    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_broadcast() {
    reset();

    use tokio::sync::broadcast;

    #[trace_borrow]
    async fn create_value() -> i32 {
        let value = 42;
        value
    }

    let (tx, mut rx1) = broadcast::channel(16);
    let mut rx2 = tx.subscribe();

    let value = create_value().await;
    tx.send(value).unwrap();

    let result1 = rx1.recv().await.unwrap();
    let result2 = rx2.recv().await.unwrap();

    assert_eq!(result1, 42);
    assert_eq!(result2, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_semaphore() {
    reset();

    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[trace_borrow]
    async fn limited_task(mut sem: Arc<Semaphore>, id: i32) -> i32 {
        let _permit = sem.acquire().await.unwrap();
        let result = id * 2;
        result
    }

    let sem = Arc::new(Semaphore::new(2));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let sem_clone = Arc::clone(&sem);
            tokio::spawn(limited_task(sem_clone, i))
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 5);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_barrier() {
    reset();

    use std::sync::Arc;
    use tokio::sync::Barrier;

    #[trace_borrow]
    async fn barrier_task(mut barrier: Arc<Barrier>, id: i32) -> i32 {
        let value = id;
        barrier.wait().await;
        value
    }

    let barrier = Arc::new(Barrier::new(3));

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let barrier_clone = Arc::clone(&barrier);
            tokio::spawn(barrier_task(barrier_clone, i))
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    assert_eq!(results.len(), 3);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_watch() {
    reset();

    use tokio::sync::watch;

    #[trace_borrow]
    async fn create_value() -> i32 {
        let value = 42;
        value
    }

    let (tx, mut rx) = watch::channel(0);

    let value = create_value().await;
    tx.send(value).unwrap();

    rx.changed().await.unwrap();
    let result = *rx.borrow();

    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_notify() {
    reset();

    use std::sync::Arc;
    use tokio::sync::Notify;

    #[trace_borrow]
    async fn waiter_task(notify: Arc<Notify>) -> i32 {
        notify.notified().await;
        let value = 42;
        value
    }

    #[trace_borrow]
    async fn notifier_task(notify: Arc<Notify>) {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        notify.notify_one();
    }

    let notify = Arc::new(Notify::new());

    let waiter = tokio::spawn(waiter_task(Arc::clone(&notify)));
    let notifier = tokio::spawn(notifier_task(notify));

    notifier.await.unwrap();
    let result = waiter.await.unwrap();

    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_async_with_rwlock() {
    reset();

    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[trace_borrow]
    async fn reader_task(data: Arc<RwLock<i32>>) -> i32 {
        let guard = data.read().await;
        let value = *guard;
        value
    }

    #[trace_borrow]
    async fn writer_task(mut data: Arc<RwLock<i32>>) {
        let mut guard = data.write().await;
        *guard = 42;
    }

    let data = Arc::new(RwLock::new(0));

    writer_task(Arc::clone(&data)).await;
    let result = reader_task(data).await;

    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}
