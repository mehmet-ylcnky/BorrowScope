# BorrowScope Runtime

A comprehensive runtime tracking library for visualizing Rust's ownership and borrowing system. BorrowScope Runtime captures ownership transfers, borrows, smart pointer operations, and unsafe code patterns as they happen, generating structured event data for analysis and visualization.

## Why BorrowScope Runtime?

Rust's ownership system operates at compile time, making it invisible during execution. BorrowScope Runtime bridges this gap by instrumenting your code to capture every ownership operation at runtime. Whether you're learning Rust, debugging complex ownership issues, or analyzing memory patterns, this library makes the invisible mechanics of Rust's memory model visible.

## Features

- **Comprehensive Coverage**: 41+ tracking functions covering all Rust ownership patterns
- **Zero-Cost Abstraction**: Complete compile-time elimination when `track` feature is disabled
- **Thread Safety**: All operations are thread-safe with efficient synchronization
- **RAII Guards**: Automatic drop tracking with guard types
- **Event Sourcing**: Store events and build ownership graphs on demand
- **Rich Analysis**: Lifetime analysis, timeline construction, and graph statistics
- **JSON Export**: Export tracking data for visualization tools
- **Performance**: ~75-80ns per tracking call, ~80 bytes per event

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }
```

Basic usage:

```rust
use borrowscope_runtime::*;

fn main() {
    reset(); // Clear any previous tracking data
    
    // Track variable creation
    let data = track_new("data", vec![1, 2, 3]);
    
    // Track borrowing
    let r1 = track_borrow("r1", &data);
    let r2 = track_borrow("r2", &data);
    println!("Borrowed: {:?}, {:?}", r1, r2);
    
    // Track drops
    track_drop("r2");
    track_drop("r1");
    track_drop("data");
    
    // Export events as JSON
    let events = get_events();
    println!("{}", serde_json::to_string_pretty(&events).unwrap());
    
    // Build ownership graph
    let graph = get_graph();
    println!("Graph: {} variables, {} relationships", 
             graph.nodes.len(), graph.edges.len());
}
```

## Comprehensive API

### Basic Ownership Tracking

Track fundamental ownership operations:

```rust
// Variable creation and destruction
let x = track_new("x", 42);                    // Track variable creation
let y = track_move("y", x);                    // Track ownership transfer
track_drop("y");                               // Track variable drop

// Borrowing operations
let data = track_new("data", vec![1, 2, 3]);
let r = track_borrow("r", &data);              // Track immutable borrow
let r_mut = track_borrow_mut("r_mut", &mut data); // Track mutable borrow
```

### RAII Guards (Automatic Drop Tracking)

Eliminate manual drop tracking with RAII guards:

```rust
{
    let data = track_new_guard("data", vec![1, 2, 3]);
    let r = track_borrow_guard("r", &*data);
    let r_mut = track_borrow_mut_guard("r_mut", &mut *data);
    // track_drop called automatically when guards go out of scope
}

// Guards support transparent access via Deref/DerefMut
let guard = track_new_guard("x", 42);
println!("{}", *guard); // Access inner value
```

### Smart Pointer Tracking

Track reference-counted smart pointers with precise count monitoring:

```rust
use std::rc::Rc;
use std::sync::Arc;

// Rc<T> tracking
let rc1 = track_rc_new("rc1", Rc::new(42));
let rc2 = track_rc_clone("rc2", "rc1", rc1.clone());
// Automatically tracks strong_count and weak_count

// Arc<T> tracking (thread-safe)
let arc1 = track_arc_new("arc1", Arc::new(vec![1, 2, 3]));
let arc2 = track_arc_clone("arc2", "arc1", arc1.clone());
```

### Interior Mutability Tracking

Monitor `RefCell` and `Cell` operations with runtime borrow checking:

```rust
use std::cell::{RefCell, Cell};

// RefCell tracking
let cell = track_refcell_new("cell", RefCell::new(42));
let borrow = refcell_borrow!("borrow", "cell", cell.borrow());
let borrow_mut = refcell_borrow_mut!("borrow_mut", "cell", cell.borrow_mut());
refcell_drop!("cell");

// Cell tracking
let cell = track_cell_new("cell", Cell::new(10));
let value = track_cell_get("cell", cell.get());
track_cell_set("cell");
```

### Unsafe Code Tracking

Comprehensive tracking for unsafe operations:

```rust
// Raw pointer operations
let x = 42;
let ptr = track_raw_ptr("ptr", &x as *const i32);
let ptr_mut = track_raw_ptr_mut("ptr_mut", &mut x as *mut i32);
unsafe {
    track_raw_ptr_deref("ptr");
    let value = *ptr;
}

// Unsafe block tracking
unsafe {
    track_unsafe_block_enter("block1");
    // unsafe operations
    track_unsafe_block_exit("block1");
}

// FFI and transmute tracking
track_ffi_call("libc_malloc");
track_transmute("u32_to_f32", "u32", "f32");
track_unsafe_fn_call("dangerous_function");
track_union_field_access("MyUnion", "field1");
```

### Static and Const Tracking

Monitor global variable access patterns:

```rust
// Static variable tracking
static mut COUNTER: i32 = 0;
track_static_init("COUNTER", "COUNTER_id", "i32", true);
track_static_access("COUNTER_id", "COUNTER", true, "main.rs:10");

// Const evaluation tracking
const PI: f64 = 3.14159;
track_const_eval("PI", "PI_id", "f64", "main.rs:5");
```

### Advanced API Features

#### Custom ID Correlation

All tracking functions have `_with_id` variants for custom correlation:

```rust
let custom_id = "user_defined_123";
let x = track_new_with_id("x", custom_id, 42);
let r = track_borrow_with_id("r", "r_id", custom_id, &x);
```

#### Event Querying and Filtering

Rich querying capabilities for event analysis:

```rust
// Get all events or filtered subsets
let all_events = get_events();
let new_events = get_new_events();
let borrow_events = get_borrow_events();
let move_events = get_move_events();
let drop_events = get_drop_events();

// Filter events by variable or criteria
let var_events = get_events_for_var("data");
let filtered = get_events_filtered(|event| event.is_unsafe());

// Get event statistics
let counts = get_event_counts();
let summary = get_summary();
print_summary();
```

#### Batch Operations

Efficient batch processing for performance:

```rust
let var_names = vec!["x", "y", "z"];
track_drop_batch(&var_names);
```

### Ownership Graph Analysis

Build and analyze ownership relationships:

```rust
let graph = get_graph();

// Graph statistics
let stats = graph.stats();
println!("Variables: {}, Relationships: {}", 
         stats.total_variables, stats.total_relationships);

// Find specific variables and relationships
let var = graph.find_variable("data");
let borrows = graph.find_borrows("data");

// Analyze ownership patterns
for relationship in &graph.edges {
    match relationship {
        Relationship::BorrowsImmut { from, to, start, end } => {
            println!("{} borrows {} from {} to {}", from, to, start, end);
        }
        Relationship::BorrowsMut { from, to, start, end } => {
            println!("{} mutably borrows {} from {} to {}", from, to, start, end);
        }
        Relationship::Owns { from, to } => {
            println!("{} owns {}", from, to);
        }
    }
}
```

### Lifetime Analysis

Advanced lifetime relationship analysis:

```rust
// Build timeline from events
let timeline = Timeline::from_events(&get_events());

// Analyze lifetime relationships
let relations = timeline.analyze_lifetimes();
for relation in relations {
    match relation {
        LifetimeRelation::Contains { outer, inner } => {
            println!("Lifetime {} contains {}", outer, inner);
        }
        LifetimeRelation::Overlaps { first, second } => {
            println!("Lifetimes {} and {} overlap", first, second);
        }
    }
}

// Detect elision rules
let elision_rules = timeline.detect_elision_rules();
```

### Export and Visualization

Export tracking data for external analysis:

```rust
// Export to JSON file
export_json("ownership_analysis.json").unwrap();

// Manual export with custom data
let events = get_events();
let graph = get_graph();
let export_data = ExportData::new(graph, events);
export_data.to_file("custom_export.json").unwrap();
```

## Event Types

BorrowScope Runtime tracks 20+ event types covering all ownership patterns:

| Category | Events |
|----------|--------|
| **Basic Ownership** | `New`, `Borrow`, `Move`, `Drop` |
| **Smart Pointers** | `RcNew`, `RcClone`, `ArcNew`, `ArcClone` |
| **Interior Mutability** | `RefCellNew`, `RefCellBorrow`, `RefCellDrop`, `CellNew`, `CellGet`, `CellSet` |
| **Static/Const** | `StaticInit`, `StaticAccess`, `ConstEval` |
| **Unsafe Operations** | `RawPtrCreated`, `RawPtrDeref`, `UnsafeBlockEnter`, `UnsafeBlockExit`, `UnsafeFnCall` |
| **FFI/Transmute** | `FfiCall`, `Transmute`, `UnionFieldAccess` |

All events include timestamps and are serializable to JSON for analysis.

## Architecture

BorrowScope Runtime uses an event sourcing architecture:

1. **Event Recording**: Track operations as timestamped events
2. **Thread-Safe Storage**: Store events in a global, thread-safe tracker
3. **On-Demand Analysis**: Build ownership graphs and timelines from event streams
4. **Export Pipeline**: Serialize data to JSON for visualization tools

Key components:
- **Tracker**: Global event recorder with atomic timestamp generation
- **Event System**: Comprehensive event types with JSON serialization
- **Graph Builder**: Constructs ownership graphs from event streams
- **RAII Guards**: Automatic resource tracking with scope-based cleanup
- **Lifetime Analyzer**: Timeline construction and relationship analysis

## Performance

Optimized for minimal runtime overhead:

- **Single tracking call**: ~75-80ns
- **1000 operations**: ~150μs
- **JSON export (1000 events)**: ~1ms
- **Memory per event**: ~80 bytes
- **Zero cost when disabled**: Complete compile-time elimination

Thread safety achieved through:
- `parking_lot::Mutex` for efficient locking (40-60% faster than std)
- `AtomicU64` for lock-free timestamp generation
- Event sourcing to avoid complex concurrent graph updates

## Feature Flags

- `track` - Enables runtime tracking. Without this feature, all tracking functions compile to no-ops with zero overhead.

```toml
# Development/debugging (with tracking)
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }

# Production (zero overhead)
[dependencies]
borrowscope-runtime = "0.1"
```

## Testing

Comprehensive test suite with 555+ tests:

```bash
# Run all tests
cargo test --package borrowscope-runtime --features track

# Run specific test categories
cargo test --package borrowscope-runtime --features track --test integration_tests
cargo test --package borrowscope-runtime --features track --test performance_tests
cargo test --package borrowscope-runtime --features track --test unsafe_code_tests

# Run benchmarks
cargo bench --package borrowscope-runtime
```

## Error Handling

Robust error handling with comprehensive error types:

```rust
use borrowscope_runtime::{Result, Error};

match export_json("output.json") {
    Ok(()) => println!("Export successful"),
    Err(Error::SerializationError(e)) => eprintln!("JSON error: {}", e),
    Err(Error::IoError(e)) => eprintln!("File error: {}", e),
    Err(Error::ExportError(msg)) => eprintln!("Export failed: {}", msg),
    Err(Error::InvalidEventSequence(msg)) => eprintln!("Invalid events: {}", msg),
    Err(Error::LockError(msg)) => eprintln!("Lock error: {}", msg),
}
```

## Documentation

Generate and view complete API documentation:

```bash
cargo doc --package borrowscope-runtime --features track --open
```

## License

Licensed under the Apache License, Version 2.0. See the main BorrowScope repository for full license information.
