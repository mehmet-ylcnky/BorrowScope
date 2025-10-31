# Section 27: Integration Testing

## Learning Objectives

By the end of this section, you will:
- Write end-to-end integration tests
- Test macro and runtime together
- Verify real-world scenarios
- Use test fixtures effectively
- Implement snapshot testing

## Prerequisites

- Completed Section 26 (Performance Optimization)
- Understanding of Rust's test framework
- Familiarity with integration vs unit tests

---

## Integration vs Unit Tests

**Unit tests:** Test individual functions in isolation.

```rust
#[test]
fn test_event_creation() {
    let event = Event::New { /* ... */ };
    assert_eq!(event.id, 1);
}
```

**Integration tests:** Test multiple components working together.

```rust
#[test]
fn test_macro_with_runtime() {
    #[track_ownership]
    fn example() {
        let x = 42;
        let r = &x;
    }
    
    example();
    let events = get_events();
    assert_eq!(events.len(), 3); // New, Borrow, Drop
}
```

---

## Setting Up Integration Tests

Create `borrowscope-runtime/tests/integration/mod.rs`:

```rust
//! Integration test utilities

use borrowscope_runtime::*;

/// Test fixture that resets tracker before each test
pub struct TestFixture;

impl TestFixture {
    pub fn new() -> Self {
        reset_tracker();
        Self
    }
    
    pub fn events(&self) -> Vec<Event> {
        get_events()
    }
    
    pub fn event_count(&self) -> usize {
        get_events().len()
    }
    
    pub fn assert_event_types(&self, expected: &[&str]) {
        let events = self.events();
        let actual: Vec<&str> = events.iter()
            .map(|e| match e {
                Event::New { .. } => "New",
                Event::Borrow { .. } => "Borrow",
                Event::Move { .. } => "Move",
                Event::Drop { .. } => "Drop",
            })
            .collect();
        
        assert_eq!(actual, expected);
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        reset_tracker();
    }
}
```

---

## Test 1: Simple Variable Lifecycle

Create `borrowscope-runtime/tests/simple_lifecycle.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_variable_creation_and_drop() {
    let fixture = TestFixture::new();
    
    {
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        assert_eq!(x, 42);
        track_drop(1, "test.rs:2:1");
    }
    
    fixture.assert_event_types(&["New", "Drop"]);
}

#[test]
fn test_multiple_variables() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let y = track_new(2, "y", "i32", "test.rs:2:1", 100);
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    assert_eq!(fixture.event_count(), 4);
    fixture.assert_event_types(&["New", "New", "Drop", "Drop"]);
}
```

---

## Test 2: Borrowing Scenarios

Create `borrowscope-runtime/tests/borrowing.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_immutable_borrow() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    
    assert_eq!(*r, 42);
    
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    fixture.assert_event_types(&["New", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_mutable_borrow() {
    let fixture = TestFixture::new();
    
    let mut x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow_mut(2, 1, true, "test.rs:2:1", &mut x);
    
    *r = 100;
    assert_eq!(*r, 100);
    
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    fixture.assert_event_types(&["New", "Borrow", "Drop", "Drop"]);
    
    // Verify borrow was mutable
    let events = fixture.events();
    if let Event::Borrow { is_mutable, .. } = &events[1] {
        assert!(is_mutable);
    } else {
        panic!("Expected Borrow event");
    }
}

#[test]
fn test_multiple_immutable_borrows() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r1 = track_borrow(2, 1, false, "test.rs:2:1", &x);
    let r2 = track_borrow(3, 1, false, "test.rs:3:1", &x);
    
    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);
    
    track_drop(3, "test.rs:4:1");
    track_drop(2, "test.rs:5:1");
    track_drop(1, "test.rs:6:1");
    
    fixture.assert_event_types(&["New", "Borrow", "Borrow", "Drop", "Drop", "Drop"]);
}
```

---

## Test 3: Move Semantics

Create `borrowscope-runtime/tests/moves.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_simple_move() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "String", "test.rs:1:1", String::from("hello"));
    track_move(1, 2, "test.rs:2:1");
    let y = x; // Actual move
    
    assert_eq!(y, "hello");
    
    track_drop(2, "test.rs:3:1");
    
    fixture.assert_event_types(&["New", "Move", "Drop"]);
}

#[test]
fn test_move_chain() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "String", "test.rs:1:1", String::from("data"));
    track_move(1, 2, "test.rs:2:1");
    let y = x;
    track_move(2, 3, "test.rs:3:1");
    let z = y;
    
    assert_eq!(z, "data");
    
    track_drop(3, "test.rs:4:1");
    
    fixture.assert_event_types(&["New", "Move", "Move", "Drop"]);
}
```

---

## Test 4: Graph Building

Create `borrowscope-runtime/tests/graph_building.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_graph_from_events() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    let events = fixture.events();
    let graph = OwnershipGraph::from_events(&events);
    
    let stats = graph.statistics();
    assert_eq!(stats.total_variables, 2);
    assert_eq!(stats.total_borrows, 1);
}

#[test]
fn test_graph_relationships() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r1 = track_borrow(2, 1, false, "test.rs:2:1", &x);
    let r2 = track_borrow(3, 1, false, "test.rs:3:1", &x);
    
    track_drop(3, "test.rs:4:1");
    track_drop(2, "test.rs:5:1");
    track_drop(1, "test.rs:6:1");
    
    let events = fixture.events();
    let graph = OwnershipGraph::from_events(&events);
    
    // Verify relationships
    let borrows = graph.get_borrows(1);
    assert_eq!(borrows.len(), 2);
}
```

---

## Test 5: JSON Export

Create `borrowscope-runtime/tests/json_export.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;
use serde_json::Value;

#[test]
fn test_json_structure() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    track_drop(1, "test.rs:2:1");
    
    let json = export_json().unwrap();
    let data: Value = serde_json::from_str(&json).unwrap();
    
    assert!(data["nodes"].is_array());
    assert!(data["edges"].is_array());
    assert!(data["events"].is_array());
    assert!(data["metadata"].is_object());
}

#[test]
fn test_json_content() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    let json = export_json().unwrap();
    let data: Value = serde_json::from_str(&json).unwrap();
    
    // Check nodes
    assert_eq!(data["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(data["nodes"][0]["name"], "x");
    
    // Check edges
    assert_eq!(data["edges"].as_array().unwrap().len(), 1);
    assert_eq!(data["edges"][0]["relationship"], "borrows_immut");
    
    // Check events
    assert_eq!(data["events"].as_array().unwrap().len(), 4);
}
```

---

## Test 6: Real-World Scenarios

Create `borrowscope-runtime/tests/real_world.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_vector_operations() {
    let fixture = TestFixture::new();
    
    let mut v = track_new(1, "v", "Vec<i32>", "test.rs:1:1", vec![1, 2, 3]);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &v);
    
    assert_eq!(r.len(), 3);
    
    track_drop(2, "test.rs:3:1");
    
    let r_mut = track_borrow_mut(3, 1, true, "test.rs:4:1", &mut v);
    r_mut.push(4);
    
    track_drop(3, "test.rs:5:1");
    track_drop(1, "test.rs:6:1");
    
    fixture.assert_event_types(&["New", "Borrow", "Drop", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_struct_field_access() {
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }
    
    let fixture = TestFixture::new();
    
    let p = track_new(1, "p", "Point", "test.rs:1:1", Point { x: 10, y: 20 });
    let r = track_borrow(2, 1, false, "test.rs:2:1", &p);
    
    assert_eq!(r.x, 10);
    assert_eq!(r.y, 20);
    
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    assert_eq!(fixture.event_count(), 4);
}

#[test]
fn test_nested_scopes() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    
    {
        let r1 = track_borrow(2, 1, false, "test.rs:3:1", &x);
        assert_eq!(*r1, 42);
        track_drop(2, "test.rs:4:1");
    }
    
    {
        let r2 = track_borrow(3, 1, false, "test.rs:7:1", &x);
        assert_eq!(*r2, 42);
        track_drop(3, "test.rs:8:1");
    }
    
    track_drop(1, "test.rs:10:1");
    
    fixture.assert_event_types(&["New", "Borrow", "Drop", "Borrow", "Drop", "Drop"]);
}
```

---

## Snapshot Testing

For complex outputs, use snapshot testing with `insta`:

Add to `Cargo.toml`:

```toml
[dev-dependencies]
insta = "1.34"
```

Create `borrowscope-runtime/tests/snapshots.rs`:

```rust
mod integration;
use integration::TestFixture;
use borrowscope_runtime::*;

#[test]
fn test_json_snapshot() {
    let fixture = TestFixture::new();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    track_drop(2, "test.rs:3:1");
    track_drop(1, "test.rs:4:1");
    
    let json = export_json().unwrap();
    
    insta::assert_snapshot!(json);
}
```

Run and review:

```bash
cargo test snapshots
cargo insta review
```

---

## Test Coverage

Measure test coverage with `tarpaulin`:

```bash
cargo install cargo-tarpaulin

cargo tarpaulin --package borrowscope-runtime --out Html
```

Open `tarpaulin-report.html` to see coverage.

**Target:** >80% coverage for runtime crate.

---

## Continuous Integration

Update `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all
      
      - name: Run integration tests
        run: cargo test --package borrowscope-runtime --test '*'
      
      - name: Check coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --package borrowscope-runtime --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Key Takeaways

✅ **Integration tests verify end-to-end behavior**  
✅ **Test fixtures reduce boilerplate**  
✅ **Snapshot testing catches unexpected changes**  
✅ **Real-world scenarios ensure practical correctness**  
✅ **Coverage tools identify untested code**  
✅ **CI ensures tests run on every commit**  

---

## Further Reading

- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Integration testing patterns](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html)
- [insta snapshot testing](https://insta.rs/)
- [tarpaulin coverage](https://github.com/xd009642/tarpaulin)

---

**Previous:** [26-performance-optimization.md](./26-performance-optimization.md)  
**Next:** [28-error-handling.md](./28-error-handling.md)

**Progress:** 7/15 ⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜
