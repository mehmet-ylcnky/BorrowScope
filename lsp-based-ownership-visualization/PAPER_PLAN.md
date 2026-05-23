# borrowscope-lsp: Real-Time Ownership Visualization via Language Server Protocol

## Detailed Paper Guideline

---

## Abstract

**Summary:** 3-paragraph abstract. First paragraph: the problem (ownership is invisible in editors, developers reason about it mentally). Second paragraph: the solution (LSP server using ra_ap_* APIs for real-time semantic ownership analysis with zero heuristics). Third paragraph: results (107 protocol tests, <100ms analysis budget, 8 custom LSP requests, field-level memory layout, cross-function borrow tracking).

---

## 1. Introduction

### 1.1 The Invisible Ownership Problem

**Summary:** Rust's ownership system is enforced at compile time but invisible during development. Developers must mentally track: who owns what, which borrows are active, where moves happen, which scopes overlap. The compiler only reports errors - it never shows the positive case ("here's what's working correctly"). This cognitive load is the #1 barrier for Rust adoption.

**Codeblock placeholder:** A simple Rust function with 3 variables, 2 borrows, 1 move - annotated with comments showing what a developer must mentally track.

### 1.2 From Runtime Instrumentation to Static Analysis

**Summary:** Explain the BorrowScope ecosystem progression:
- **borrowscope-runtime**: Records 88 event types at execution time. Sees actual Rc strong counts, real drop order, precise timing. Limitation: requires compilation + execution, sees only one path, has overhead.
- **borrowscope-macro**: Instruments code with `track_*` calls using semantic type-info from the analyzer. Bridge between static analysis and runtime. Limitation: still requires compilation and execution.
- **borrowscope-lsp**: Fills the gap - analyzes ownership statically, in real-time, as the developer types. No compilation, no execution, all paths visible, editor-native feedback.
- **How they complement**: LSP = what *could* happen (all paths, instant). Runtime = what *did* happen (precise, one path). VS Code merges both with divergence detection.

**Table placeholder:** 3-column comparison table (Runtime vs Macro vs LSP) with rows: requires compilation, sees all paths, real-time feedback, precision level, overhead.

### 1.3 Limitations of Existing Approaches

**Summary:** Brief critique of Aquascope (batch-only, no editor integration), RustViz (manual annotation), Flowistry (information flow not ownership), rust-analyzer (type info but no ownership graph). None provide: real-time ownership graphs, cross-function borrow tracking, memory layout visualization, or conflict detection as editor feedback.

### 1.4 Research Questions

**Summary:** 4 research questions:
- RQ1: Can ownership relationships be extracted in real-time (<100ms) using compiler APIs?
- RQ2: Can cross-function borrow paths be resolved without whole-program analysis?
- RQ3: Can memory layout be reconstructed from type information alone?
- RQ4: Does real-time ownership feedback reduce developer cognitive load?

### 1.5 Contributions

**Summary:** Bullet list of 6 contributions: (1) LSP server with 8 custom requests, (2) ownership analysis engine using ra_ap_* with zero heuristics, (3) cross-function borrow tracking, (4) field-level memory layout visualization, (5) debounced incremental analysis pipeline, (6) 107 protocol tests validating correctness.

---

## 2. Related Work

### 2.1 Aquascope (Brown University)

**Summary:** Aquascope visualizes permissions (read/write/own) on a per-statement basis. Generates static images. Strengths: pedagogically clear, based on MIR. Weaknesses: batch-only (not real-time), no editor integration, no cross-function tracking, no memory layout.

### 2.2 RustViz (University of Michigan)

**Summary:** RustViz generates SVG timelines of ownership events. Requires manual annotation of source code with special comments. Strengths: beautiful output. Weaknesses: manual effort, not automated, no editor integration, no real-time.

### 2.3 Flowistry (Stanford)

**Summary:** Flowistry tracks information flow (which outputs depend on which inputs). Uses MIR-based analysis. Strengths: precise data flow. Weaknesses: not ownership-specific, no borrow scope visualization, no memory layout.

### 2.4 REVIS

**Summary:** REVIS provides runtime visualization of Rust execution. Strengths: dynamic view. Weaknesses: requires execution, not real-time in editor.

### 2.5 rust-analyzer Built-in Features

**Summary:** rust-analyzer provides type inference, go-to-definition, inlay hints for types. But: no ownership graph, no borrow scope visualization, no conflict detection, no memory layout, no cross-function borrow tracking. borrowscope-lsp builds ON TOP of rust-analyzer's semantic engine (ra_ap_*) to add ownership-specific analysis.

### 2.6 Positioning of borrowscope-lsp

**Summary:** Comparison table showing feature matrix across all tools. borrowscope-lsp is the only tool that provides: real-time + editor-native + ownership graph + cross-function + memory layout + conflict detection.

**Table placeholder:** Feature comparison matrix (6 tools x 8 features).

---

## 3. Architecture

### 3.1 System Overview

**Summary:** High-level architecture: VS Code extension <-> LSP protocol (JSON-RPC over stdio) <-> borrowscope-lsp server <-> ra_ap_* semantic engine (Salsa DB + VFS). The server runs as a separate process, communicates via stdin/stdout.

**SVG placeholder:** Architecture pipeline diagram showing: [VS Code] --LSP--> [borrowscope-lsp] --hir API--> [Salsa DB] --VFS--> [Source Files]. Arrows show data flow direction. Click to highlight each component.

### 3.2 The ra_ap_* Semantic Engine

**Summary:** Explain the ra_ap_* crate ecosystem used: ra_ap_hir (high-level IR, Type, Function, Module), ra_ap_ide_db (RootDatabase, Semantics), ra_ap_syntax (AST, TextRange), ra_ap_vfs (virtual file system), ra_ap_load_cargo (workspace loading), ra_ap_hir_ty (attach_db for type queries). These are the same crates rust-analyzer uses internally - we get full compiler-grade semantic information.

**Codeblock placeholder:** The key imports showing which ra_ap_* crates are used and what each provides.

### 3.3 Workspace Loading via load_workspace_at

**Summary:** The expensive one-time operation (~30-40s) that loads the entire workspace: resolves dependencies, discovers sysroot, builds the Salsa database. Uses `ra_ap_load_cargo::load_workspace_at` with CargoConfig (sysroot discovery) and LoadCargoConfig (out dirs, no proc macros, prefill caches).

**Codeblock placeholder:** The `load_workspace` function showing CargoConfig and LoadCargoConfig setup.

### 3.4 Salsa Incremental Computation Model

**Summary:** Salsa is a demand-driven incremental computation framework. When a file changes, only queries that depend on that file are re-evaluated. The VFS tracks file changes, `apply_change` pushes them to the database, and subsequent queries automatically recompute only what's needed. This is why re-analysis after an edit is fast (<100ms) even though initial loading is slow.

**Codeblock placeholder:** The `apply_vfs_changes` function showing how file edits are pushed to Salsa.

### 3.5 LSP Protocol Layer

**Summary:** Standard LSP over stdio using `lsp-server` crate. Connection handles JSON-RPC framing. Server declares capabilities: textDocumentSync (full), hover, codeLens, inlayHints. Main loop polls with 50ms timeout to check debounce timer between messages.

**Codeblock placeholder:** The `server_capabilities()` function.

### 3.6 Custom Request Protocol

**Summary:** 5 custom requests beyond standard LSP: `borrowscope/ownershipGraph`, `borrowscope/borrowScopes`, `borrowscope/variableInfo`, `borrowscope/crossFunctionBorrows`, `borrowscope/memoryLayout`. Each takes a TextDocumentPosition or TextDocument param and returns structured JSON. Plus 1 custom notification: `borrowscope/analysisUpdated`.

**Table placeholder:** Request/response schema table for all 5 custom requests.

### 3.7 Background Loading with Progress Reporting

**Summary:** Workspace loading runs in a background thread. The main loop remains responsive during loading (returns `_status: "loading"` for requests). Progress is reported via standard LSP `$/progress` notifications (WorkDoneProgressBegin/End). When loading completes, the receiver channel delivers the WorkspaceData.

**Codeblock placeholder:** The `start_background_loading` function showing thread spawn + progress notifications.

---

## 4. Ownership Analysis Engine

### 4.1 Full Type Extraction (40+ Properties)

**Summary:** For every `let` binding, we extract exhaustive type information using `hir::Type` methods: 17 boolean properties (is_reference, is_copy, is_closure, etc.), decomposition (reference inner, ADT info, type arguments), layout (size, alignment, drop glue), callable info, autoderef chain, struct fields, tuple fields. This is the foundation for all subsequent analysis.

**Codeblock placeholder:** The `extract_full_type_info` function signature and the VariableOwnershipInfo struct showing all 40+ fields.

### 4.2 Ownership Classification Algorithm

**Summary:** Every variable is classified into one of 8 categories: Owned, SharedRef, MutableRef, Rc, Arc, InteriorMut, RawPointer, Copy. Classification uses a priority chain: unknown -> raw_ptr -> &mut -> & -> Copy -> (check ADT path for Rc/Arc/Cell) -> Owned. ADT path is resolved semantically via `hir::Adt::module().path_to_root()` - no string matching on display names.

**Codeblock placeholder:** The `classify_ownership` function.

### 4.3 Borrow Scope Computation via Definition::usages

**Summary:** For each reference binding (`let r = &x`), compute the active scope: start = declaration line, end = last use of the borrower variable. Last use is found via `Definition::Local(local).usages(sema).all()` which returns all references across the file. The maximum TextRange end among all usages gives the scope end. Also handles guard types (RefCell::borrow, Mutex::lock) as implicit borrows.

**SVG placeholder:** Step-through visualization showing: (1) let r = &x declared at line 5, (2) r used at line 8, (3) r used at line 12 (last use), (4) scope computed as lines 5-12. Click to advance steps.

**Codeblock placeholder:** The `compute_borrow_scopes` and `find_last_use` functions.

### 4.4 Move Detection (4 Patterns)

**Summary:** Four move patterns detected semantically:
1. `let b = a` - assignment move (non-Copy, non-reference path expression)
2. `foo(a)` - function argument move (non-Copy passed by value)
3. `return a` - return move
4. `move || { a }` - closure capture move (via `closure_hir.captured_items`)

Each checks `ty.is_copy(db)` and `ty.is_reference()` to avoid false positives.

**Codeblock placeholder:** The `detect_moves` dispatcher and `detect_let_move` showing the Copy/reference guards.

### 4.5 Closure Capture Analysis via CaptureKind

**Summary:** For each closure expression, resolve its hir::ClosureId, then query `captured_items(db)` to get each captured variable with its CaptureKind (SharedRef, UniqueSharedRef, MutableRef, Move). Also determine which Fn trait the closure implements (Fn, FnMut, FnOnce) via `closure_hir.fn_trait(db)`.

**Codeblock placeholder:** The `analyze_closures` function showing CaptureKind mapping.

### 4.6 Rc/Arc Clone Tracking

**Summary:** Two patterns detected:
1. Method clone: `let b = a.clone()` where receiver type is Rc/Arc (checked via ADT path)
2. Explicit clone: `let b = Rc::clone(&a)` (path segments resolved to Rc/Arc qualifier)

Classification uses `classify_rc_arc` which checks `ty.as_adt()` module path for "rc::rc" or "sync::arc".

**Codeblock placeholder:** The `detect_method_clone` and `classify_rc_arc` functions.

### 4.7 Conflict Detection (Overlap Algorithm)

**Summary:** O(n^2) pairwise comparison of borrow scopes. Two borrows conflict if: (1) same target variable, (2) at least one is mutable, (3) line ranges overlap (overlap_start = max(a.start, b.start), overlap_end = min(a.end, b.end), conflict if overlap_start <= overlap_end). Reports ConflictKind: MutableAndShared or MultipleMutable.

**SVG placeholder:** Two borrow scope bars (one shared, one mutable) targeting the same variable. Overlap region highlighted in red. Click to show different conflict scenarios (no overlap, partial overlap, full containment).

**Codeblock placeholder:** The `detect_conflicts` function.

### 4.8 Method Call Resolution (Self Borrow Detection)

**Summary:** For every method call, resolve via `sema.resolve_method_call()` to get the `hir::Function`. Then check `func.self_param(db).access(db)` to determine if the method borrows self as Shared (&self), Exclusive (&mut self), or Owned (self). Also resolves: canonical path, return type, whether it's a trait method, unsafe status.

**Codeblock placeholder:** The `resolve_method_calls` function and SelfBorrow enum.

---

## 5. Cross-Function Borrow Tracking

### 5.1 Problem: Borrows That Escape Function Boundaries

**Summary:** When `process(&data)` is called, the borrow of `data` extends into the callee. Standard borrow scope computation only sees the caller's scope. To understand the full borrow lifetime, we need to trace into the callee and determine how long the parameter lives there.

**Codeblock placeholder:** Example showing a borrow passed to a function, with the question: "how long does this borrow live inside process()?"

### 5.2 Call Target Resolution via sema.resolve_path

**Summary:** For `CallExpr`, resolve the callee path expression via `sema.resolve_path(&path)`. If it resolves to `PathResolution::Def(ModuleDef::Function(f))`, we have the target function. Extract: function name, source file (via `f.source(db)`), parameter names (via `f.params_without_self(db)`).

**Codeblock placeholder:** The `resolve_call_target` function.

### 5.3 Method Receiver Analysis via self_param.access

**Summary:** For `MethodCallExpr`, resolve via `sema.resolve_method_call()`. Check if receiver is passed as &self or &mut self via `func.self_param(db).access(db)`. This determines whether the method call creates a shared or exclusive borrow of the receiver that extends into the callee.

### 5.4 Borrow Path Construction

**Summary:** Build a `CrossFunctionBorrow` with a path of segments: Origin (where the borrow starts in the caller) -> Parameter (where it arrives in the callee). Each segment records: file, function_name, variable name, line range, mutability, kind (Origin/Parameter/PassThrough/Return).

**SVG placeholder:** Cross-function borrow path visualization. Two function boxes (caller and callee). Arrow showing the borrow flowing from caller's variable through the function call into the callee's parameter. Click to show different scenarios (shared, mutable, chained).

### 5.5 Performance Guards (Time Budget, Max Results)

**Summary:** Cross-function analysis can be expensive (many call sites). Two guards: MAX_CROSS_BORROWS = 50 (stop after 50 results), MAX_CROSS_ANALYSIS_TIME_MS = 200 (abort if taking too long). These ensure the LSP stays responsive even for large files with many function calls.

---

## 6. Memory Layout Visualization

### 6.1 Stack Frame Reconstruction

**Summary:** For each function, iterate all `let` bindings. For each, query `ty.layout(db)` to get size and alignment. Compute offsets by simulating stack allocation (align each variable to its alignment requirement, then advance by its size). Result: a StackFrame with total_size and per-variable offset/size/alignment.

**SVG placeholder:** Stack frame diagram showing variables stacked vertically with byte offsets on the left, sizes on the right, alignment gaps shown as gray regions. Click on a variable to expand its internal fields.

**Codeblock placeholder:** The offset computation loop from `analyze_memory_layout`.

### 6.2 Field-Level Layout for Standard Library Types

**Summary:** For 12+ standard library types, provide known internal field layouts: String (ptr/len/cap), Vec (ptr/len/cap), Box (NonNull), Rc/Arc (NonNull to inner), RefCell (borrow_flag + UnsafeCell), HashMap (ctrl/bucket_mask/items/growth_left), Option (discriminant + value), Result (discriminant + value).

**Codeblock placeholder:** The match arms in `extract_type_fields` for String, Vec, Rc, HashMap.

### 6.3 User-Defined Struct Field Extraction via ty.fields

**Summary:** For user-defined structs, call `ty.fields(db)` to get all fields with their types. For each field, query `field_ty.layout(db)` for size and alignment. Compute field offsets respecting alignment. This gives field-level detail for any struct the user defines.

**Codeblock placeholder:** The ADT struct branch in `extract_type_fields`.

### 6.4 Heap Allocation Detection and Estimation

**Summary:** Classify variables by MemoryCategory. HeapBacked types (Vec, String, Box, HashMap) get a HeapAllocation entry with estimated heap size. RefCounted types (Rc, Arc) get heap allocation for the inner value + reference counts. Estimation is rough (based on type name) but gives useful relative sizing.

### 6.5 Pointer Relationship Graph

**Summary:** Build a graph of pointer relationships: HeapBacked variables "owns_heap" their backing storage, RefCounted variables "owns_heap" their inner, Reference variables "borrows" their target. This graph is rendered as arrows in the memory layout visualization.

### 6.6 Memory Category Classification

**Summary:** 5 categories: StackOnly (Copy types <= 16 bytes), HeapBacked (Vec/String/Box/HashMap), Reference (&T/&mut T), RefCounted (Rc/Arc), InteriorMut (RefCell/Cell/Mutex/RwLock). Classification uses ADT name resolution via hir.

---

## 7. Real-Time Editor Integration

### 7.1 Debounced Analysis Pipeline

**Summary:** File changes are debounced (default 300ms, configurable via initializationOptions). Each didChange stores content immediately but delays analysis. When debounce expires: (1) mark cache stale, (2) apply VFS changes to Salsa, (3) send analysisUpdated notification, (4) publish diagnostics. This prevents thrashing during rapid typing.

**SVG placeholder:** Timeline showing rapid keystrokes, debounce window, and single analysis trigger. Contrast with non-debounced (analysis on every keystroke).

**Codeblock placeholder:** The debounce check in main_loop and `flush_pending_changes`.

### 7.2 Smart Notification Filtering (Only Affected Functions)

**Summary:** When a file changes, don't notify about ALL functions - only those whose bodies actually changed. Compare old and new content: extract function names and bodies, diff them, report only added/removed/modified functions. This prevents unnecessary re-rendering in the VS Code extension.

**Codeblock placeholder:** The `send_analysis_updated_if_changed` function with `extract_function_bodies` diffing.

### 7.3 Analysis Cache (Fresh/Stale States)

**Summary:** Per-file, per-function cache with two states: Ready (fresh result) and Stale (previous result, file changed since). On file change: mark all entries Stale. On request: return Stale data with `_stale: true` flag while re-analysis runs. Eviction: closed files evicted after grace period, LRU eviction if total cache exceeds memory budget.

**Codeblock placeholder:** The AnalysisCache struct and get/set_ready/mark_all_stale methods.

### 7.4 CodeLens: Per-Function Ownership Summary

**Summary:** Above each function, display a CodeLens showing: "N vars, M borrows, K moves, J conflicts!" (red if conflicts > 0). Plus a memory CodeLens: "Stack: XB | Heap: ~YB | Z ptrs". Clicking triggers `borrowscope.showGraph` command to open the WebView panel.

**SVG placeholder:** Mock of VS Code editor showing CodeLens text above a function signature. Two lines: ownership stats and memory stats.

### 7.5 InlayHints: Inline Ownership Annotations

**Summary:** After variable names, show ownership category as inline hints: `[&]` for shared refs, `[&mut]` for mutable refs, `[Rc]`, `[Arc]`, `[Cell]`, `[*ptr]`, `[closure]`. Only shown for non-trivial categories (not for plain Owned or Copy). Filtered to the visible range for performance.

**SVG placeholder:** Mock of code with inline hints appearing after variable names in a different color.

### 7.6 Hover: Rich Ownership Tooltips

**Summary:** On hover over a variable declaration line, show markdown tooltip with: variable name, type, ownership category, Copy status, borrows-from list, borrowed-by list, moved-to destination, layout size. All information from the FunctionOwnershipSummary.

### 7.7 publishDiagnostics: Borrow Conflict Warnings

**Summary:** When conflicts are detected, publish them as LSP diagnostics (severity: Information). Each diagnostic includes: message explaining the conflict, range covering the overlap, relatedInformation pointing to both borrow sites. These appear as squiggly lines in the editor.

**Codeblock placeholder:** The `publish_diagnostics` function showing how conflicts become LSP diagnostics with relatedInformation.

### 7.8 VS Code WebView Panel Integration

**Summary:** The VS Code extension receives ownershipGraph responses and renders them in a WebView panel as an interactive force-directed graph. Nodes = variables (colored by ownership category), edges = borrows/moves/clones (styled by type). Panel updates in real-time as the user edits code (triggered by analysisUpdated notifications).

**SVG placeholder:** Full ownership graph visualization showing 5-6 nodes with different colors (blue=owned, green=shared ref, red=mutable ref, purple=Rc) connected by labeled edges (borrow, move, clone). Interactive: hover to highlight connected nodes.

---

## 8. Evaluation

### 8.1 Response Time Measurements

**Summary:** Measure time for each request type. Target: <100ms for ownershipGraph (single function), <50ms for codeLens (per file), <200ms for crossFunctionBorrows. Report: initial workspace loading time, incremental re-analysis time after edit, debounce-to-notification latency.

**Table placeholder:** Response time table (request type, p50, p95, p99).

### 8.2 Test Coverage (107 Protocol Tests)

**Summary:** 107 integration tests covering: lifecycle (initialize/shutdown/exit), text sync (open/change/close), custom requests (all 5), notifications (analysisUpdated filtering), debounce behavior, cache management, error handling, performance budgets. All tests run against the actual binary via stdio.

**Table placeholder:** Test category breakdown table.

### 8.3 Incremental Re-analysis Performance

**Summary:** After initial load, measure re-analysis time for: single-line edit, function body rewrite, new function added, file-level refactor. Show that Salsa's incremental computation keeps re-analysis under 100ms for typical edits.

### 8.4 Memory Usage and Cache Efficiency

**Summary:** Measure: cache size per function, total memory for 10/50/100 open files, eviction effectiveness. Show that LRU eviction keeps memory bounded even with many open files.

### 8.5 Comparison with Existing Tools

**Summary:** Feature and performance comparison with Aquascope, RustViz, Flowistry, rust-analyzer. Show that borrowscope-lsp is the only tool providing real-time + editor-native + full ownership analysis.

**Table placeholder:** Comparison table (tool x feature x performance).

---

## 9. Limitations and Future Work

### 9.1 Workspace Loading Latency (~30-40s)

**Summary:** Initial workspace loading takes 30-40s for medium projects. During this time, only cached/stale results are available. Mitigation: background loading with progress, immediate response with loading status. Future: persistent cache across sessions.

### 9.2 Trait Implementation Detection

**Summary:** Currently, trait_impls (Send, Sync, Clone, Drop) are not fully resolved - requires looking up traits by name in the database. The infrastructure exists (hir::Type methods) but trait lookup by well-known name is not yet implemented.

### 9.3 Generic Type Layout Estimation

**Summary:** For generic types like `Vec<T>` where T is a type parameter, layout.size() returns the monomorphized size. But for unresolved generics, layout may fail. Current behavior: return size=0 for unknown layouts.

### 9.4 Cross-Crate Borrow Tracking

**Summary:** Current cross-function tracking resolves callees within the same crate. Cross-crate resolution (into dependencies) requires loading dependency source, which is available in the Salsa DB but not yet traversed for borrow paths.

### 9.5 Planned: Runtime Event Merge

**Summary:** The VS Code extension already supports loading runtime events (memory-events.json) alongside static analysis. Future work: formal divergence detection (where static prediction differs from runtime observation), confidence scoring, and unified timeline view.

---

## 10. Conclusion

**Summary:** 3 paragraphs. (1) Restate the problem and solution. (2) Key results: 8 custom LSP requests, <100ms analysis, 107 tests, zero heuristics, field-level memory layout. (3) Impact: borrowscope-lsp makes Rust's ownership system visible in real-time, reducing cognitive load and enabling developers to understand ownership relationships without running the compiler.
