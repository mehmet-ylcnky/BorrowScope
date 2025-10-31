# Performance Baselines

This document tracks expected performance characteristics for the BorrowScope runtime.

## Single Operations

| Operation | Target | Typical |
|-----------|--------|---------|
| track_new | <100ns | ~80ns |
| track_borrow | <100ns | ~75ns |
| track_borrow_mut | <100ns | ~75ns |
| track_move | <50ns | ~11ns |
| track_drop | <50ns | ~34ns |

## Bulk Operations

| Size | Operation | Target | Typical |
|------|-----------|--------|---------|
| 100 | track_new | <20μs | ~14μs |
| 1000 | track_new | <200μs | ~150μs |
| 10000 | track_new | <2ms | ~1.5ms |
| 100 | mixed_operations | <40μs | ~29μs |
| 1000 | mixed_operations | <400μs | ~290μs |
| 10000 | mixed_operations | <4ms | ~2.9ms |

## Graph Building

| Events | Target | Typical |
|--------|--------|---------|
| 100 | <100μs | ~50μs |
| 1000 | <1ms | ~500μs |
| 5000 | <5ms | ~2.5ms |

## JSON Export

| Events | Target | Typical |
|--------|--------|---------|
| 100 | <200μs | ~100μs |
| 1000 | <2ms | ~1ms |
| 5000 | <10ms | ~5ms |

## Concurrent Operations

| Scenario | Target | Typical |
|----------|--------|---------|
| 4 threads × 100 ops | <500μs | ~300μs |

## Memory Overhead

| Metric | Value |
|--------|-------|
| Event size | ~80 bytes |
| 1000 events clone | <50μs |

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --package borrowscope-runtime

# Run specific benchmark group
cargo bench --package borrowscope-runtime --bench performance -- single_operations

# Save baseline
cargo bench --package borrowscope-runtime -- --save-baseline main

# Compare against baseline
cargo bench --package borrowscope-runtime -- --baseline main
```

## Performance Goals

- **Zero-cost abstractions**: Tracking functions are inlined and add minimal overhead
- **Thread safety**: All operations are thread-safe with efficient synchronization
- **Scalability**: Linear performance scaling with event count
- **Memory efficiency**: <1KB per 1000 events

## Notes

- Benchmarks run in release mode with optimizations enabled
- Results may vary based on hardware and system load
- "Performance has regressed" warnings are relative to previous runs
- Use `--save-baseline` to establish reference points for comparison
