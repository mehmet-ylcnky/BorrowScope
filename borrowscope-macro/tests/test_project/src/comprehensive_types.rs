// Additional type coverage for macro tests
use std::sync::{Mutex, RwLock, Arc, mpsc};
use std::cell::{OnceCell, RefCell, Cell};
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::collections::HashMap;

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
pub fn test_mutex_operations() {
    let mutex = Mutex::new(42);
    let _guard = mutex.lock().unwrap();
}

#[trace_borrow]
pub fn test_rwlock_operations() {
    let rwlock = RwLock::new(100);
    let _read_guard = rwlock.read().unwrap();
    drop(_read_guard);
    let _write_guard = rwlock.write().unwrap();
}

#[trace_borrow]
pub fn test_once_cell_operations() {
    let cell: OnceCell<i32> = OnceCell::new();
    let _ = cell.set(42);
    let _val = cell.get();
}

#[trace_borrow]
pub fn test_once_cell_string() {
    let cell: OnceCell<String> = OnceCell::new();
    let _ = cell.set(String::from("hello"));
    let _val = cell.get();
    let _init = cell.get_or_init(|| String::from("default"));
}

#[trace_borrow]
pub fn test_maybe_uninit_operations() {
    let _uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    let _init: MaybeUninit<i32> = MaybeUninit::new(42);
    let mut mu = MaybeUninit::<String>::uninit();
    mu.write(String::from("hello"));
    let _val = unsafe { mu.assume_init() };
}

#[trace_borrow]
pub fn test_channel_operations() {
    let (tx, rx) = mpsc::channel();
    tx.send(42).unwrap();
    let _val = rx.recv().unwrap();
}

#[trace_borrow]
pub fn test_channel_typed() {
    let (sender, receiver) = mpsc::channel::<i32>();
    sender.send(100).unwrap();
    let _received = receiver.recv().unwrap();
}

#[trace_borrow]
pub fn test_closure_captures() {
    let data = vec![1, 2, 3];
    let sum = data.iter().sum::<i32>();
    let _closure = || println!("{}", sum);
    let _move_closure = move || println!("{:?}", data);
}

#[trace_borrow]
pub fn test_closure_ref_capture() {
    let x = String::from("hello");
    let _borrow_closure = || println!("{}", x);
    _borrow_closure();
}

#[trace_borrow]
pub fn test_closure_mut_capture() {
    let mut counter = 0;
    let mut _inc = || counter += 1;
    _inc();
    _inc();
}

#[trace_borrow]
pub fn test_generic_simple<T: Clone + std::fmt::Debug>(value: T) {
    let _cloned = value.clone();
    println!("{:?}", _cloned);
}

#[trace_borrow]
pub fn test_generic_bounded<T: Clone + Default>(x: T) -> T {
    let _default = T::default();
    x.clone()
}

#[trace_borrow]
pub fn test_generic_multiple<T: Clone, U: Default>(_t: T, _u: U) {
    let _t2 = _t.clone();
    let _u2 = U::default();
}

#[trace_borrow]
pub fn test_rc_operations() {
    let rc1 = Rc::new(vec![1, 2, 3]);
    let rc2 = rc1.clone();
    let rc3 = Rc::clone(&rc1);
    let _weak = Rc::downgrade(&rc1);
    drop(rc2);
    drop(rc3);
}

#[trace_borrow]
pub fn test_arc_operations() {
    let arc1 = Arc::new(String::from("shared"));
    let arc2 = arc1.clone();
    let arc3 = Arc::clone(&arc1);
    let _weak = Arc::downgrade(&arc1);
    drop(arc2);
    drop(arc3);
}

#[trace_borrow]
pub fn test_refcell_operations() {
    let cell = RefCell::new(vec![1, 2, 3]);
    {
        let _guard = cell.borrow();
    }
    {
        let mut _guard = cell.borrow_mut();
        _guard.push(4);
    }
}

#[trace_borrow]
pub fn test_cell_operations() {
    let cell = Cell::new(0);
    cell.set(42);
    let _val = cell.get();
}

#[trace_borrow]
pub fn test_hashmap_operations() {
    let mut map = HashMap::new();
    map.insert("key", 42);
    let _val = map.get("key");
}

#[trace_borrow]
pub fn test_box_operations() {
    let boxed = Box::new(42);
    let _val = *boxed;
    let boxed2 = Box::new(String::from("hello"));
    let _unboxed = *boxed2;
}

#[trace_borrow]
pub fn test_option_result() {
    let opt: Option<String> = Some(String::from("value"));
    let _val = opt.unwrap();
    let result: Result<i32, String> = Ok(42);
    let _ok = result.unwrap();
}

#[trace_borrow]
pub fn test_string_operations() {
    let s = String::from("hello");
    let _s2 = s.clone();
    let _len = s.len();
    let _upper = s.to_uppercase();
}

#[trace_borrow]
pub fn test_vec_operations() {
    let mut v = vec![1, 2, 3];
    v.push(4);
    let _popped = v.pop();
    let _len = v.len();
    let _slice = &v[..];
}

#[trace_borrow]
pub fn test_iterator_operations() {
    let data = vec![1, 2, 3, 4, 5];
    let _sum: i32 = data.iter().sum();
    let _doubled: Vec<i32> = data.iter().map(|x| x * 2).collect();
}

#[trace_borrow]
pub fn test_trait_object() {
    let _boxed: Box<dyn std::fmt::Display> = Box::new(42);
    let _vec: Vec<Box<dyn std::fmt::Debug>> = vec![Box::new(1), Box::new("hello")];
}

#[trace_borrow]
pub fn test_nested_smart_pointers() {
    let _rc_vec = Rc::new(RefCell::new(vec![1, 2, 3]));
    let _arc_mutex = Arc::new(Mutex::new(HashMap::<String, i32>::new()));
}

#[trace_borrow]
pub fn create_weak_sync() {
    let strong = Arc::new(42);
    let _weak = Arc::downgrade(&strong);
}

#[trace_borrow]
pub fn create_weak_rc() {
    let strong = Rc::new(42);
    let _weak = Rc::downgrade(&strong);
}

#[trace_borrow]
pub fn test_fn_once_cell() {
    let cell: OnceCell<i32> = OnceCell::new();
    let _ = cell.set(42);
    let _val = cell.get();
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
pub fn test_box_into_raw_fn() {
    let b = Box::new(42);
    let _raw = Box::into_raw(b);
}

#[trace_borrow]
pub fn test_box_raw_roundtrip_fn() {
    let b = Box::new(String::from("hello"));
    let raw = Box::into_raw(b);
    let _recovered = unsafe { Box::from_raw(raw) };
}
