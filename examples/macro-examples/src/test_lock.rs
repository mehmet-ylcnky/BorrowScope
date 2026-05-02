use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::sync::Mutex;

#[trace_borrow]
fn test_lock() {
    let mutex = Mutex::new(42);
    let guard = mutex.lock().unwrap();
    println!("{}", *guard);
}

fn main() {
    reset();
    test_lock();
}
