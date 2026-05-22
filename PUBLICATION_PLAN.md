# BorrowScope: Publication Plan

## Published Papers

| # | Title | Component | Date | Link |
|---|-------|-----------|------|------|
| 1 | Technical White Paper: Architecture & Design | borrowscope-runtime | Dec 2025 | [Link](borrowscope-runtime/) |
| 2 | Automating Instrumentation with a Procedural Macro | borrowscope-macro | Jan 2026 | [Link](borrowscope-macro-intro/) |
| 3 | Rust's Type Information Barrier | borrowscope-macro | Feb 2026 | [Link](borrowscope-macro-whitepaper/) |
| 4 | Bridging the Type Information Gap | borrowscope-analyzer | Mar 2026 | [Link](borrowscope-analyzer-whitepaper/) |
| 5 | Eliminating Heuristics from Rust Procedural Macros | borrowscope-analyzer + macro | May 2026 | [Link](eliminating-heuristics-from-rust-proc-macros/) |
| 6 | Ownership Graph Construction from Runtime Event Streams | borrowscope-graph | May 2026 | [Link](ownership-graph-construction/) |
| 7 | Battle-Testing BorrowScope on Real-World Crates | All | May 2026 | [Link](battle-test-whitepaper/) |

---

## Planned Papers

### Paper 8: Real-Time Ownership Visualization via Language Server Protocol

**Component:** `borrowscope-lsp`

**Abstract:** Presents a Language Server Protocol implementation that provides real-time ownership analysis for Rust code using rust-analyzer's semantic engine (ra_ap_* crates). The server computes ownership graphs, NLL-aware borrow scopes, cross-function borrow tracking, and field-level memory layouts — all without requiring code instrumentation.

**Key Contributions:**
- 5 custom LSP request types for ownership visualization
- NLL borrow scope computation via `Definition::usages()`
- Cross-function borrow resolution with `HasSource` for file navigation
- Memory layout with field decomposition for all types via `ty.layout(db)` + `ty.fields(db)`
- Background workspace loading with debounced incremental analysis
- LRU analysis cache with stale detection
- 107 protocol tests + integration tests

**Sections:**
1. Introduction — Why a dedicated LSP for ownership?
2. Background — LSP protocol, rust-analyzer APIs, existing tools
3. Architecture — Server loop, state management, workspace loading
4. Ownership Graph Computation — Variable extraction, borrow scopes, move detection
5. Cross-Function Analysis — Call graph resolution, borrow path tracking
6. Memory Layout — Field-level decomposition, padding, pointer relationships
7. Performance — <5ms per request, debounce, caching strategy
8. Evaluation — 107 tests, real-world projects
9. Related Work — rust-analyzer, Flowistry, Aquascope
10. Conclusion

**Target Venue:** ICSE 2027 Tool Demo Track, or VSCode Extension Ecosystem Workshop

---

### Paper 9: Interactive Multi-View Ownership Visualization for Rust

**Component:** `borrowscope-vscode`

**Abstract:** Presents an interactive VS Code extension with 11 complementary visualization views for understanding Rust's ownership system. The extension combines inline editor decorations (lifeline flows, colored annotations, CodeLens) with a WebView panel offering force-directed graphs, timelines, scope nesting, reference count charts, and memory layout visualization — all updated in real-time as the developer edits code.

**Key Contributions:**
- 11 visualization views: Graph, Table, Timeline, Scopes, RefCount, Moves, Conflicts, Compare, CrossRefs, Memory, Runtime
- Inline editor decorations: lifeline flow lines (├─ │ ╰─), colored ownership hints, CodeLens stats
- Landing page with icon grid navigation
- Memory timeline with play/scrub slider showing variable lifecycle
- D3.js force-directed graph with linked highlighting
- Vertical icon sidebar for view switching
- Accessibility: ARIA labels, keyboard navigation, screen reader descriptions
- 33 configuration settings, 10 theme colors (dark/light/high-contrast)
- 689 extension tests

**Sections:**
1. Introduction — The need for ownership visualization in IDEs
2. Design Principles — Multiple views, progressive disclosure, linked highlighting
3. Editor Decorations — Lifelines, annotations, CodeLens, highlights
4. Visualization Panel — 11 views with interaction design
5. Memory Layout View — Stack/heap columns, field detail, timeline slider
6. User Experience — Landing page, keyboard shortcuts, accessibility
7. Theme Integration — Dark/light/high-contrast adaptation
8. Performance — <100ms render, debounced updates
9. User Study (if conducted) — Developer comprehension improvement
10. Related Work — Aquascope, Boris, Flowistry, REVIS

**Target Venue:** VL/HCC 2027, CHI 2027 (with user study), or UIST 2027

---

### Paper 10: Merging Static and Runtime Analysis for Ownership Visualization

**Component:** `borrowscope-vscode` (runtime integration, Milestone 12)

**Abstract:** Presents a system that merges compile-time ownership analysis (from an LSP) with runtime execution data (from instrumented code) into a unified visualization. The system detects divergences between what the compiler predicts and what actually happens at runtime — including Rc/Arc leaks, conditional moves, async borrows held across await points, and unsafe code hiding ownership information.

**Key Contributions:**
- 88 runtime event types consumed from file or WebSocket
- 4-tier variable mapping: runtime events → static analysis variables
- 16 divergence detection kinds with severity levels and suggestions
- Rc/Arc reference count timeline with leak and cycle detection
- Drop order analysis with LIFO verification
- Async borrow tracking across await points
- Merged view: green (agreement), red (divergence), with hover details
- Status bar indicator: `Static ✓ | Runtime ✓ (103 events, 2s ago)`
- File watcher + WebSocket live connection modes

**Sections:**
1. Introduction — Why combine static and runtime analysis?
2. Background — Static analysis limitations, runtime tracking capabilities
3. Architecture — Two data sources, one unified renderer
4. Event Ingestion — File-based and WebSocket modes
5. Variable Mapping — Name + line + type matching with confidence levels
6. Merge Algorithm — Combining static graph with runtime events
7. Divergence Detection — 16 kinds, severity classification, suggestions
8. Reference Count Timeline — Leak detection, cycle detection
9. Async Borrow Tracking — Borrows held across await points
10. Evaluation — Real-world divergence examples
11. Related Work — Miri, Valgrind, sanitizers

**Target Venue:** OOPSLA 2027, DLS 2027, or Runtime Verification Workshop

---

### Paper 11: End-to-End Ownership Instrumentation — From Source to Visualization

**Component:** All (analyzer → macro → runtime → LSP → VS Code)

**Abstract:** Presents the complete BorrowScope pipeline: a static analyzer extracts type information, a proc macro uses it to instrument code with zero-cost tracking calls, the instrumented program generates runtime events, and a VS Code extension merges static and runtime data into interactive visualizations. The paper evaluates the full pipeline on real Rust projects, measuring instrumentation accuracy, runtime overhead, and developer comprehension.

**Key Contributions:**
- Complete pipeline: analyzer (207 variables) → macro (133 instrumentation points) → runtime (88 event types) → LSP (5 endpoints) → VS Code (11 views)
- The circular dependency problem and its solution (test_project pattern)
- Semantic variant resolution feeding into runtime event classification
- Performance budget: analyzer 15-30s, macro 0ns without feature, runtime 75ns/call, LSP <5ms/request
- 2,100+ tests across all components
- Zero heuristics throughout the entire pipeline

**Sections:**
1. Introduction — The vision: making ownership visible
2. The Pipeline — 5 components, data flow diagram
3. Static Analysis Phase — Analyzer + type-info.json
4. Instrumentation Phase — Macro + 133 transformation points
5. Runtime Phase — Event generation + export
6. Visualization Phase — LSP + VS Code panel
7. Integration Challenges — Circular dependencies, name collisions, unsafe blocks
8. Evaluation — Real projects, test coverage, performance
9. Limitations — Generic monomorphization, doc tests, unsafe inner blocks
10. Future Work — borrowscope-memory, borrowscope-cli, borrowscope-web

**Target Venue:** ICSE 2027 (main track), or RustConf 2027 (industry talk)

---

### Paper 12 (Update): Eliminating Heuristics v3.0

**Component:** borrowscope-analyzer + macro

**New content for existing paper:**
- Enum variant resolution via `sema.resolve_path()` → `PathResolution::Def(Variant)` (Cow::Borrowed vs Cow::Owned)
- Removal of `by_name` fallback (cross-function collision fix)
- `contains_block_closure` bypass for Rc/Arc/thread/unsafe patterns
- Unsafe block visitor with `syn::parse2` fallback
- 563 tests (up from 364)
- `is_sync_weak` field for Arc::downgrade distinction

---

## Publication Timeline

| Month | Paper | Status |
|-------|-------|--------|
| Jun 2026 | Paper 12: Analyzer v3.0 update | Ready to write |
| Jul 2026 | Paper 8: LSP Server | Ready to write |
| Aug 2026 | Paper 9: VS Code Visualization | Ready to write |
| Sep 2026 | Paper 10: Runtime + Static Merge | Ready to write |
| Oct 2026 | Paper 11: End-to-End Pipeline | Ready to write |

---

## Venue Strategy

| Venue | Deadline | Paper(s) | Type |
|-------|----------|----------|------|
| ICSE 2027 | Sep 2026 | Paper 11 (main) or Paper 8 (tool demo) | Conference |
| OOPSLA 2027 | Oct 2026 | Paper 10 | Conference |
| VL/HCC 2027 | May 2026 | Paper 9 | Conference |
| RustConf 2027 | Mar 2027 | Paper 11 | Industry talk |
| EuroRust 2027 | Jun 2027 | Paper 8 or 9 | Talk |
| Rust Verification Workshop | TBD | Paper 10 | Workshop |

---

## Total Publication Count

- **Published:** 7 papers
- **Planned:** 5 papers (4 new + 1 update)
- **Grand total:** 12 papers covering the complete BorrowScope ecosystem
