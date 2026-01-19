//! Comprehensive type coverage test for borrowscope-analyzer
//!
//! Tests all major Rust type categories to validate semantic analysis.

use std::borrow::Cow;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::num::{NonZeroI32, NonZeroUsize};
use std::cmp::Ordering;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::task::Poll;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

// ============================================================================
// Static and Const declarations
// ============================================================================
static STATIC_I32: i32 = 42;
static STATIC_STR: &str = "hello";
static STATIC_ARRAY: [u8; 4] = [1, 2, 3, 4];
static mut STATIC_MUT: i32 = 0;

const CONST_USIZE: usize = 100;
const CONST_TUPLE: (i32, &str) = (1, "const");
const CONST_ARRAY: [i32; 3] = [1, 2, 3];

// ============================================================================
// Type aliases
// ============================================================================
type MyRc<T> = Rc<T>;
type MyArc<T> = Arc<T>;
type StringVec = Vec<String>;
type ResultI32 = Result<i32, String>;

fn test_type_aliases() {
    let my_rc: MyRc<String> = MyRc::new(String::from("aliased"));
    let my_arc: MyArc<i32> = MyArc::new(42);
    let string_vec: StringVec = vec!["a".into(), "b".into()];
    let result_ok: ResultI32 = Ok(42);
    let result_err: ResultI32 = Err("error".into());

    println!("{:?} {:?} {:?} {:?} {:?}", my_rc, my_arc, string_vec, result_ok, result_err);
}

// ============================================================================
// Binding patterns (for macro transformation decisions)
// ============================================================================
fn test_binding_patterns() {
    // Tuple bindings
    let (tuple_a, tuple_b) = (1, 2);
    let (tuple_x, tuple_y, tuple_z) = ("a", "b", "c");

    // Mutable bindings
    let mut mut_int = 42;
    let mut mut_vec = vec![1, 2, 3];
    let mut mut_string = String::new();

    // Nested tuple
    let ((nested_a, nested_b), nested_c) = ((1, 2), 3);

    // Binding modes - explicit ref patterns
    let ref ref_pattern_1 = 42;
    let ref ref_pattern_2 = String::from("ref");
    let ref ref_pattern_3 = vec![1, 2, 3];
    
    // Binding modes - explicit ref mut patterns
    let ref mut ref_mut_1 = 42;
    let ref mut ref_mut_2 = String::from("ref_mut");
    let ref mut ref_mut_3 = vec![1, 2, 3];

    // Use them to avoid warnings
    mut_int += 1;
    mut_vec.push(4);
    mut_string.push_str("hello");
    *ref_mut_1 += 1;
    ref_mut_2.push_str("!");
    ref_mut_3.push(4);
    println!("{} {} {} {} {} {} {} {} {}", tuple_a, tuple_b, tuple_x, tuple_y, tuple_z, mut_int, mut_vec.len(), mut_string, nested_a + nested_b + nested_c);
    println!("{} {} {} {} {} {}", ref_pattern_1, ref_pattern_2, ref_pattern_3.len(), ref_mut_1, ref_mut_2, ref_mut_3.len());
}

fn test_lifetime_types() {
    // Explicit 'static lifetime
    let static_str: &'static str = "static";
    let static_bytes: &'static [u8] = b"bytes";
    
    // References with inferred lifetimes
    let local = String::from("local");
    let local_ref: &str = &local;

    println!("{} {:?} {}", static_str, static_bytes, local_ref);
}

// Lifetime in struct - tests lifetime extraction from generic args
struct BorrowedData<'a> {
    data: &'a str,
}

fn test_lifetime_in_generics() {
    let owned = String::from("owned");
    let borrowed: BorrowedData<'_> = BorrowedData { data: &owned };
    println!("{}", borrowed.data);
}

// ============================================================================
// Union types
// ============================================================================
#[repr(C)]
union IntOrFloat {
    i: i32,
    f: f32,
}

#[repr(C)]
union DataUnion {
    bytes: [u8; 8],
    value: u64,
}

fn test_unions() {
    let int_or_float = IntOrFloat { i: 42 };
    let data_union = DataUnion { value: 0xDEADBEEF };

    // Reading union fields requires unsafe
    unsafe {
        let i_val = int_or_float.i;
        let f_val = int_or_float.f;
        let bytes = data_union.bytes;
        let value = data_union.value;
        println!("{} {} {:?} {}", i_val, f_val, bytes, value);
    }
}

// ============================================================================
// FFI / Extern types
// ============================================================================
fn test_ffi_types() {
    // C primitive types
    let c_int_val: c_int = 42;
    let c_char_val: c_char = b'A' as c_char;

    // c_void pointer (common in FFI)
    let mut data: i32 = 100;
    let void_ptr: *mut c_void = &mut data as *mut i32 as *mut c_void;
    let const_void_ptr: *const c_void = &data as *const i32 as *const c_void;

    // CString and CStr
    let c_string: CString = CString::new("hello").unwrap();
    let c_str: &CStr = c_string.as_c_str();

    // OsString and OsStr
    let os_string: OsString = OsString::from("path");
    let os_str: &OsStr = os_string.as_os_str();

    println!(
        "{} {} {:?} {:?} {:?} {:?} {:?} {:?}",
        c_int_val, c_char_val, void_ptr, const_void_ptr, c_string, c_str, os_string, os_str
    );
}

// ============================================================================
// Primitives and Copy types
// ============================================================================
fn test_primitives() {
    let i: i8 = 1;
    let j: i16 = 2;
    let k: i32 = 3;
    let l: i64 = 4;
    let m: i128 = 5;
    let n: isize = 6;

    let u1: u8 = 1;
    let u2: u16 = 2;
    let u3: u32 = 3;
    let u4: u64 = 4;
    let u5: u128 = 5;
    let u6: usize = 6;

    let f1: f32 = 1.0;
    let f2: f64 = 2.0;

    let b: bool = true;
    let c: char = 'x';
    let unit: () = ();

    // Tuples
    let tuple2 = (1, 2);
    let tuple3 = (1, "hello", 3.14);
    let tuple_nested = ((1, 2), (3, 4));

    // Arrays
    let arr_fixed: [i32; 5] = [1, 2, 3, 4, 5];
    let arr_repeat: [u8; 100] = [0; 100];

    println!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {:?} {:?} {:?} {:?} {:?}",
        i, j, k, l, m, n, u1, u2, u3, u4, u5, u6, f1, f2, b, c, tuple2, tuple3, tuple_nested,
        arr_fixed, arr_repeat
    );
    let _ = unit;
}

// ============================================================================
// Smart Pointers
// ============================================================================
fn test_smart_pointers() {
    // Box
    let boxed_int = Box::new(42);
    let boxed_string = Box::new(String::from("boxed"));
    let boxed_vec = Box::new(vec![1, 2, 3]);
    let boxed_nested = Box::new(Box::new(42));

    // Rc
    let rc_int = Rc::new(42);
    let rc_string = Rc::new(String::from("shared"));
    let rc_clone = Rc::clone(&rc_int);
    let weak_ref: Weak<i32> = Rc::downgrade(&rc_int);
    let weak_new: Weak<i32> = Weak::new();  // Weak::new pattern

    // Arc
    let arc_int = Arc::new(42);
    let arc_string = Arc::new(String::from("thread-safe"));
    let arc_clone = Arc::clone(&arc_int);
    let arc_weak: std::sync::Weak<i32> = Arc::downgrade(&arc_int);
    let arc_weak_new: std::sync::Weak<i32> = std::sync::Weak::new();  // Arc Weak::new

    println!(
        "{} {} {:?} {} {} {} {} {:?} {:?} {} {} {} {:?} {:?}",
        boxed_int,
        boxed_string,
        boxed_vec,
        boxed_nested,
        rc_int,
        rc_string,
        rc_clone,
        weak_ref.upgrade(),
        weak_new.upgrade(),
        arc_int,
        arc_string,
        arc_clone,
        arc_weak.upgrade(),
        arc_weak_new.upgrade()
    );
}

// ============================================================================
// Interior Mutability
// ============================================================================
fn test_interior_mutability() {
    // Cell
    let cell_int = Cell::new(42);
    let cell_bool = Cell::new(true);
    
    // UnsafeCell - the primitive for interior mutability
    use std::cell::UnsafeCell;
    let unsafe_cell: UnsafeCell<i32> = UnsafeCell::new(42);

    // RefCell
    let refcell_int = RefCell::new(42);
    let refcell_vec = RefCell::new(vec![1, 2, 3]);
    let refcell_string = RefCell::new(String::from("mutable"));

    // Mutex
    let mutex_int = Mutex::new(42);
    let mutex_vec = Mutex::new(vec![1, 2, 3]);

    // RwLock
    let rwlock_int = RwLock::new(42);
    let rwlock_map = RwLock::new(HashMap::<String, i32>::new());

    println!(
        "{} {} {:?} {} {:?} {} {:?} {:?} {:?} {:?}",
        cell_int.get(),
        cell_bool.get(),
        unsafe_cell.get(),
        refcell_int.borrow(),
        refcell_vec.borrow(),
        refcell_string.borrow(),
        mutex_int.lock(),
        mutex_vec.lock(),
        rwlock_int.read(),
        rwlock_map.read()
    );
}

// ============================================================================
// Collections
// ============================================================================
fn test_collections() {
    use std::collections::{BTreeSet, BinaryHeap, LinkedList};
    
    // Vec
    let vec_int: Vec<i32> = vec![1, 2, 3];
    let vec_string: Vec<String> = vec!["a".to_string(), "b".to_string()];
    let vec_empty: Vec<u8> = Vec::new();
    let vec_with_capacity: Vec<i32> = Vec::with_capacity(100);

    // String
    let string_new = String::new();
    let string_from = String::from("hello");
    let string_to = "world".to_string();
    let string_owned = "owned".to_owned();

    // HashMap
    let hashmap_empty: HashMap<String, i32> = HashMap::new();
    let mut hashmap_filled = HashMap::new();
    hashmap_filled.insert("key", 42);

    // HashSet
    let hashset_empty: HashSet<i32> = HashSet::new();

    // BTreeMap
    let btreemap: BTreeMap<i32, String> = BTreeMap::new();
    
    // BTreeSet
    let btreeset: BTreeSet<i32> = BTreeSet::new();

    // VecDeque
    let vecdeque: VecDeque<i32> = VecDeque::new();
    
    // LinkedList
    let linkedlist: LinkedList<i32> = LinkedList::new();
    
    // BinaryHeap
    let binaryheap: BinaryHeap<i32> = BinaryHeap::new();

    // Option and Result
    let opt_some: Option<i32> = Some(42);
    let opt_none: Option<String> = None;
    let res_ok: Result<i32, String> = Ok(42);
    let res_err: Result<i32, String> = Err("error".to_string());

    println!(
        "{:?} {:?} {:?} {:?} {} {} {} {} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        vec_int,
        vec_string,
        vec_empty,
        vec_with_capacity,
        string_new,
        string_from,
        string_to,
        string_owned,
        hashmap_empty,
        hashmap_filled,
        hashset_empty,
        btreemap,
        btreeset,
        vecdeque,
        opt_some,
        opt_none,
        res_ok,
        res_err,
        linkedlist,
        binaryheap
    );
}

// ============================================================================
// References
// ============================================================================
fn test_references() {
    let value = 42;
    let string = String::from("hello");
    let vec = vec![1, 2, 3];

    // Immutable references
    let ref_int: &i32 = &value;
    let ref_string: &String = &string;
    let ref_vec: &Vec<i32> = &vec;
    let ref_str: &str = &string;
    let ref_slice: &[i32] = &vec;

    // Mutable references
    let mut mutable = 42;
    let mut mut_string = String::from("mutable");
    let mut mut_vec = vec![1, 2, 3];

    let ref_mut_int: &mut i32 = &mut mutable;
    *ref_mut_int += 1;
    let ref_mut_string: &mut String = &mut mut_string;
    ref_mut_string.push_str("!");
    let ref_mut_vec: &mut Vec<i32> = &mut mut_vec;
    ref_mut_vec.push(4);

    println!(
        "{} {} {:?} {} {:?} {} {} {:?}",
        ref_int, ref_string, ref_vec, ref_str, ref_slice, mutable, mut_string, mut_vec
    );
}

// ============================================================================
// Raw Pointers
// ============================================================================
fn test_raw_pointers() {
    let value = 42;
    let mut mutable = 42;

    let ptr_const: *const i32 = &value;
    let ptr_mut: *mut i32 = &mut mutable;

    let null_const: *const u8 = std::ptr::null();
    let null_mut: *mut u8 = std::ptr::null_mut();

    println!(
        "{:?} {:?} {:?} {:?}",
        ptr_const, ptr_mut, null_const, null_mut
    );
}

// ============================================================================
// Function Pointers and Closures
// ============================================================================
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn test_fn_pointers() {
    let fn_ptr: fn(i32, i32) -> i32 = add;

    let closure_simple = || 42;
    let closure_args = |x: i32| x * 2;

    let value = 10;
    let closure_capture = |x: i32| x + value;
    let closure_move = {
        let v = 10;
        move || v
    };

    println!(
        "{} {} {} {} {}",
        fn_ptr(1, 2),
        closure_simple(),
        closure_args(5),
        closure_capture(5),
        closure_move()
    );
}

// ============================================================================
// Guards (borrow scope tracking)
// ============================================================================
fn test_guards() {
    use std::cell::{Ref, RefMut};
    use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

    let refcell = RefCell::new(42);
    let mutex = Mutex::new(42);
    let rwlock = RwLock::new(42);

    // RefCell guards
    let ref_guard: Ref<i32> = refcell.borrow();
    drop(ref_guard);
    let refmut_guard: RefMut<i32> = refcell.borrow_mut();
    drop(refmut_guard);

    // Mutex guard
    let mutex_guard: MutexGuard<i32> = mutex.lock().unwrap();
    drop(mutex_guard);

    // RwLock guards
    let read_guard: RwLockReadGuard<i32> = rwlock.read().unwrap();
    drop(read_guard);
    let write_guard: RwLockWriteGuard<i32> = rwlock.write().unwrap();
    drop(write_guard);

    println!("Guards tested");
}

// ============================================================================
// Futures and Iterators
// ============================================================================
async fn async_fn() -> i32 {
    42
}

async fn async_returns_string() -> String {
    String::from("async result")
}

async fn async_with_multiple_awaits() -> i32 {
    let a = async_fn().await;
    let b = async_fn().await;
    let c = async { a + b }.await;
    let _s = async_returns_string().await;
    c
}

fn test_futures_and_iterators() {
    // Future
    let future = async_fn();
    let future_block = async { 42 };
    let future_with_await = async_with_multiple_awaits();

    // Iterators - use into_iter to avoid lifetime issues
    let iter_vec = vec![1, 2, 3].into_iter();
    let iter_range = (0..10).into_iter();
    let iter_map = vec![1, 2, 3].into_iter().map(|x| x * 2);
    let iter_filter = vec![1, 2, 3].into_iter().filter(|x| *x > 1);
    let iter_chain = vec![1, 2].into_iter().chain(vec![3, 4].into_iter());

    println!(
        "{:?} {:?} {:?} {:?} {:?}",
        iter_vec.collect::<Vec<_>>(),
        iter_range.collect::<Vec<_>>(),
        iter_map.collect::<Vec<_>>(),
        iter_filter.collect::<Vec<_>>(),
        iter_chain.collect::<Vec<_>>()
    );
    let _ = (future, future_block);
}

// ============================================================================
// Complex Nested Types
// ============================================================================
fn test_nested_types() {
    // Nested smart pointers
    let rc_refcell: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(vec![1, 2, 3]));
    let arc_mutex: Arc<Mutex<HashMap<String, i32>>> = Arc::new(Mutex::new(HashMap::new()));
    let box_rc: Box<Rc<String>> = Box::new(Rc::new(String::from("nested")));

    // Nested collections
    let vec_vec: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4]];
    let map_vec: HashMap<String, Vec<i32>> = HashMap::new();
    let vec_option: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let option_vec: Option<Vec<i32>> = Some(vec![1, 2, 3]);

    // Triple nesting
    let triple: Rc<RefCell<Vec<Option<String>>>> =
        Rc::new(RefCell::new(vec![Some("a".to_string())]));

    println!(
        "{:?} {:?} {} {:?} {:?} {:?} {:?} {:?}",
        rc_refcell.borrow(),
        arc_mutex.lock(),
        box_rc,
        vec_vec,
        map_vec,
        vec_option,
        option_vec,
        triple.borrow()
    );
}

// ============================================================================
// Structs and Enums (user-defined)
// ============================================================================
#[derive(Debug)]
#[allow(dead_code)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Generic<T> {
    value: T,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Status {
    Active,
    Inactive,
    Pending(String),
}

fn test_user_types() {
    let point = Point { x: 10, y: 20 };
    let point_new = Point::new(5, 15);  // user_struct via constructor
    let generic_int = Generic { value: 42 };
    let generic_string = Generic {
        value: String::from("generic"),
    };
    let status_active = Status::Active;
    let status_inactive = Status::Inactive;
    let status_pending = Status::Pending("waiting".to_string());

    println!(
        "{:?} {:?} {:?} {:?} {:?} {:?}",
        point, generic_int, generic_string, status_active, status_inactive, status_pending
    );
}

// ============================================================================
// Path types
// ============================================================================
fn test_path() {
    let path_new = PathBuf::new();  // PathBuf::new pattern
    let path = PathBuf::from("/tmp/test");
    let path_ref: &std::path::Path = path.as_path();
    
    // OsString
    let osstring_new = OsString::new();  // OsString::new pattern
    let osstring_from = OsString::from("hello");

    println!("{:?} {:?} {:?} {:?} {:?}", path_new, path, path_ref, osstring_new, osstring_from);
}

// ============================================================================
// Advanced types: Cow, Pin, MaybeUninit, PhantomData, NonNull, NonZero
// ============================================================================
fn test_advanced_types() {
    // Cow (Clone-on-Write)
    let cow_borrowed: Cow<str> = Cow::Borrowed("borrowed");
    let cow_owned: Cow<str> = Cow::Owned(String::from("owned"));
    let cow_vec: Cow<[i32]> = Cow::Borrowed(&[1, 2, 3]);

    // Pin
    let pinned_box: Pin<Box<String>> = Box::pin(String::from("pinned"));
    let mut data = String::from("stack pinned");
    let pinned_ref: Pin<&mut String> = Pin::new(&mut data);

    // MaybeUninit
    let uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    let init: MaybeUninit<i32> = MaybeUninit::new(42);
    
    // MaybeUninit method calls
    let mut mu: MaybeUninit<i32> = MaybeUninit::uninit();
    mu.write(42);
    let _val = unsafe { mu.assume_init() };
    
    let mut mu2: MaybeUninit<i32> = MaybeUninit::new(100);
    let _read = unsafe { mu2.assume_init_read() };
    
    let mut mu3: MaybeUninit<String> = MaybeUninit::new(String::from("drop me"));
    unsafe { mu3.assume_init_drop() };

    // PhantomData
    let phantom: PhantomData<String> = PhantomData;
    let phantom_lifetime: PhantomData<&'static str> = PhantomData;

    // NonNull
    let mut value = 42i32;
    let non_null: NonNull<i32> = NonNull::from(&mut value);
    let non_null_new: Option<NonNull<i32>> = NonNull::new(&mut value);  // NonNull::new pattern
    let non_null_dangling: NonNull<i32> = NonNull::dangling();  // NonNull::dangling pattern

    // NonZero types
    let non_zero_i32: NonZeroI32 = NonZeroI32::new(42).unwrap();
    let non_zero_usize: NonZeroUsize = NonZeroUsize::new(100).unwrap();

    // ManuallyDrop
    let manual_drop: ManuallyDrop<String> = ManuallyDrop::new(String::from("manual"));
    let manual_vec: ManuallyDrop<Vec<i32>> = ManuallyDrop::new(vec![1, 2, 3]);

    println!(
        "{} {} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        cow_borrowed, cow_owned, cow_vec, pinned_box, pinned_ref,
        uninit.as_ptr(), init.as_ptr(), phantom, phantom_lifetime,
        non_null, non_zero_i32, non_zero_usize
    );
    let _ = (manual_drop, manual_vec);
}

// ============================================================================
// OnceCell, OnceLock, Range types, Duration, Instant
// ============================================================================
fn test_once_and_time() {
    // OnceCell (single-threaded)
    let once_cell: OnceCell<String> = OnceCell::new();
    let _ = once_cell.set(String::from("initialized"));
    let once_cell_filled: OnceCell<i32> = OnceCell::from(42);

    // OnceLock (thread-safe)
    let once_lock: OnceLock<String> = OnceLock::new();
    let _ = once_lock.set(String::from("thread-safe"));

    // Range types (all 6 variants from std::ops)
    let range: Range<i32> = 0..10;
    let range_inclusive: RangeInclusive<i32> = 0..=10;
    let range_to: RangeTo<i32> = ..10;
    let range_from: RangeFrom<i32> = 5..;
    let range_to_inclusive: RangeToInclusive<i32> = ..=10;
    let range_full: RangeFull = ..;
    let range_usize: Range<usize> = 0..100;

    // Duration and Instant
    let duration_new: Duration = Duration::new(60, 0);  // Duration::new pattern
    let duration: Duration = Duration::from_secs(60);
    let duration_millis: Duration = Duration::from_millis(500);
    let duration_micros: Duration = Duration::from_micros(1000);
    let duration_nanos: Duration = Duration::from_nanos(1_000_000);
    let instant: Instant = Instant::now();

    // Ordering (comparison result type)
    let ordering_less: Ordering = Ordering::Less;
    let ordering_equal: Ordering = Ordering::Equal;
    let ordering_greater: Ordering = Ordering::Greater;
    let ordering_cmp: Ordering = 1.cmp(&2);

    // Poll (async support type)
    let poll_ready: Poll<i32> = Poll::Ready(42);
    let poll_pending: Poll<i32> = Poll::Pending;

    println!(
        "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        once_cell, once_cell_filled, once_lock, range, range_inclusive,
        range_usize, duration_new, duration, duration_millis, duration_micros, instant,
        range_to, range_from, range_to_inclusive, range_full, ordering_less, poll_ready
    );
    let _ = (duration_nanos, ordering_equal, ordering_greater, ordering_cmp, poll_pending);
}

// ============================================================================
// Trait objects and boxed slices
// ============================================================================
trait Animal {
    fn speak(&self) -> &str;
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) -> &str { "woof" }
}

fn test_trait_objects() {
    // Trait objects (behind pointers - the pointer is Sized, the dyn Trait is !Sized)
    let dyn_animal: &dyn Animal = &Dog;
    let boxed_dyn: Box<dyn Animal> = Box::new(Dog);
    let arc_dyn: Arc<dyn Animal> = Arc::new(Dog);
    let rc_dyn: Rc<dyn Animal> = Rc::new(Dog);

    // Boxed slices (Box<[T]> is Sized, but [T] inside is !Sized)
    let boxed_slice: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let boxed_str: Box<str> = String::from("boxed str").into_boxed_str();

    // Vec to boxed array
    let boxed_array: Box<[i32; 3]> = Box::new([1, 2, 3]);

    println!(
        "{} {} {} {} {:?} {} {:?}",
        dyn_animal.speak(), boxed_dyn.speak(), arc_dyn.speak(), rc_dyn.speak(),
        boxed_slice, boxed_str, boxed_array
    );
}

// ============================================================================
// Unsized types (!Sized) - trait objects and slices
// ============================================================================
fn test_unsized_types() {
    // Slice references - the slice [T] is !Sized, but &[T] is Sized
    let slice_ref: &[i32] = &[1, 2, 3, 4, 5];
    let str_ref: &str = "hello unsized";
    let mut_slice: &mut [u8] = &mut [0u8; 10];
    
    // Nested slices
    let slice_of_slices: &[&[i32]] = &[&[1, 2], &[3, 4, 5]];
    
    // CStr is !Sized (it's a dynamically-sized type like str)
    let c_str_ref: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"hello\0") };
    
    // OsStr is !Sized
    let os_str_ref: &OsStr = OsStr::new("os string");
    
    // Path is !Sized
    let path_ref: &std::path::Path = std::path::Path::new("/tmp/test");
    
    println!(
        "{:?} {} {:?} {:?} {:?} {:?} {:?}",
        slice_ref, str_ref, mut_slice, slice_of_slices, c_str_ref, os_str_ref, path_ref
    );
}

// ============================================================================
// Atomic types
// ============================================================================
fn test_atomics() {
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicIsize, AtomicUsize, AtomicPtr};

    let atomic_bool: AtomicBool = AtomicBool::new(true);
    let atomic_i32: AtomicI32 = AtomicI32::new(42);
    let atomic_i64: AtomicI64 = AtomicI64::new(100);
    let atomic_isize: AtomicIsize = AtomicIsize::new(-1);
    let atomic_usize: AtomicUsize = AtomicUsize::new(0);

    let mut value = 42i32;
    let atomic_ptr: AtomicPtr<i32> = AtomicPtr::new(&mut value);

    println!(
        "{:?} {:?} {:?} {:?} {:?} {:?}",
        atomic_bool, atomic_i32, atomic_i64, atomic_isize, atomic_usize, atomic_ptr
    );
}

// ============================================================================
// Channel types
// ============================================================================
fn test_channels() {
    use std::sync::mpsc::{channel, sync_channel, Sender, Receiver, SyncSender};

    let (tx, rx): (Sender<i32>, Receiver<i32>) = channel();
    let (sync_tx, sync_rx): (SyncSender<String>, Receiver<String>) = sync_channel(10);

    println!("{:?} {:?} {:?} {:?}", tx, rx, sync_tx, sync_rx);
}

// ============================================================================
// IO types
// ============================================================================
fn test_io_types() {
    use std::io::{BufReader, BufWriter, Cursor, Empty, Repeat, Sink};

    let cursor: Cursor<Vec<u8>> = Cursor::new(vec![1, 2, 3]);
    let cursor_str: Cursor<&str> = Cursor::new("hello");

    let empty: Empty = std::io::empty();
    let repeat: Repeat = std::io::repeat(0u8);
    let sink: Sink = std::io::sink();

    // BufReader/BufWriter with Cursor (no actual file needed)
    let buf_reader: BufReader<Cursor<Vec<u8>>> = BufReader::new(Cursor::new(vec![]));
    let buf_writer: BufWriter<Cursor<Vec<u8>>> = BufWriter::new(Cursor::new(vec![]));

    println!(
        "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        cursor, cursor_str, empty, repeat, sink, buf_reader, buf_writer
    );
}

// ============================================================================
// Impl Trait types (opaque types)
// ============================================================================
fn returns_impl_iterator() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}

fn returns_impl_fn() -> impl Fn(i32) -> i32 {
    |x| x * 2
}

fn accepts_impl_trait(iter: impl Iterator<Item = i32>) -> i32 {
    iter.sum()
}

fn test_impl_trait() {
    // impl Trait in return position - type is opaque
    let iter = returns_impl_iterator();
    let func = returns_impl_fn();
    
    // impl Trait in argument position
    let sum = accepts_impl_trait(vec![1, 2, 3].into_iter());
    
    let iter_sum: i32 = iter.sum();
    let result = func(21);
    
    println!("{} {} {}", sum, iter_sum, result);
}

// ============================================================================
// Never type (!) - type with no values
// ============================================================================
fn test_never_type() {
    // Never type appears in diverging expressions
    // We can't create a value of type !, but we can use it in type annotations
    
    // Result<T, !> means infallible - can never be Err
    use std::convert::Infallible;
    let infallible_ok: Result<i32, Infallible> = Ok(42);
    
    // Unwrap is safe because Err variant is uninhabited
    let value = match infallible_ok {
        Ok(v) => v,
        Err(e) => match e {}, // e is Infallible, which is equivalent to !
    };
    
    println!("{}", value);
}

// ============================================================================
// Panic support types (PanicInfo, Location)
// ============================================================================
fn test_panic_support() {
    use std::panic::Location;
    
    // Location::caller() returns &'static Location<'static>
    let location: &Location = Location::caller();
    let file: &str = location.file();
    let line: u32 = location.line();
    let column: u32 = location.column();
    
    println!("Called from {}:{}:{}", file, line, column);
    
    // PanicInfo is only available in panic hooks, but we can set one up
    // to demonstrate the type exists
    use std::panic;
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(|panic_info| {
        // panic_info: &PanicInfo
        if let Some(location) = panic_info.location() {
            let _file = location.file();
            let _line = location.line();
        }
        if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
            let _ = msg;
        }
    }));
    // Restore default hook without triggering panic
    panic::set_hook(default_hook);
}

// ============================================================================
// METHOD CALL TRACKING TESTS (Semantic Operation Classification)
// ============================================================================

// Type alias for testing semantic resolution through aliases
type MyCell<T> = Cell<T>;
type MyCow<'a, T: ?Sized + ToOwned> = Cow<'a, T>;
type MyOption<T> = Option<T>;
type MyResult<T, E> = Result<T, E>;

fn test_method_calls_cell() {
    // Direct Cell
    let cell = Cell::new(42);
    cell.set(100);  // cell_set
    let _ = cell.get();  // cell_get
    
    // Cell via type alias - should still resolve to cell_* operations
    let my_cell: MyCell<i32> = MyCell::new(42);
    my_cell.set(200);  // Should be cell_set (semantic)
    let _ = my_cell.get();  // Should be cell_get (semantic)
}

fn test_method_calls_cow() {
    // Direct Cow
    let mut cow: Cow<str> = Cow::Borrowed("hello");
    let _ = cow.to_mut();  // cow_to_mut
    
    // Cow via type alias
    let mut my_cow: MyCow<str> = MyCow::Borrowed("world");
    let _ = my_cow.to_mut();  // Should be cow_to_mut (semantic)
}

fn test_method_calls_once_cell() {
    let cell: OnceCell<i32> = OnceCell::new();
    let _ = cell.set(42);  // once_cell_set
    let _ = cell.get();  // once_cell_get
    let _ = cell.get_or_init(|| 42);  // once_cell_get_or_init
}

fn test_method_calls_channels() {
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(42);  // channel_send
    let _ = rx.recv();  // channel_recv
    let _ = rx.try_recv();  // channel_try_recv
}

fn test_method_calls_thread_join() {
    use std::thread;
    let handle = thread::spawn(|| 42);
    let _ = handle.join();  // thread_join (consuming)
}

fn test_method_calls_smart_pointers() {
    // Rc
    let rc = Rc::new(42);
    let _ = rc.clone();  // rc_clone
    let weak = Rc::downgrade(&rc);
    let _ = weak.upgrade();  // weak_upgrade
    
    // Arc
    let arc = Arc::new(42);
    let _ = arc.clone();  // arc_clone
    let arc_weak = Arc::downgrade(&arc);
    let _ = arc_weak.upgrade();  // weak_upgrade
    
    // Rc via type alias
    let my_rc: MyRc<i32> = MyRc::new(42);
    let _ = my_rc.clone();  // Should be rc_clone (semantic)
    
    // Arc via type alias
    let my_arc: MyArc<i32> = MyArc::new(42);
    let _ = my_arc.clone();  // Should be arc_clone (semantic)
}

fn test_method_calls_refcell() {
    let refcell = RefCell::new(42);
    let _ = refcell.borrow();  // refcell_borrow
    let _ = refcell.borrow_mut();  // refcell_borrow_mut
}

fn test_method_calls_mutex_rwlock() {
    let mutex = Mutex::new(42);
    let _guard1 = mutex.lock();  // mutex_lock
    drop(_guard1);
    let _guard2 = mutex.try_lock();  // mutex_try_lock
    drop(_guard2);
    
    let rwlock = RwLock::new(42);
    let _guard3 = rwlock.read();  // rwlock_read
    drop(_guard3);
    let _guard4 = rwlock.write();  // rwlock_write
    drop(_guard4);
}

fn test_method_calls_option_result() {
    // Direct Option
    let opt: Option<i32> = Some(42);
    let _ = opt.unwrap();  // option_unwrap
    
    let opt2: Option<i32> = Some(42);
    let _ = opt2.expect("msg");  // option_expect
    
    let opt3: Option<i32> = None;
    let _ = opt3.unwrap_or(0);  // option_unwrap_or
    
    // Option via type alias
    let my_opt: MyOption<i32> = Some(42);
    let _ = my_opt.unwrap();  // Should be option_unwrap (semantic)
    
    // Direct Result
    let res: Result<i32, &str> = Ok(42);
    let _ = res.unwrap();  // result_unwrap
    
    let res2: Result<i32, &str> = Ok(42);
    let _ = res2.expect("msg");  // result_expect
    
    // Result via type alias
    let my_res: MyResult<i32, &str> = Ok(42);
    let _ = my_res.unwrap();  // Should be result_unwrap (semantic)
}

fn test_method_calls_clone() {
    let s = String::from("hello");
    let _ = s.clone();  // clone
    
    let v = vec![1, 2, 3];
    let _ = v.clone();  // clone
}

fn test_standalone_expressions() {
    let x = Box::new(42);
    let y = Box::new(43);
    drop(x);  // drop, argument: "x"
    std::mem::forget(y);  // forget, argument: "y"
    
    // Closure with no captures
    let _ = std::thread::spawn(|| 42);  // thread_spawn, argument: "<closure>"
    
    // Closure with captures
    let data = vec![1, 2, 3];
    let multiplier = 2;
    let _ = std::thread::spawn(move || {
        // Captures: data, multiplier
        data.iter().map(|x| x * multiplier).sum::<i32>()
    });  // thread_spawn, argument: "<closure captures: data, multiplier>"
    
    // mem::replace, swap, take
    let mut a = String::from("hello");
    let old = std::mem::replace(&mut a, String::from("world"));
    let _ = old;
    
    let mut b = String::from("foo");
    let mut c = String::from("bar");
    std::mem::swap(&mut b, &mut c);
    
    let mut d = Some(42);
    let taken = std::mem::take(&mut d);
    let _ = taken;
    
    // transmute (unsafe)
    let num: u32 = 42;
    let _bytes: [u8; 4] = unsafe { std::mem::transmute(num) };
    
    // transmute_copy
    let src: u32 = 0x12345678;
    let _dst: [u8; 4] = unsafe { std::mem::transmute_copy(&src) };
    
    // ptr operations (unsafe)
    let mut val = 100i32;
    let ptr = &mut val as *mut i32;
    unsafe {
        let read_val = std::ptr::read(ptr);
        std::ptr::write(ptr, read_val + 1);
        
        // volatile operations
        let _volatile_read = std::ptr::read_volatile(ptr);
        std::ptr::write_volatile(ptr, 200);
        
        // copy operations
        let mut dest = 0i32;
        let dest_ptr = &mut dest as *mut i32;
        std::ptr::copy(ptr, dest_ptr, 1);
        std::ptr::copy_nonoverlapping(ptr, dest_ptr, 1);
    }
}

fn main() {
    test_type_aliases();
    test_binding_patterns();
    test_lifetime_types();
    test_unions();
    test_ffi_types();
    test_primitives();
    test_smart_pointers();
    test_interior_mutability();
    test_collections();
    test_references();
    test_raw_pointers();
    test_fn_pointers();
    test_guards();
    test_futures_and_iterators();
    test_nested_types();
    test_user_types();
    test_path();
    test_advanced_types();
    test_once_and_time();
    test_trait_objects();
    test_unsized_types();
    test_atomics();
    test_channels();
    test_io_types();
    test_impl_trait();
    test_never_type();
    test_panic_support();
    test_macro_initializers();
    
    // Method call tracking tests
    test_method_calls_cell();
    test_method_calls_cow();
    test_method_calls_once_cell();
    test_method_calls_channels();
    test_method_calls_thread_join();
    test_method_calls_smart_pointers();
    test_method_calls_refcell();
    test_method_calls_mutex_rwlock();
    test_method_calls_option_result();
    test_method_calls_clone();
    test_standalone_expressions();
    test_closure_capture_modes();
    test_trait_vs_inherent_methods();
    test_unsafe_operations();
}

// ============================================================================
// Closure capture modes (semantic via rust-analyzer CaptureKind)
// ============================================================================
fn test_closure_capture_modes() {
    // Capture by shared reference (read-only access)
    let shared_data = vec![1, 2, 3];
    let closure_shared = || {
        println!("{:?}", shared_data); // shared_ref capture
    };
    closure_shared();
    
    // Capture by mutable reference (mutation)
    let mut mutable_data = String::from("hello");
    let mut closure_mut = || {
        mutable_data.push_str(" world"); // mutable_ref capture
    };
    closure_mut();
    
    // Capture by move (ownership transfer)
    let owned_data = String::from("moved");
    let closure_move = move || {
        drop(owned_data); // move capture
    };
    closure_move();
    
    // Mixed captures in one closure
    let read_only = 42;
    let mut mutated = vec![1];
    let moved = Box::new(100);
    let mixed_closure = move || {
        let _ = read_only;      // move (Copy type, but move closure)
        mutated.push(2);        // move (move closure)
        drop(moved);            // move
    };
    mixed_closure();
    
    // Spawn with captures (tests semantic capture extraction)
    let spawn_data = std::sync::Arc::new(std::sync::Mutex::new(0));
    let spawn_data_clone = spawn_data.clone();
    let handle = std::thread::spawn(move || {
        let mut guard = spawn_data_clone.lock().unwrap();
        *guard += 1;
    });
    let _ = handle.join();
    
    println!("{}", mutable_data);
}

// ============================================================================
// Trait vs inherent method resolution (semantic via ItemContainer)
// ============================================================================
fn test_trait_vs_inherent_methods() {
    // Clone trait method
    let s = String::from("hello");
    let s_cloned = s.clone(); // Clone::clone - trait method
    
    // Iterator trait methods
    let v = vec![1, 2, 3];
    let mapped: Vec<_> = v.iter().map(|x| x * 2).collect(); // Iterator::map, Iterator::collect
    
    // Display trait (via to_string from ToString which blanket impls Display)
    let num = 42;
    let num_str = num.to_string(); // ToString::to_string - trait method
    
    // Inherent methods (not from traits)
    let mut vec = Vec::new();
    vec.push(1);        // Vec::push - inherent method
    vec.push(2);
    let len = vec.len(); // Vec::len - inherent method
    
    // String inherent methods
    let mut string = String::new();
    string.push_str("hello"); // String::push_str - inherent method
    
    // Option inherent methods
    let opt = Some(42);
    let unwrapped = opt.unwrap(); // Option::unwrap - inherent method
    
    // Deref trait (implicit)
    let boxed = Box::new(String::from("boxed"));
    let box_len = boxed.len(); // Deref to String, then String::len
    
    println!("{} {:?} {} {} {} {}", s_cloned, mapped, num_str, len, string, unwrapped);
    println!("{}", box_len);
}

// ============================================================================
// Unsafe operations tracking (semantic via is_unsafe_to_call)
// ============================================================================
fn test_unsafe_operations() {
    // Unsafe function calls (tracked in expressions)
    let mut value = 42i32;
    let ptr = &mut value as *mut i32;
    
    unsafe {
        // ptr::read is unsafe
        let read_val = std::ptr::read(ptr);
        
        // ptr::write is unsafe
        std::ptr::write(ptr, 100);
        
        // transmute is unsafe
        let bytes: [u8; 4] = std::mem::transmute(read_val);
        
        println!("{:?}", bytes);
    }
    
    // Unsafe method calls
    let mut uninit: std::mem::MaybeUninit<i32> = std::mem::MaybeUninit::uninit();
    unsafe {
        uninit.as_mut_ptr().write(42);
        let initialized = uninit.assume_init(); // unsafe method
        println!("{}", initialized);
    }
    
    // Safe wrappers around unsafe
    let v = vec![1, 2, 3];
    let first = v.get(0); // safe - returns Option
    let first_unchecked = unsafe { v.get_unchecked(0) }; // unsafe method
    
    println!("{:?} {}", first, first_unchecked);
}

// ============================================================================
// Macro initializers (semantic macro resolution)
// ============================================================================
fn test_macro_initializers() {
    // vec! macro
    let vec_init = vec![1, 2, 3];
    
    // format! macro
    let format_init = format!("hello {}", "world");
    
    // String formatting macros as initializers
    let formatted = format!("{:?}", vec_init);
    
    // concat! macro (compile-time)
    let concat_init = concat!("hello", " ", "world");
    
    // env! macro
    let env_init = env!("CARGO_PKG_NAME");
    
    // include_str! / include_bytes! would need actual files
    
    // stringify! macro
    let stringify_init = stringify!(some_identifier);
    
    // line!/column!/file! macros
    let line_init = line!();
    let column_init = column!();
    let file_init = file!();
    
    // module_path! macro
    let module_init = module_path!();
    
    // option_env! macro
    let option_env_init = option_env!("NONEXISTENT_VAR");
    
    println!("{:?} {} {} {} {} {} {} {} {} {:?}", 
        vec_init, format_init, formatted, concat_init, env_init, 
        stringify_init, line_init, column_init, file_init, option_env_init);
    println!("{}", module_init);
}

// ============================================================================
// Unsafe operations - comprehensive test cases
// ============================================================================

// External FFI function declaration
extern "C" {
    fn abs(x: i32) -> i32;
}

fn test_unsafe_operations() {
    // 1. Raw pointer dereference
    let value = 42i32;
    let ptr: *const i32 = &value;
    let mut_value = 100i32;
    let mut_ptr: *mut i32 = &mut mut_value as *mut i32;
    
    unsafe {
        // Dereference raw pointers (inside unsafe block)
        let deref_const = *ptr;
        let deref_mut = *mut_ptr;
        *mut_ptr = 200;
        println!("Dereferenced: {} {}", deref_const, deref_mut);
    }
    
    // 2. FFI call
    unsafe {
        let abs_result = abs(-42);
        println!("abs(-42) = {}", abs_result);
    }
    
    // 3. Mutable static access
    unsafe {
        STATIC_MUT = 42;
        let static_val = STATIC_MUT;
        println!("Mutable static: {}", static_val);
    }
    
    // 4. Unsafe method calls (e.g., get_unchecked)
    let vec = vec![1, 2, 3, 4, 5];
    unsafe {
        let unchecked = vec.get_unchecked(2);
        println!("Unchecked access: {}", unchecked);
    }
    
    // 5. Transmute (unsafe function)
    unsafe {
        let bytes: [u8; 4] = std::mem::transmute(42i32);
        println!("Transmuted bytes: {:?}", bytes);
    }
}

// Test unsafe outside of unsafe block (would be compile error, but we track it)
fn test_unsafe_detection() {
    let x = 42i32;
    let ptr: *const i32 = &x;
    
    // This would be a compile error without unsafe, but we want to detect it
    // Commenting out to allow compilation, but the analyzer should detect raw ptr creation
    // let _deref = *ptr; // ERROR: requires unsafe
    
    println!("Raw pointer created: {:?}", ptr);
}


// ============================================================================
// Drop point test cases - variables in different scopes
// ============================================================================
fn test_drop_points() {
    // Variable in function scope - drops at end of function
    let func_scope_var = String::from("function scope");
    
    // Variable in block scope - drops at end of block
    {
        let block_scope_var = String::from("block scope");
        println!("{}", block_scope_var);
    } // block_scope_var drops here
    
    // Variable in if block
    if true {
        let if_scope_var = String::from("if scope");
        println!("{}", if_scope_var);
    } // if_scope_var drops here
    
    // Variable in loop
    for i in 0..1 {
        let loop_scope_var = String::from("loop scope");
        println!("{} {}", i, loop_scope_var);
    } // loop_scope_var drops here each iteration
    
    // Variable in while loop
    let mut counter = 0;
    while counter < 1 {
        let while_scope_var = String::from("while scope");
        println!("{}", while_scope_var);
        counter += 1;
    } // while_scope_var drops here each iteration
    
    // Nested scopes
    {
        let outer_var = String::from("outer");
        {
            let inner_var = String::from("inner");
            println!("{} {}", outer_var, inner_var);
        } // inner_var drops here
        println!("{}", outer_var);
    } // outer_var drops here
    
    println!("{}", func_scope_var);
} // func_scope_var drops here


// ============================================================================
// Variable usage test cases - reads and writes
// ============================================================================
fn test_variable_usages() {
    // Simple read
    let read_only = 42;
    println!("{}", read_only); // read
    
    // Multiple reads
    let multi_read = String::from("hello");
    println!("{}", multi_read); // read 1
    println!("{}", multi_read); // read 2
    println!("{}", multi_read); // read 3
    
    // Write then read
    let mut write_then_read = 0;
    write_then_read = 42; // write
    println!("{}", write_then_read); // read
    
    // Multiple writes
    let mut multi_write = 0;
    multi_write = 1; // write 1
    multi_write = 2; // write 2
    multi_write = 3; // write 3
    println!("{}", multi_write);
    
    // Read-modify-write (compound assignment)
    let mut compound = 10;
    compound += 5; // read + write
    compound *= 2; // read + write
    println!("{}", compound);
    
    // Method call that mutates
    let mut vec_usage = vec![1, 2, 3];
    vec_usage.push(4); // write via method
    let _len = vec_usage.len(); // read via method
    println!("{:?}", vec_usage);
}


// ============================================================================
// Borrow Span Tests
// ============================================================================

fn test_borrow_spans() {
    // Borrow with multiple uses - end should be at last use
    let data = vec![1, 2, 3, 4, 5];
    let borrow_multi = &data;
    println!("{:?}", borrow_multi);  // use 1
    println!("{}", borrow_multi.len());  // use 2
    let _first = borrow_multi.first();  // use 3 - this should be the end
    
    // Mutable borrow with multiple uses
    let mut mutable_data = String::from("hello");
    let borrow_mut_multi = &mut mutable_data;
    borrow_mut_multi.push_str(" world");  // use 1
    borrow_mut_multi.push('!');  // use 2 - this should be the end
    
    // Nested borrows
    let outer = vec![1, 2, 3];
    let outer_ref = &outer;
    let _inner = &outer_ref[0];  // borrow of borrow
    println!("{:?}", outer_ref);  // outer_ref ends here
}


// ============================================================================
// Macro Classification Tests (Semantic via MacroId)
// ============================================================================

fn test_macro_classification() {
    // Collection macros
    let vec_test = vec![1, 2, 3];
    
    // Format macros
    let formatted = format!("hello {}", "world");
    
    // Concat macro
    let concatenated = concat!("hello", " ", "world");
    
    // Stringify macro
    let stringified = stringify!(some_identifier);
    
    // Env macro (compile-time)
    let cargo_pkg = env!("CARGO_PKG_NAME");
    
    // Line/column/file macros
    let current_line = line!();
    let current_column = column!();
    let current_file = file!();
    let current_module = module_path!();
    
    println!("{:?} {} {} {} {} {} {} {}", vec_test, formatted, concatenated, stringified, cargo_pkg, current_line, current_column, current_file);
    println!("{}", current_module);
}
