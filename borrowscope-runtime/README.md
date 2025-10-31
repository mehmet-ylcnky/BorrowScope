# BorrowScope Runtime

The runtime tracking system for BorrowScope that records ownership and borrowing events during program execution.

## Features

- **Zero-cost abstractions**: Tracking functions are inlined and return values unchanged
- **Type safety**: Generic functions work with any type without boxing
- **Thread safety**: All operations are thread-safe using efficient synchronization
- **Event sourcing**: Store events and build graphs on demand
- **JSON export**: Export tracking data for visualization

## Usage

```rust
use borrowscope_runtime::*;

// Track variable creation
let x = track_new("x", 42);

// Track borrowing
let r = track_borrow("r", &x);

// Track drops
track_drop("r");
track_drop("x");

// Get events
let events = get_events();
println!("Tracked {} events", events.len());

// Build ownership graph
let graph = get_graph();
println!("Graph has {} variables", graph.nodes.len());

// Export to JSON
export_json("output.json").unwrap();
```

## API Overview

### Tracking Functions

- `track_new(name, value)` - Track variable creation
- `track_borrow(name, value)` - Track immutable borrow
- `track_borrow_mut(name, value)` - Track mutable borrow
- `track_move(from, to, value)` - Track ownership move
- `track_drop(name)` - Track variable drop

### Query Functions

- `get_events()` - Get all tracked events
- `get_graph()` - Build ownership graph from events
- `reset()` - Clear all tracking data

### Export Functions

- `export_json(path)` - Export to JSON file

## Architecture

The runtime uses an event sourcing pattern:

1. **Track operations as events** (New, Borrow, Move, Drop)
2. **Store events** in a thread-safe global tracker
3. **Build graphs** from event streams on demand
4. **Export data** to JSON for visualization

## Performance

- Single operation: ~75ns
- 1000 operations: ~150μs
- JSON export (1000 events): ~1ms
- Memory: ~80 bytes per event

See [BASELINE.md](benches/BASELINE.md) for detailed performance metrics.

## Testing

```bash
# Run all tests (single-threaded due to global state)
cargo test --package borrowscope-runtime -- --test-threads=1

# Run benchmarks
cargo bench --package borrowscope-runtime
```

## Documentation

```bash
# Generate and open documentation
cargo doc --package borrowscope-runtime --open
```

## Error Handling

The runtime uses a custom `Result<T>` type with comprehensive error variants:

- `SerializationError` - JSON serialization failed
- `IoError` - File I/O error
- `ExportError` - Export operation failed
- `InvalidEventSequence` - Invalid event data
- `LockError` - Lock acquisition failed

## Thread Safety

All tracking operations are thread-safe. The runtime uses:

- `parking_lot::Mutex` for efficient locking (40-60% faster than std)
- `AtomicU64` for lock-free timestamp generation
- Event sourcing to avoid complex concurrent graph updates

## License

See the main BorrowScope repository for license information.
