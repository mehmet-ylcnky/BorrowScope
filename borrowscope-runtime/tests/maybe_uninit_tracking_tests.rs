//! Comprehensive tests for MaybeUninit tracking

use borrowscope_runtime::*;
use std::mem::MaybeUninit;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// MaybeUninit Creation Tests
// =============================================================================

#[test]
fn test_maybe_uninit_uninit() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let _uninit: MaybeUninit<i32> =
        track_maybe_uninit_uninit("data", "test:1", MaybeUninit::uninit());

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

#[test]
fn test_maybe_uninit_new() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let init: MaybeUninit<i32> = track_maybe_uninit_new("data", "test:1", MaybeUninit::new(42));

    // Safe to assume_init since we used new()
    let value = unsafe { init.assume_init() };
    assert_eq!(value, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

// =============================================================================
// MaybeUninit Write Tests
// =============================================================================

#[test]
fn test_maybe_uninit_write() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    let written = track_maybe_uninit_write("data", "test:1", uninit.write(42));

    assert_eq!(*written, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

#[test]
fn test_maybe_uninit_write_complex_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mut uninit: MaybeUninit<String> = MaybeUninit::uninit();
    let written = track_maybe_uninit_write("data", "test:1", uninit.write(String::from("hello")));

    assert_eq!(written, "hello");

    // Clean up
    unsafe { std::ptr::drop_in_place(written) };

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// MaybeUninit assume_init Tests
// =============================================================================

#[test]
fn test_maybe_uninit_assume_init() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let init: MaybeUninit<i32> = MaybeUninit::new(42);
    let value = track_maybe_uninit_assume_init("data", "test:1", unsafe { init.assume_init() });

    assert_eq!(value, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

#[test]
fn test_maybe_uninit_assume_init_read() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let init: MaybeUninit<i32> = MaybeUninit::new(42);
    let value =
        track_maybe_uninit_assume_init_read("data", "test:1", unsafe { init.assume_init_read() });

    assert_eq!(value, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

#[test]
fn test_maybe_uninit_assume_init_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mut init: MaybeUninit<String> = MaybeUninit::new(String::from("hello"));

    unsafe { init.assume_init_drop() };
    track_maybe_uninit_assume_init_drop("data", "test:1");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_maybe_uninit());
}

// =============================================================================
// MaybeUninit Array Tests
// =============================================================================

#[test]
fn test_maybe_uninit_array() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create array of uninitialized values
    let mut arr: [MaybeUninit<i32>; 3] = unsafe { MaybeUninit::uninit().assume_init() };

    // Initialize each element
    for (i, elem) in arr.iter_mut().enumerate() {
        let _ = track_maybe_uninit_write(&format!("arr[{}]", i), "test:1", elem.write(i as i32));
    }

    // Read values
    for (i, elem) in arr.iter().enumerate() {
        let value = unsafe { elem.assume_init_read() };
        assert_eq!(value, i as i32);
    }

    let events = get_events();
    assert_eq!(events.len(), 3);
}

// =============================================================================
// Lifecycle Tests
// =============================================================================

#[test]
fn test_maybe_uninit_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create uninitialized
    let mut uninit: MaybeUninit<i32> =
        track_maybe_uninit_uninit("data", "test:1", MaybeUninit::uninit());

    // Write value
    let _ = track_maybe_uninit_write("data", "test:2", uninit.write(42));

    // Assume init and use
    let value = track_maybe_uninit_assume_init("data", "test:3", unsafe { uninit.assume_init() });
    assert_eq!(value, 42);

    let events = get_events();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|e| e.is_maybe_uninit()));
}

#[test]
fn test_maybe_uninit_with_drop_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    // Create with value
    let mut init: MaybeUninit<Vec<i32>> =
        track_maybe_uninit_new("vec", "test:1", MaybeUninit::new(vec![1, 2, 3]));

    // Drop properly
    unsafe { init.assume_init_drop() };
    track_maybe_uninit_assume_init_drop("vec", "test:2");

    let events = get_events();
    assert_eq!(events.len(), 2);
}
