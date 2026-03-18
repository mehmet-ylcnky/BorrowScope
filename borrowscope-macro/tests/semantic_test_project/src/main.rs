use std::cell::{Cell, RefCell, OnceCell};
use std::sync::{Mutex, RwLock};
use std::sync::mpsc;
use std::mem::MaybeUninit;
use std::borrow::Cow;
use std::rc::{Rc, Weak};
use std::sync::Arc;

fn test_basic_variables() {
    let x = 5;
    let y = &x;
    let mut z = 10;
    let w = &mut z;
    let _ = (*w, y);
}

fn test_multiple_variables() {
    let a = 1;
    let b = 2;
    let c = 3;
    let _ = (a, b, c);
}

fn test_nested_blocks() {
    let outer = 1;
    {
        let inner = 2;
        let _ = inner;
    }
    let _ = outer;
}

fn test_scope_depth() {
    let depth0 = 1;
    {
        let depth1 = 2;
        {
            let depth2 = 3;
            let _ = depth2;
        }
        let _ = depth1;
    }
    let _ = depth0;
}

fn test_clone(data: Vec<i32>) {
    let cloned = data.clone();
    let _ = cloned;
}

fn test_cell_operations() {
    let counter = Cell::new(0);
    counter.set(42);
    let val = counter.get();
    let _ = val;
}

fn test_refcell_operations() {
    let refcell = RefCell::new(String::new());
    let borrowed = refcell.borrow();
    let _ = &*borrowed;
    drop(borrowed);
    let mut_borrowed = refcell.borrow_mut();
    let _ = &*mut_borrowed;
}

fn test_mutex_rwlock() {
    let mutex = Mutex::new(0);
    let guard = mutex.lock().unwrap();
    let _ = *guard;
    drop(guard);
    let try_guard = mutex.try_lock();
    let _ = try_guard;

    let rwlock = RwLock::new(0);
    let read_guard = rwlock.read().unwrap();
    let _ = *read_guard;
    drop(read_guard);
    let write_guard = rwlock.write().unwrap();
    let _ = *write_guard;
    drop(write_guard);
    let try_read = rwlock.try_read();
    let _ = try_read;
    let try_write = rwlock.try_write();
    let _ = try_write;
}

fn test_option_result() {
    let option: Option<i32> = Some(42);
    let unwrapped = option.unwrap();
    let _ = unwrapped;

    let result: Result<i32, &str> = Ok(42);
    let expected = result.expect("error");
    let _ = expected;
}

fn test_rc_arc() {
    let rc = Rc::new(42);
    let rc2 = rc.clone();
    let weak: Weak<i32> = Rc::downgrade(&rc);
    let upgraded = weak.upgrade();
    let _ = (rc2, upgraded);

    let arc = Arc::new(42);
    let arc2 = arc.clone();
    let _ = arc2;
}

fn test_cow() {
    let cow: Cow<str> = Cow::Borrowed("hello");
    let owned = cow.to_string();
    let mut cow2: Cow<str> = Cow::Borrowed("world");
    let mutref = cow2.to_mut();
    let _ = (owned, mutref);
}

fn test_channel() {
    let (tx, rx) = mpsc::channel::<i32>();
    let _ = tx.send(42);
    let received = rx.recv();
    let _ = received;
}

fn test_once_cell() {
    let once = OnceCell::new();
    let _ = once.set(42);
    let got = once.get();
    let _ = got;
    let once2 = OnceCell::<i32>::new();
    let inited = once2.get_or_init(|| 99);
    let _ = inited;
}

fn test_maybe_uninit() {
    let mut uninit = MaybeUninit::<i32>::uninit();
    uninit.write(42);
    let inited = unsafe { uninit.assume_init() };
    let _ = inited;
    let mut zeroed = MaybeUninit::<i32>::zeroed();
    zeroed.write(7);
    let read_val = unsafe { zeroed.assume_init_read() };
    let _ = read_val;
    let mut to_drop = MaybeUninit::<i32>::uninit();
    to_drop.write(99);
    unsafe { to_drop.assume_init_drop() };
}

fn test_transmute() {
    let src: u32 = 42;
    let dst: f32 = unsafe { std::mem::transmute::<u32, f32>(src) };
    let _ = dst;
    // Second transmute — tests line-based disambiguation
    let src2: u64 = 123;
    let dst2: f64 = unsafe { std::mem::transmute::<u64, f64>(src2) };
    let _ = dst2;
}

pub fn test_raw_ptr() {
    let b = Box::new(42);
    let _p: *mut i32 = Box::into_raw(b);
    let x = 10;
    let _q: *const i32 = &x as *const i32;
}

fn test_pin() {
    use std::pin::Pin;
    let mut x = 42;
    let mut pinned = Pin::new(&mut x);
    let _r = pinned.as_ref();
    let _m = pinned.as_mut();
}

fn test_visibility() {
    let visible = 5;
    let _ = visible;
}

fn test_function_signature(param: i32) -> i32 {
    let local = param;
    local
}

fn test_join_handle() {
    let handle = std::thread::spawn(|| 42);
    let joined = handle.join().unwrap();
    let _ = joined;
}

fn test_self_borrow_kinds() {
    let mut v = vec![1, 2, 3];
    v.push(4);           // &mut self
    v.sort();            // &mut self
    let _l = v.len();    // &self
    let _f = v.first();  // &self
    let _it = v.into_iter(); // self (consuming)
}

fn helper_fn(x: i32) -> i32 { x + 1 }

fn test_callable() {
    let f: fn(i32) -> i32 = helper_fn;
    let _result = f(42);
}

fn test_field_access() {
    struct Point { x: i32, y: i32 }
    let p = Point { x: 1, y: 2 };
    let _px = p.x;
    let _py = p.y;
}

fn test_atomic() {
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let flag = AtomicBool::new(false);
    let atom_count = AtomicUsize::new(0);
    let _ = flag;
    let _ = atom_count;
}

fn test_time_types() {
    use std::time::{Duration, Instant};
    let dur = Duration::from_secs(1);
    let now = Instant::now();
    let _ = dur;
    let _ = now;
}

fn main() {
    test_basic_variables();
    test_multiple_variables();
    test_nested_blocks();
    test_scope_depth();
    test_clone(vec![1, 2, 3]);
    test_cell_operations();
    test_refcell_operations();
    test_mutex_rwlock();
    test_option_result();
    test_rc_arc();
    test_cow();
    test_channel();
    test_once_cell();
    test_maybe_uninit();
    test_transmute();
    test_raw_ptr();
    test_pin();
    test_join_handle();
    test_self_borrow_kinds();
    test_callable();
    test_field_access();
    test_atomic();
    test_time_types();
    test_visibility();
    let _ = test_function_signature(42);
}
