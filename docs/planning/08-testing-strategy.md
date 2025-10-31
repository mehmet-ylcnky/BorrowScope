# 8. Testing Strategy

## Testing Pyramid

```
        /\
       /  \
      / E2E\         10% - End-to-End Tests
     /______\
    /        \
   /Integration\    30% - Integration Tests
  /____________\
 /              \
/   Unit Tests   \  60% - Unit Tests
/__________________\
```

---

## Unit Tests

### borrowscope-macro

**Test Categories:**

#### 1. AST Parsing Tests
```rust
#[test]
fn test_parse_simple_function() {
    let input = quote! {
        #[trace_borrow]
        fn example() {
            let x = 5;
        }
    };
    let result = parse_trace_borrow(input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_with_parameters() {
    let input = quote! {
        #[trace_borrow]
        fn example(x: i32, y: &str) {
            let z = x + 1;
        }
    };
    let result = parse_trace_borrow(input);
    assert!(result.is_ok());
}
```

#### 2. Code Generation Tests
```rust
#[test]
fn test_inject_track_new() {
    let input = quote! { let s = String::from("hello"); };
    let output = inject_tracking(input);
    assert!(output.to_string().contains("track_new"));
}

#[test]
fn test_inject_track_borrow() {
    let input = quote! { let r = &s; };
    let output = inject_tracking(input);
    assert!(output.to_string().contains("track_borrow"));
}
```

#### 3. Compile Tests with `trybuild`
```rust
#[test]
fn test_macro_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
}
```

**Test Files:**
```
tests/ui/
├── pass/
│   ├── simple_borrow.rs
│   ├── mutable_borrow.rs
│   ├── multiple_borrows.rs
│   └── nested_scopes.rs
└── fail/
    ├── invalid_syntax.rs
    └── unsupported_pattern.rs
```

**Coverage Target:** 85%

---

### borrowscope-runtime

**Test Categories:**

#### 1. Event Tracking Tests
```rust
#[test]
fn test_track_new() {
    reset();
    let s = track_new("s", String::from("hello"));
    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::New { .. }));
}

#[test]
fn test_track_borrow() {
    reset();
    let s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[1], Event::Borrow { mutable: false, .. }));
}

#[test]
fn test_track_borrow_mut() {
    reset();
    let mut s = track_new("s", String::from("hello"));
    let r = track_borrow_mut("r", &mut s);
    let events = get_events();
    assert!(matches!(events[1], Event::Borrow { mutable: true, .. }));
}
```

#### 2. Graph Building Tests
```rust
#[test]
fn test_graph_construction() {
    reset();
    let s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    track_drop("r");
    track_drop("s");
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_multiple_borrows() {
    reset();
    let s = track_new("s", String::from("hello"));
    let r1 = track_borrow("r1", &s);
    let r2 = track_borrow("r2", &s);
    
    let graph = get_graph();
    assert_eq!(graph.edges.len(), 2);
}
```

#### 3. Serialization Tests
```rust
#[test]
fn test_json_export() {
    reset();
    let s = track_new("s", String::from("hello"));
    track_drop("s");
    
    let json = export_json_string().unwrap();
    let parsed: BorrowScopeOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.events.len(), 2);
}
```

#### 4. Thread Safety Tests
```rust
#[test]
fn test_concurrent_tracking() {
    use std::thread;
    reset();
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let s = track_new(&format!("s{}", i), String::from("test"));
                track_drop(&format!("s{}", i));
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let events = get_events();
    assert_eq!(events.len(), 20); // 10 new + 10 drop
}
```

**Coverage Target:** 90%

---

### borrowscope-cli

**Test Categories:**

#### 1. Command Parsing Tests
```rust
#[test]
fn test_visualize_command() {
    let args = vec!["borrowscope", "visualize", "src/main.rs"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.command, Command::Visualize { .. }));
}

#[test]
fn test_export_command_with_options() {
    let args = vec!["borrowscope", "export", "--output", "graph.json"];
    let cli = Cli::parse_from(args);
    assert!(matches!(cli.command, Command::Export { .. }));
}
```

#### 2. File Instrumentation Tests
```rust
#[test]
fn test_instrument_simple_file() {
    let input = r#"
        fn example() {
            let s = String::from("hello");
        }
    "#;
    let output = instrument_code(input).unwrap();
    assert!(output.contains("#[trace_borrow]"));
}

#[test]
fn test_preserve_existing_attributes() {
    let input = r#"
        #[inline]
        fn example() {
            let s = String::from("hello");
        }
    "#;
    let output = instrument_code(input).unwrap();
    assert!(output.contains("#[inline]"));
    assert!(output.contains("#[trace_borrow]"));
}
```

#### 3. Config File Tests
```rust
#[test]
fn test_load_config() {
    let config_str = r#"
        [instrumentation]
        auto_instrument = true
        exclude_functions = ["main"]
    "#;
    let config: Config = toml::from_str(config_str).unwrap();
    assert!(config.instrumentation.auto_instrument);
    assert_eq!(config.instrumentation.exclude_functions, vec!["main"]);
}
```

**Coverage Target:** 80%

---

### borrowscope-ui

**Test Categories:**

#### 1. Data Parsing Tests (JavaScript)
```javascript
describe('JSON Parser', () => {
  test('parses valid BorrowScope output', () => {
    const json = {
      version: "0.1.0",
      events: [{ type: "New", timestamp: 1, var_name: "s" }],
      graph: { nodes: [], edges: [] }
    };
    const result = parseBorrowScopeData(json);
    expect(result.events.length).toBe(1);
  });

  test('handles invalid JSON gracefully', () => {
    const invalid = { invalid: "data" };
    expect(() => parseBorrowScopeData(invalid)).toThrow();
  });
});
```

#### 2. Graph Rendering Tests
```javascript
describe('Graph Renderer', () => {
  test('creates nodes for variables', () => {
    const graph = { nodes: [{ id: "s_1", name: "s" }], edges: [] };
    const cy = renderGraph(graph);
    expect(cy.nodes().length).toBe(1);
  });

  test('creates edges for borrows', () => {
    const graph = {
      nodes: [{ id: "s_1" }, { id: "r_2" }],
      edges: [{ from_id: "r_2", to_id: "s_1", kind: "BorrowsImmut" }]
    };
    const cy = renderGraph(graph);
    expect(cy.edges().length).toBe(1);
  });
});
```

#### 3. Tauri Command Tests (Rust)
```rust
#[test]
fn test_load_graph_data() {
    let temp_file = create_temp_json();
    let result = load_graph_data(temp_file.path().to_str().unwrap().to_string());
    assert!(result.is_ok());
}
```

**Coverage Target:** 75%

---

## Integration Tests

### End-to-End Macro + Runtime Tests

```rust
#[test]
fn test_simple_ownership() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let r = &s;
        println!("{}", r);
    }
    
    reset();
    example();
    
    let events = get_events();
    assert_eq!(events.len(), 4); // new, borrow, drop r, drop s
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_mutable_borrow() {
    #[trace_borrow]
    fn example() {
        let mut s = String::from("hello");
        let r = &mut s;
        r.push_str(" world");
    }
    
    reset();
    example();
    
    let events = get_events();
    let borrow_event = events.iter()
        .find(|e| matches!(e, Event::Borrow { .. }))
        .unwrap();
    
    if let Event::Borrow { mutable, .. } = borrow_event {
        assert!(mutable);
    }
}

#[test]
fn test_move_semantics() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let t = s; // move
        println!("{}", t);
    }
    
    reset();
    example();
    
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Move { .. })));
}
```

### CLI Integration Tests

```rust
#[test]
fn test_cli_visualize_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    fs::write(&test_file, r#"
        fn example() {
            let s = String::from("hello");
            let r = &s;
        }
    "#).unwrap();
    
    let output = Command::new("cargo")
        .args(&["borrowscope", "visualize", test_file.to_str().unwrap(), "--no-ui"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let json_file = temp_dir.path().join("borrowscope-output.json");
    assert!(json_file.exists());
}

#[test]
fn test_cli_export_command() {
    let output = Command::new("cargo")
        .args(&["borrowscope", "export", "--output", "test.json"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    assert!(Path::new("test.json").exists());
}
```

**Coverage Target:** 70%

---

## End-to-End Tests

### Real-World Scenarios

#### Test Case 1: Simple Ownership
```rust
// tests/e2e/simple_ownership.rs
#[trace_borrow]
fn test_simple() {
    let s = String::from("hello");
    println!("{}", s);
}

#[test]
fn verify_simple_ownership() {
    reset();
    test_simple();
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);
    
    let json = export_json_string().unwrap();
    assert!(json.contains("\"var_name\":\"s\""));
}
```

#### Test Case 2: Multiple Borrows
```rust
#[trace_borrow]
fn test_multiple_borrows() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
}

#[test]
fn verify_multiple_borrows() {
    reset();
    test_multiple_borrows();
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 3); // s, r1, r2
    assert_eq!(graph.edges.len(), 2); // r1->s, r2->s
}
```

#### Test Case 3: Nested Scopes
```rust
#[trace_borrow]
fn test_nested_scopes() {
    let s = String::from("hello");
    {
        let r = &s;
        println!("{}", r);
    }
    println!("{}", s);
}

#[test]
fn verify_nested_scopes() {
    reset();
    test_nested_scopes();
    
    let events = get_events();
    let drop_r = events.iter().position(|e| {
        matches!(e, Event::Drop { var_id, .. } if var_id.starts_with("r"))
    }).unwrap();
    let drop_s = events.iter().position(|e| {
        matches!(e, Event::Drop { var_id, .. } if var_id.starts_with("s"))
    }).unwrap();
    
    assert!(drop_r < drop_s); // r dropped before s
}
```

#### Test Case 4: Closures
```rust
#[trace_borrow]
fn test_closure() {
    let s = String::from("hello");
    let closure = || println!("{}", s);
    closure();
}

#[test]
fn verify_closure_capture() {
    reset();
    test_closure();
    
    let events = get_events();
    // Verify closure captures s
    assert!(events.iter().any(|e| matches!(e, Event::Borrow { .. })));
}
```

**Coverage Target:** 50%

---

## Performance Tests

### Benchmarks with Criterion

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_track_new(c: &mut Criterion) {
    c.bench_function("track_new", |b| {
        b.iter(|| {
            reset();
            track_new(black_box("s"), black_box(String::from("hello")))
        });
    });
}

fn bench_track_borrow(c: &mut Criterion) {
    let s = String::from("hello");
    c.bench_function("track_borrow", |b| {
        b.iter(|| {
            track_borrow(black_box("r"), black_box(&s))
        });
    });
}

fn bench_graph_construction(c: &mut Criterion) {
    c.bench_function("graph_construction_100_vars", |b| {
        b.iter(|| {
            reset();
            for i in 0..100 {
                let s = track_new(&format!("s{}", i), String::from("test"));
                track_drop(&format!("s{}", i));
            }
            get_graph()
        });
    });
}

criterion_group!(benches, bench_track_new, bench_track_borrow, bench_graph_construction);
criterion_main!(benches);
```

**Performance Targets:**
- `track_new()`: <100ns per call
- `track_borrow()`: <50ns per call
- Graph construction (100 vars): <1ms
- JSON export (1000 events): <10ms

---

## Property-Based Tests

### Using Proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_any_string_trackable(s in "\\PC*") {
        reset();
        let tracked = track_new("test", s.clone());
        assert_eq!(tracked, s);
        let events = get_events();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_multiple_borrows_valid(count in 1..100usize) {
        reset();
        let s = track_new("s", String::from("test"));
        for i in 0..count {
            track_borrow(&format!("r{}", i), &s);
        }
        let graph = get_graph();
        assert_eq!(graph.edges.len(), count);
    }
}
```

---

## Test Data & Fixtures

### Example Test Programs

```
tests/fixtures/
├── simple/
│   ├── ownership.rs
│   ├── borrowing.rs
│   └── moves.rs
├── intermediate/
│   ├── nested_scopes.rs
│   ├── multiple_borrows.rs
│   └── mutable_borrows.rs
└── advanced/
    ├── closures.rs
    ├── async_await.rs
    └── smart_pointers.rs
```

### Expected Output Files

```
tests/expected/
├── simple_ownership.json
├── multiple_borrows.json
└── nested_scopes.json
```

### Snapshot Testing with Insta

```rust
#[test]
fn test_json_output_snapshot() {
    reset();
    let s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    track_drop("r");
    track_drop("s");
    
    let json = export_json_string().unwrap();
    insta::assert_json_snapshot!(json);
}
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
```

---

## Test Execution Strategy

### Local Development
```bash
# Run all tests
cargo test --workspace

# Run specific component tests
cargo test -p borrowscope-macro
cargo test -p borrowscope-runtime

# Run with coverage
cargo tarpaulin --workspace

# Run benchmarks
cargo bench
```

### Pre-commit Checks
```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --all-targets --all-features

# Tests
cargo test --all-features

# Documentation
cargo doc --no-deps
```

### Release Checklist
- [ ] All tests pass on all platforms
- [ ] Coverage >80%
- [ ] Benchmarks meet targets
- [ ] No clippy warnings
- [ ] Documentation builds
- [ ] Examples run successfully
- [ ] E2E tests pass

---

## Quality Gates

### Minimum Requirements for Merge
- ✅ All unit tests pass
- ✅ No clippy warnings
- ✅ Code formatted with rustfmt
- ✅ Coverage doesn't decrease
- ✅ New features have tests

### Minimum Requirements for Release
- ✅ All tests pass on Linux/macOS/Windows
- ✅ Overall coverage >80%
- ✅ Performance benchmarks meet targets
- ✅ E2E tests pass
- ✅ Documentation complete
- ✅ Security audit clean
