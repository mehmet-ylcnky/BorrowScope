<div align="center">
  <img src="logo.png" alt="BorrowScope Logo" width="400"/>
  
  > Visualize Rust's ownership and borrowing at runtime

  [![Crates.io](https://img.shields.io/crates/v/borrowscope-runtime.svg)](https://crates.io/crates/borrowscope-runtime)
  [![CI](https://github.com/mehmet-ylcnky/BorrowScope/actions/workflows/ci.yml/badge.svg)](https://github.com/mehmet-ylcnky/BorrowScope/actions)
  [![codecov](https://codecov.io/gh/mehmet-ylcnky/BorrowScope/branch/main/graph/badge.svg)](https://codecov.io/gh/mehmet-ylcnky/BorrowScope)
  [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
  [![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
  [![Tests](https://img.shields.io/badge/tests-524%20passing-brightgreen.svg)](https://github.com/mehmet-ylcnky/BorrowScope)
  
  📄 [Read the Technical Whitepaper](https://mehmet-ylcnky.github.io/BorrowScope/)
  
  📖 [Read the introductory article on LinkedIn](https://www.linkedin.com/feed/update/urn:li:ugcPost:7407928315648503808/)
</div>

---

BorrowScope is a runtime tracking library for Rust that captures ownership transfers, borrows, and smart pointer operations as they happen. It generates structured event data that can be exported to JSON for analysis, visualization, or debugging.

## Why BorrowScope?

Rust's ownership and borrowing system is one of its most powerful features, but also one of the hardest to learn. The borrow checker operates at compile time, rejecting invalid code with error messages that can be cryptic for newcomers. Even experienced developers sometimes struggle to visualize how ownership flows through complex code paths involving smart pointers, interior mutability, or async boundaries.

BorrowScope was created to bridge this gap. By instrumenting your code with lightweight tracking calls, you can capture every ownership transfer, borrow, and drop as it happens at runtime. The resulting event stream can be exported to JSON for analysis, fed into visualization tools, or simply printed to understand what your code is actually doing. Whether you're learning Rust, teaching others, or debugging a tricky ownership issue in production code, BorrowScope makes the invisible mechanics of Rust's memory model visible.

## Installation

Available on [crates.io](https://crates.io/crates/borrowscope-runtime). Add to your `Cargo.toml`:

```toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }
borrowscope-macro = "0.1"  # Optional: for automatic instrumentation
```

The `track` feature enables runtime tracking. Without it, all tracking functions compile to no-ops with zero overhead.

## Quick Start

### Manual Tracking

```rust
use borrowscope_runtime::*;

fn main() {
    reset(); // Clear any previous tracking data
    
    // Track variable creation
    let data = track_new("data", vec![1, 2, 3]);
    
    // Track borrows
    let r1 = track_borrow("r1", &data);
    let r2 = track_borrow("r2", &data);
    println!("Borrowed: {:?}, {:?}", r1, r2);
    
    // Export events as JSON
    let events = get_events();
    println!("{}", serde_json::to_string_pretty(&events).unwrap());
}
```

### Automatic Instrumentation (Macro)

```rust
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn example() {
    let data = vec![1, 2, 3];  // Automatically tracked
    let r = &data;              // Borrow tracked
    println!("{:?}", r);
}                               // Drops tracked

fn main() {
    reset();
    example();
    print_summary();
}
```

Output:
```
=== BorrowScope Summary ===
Variables: 1 created, 1 dropped
Borrows: 1 immutable, 0 mutable
```

## Tracking Functions

### Basic Ownership

| Function | Description |
|----------|-------------|
| `track_new(name, value)` | Track variable creation |
| `track_borrow(name, ref)` | Track immutable borrow |
| `track_borrow_mut(name, ref)` | Track mutable borrow |
| `track_move(name, value)` | Track ownership transfer |
| `track_drop(name)` | Track variable going out of scope |

### Smart Pointers

| Function | Description |
|----------|-------------|
| `track_rc_new(name, rc)` | Track `Rc<T>` creation |
| `track_rc_clone(name, source, rc)` | Track `Rc<T>` clone |
| `track_arc_new(name, arc)` | Track `Arc<T>` creation |
| `track_arc_clone(name, source, arc)` | Track `Arc<T>` clone |

### Interior Mutability

| Function | Description |
|----------|-------------|
| `track_refcell_new(name, refcell)` | Track `RefCell<T>` creation |
| `track_refcell_borrow(name, guard)` | Track `RefCell::borrow()` |
| `track_refcell_borrow_mut(name, guard)` | Track `RefCell::borrow_mut()` |
| `track_cell_new(name, cell)` | Track `Cell<T>` creation |
| `track_cell_get(name, value)` | Track `Cell::get()` |
| `track_cell_set(name)` | Track `Cell::set()` |

### Unsafe & Advanced

| Function | Description |
|----------|-------------|
| `track_raw_ptr_create(name, ptr)` | Track raw pointer creation |
| `track_raw_ptr_deref(name)` | Track raw pointer dereference |
| `track_unsafe_block_enter(name)` | Track entering unsafe block |
| `track_unsafe_block_exit(name)` | Track exiting unsafe block |
| `track_transmute(name, from, to)` | Track `std::mem::transmute` |
| `track_ffi_call(name, fn_name)` | Track FFI function calls |

### Async

| Function | Description |
|----------|-------------|
| `track_future_create(name)` | Track future creation |
| `track_future_poll(name, state)` | Track future poll |
| `track_async_block_enter(name)` | Track async block entry |
| `track_async_block_exit(name)` | Track async block exit |

All functions have `_with_id` variants that accept a custom identifier for correlation.

## Automatic Instrumentation (borrowscope-macro)

The `#[trace_borrow]` attribute macro automatically instruments functions:

```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]                              // Standard tracking
#[trace_borrow(quiet)]                       // Ownership only
#[trace_borrow(skip = "loops,branches")]     // Skip noisy features
#[trace_borrow(debug_only)]                  // Only in debug builds
```

### Filtering & Sampling

For performance-sensitive code, use filtering and sampling:

```rust
// Only track variables matching pattern (* = any chars, ? = single char)
#[trace_borrow(filter = "user_*")]
fn track_specific_vars() {
    let user_data = vec![1, 2, 3];  // Tracked
    let temp = 42;                   // NOT tracked (doesn't match)
}

// Probabilistic sampling - track ~10% of operations
#[trace_borrow(sample = 0.1)]
fn high_frequency_function() {
    // Reduces overhead in hot paths
}

// Combine for maximum control
#[trace_borrow(debug_only, filter = "important_*", sample = 0.5)]
fn production_ready() { }
```

See [borrowscope-macro examples](borrowscope-macro/examples/) for more.

## Example Projects

The `examples/` directory contains standalone projects demonstrating different aspects of BorrowScope:

| Example | Description |
|---------|-------------|
| [ownership-patterns](examples/ownership-patterns/) | Basic ownership, moves, and borrows |
| [smart-pointers](examples/smart-pointers/) | `Rc`, `Arc`, `RefCell`, and `Cell` tracking |
| [borrow-conflicts](examples/borrow-conflicts/) | Scenarios that would trigger borrow checker errors |
| [async-ownership](examples/async-ownership/) | Ownership across async boundaries |
| [graph-visualization](examples/graph-visualization/) | Exporting tracking data to DOT format |
| [allocator-sim](examples/allocator-sim/) | Advanced patterns: raw pointers, FFI, unions, transmute |

Run any example:
```bash
cd examples/ownership-patterns
cargo run
```

## Project Structure

```
BorrowScope/
├── borrowscope-runtime/     # Core tracking library
│   ├── src/
│   │   ├── tracker.rs      # 125+ tracking functions
│   │   ├── event.rs        # Event types and serialization
│   │   ├── graph.rs        # Graph data structures
│   │   └── export.rs       # JSON export utilities
│   └── tests/              # 290+ tests
│
├── borrowscope-macro/       # Procedural macro for automatic instrumentation
│   ├── src/
│   │   ├── lib.rs          # #[trace_borrow] attribute macro
│   │   ├── config.rs       # Configuration parsing
│   │   └── transform_visitor.rs  # AST transformation
│   └── tests/              # 300+ tests
│
└── examples/               # Standalone example projects
    ├── ownership-patterns/
    ├── smart-pointers/
    ├── borrow-conflicts/
    ├── async-ownership/
    ├── graph-visualization/
    └── allocator-sim/
```

## Performance

With `track` feature enabled:
- ~75-80ns per tracking call
- ~80 bytes per event
- Linear memory scaling O(n)

Without `track` feature:
- Zero overhead - all tracking compiles away

## Testing

```bash
# Run all tests
cargo test -p borrowscope-runtime --features track

# Run specific test file
cargo test -p borrowscope-runtime --features track --test integration_tests
```

## Roadmap

Future development will add:

- **borrowscope-graph** - Graph algorithms for ownership analysis
- **borrowscope-cli** - Command-line tool for analyzing Rust projects
- **borrowscope-ui** - Interactive visualization application

## Related Work

BorrowScope joins a growing ecosystem of tools and resources aimed at making Rust's ownership system more understandable. Here are some notable projects and articles in this space:

| Project | Description |
|---------|-------------|
| [Aquascope](https://github.com/cognitive-engineering-lab/aquascope) | Interactive visualizations of Rust at compile-time and runtime. Developed by Brown University's Cognitive Engineering Lab, it shows permission changes (read/write/own) on variables and visualizes how the borrow checker reasons about code. Powers the visualizations in "The Rust Book Experiment." |
| [Boris](https://github.com/ChristianSchott/boris) | A standalone ownership and borrowing visualizer that renders interactive diagrams showing memory layout, ownership transfers, and borrow scopes. Designed to help beginners understand Rust's memory model through visual exploration. |
| [Flowistry](https://github.com/willcrichton/flowistry) | A VSCode extension using information flow analysis to help developers understand Rust programs. Highlights which code can affect other code, providing a "focus mode" to filter out irrelevant parts when debugging or comprehending complex codebases. |
| [REVIS](https://github.com/weirane/vscode-revis) | A VSCode extension that visualizes lifetime-related Rust compiler errors. Draws lifetime spans directly in the editor to help developers understand and fix borrow checker errors, particularly useful for learning lifetime concepts. |
| [Graphical Depiction of Ownership](https://rufflewind.com/2017-02-15/rust-move-copy-borrow) | A visual guide by Phil Ruffwind depicting move, copy, and borrow semantics through annotated diagrams. Shows how values flow between variables and how borrows create temporary access without transferring ownership. |
| [Think Spatially to Grok Lifetimes](https://www.justanotherdot.com/posts/think-spatially-to-grok-lifetimes.html) | An article presenting a mental model for understanding lifetimes by thinking of programs as nested spaces. Values exist within scopes, and borrows create "bridges" between spaces that must not outlive their source. |
| [Rust Lifetime Visualization Ideas](https://blog.adamant-lang.org/2019/rust-lifetime-visualization-ideas/) | A design exploration of how IDEs could visualize lifetimes inline with code. Proposes compact notations and color-coding schemes that could be integrated into editors without disrupting the coding experience. |

BorrowScope differs from these tools by focusing on runtime event capture rather than static analysis or compile-time visualization. This makes it useful for understanding actual execution behavior, especially in complex scenarios involving smart pointers, interior mutability, or async code.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

<div align="center">
  <strong>Making Rust's ownership system visible, one event at a time.</strong>
</div>
