// All function patterns needed by macro tests
// This file ensures type-info.json contains entries for every
// #[trace_borrow] function used in the test suite.

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::rc::Rc;
use std::cell::{OnceCell, RefCell, Cell};
use std::mem::MaybeUninit;
use std::borrow::Cow;
use std::thread;

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

// === Generic functions ===
#[trace_borrow]
pub fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T { a + b }

#[trace_borrow]
pub fn example_generic<T: Clone>(val: T) -> T { val.clone() }

#[trace_borrow]
pub fn example_default<T: Default>() -> T { T::default() }

#[trace_borrow]
pub fn example_two<T: Clone, U: Default>(_t: T) -> U { U::default() }

#[trace_borrow]
pub fn example_const<const N: usize>() -> [i32; N] { [0; N] }

// === Rc/Arc operations ===
#[trace_borrow]
pub fn create_and_clone_arc() {
    let a = Arc::new(42);
    let _b = a.clone();
    let _c = Arc::clone(&a);
}

#[trace_borrow]
pub fn create_weak() {
    let rc = Rc::new(42);
    let _weak = Rc::downgrade(&rc);
}

#[trace_borrow]
pub fn create_weak_sync() {
    let arc = Arc::new(42);
    let _weak = Arc::downgrade(&arc);
}

#[trace_borrow]
pub fn clone_weak() {
    let rc = Rc::new(42);
    let weak1 = Rc::downgrade(&rc);
    let _weak2 = weak1.clone();
}

#[trace_borrow]
pub fn upgrade_weak() {
    let rc = Rc::new(42);
    let weak = Rc::downgrade(&rc);
    let _upgraded = weak.upgrade();
}

#[trace_borrow]
pub fn upgrade_after_drop() {
    let rc = Rc::new(42);
    let weak = Rc::downgrade(&rc);
    drop(rc);
    let _result = weak.upgrade();
}

// === Channels ===
#[trace_borrow]
pub fn create_channel() {
    let (_tx, _rx) = mpsc::channel::<i32>();
}

#[trace_borrow]
pub fn channel_with_names() {
    let (sender, receiver) = mpsc::channel::<i32>();
    sender.send(42).unwrap();
    let _val = receiver.recv().unwrap();
}

// === Threads ===
#[trace_borrow]
pub fn spawn_thread() {
    let handle = thread::spawn(|| 42);
    let _result = handle.join().unwrap();
}

#[trace_borrow]
pub fn join_thread() {
    let h = thread::spawn(|| "hello");
    let _s = h.join().unwrap();
}

#[trace_borrow]
pub fn thread_channel_combo() {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || { tx.send(42).unwrap(); });
    let _val = rx.recv().unwrap();
    handle.join().unwrap();
}

// === Cow ===
#[trace_borrow]
pub fn create_borrowed() {
    let _cow: Cow<str> = Cow::Borrowed("hello");
}

#[trace_borrow]
pub fn create_owned() {
    let _cow: Cow<str> = Cow::Owned(String::from("hello"));
}

#[trace_borrow]
pub fn use_to_mut() {
    let mut cow: Cow<str> = Cow::Borrowed("hello");
    cow.to_mut().push_str(" world");
}

// === Mutex/RwLock ===
#[trace_borrow]
pub fn lock_mutex() {
    let m = Mutex::new(42);
    let _guard = m.lock().unwrap();
}

#[trace_borrow]
pub fn read_rwlock() {
    let rw = RwLock::new(100);
    let _g = rw.read().unwrap();
}

#[trace_borrow]
pub fn write_rwlock() {
    let rw = RwLock::new(100);
    let _g = rw.write().unwrap();
}

// === Box ===
#[trace_borrow]
pub fn create_box() {
    let _b = Box::new(42);
}

#[trace_borrow]
pub fn box_to_raw() {
    let b = Box::new(42);
    let _raw = Box::into_raw(b);
}

#[trace_borrow]
pub fn box_from_raw() {
    let b = Box::new(42);
    let raw = Box::into_raw(b);
    let _recovered = unsafe { Box::from_raw(raw) };
}

#[trace_borrow]
pub fn box_roundtrip() {
    let b = Box::new(String::from("hello"));
    let raw = Box::into_raw(b);
    let _recovered = unsafe { Box::from_raw(raw) };
}

// === Closures ===
#[trace_borrow]
pub fn closure_with_captures() {
    let data = vec![1, 2, 3];
    let _c = || data.len();
}

#[trace_borrow]
pub fn create_ref_closure() {
    let x = String::from("hello");
    let _c = || println!("{}", x);
}

#[trace_borrow]
pub fn create_move_closure() {
    let data = vec![1, 2, 3];
    let _c = move || println!("{:?}", data);
}

#[trace_borrow]
pub fn move_closure_captures() {
    let s = String::from("captured");
    let c = move || s.len();
    let _len = c();
}

// === OnceCell ===
#[trace_borrow]
pub fn test_fn_once_cell() {
    let cell: OnceCell<i32> = OnceCell::new();
    let _ = cell.set(42);
    let _val = cell.get();
}

// === MaybeUninit ===
#[trace_borrow]
pub fn test_fn_maybe_uninit() {
    let _uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    let _init: MaybeUninit<i32> = MaybeUninit::new(42);
}

// === Combined ===
#[trace_borrow]
pub fn combined_ops() {
    let rc = Rc::new(vec![1, 2, 3]);
    let _clone = rc.clone();
    let arc = Arc::new(String::from("shared"));
    let _arc2 = arc.clone();
}

#[trace_borrow]
pub fn combined_test() {
    let _x = 42;
    let _s = String::from("hello");
    let _v = vec![1, 2, 3];
}

// === Track new edge cases ===
#[trace_borrow]
pub fn example() {
    let x = 42;
    let _y = x;
    let _s = String::from("hello");
}
