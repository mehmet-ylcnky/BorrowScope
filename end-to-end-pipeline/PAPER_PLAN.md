# End-to-End Ownership Instrumentation Pipeline for Rust

## Detailed Paper Plan

---

## Abstract

**Summary:** This paper presents the complete BorrowScope pipeline as a unified system: from source code to visual feedback in one click. The analyzer extracts type-info.json, the macro instruments code using that type information, the runtime records events during execution, the LSP provides real-time static analysis, and the VS Code extension merges both into interactive visualizations. A new "Run Instrumented" command orchestrates the entire flow from a single status bar button, eliminating all manual steps. The paper documents the data flow between components, the type-info.json schema as the central bridge, performance budgets across all stages, and the 2,100+ tests validating the ecosystem.

---

## 1. Introduction

### 1.1 The Vision: One Click from Source to Visualization

**Summary:** The developer writes Rust code, clicks one button, and sees ownership visualized. No terminal commands, no configuration, no manual steps. This is the end state that the pipeline achieves.

**Screenshot placeholder:** The VS Code status bar showing "▶ BorrowScope" button (idle state).

### 1.2 The Five Components

**Summary:** Brief overview of each component's role: analyzer (type extraction), macro (instrumentation), runtime (event recording), LSP (static analysis), VS Code extension (visualization + merge). Each was presented in a separate paper; this paper shows how they connect.

**SVG placeholder:** Interactive pipeline diagram showing 5 components as boxes with data flowing between them. Click each component to highlight its inputs and outputs.

### 1.3 Why a Unified Pipeline Matters

**Summary:** Without integration, the developer must: (1) run analyzer manually, (2) add macros manually, (3) compile and run, (4) check events file, (5) enable runtime overlay. With the pipeline: click one button. The paper demonstrates that all 5 components can be orchestrated automatically.

### 1.4 Contributions

**Summary:** (1) One-click E2E command with status bar UI, (2) type-info.json as the schema bridge between analyzer and macro, (3) performance budget across all stages, (4) 2,100+ tests across the ecosystem, (5) zero heuristics throughout the entire pipeline.

---

## 2. Pipeline Architecture

### 2.1 Data Flow Overview

**Summary:** Source code -> analyzer -> type-info.json -> macro (at compile time) -> instrumented binary -> runtime events -> events.json -> VS Code merge -> visualization. Each arrow is a well-defined data format.

**SVG placeholder:** Animated data flow diagram. Boxes for each component, arrows showing data format between them (type-info.json, events.json, LSP JSON-RPC). Animated dots flowing along arrows.

### 2.2 The Orchestration Command (borrowscope.runInstrumented)

**Summary:** The VS Code command that ties everything together: resolves binary paths, spawns analyzer, spawns cargo run, monitors progress, enables runtime overlay on completion. Status bar shows live state.

**Codeblock placeholder:** The `executeE2E()` function showing the orchestration flow.

**Screenshot placeholder:** The status bar cycling through states: "⏳ Analyzing..." -> "✓ 18 events"

### 2.3 Component Independence

**Summary:** Each component can be used independently (analyzer without macro, LSP without runtime, etc.). The pipeline is additive: more components = richer visualization, but any subset works. This modularity enables incremental adoption.

---

## 3. The type-info.json Bridge

### 3.1 Schema Design

**Summary:** The type-info.json file contains per-variable semantic information: name, type, file, line, 40+ boolean flags (is_rc, is_arc, is_box, is_weak, is_closure...), initializer_kind (rc_new, arc_new, cow_borrowed...), binding_mode, deref_chain. This is the contract between analyzer and macro.

**Codeblock placeholder:** Example type-info.json entry showing all fields for an Rc variable.

### 3.2 How the Analyzer Produces It

**Summary:** The analyzer loads the workspace via ra_ap_load_cargo, resolves types for every let binding using hir::Type methods, classifies each variable, and writes the JSON file. 94-100% type resolution on typical projects.

**Codeblock placeholder:** The analyzer's main flow: load workspace -> iterate functions -> extract types -> write JSON.

### 3.3 How the Macro Consumes It

**Summary:** At compile time, the macro's proc_macro_attribute reads type-info.json from CARGO_MANIFEST_DIR/.borrowscope/. For each let binding, it looks up the variable by (function_name, var_name, decl_index) and uses initializer_kind to decide which track_* function to emit. Without type-info.json, the macro falls back to generic tracking.

**Codeblock placeholder:** The macro's lookup function showing how it reads and dispatches based on type-info.

### 3.4 Staleness and Refresh

**Summary:** type-info.json becomes stale when code changes. The E2E command solves this by always running the analyzer first. In manual mode, the developer must re-run the analyzer after editing. The LSP's Salsa database is never stale (it's always live), which is why the LSP was created as an alternative.

---

## 4. Instrumentation via Macro

### 4.1 The #[trace_borrow] Attribute

**Summary:** Applied to a function, transforms each ownership operation into a track_* call. Uses type-info.json to make semantic decisions: Rc::clone gets track_rc_clone (not generic track_clone), Cow::Borrowed gets track_cow_borrowed (not generic track_new). 82 initializer categories supported.

**Codeblock placeholder:** Before/after showing a function with and without #[trace_borrow] expansion.

### 4.2 Semantic Dispatch (82 Categories)

**Summary:** The macro dispatches to specific tracking functions based on the initializer_kind field from type-info.json: rc_new, arc_new, box_new, weak_new, refcell_new, cow_borrowed, cow_owned, join_handle, channel_new, atomic_new, etc. Each produces a type-specific runtime event with relevant metadata (strong_count, weak_count, etc.).

### 4.3 Zero-Cost When Disabled

**Summary:** The runtime crate uses a "track" feature flag. When disabled, all track_* functions compile to identity functions (return their argument unchanged). No runtime overhead, no binary size increase. This allows instrumented code to be committed to the repository without affecting production builds.

---

## 5. Runtime Event Generation

### 5.1 Event Recording Architecture

**Summary:** Thread-local event buffer (no locking for single-threaded recording), nanosecond timestamps via std::time::Instant, automatic var_id generation (name_counter format). Events are accumulated during execution and exported at program exit.

### 5.2 The 88 Event Types in Practice

**Summary:** Show which events are generated for the E2E test project: New (variable creation), Borrow (reference creation), RcNew/RcClone (Rc operations), Drop (destruction), Move (ownership transfer). Map each source line to the events it produces.

**Codeblock placeholder:** The E2E test project's main.rs alongside the 18 events it produces, showing the correspondence.

### 5.3 Export to JSON

**Summary:** At program exit (or on demand), events are serialized to JSON via serde and written to .borrowscope/events.json. The file is written atomically (write to temp + rename) to avoid partial reads by the file watcher.

---

## 6. Static Analysis via LSP

### 6.1 Parallel Analysis (No Instrumentation Needed)

**Summary:** The LSP provides ownership analysis without requiring instrumentation. It runs in parallel with the runtime pipeline, providing instant static feedback while the developer waits for the instrumented run to complete. After the run, the extension merges both.

### 6.2 What the LSP Provides That Runtime Cannot

**Summary:** All-path analysis (sees every branch), cross-function borrow tracking (resolves call targets semantically), memory layout (field-level detail from type information), and borrow conflicts (detects overlapping scopes). None of these require execution.

### 6.3 What Runtime Provides That the LSP Cannot

**Summary:** Actual reference counts, precise timing, real drop order, which branches were taken, async suspension durations, and channel/lock contention patterns. These require execution.

---

## 7. The Merge: Combining Both Worlds

### 7.1 Automatic Merge After Pipeline Completion

**Summary:** When the E2E command completes, the file watcher detects events.json, the merge runs automatically, and the visualization updates. The developer sees: static decorations (from LSP, instant) enriched with runtime observations (from events, after run).

### 7.2 Divergence Detection in the Pipeline Context

**Summary:** With the full pipeline, divergences are especially valuable: the analyzer ensures type-info is fresh, the macro ensures instrumentation is correct, and the runtime ensures events are complete. Any divergence detected is therefore highly reliable (not caused by stale data).

### 7.3 Enhanced Memory Visualization

**Summary:** With the pipeline active, the memory view shows both static estimates (from LSP's ty.layout) AND runtime observations (actual heap sizes, allocation origins from type-info.json's init_kind). The init_kind field enables the visualization to distinguish Rc::new allocations from Box::new allocations from Vec::with_capacity pre-allocations.

---

## 8. Performance Budget

### 8.1 Per-Stage Timing

**Summary:** Measure each stage: analyzer (30-40s one-time), macro expansion (0ms additional compile time without track feature), cargo build (normal), cargo run (program execution + ~75ns per track_* call), LSP analysis (<100ms per function), merge (<100ms for 1000 events), visualization render (<50ms).

**Table placeholder:** Stage | Time | Bottleneck | Amortized?

### 8.2 End-to-End Latency

**Summary:** From button click to visualization update: analyzer (30-40s, one-time if type-info is fresh) + compile + run (project-dependent) + merge (<100ms) + render (<50ms). For the E2E test project: ~35s first run, ~3s subsequent runs (analyzer cached, only recompile + run).

### 8.3 Overhead of Instrumentation

**Summary:** With track feature enabled: ~75ns per ownership event, ~5-10% total overhead for typical programs. With track feature disabled: zero overhead (identity functions inlined away). The overhead is acceptable for development but not production.

---

## 9. Test Coverage Across the Ecosystem

### 9.1 Per-Component Test Counts

**Summary:** Analyzer: 11 integration tests (100s each). Macro: 563 tests (187 unit + 376 integration). Runtime: 775 tests. LSP: 107 protocol tests. VS Code: 721 tests (including 18 E2E pipeline tests). Graph: test count. Total: 2,100+.

**Table placeholder:** Component | Unit Tests | Integration Tests | Total

### 9.2 Cross-Component Validation

**Summary:** The E2E test project validates the full chain: analyzer produces type-info -> macro reads it correctly -> runtime generates expected events -> merge produces correct MergedVariables. If any component breaks the contract, the E2E test fails.

### 9.3 Zero Heuristics Guarantee

**Summary:** Throughout the entire pipeline, no decision is made based on string matching, naming conventions, or guesswork. Analyzer uses hir::Type semantic APIs. Macro uses type-info.json (semantically resolved). LSP uses ra_ap_* APIs. Extension uses structured JSON responses. The "Eliminating Heuristics" paper documents this property; the E2E pipeline preserves it end-to-end.

---

## 10. Limitations and Future Work

### 10.1 First-Run Analyzer Latency

**Summary:** The analyzer takes 30-40s on first run (workspace loading). Subsequent runs are faster if type-info.json is still fresh. Future: incremental analyzer that watches for file changes.

### 10.2 Manual #[trace_borrow] Annotation

**Summary:** Currently, the developer must add #[trace_borrow] to each function they want instrumented. Future: automatic annotation via a cargo subcommand or VS Code quick-fix.

### 10.3 Planned: Selective Instrumentation UI

**Summary:** A VS Code quick-pick menu: "What do you want to track? [All] [Rc/Arc only] [Moves only] [Borrows only]" that sets the macro's configuration before running.

---

## 11. Conclusion

**Summary:** 3 paragraphs. (1) The pipeline connects 5 components into a one-click workflow. (2) Key results: type-info.json bridge, zero heuristics, 2,100+ tests, <100ms merge time. (3) Impact: developers get complete ownership visibility without leaving the editor, combining the breadth of static analysis with the precision of runtime observation.
