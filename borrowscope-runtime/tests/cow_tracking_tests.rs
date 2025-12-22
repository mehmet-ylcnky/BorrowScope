//! Comprehensive tests for Cow (Clone-on-Write) tracking
//!
//! Tests cover:
//! - Cow::Borrowed creation
//! - Cow::Owned creation
//! - Cow::to_mut (clone-on-write trigger)
//! - Borrowed vs Owned state transitions
//! - Various data types (str, [T], custom)

use borrowscope_runtime::*;
use std::borrow::Cow;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Cow::Borrowed Tests
// =============================================================================

#[test]
fn test_cow_borrowed_str() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let s = "hello world";
    let cow: Cow<str> = track_cow_borrowed("cow", "test:1", Cow::Borrowed(s));

    assert!(matches!(cow, Cow::Borrowed(_)));
    assert_eq!(&*cow, "hello world");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_cow());
}

#[test]
fn test_cow_borrowed_slice() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arr = [1, 2, 3, 4, 5];
    let cow: Cow<[i32]> = track_cow_borrowed("cow", "test:1", Cow::Borrowed(&arr));

    assert!(matches!(cow, Cow::Borrowed(_)));
    assert_eq!(&*cow, &[1, 2, 3, 4, 5]);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_cow());
}

#[test]
fn test_cow_borrowed_static_str() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    static STATIC_STR: &str = "static string";
    let cow: Cow<str> = track_cow_borrowed("cow", "test:1", Cow::Borrowed(STATIC_STR));

    assert_eq!(&*cow, "static string");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_cow_borrowed_empty() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cow: Cow<str> = track_cow_borrowed("cow", "test:1", Cow::Borrowed(""));
    assert!(cow.is_empty());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Cow::Owned Tests
// =============================================================================

#[test]
fn test_cow_owned_string() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cow: Cow<str> = track_cow_owned("cow", "test:1", Cow::Owned(String::from("owned string")));

    assert!(matches!(cow, Cow::Owned(_)));
    assert_eq!(&*cow, "owned string");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_cow());
}

#[test]
fn test_cow_owned_vec() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let cow: Cow<[i32]> = track_cow_owned("cow", "test:1", Cow::Owned(vec![10, 20, 30]));

    assert!(matches!(cow, Cow::Owned(_)));
    assert_eq!(&*cow, &[10, 20, 30]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_cow_owned_from_computation() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let computed = (0..5).map(|x| x * 2).collect::<Vec<_>>();
    let cow: Cow<[i32]> = track_cow_owned("cow", "test:1", Cow::Owned(computed));

    assert_eq!(&*cow, &[0, 2, 4, 6, 8]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Cow::to_mut Tests (Clone-on-Write)
// =============================================================================

#[test]
fn test_cow_to_mut_triggers_clone() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let s = "original";
    let mut cow: Cow<str> = Cow::Borrowed(s);

    // Before to_mut, it's borrowed
    assert!(matches!(cow, Cow::Borrowed(_)));

    // to_mut triggers clone
    track_cow_to_mut("cow", true, "test:1");
    let mutref = cow.to_mut();
    mutref.push_str(" modified");

    // After to_mut, it's owned
    assert!(matches!(cow, Cow::Owned(_)));
    assert_eq!(&*cow, "original modified");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_cow());
}

#[test]
fn test_cow_to_mut_no_clone_when_owned() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let mut cow: Cow<str> = Cow::Owned(String::from("already owned"));

    // to_mut on owned doesn't clone
    track_cow_to_mut("cow", false, "test:1");
    let mutref = cow.to_mut();
    mutref.push_str("!");

    assert_eq!(&*cow, "already owned!");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_cow_to_mut_slice() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let arr = [1, 2, 3];
    let mut cow: Cow<[i32]> = Cow::Borrowed(&arr);

    track_cow_to_mut("cow", true, "test:1");
    let mutref = cow.to_mut();
    mutref[0] = 100;

    assert_eq!(&*cow, &[100, 2, 3]);
    // Original unchanged
    assert_eq!(arr, [1, 2, 3]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Lifecycle and Transition Tests
// =============================================================================

#[test]
fn test_cow_borrowed_to_owned_transition() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let original = "start";
    let cow: Cow<str> = track_cow_borrowed("cow", "test:1", Cow::Borrowed(original));

    // Convert to owned
    let owned_cow: Cow<str> = track_cow_owned("cow_owned", "test:2", Cow::Owned(cow.into_owned()));

    assert!(matches!(owned_cow, Cow::Owned(_)));

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_cow_multiple_borrows() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let data = "shared";
    let cow1: Cow<str> = track_cow_borrowed("cow1", "test:1", Cow::Borrowed(data));
    let cow2: Cow<str> = track_cow_borrowed("cow2", "test:2", Cow::Borrowed(data));
    let cow3: Cow<str> = track_cow_borrowed("cow3", "test:3", Cow::Borrowed(data));

    assert_eq!(&*cow1, &*cow2);
    assert_eq!(&*cow2, &*cow3);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_cow_conditional_mutation() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    fn maybe_modify(mut cow: Cow<str>, should_modify: bool) -> Cow<str> {
        if should_modify {
            track_cow_to_mut("cow", matches!(cow, Cow::Borrowed(_)), "test:modify");
            cow.to_mut().push_str(" modified");
        }
        cow
    }

    let borrowed: Cow<str> = Cow::Borrowed("test");
    let result = maybe_modify(borrowed, true);
    assert_eq!(&*result, "test modified");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Real-World Patterns
// =============================================================================

#[test]
fn test_cow_string_processing() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    fn process_string(input: &str) -> Cow<str> {
        if input.contains("bad") {
            let cleaned = input.replace("bad", "good");
            track_cow_owned("result", "process:owned", Cow::Owned(cleaned))
        } else {
            track_cow_borrowed("result", "process:borrowed", Cow::Borrowed(input))
        }
    }

    let clean = process_string("hello world");
    assert!(matches!(clean, Cow::Borrowed(_)));

    reset();

    let dirty = process_string("hello bad world");
    assert!(matches!(dirty, Cow::Owned(_)));
    assert_eq!(&*dirty, "hello good world");
}

#[test]
fn test_cow_path_normalization() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    fn normalize_path(path: &str) -> Cow<str> {
        if path.contains("//") {
            let normalized = path.replace("//", "/");
            track_cow_owned("path", "normalize:owned", Cow::Owned(normalized))
        } else {
            track_cow_borrowed("path", "normalize:borrowed", Cow::Borrowed(path))
        }
    }

    let normal = normalize_path("/home/user/file");
    assert!(matches!(normal, Cow::Borrowed(_)));

    reset();

    let abnormal = normalize_path("/home//user//file");
    assert!(matches!(abnormal, Cow::Owned(_)));
    assert_eq!(&*abnormal, "/home/user/file");
}

#[test]
fn test_cow_in_collection() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let items: Vec<Cow<str>> = vec![
        track_cow_borrowed("item0", "test:1", Cow::Borrowed("static")),
        track_cow_owned("item1", "test:2", Cow::Owned(String::from("dynamic"))),
        track_cow_borrowed("item2", "test:3", Cow::Borrowed("another static")),
    ];

    assert_eq!(items.len(), 3);
    assert!(matches!(items[0], Cow::Borrowed(_)));
    assert!(matches!(items[1], Cow::Owned(_)));
    assert!(matches!(items[2], Cow::Borrowed(_)));

    let events = get_events();
    assert_eq!(events.len(), 3);
}
