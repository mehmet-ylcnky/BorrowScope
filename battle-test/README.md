# BorrowScope Battle Testing

Real-world validation of borrowscope-runtime and borrowscope-macro against popular Rust projects.

## Objective

Validate BorrowScope's compatibility with production Rust code by instrumenting well-known open-source projects, identifying gaps, and documenting compatibility.

## Test Projects

| # | Project | Stars | Status | Pass Rate | Report |
|---|---------|-------|--------|-----------|--------|
| 1 | [zoxide](https://github.com/ajeetdsouza/zoxide) | 23k | ✅ Complete | 67% (66/99) | [Report](zoxide/BATTLE_TEST_REPORT.md) |
| 2 | [bat](https://github.com/sharkdp/bat) | 50k | ✅ Complete | 92% (297/323) | [Report](bat/BATTLE_TEST_REPORT.md) |
| 3 | [fd](https://github.com/sharkdp/fd) | 35k | ✅ Complete | 90% (123/137) | [Report](fd/BATTLE_TEST_REPORT.md) |
| 4 | [ripgrep](https://github.com/BurntSushi/ripgrep) | 49k | ✅ Complete | 99.4% (2640/2657) | [Report](ripgrep/BATTLE_TEST_REPORT.md) |
| 5 | [tokio](https://github.com/tokio-rs/tokio) | 27k | ✅ Complete | 49% (148/303 files) | [Report](tokio/BATTLE_TEST_REPORT.md) |
| 6 | [axum](https://github.com/tokio-rs/axum) | 20k | ⏳ Planned | - | - |
| 6 | [serde](https://github.com/serde-rs/serde) | 9k | ⏳ Planned | - | - |
| 7 | [actix-web](https://github.com/actix/actix-web) | 22k | ⏳ Planned | - | - |
| 8 | [polars](https://github.com/pola-rs/polars) | 30k | ⏳ Planned | - | - |
| 9 | [datafusion](https://github.com/apache/datafusion) | 6k | ⏳ Planned | - | - |
| 10 | [tokio](https://github.com/tokio-rs/tokio) | 27k | ⏳ Planned | - | - |
| 11 | [rand](https://github.com/rust-random/rand) | 2k | ⏳ Planned | - | - |
| 12 | [diesel](https://github.com/diesel-rs/diesel) | 13k | ⏳ Planned | - | - |
| 13 | [portable-atomic](https://github.com/taiki-e/portable-atomic) | 500 | ⏳ Planned | - | - |

### Project Descriptions

**zoxide** - A smarter `cd` command that learns your habits. Tracks directory visit frequency and recency to enable fast navigation with partial path matching (e.g., `z proj` jumps to `/home/user/projects`). Replaces autojump/z. *Ownership patterns: Path handling, database I/O, Result chains.*

**bat** - A `cat` clone with syntax highlighting, Git integration, and automatic paging. Displays file contents with line numbers, diff markers, and language-aware coloring. Used by developers for reading code in terminal. *Ownership patterns: File I/O, iterators, builder patterns, asset management.*

**fd** - A fast, user-friendly alternative to `find`. Searches for files/directories with intuitive syntax, smart defaults, and parallel execution. Ignores .gitignore patterns by default. *Ownership patterns: Filesystem traversal, parallel iterators, regex matching.*

**ripgrep** - Blazingly fast recursive grep that respects .gitignore. Searches file contents using regex with automatic encoding detection. The gold standard for code search tools. *Ownership patterns: Memory-mapped I/O, parallel search, streaming results.*

**axum** - Ergonomic web framework built on Tokio. Provides type-safe routing, middleware, and async request handling. Used for building REST APIs and web services. *Ownership patterns: Async/await, trait bounds, tower middleware, extractors.*

**serde** - The de facto serialization framework for Rust. Provides derive macros for automatic serialization/deserialization to JSON, YAML, TOML, and many other formats. *Ownership patterns: Derive macros, visitor pattern, generic bounds, zero-copy deserialization.*

**actix-web** - Powerful, pragmatic web framework with actor model. High-performance HTTP server with middleware, extractors, and WebSocket support. *Ownership patterns: Actor system, async handlers, request/response ownership, middleware chains.*

**polars** - Fast DataFrame library for Rust and Python. Lazy evaluation, parallel execution, and Apache Arrow memory format. *Ownership patterns: Expression trees, lazy evaluation, chunked arrays, SIMD operations.*

**datafusion** - Apache Arrow-native query engine. SQL and DataFrame APIs for building data processing applications. *Ownership patterns: Query planning, expression evaluation, batch processing, async streams.*

**tokio** - Async runtime for Rust. Provides async I/O, timers, channels, and task scheduling. Foundation for async Rust ecosystem. *Ownership patterns: Futures, pinning, wakers, task spawning, channel ownership.*

**rand** - Random number generation library. Cryptographic and non-cryptographic RNGs with distribution sampling. *Ownership patterns: RNG state, distribution traits, generic bounds, seed handling.*

**diesel** - Safe, extensible ORM and query builder. Compile-time checked queries with zero-cost abstractions. *Ownership patterns: Query builder, connection pooling, transaction scopes, type-state pattern.*

**portable-atomic** - Portable atomic types including 128-bit atomics. Provides atomic operations on platforms without native support. *Ownership patterns: Atomic ordering, memory barriers, unsafe abstractions, platform-specific implementations.*

## Known Issues

| ID | Error | Description | Rust Error | Severity |
|----|-------|-------------|------------|----------|
| ERR-001 | Lifetime-breaking shadowing | Macro shadows input params, breaking return lifetime | E0515 | Critical |
| ERR-002 | Tuple destructuring | Tuple patterns not properly extracted | E0425 | Critical |
| ERR-003 | Mutable method chains | track_borrow returns &T, but method needs &mut T | E0596 | Critical |
| ERR-004 | Const context tracking | track_branch called in const/static context | E0015 | Critical |
| ERR-005 | Build.rs inclusion | Files included by build.rs can't use proc macros | E0433 | Critical |
| ERR-006 | Temporary dropped | Tracking extends temporary lifetime requirements | E0716 | Critical |
| ERR-007 | &'static str mismatch | Macro doesn't preserve &'static lifetime on params | E0308 | Critical |
| ERR-008 | impl Into<T> fails | Macro breaks impl Trait parameter types | E0277/E0282 | Critical |
| ERR-009 | Self-consuming functions | Macro wraps self in borrow, breaks ownership transfer | E0507/E0515/E0308 | Critical |
| ERR-010 | Range indexing fails | Macro breaks .get(range) method calls | E0061 | Critical |
| ERR-011 | Struct field access fails | Macro changes self type, breaks field access | E0609 | Critical |
| ERR-012 | Trait impl methods | Macro changes method signature, breaks trait conformance | E0407/E0599 | Critical |
| ERR-013 | Lifetime param mismatch | Macro creates temporaries that don't satisfy impl<'a> lifetimes | E0597 | Critical |

## Methodology

### Phase 1: Reconnaissance
- Clone project
- Analyze structure and key modules
- Identify functions with interesting ownership patterns

### Phase 2: Instrumentation
- Add borrowscope dependencies
- Apply `#[trace_borrow]` to selected functions
- Document any macro expansion failures

### Phase 3: Execution
- Build instrumented code
- Run tests/examples
- Collect tracking events

### Phase 4: Analysis
- Verify events are correct and complete
- Identify missing patterns
- Document gaps and workarounds

### Phase 5: Reporting
- Create detailed report
- Categorize issues by severity
- Propose fixes or document limitations

## Compatibility Matrix

| Pattern | zoxide | bat | fd | ripgrep | tokio |
|---------|--------|-----|----|---------| ------|
| Basic ownership | ✅ | ✅ | ✅ | ✅ | ✅ |
| Iterators | ✅ | ✅ | ✅ | ✅ | ✅ |
| Smart pointers | ➖ | ✅ | ➖ | ➖ | ✅ |
| Interior mutability | ➖ | ➖ | ➖ | ➖ | ✅ |
| Async/await | ➖ | ➖ | ➖ | ➖ | ✅ |
| Closures | ✅ | ✅ | ✅ | ✅ | ✅ |
| Generics | ✅ | ✅ | ✅ | ✅ | ✅ |
| Lifetimes | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ |
| Unsafe | ➖ | ➖ | ➖ | ➖ | ✅ |
| Mutable method chains | ❌ | ❌ | ❌ | ⚠️ | ❌ |
| Tuple destructuring | ❌ | ➖ | ❌ | ⚠️ | ➖ |
| Const expressions | ❌ | ➖ | ➖ | ➖ | ➖ |
| impl Into/AsRef params | ➖ | ❌ | ❌ | ✅ | ✅ |
| Builder patterns | ➖ | ❌ | ➖ | ✅ | ✅ |
| Range indexing | ➖ | ❌ | ❌ | ✅ | ⚠️ |
| Trait implementations | ➖ | ➖ | ➖ | ❌ | ✅ |
| Pin<&mut Self> | ➖ | ➖ | ➖ | ➖ | ❌ |
| Futures/async traits | ➖ | ➖ | ➖ | ➖ | ✅ |

Legend: ✅ Works | ⚠️ Partial | ❌ Fails | ➖ N/A

## Summary

### zoxide (Complete)
- **99 functions tested** across 16 files
- **66 pass (67%)**, 33 fail
- Most failures due to ERR-003 (mutable method chains)
- Files with 100% pass rate: config.rs, error.rs, main.rs, shell.rs, db/dir.rs, cmd/mod.rs

### bat (Complete)
- **323 functions tested** across 37 files
- **297 pass (92%)**, 26 fail
- Most failures due to ERR-003 (mutable method chains) and ERR-008 (impl Into<T>)
- Files with 100% pass rate: style.rs, line_range.rs, less.rs, lessopen.rs, decorations.rs, syntax_mapping.rs, assets.rs, all bin/bat/ files

### fd (Complete)
- **137 functions tested** across 19 files
- **123 pass (90%)**, 14 fail
- Most failures due to ERR-003 (mutable borrows) and ERR-009 (self-consuming)
- Files with 100% pass rate: error.rs, config.rs, filetypes.rs, filesystem.rs, output.rs, regex_helper.rs, exec/command.rs, exec/input.rs, exec/job.rs, exec/token.rs
- Note: Tested on v9.0.0 (latest requires Rust 1.90, system has 1.87)

### ripgrep (Complete)
- **2,657 functions tested** across 71 files in 9 crates
- **2,640 pass (99.4%)**, 17 fail
- 21 compilation errors, but only 17 unique functions affected (some cascading)
- Most failures due to ERR-012 (trait impl methods) in crates/matcher
- **68 out of 71 files with 100% pass rate**
- Highest pass rate of all tested projects

### tokio (Complete)
- **303 files tested** with 3,784 functions
- **148 files pass (49%)**, 155 fail
- **Lowest pass rate** of all tested projects
- Most failures due to ERR-009 (self-consuming methods like guard.map()) and ERR-003 (mutable method chains)
- Async primitives heavily use self-consuming patterns that break with current macro
- Simple async patterns (spawn, yield, basic futures) work correctly

---

## Running Tests

```bash
cd battle-test/<project>/repo
# Follow project-specific instructions in BATTLE_TEST_REPORT.md
```

---

## Future Testing Phases

Current battle testing validates **compilation only**. For production readiness, additional testing phases are needed:

### Phase A: Runtime Correctness
- [ ] Verify tracking events are emitted for all instrumented operations
- [ ] Validate event ordering matches actual execution order
- [ ] Check borrow/drop pairing is correct
- [ ] Test event data accuracy (variable names, locations, types)

### Phase B: Event Completeness
- [ ] Compare tracked events against expected ownership transfers
- [ ] Verify no events are missed in complex control flow
- [ ] Test async/await event correlation
- [ ] Validate smart pointer reference counting accuracy

### Phase C: Performance Impact
- [ ] Measure overhead per tracking call
- [ ] Benchmark memory usage with large event counts
- [ ] Test with `track` feature disabled (zero-cost verification)
- [ ] Profile hot paths in real applications

### Phase D: Integration Testing
- [ ] Run instrumented project test suites
- [ ] Verify no behavioral changes from instrumentation
- [ ] Test JSON export with real event data
- [ ] Validate cross-crate tracking
