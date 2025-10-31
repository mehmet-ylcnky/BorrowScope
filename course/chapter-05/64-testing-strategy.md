# Section 64: Comprehensive Testing Strategy

## Learning Objectives

By the end of this section, you will:
- Design comprehensive test suites
- Test all ownership patterns
- Verify edge cases
- Implement property-based testing
- Create integration test framework

## Prerequisites

- Completed Section 63 (Performance)
- Understanding of testing strategies
- Familiarity with test frameworks

---

## Test Pyramid

```
        /\
       /  \      E2E Tests (Few)
      /____\
     /      \    Integration Tests (Some)
    /________\
   /          \  Unit Tests (Many)
  /____________\
```

---

## Unit Tests

Test individual functions:

```rust
// borrowscope-runtime/src/tracker.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_track_new_returns_value() {
        reset_tracker();
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        assert_eq!(x, 42);
    }
    
    #[test]
    fn test_track_new_records_event() {
        reset_tracker();
        track_new(1, "x", "i32", "test.rs:1:1", 42);
        
        let events = get_events();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            Event::New { id, name, .. } => {
                assert_eq!(*id, 1);
                assert_eq!(name, "x");
            }
            _ => panic!("Expected New event"),
        }
    }
    
    #[test]
    fn test_unique_ids() {
        reset_tracker();
        track_new(1, "x", "i32", "test.rs:1:1", 42);
        track_new(2, "y", "i32", "test.rs:2:1", 100);
        
        let events = get_events();
        assert_eq!(events.len(), 2);
        
        // IDs should be unique
        let ids: Vec<usize> = events.iter()
            .filter_map(|e| match e {
                Event::New { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        
        assert_eq!(ids, vec![1, 2]);
    }
}
```

---

## Integration Tests

Test macro + runtime together:

```rust
// borrowscope-macro/tests/integration/full_pipeline.rs
use borrowscope_runtime::*;

#[test]
fn test_simple_variable() {
    reset_tracker();
    
    // Simulate macro output
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    track_drop(1, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 2);
    
    // Build graph
    let graph = OwnershipGraph::from_events(&events);
    let stats = graph.statistics();
    
    assert_eq!(stats.total_variables, 1);
}

#[test]
fn test_borrow_lifecycle() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    
    track_drop(2, "scope_end");
    track_drop(1, "scope_end");
    
    let events = get_events();
    let graph = OwnershipGraph::from_events(&events);
    
    // Verify borrow relationship
    let borrows = graph.get_borrows(1);
    assert_eq!(borrows.len(), 1);
}

#[test]
fn test_move_semantics() {
    reset_tracker();
    
    let s1 = track_new(1, "s1", "String", "test.rs:1:1", String::from("hello"));
    track_move(1, 2, "test.rs:2:1");
    let s2 = s1;
    
    track_drop(2, "scope_end");
    
    let events = get_events();
    
    // Should have: New, Move, Drop
    assert_eq!(events.len(), 3);
}
```

---

## Compile Tests

Verify macro transformations compile:

```rust
// borrowscope-macro/tests/compile/pass/simple.rs
use borrowscope_macro::track_ownership;

#[track_ownership]
fn example() {
    let x = 42;
    let y = &x;
}

fn main() {
    example();
}
```

```rust
// borrowscope-macro/tests/compile_test.rs
#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/*.rs");
    t.compile_fail("tests/compile/fail/*.rs");
}
```

---

## Property-Based Testing

Use proptest for random testing:

```rust
// Cargo.toml
[dev-dependencies]
proptest = "1.0"
```

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_track_new_preserves_value(value: i32) {
        reset_tracker();
        let result = track_new(1, "x", "i32", "test.rs:1:1", value);
        prop_assert_eq!(result, value);
    }
    
    #[test]
    fn test_multiple_variables(values: Vec<i32>) {
        reset_tracker();
        
        for (i, value) in values.iter().enumerate() {
            track_new(i, "x", "i32", "test.rs:1:1", *value);
        }
        
        let events = get_events();
        prop_assert_eq!(events.len(), values.len());
    }
    
    #[test]
    fn test_borrow_count(borrow_count in 1..10usize) {
        reset_tracker();
        
        let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
        
        for i in 0..borrow_count {
            track_borrow(i + 2, 1, false, "test.rs:2:1", &x);
        }
        
        let events = get_events();
        let borrow_events = events.iter()
            .filter(|e| matches!(e, Event::Borrow { .. }))
            .count();
        
        prop_assert_eq!(borrow_events, borrow_count);
    }
}
```

---

## Snapshot Testing

Use insta for snapshot tests:

```rust
use insta::assert_snapshot;

#[test]
fn test_json_output() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    track_drop(2, "scope_end");
    track_drop(1, "scope_end");
    
    let json = export_json().unwrap();
    assert_snapshot!(json);
}
```

---

## Fuzzing

Use cargo-fuzz:

```bash
cargo install cargo-fuzz
cargo fuzz init
```

```rust
// fuzz/fuzz_targets/track_operations.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use borrowscope_runtime::*;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    
    reset_tracker();
    
    let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let value = data.get(4).copied().unwrap_or(0) as i32;
    
    // Fuzz tracking operations
    let x = track_new(id, "x", "i32", "fuzz.rs:1:1", value);
    track_drop(id, "scope_end");
    
    // Should not panic
    let _ = get_events();
});
```

---

## Coverage Testing

```bash
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage/index.html
```

**Target:** >80% code coverage

---

## Test Organization

```
borrowscope/
├── borrowscope-runtime/
│   ├── src/
│   │   └── *.rs (with #[cfg(test)] mod tests)
│   ├── tests/
│   │   ├── integration/
│   │   │   ├── mod.rs
│   │   │   ├── simple.rs
│   │   │   ├── borrows.rs
│   │   │   └── moves.rs
│   │   └── *.rs
│   └── benches/
│       └── *.rs
├── borrowscope-macro/
│   ├── src/
│   │   └── *.rs (with #[cfg(test)] mod tests)
│   ├── tests/
│   │   ├── compile/
│   │   │   ├── pass/
│   │   │   └── fail/
│   │   └── *.rs
│   └── examples/
│       └── *.rs
└── tests/
    └── e2e/
        └── *.rs
```

---

## Test Matrix

Test all combinations:

```rust
#[test]
fn test_all_patterns() {
    let patterns = vec![
        "simple variable",
        "tuple destructure",
        "struct destructure",
        "nested pattern",
    ];
    
    let operations = vec![
        "new",
        "borrow",
        "borrow_mut",
        "move",
        "drop",
    ];
    
    for pattern in &patterns {
        for operation in &operations {
            // Test combination
            test_pattern_operation(pattern, operation);
        }
    }
}
```

---

## Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test '*'
      
      - name: Run compile tests
        run: cargo test --package borrowscope-macro compile_test
      
      - name: Check coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Test Checklist

### Runtime Tests
- [ ] track_new returns value unchanged
- [ ] track_borrow preserves reference
- [ ] track_move records transfer
- [ ] track_drop records cleanup
- [ ] Events recorded correctly
- [ ] Timestamps are monotonic
- [ ] IDs are unique
- [ ] Thread safety verified

### Macro Tests
- [ ] Simple variables transformed
- [ ] Borrows detected
- [ ] Moves detected
- [ ] Drops inserted
- [ ] Patterns handled
- [ ] Control flow works
- [ ] Smart pointers detected
- [ ] Compiles successfully

### Integration Tests
- [ ] Macro + runtime work together
- [ ] Graph builds correctly
- [ ] JSON exports properly
- [ ] Visualization data correct
- [ ] Edge cases handled

### Performance Tests
- [ ] Overhead <50ns
- [ ] Memory usage acceptable
- [ ] No memory leaks
- [ ] Scales to large programs

---

## Key Takeaways

✅ **Test at all levels** - Unit, integration, E2E  
✅ **Property-based testing** - Find edge cases  
✅ **Snapshot testing** - Catch regressions  
✅ **Fuzzing** - Find crashes  
✅ **Coverage >80%** - Ensure thorough testing  
✅ **CI/CD** - Automate testing  

---

## Further Reading

- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest](https://github.com/proptest-rs/proptest)
- [insta](https://insta.rs/)
- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)

---

**Previous:** [63-performance-considerations.md](./63-performance-considerations.md)  
**Next:** [65-final-chapter-summary.md](./65-final-chapter-summary.md)

**Progress:** 14/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜
