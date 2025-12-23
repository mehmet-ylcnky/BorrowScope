# borrowscope-macro Roadmap

## Overview

This document outlines planned improvements for `borrowscope-macro`, prioritized by impact and implementation complexity. The type information barrier (see [LIMITATIONS.md](LIMITATIONS.md)) is a fundamental constraint that cannot be overcome within the macro itself.

## Priority Levels

- **P0**: Critical - High impact, should be done first
- **P1**: Important - Significant value, do after P0
- **P2**: Nice to have - Lower priority, do when time permits

---

## P0: Critical Improvements

### 1. Annotation-Based Disambiguation

**Status**: Not started  
**Complexity**: Medium  
**Impact**: High - Addresses the type information barrier workaround

Add attribute parameters to declare type-dependent patterns:

```rust
#[trace_borrow(
    ffi = ["c_read", "c_write", "external_process"],
    unions = ["RawData", "PackedValue"],
    statics = ["GLOBAL_CONFIG", "SHARED_STATE"]
)]
fn process_data() {
    c_read(ptr);           // → track_ffi_call()
    let x = data.int_val;  // → track_union_field_access()
    GLOBAL_CONFIG.load();  // → track_static_access()
}
```

**Tasks**:
- [ ] Extend `TraceConfig` to hold FFI/union/static lists
- [ ] Parse new attribute parameters in `TraceArgs`
- [ ] Match function calls against FFI list
- [ ] Match field access base types against union list
- [ ] Match identifiers against static list
- [ ] Generate appropriate tracking calls

### 2. Conditional Compilation Support

**Status**: ✅ Complete  
**Complexity**: Low  
**Impact**: High - Zero overhead in release builds

```rust
#[trace_borrow(debug_only)]  // Only instrument in debug builds
fn hot_path() { ... }

#[trace_borrow(release_only)]  // Only instrument in release builds
fn release_tracking() { ... }

#[trace_borrow(feature = "tracing")]  // Only when feature enabled
fn optional_tracking() { ... }

// Combine with other options
#[trace_borrow(debug_only, skip(loops, branches))]
fn combined() { ... }
```

**Tasks**:
- [x] Add `ConditionalMode` enum to config (Always, DebugOnly, ReleaseOnly, Feature)
- [x] Parse `debug_only`, `release_only`, `feature = "name"` in `TraceArgs`
- [x] Generate both instrumented and non-instrumented versions with `#[cfg(...)]`
- [x] Add unit tests for conditional mode parsing and cfg token generation
- [x] Support combining conditional modes with other options (skip, only)
- [x] Support parentheses syntax: `skip(loops, branches)` in addition to `skip = "loops, branches"`
- [x] Add example: `examples/conditional_compilation.rs`

### 3. Better Error Messages

**Status**: ✅ Complete  
**Complexity**: Medium  
**Impact**: High - Developer experience

Compile-time warnings for ambiguous patterns:

```rust
#[trace_borrow(warn)]
fn example() {
    c_read(ptr);        // Warning: cannot determine if this is an FFI call
                        // Hint: use #[trace_borrow(ffi = ["c_read"])] to track as FFI

    let x = GLOBAL_VAR; // Warning: cannot determine if this is a static variable
                        // Hint: use #[trace_borrow(statics = ["GLOBAL_VAR"])]
}

// Suppress warnings for known patterns
#[trace_borrow(warn, ffi = ["c_read"], statics = ["GLOBAL_VAR"])]
fn with_declarations() {
    c_read(ptr);        // No warning - declared as known FFI
    let x = GLOBAL_VAR; // No warning - declared as known static
}
```

**Tasks**:
- [x] Add `warn` / `warn_ambiguous` attribute option
- [x] Add `ffi = [...]`, `unions = [...]`, `statics = [...]` for known patterns
- [x] Emit warnings for potential FFI calls (c_*, malloc, etc.)
- [x] Emit warnings for potential static variables (SCREAMING_SNAKE_CASE)
- [x] Emit warnings for potential union field access
- [x] Emit warnings for transmute (type info unavailable)
- [x] Use `proc-macro-warning` crate for stable Rust support
- [x] Proper span locations for all warnings
- [x] Add example: `examples/better_error_messages.rs`
- [x] Add unit tests for diagnostics module

---

## P1: Important Improvements

### 4. Reduced Code Generation

**Status**: Not started  
**Complexity**: Medium  
**Impact**: Medium - Faster compile times, smaller binaries

Current generation is verbose. Optimize by:
- Combining sequential tracking calls
- Using helper macros instead of inline code
- Eliminating redundant type annotations

**Tasks**:
- [ ] Audit generated code size
- [ ] Create `borrowscope_runtime::__internal` helper macros
- [ ] Batch sequential tracking calls
- [ ] Benchmark compile time improvement

### 5. Closure Capture Tracking

**Status**: Partial  
**Complexity**: High  
**Impact**: Medium - Important for async code

Better tracking of what closures capture:

```rust
let x = String::new();
let closure = move || {
    // Track: closure captures 'x' by move
    println!("{}", x);
};
```

**Tasks**:
- [ ] Detect `move` keyword on closures
- [ ] Analyze closure body for captured variables
- [ ] Correlate captures with outer scope variables
- [ ] Generate `track_closure_capture()` with capture mode

### 6. Filter & Sampling

**Status**: Not started  
**Complexity**: Low  
**Impact**: Medium - Performance in production

```rust
#[trace_borrow(filter = "name:data*")]  // Only track vars starting with "data"
#[trace_borrow(sample = 0.01)]          // Track 1% of calls (random)
```

**Tasks**:
- [ ] Add filter pattern to config
- [ ] Add sample rate to config
- [ ] Generate conditional tracking based on filter
- [ ] Generate random sampling logic

### 7. Integration with `tracing` Crate

**Status**: Not started  
**Complexity**: Medium  
**Impact**: Medium - Ecosystem integration

```rust
#[trace_borrow(backend = "tracing")]  // Use tracing instead of borrowscope-runtime
fn example() { ... }
```

**Tasks**:
- [ ] Define trait for tracking backend
- [ ] Implement `tracing` backend
- [ ] Make backend configurable via attribute

---

## P2: Nice to Have

### 8. IDE Integration Hints

**Status**: Not started  
**Complexity**: High  
**Impact**: Low-Medium

Provide hints for rust-analyzer:
- Show which patterns are tracked
- Highlight patterns needing manual instrumentation
- Quick-fix to add annotations

### 9. Coverage Report Tool

**Status**: Not started  
**Complexity**: Medium  
**Impact**: Low

```bash
cargo borrowscope coverage src/lib.rs
# Output:
# Tracked: 95% (142/150 patterns)
# Manual needed: 5% (8 patterns)
#   - line 45: FFI call to `c_func`
#   - line 78: Union field access `data.value`
```

### 10. New Pattern Support

**Status**: Ongoing  
**Complexity**: Varies  
**Impact**: Low

Additional patterns to track:
- [ ] `Pin<T>` operations (`Pin::new`, `Pin::into_inner`)
- [ ] Iterator chains (`.map().filter().collect()`)
- [ ] `?` with custom `Try` implementations
- [ ] `mem::take`, `mem::replace`

### 11. Performance Benchmarks

**Status**: Not started  
**Complexity**: Low  
**Impact**: Low

- Macro expansion time benchmarks
- Runtime overhead benchmarks
- Memory usage benchmarks

---

## Implementation Order

```
Phase 1 (v0.2)
├── P0.1: Annotation-based disambiguation
├── P0.2: Conditional compilation
└── P0.3: Better error messages

Phase 2 (v0.3)
├── P1.4: Reduced code generation
├── P1.5: Closure capture tracking
└── P1.6: Filter & sampling

Phase 3 (v0.4+)
├── P1.7: Tracing integration
├── P2.8: IDE hints
├── P2.9: Coverage tool
└── P2.10: New patterns
```

---

## Contributing

Contributions welcome! Pick an item from this roadmap and:
1. Open an issue to discuss approach
2. Submit PR with tests
3. Update documentation

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
