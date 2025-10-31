//! Comprehensive integration tests for macro + runtime

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

lazy_static::lazy_static! {
    static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

// ============================================================================
// Simple Variable Tests
// ============================================================================

#[test]
fn test_simple_variable_creation() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        assert_eq!(x, 42);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should have tracked events");

    // Should have at least one New or Move event
    let has_creation = events.iter().any(|e| e.is_new() || e.is_move());
    assert!(has_creation, "Should have variable creation event");

    // Should have Drop event
    let has_drop = events.iter().any(|e| e.is_drop());
    assert!(has_drop, "Should have drop event");
}

#[test]
fn test_multiple_variables() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 1;
        let y = 2;
        let z = x + y;
        assert_eq!(z, 3);
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track all three variables");
}

#[test]
fn test_variable_shadowing() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 1;
        let x = x + 1;
        let x = x * 2;
        assert_eq!(x, 4);
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track all shadowed variables");
}

// ============================================================================
// Borrow Tests
// ============================================================================

#[test]
fn test_immutable_borrow() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = String::from("hello");
        let r = &x;
        assert_eq!(r, "hello");
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should have tracked borrow");
}

#[test]
fn test_mutable_borrow() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let mut x = 42;
        let r = &mut x;
        *r += 1;
        assert_eq!(*r, 43);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should have tracked mutable borrow");
}

#[test]
fn test_multiple_immutable_borrows() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = vec![1, 2, 3];
        let r1 = &x;
        let r2 = &x;
        assert_eq!(r1.len(), 3);
        assert_eq!(r2.len(), 3);
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track x, r1, and r2");
}

// ============================================================================
// Move Tests
// ============================================================================

#[test]
fn test_simple_move() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s1 = String::from("hello");
        let s2 = s1;
        assert_eq!(s2, "hello");
    }

    example();

    let events = get_events();

    // Should have Move event
    let has_move = events.iter().any(|e| e.is_move());
    assert!(has_move, "Should have move event");
}

#[test]
fn test_move_chain() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s1 = String::from("hello");
        let s2 = s1;
        let s3 = s2;
        assert_eq!(s3, "hello");
    }

    example();

    let events = get_events();

    // Count move events
    let move_count = events.iter().filter(|e| e.is_move()).count();
    assert!(move_count >= 2, "Should have at least 2 move events");
}

#[test]
fn test_move_into_function() {
    let _lock = TEST_LOCK.lock();
    reset();

    fn takes_ownership(_s: String) {}

    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        takes_ownership(s);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track variable before move");
}

// ============================================================================
// Pattern Tests
// ============================================================================

#[test]
fn test_tuple_destructuring() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let (x, y) = (1, 2);
        assert_eq!(x, 1);
        assert_eq!(y, 2);
    }

    example();

    let events = get_events();
    assert!(events.len() >= 2, "Should track destructured variables");
}

#[test]
fn test_struct_destructuring() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        struct Point {
            x: i32,
            y: i32,
        }
        let p = Point { x: 10, y: 20 };
        let Point { x, y } = p;
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track struct destructuring");
}

// ============================================================================
// Control Flow Tests
// ============================================================================

#[test]
fn test_if_else() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example(condition: bool) -> i32 {
        if condition {
            let x = 1;
            x
        } else {
            let y = 2;
            y
        }
    }

    let result = example(true);
    assert_eq!(result, 1);

    let events = get_events();
    assert!(!events.is_empty(), "Should track if branch variable");
}

#[test]
fn test_match_expression() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example(opt: Option<i32>) -> i32 {
        match opt {
            Some(x) => {
                let y = x + 1;
                y
            }
            None => 0,
        }
    }

    let result = example(Some(42));
    assert_eq!(result, 43);

    let events = get_events();
    assert!(!events.is_empty(), "Should track match arm variable");
}

#[test]
fn test_for_loop() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        for i in 0..3 {
            let x = i * 2;
            assert!(x < 10);
        }
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track loop iterations");
}

#[test]
fn test_while_loop() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let mut count = 0;
        while count < 3 {
            let x = count;
            count += 1;
            assert!(x < 3);
        }
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track loop variables");
}

// ============================================================================
// Nested Scope Tests
// ============================================================================

#[test]
fn test_nested_blocks() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 1;
        {
            let y = 2;
            {
                let z = 3;
                assert_eq!(x + y + z, 6);
            }
        }
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track all nested variables");
}

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_string_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        assert_eq!(s.len(), 5);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track String");
}

#[test]
fn test_vec_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let v = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track Vec");
}

#[test]
fn test_box_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let b = Box::new(42);
        assert_eq!(*b, 42);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track Box");
}

#[test]
fn test_option_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let opt = Some(42);
        assert!(opt.is_some());
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track Option");
}

// ============================================================================
// Generic Function Tests
// ============================================================================

#[test]
fn test_generic_function_with_i32() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn identity<T>(value: T) -> T {
        let x = value;
        x
    }

    let result = identity(42);
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty(), "Should track generic function");
}

#[test]
fn test_generic_function_with_string() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn identity<T>(value: T) -> T {
        let x = value;
        x
    }

    let result = identity(String::from("hello"));
    assert_eq!(result, "hello");

    let events = get_events();
    assert!(
        !events.is_empty(),
        "Should track generic function with String"
    );
}

#[test]
fn test_generic_with_trait_bound() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn clone_value<T: Clone>(value: T) -> T {
        let x = value.clone();
        x
    }

    let result = clone_value(42);
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty(), "Should track generic with trait bound");
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_complex_ownership_scenario() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r1 = &s;
        let r2 = &s;
        let len = r1.len();
        assert_eq!(len, 5);
        assert_eq!(r2.len(), 5);
    }

    example();

    let events = get_events();
    assert!(events.len() >= 3, "Should track complex scenario");
}

#[test]
fn test_method_chaining() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s = String::from("  hello  ");
        let trimmed = s.trim();
        let upper = trimmed.to_uppercase();
        assert_eq!(upper, "HELLO");
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track method chaining");
}

#[test]
fn test_closure_capture() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        let closure = || x + 1;
        let result = closure();
        assert_eq!(result, 43);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track closure capture");
}

// ============================================================================
// Graph Building Tests
// ============================================================================

#[test]
fn test_graph_building() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        let y = x + 1;
        assert_eq!(y, 43);
    }

    example();

    let events = get_events();
    let graph = build_graph(&events);

    assert!(!graph.nodes.is_empty(), "Graph should have nodes");

    let stats = graph.stats();
    assert!(stats.total_variables > 0, "Should have variables in graph");
}

#[test]
fn test_graph_with_borrows() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = String::from("hello");
        let r = &x;
        assert_eq!(r, "hello");
    }

    example();

    let events = get_events();
    let graph = build_graph(&events);

    let stats = graph.stats();
    assert!(stats.total_variables > 0, "Should have variables");
}

// ============================================================================
// Export Tests
// ============================================================================

#[test]
fn test_json_export() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let x = 42;
        let y = x + 1;
        assert_eq!(y, 43);
    }

    example();

    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    let json = serde_json::to_string(&export).unwrap();
    assert!(!json.is_empty(), "Should export to JSON");

    // Verify JSON structure
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(data["nodes"].is_array());
    assert!(data["events"].is_array());
    assert!(data["metadata"].is_object());
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_performance_many_variables() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        for i in 0..100 {
            let x = i * 2;
            assert!(x < 300);
        }
    }

    let start = std::time::Instant::now();
    example();
    let duration = start.elapsed();

    println!("100 iterations took: {:?}", duration);
    assert!(
        duration.as_millis() < 1000,
        "Should complete in reasonable time"
    );

    let events = get_events();
    assert!(events.len() >= 100, "Should track all iterations");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_result_type() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() -> std::result::Result<i32, String> {
        let x = 42;
        Ok(x)
    }

    let result = example();
    assert!(result.is_ok());

    let events = get_events();
    assert!(!events.is_empty(), "Should track Result type");
}

#[test]
fn test_option_unwrap() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() -> i32 {
        let opt = Some(42);
        let x = opt.unwrap();
        x
    }

    let result = example();
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty(), "Should track Option unwrap");
}

// ============================================================================
// Real-World Scenarios
// ============================================================================

#[test]
fn test_vector_operations() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let mut v = vec![1, 2, 3];
        v.push(4);
        v.push(5);
        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 15);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track vector operations");
}

#[test]
fn test_string_manipulation() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let s1 = String::from("Hello");
        let s2 = String::from(" World");
        let s3 = format!("{}{}", s1, s2);
        assert_eq!(s3, "Hello World");
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track string manipulation");
}

#[test]
fn test_nested_data_structures() {
    let _lock = TEST_LOCK.lock();
    reset();

    #[trace_borrow]
    fn example() {
        let v = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let first = &v[0];
        assert_eq!(first.len(), 2);
    }

    example();

    let events = get_events();
    assert!(!events.is_empty(), "Should track nested structures");
}
