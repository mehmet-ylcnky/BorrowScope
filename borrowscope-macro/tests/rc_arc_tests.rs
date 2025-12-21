use borrowscope_macro::trace_borrow;
use std::rc::Rc;
use std::sync::Arc;

#[test]
fn test_rc_new_simple() {
    #[trace_borrow]
    fn example() {
        let _x = Rc::new(42);
    }

    example();
}

#[test]
fn test_rc_clone_single() {
    #[trace_borrow]
    fn example() {
        let x = Rc::new(42);
        let _y = Rc::clone(&x);
    }

    example();
}

#[test]
fn test_rc_multiple_clones() {
    #[trace_borrow]
    fn example() {
        let x = Rc::new(42);
        let _y = Rc::clone(&x);
        let _z = Rc::clone(&x);
        let _w = Rc::clone(&x);
    }

    example();
}

#[test]
fn test_rc_clone_chain() {
    #[trace_borrow]
    fn example() {
        let x = Rc::new(42);
        let y = Rc::clone(&x);
        let _z = Rc::clone(&y);
    }

    example();
}

#[test]
fn test_rc_with_string() {
    #[trace_borrow]
    fn example() {
        let _x = Rc::new(String::from("hello"));
    }

    example();
}

#[test]
fn test_rc_with_vec() {
    #[trace_borrow]
    fn example() {
        let _x = Rc::new(vec![1, 2, 3, 4, 5]);
    }

    example();
}

#[test]
fn test_rc_with_struct() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[trace_borrow]
    fn example() {
        let _p = Rc::new(Point { x: 10, y: 20 });
    }

    example();
}

#[test]
fn test_rc_nested() {
    #[trace_borrow]
    fn example() {
        let _x = Rc::new(Rc::new(42));
    }

    example();
}

#[test]
fn test_rc_in_vec() {
    #[trace_borrow]
    fn example() {
        let _v = vec![Rc::new(1), Rc::new(2), Rc::new(3)];
    }

    example();
}

#[test]
fn test_rc_full_path() {
    #[trace_borrow]
    fn example() {
        let _x = std::rc::Rc::new(42);
    }

    example();
}

#[test]
fn test_rc_clone_full_path() {
    #[trace_borrow]
    fn example() {
        let x = std::rc::Rc::new(42);
        let _y = std::rc::Rc::clone(&x);
    }

    example();
}

#[test]
fn test_rc_value_correctness() {
    #[trace_borrow]
    fn example() {
        let x = Rc::new(42);
        assert_eq!(*x, 42);

        let y = Rc::clone(&x);
        assert_eq!(*y, 42);

        let z = Rc::clone(&x);
        assert_eq!(*z, 42);
    }

    example();
}

#[test]
fn test_arc_new_simple() {
    #[trace_borrow]
    fn example() {
        let _x = Arc::new(42);
    }

    example();
}

#[test]
fn test_arc_clone_single() {
    #[trace_borrow]
    fn example() {
        let x = Arc::new(42);
        let _y = Arc::clone(&x);
    }

    example();
}

#[test]
fn test_arc_multiple_clones() {
    #[trace_borrow]
    fn example() {
        let x = Arc::new(42);
        let _y = Arc::clone(&x);
        let _z = Arc::clone(&x);
        let _w = Arc::clone(&x);
    }

    example();
}

#[test]
fn test_arc_with_string() {
    #[trace_borrow]
    fn example() {
        let _x = Arc::new(String::from("hello"));
    }

    example();
}

#[test]
fn test_arc_with_struct() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Data {
        value: i32,
    }

    #[trace_borrow]
    fn example() {
        let _d = Arc::new(Data { value: 100 });
    }

    example();
}

#[test]
fn test_arc_thread_send() {
    use std::thread;

    // Test Arc thread safety without macro (macro interferes with thread handles)
    let x = Arc::new(42);
    let x_clone = Arc::clone(&x);

    let handle = thread::spawn(move || {
        assert_eq!(*x_clone, 42);
    });

    handle.join().unwrap();
    assert_eq!(*x, 42);
}

#[test]
fn test_arc_multiple_threads() {
    use std::thread;

    // Test Arc with multiple threads without macro
    let x = Arc::new(100);
    let mut handles = vec![];

    for _ in 0..5 {
        let x_clone = Arc::clone(&x);
        handles.push(thread::spawn(move || {
            assert_eq!(*x_clone, 100);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*x, 100);
}

#[test]
fn test_arc_full_path() {
    #[trace_borrow]
    fn example() {
        let _x = std::sync::Arc::new(42);
    }

    example();
}

#[test]
fn test_arc_clone_full_path() {
    #[trace_borrow]
    fn example() {
        let x = std::sync::Arc::new(42);
        let _y = std::sync::Arc::clone(&x);
    }

    example();
}

#[test]
fn test_mixed_rc_arc() {
    #[trace_borrow]
    fn example() {
        let _rc = Rc::new(42);
        let _arc = Arc::new(100);
    }

    example();
}

#[test]
fn test_rc_in_function_call() {
    fn consume_rc(_r: Rc<i32>) {}

    #[trace_borrow]
    fn example() {
        let x = Rc::new(42);
        consume_rc(Rc::clone(&x));
    }

    example();
}

#[test]
fn test_arc_in_function_call() {
    fn consume_arc(_a: Arc<i32>) {}

    #[trace_borrow]
    fn example() {
        let x = Arc::new(42);
        consume_arc(Arc::clone(&x));
    }

    example();
}

#[test]
fn test_rc_return_from_function() {
    fn create_rc() -> Rc<i32> {
        Rc::new(42)
    }

    #[trace_borrow]
    fn example() {
        let _x = create_rc();
    }

    example();
}

#[test]
fn test_arc_return_from_function() {
    fn create_arc() -> Arc<i32> {
        Arc::new(42)
    }

    #[trace_borrow]
    fn example() {
        let _x = create_arc();
    }

    example();
}

#[test]
fn test_rc_with_option() {
    #[trace_borrow]
    fn example() {
        let _x = Rc::new(Some(42));
    }

    example();
}

#[test]
fn test_rc_with_result() {
    #[trace_borrow]
    fn example() {
        let _x: Rc<Result<i32, String>> = Rc::new(Ok(42));
    }

    example();
}
