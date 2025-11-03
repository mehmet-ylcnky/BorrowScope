# Testing Documentation

## Overview

BorrowScope maintains high code quality through comprehensive testing and coverage tracking.

## Test Suites

### Unit Tests
Located in `src/` files with `#[cfg(test)]` modules:
```bash
cargo test --lib --all-features
```

### Integration Tests
Located in `tests/` directories:
```bash
cargo test --test '*' --all-features
```

### Property-Based Tests
Using PropTest and QuickCheck:
```bash
cargo test --test property_based_tests --features track
```

### Performance Tests
Benchmarks and performance regression tests:
```bash
cargo test --test performance_integration_tests --features track
cargo bench --workspace
```

## Coverage

### Generate Coverage Reports

**HTML Report (Interactive)**
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --html --open
```

**JSON Report (CI/CD)**
```bash
cargo llvm-cov --all-features --workspace --json --output-path coverage.json
```

**LCOV Format (Codecov/Coveralls)**
```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Coverage Targets

| Module | Target | Current |
|--------|--------|---------|
| borrowscope-graph | >90% | 94.7% |
| borrowscope-runtime | >85% | TBD |
| borrowscope-macro | >85% | TBD |
| borrowscope-cli | >80% | TBD |

### View Coverage Online

- **Codecov Dashboard**: https://codecov.io/gh/mehmet-ylcnky/BorrowScope
- **HTML Report**: https://mehmet-ylcnky.github.io/BorrowScope/coverage/

## CI/CD Integration

Coverage is automatically tracked on every push and pull request:

1. **Coverage Job**: Runs `cargo llvm-cov` and uploads to Codecov
2. **Coverage Docs**: Deploys HTML report to GitHub Pages (main branch only)
3. **Badge**: README badge updates automatically from Codecov

## Running Tests Locally

### Quick Test
```bash
cargo test --workspace --features track
```

### Full Test Suite
```bash
# All tests with coverage
cargo llvm-cov --all-features --workspace --html

# Single-threaded (for runtime tests)
cargo test --package borrowscope-runtime --features track -- --test-threads=1

# Specific package
cargo test --package borrowscope-graph --all-features
```

### Test Categories

**Fast Tests** (< 1s total)
```bash
cargo test --lib --all-features
```

**Slow Tests** (property-based, integration)
```bash
cargo test --test '*' --all-features
```

**Ignored Tests** (manual only)
```bash
cargo test -- --ignored
```

## Code Quality

### Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Formatting
```bash
cargo fmt --all -- --check
```

### Documentation
```bash
cargo doc --all-features --no-deps --open
```

## Contributing

When adding new features:

1. Write unit tests for core functionality
2. Add integration tests for user-facing APIs
3. Ensure coverage stays above target thresholds
4. Run full test suite before submitting PR
5. Check CI passes all quality gates

## Troubleshooting

**Coverage tool not found**
```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

**Tests hanging**
- Use `--test-threads=1` for runtime tests
- Check for deadlocks in concurrent code

**Coverage report empty**
- Ensure `--all-features` flag is used
- Check that tests actually run (not all ignored)
