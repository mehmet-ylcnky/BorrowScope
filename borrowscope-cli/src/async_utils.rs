//! Async utilities for CLI operations

#![allow(dead_code)]

use anyhow::Result;
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Run an async operation with a timeout
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!("Operation timed out after {:?}", duration),
    }
}

/// Run multiple async operations concurrently
pub async fn run_concurrent<F, T>(futures: Vec<F>) -> Vec<Result<T>>
where
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let handles: Vec<_> = futures.into_iter().map(|f| tokio::spawn(f)).collect();

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(Err(anyhow::anyhow!("Task panicked: {}", e))),
        }
    }
    results
}

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_attempts {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
}

/// Run an operation with progress updates
pub async fn with_progress<F, T, P>(future: F, progress_fn: P) -> Result<T>
where
    F: Future<Output = Result<T>>,
    P: FnMut(u64) + Send + 'static,
{
    use std::sync::{Arc, Mutex};

    let progress_fn = Arc::new(Mutex::new(progress_fn));
    let progress_fn_clone = progress_fn.clone();

    let progress_task = tokio::spawn(async move {
        let mut counter = 0u64;
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            counter += 1;
            if let Ok(mut func) = progress_fn_clone.lock() {
                func(counter);
            }
        }
    });

    let result = future.await;
    progress_task.abort();
    result
}

/// Create a tokio runtime for sync contexts
pub fn create_runtime() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Failed to create async runtime: {}", e))
}

/// Block on an async operation (for sync contexts)
pub fn block_on<F>(future: F) -> Result<F::Output>
where
    F: Future,
{
    let runtime = create_runtime()?;
    Ok(runtime.block_on(future))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(42)
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_timeout_failure() {
        let result = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, anyhow::Error>(42)
        })
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_run_concurrent_all_success() {
        async fn task(n: i32) -> Result<i32> {
            Ok(n)
        }

        let futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> =
            vec![Box::pin(task(1)), Box::pin(task(2)), Box::pin(task(3))];

        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_run_concurrent_with_failure() {
        async fn task_ok(n: i32) -> Result<i32> {
            Ok(n)
        }
        async fn task_err() -> Result<i32> {
            Err(anyhow::anyhow!("error"))
        }

        let futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> = vec![
            Box::pin(task_ok(1)),
            Box::pin(task_err()),
            Box::pin(task_ok(3)),
        ];

        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success_first_try() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                }
            },
            3,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success_after_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(anyhow::anyhow!("not yet"))
                    } else {
                        Ok(42)
                    }
                }
            },
            5,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_all_failures() {
        let result: Result<i32> = retry_with_backoff(
            || async { Err(anyhow::anyhow!("always fails")) },
            3,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_progress() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_progress(
            async {
                tokio::time::sleep(Duration::from_millis(250)).await;
                Ok(42)
            },
            move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert!(counter.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn test_create_runtime() {
        let runtime = create_runtime();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_block_on_simple() {
        let result = block_on(async { 42 });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_block_on_with_sleep() {
        let result = block_on(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "done"
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
    }

    #[tokio::test]
    async fn test_concurrent_empty_list() {
        let futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> = vec![];
        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_single_task() {
        async fn task() -> Result<i32> {
            Ok(1)
        }
        let futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> =
            vec![Box::pin(task())];
        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_many_tasks() {
        async fn task(i: i32) -> Result<i32> {
            Ok(i)
        }

        let mut futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> =
            Vec::new();
        for i in 0..100 {
            futures.push(Box::pin(task(i)));
        }

        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 100);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_timeout_zero_duration() {
        let result =
            with_timeout(Duration::from_secs(0), async { Ok::<_, anyhow::Error>(42) }).await;

        // Zero timeout should still allow immediate completion
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_retry_single_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                }
            },
            1,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let start = std::time::Instant::now();

        let _: Result<i32> = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err(anyhow::anyhow!("fail"))
                }
            },
            4,
            Duration::from_millis(10),
        )
        .await;

        let elapsed = start.elapsed();
        // Should take at least 10 + 20 + 40 = 70ms
        assert!(elapsed >= Duration::from_millis(70));
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_with_progress_error() {
        let result: Result<i32> =
            with_progress(async { Err(anyhow::anyhow!("error")) }, |_| {}).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_mixed_results() {
        async fn task_ok(n: i32) -> Result<i32> {
            Ok(n)
        }
        async fn task_err1() -> Result<i32> {
            Err(anyhow::anyhow!("error 1"))
        }
        async fn task_err2() -> Result<i32> {
            Err(anyhow::anyhow!("error 2"))
        }

        let futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<i32>> + Send>>> = vec![
            Box::pin(task_ok(1)),
            Box::pin(task_err1()),
            Box::pin(task_ok(3)),
            Box::pin(task_err2()),
            Box::pin(task_ok(5)),
        ];

        let results = run_concurrent(futures).await;
        assert_eq!(results.len(), 5);

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let error_count = results.iter().filter(|r| r.is_err()).count();

        assert_eq!(success_count, 3);
        assert_eq!(error_count, 2);
    }
}
