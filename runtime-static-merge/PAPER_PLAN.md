# Merging Static and Runtime Analysis for Rust Ownership Visualization

## Detailed Paper Plan

---

## Abstract

**Summary:** 3 paragraphs. (1) Static analysis sees all paths but no runtime values; runtime sees one path with precise timing. Neither alone is sufficient. (2) This paper presents a merge algorithm that correlates runtime events (88 types) with static analysis results (from the LSP), detects 9 kinds of divergences, and surfaces them with severity levels and actionable suggestions. (3) Results: merge completes in <100ms for 1000 events, 9 divergence kinds with zero false positives on tested scenarios, validated by 64 merge/divergence tests.

---

## 1. Introduction

### 1.1 The Two Worlds of Ownership Analysis

**Summary:** Static analysis (LSP) predicts what *could* happen: all branches, all borrows, all possible conflicts. Runtime observation records what *did* happen: one execution path, actual reference counts, real timing, precise drop order. Each has blind spots the other can fill.

**SVG placeholder:** Split diagram showing "Static World" (all paths, instant, no values) vs "Runtime World" (one path, precise, actual counts). Animated arrows showing what each misses.

### 1.2 Why Neither Alone is Sufficient

**Summary:** Static analysis cannot detect: Rc cycles (needs runtime ref count observation), conditional moves (branch not taken), actual drop order (compiler may reorder), borrows held across await (needs execution trace). Runtime cannot detect: all possible conflicts (only sees executed path), ownership category (needs type resolution), cross-function borrow paths (needs semantic resolution).

**Table placeholder:** 2-column table showing "What static misses" vs "What runtime misses" with 5-6 examples each.

### 1.3 The Merge Hypothesis

**Summary:** By correlating static predictions with runtime observations, we can detect meaningful divergences that indicate bugs (Rc leaks, use-after-move), performance issues (unnecessary clones), or incomplete analysis (unsafe code hiding ownership). The merge produces a unified view where each variable shows both its predicted and observed behavior.

### 1.4 Contributions

**Summary:** 5 contributions: (1) merge algorithm correlating runtime IDs to static declarations, (2) 9 divergence detectors with severity and suggestions, (3) async borrow tracking across await points, (4) visualization layer showing agreement/divergence inline, (5) 64 tests validating merge correctness and divergence detection.

---

## 2. Background

### 2.1 Static Ownership Analysis (borrowscope-lsp)

**Summary:** Recap what the LSP provides per function: variables with ownership categories, borrow scopes with line ranges, moves with destinations, Rc/Arc clones, conflicts. Reference the LSP paper. Emphasize: this is the "prediction" side of the merge.

### 2.2 Runtime Event Tracking (borrowscope-runtime)

**Summary:** Recap what the runtime records: timestamped events for every ownership operation. 88 event types covering creation, borrows, moves, drops, Rc/Arc clones, RefCell borrows, unsafe blocks, async operations, channels, locks. Reference the runtime paper. Emphasize: this is the "observation" side.

### 2.3 The 88 Event Type Taxonomy

**Summary:** Categorize the 88 events into groups: core ownership (New, Borrow, Move, Drop), smart pointers (RcNew, RcClone, ArcNew, ArcClone, WeakNew, WeakUpgrade), interior mutability (RefCellBorrow, CellGet, CellSet), unsafe (RawPtrCreated, RawPtrDeref, UnsafeBlockEnter, Transmute), async (AsyncBlockEnter, AwaitStart, AwaitEnd), concurrency (ThreadSpawn, ChannelSend, LockGuardAcquire), control flow (FnEnter, FnExit, LoopEnter, Branch, Return).

**Codeblock placeholder:** The RuntimeEvent union type showing all 88 variants (abbreviated, showing 5-6 per category).

**SVG placeholder:** Interactive taxonomy tree. Click a category to expand and see its event types with their fields.

### 2.4 Related Work (Miri, Valgrind, Sanitizers, KLEE)

**Summary:** Miri: interprets MIR, detects UB, but slow and no visualization. Valgrind: memory errors at runtime, no ownership semantics. AddressSanitizer/ThreadSanitizer: detect memory/thread bugs, no ownership model. KLEE: symbolic execution, explores all paths but doesn't merge with runtime. BorrowScope is unique in merging static ownership analysis with runtime observation for visualization (not bug detection).

---

## 3. Event Ingestion

### 3.1 File-Based Ingestion (JSON Watcher)

**Summary:** The primary ingestion mode. A file watcher monitors `.borrowscope/events.json` for changes. When the file is written (after program execution), the watcher reads it, parses the JSON array, and triggers the merge pipeline. The watcher uses VS Code's `FileSystemWatcher` API with debouncing to avoid reading partial writes.

**Codeblock placeholder:** RuntimeWatcher class showing file watch setup and event emission.

### 3.2 WebSocket Live Streaming

**Summary:** Alternative mode for live observation during execution. The runtime connects to a WebSocket server (configurable port, default 9876) and streams events as they occur. The extension receives events in real-time and updates the visualization incrementally. Useful for long-running programs or servers where you want to see ownership patterns as they happen.

**Codeblock placeholder:** RuntimeSocket class showing WebSocket connection and message handling.

### 3.3 Event Validation and Parsing

**Summary:** Raw JSON is validated against the RuntimeEvent type. Each event must have a timestamp and type-specific fields. Invalid events are skipped with a warning (not fatal). The parser handles both formats: internally-tagged (`{"type":"New","var_name":"x",...}`) and externally-tagged (`{"New":{"var_name":"x",...}}`). This dual-format support ensures compatibility with different serialization modes of borrowscope-runtime.

**Codeblock placeholder:** The `eventType()` and `eventData()` helper functions that handle both formats.

### 3.4 Internally-Tagged vs Externally-Tagged Format Handling

**Summary:** Serde's default for Rust enums is externally-tagged (`{"VariantName":{fields}}`). But some configurations use internally-tagged (`{"type":"VariantName",fields}`). The parser detects which format is used by checking if the top-level object has a `type` field. This transparent handling means the extension works regardless of how the runtime is configured to serialize events.

---

## 4. Variable Mapping

### 4.1 The Correlation Problem (Runtime IDs vs Static Declarations)

**Summary:** Runtime events use IDs like `"data_0"`, `"r_1"`. Static analysis has variable declarations with names and line numbers. The challenge: match `"data_0"` to the `data: Vec<i32>` declared at line 5 of main.rs. Multiple variables can share the same name (shadowing), and runtime IDs include a counter suffix to disambiguate.

**SVG placeholder:** Interactive visualization showing runtime event stream on the left, static variable list on the right, with animated matching lines connecting them. Click to step through the matching algorithm.

### 4.2 Name + Line + File Matching Strategy

**Summary:** The mapping algorithm: (1) extract var_name from the runtime ID (strip the `_N` suffix), (2) find static variables with the same name in the same file, (3) if multiple matches (shadowing), use the declaration line closest to the event's location. The `mapVariables()` function returns a `MappedVariable[]` with each entry containing the runtime events and the matched static declaration.

**Codeblock placeholder:** The `mapVariables()` function showing the matching logic.

### 4.3 Handling Shadowed Variables

**Summary:** When `let x = 1; let x = x + 1;` produces runtime IDs `"x_0"` and `"x_1"`, the counter suffix disambiguates. The mapper matches `"x_0"` to the first declaration and `"x_1"` to the second by declaration order (decl_index). If the counter exceeds the number of declarations with that name, the variable is marked as unmapped.

### 4.4 Unmapped Variables (runtime_only and static_only)

**Summary:** Not all variables map cleanly. `runtime_only`: a runtime event references a variable not found in static analysis (e.g., compiler-generated temporaries, variables in macros). `static_only`: a static variable has no corresponding runtime events (e.g., the function was never called, or the variable is in a branch not taken). Both are included in the merged output with their respective agreement status.

---

## 5. The Merge Algorithm

### 5.1 MergedVariable Construction

**Summary:** For each mapped variable, construct a `MergedVariable` combining: identity (name, var_id, line, file), static_info (type, category, is_copy from LSP), runtime_info (aggregated from events), agreement status, and divergence list. Unmapped variables get null for the missing side.

**Codeblock placeholder:** The MergedVariable interface (from merge-views.ts).

### 5.2 RuntimeInfo Aggregation from Event Streams

**Summary:** For each mapped variable, iterate its events and aggregate: creation timestamp, drop timestamp, borrow count (shared + mutable), move status and destination, ref count peak and final, clone count, weak count, unsafe accesses, await crossings. Each event type contributes to specific fields via a switch statement.

**Codeblock placeholder:** The `buildRuntimeInfo()` function showing the event aggregation loop.

**SVG placeholder:** Animated event stream flowing into an aggregation box, with counters incrementing as events are processed. Click to step through events and watch RuntimeInfo fields update.

### 5.3 Drop Order Computation

**Summary:** Build a global drop order map by scanning all Drop events in timestamp order. Each variable gets a position (0, 1, 2, ...) indicating when it was destroyed relative to others. This reveals the actual LIFO destruction order, which may differ from what the developer expects (especially with early drops, nested scopes, or `std::mem::drop`).

**Codeblock placeholder:** The `buildDropOrder()` function.

### 5.4 Agreement Classification (match, diverge, runtime_only, static_only)

**Summary:** After building RuntimeInfo and running divergence detection: if no static_info -> "runtime_only". If no runtime_info -> "static_only". If divergences detected -> "diverge". Otherwise -> "match". This classification drives the visualization: green for match, yellow/red for diverge, gray for one-sided.

### 5.5 Merge Summary Statistics

**Summary:** The `mergeSummary()` function counts: total variables, matches, divergences, runtime_only, static_only. Displayed in the status bar and used by the extension to decide whether to show the runtime overlay (if 0 matches, runtime data likely doesn't correspond to the current code).

---

## 6. Divergence Detection

### 6.1 Detection Architecture (Per-Variable Analysis)

**Summary:** The `detectAllDivergences()` function runs all 9 checks on each MergedVariable independently. Each check produces zero or more `DetailedDivergence` entries with: kind, severity (info/warning/error), description, suggestion, and runtime_evidence. Checks are ordered from most severe (rc_leak) to least (conditional_move).

**Codeblock placeholder:** The `detectAllDivergences()` function signature and structure.

### 6.2 Rc/Arc Leak Detection (rc_leak)

**Summary:** Condition: static says Rc/Arc, runtime shows drop_timestamp < 0 (never dropped) AND ref_count_final > 0. Severity: error. This means the reference-counted value will never be freed. Suggestion: "Check for reference cycles. Consider using Weak references."

**SVG placeholder:** Ref count chart showing count going up (clones) but never reaching 0. Red warning indicator at the end.

### 6.3 Reference Cycle Detection (rc_cycle)

**Summary:** Condition: Rc/Arc, ref_count_peak > 1, clone_count >= 2, ref_count_final > 0, never dropped. Severity: error. Distinguishes from simple leak (single Rc forgotten) by requiring multiple clones that form a cycle. Suggestion: "Break the cycle with Weak<T> or restructure ownership."

### 6.4 Missing Drop Detection (missing_drop)

**Summary:** Condition: non-Copy, non-moved, never dropped, not Rc/Arc. Severity: warning. Possible causes: `std::mem::forget`, program exit before scope end, or panic unwinding. Suggestion: "This may indicate a leak, std::mem::forget, or program exit before scope end."

### 6.5 Async Borrow Held Across Await (async_borrow_held)

**Summary:** Condition: a Borrow event's owner_id has active borrows when an AwaitStart event fires. Severity: warning. This means a borrow is held across a suspension point, which may prevent the future from being Send. Suggestion: "Consider cloning before the await or restructuring to drop the borrow first."

**Codeblock placeholder:** The `detectAwaitCrossings()` function showing how active borrows are tracked across await points.

### 6.6 Unsafe Hidden Behavior (unsafe_hidden)

**Summary:** Condition: unsafe_accesses > 0 on a variable that is not classified as RawPointer. Severity: info. Static analysis cannot verify ownership inside unsafe blocks, so the runtime observation may reveal behavior the static analysis missed. Suggestion: "Review unsafe blocks for soundness."

### 6.7 Conditional Move Detection (conditional_move)

**Summary:** Condition: static says Owned (could be moved), non-Copy, but runtime shows: never moved, never borrowed, dropped normally. Severity: info. The move expression exists in code but the branch containing it was not taken in this execution. Suggestion: "The move may be in a branch that wasn't executed in this run."

### 6.8 Weak Upgrade Failure (weak_upgrade_fail)

**Summary:** Condition: WeakUpgrade event with success=false. Severity: warning. The strong reference was already dropped when Weak::upgrade was called. Suggestion: "Handle the None case from Weak::upgrade, or ensure the strong reference outlives the weak."

### 6.9 Channel Receive Failure (channel_recv_fail)

**Summary:** Condition: ChannelRecv event with success=false. Severity: warning. The sender was dropped before the receiver could read. Suggestion: "Ensure sender outlives receiver, or handle the RecvError."

### 6.10 Use After Move (use_after_move)

**Summary:** Condition: events with this var_id exist after a Move event from it (excluding Drop). Severity: error. Should not happen in safe Rust. Indicates unsafe code or instrumentation error. Suggestion: "This should not happen in safe Rust. Check for unsafe code or instrumentation errors."

### 6.11 Severity Classification and Actionable Suggestions

**Summary:** Three severity levels: error (rc_leak, rc_cycle, use_after_move - genuine bugs), warning (missing_drop, async_borrow_held, weak_upgrade_fail, channel_recv_fail - potential issues), info (unsafe_hidden, conditional_move - informational). Each divergence includes a suggestion string that tells the developer what to do, not just what went wrong.

**Table placeholder:** Full table of all 9 divergences with kind, severity, condition, and suggestion.

---

## 7. Visualization of Merged Data

### 7.1 Inline Runtime Decorations (Timing, Drop Order, Divergence Highlights)

**Summary:** Three decoration types applied to the editor: green timing annotations showing actual lifetime ("lived 4.2ms"), numbered drop order badges (#1, #2, #3), and colored divergence highlights (yellow for warnings, red for errors). Only shown when runtime.enabled = true.

**Screenshot placeholder:** Editor showing all three decoration types on a function.

### 7.2 Runtime View: Timeline Sub-tab

**Summary:** Horizontal bars showing each variable's actual lifetime (from New timestamp to Drop timestamp). Bars colored by category. Play/scrub control allows stepping through time. Borrow regions shown as overlays on target variables.

**Screenshot placeholder:** Timeline sub-tab with colored bars and play controls.

### 7.3 Runtime View: Drop Order Sub-tab

**Summary:** Numbered list showing the exact destruction sequence. Each entry: numbered badge, variable name, lifetime duration. Reveals actual LIFO order and highlights any deviations from expected order.

### 7.4 Runtime View: Reference Count Sub-tab

**Summary:** Step chart built from actual runtime events (RcNew, RcClone, Drop). Shows real strong_count at each point. Detects leaks where count never reaches zero. More accurate than the static ref count view (Section 4.7 of VS Code paper) because it includes clones from other functions.

**SVG placeholder:** Interactive step chart with event labels. Click events to see details. Red warning if final count > 0.

### 7.5 Runtime View: Event Log Sub-tab

**Summary:** Scrollable log of all runtime events with color-coded type badges, timestamps, and affected variable names. Click any event to navigate to the source line. Filterable by event type.

### 7.6 Status Bar Integration (Event Count, Divergence Badge)

**Summary:** Status bar item showing: "Static OK | Runtime OK (103 events, 2s ago)" when no divergences, or "Static OK | Runtime: 2 warnings" with yellow/red coloring when divergences exist. Click to open the Runtime view.

---

## 8. Async Borrow Tracking

### 8.1 The Problem: Borrows Held Across Await Points

**Summary:** In async Rust, a borrow that is active when `.await` is called must be stored in the future's state machine. If the borrow is mutable, the future cannot be Send (cannot be moved to another thread). This is a common source of confusing compiler errors. The runtime can detect exactly which borrows are held at each await point.

**Codeblock placeholder:** Example async function with a borrow held across await, showing the compiler error it would produce.

### 8.2 Detecting AwaitStart Events During Active Borrows

**Summary:** The algorithm maintains a set of active borrows (var_ids that have been borrowed but not yet dropped). When an AwaitStart event fires, any active borrows are recorded as await crossings. The crossing records: which await point (line), how long the await lasted (duration from AwaitStart to AwaitEnd), and which future was awaited.

**Codeblock placeholder:** The `detectAwaitCrossings()` function.

**SVG placeholder:** Timeline showing a borrow's lifetime with an await point in the middle. The borrow bar spans across the await gap, highlighted in yellow to show the crossing.

### 8.3 Duration Measurement and Future Identification

**Summary:** Each AwaitCrossing records: await_line (where the .await is), duration_ns (how long the suspension lasted), and future_name (which future was awaited). Duration is computed from AwaitEnd.timestamp - AwaitStart.timestamp. Long durations indicate the borrow is held for extended periods, increasing the risk of contention.

### 8.4 Implications for Send Trait Compliance

**Summary:** If a borrow held across await is mutable (&mut T where T is not Sync), the future is not Send. The divergence detector flags this as async_borrow_held with a suggestion to clone before the await or restructure. This directly explains the common "future is not Send" compiler error that confuses many Rust developers.

---

## 9. Evaluation

### 9.1 Merge Performance (Events per Second)

**Summary:** Measure merge time for different event counts: 100, 500, 1000, 5000 events. Target: <100ms for 1000 events. The merge is O(n) in event count (single pass aggregation) with O(m) variable mapping (m = number of unique variables).

**Table placeholder:** Event count vs merge time table.

### 9.2 Divergence Detection Accuracy

**Summary:** Test each divergence kind with crafted scenarios: Rc that leaks, Rc cycle, variable forgotten with mem::forget, async borrow across await, unsafe ptr deref, conditional move in if-else, Weak upgrade after strong dropped, channel with dropped sender, move then use in unsafe. All 9 detected correctly with zero false negatives.

### 9.3 False Positive Analysis

**Summary:** Run the merge on well-behaved programs (no bugs). Verify: zero divergences reported for correct code. The conditional_move detector is the most likely source of false positives (it flags variables that could have been moved but weren't), which is why it's severity "info" not "warning".

### 9.4 Real-World Divergence Examples

**Summary:** Show 2-3 real examples from the demo project or open-source crates where the merge detected something useful: an Rc that was cloned but one clone was never dropped (leak), a borrow held across an await in a tokio handler, a channel receiver that fails because the sender task panicked.

---

## 10. Limitations and Future Work

### 10.1 Single Execution Path Limitation

**Summary:** Each runtime trace captures one execution path. Branches not taken, error paths not triggered, and rare race conditions are invisible. Multiple runs with different inputs would be needed for coverage.

### 10.2 Thread Interleaving Sensitivity

**Summary:** Multi-threaded programs may produce different event orderings on each run. The merge results depend on which interleaving was observed. A divergence detected in one run may not reproduce in another.

### 10.3 Planned: Multi-Run Aggregation

**Summary:** Future work: aggregate events from multiple runs to build a probabilistic model. "This Rc leaks in 3/10 runs" is more informative than "leaked in this run." Requires storing and comparing multiple event files.

### 10.4 Planned: Confidence Scoring for Divergences

**Summary:** Future work: assign confidence scores to divergences based on how many events support the conclusion. An rc_leak with 5 clones and 0 drops is high confidence. A conditional_move with 1 event is low confidence.

### 10.5 Planned: Integration with borrowscope-macro for Automated Instrumentation

**Summary:** Future work: the VS Code extension triggers `#[trace_borrow]` instrumentation, compilation, and execution from a single command. The developer clicks "Run with BorrowScope" and the merge happens automatically. No manual steps needed.

---

## 11. Conclusion

**Summary:** 3 paragraphs. (1) Restate the problem: static and runtime analysis are complementary but disconnected. (2) The merge algorithm correlates them, detects 9 divergence kinds, and surfaces actionable suggestions. (3) Impact: developers get the best of both worlds in a single visualization, catching bugs (Rc leaks, use-after-move) that neither analysis alone can find.
