# borrowscope-vscode: Interactive Ownership Visualization for Rust in VS Code

## Detailed Paper Plan

---

## Abstract

**Summary:** This paper presents borrowscope-vscode, a VS Code extension that consumes the borrowscope-lsp server output and renders it as 11 interactive visualization views in a WebView panel. The extension bridges the gap between raw LSP analysis data and developer-facing visual feedback. It provides inline decorations (lifelines, gutter icons, inlay hints), a D3.js force-directed ownership graph, timeline views, memory layout scrubbing, and a runtime-static merge layer with divergence detection. The extension is the user-facing layer of the BorrowScope ecosystem, turning compiler-grade ownership analysis into actionable visual information. Frame it as the natural continuation after the LSP paper: the LSP provides the data, this extension provides the experience.

---

## 1. Introduction

### 1.1 From Data to Experience
- The LSP server produces structured JSON (variables, borrows, moves, clones, conflicts, memory layout, cross-function paths)
- But JSON is not actionable for a developer. It needs visual representation.
- This paper presents the visualization layer that transforms LSP responses into interactive editor feedback.

### 1.2 Design Goals
- Real-time: updates as the developer types (triggered by analysisUpdated notifications)
- Non-intrusive: decorations are subtle, panel is opt-in (click CodeLens to open)
- Multi-modal: 11 views for different aspects of ownership (graph, timeline, scopes, memory, etc.)
- Linked: clicking a node in the graph navigates to the source line; hovering a table row highlights the graph node
- Accessible: ARIA labels, keyboard navigation, screen reader descriptions, high-contrast support
- Configurable: 33 settings covering every visual aspect

### 1.3 Relationship to borrowscope-lsp
- The extension is a pure consumer of LSP data. It makes no ownership decisions itself.
- All analysis is done by the LSP server. The extension only renders and navigates.
- The analysisUpdated notification triggers re-rendering. The extension caches graphs per function.
- Table showing: what the LSP provides vs what the extension renders from it.

### 1.4 Contributions
- 11 interactive visualization views in a single WebView panel
- Inline editor decorations (lifelines, gutter icons, conflict markers, cross-function annotations)
- Runtime-static merge with divergence detection (9 divergence kinds)
- 689 tests covering all views, decorations, and interactions
- 33 configuration settings, 6 keyboard shortcuts, 27 commands

---

## 2. Architecture

### 2.1 Extension Activation and Lifecycle
- activate(): starts LSP client, runtime watcher, status bar, registers commands
- deactivate(): stops client, disposes watcher
- Subscription management via context.subscriptions

### 2.2 LSP Client Connection
- LanguageClient from vscode-languageclient/node
- ServerOptions: spawns borrowscope-lsp binary via stdio
- Listens for analysisUpdated and publishDiagnostics notifications
- Debounced refresh on editor change (150ms for editor switch, 300ms for text change)
- Pre-fetches all functions in background to warm Salsa cache

### 2.3 Graph Cache and State Management
- graphCache: Map<string, any> per function name
- Invalidated on analysisUpdated notification
- Stale data served while fresh analysis runs
- Previous graph stored for comparison view

### 2.4 Configuration System (33 Settings)
- 9 groups: server, analysis, decorations, graph, colors, crossFunction, memoryLayout, runtime, diagnostics
- Typed BorrowScopeConfig interface
- Live reload on configuration change (no restart needed)
- Color customization for all ownership categories

**Codeblock placeholder:** The BorrowScopeConfig interface showing all 33 settings.

---

## 3. Inline Editor Decorations

### 3.1 Ownership Category Annotations (InlayHints)
- Color-coded labels after variable names: [&], [&mut], [Rc], [Arc], [Cell], [*ptr], [closure]
- Only shown for non-trivial categories (not Owned, not Copy)
- Uses TextEditorDecorationType with after.contentText
- Colors configurable via borrowscope.colors.*

**SVG placeholder:** Dark-theme editor mock showing colored annotations inline.

### 3.2 Borrow Lifelines (Gutter Decorations)
- Vertical lines in the gutter showing borrow scope duration
- Characters: `├─` (start), `│` (active), `╰─` (end)
- Emoji labels: 👁 (shared borrow), 🔒 (mutable borrow), 💧 (released), ❄ (frozen target), ↦ (move), ─┘ (dead)
- Color-coded: blue for shared, red for mutable, orange for moves, yellow for conflicts, purple for Rc clones
- Hover messages explain each decoration

**SVG placeholder:** Editor gutter showing lifeline characters with emoji labels.

### 3.3 Conflict Zone Highlighting
- Background highlight on lines where borrows overlap
- Triggered by publishDiagnostics from LSP
- Squiggly underline + background color on conflict region
- Linked to next/prev conflict navigation commands

### 3.4 Cross-Function Borrow Annotations
- Inline annotations after call sites: `──→ 👁 &data enters process(param)`
- Shows which borrows escape into called functions
- Color: teal (rgba(26, 188, 156, 0.6))
- Hover shows full cross-function borrow path

### 3.5 Runtime Overlay Decorations
- Green timing annotations from runtime events
- Drop order markers (numbered badges)
- Divergence highlights (yellow for warnings, red for errors)
- Only shown when runtime.enabled = true and events file exists

---

## 4. The WebView Panel (11 Views)

### 4.1 Landing Page and View Selection
- Circular icon buttons in a 4x3 grid for each view
- Each button has emoji icon, label, and tooltip description
- Click opens the selected view. Home button returns to landing.
- Last selected view persisted across sessions via workspaceState

**SVG placeholder:** The landing page grid with all 11 view icons.

### 4.2 Force-Directed Ownership Graph (D3.js)
- Nodes: variables colored by ownership category (10 colors)
- Edges: borrows (dashed), moves (solid), clones (dotted), captures (dash-dot)
- Arrow markers per edge kind
- Zoom, pan, drag nodes
- Click node: navigate to source line
- Hover node: tooltip with type, category, line, Copy status
- Linked highlighting: hover graph node highlights table row and vice versa

**SVG placeholder:** Interactive graph with 6 nodes and 4 edges, showing hover tooltip.

### 4.3 Filter Bar (Category Filtering)
- Buttons for each ownership category present in the graph
- Click to hide/show nodes of that category
- Hidden categories shown with strikethrough and reduced opacity
- Edges connected to hidden nodes also hidden

### 4.4 Table View
- Collapsible sections: Variables, Borrow Scopes, Moves, Rc/Arc Clones, Conflicts
- Each row is clickable (navigates to source line)
- Hover row highlights corresponding graph node
- Shows type, category, line number for each variable

### 4.5 Timeline View (Gantt Chart)
- Horizontal bars showing variable lifetimes (start line to end line)
- Borrow overlays on target variables (blue for shared, red for mutable)
- Conflict zones as red vertical bands
- X-axis: line numbers. Y-axis: variable names.
- Click bar: navigate to source line
- Hover bar: highlight in table

**SVG placeholder:** Gantt chart with 5 variables, borrow overlays, and a conflict zone.

### 4.6 Scope Nesting View
- Nested boxes showing function scope and inner block scopes
- Variables listed inside their containing scope
- Drop order shown at bottom (reverse declaration order)
- Click variable: navigate to source line

### 4.7 Reference Count View
- Step chart showing strong_count over time for each Rc/Arc variable
- X-axis: line numbers. Y-axis: reference count.
- Events: new (count=1), clone (count++), drop (count--)
- Leak warning if final count > 0
- Dots at each event with labels

**SVG placeholder:** Step chart showing ref count going 1→2→3→2→1→0.

### 4.8 Move Chain View
- Source box (crossed out, opacity reduced) → arrow → destination box
- Shows type, line number, alive/dead status
- Click boxes: navigate to source line

### 4.9 Conflict Detail View
- Card per conflict: variable name, borrow A vs borrow B, overlap lines
- "Go to line" link for navigation
- Green checkmark if no conflicts

### 4.10 Comparison View (Diff)
- Side-by-side comparison of previous and current function state
- Shows: added variables, removed variables, added borrows, removed borrows
- Green for additions, red with strikethrough for removals
- Summary badge: "+2 vars, -1 borrow"

### 4.11 Cross-References View
- Left panel: file tree showing which files have cross-function borrows
- Right panel: D3 graph with function nodes and borrow edges
- Click function node: navigate to that function in the editor
- Edge labels show variable name and mutability

### 4.12 Memory Layout View
- Stack column: variables with offset, size, alignment, internal fields
- Heap column: allocations with owner, estimated size, capacity bar
- Pointer relationships: arrows from stack to heap
- Play/scrub timeline: step through lines to see stack frame evolve
- Dropped variables shown with strikethrough
- Runtime mode: actual hex addresses from execution

**SVG placeholder:** Stack/heap split view with pointer arrows and field-level detail.

### 4.13 Runtime View (4 Sub-tabs)
- Timeline: horizontal bars with play/scrub, borrow overlays
- Drop Order: numbered list with lifetime duration
- Ref Count: step chart from runtime events
- Events: scrollable event log with type badges and timestamps
- Click any item: navigate to source line

---

## 5. Runtime-Static Merge

### 5.1 The Merge Problem
- Static analysis (LSP) sees all paths but no runtime values
- Runtime events see one path with precise timing and counts
- Need to correlate: which runtime var_id corresponds to which static variable?
- Matching by name + line + file (via runtime-mapper.ts)

### 5.2 MergedVariable Structure
- Combines static_info (type, category, is_copy) with runtime_info (lifetime, borrow count, ref counts, drop order, await crossings)
- Agreement field: "match", "diverge", "runtime_only", "static_only"
- Divergences list with kind, description, suggestion

### 5.3 Runtime Event Processing
- 88 event types parsed from JSON (file watcher or WebSocket)
- Events filtered by file, then by ownership-relevant types
- Mapped to static variables via name/line correlation
- Drop order computed globally across all events

### 5.4 Divergence Detection (9 Kinds)
- rc_leak: Rc/Arc never dropped, ref_count > 0
- rc_cycle: ref count went up, never came back to 0
- missing_drop: non-Copy, non-moved, never dropped
- async_borrow_held: borrow active across await point
- unsafe_hidden: unsafe accesses that static analysis cannot verify
- conditional_move: static predicted move, runtime didn't (branch not taken)
- weak_upgrade_fail: Weak::upgrade returned None
- channel_recv_fail: channel receive failed (sender dropped)
- use_after_move: events after move (should not happen in safe Rust)

**SVG placeholder:** Divergence detection flow: static vars + runtime events → merge → divergence list with severity badges.

### 5.5 Divergence Visualization
- Yellow background for warnings, red for errors
- Inline annotations showing divergence description
- Status bar showing divergence count
- Detailed panel in Runtime view

---

## 6. Performance and Responsiveness

### 6.1 Debounced Refresh Pipeline
- Editor change → 300ms debounce → refresh decorations
- Editor switch → 150ms debounce → refresh
- analysisUpdated notification → immediate graph cache invalidation + refresh
- Pre-fetch all functions in background (parallel requests)

### 6.2 Performance Monitoring
- PerformanceMonitor class: records timing per operation
- Slow threshold: 100ms (logged to output channel)
- Performance report command: shows avg/max/min/count per operation
- File size guard: skip analysis for files > 10,000 lines

### 6.3 Graph Cache Strategy
- Per-function cache (graphCache: Map<string, any>)
- Invalidated on analysisUpdated
- Previous graph stored for comparison view
- Workspace state persistence for panel restoration

---

## 7. Accessibility and Theming

### 7.1 ARIA Labels and Screen Reader Support
- Graph nodes have role="button", tabindex="0", aria-label with variable info
- Graph description (aria-live region) summarizes the entire graph in text
- Keyboard navigation: Enter on focused node navigates to source

### 7.2 Theme Integration
- 10 custom theme colors (borrowscope.sharedBorrow, borrowscope.mutableBorrow, etc.)
- Adapts to light, dark, and high-contrast themes
- CSS variables from VS Code theme (--vscode-editor-background, etc.)
- High-contrast mode: thicker borders, larger stroke widths

### 7.3 Keyboard Shortcuts (6)
- Ctrl+Shift+B: toggle decorations
- Ctrl+Shift+G: focus graph panel
- Ctrl+Shift+N: next conflict
- Ctrl+Shift+P: previous conflict
- Ctrl+Shift+T: show timeline
- Ctrl+Shift+M: show memory layout

---

## 8. Testing

### 8.1 Test Architecture
- 689 tests across 42 test files
- Mocha test runner with VS Code mock (__mocks__/vscode.js)
- Tests run without VS Code instance (pure unit tests on logic)
- Each source file has a corresponding test file

### 8.2 Test Categories
- Extension lifecycle (activate/deactivate)
- Client connection and notification handling
- All 11 panel views (graph model, timeline, scopes, refcount, moves, conflicts, compare, crossrefs, memory, runtime)
- Decorations (inlay hints, lifelines, highlights, conflicts)
- Runtime integration (watcher, parser, mapper, socket, status, merge, divergence)
- Configuration (all 33 settings, live reload)
- Commands (all 27 commands)
- Accessibility (ARIA labels, keyboard nav)
- Performance (debouncer, monitor, file size guard)
- Keybindings and theme integration

### 8.3 Test Coverage Table
- Table showing: module, test count, what is tested

---

## 9. Evaluation

### 9.1 Rendering Performance
- Graph render time for N nodes (10, 50, 100 variables)
- Timeline render time
- Memory view scrub responsiveness
- Decoration application time

### 9.2 User Experience Metrics
- Time from keystroke to visual update (debounce + LSP + render)
- Number of clicks to reach any ownership information
- Information density per view (what can be learned at a glance)

---

## 10. Limitations and Future Work

### 10.1 WebView Isolation
- WebView runs in a separate iframe, cannot directly access editor state
- Communication via postMessage (adds latency for navigation)
- Cannot overlay graph directly on source code

### 10.2 D3.js Bundle Size
- d3.min.js adds ~250KB to extension package
- Could be replaced with lighter alternatives for simple graphs

### 10.3 Runtime Event File Dependency
- Runtime merge requires the instrumented program to be executed first
- Events file must be in the expected location (.borrowscope/events.json)
- WebSocket mode requires the program to be running

### 10.4 Planned Enhancements
- Live collaboration: share ownership graphs between team members
- Git integration: show ownership changes between commits
- AI-assisted suggestions: recommend ownership patterns based on analysis
- Custom view plugins: allow users to create their own visualization views

---

## 11. Conclusion

**Summary:** 3 paragraphs. (1) The extension transforms raw LSP data into 11 interactive views. (2) Key results: 689 tests, 33 settings, 27 commands, runtime-static merge with 9 divergence kinds. (3) Together with borrowscope-lsp, it provides the complete developer experience for understanding Rust ownership in real-time.

---

## Interactive SVG Visualizations (embedded in sections)

- Section 3.1: Dark-theme editor with colored inlay hint annotations
- Section 3.2: Editor gutter with lifeline characters and emoji labels
- Section 4.1: Landing page grid with 11 view icons
- Section 4.2: Force-directed graph with nodes, edges, tooltip
- Section 4.5: Timeline Gantt chart with borrow overlays
- Section 4.7: Reference count step chart
- Section 4.12: Memory layout stack/heap split with pointer arrows
- Section 5.4: Divergence detection pipeline flow
