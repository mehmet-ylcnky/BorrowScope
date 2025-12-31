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
    // Tuple bindings (ERR-002)
    let (tuple_a, tuple_b) = (1, 2);
    let (tuple_x, tuple_y, tuple_z) = ("a", "b", "c");

    // Mutable bindings (ERR-003)
    let mut mut_int = 42;
    let mut mut_vec = vec![1, 2, 3];
    let mut mut_string = String::new();

    // Nested tuple
    let ((nested_a, nested_b), nested_c) = ((1, 2), 3);

    // Use them to avoid warnings
    mut_int += 1;
    mut_vec.push(4);
    mut_string.push_str("hello");
    println!("{} {} {} {} {} {} {} {} {}", tuple_a, tuple_b, tuple_x, tuple_y, tuple_z, mut_int, mut_vec.len(), mut_string, nested_a + nested_b + nested_c);
}

fn test_lifetime_types() {
    // Explicit lifetime in type (ERR-013)
    let static_str: &'static str = "static";
    
    // References with inferred lifetimes
    let local = String::from("local");
    let local_ref: &str = &local;

    println!("{} {}", static_str, local_ref);
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

fn test_futures_and_iterators() {
    // Future
    let future = async_fn();
    let future_block = async { 42 };

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
}
