use borrowscope_macro::trace_borrow;

#[test]
fn test_box_new_simple() {
    #[trace_borrow]
    fn example() {
        let _x = Box::new(42);
    }

    example();
}

#[test]
fn test_box_new_string() {
    #[trace_borrow]
    fn example() {
        let _x = Box::new(String::from("hello"));
    }

    example();
}

#[test]
fn test_box_move() {
    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _y = x; // Move
    }

    example();
}

#[test]
fn test_box_deref_borrow() {
    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _r = &*x; // Explicit deref and borrow
    }

    example();
}

#[test]
fn test_box_deref_mut_borrow() {
    #[trace_borrow]
    fn example() {
        let mut x = Box::new(42);
        let _r = &mut *x; // Mutable deref and borrow
    }

    example();
}

#[test]
fn test_box_nested() {
    #[trace_borrow]
    fn example() {
        let _x = Box::new(Box::new(42));
    }

    example();
}

#[test]
fn test_box_in_vec() {
    #[trace_borrow]
    fn example() {
        let _v = vec![Box::new(1), Box::new(2), Box::new(3)];
    }

    example();
}

#[test]
fn test_box_with_struct() {
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[trace_borrow]
    fn example() {
        let _p = Box::new(Point { x: 10, y: 20 });
    }

    example();
}

#[test]
fn test_box_clone_content() {
    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _y = x.clone(); // Clone the Box (creates new Box)
    }

    example();
}

#[test]
fn test_box_multiple_operations() {
    #[trace_borrow]
    fn example() {
        let x = Box::new(100);
        let y = Box::new(200);
        let _r1 = &*x;
        let _r2 = &*y;
        let _z = x; // Move x
    }

    example();
}

#[test]
fn test_box_in_function_call() {
    fn consume_box(_b: Box<i32>) {}

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        consume_box(x); // Move into function
    }

    example();
}

#[test]
fn test_box_return_from_function() {
    fn create_box() -> Box<i32> {
        Box::new(42)
    }

    #[trace_borrow]
    fn example() {
        let _x = create_box();
    }

    example();
}

#[test]
fn test_box_with_generic() {
    #[trace_borrow]
    fn example<T: Default>() {
        let _x = Box::new(T::default());
    }

    example::<i32>();
}

#[test]
fn test_box_pattern_matching() {
    #[trace_borrow]
    fn example() {
        let x = Box::new(Some(42));
        if let Some(_val) = *x {
            // Pattern match on dereferenced Box
        }
    }

    example();
}

#[test]
fn test_box_full_path() {
    #[trace_borrow]
    fn example() {
        let _x = std::boxed::Box::new(42);
    }

    example();
}
