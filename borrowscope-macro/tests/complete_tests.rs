use borrowscope_macro::trace_borrow;

#[test]
fn test_complete_simple() {
    #[trace_borrow]
    fn example() {
        let x = 5;
        let y = 10;
        assert_eq!(x + y, 15);
    }

    example();
}

#[test]
fn test_complete_with_borrows() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r = &s;
        assert_eq!(r, "hello");
    }

    example();
}

#[test]
fn test_complete_with_mut_borrow() {
    #[trace_borrow]
    fn example() {
        let mut x = 5;
        let y = &mut x;
        *y += 10;
        assert_eq!(*y, 15);
    }

    example();
}

#[test]
fn test_complete_with_tuple() {
    #[trace_borrow]
    fn example() {
        let (x, y) = (1, 2);
        assert_eq!(x + y, 3);
    }

    example();
}

#[test]
fn test_complete_with_string() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let len = s.len();
        assert_eq!(len, 5);
    }

    example();
}

#[test]
fn test_complete_with_params() {
    #[trace_borrow]
    fn add(a: i32, b: i32) -> i32 {
        let result = a + b;
        result
    }

    let sum = add(5, 10);
    assert_eq!(sum, 15);
}

#[test]
fn test_complete_with_generics() {
    #[trace_borrow]
    fn identity<T>(value: T) -> T {
        value
    }

    assert_eq!(identity(42), 42);
    assert_eq!(identity("hello"), "hello");
}

#[test]
fn test_complete_with_lifetimes() {
    #[trace_borrow]
    fn first<'a>(x: &'a str, _y: &'a str) -> &'a str {
        x
    }

    let result = first("hello", "world");
    assert_eq!(result, "hello");
}

#[test]
fn test_complete_with_control_flow() {
    #[trace_borrow]
    fn example(n: i32) -> i32 {
        let mut sum = 0;
        for i in 0..n {
            sum += i;
        }
        sum
    }

    assert_eq!(example(5), 10);
}

#[test]
fn test_complete_with_match() {
    #[trace_borrow]
    fn example(x: Option<i32>) -> i32 {
        #[allow(clippy::manual_unwrap_or, clippy::manual_unwrap_or_default)]
        match x {
            Some(val) => val,
            None => 0,
        }
    }

    assert_eq!(example(Some(42)), 42);
    assert_eq!(example(None), 0);
}
