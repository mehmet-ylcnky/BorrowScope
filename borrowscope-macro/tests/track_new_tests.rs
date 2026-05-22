use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_simple_variable_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        assert_eq!(x, 42);
    }

    example_2();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
    assert!(events[1].is_drop());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_typed_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_2() {
        let x: i32 = 42;
        assert_eq!(x, 42);
    }

    example_3();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_string_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_3() {
        let s = String::from("hello");
        assert_eq!(s, "hello");
    }

    example_4();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_vec_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_4() {
        let v = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }

    example_5();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_multiple_variables() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_5() {
        let x = 42;
        let y = 100;
        let z = x + y;
        assert_eq!(z, 142);
    }

    example_6();

    let events = get_events();
    assert_eq!(events.len(), 6); // 3 New + 3 Drop
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_complex_expression() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_6() {
        let x = 1 + 2 * 3;
        assert_eq!(x, 7);
    }

    example_7();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

fn get_value_helper() -> i32 {
    42
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_function_call_initializer() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_7() {
        let x = get_value_helper();
        assert_eq!(x, 42);
    }

    example_8();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_mutable_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_8() {
        let mut x = 42;
        x += 1;
        assert_eq!(x, 43);
    }

    example_9();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_nested_blocks() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_9() {
        let x = 1;
        {
            let y = 2;
            assert_eq!(y, 2);
        }
        assert_eq!(x, 1);
    }

    example_10();

    let events = get_events();
    assert_eq!(events.len(), 4); // 2 New + 2 Drop
}

#[test]
    #[ignore = "requires borrowscope-analyzer pipeline"]
fn test_preserves_return_value() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example_10() -> i32 {
        let x = 42;
        x
    }

    let result = example_11();
    assert_eq!(result, 42);

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}
