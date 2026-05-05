//! Async tracking: async blocks, await expressions
//!
//! Run with: cargo run --example async_ops

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

// Simple future for demonstration
struct Ready<T>(Option<T>);

impl<T> Future for Ready<T> {
    type Output = T;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're not moving the data, just taking from the Option
        let this = unsafe { self.get_unchecked_mut() };
        Poll::Ready(this.0.take().unwrap())
    }
}

fn ready<T>(value: T) -> Ready<T> {
    Ready(Some(value))
}

#[trace_borrow]
async fn async_basic() {
    let x = 42;
    println!("Async basic: {}", x);
}

#[trace_borrow]
async fn async_with_await() {
    let value = ready(42).await;
    println!("Awaited value: {}", value);
}

#[trace_borrow]
async fn async_multiple_awaits() {
    let a = ready(10).await;
    let b = ready(20).await;
    let c = ready(30).await;
    println!("Sum: {}", a + b + c);
}

#[trace_borrow]
async fn async_with_ownership() {
    let s = String::from("hello");
    let len = ready(s.len()).await;
    println!("Length: {}", len);
    // s is still valid
    println!("String: {}", s);
}

#[trace_borrow]
async fn async_move_across_await() {
    let s = String::from("moved");
    let result = ready(s).await; // s moved into future
    println!("Result: {}", result);
}

#[trace_borrow]
async fn async_borrow_before_await() {
    let data = vec![1, 2, 3];
    let r = &data;
    println!("Before await: {:?}", r);
    
    let _ = ready(()).await;
    
    // Can still use data after await
    println!("After await: {:?}", data);
}

#[trace_borrow]
async fn async_with_loop() {
    for i in 0..3 {
        let value = ready(i * 10).await;
        println!("Iteration {}: {}", i, value);
    }
}

#[trace_borrow]
async fn async_with_branch() {
    let condition = true;
    
    if condition {
        let a = ready(1).await;
        println!("Branch A: {}", a);
    } else {
        let b = ready(2).await;
        println!("Branch B: {}", b);
    }
}


async fn async_nested() {
    async fn inner() -> i32 {
        ready(42).await
    }
    
    let result = inner().await;
    println!("Nested result: {}", result);
}


fn async_block_in_sync() {
    let future = async {
        let x = 10;
        let y = ready(20).await;
        x + y
    };
    
    // Note: We're not actually running the future here
    println!("Created async block");
}

#[trace_borrow]
async fn async_with_closure() {
    let multiplier = 2;
    let values = vec![1, 2, 3];
    
    for v in values {
        let result = ready(v * multiplier).await;
        println!("{} * {} = {}", v, multiplier, result);
    }
}

// Simple blocking executor for running async functions
fn block_on<F: Future>(mut future: F) -> F::Output {
    use std::task::{RawWaker, RawWakerVTable, Waker};
    
    fn dummy_raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker { dummy_raw_waker() }
        
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    
    let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    
    loop {
        let future = unsafe { Pin::new_unchecked(&mut future) };
        match future.poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => panic!("Future returned Pending"),
        }
    }
}

fn main() {
    println!("=== Async Basic ===");
    reset();
    block_on(async_basic());
    print_events("async_basic");

    println!("\n=== Async with Await ===");
    reset();
    block_on(async_with_await());
    print_events("async_with_await");

    println!("\n=== Async Multiple Awaits ===");
    reset();
    block_on(async_multiple_awaits());
    print_events("async_multiple_awaits");

    println!("\n=== Async with Ownership ===");
    reset();
    block_on(async_with_ownership());
    print_events("async_with_ownership");

    println!("\n=== Async Move Across Await ===");
    reset();
    block_on(async_move_across_await());
    print_events("async_move_across_await");

    println!("\n=== Async Borrow Before Await ===");
    reset();
    block_on(async_borrow_before_await());
    print_events("async_borrow_before_await");

    println!("\n=== Async with Loop ===");
    reset();
    block_on(async_with_loop());
    print_events("async_with_loop");

    println!("\n=== Async with Branch ===");
    reset();
    block_on(async_with_branch());
    print_events("async_with_branch");

    println!("\n=== Async Nested ===");
    reset();
    block_on(async_nested());
    print_events("async_nested");

    println!("\n=== Async Block in Sync ===");
    reset();
    async_block_in_sync();
    print_events("async_block_in_sync");

    println!("\n=== Async with Closure ===");
    reset();
    block_on(async_with_closure());
    print_events("async_with_closure");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
