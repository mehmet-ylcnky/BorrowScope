use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[test]
fn test_simple_variable_tracking() {
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        assert_eq!(x, 42);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
    assert!(events[1].is_drop());
}

#[test]
fn test_typed_variable() {
    reset();

    #[trace_borrow]
    fn example() {
        let x: i32 = 42;
        assert_eq!(x, 42);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_string_variable() {
    reset();

    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        assert_eq!(s, "hello");
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_vec_variable() {
    reset();

    #[trace_borrow]
    fn example() {
        let v = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
    assert!(events[0].is_new());
}

#[test]
fn test_multiple_variables() {
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        let y = 100;
        let z = x + y;
        assert_eq!(z, 142);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 6); // 3 New + 3 Drop
}

#[test]
fn test_complex_expression() {
    reset();

    #[trace_borrow]
    fn example() {
        let x = 1 + 2 * 3;
        assert_eq!(x, 7);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

fn get_value_helper() -> i32 {
    42
}

#[test]
fn test_function_call_initializer() {
    reset();

    #[trace_borrow]
    fn example() {
        let x = get_value_helper();
        assert_eq!(x, 42);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

#[test]
fn test_mutable_variable() {
    reset();

    #[trace_borrow]
    fn example() {
        let mut x = 42;
        x += 1;
        assert_eq!(x, 43);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}

#[test]
fn test_nested_blocks() {
    reset();

    #[trace_borrow]
    fn example() {
        let x = 1;
        {
            let y = 2;
            assert_eq!(y, 2);
        }
        assert_eq!(x, 1);
    }

    example();

    let events = get_events();
    assert_eq!(events.len(), 4); // 2 New + 2 Drop
}

#[test]
fn test_preserves_return_value() {
    reset();

    #[trace_borrow]
    fn example() -> i32 {
        let x = 42;
        x
    }

    let result = example();
    assert_eq!(result, 42);

    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}
