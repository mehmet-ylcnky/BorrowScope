#![allow(unused_variables, unused_imports, dead_code, unused_mut, unused_assignments)]

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::rc::Rc;
use std::cell::{OnceCell, RefCell, Cell};
use std::mem::MaybeUninit;
use std::borrow::Cow;
use std::thread;
use std::time::Duration;
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

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

#[trace_borrow]
pub fn create_weak() {
        let strong = Rc::new(42);
        let _weak = Rc::downgrade(&strong);
    }

#[trace_borrow]
pub fn create_weak_sync() {
        let strong = Arc::new(42);
        let _weak = Arc::downgrade(&strong);
    }

#[trace_borrow]
pub fn clone_weak() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        let _weak2 = weak.clone();
    }

#[trace_borrow]
pub fn upgrade_weak() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        let _upgraded = weak.upgrade();
    }

#[trace_borrow]
pub fn upgrade_after_drop() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);
        drop(strong);
        let _result = weak.upgrade(); // Should be None
    }

#[trace_borrow]
pub fn spawn_thread() {
        let handle = thread::spawn(|| 42);
        let _ = handle.join();
    }

#[trace_borrow]
pub fn join_thread() {
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(1));
            42
        });
        let _ = handle.join();
    }

#[trace_borrow]
pub fn create_channel() {
        let (_tx, _rx) = mpsc::channel::<i32>();
    }

#[trace_borrow]
pub fn channel_with_names() {
        let (sender, receiver) = mpsc::channel::<i32>();
        drop(sender);
        drop(receiver);
    }

#[trace_borrow]
pub fn combined_ops() {
        // Cow operations
        let mut cow: Cow<str> = Cow::Borrowed("test");
        cow.to_mut().push_str("!");

        // Weak operations
        let rc = Rc::new(100);
        let weak = Rc::downgrade(&rc);
        let weak2 = weak.clone();
        let _ = weak.upgrade();
        let _ = weak2.upgrade();
    }

#[trace_borrow]
pub fn thread_channel_combo() {
        let channel = mpsc::channel();
        let tx = channel.0;
        let rx = channel.1;

        let handle = thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let _ = rx.recv();
        let _ = handle.join();
    }

#[trace_borrow]
pub fn lock_mutex() {
        let mutex = Mutex::new(42);
        let _guard = mutex.lock().unwrap();
    }

#[trace_borrow]
pub fn read_rwlock() {
        let rwlock = RwLock::new(100);
        let _guard = rwlock.read().unwrap();
    }

#[trace_borrow]
pub fn write_rwlock() {
        let rwlock = RwLock::new(100);
        let _guard = rwlock.write().unwrap();
    }

#[trace_borrow]
pub fn create_ref_closure() {
        let x = 10;
        let _closure = |a: i32| a + x;
    }

#[trace_borrow]
pub fn create_move_closure() {
        let data = vec![1, 2, 3];
        let _closure = move || data.len();
    }

#[trace_borrow]
pub fn closure_with_captures() {
        let x = 10;
        let y = 20;
        let _closure = |a: i32| a + x + y;
    }

#[trace_borrow]
pub fn move_closure_captures() {
        let data = String::from("hello");
        let _closure = move || data.len();
    }

#[trace_borrow]
pub fn create_box() {
        let _boxed = Box::new(42);
    }

#[trace_borrow]
pub fn box_to_raw() {
        let boxed = Box::new(42);
        let _ptr = Box::into_raw(boxed);
    }

#[trace_borrow]
pub fn box_from_raw() {
        let boxed = Box::new(42);
        let ptr = Box::into_raw(boxed);
        let _recovered = unsafe { Box::from_raw(ptr) };
    }

#[trace_borrow]
pub fn box_roundtrip() {
        let original = Box::new(String::from("test"));
        let ptr = Box::into_raw(original);
        let recovered = unsafe { Box::from_raw(ptr) };
        assert_eq!(*recovered, "test");
    }

#[trace_borrow]
pub fn combined_test() {
        // Lock guard
        let mutex = Mutex::new(42);
        {
            let _guard = mutex.lock().unwrap();
        }

        // Closure
        let x = 10;
        let _closure = |a: i32| a + x;

        // Box raw
        let boxed = Box::new(100);
        let _ptr = Box::into_raw(boxed);
    }

#[trace_borrow]
pub fn test_fn_1() {
        use std::cell::OnceCell;
        let _cell: OnceCell<i32> = OnceCell::new();
    }

#[trace_borrow]
pub fn test_fn_2() {
        use std::sync::OnceLock;
        let _lock: OnceLock<i32> = OnceLock::new();
    }

#[trace_borrow]
pub fn test_fn_3() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.set(42);
    }

#[trace_borrow]
pub fn test_fn_4() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.get();
    }

#[trace_borrow]
pub fn test_fn_5() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.get_or_init(|| 42);
    }

#[trace_borrow]
pub fn test_fn_6() {
        use std::mem::MaybeUninit;
        let _uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    }

#[trace_borrow]
pub fn test_fn_7() {
        use std::mem::MaybeUninit;
        let _init: MaybeUninit<i32> = MaybeUninit::new(42);
    }

#[trace_borrow]
pub fn test_fn_8() {
        use std::mem::MaybeUninit;
        let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();
        let _ = uninit.write(42);
    }

#[trace_borrow]
pub fn test_fn_9() {
        use std::mem::MaybeUninit;
        let init: MaybeUninit<i32> = MaybeUninit::new(42);
        let _ = unsafe { init.assume_init() };
    }

#[trace_borrow]
pub fn test_fn_10() {
        use std::mem::MaybeUninit;
        let init: MaybeUninit<i32> = MaybeUninit::new(42);
        let _ = unsafe { init.assume_init_read() };
    }

#[trace_borrow]
pub fn test_fn_11() {
        use std::mem::MaybeUninit;
        let mut init: MaybeUninit<String> = MaybeUninit::new(String::from("test"));
        let val = unsafe { std::ptr::read(init.as_ptr()) }; drop(val);
    }

#[trace_borrow]
pub fn test_fn_12() {
        use std::cell::OnceCell;
        let cell: OnceCell<String> = OnceCell::new();
        let _ = cell.get(); // None
        let _ = cell.set(String::from("value"));
        let _ = cell.get(); // Some
        let _ = cell.get_or_init(|| String::from("other")); // Returns existing
    }

#[trace_borrow]
pub fn test_fn_13() {
        use std::mem::MaybeUninit;
        let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();
        let _ = uninit.write(42);
        let _ = unsafe { uninit.assume_init() };
    }

#[trace_borrow]
pub fn test_fn() {
        let _x = Rc::new(42);
    }

#[trace_borrow]
pub fn test_fn_x2() -> i32 {
        let x = Rc::new(42);
        *x
    }

#[trace_borrow]
pub fn test_fn_x3() -> (i32, i32) {
        let x = Rc::new(42);
        let y = Rc::clone(&x);
        (*x, *y)
    }

#[trace_borrow]
pub fn test_fn_x4() -> String {
        let x = Rc::new(String::from("hello"));
        (*x).clone()
    }

#[trace_borrow]
pub fn test_fn_x5() -> usize {
        let v = Arc::new(vec![1, 2, 3, 4, 5]);
        v.len()
    }

#[trace_borrow]
pub fn create_and_clone_arc() -> (Arc<i32>, Arc<i32>) {
        let x = Arc::new(42);
        let x_clone = Arc::clone(&x);
        (x, x_clone)
    }

#[trace_borrow]
pub fn test_fn_x6() -> Option<i32> {
        let x = Rc::new(Some(42));
        *x
    }

#[trace_borrow]
pub fn test_fn_x7() -> std::result::Result<i32, String> {
        let x = Rc::new(Ok::<i32, String>(42));
        (*x).clone()
    }

#[trace_borrow]
pub fn example_adv1() {
        let _x = 1;
        let _y = 2;
        let _z = 3;
    }

#[trace_borrow]
pub fn example_adv2() {
        let x = String::from("hello");
        let _y = x; // Move
    }

#[trace_borrow]
pub fn example_adv3() {
        let x = 42;
        let _r = &x;
    }

#[trace_borrow]
pub fn example_adv4() {
        let mut x = vec![1, 2, 3];
        let _r = &mut x;
    }

#[trace_borrow]
pub fn example_adv5() {
        let _x = 42;
    }

#[trace_borrow]
pub fn example_adv6() {
        let x = String::from("hello");
        let r1 = &x;
        let _r2 = &r1;
    }

#[trace_borrow]
pub fn example_adv7() {
        let _a = 1;
        let _b = 2;
        let _c = 3;
        let _d = 4;
        let _e = 5;
    }

#[trace_borrow]
pub fn example_adv8() {
        let x = 42;
        let y = x;
        let _z = y;
    }

#[trace_borrow]
pub fn example_adv9() {
        let x = String::from("test");
        let _r = &x;
        let _y = x;
    }

#[trace_borrow]
pub fn example_adv10() {
        let _x = Box::new(42);
    }

#[trace_borrow]
pub fn example_adv11() {
        let _x = 1;
        let _y = 2;
        let _z = 3;
    }

#[trace_borrow]
pub fn example<T>(value: T) -> T {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_2<T>(value: T) -> T {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_3<T>(value: T) -> T {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_4<T, U>(t: T, u: U) -> (T, U) {
        let x = t;
        let y = u;
        (x, y)
    }

#[trace_borrow]
pub fn example_5<T: Clone>(value: T) -> T {
        let x = value.clone();
        x
    }

#[trace_borrow]
pub fn example_6<T>(value: T) -> T
    where
        T: Clone + std::fmt::Debug,
    {
        let x = value.clone();
        x
    }

#[trace_borrow]
pub fn example_7<'a>(x: &'a str) -> &'a str {
        let y = x;
        y
    }

#[trace_borrow]
pub fn example_8<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
        let z = x;
        z
    }

#[trace_borrow]
pub fn example_9<const N: usize>(arr: [i32; N]) -> [i32; N] {
        let x = arr;
        x
    }

#[trace_borrow]
pub fn example_10<T>(value: Option<T>) -> Option<T> {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_11<T, E>(value: std::result::Result<T, E>) -> std::result::Result<T, E> {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_12<T>(value: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_13<T, U>(value: (T, U)) -> (T, U) {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_14<T: Default>() -> T {
        let x = T::default();
        x
    }

#[trace_borrow]
pub fn example_15<T: Clone>(value: T) -> T {
        let x = value.clone();
        let y = x.clone();
        y
    }

#[trace_borrow]
pub fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
        let result = a + b;
        result
    }

#[trace_borrow]
pub fn example_16<T>(value: Box<T>) -> Box<T> {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_17<T>(value: &T) -> &T {
        let x = value;
        x
    }

#[trace_borrow]
pub fn example_18<T: Clone>(value: T) -> T {
        // Use clone to create a new value (New event)
        let x = value.clone();
        x
    }

#[trace_borrow]
pub fn example_x2() {
        let x = 42;
        assert_eq!(x, 42);
    }

#[trace_borrow]
pub fn example_2_x2() {
        let x: i32 = 42;
        assert_eq!(x, 42);
    }

#[trace_borrow]
pub fn example_3_x2() {
        let s = String::from("hello");
        assert_eq!(s, "hello");
    }

#[trace_borrow]
pub fn example_4_x2() {
        let v = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }

#[trace_borrow]
pub fn example_5_x2() {
        let x = 42;
        let y = 100;
        let z = x + y;
        assert_eq!(z, 142);
    }

#[trace_borrow]
pub fn example_6_x2() {
        let x = 1 + 2 * 3;
        assert_eq!(x, 7);
    }

#[trace_borrow]
pub fn example_7_x2() {
        let x = get_value_helper();
        assert_eq!(x, 42);
    }

#[trace_borrow]
pub fn example_8_x2() {
        let mut x = 42;
        x += 1;
        assert_eq!(x, 43);
    }

#[trace_borrow]
pub fn example_9_x2() {
        let x = 1;
        {
            let y = 2;
            assert_eq!(y, 2);
        }
        assert_eq!(x, 1);
    }

#[trace_borrow]
pub fn example_10_x2() -> i32 {
        let x = 42;
        x
    }

