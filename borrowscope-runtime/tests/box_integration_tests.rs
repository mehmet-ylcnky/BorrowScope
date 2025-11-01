use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

#[test]
fn test_box_new_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = Box::new(42);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should have tracked events");

    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should have New event for Box");
}

#[test]
fn test_box_move_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _y = x;
    }

    example();

    let events = get_events();
    assert!(!events.is_empty());

    let has_move = events.iter().any(|e| e.is_move());
    assert!(has_move, "Should have Move event for Box");
}

#[test]
fn test_box_borrow_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _r = &x; // Borrow the Box itself
    }

    example();

    let events = get_events();
    assert!(!events.is_empty());

    let has_borrow = events.iter().any(|e| e.is_borrow());
    assert!(has_borrow, "Should have Borrow event for Box");
}

#[test]
fn test_box_mut_borrow_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let mut x = Box::new(42);
        let _r = &mut x; // Mutable borrow of the Box itself
    }

    example();

    let events = get_events();
    assert!(!events.is_empty());

    let has_mut_borrow = events.iter().any(|e| {
        if let Event::Borrow { mutable, .. } = e {
            *mutable
        } else {
            false
        }
    });
    assert!(has_mut_borrow, "Should have mutable Borrow event");
}

#[test]
fn test_box_nested() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = Box::new(Box::new(42));
    }

    example();

    let events = get_events();
    assert!(!events.is_empty());

    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should track nested Box creation");
}

#[test]
fn test_box_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = Box::new(String::from("hello"));
    }

    example();

    let events = get_events();
    assert!(!events.is_empty());

    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should track Box<String> creation");
}

#[test]
fn test_box_multiple_allocations() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = Box::new(1);
        let _y = Box::new(2);
        let _z = Box::new(3);
    }

    example();

    let events = get_events();
    let new_count = events.iter().filter(|e| e.is_new()).count();
    assert!(new_count >= 3, "Should track all three Box allocations");
}

#[test]
fn test_box_move_chain() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let y = x;
        let _z = y;
    }

    example();

    let events = get_events();
    let move_count = events.iter().filter(|e| e.is_move()).count();
    assert!(move_count >= 2, "Should track both moves");
}

#[test]
fn test_box_borrow_multiple() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _r1 = &x; // Borrow the Box
        let _r2 = &x; // Another borrow of the Box
    }

    example();

    let events = get_events();
    let borrow_count = events.iter().filter(|e| e.is_borrow()).count();
    assert!(borrow_count >= 2, "Should track multiple borrows");
}

#[test]
fn test_box_value_correctness() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        assert_eq!(*x, 42);

        let y = x;
        assert_eq!(*y, 42);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should have events");
}

#[test]
fn test_box_with_struct() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[trace_borrow]
    fn example() {
        let p = Box::new(Point { x: 10, y: 20 });
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    example();

    let events = get_events();
    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should track Box<Point> creation");
}

#[test]
fn test_box_deref_in_expression() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(10);
        let y = Box::new(20);
        let sum = *x + *y;
        assert_eq!(sum, 30);
    }

    example();

    let events = get_events();
    let new_count = events.iter().filter(|e| e.is_new()).count();
    assert!(new_count >= 2, "Should track both Box allocations");
}

#[test]
fn test_box_clone_creates_new_box() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        let _y = x.clone();
    }

    example();

    let events = get_events();
    let new_count = events.iter().filter(|e| e.is_new()).count();
    assert!(new_count >= 2, "Clone should create new Box");
}

#[test]
fn test_box_in_function_call() {
    fn consume_box(_b: Box<i32>) {}

    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = Box::new(42);
        consume_box(x);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track Box creation and move");
}

#[test]
fn test_box_return_from_function() {
    fn create_box() -> Box<i32> {
        Box::new(42)
    }

    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = create_box();
    }

    example();

    let events = get_events();
    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should track Box assignment");
}

#[test]
fn test_box_full_path() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = std::boxed::Box::new(42);
    }

    example();

    let events = get_events();
    let has_new = events.iter().any(|e| e.is_new());
    assert!(has_new, "Should track Box with full path");
}

#[test]
fn test_box_drop_tracking() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let _x = Box::new(42);
    }

    example();

    let events = get_events();
    let has_drop = events.iter().any(|e| e.is_drop());
    assert!(has_drop, "Should track Box drop");
}
