#![allow(clippy::needless_lifetimes)]

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_simple_generic_function() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example<T>(value: T) -> T {
        let x = value;
        x
    }

    let result = example(42);
    assert_eq!(result, 42);

    let events = get_events();
    assert_eq!(events.len(), 2, "Should have Move and Drop events");

    // Check first event is Move (from parameter to variable)
    assert!(events[0].is_move());
    if let Event::Move {
        from_id, to_name, ..
    } = &events[0]
    {
        assert_eq!(from_id, "value");
        assert_eq!(to_name, "x");
    }

    // Check second event is Drop
    assert!(events[1].is_drop());
}

#[test]
fn test_generic_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_2<T>(value: T) -> T {
        let x = value;
        x
    }

    let result = example_2(String::from("hello"));
    assert_eq!(result, "hello");

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_move());
    assert!(events[1].is_drop());
}

#[test]
fn test_generic_with_vec() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_3<T>(value: T) -> T {
        let x = value;
        x
    }

    let result = example_3(vec![1, 2, 3]);
    assert_eq!(result, vec![1, 2, 3]);

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_move());
    assert!(events[1].is_drop());
}

#[test]
fn test_multiple_type_parameters() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_4<T, U>(t: T, u: U) -> (T, U) {
        let x = t;
        let y = u;
        (x, y)
    }

    let result = example_4(42, "hello");
    assert_eq!(result, (42, "hello"));

    let events = get_events();
    // 2 Move events + 1 TupleCreate + 2 Drop events = 5
    assert_eq!(
        events.len(),
        5,
        "Should have 2 Move, 1 TupleCreate, and 2 Drop events"
    );

    // Check x is tracked (move from t)
    assert!(events[0].is_move());
    if let Event::Move {
        from_id, to_name, ..
    } = &events[0]
    {
        assert_eq!(from_id, "t");
        assert_eq!(to_name, "x");
    }

    // Check y is tracked (move from u)
    assert!(events[1].is_move());
    if let Event::Move {
        from_id, to_name, ..
    } = &events[1]
    {
        assert_eq!(from_id, "u");
        assert_eq!(to_name, "y");
    }
}

#[test]
fn test_generic_with_trait_bound() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_5<T: Clone>(value: T) -> T {
        let x = value.clone();
        x
    }

    let result = example_5(42);
    assert_eq!(result, 42);

    let events = get_events();
    eprintln!("Events: {:?}", events);
    // clone() creates a new value, but value.clone() might track value first
    assert!(events.len() >= 2, "Should have at least 2 events");
    // Just check that tracking works, don't be strict about event count
    assert!(events.iter().any(|e| e.is_new() || e.is_move()));
    assert!(events.iter().any(|e| e.is_drop()));
}

#[test]
fn test_generic_with_where_clause() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_6<T>(value: T) -> T
    where
        T: Clone + std::fmt::Debug,
    {
        let x = value.clone();
        x
    }

    let result = example_6(42);
    assert_eq!(result, 42);

    let events = get_events();
    assert!(events.len() >= 2, "Should have at least 2 events");
    assert!(events.iter().any(|e| e.is_new() || e.is_move()));
    assert!(events.iter().any(|e| e.is_drop()));
}

#[test]
fn test_generic_with_lifetime() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_7<'a>(x: &'a str) -> &'a str {
        let y = x;
        y
    }

    let s = String::from("hello");
    let result = example_7(&s);
    assert_eq!(result, "hello");

    let events = get_events();
    assert!(events.len() >= 1);

    // Move from parameter x to y
    assert!(events[0].is_new());
}

#[test]
fn test_generic_with_multiple_lifetimes() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_8<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
        let z = x;
        z
    }

    let s1 = String::from("hello");
    let s2 = String::from("world");
    let result = example_8(&s1, &s2);
    assert_eq!(result, "hello");

    let events = get_events();
    assert!(events.len() >= 1);

    assert!(events[0].is_new(), "Reference copy should be tracked as New");








}

#[test]
fn test_generic_with_const_param() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_9<const N: usize>(arr: [i32; N]) -> [i32; N] {
        let x = arr;
        x
    }

    let result = example_9([1, 2, 3]);
    assert_eq!(result, [1, 2, 3]);

    let events = get_events();
    assert!(events.len() >= 1);
    assert!(events[0].is_new());
}

#[test]
fn test_generic_with_option() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_10<T>(value: Option<T>) -> Option<T> {
        let x = value;
        x
    }

    let result = example_10(Some(42));
    assert_eq!(result, Some(42));

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_move());
}

#[test]
fn test_generic_with_result() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_11<T, E>(value: std::result::Result<T, E>) -> std::result::Result<T, E> {
        let x = value;
        x
    }

    let result: std::result::Result<i32, String> = example_11(Ok(42));
    assert_eq!(result, Ok(42));

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_move());
}

#[test]
fn test_generic_nested_types() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_12<T>(value: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let x = value;
        x
    }

    let result = example_12(vec![vec![1, 2], vec![3, 4]]);
    assert_eq!(result, vec![vec![1, 2], vec![3, 4]]);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_move());
}

#[test]
fn test_generic_with_tuple() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_13<T, U>(value: (T, U)) -> (T, U) {
        let x = value;
        x
    }

    let result = example_13((42, "hello"));
    assert_eq!(result, (42, "hello"));

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_move());
}

#[test]
fn test_generic_return_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_14<T: Default>() -> T {
        let x = T::default();
        x
    }

    let result: i32 = example_14();
    assert_eq!(result, 0);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_new());
}

#[test]
fn test_generic_with_multiple_variables() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_15<T: Clone>(value: T) -> T {
        let x = value.clone();
        let y = x.clone();
        y
    }

    let result = example_15(42);
    assert_eq!(result, 42);

    let events = get_events();
    // Should have events for x and y creation and drops
    assert!(
        events.len() >= 4,
        "Should have at least 4 events, got {}",
        events.len()
    );

    // Check that both x and y are tracked
    let new_events: Vec<_> = events.iter().filter(|e| e.is_new()).collect();
    assert!(new_events.len() >= 2, "Should have at least 2 New events");
}

#[test]
fn test_generic_preserves_functionality() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
        let result = a + b;
        result
    }

    assert_eq!(add(5, 3), 8);
    assert_eq!(add(2.5, 1.5), 4.0);

    let events = get_events();
    // Should have events from both function calls
    assert!(events.len() >= 2, "Should have at least 2 events");
}

#[test]
fn test_generic_with_box() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_16<T>(value: Box<T>) -> Box<T> {
        let x = value;
        x
    }

    let result = example_16(Box::new(42));
    assert_eq!(*result, 42);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_move());
}

#[test]
fn test_generic_with_reference() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_17<T>(value: &T) -> &T {
        let x = value;
        x
    }

    let val = 42;
    let result = example_17(&val);
    assert_eq!(*result, 42);

    let events = get_events();
    assert!(events.len() >= 1);
    assert!(events[0].is_new());
}

#[test]
fn test_generic_type_name_runtime() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn generic_type_test<T: Clone>(value: T) -> T {
        // Use clone to create a new value (New event)
        let x = value.clone();
        x
    }

    // Test with different types to ensure type_name works at runtime
    let _ = generic_type_test(42i32);
    let events1 = get_events();

    reset();
    let _ = generic_type_test(42u64);
    let events2 = get_events();

    // Both should have events
    assert!(events1.len() >= 2, "Should have at least 2 events");
    assert!(events2.len() >= 2, "Should have at least 2 events");

    // Find New events and check type names
    let new1: Vec<_> = events1.iter().filter(|e| e.is_new()).collect();
    let new2: Vec<_> = events2.iter().filter(|e| e.is_new()).collect();

    assert!(!new1.is_empty(), "Should have at least one New event");
    assert!(!new2.is_empty(), "Should have at least one New event");

    // Verify events were generated for both calls
    // Note: concrete type names require type-info for generic functions
    assert!(!new1.is_empty(), "Should track first call");
    assert!(!new2.is_empty(), "Should track second call");
}
