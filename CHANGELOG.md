# Changelog

All notable changes to BorrowScope will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-05

### Breaking Changes
- **Analyzer is now mandatory** — `#[trace_borrow]` panics at compile time if `.borrowscope/type-info.json` is missing. Run `cargo run -p borrowscope-analyzer -- .` before building.
- All heuristic fallback code removed — zero string-matching heuristics remain.

### Added

#### borrowscope-analyzer
- 17 top-level analysis maps (await_points, borrow_spans, unsafe_operations, closure_traits, field_accesses, destructuring, match_bindings, variants, lifetimes, labels, const_patterns, callables, record_field_exprs, record_field_pats, method_borrows, function_calls, trait_impls)
- 100% semantic coverage — 109/109 ownership patterns via rust-analyzer APIs
- 78 semantic initializer categories

#### borrowscope-macro
- 81 fields consumed from VariableTypeInfo (Groups 1-3)
- 10 MethodCallInfo fields consumed (receiver_type, result_type, is_unsafe)
- `safe_parse_quote!` macro for handling closures with block bodies
- `contains_block_closure()` helper for detecting problematic expressions
- `in_nested_expr` flag to prevent region tracking in closure bodies
- Guard detection via initializer_kind patterns

#### borrowscope-runtime
- 9 new tracking functions: `track_drop_at`, `track_atomic_new`, `track_duration_new`, `track_instant_new`, `track_autoref`, `track_autoderef`, `track_var_read`, `track_var_write`, `track_method_call`
- 7 enriched tracking functions: `track_await_start_with_live_vars`, `track_unsafe_block_enter_enriched`, `track_closure_create_with_trait`, `track_match_arm_with_bindings`, `track_borrow_span`, `track_destructure`, `track_variant_construct`
- Event enrichments: `AwaitStart.live_variables`, `UnsafeBlockEnter.operation_kind/context`, `ClosureCreate.fn_trait`, `MatchArm.bindings`, `Call.receiver_type/result_type`, `Drop.location`
- 88 event types total

#### Testing
- 79 phase tests (phase1-phase5) covering all semantic patterns
- 24 compiled macro examples
- 3 battle tests: lru crate, uuid crate, ripgrep source code
- Regression baseline captured

### Changed
- `transform_method_call` uses `pending_inserts` instead of receiver wrapping (fixes borrow conflicts)
- `transform_unwrap` uses `pending_inserts` for closure arguments
- `transform_try` uses `pending_inserts` instead of block wrapping
- `transform_by_initializer_kind` skips wrapping for block-producing expressions
- `mc_info` lookup uses `lookup_in_function` with current function context
- Guard dispatch returns `None` (tracking handled by `visit_expr_mut`)
- Channel detection uses tuple pattern name lookup
- OnceLock uses generic tracking (separate from OnceCell)
- Transmute detection checks function name before emitting

### Removed
- `smart_pointer.rs` (all 17 `detect_*` functions)
- All `expect()` panics replaced with graceful fallbacks
- ~220 lines of dead heuristic code
- 7 outdated macro planning docs
- `PHASE2_IMPLEMENTATION_PLAN.md`
- `docs/planning/` and `docs/architecture/` directories

## [0.1.2] - 2024-12-22

### Added
- borrowscope-macro crate with `#[trace_borrow]` attribute
- Filter pattern support (`filter = "data*"`)
- Sampling support (`sample = 0.1`)
- Conditional compilation (`debug_only`, `release_only`)
- Feature groups: ownership, smart_pointers, loops, branches, control_flow, try, methods, async, unsafe, expressions, functions

## [0.1.1] - 2024-12-15

### Added
- 125+ tracking functions covering all ownership patterns
- Smart pointer tracking (Rc, Arc, Weak, Box, Pin, Cow)
- Interior mutability (RefCell, Cell, OnceCell, MaybeUninit)
- Concurrency (threads, channels, lock guards)
- Unsafe code tracking (raw pointers, FFI, transmute)
- Async tracking (async blocks, await)
- Control flow tracking (loops, match, branches)

## [0.1.0] - 2024-12-01

### Added
- Initial release
- Core tracking functions (track_new, track_borrow, track_move, track_drop)
- Event types and JSON serialization
- Ownership graph building
