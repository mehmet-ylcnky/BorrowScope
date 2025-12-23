# Changelog

All notable changes to BorrowScope will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### borrowscope-macro
- **Filter pattern support** - Track only variables matching glob patterns (`filter = "data*"`)
  - `*` matches zero or more characters
  - `?` matches exactly one character
  - Applied at compile-time (zero overhead for non-matching variables)
- **Sampling support** - Probabilistic tracking for reduced overhead (`sample = 0.1`)
  - Uses fast xorshift64 PRNG
  - Useful for high-frequency code paths
- **Conditional compilation** - Control when tracking is enabled
  - `debug_only` - Only in debug builds
  - `release_only` - Only in release builds
  - `feature = "name"` - Only when cargo feature enabled
- New example: `filter_sampling.rs`

#### borrowscope-runtime
- `should_sample(rate: f64) -> bool` - Check if call should be sampled
- Sampled tracking functions:
  - `track_new_sampled`
  - `track_new_with_id_sampled`
  - `track_borrow_sampled`
  - `track_borrow_mut_sampled`
  - `track_drop_sampled`
  - `track_move_sampled`
- New example: `sampling.rs`
- Improved cargo docs with comprehensive function tables

### Changed
- Updated README with macro documentation and filter/sampling examples
- Reorganized cargo docs by category

## [0.1.2] - 2024-12-XX

### Added
- borrowscope-macro crate with `#[trace_borrow]` attribute
- Feature groups: ownership, smart_pointers, loops, branches, control_flow, try, methods, async, unsafe, expressions, functions
- Presets: quiet, verbose, standard
- Skip/only options for fine-grained control

## [0.1.1] - 2024-XX-XX

### Added
- 125+ tracking functions covering all ownership patterns
- Smart pointer tracking (Rc, Arc, Weak, Box, Pin, Cow)
- Interior mutability (RefCell, Cell, OnceCell, MaybeUninit)
- Concurrency (threads, channels, lock guards)
- Unsafe code tracking (raw pointers, FFI, transmute)
- Async tracking (async blocks, await)
- Control flow tracking (loops, match, branches)
- Expression tracking (structs, tuples, arrays, closures)
- Static/const tracking

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Core tracking functions (track_new, track_borrow, track_move, track_drop)
- Event types and JSON serialization
- Ownership graph building
- RAII guards for automatic drop tracking
