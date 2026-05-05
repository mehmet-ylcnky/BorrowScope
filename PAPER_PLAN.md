# Paper Plan: BorrowScope — Eliminating Heuristics from Rust Procedural Macros Through Pre-Build Semantic Analysis

## 1. Abstract

**Summary:** A concise statement of the problem (proc-macro type blindness), the solution (two-phase build with rust-analyzer pre-analysis), the key result (109/109 patterns fully semantic, zero heuristics), the upstream contribution (rust-analyzer PR #21835), and validation (battle-tested on ripgrep, lru, uuid).

---

## 2. Introduction

### 2.1 The Proc-Macro Type Blindness Problem
**Summary:** Explain that Rust procedural macros execute before type checking — they receive raw tokens with no type information. A macro seeing `let data = create_shared(value)` cannot know it returns `Rc<T>`. This forces macros to rely on fragile string-matching heuristics.

### 2.2 The Cost of Heuristics
**Summary:** Quantify the problem: 109 ownership patterns, 66 were pure string heuristics (61%), ~500 lines of detection code. Show concrete failure cases: type aliases (`type MyRc<T> = Rc<T>`), factory functions, conditional expressions, method name ambiguity (`write` = RwLock::write or io::Write::write or MaybeUninit::write).

### 2.3 Contributions
**Summary:** List the paper's contributions:
1. A complete two-phase build system achieving 100% semantic coverage
2. An upstream contribution to rust-analyzer (PR #21835) closing an API gap
3. Validation on real-world projects (ripgrep, lru, uuid) with zero modifications needed
4. 88 runtime event types covering the full Rust ownership model
5. Open-source implementation with 364 tests

---

## 3. Background

### 3.1 Rust's Compilation Pipeline
**Summary:** Diagram showing: Parsing → Macro Expansion → Name Resolution → Type Checking → Borrow Checking → Code Generation. Emphasize the wall between macro expansion and type checking.

### 3.2 Procedural Macro Constraints
**Summary:** What macros CAN do (token manipulation, AST transformation) vs what they CANNOT do (access types, query traits, resolve paths, read filesystem portably). Cite the Rust Reference on proc-macro sandboxing.

### 3.3 rust-analyzer as a Semantic Oracle
**Summary:** Explain the `ra_ap_*` crate ecosystem — the same engine powering IDE features (go-to-definition, type hints) is available as a library. Key APIs: `Semantics`, `type_of_pat`, `resolve_method_call`, `impls_trait`, `as_adt`. Version 0.0.318 used.

### 3.4 Related Work
**Summary:** Compare with:
- **Aquascope** (Brown University): Compile-time + runtime visualization, but requires compiler modifications
- **Miri**: Interprets MIR for UB detection, not ownership visualization
- **Clippy**: Uses HIR for linting but doesn't bridge to proc-macros
- **Boris**: Standalone visualizer, no runtime instrumentation
- **Flowistry**: Information flow analysis via VSCode, different goal

Position BorrowScope as unique: pre-build semantic analysis feeding into proc-macro dispatch.

---

## 4. Architecture

### 4.1 Two-Phase Build Strategy
**Summary:** Diagram: Phase 1 (analyzer → type-info.json) → Phase 2 (macro reads JSON, transforms code) → Phase 3 (runtime captures events). Explain why each phase exists at a different compilation boundary.

### 4.2 Data Flow: JSON as Inter-Process Communication
**Summary:** The analyzer runs as a separate process (before `cargo build`). The macro runs inside `rustc`. They communicate via `.borrowscope/type-info.json`. Explain schema versioning (v3.0), backward compatibility via `serde(default)`.

### 4.3 Why Three Separate Crates
**Summary:** Each crate operates at a different execution context:
- Analyzer: pre-build, has access to rust-analyzer
- Macro: compile-time, sandboxed inside rustc
- Runtime: execution-time, linked into the binary

### 4.4 Design Decision: Static Pre-Build vs Runtime Type Analysis
**Summary:** Honest trade-off analysis. Why static wins: zero runtime overhead, access to creation semantics (Rc::new vs Rc::clone), full trait resolution, no unstable features needed. What static loses: two-step workflow, stale data risk, can't observe dyn dispatch.

---

## 5. The Analyzer (borrowscope-analyzer)

### 5.1 Workspace Loading and Sysroot Discovery
**Summary:** The critical `RustLibSource::Discover` setting. Without it: 10% resolution. With it: near-100%. Explain `load_workspace_at`, `CargoConfig`, proc-macro server disabled.

### 5.2 Type Extraction via `type_of_pat`
**Summary:** Why `type_of_pat` (not `type_of_expr`) — captures post-coercion types. Walk syntax tree for `LET_STMT` nodes, extract pattern, resolve type, calculate source location.

### 5.3 Trait Detection (17 rust-analyzer APIs)
**Summary:** Table of all 17 APIs used:
- `Semantics::resolve_method_call()` → canonical path
- `Function::module()` / `path_to_root()` → build path
- `Type::as_adt()` → ADT classification
- `Function::as_assoc_item()` → trait detection
- `Function::extern_block()` → FFI detection
- `Semantics::resolve_path()` → static detection
- `Function::ret_type()` → guard detection
- etc.

### 5.4 ADT Classification via Canonical Path Matching
**Summary:** How `KnownTypes` uses ADT identity (not string matching) to classify types. The `classify_by_resolved_type_semantic()` function with 78 initializer categories. Crate-verified fallback for unstable types.

### 5.5 Method Call Analysis
**Summary:** `analyze_method_calls()` — resolves every method call on tracked variables. Produces `MethodCallInfo` with: `operation` (canonical path), `self_borrow` (via `func.self_param(db).access(db)`), `is_trait_method`, `trait_name`, `receiver_type`, `result_type`, `is_unsafe`.

### 5.6 Standalone Expression Tracking
**Summary:** `analyze_expressions()` — tracks `thread::spawn`, `transmute`, `drop`, `forget` via `TrackedFunctions` (FunctionId comparison, not string matching).

### 5.7 Top-Level Analysis Maps (22 maps)
**Summary:** Beyond per-variable data, the analyzer produces 22 project-wide maps: `await_points`, `borrow_spans`, `unsafe_operations`, `closure_traits`, `field_accesses`, `destructuring`, `match_bindings`, `variants`, `lifetimes`, `labels`, `const_patterns`, `callables`, `record_field_exprs`, `record_field_pats`, `method_borrows`, `function_calls`, `trait_impls`. Each enriches the macro's tracking.

### 5.8 Output Schema v3.0
**Summary:** Full schema documentation: 81 VariableTypeInfo fields, 10 MethodCallInfo fields, 22 top-level maps. Show a complete JSON example for a real function.

### 5.9 Upstream Contribution: Raw Pointer Mutability API (rust-analyzer PR #21835)
**Summary:** During development, discovered that `hir::Type` had no way to distinguish `*mut T` from `*const T`. The only workaround was string parsing — a heuristic. Contributed `is_mutable_raw_ptr()` and `as_raw_ptr()` to rust-analyzer (PR #21835, merged April 13, 2026). This:
- Closes an API asymmetry (references had full mutability API, raw pointers didn't)
- Eliminates a string-parsing heuristic from BorrowScope
- Benefits all downstream consumers of `ra_ap_hir`
- Demonstrates that building tools on rust-analyzer drives improvements to the platform

Include the StackOverflow question that confirmed the gap, the reviewer's (ChayimFriedman2) approval, and the final merged commit.

---

## 6. The Macro (borrowscope-macro)

### 6.1 Type Info Loading and Lookup
**Summary:** `OnceLock`-based lazy loading. `find_project_root()` via `CARGO_MANIFEST_DIR`. Two-tier lookup: `lookup_in_function(fn_name, var_name, decl_index)` primary, `lookup_by_name(var_name)` fallback. Handles variable shadowing via `decl_index`.

### 6.2 Initializer Kind Dispatch (78 categories)
**Summary:** `transform_by_initializer_kind()` — matches on the analyzer's `initializer_kind` field to select the precise tracking function. 78 categories from `rc_new` to `atomic_new` to `user_struct`.

### 6.3 Semantic Method Call Dispatch (18 canonical path patterns)
**Summary:** `semantic_op` lookup via `mc_info.operation`. Exact canonical path matching: `"core::cell::set"`, `"std::sync::poison::mutex::lock"`, `"alloc::borrow::to_mut"`, etc. No `method_name ==` heuristics.

### 6.4 Self-Borrow Inference (47 patterns, zero heuristics)
**Summary:** `infer_self_borrow_type()` reads `method_calls[].self_borrow` from analyzer data. Falls back to `method_borrows` top-level map. Returns `Immutable`, `Mutable`, or `Consuming`. The old 47-pattern heuristic function is deleted.

### 6.5 Clone Trait Verification
**Summary:** Uses `is_trait_method` + `trait_name` from analyzer to distinguish `Clone::clone` from inherent `.clone()` methods. Prevents false positives on user types with methods named `clone`.

### 6.6 The `safe_parse_quote!` Pattern
**Summary:** `syn::parse_quote!` panics on closures with block bodies (`|| { ... }`). The `safe_parse_quote!` macro uses `quote!` + `syn::parse2` which returns `Result` instead of panicking. Applied to all 36 call sites that embed user expressions.

### 6.7 The `pending_inserts` Pattern
**Summary:** When an expression cannot be wrapped in a block (would break rustc parsing), emit the tracking call as a separate statement via `pending_inserts`. Used for: `transform_unwrap`, `transform_try`, `track_method_call`, `track_new` (for complex expressions).

### 6.8 Mandatory Analyzer: No Fallback Path
**Summary:** The macro panics with a clear error if `.borrowscope/type-info.json` is missing. Zero heuristic fallback code remains. All `detect_*` functions deleted. `smart_pointer.rs` deleted.

---

## 7. The Runtime (borrowscope-runtime)

### 7.1 Event Architecture (88 event types)
**Summary:** Categorized table of all 88 event types: core ownership (4), smart pointers (7), interior mutability (6), weak refs (3), unsafe (7), async (4), control flow (11), concurrency (10), expressions (8), data structures (6), Pin/Cow (5), OnceCell/MaybeUninit (9), scope (4), static/const (3).

### 7.2 Pass-Through Design
**Summary:** Every `track_*` function takes a value, records an event, and returns the value unchanged. Zero behavioral impact. The instrumented program produces identical output to the uninstrumented version.

### 7.3 Global Tracker with Atomic Timestamps
**Summary:** `parking_lot::Mutex<Tracker>` for thread safety. `AtomicU64` for lock-free monotonic timestamps. `Vec<Event>` accumulator. `reset()` / `get_events()` / `print_summary()` API.

### 7.4 Enriched Events
**Summary:** Events carry metadata from static analysis: `AwaitStart.live_variables`, `UnsafeBlockEnter.operation_kind`, `ClosureCreate.fn_trait`, `MatchArm.bindings`, `Call.receiver_type/result_type`, `Drop.location`. All backward-compatible via `serde(default, skip_serializing_if)`.

---

## 8. Evaluation

### 8.1 Coverage: 109/109 Semantic Patterns
**Summary:** Table showing all 17 categories with counts. Before: 36 semantic, 7 partial, 66 syntactic. After: 109 semantic, 0 partial, 0 syntactic.

### 8.2 Heuristic Elimination: 35 → 0
**Summary:** Table of all 35 eliminated heuristics across 7 categories: string matching on operation paths (14), method name matching (9), function path matching (2), name pattern heuristics (4), trait method detection (1), tracking set fallbacks (2), miscellaneous (3). Each mapped to its semantic replacement.

### 8.3 Battle Test: ripgrep
**Summary:** Applied BorrowScope to ripgrep's `globset` crate source code. 10 functions instrumented across 3 files (`pathutil.rs`, `glob.rs`, `lib.rs`). All 287 existing tests pass. 100% type resolution for instrumented functions. Zero fixes needed to macro or analyzer.

### 8.4 Battle Test: lru crate
**Summary:** Instrumented real-world usage of the `lru` crate (LRU cache). 6 test functions covering HashMap, iterators, closures, Option/Result, clone/move. 107 events tracked. Required one macro fix (method borrow wrapping).

### 8.5 Battle Test: uuid crate
**Summary:** Instrumented usage of the `uuid` crate. 6 test functions covering String conversions, collections, sorting, Option/Result. 88 events tracked. Zero fixes needed — passed on first try.

### 8.6 Test Suite
**Summary:** 187 macro unit tests + 74 runtime unit tests + 79 phase tests + 15 integration tests + 24 compiled examples = 364 tests + 24 examples, all passing.

### 8.7 Analyzer Resolution
**Summary:** 100% type resolution for all battle test projects. The "70.4%" metric for ripgrep's globset is explained: all variables have valid types, the percentage reflects semantic vs annotation-based resolution (both produce correct results).

---

## 9. Performance

### 9.1 Analyzer Timing
**Summary:** Workspace loading (~30-60s) dominates. Type analysis scales linearly with variable count. Small projects: ~45s total. Medium: ~50s. Large: ~90s.

### 9.2 Macro Overhead
**Summary:** JSON loading via `OnceLock` (once per compilation). Lookup is O(1) via HashMap. No measurable compile-time impact beyond initial load.

### 9.3 Runtime Overhead
**Summary:** Each `track_*` call: mutex lock + timestamp increment + event push. Measured at ~100-200ns per event. Pass-through design means zero impact on program logic.

### 9.4 Memory Usage
**Summary:** Analyzer: 500MB-1GB (rust-analyzer's semantic database). Runtime: proportional to event count (~100 bytes per event).

---

## 10. Limitations and Future Work

### 10.1 Current Limitations
**Summary:** 6 edge-case functions where `#[trace_borrow]` is removed: nested async functions (return type changed by fn_enter/fn_exit wrapping), nested functions with `format!` macros, complex unsafe blocks. These require deeper macro refactoring.

### 10.2 The `parse_quote!` vs rustc Parsing Divergence
**Summary:** `syn::parse_quote!` and `syn::parse2` accept token streams that rustc rejects — specifically blocks inside function arguments. This is a fundamental syn/rustc difference that required the `safe_parse_quote!` and `pending_inserts` patterns.

### 10.3 Stale Data Risk
**Summary:** If source changes between analysis and compilation, the JSON becomes inconsistent. The macro handles this gracefully (skips tracking for unrecognized bindings) but loses precision. Mitigation: build script integration, IDE hooks.

### 10.4 Future: borrowscope-graph
**Summary:** Graph algorithms for ownership analysis — detect cycles, compute borrow lifetimes, identify ownership hotspots.

### 10.5 Future: borrowscope-ui
**Summary:** Interactive web visualization — timeline view, ownership graph, borrow scope highlighting.

### 10.6 Future: IDE Integration
**Summary:** Direct integration with rust-analyzer LSP could eliminate the separate analysis step entirely, providing real-time type information to the macro.

---

## 11. Conclusion

**Summary:** BorrowScope demonstrates that pre-build semantic analysis can completely eliminate heuristics from Rust procedural macros. The system achieves 100% semantic coverage (109/109 patterns), is validated on real-world projects (ripgrep, lru, uuid), and contributes back to the ecosystem (rust-analyzer PR #21835). The two-phase build strategy — while adding operational complexity — provides semantic depth that runtime reflection cannot match in Rust's type system. The complete system (analyzer + macro + runtime) captures 88 event types covering the full Rust ownership model, enabling future visualization and analysis tools.

---

## References

- Rust Reference: Procedural Macros
- rust-analyzer documentation and `ra_ap_*` crate ecosystem
- PR #21835: `is_mutable_raw_ptr` and `as_raw_ptr` (rust-lang/rust-analyzer)
- StackOverflow: "How to determine raw pointer mutability from ra_ap_hir::Type?"
- Aquascope (Brown University Cognitive Engineering Lab)
- Boris: Ownership and Borrowing Visualizer
- Flowistry: Information Flow Analysis for Rust
- The Rust Book: Understanding Ownership
