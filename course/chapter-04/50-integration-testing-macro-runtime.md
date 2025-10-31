# Section 50: Integration Testing Macro + Runtime

## Learning Objectives

By the end of this section, you will:
- Test macro and runtime together
- Verify end-to-end functionality
- Create comprehensive test suites
- Test real-world scenarios
- Validate complete pipeline

## Prerequisites

- Completed Section 49 (Generic Functions)
- Understanding of integration testing
- Familiarity with test organization

---

## Integration Test Structure

```
borrowscope/
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── simple.rs
│   │   ├── borrows.rs
│   │   ├── moves.rs
│   │   ├── patterns.rs
│   │   ├── control_flow.rs
│   │   ├── smart_pointers.rs
│   │   └── generics.rs
│   └── end_to_end.rs
```

---

## Basic Integration Test

```rust
// tests/integration/simple.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn simple_variable() {
    let x = 42;
}

#[test]
fn test_simple_variable() {
    reset_tracker();
    
    simple_variable();
    
    let events = get_events();
    assert_eq!(events.len(), 2);  // New + Drop
    
    match &events[0] {
        Event::New { id, name, .. } => {
            assert_eq!(*id, 1);
            assert_eq!(name, "x");
        }
        _ => panic!("Expected New event"),
    }
    
    match &events[1] {
        Event::Drop { id, .. } => {
            assert_eq!(*id, 1);
        }
        _ => panic!("Expected Drop event"),
    }
}
```

---

## Borrow Testing

```rust
// tests/integration/borrows.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn immutable_borrow() {
    let x = 42;
    let r = &x;
    println!("{}", r);
}

#[test]
fn test_immutable_borrow() {
    reset_tracker();
    
    immutable_borrow();
    
    let events = get_events();
    
    // Should have: New(x), New(r), Borrow(x), Drop(r), Drop(x)
    assert!(events.len() >= 3);
    
    // Verify borrow event
    let borrow_event = events.iter().find(|e| matches!(e, Event::Borrow { .. }));
    assert!(borrow_event.is_some());
}

#[track_ownership]
fn mutable_borrow() {
    let mut x = 42;
    let r = &mut x;
    *r = 100;
}

#[test]
fn test_mutable_borrow() {
    reset_tracker();
    
    mutable_borrow();
    
    let events = get_events();
    
    // Find mutable borrow
    let mut_borrow = events.iter().find(|e| {
        matches!(e, Event::Borrow { is_mutable: true, .. })
    });
    
    assert!(mut_borrow.is_some());
}

#[track_ownership]
fn multiple_borrows() {
    let x = 42;
    let r1 = &x;
    let r2 = &x;
    println!("{} {}", r1, r2);
}

#[test]
fn test_multiple_borrows() {
    reset_tracker();
    
    multiple_borrows();
    
    let events = get_events();
    
    // Count borrow events
    let borrow_count = events.iter()
        .filter(|e| matches!(e, Event::Borrow { .. }))
        .count();
    
    assert_eq!(borrow_count, 2);
}
```

---

## Move Testing

```rust
// tests/integration/moves.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn simple_move() {
    let s1 = String::from("hello");
    let s2 = s1;
    println!("{}", s2);
}

#[test]
fn test_simple_move() {
    reset_tracker();
    
    simple_move();
    
    let events = get_events();
    
    // Should have Move event
    let move_event = events.iter().find(|e| matches!(e, Event::Move { .. }));
    assert!(move_event.is_some());
}

#[track_ownership]
fn move_chain() {
    let s1 = String::from("hello");
    let s2 = s1;
    let s3 = s2;
    println!("{}", s3);
}

#[test]
fn test_move_chain() {
    reset_tracker();
    
    move_chain();
    
    let events = get_events();
    
    // Count move events
    let move_count = events.iter()
        .filter(|e| matches!(e, Event::Move { .. }))
        .count();
    
    assert_eq!(move_count, 2);
}
```

---

## Pattern Testing

```rust
// tests/integration/patterns.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn tuple_pattern() {
    let (x, y) = (1, 2);
    println!("{} {}", x, y);
}

#[test]
fn test_tuple_pattern() {
    reset_tracker();
    
    tuple_pattern();
    
    let events = get_events();
    
    // Should track temp, x, and y
    let new_events: Vec<_> = events.iter()
        .filter(|e| matches!(e, Event::New { .. }))
        .collect();
    
    assert!(new_events.len() >= 2);
}

#[track_ownership]
fn struct_pattern() {
    struct Point { x: i32, y: i32 }
    let p = Point { x: 10, y: 20 };
    let Point { x, y } = p;
    println!("{} {}", x, y);
}

#[test]
fn test_struct_pattern() {
    reset_tracker();
    
    struct_pattern();
    
    let events = get_events();
    
    // Verify variables are tracked
    assert!(events.len() > 0);
}
```

---

## Control Flow Testing

```rust
// tests/integration/control_flow.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn if_else(condition: bool) {
    if condition {
        let x = 1;
        println!("{}", x);
    } else {
        let y = 2;
        println!("{}", y);
    }
}

#[test]
fn test_if_branch() {
    reset_tracker();
    
    if_else(true);
    
    let events = get_events();
    
    // Should have x, not y
    let has_x = events.iter().any(|e| {
        matches!(e, Event::New { name, .. } if name == "x")
    });
    
    assert!(has_x);
}

#[track_ownership]
fn for_loop() {
    for i in 0..3 {
        let x = i * 2;
        println!("{}", x);
    }
}

#[test]
fn test_for_loop() {
    reset_tracker();
    
    for_loop();
    
    let events = get_events();
    
    // x is created and dropped 3 times
    let new_count = events.iter()
        .filter(|e| matches!(e, Event::New { name, .. } if name == "x"))
        .count();
    
    assert_eq!(new_count, 3);
}
```

---

## Smart Pointer Testing

```rust
// tests/integration/smart_pointers.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;
use std::rc::Rc;
use std::cell::RefCell;

#[track_ownership]
fn box_test() {
    let x = Box::new(42);
    println!("{}", x);
}

#[test]
fn test_box() {
    reset_tracker();
    
    box_test();
    
    let events = get_events();
    
    // Should track Box allocation
    assert!(events.len() >= 2);
}

#[track_ownership]
fn rc_test() {
    let x = Rc::new(42);
    let y = Rc::clone(&x);
    println!("{} {}", x, y);
}

#[test]
fn test_rc() {
    reset_tracker();
    
    rc_test();
    
    let events = get_events();
    
    // Should have RcNew and RcClone events
    let has_rc_new = events.iter().any(|e| matches!(e, Event::RcNew { .. }));
    let has_rc_clone = events.iter().any(|e| matches!(e, Event::RcClone { .. }));
    
    assert!(has_rc_new);
    assert!(has_rc_clone);
}

#[track_ownership]
fn refcell_test() {
    let x = RefCell::new(42);
    let r = x.borrow();
    println!("{}", r);
}

#[test]
fn test_refcell() {
    reset_tracker();
    
    refcell_test();
    
    let events = get_events();
    
    // Should have RefCellBorrow event
    let has_borrow = events.iter().any(|e| matches!(e, Event::RefCellBorrow { .. }));
    assert!(has_borrow);
}
```

---

## Generic Function Testing

```rust
// tests/integration/generics.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn generic_function<T>(value: T) -> T {
    let x = value;
    x
}

#[test]
fn test_generic_with_i32() {
    reset_tracker();
    
    let result = generic_function(42);
    assert_eq!(result, 42);
    
    let events = get_events();
    
    match &events[0] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("i32"));
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_generic_with_string() {
    reset_tracker();
    
    let result = generic_function(String::from("hello"));
    assert_eq!(result, "hello");
    
    let events = get_events();
    
    match &events[0] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("String"));
        }
        _ => panic!("Expected New event"),
    }
}
```

---

## End-to-End Test

```rust
// tests/end_to_end.rs
use borrowscope_macro::track_ownership;
use borrowscope_runtime::*;

#[track_ownership]
fn complex_example() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    
    let len = r1.len();
    
    let s2 = s;
    
    println!("{} {} {}", r1, r2, s2);
}

#[test]
fn test_complex_example() {
    reset_tracker();
    
    complex_example();
    
    let events = get_events();
    
    // Verify we have all event types
    let has_new = events.iter().any(|e| matches!(e, Event::New { .. }));
    let has_borrow = events.iter().any(|e| matches!(e, Event::Borrow { .. }));
    let has_move = events.iter().any(|e| matches!(e, Event::Move { .. }));
    let has_drop = events.iter().any(|e| matches!(e, Event::Drop { .. }));
    
    assert!(has_new);
    assert!(has_borrow);
    assert!(has_move);
    assert!(has_drop);
    
    // Build graph
    let graph = OwnershipGraph::from_events(&events);
    let stats = graph.statistics();
    
    assert!(stats.total_variables > 0);
    assert!(stats.total_borrows > 0);
}

#[test]
fn test_json_export() {
    reset_tracker();
    
    complex_example();
    
    let json = export_json().unwrap();
    
    // Verify JSON structure
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    
    assert!(data["nodes"].is_array());
    assert!(data["edges"].is_array());
    assert!(data["events"].is_array());
    assert!(data["metadata"].is_object());
}
```

---

## Test Organization

```rust
// tests/integration/mod.rs
pub mod simple;
pub mod borrows;
pub mod moves;
pub mod patterns;
pub mod control_flow;
pub mod smart_pointers;
pub mod generics;

// Common test utilities
pub fn assert_has_event<F>(events: &[Event], predicate: F)
where
    F: Fn(&Event) -> bool,
{
    assert!(events.iter().any(predicate), "Event not found");
}

pub fn count_events<F>(events: &[Event], predicate: F) -> usize
where
    F: Fn(&Event) -> bool,
{
    events.iter().filter(|e| predicate(e)).count()
}
```

---

## Running Tests

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test integration

# Run with output
cargo test --test integration -- --nocapture

# Run specific test function
cargo test test_simple_variable
```

---

## CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run integration tests
        run: cargo test --test '*' --features track
      
      - name: Run end-to-end tests
        run: cargo test --test end_to_end --features track
```

---

## Performance Testing

```rust
#[test]
fn test_performance() {
    use std::time::Instant;
    
    reset_tracker();
    
    let start = Instant::now();
    
    for i in 0..10_000 {
        #[track_ownership]
        fn inner(x: i32) -> i32 {
            let y = x * 2;
            y
        }
        
        inner(i);
    }
    
    let duration = start.elapsed();
    
    println!("10K function calls: {:?}", duration);
    assert!(duration.as_millis() < 1000, "Too slow!");
}
```

---

## Key Takeaways

✅ **Test all features** - Simple, borrows, moves, patterns, control flow  
✅ **Test smart pointers** - Box, Rc, Arc, RefCell  
✅ **Test generics** - Multiple type parameters  
✅ **End-to-end tests** - Complete pipeline verification  
✅ **Performance tests** - Ensure acceptable overhead  
✅ **CI/CD integration** - Automated testing  

---

## Further Reading

- [Integration testing](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Test organization](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html)
- [GitHub Actions](https://docs.github.com/en/actions)

---

**Previous:** [49-handling-generic-functions.md](./49-handling-generic-functions.md)  
**Next:** Chapter 4 Summary

**Progress:** 15/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛ 100% ✅
