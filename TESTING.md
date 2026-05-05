# Testing Documentation

## Overview

BorrowScope has 364 tests across 3 crates plus 24 compiled examples and 3 battle tests against real-world crates.

## Test Suites

### Macro Unit Tests (187 tests)
```bash
cargo test -p borrowscope-macro --lib
```

### Runtime Unit Tests (74 tests)
```bash
cargo test -p borrowscope-runtime --features track --lib
```

### Phase Tests (79 tests)
Runtime-based tests covering all 109 semantic patterns:
```bash
cargo test -p borrowscope-runtime --features track \
  --test phase1_method_calls \
  --test phase2_expressions \
  --test phase3_self_borrow \
  --test phase4_unwrap \
  --test phase5_clone \
  -- --test-threads=1
```

### Integration Tests (15 tests)
```bash
cargo test -p borrowscope-runtime --features track \
  --test lifetime_tests \
  --test lifetime_challenges_tests \
  --test advanced_api_with_id_tests
```

### Examples (24 compiled binaries)
```bash
cd examples/macro-examples
cargo run -p borrowscope-analyzer -- .
cargo build  # all 24 binaries
```

### Battle Tests
Real-world crate instrumentation:
```bash
# lru crate — 107 events tracked
cd examples/battle-test && cargo run

# uuid crate — 88 events tracked
cd examples/battle-test-uuid && cargo run

# ripgrep globset — 287 tests pass with instrumented source
```

## Running All Tests

```bash
# Quick (unit tests only)
cargo test -p borrowscope-macro --lib && \
cargo test -p borrowscope-runtime --features track --lib

# Full suite
cargo test -p borrowscope-macro --lib && \
cargo test -p borrowscope-runtime --features track --lib && \
cargo test -p borrowscope-runtime --features track \
  --test phase1_method_calls \
  --test phase2_expressions \
  --test phase3_self_borrow \
  --test phase4_unwrap \
  --test phase5_clone \
  --test lifetime_tests \
  --test lifetime_challenges_tests \
  --test advanced_api_with_id_tests \
  -- --test-threads=1
```

## Test Counts

| Suite | Tests | What It Covers |
|-------|-------|----------------|
| Macro unit | 187 | AST transformation, type_info deserialization, config parsing |
| Runtime unit | 74 | Event recording, tracker state, serialization |
| Phase 1 | 18 | Method call dispatch (Cell, RefCell, Mutex, Channel, etc.) |
| Phase 2 | 30 | Expression tracking (Rc/Arc, Box, Weak, Pin, Cow, transmute) |
| Phase 3 | 12 | Self-borrow inference (immutable, mutable, consuming) |
| Phase 4 | 10 | Unwrap method tracking (5 variants on Option/Result) |
| Phase 5 | 9 | Clone trait verification (generic vs Rc/Arc/Weak) |
| Integration | 15 | Lifetime tracking, advanced API |
| **Total** | **355** | |

## Notes

- Runtime tests use shared global state — run with `--test-threads=1` to avoid flaky failures
- Examples require analyzer to be run first (`cargo run -p borrowscope-analyzer -- .`)
- The analyzer is mandatory — macro panics without `.borrowscope/type-info.json`
