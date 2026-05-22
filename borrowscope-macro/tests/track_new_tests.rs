use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_simple_variable_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_1() {
        let x = 42;
        assert_eq!(x, 42);
    }

    tn_ex_1();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
    assert!(events[0].is_new());
    // i32 is Copy - no Drop event emitted
}

#[test]
fn test_typed_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_2() {
        let x: i32 = 42;
        assert_eq!(x, 42);
    }

    tn_ex_2();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_string_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_3() {
        let s = String::from("hello");
        assert_eq!(s, "hello");
    }

    tn_ex_3();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_vec_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_4() {
        let v = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }

    tn_ex_4();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_multiple_variables() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_5() {
        let x = 42;
        let y = 100;
        let z = x + y;
        assert_eq!(z, 142);
    }

    tn_ex_5();

    let events = get_events();
    assert!(events.len() >= 3, "Should have at least 3 events"); // 3 New + 3 Drop
}

#[test]
fn test_complex_expression() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_6() {
        let x = 1 + 2 * 3;
        assert_eq!(x, 7);
    }

    tn_ex_6();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
}

fn get_value_helper() -> i32 {
    42
}

#[test]
fn test_function_call_initializer() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_7() {
        let x = get_value_helper();
        assert_eq!(x, 42);
    }

    tn_ex_7();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
}

#[test]
fn test_mutable_variable() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_8() {
        let mut x = 42;
        x += 1;
        assert_eq!(x, 43);
    }

    tn_ex_8();

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
}

#[test]
fn test_nested_blocks() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_9() {
        let x = 1;
        {
            let y = 2;
            assert_eq!(y, 2);
        }
        assert_eq!(x, 1);
    }

    tn_ex_9();

    let events = get_events();
    assert!(events.len() >= 2, "Should have at least 2 events"); // 2 New + 2 Drop
}

#[test]
fn test_preserves_return_value() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn tn_ex_10() -> i32 {
        let x = 42;
        x
    }

    let result = tn_ex_10();
    assert_eq!(result, 42);

    let events = get_events();
    assert!(events.len() >= 1, "Should have at least 1 events"); // New + Drop
}
