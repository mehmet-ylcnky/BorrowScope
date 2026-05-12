# borrowscope-vscode: Implementation Plan

## Overview

A VS Code extension that provides real-time ownership visualization by running a persistent rust-analyzer-based language server. The server loads the workspace once, maintains a live semantic database using `ra_ap_*` crates, and serves ownership queries instantly on every keystroke. No separate build step. No JSON files. The same UX as rust-analyzer's type hints, but for ownership relationships.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          VS Code                                 │
│                                                                  │
│  ┌────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Editor Pane   │  │  WebView Panel  │  │  Problems Panel  │  │
│  │                │  │                 │  │                  │  │
│  │  - Borrow      │  │  - Live         │  │  - Conflicts     │  │
│  │    scope       │  │    ownership    │  │  - Use-after-    │  │
│  │    highlights  │  │    graph        │  │    move          │  │
│  │  - Gutter      │  │  - Interactive  │  │                  │  │
│  │    icons       │  │    D3.js/SVG    │  │                  │  │
│  │  - CodeLens    │  │                 │  │                  │  │
│  └───────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│          │                     │                     │           │
│  ┌───────┴─────────────────────┴─────────────────────┴────────┐ │
│  │           Extension Frontend (TypeScript)                    │ │
│  │                                                             │ │
│  │  - Receives ownership data via LSP custom notifications     │ │
│  │  - Renders decorations, graph, diagnostics                  │ │
│  │  - Sends cursor position / active file to server            │ │
│  └─────────────────────────────┬───────────────────────────────┘ │
└────────────────────────────────┼─────────────────────────────────┘
                                 │ LSP (JSON-RPC over stdio)
                                 │
┌────────────────────────────────┴─────────────────────────────────┐
│              BorrowScope Language Server (Rust binary)             │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  ra_ap_* Engine (same as rust-analyzer)                       │ │
│  │                                                               │ │
│  │  - RootDatabase (full semantic model of the project)          │ │
│  │  - Salsa incremental computation (re-analyzes only changes)   │ │
│  │  - Sysroot + dependency resolution                            │ │
│  │  - All 55 hir::Type methods available                         │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  Ownership Analysis Layer                                     │ │
│  │                                                               │ │
│  │  - Variable classification (all 55 Type methods)              │ │
│  │  - Method call resolution (self_borrow, canonical path)       │ │
│  │  - Borrow scope computation                                   │ │
│  │  - Conflict detection (overlapping &mut / &)                  │ │
│  │  - Move tracking                                              │ │
│  │  - Closure capture analysis                                   │ │
│  │  - Rc/Arc reference counting                                  │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  LSP Server                                                   │ │
│  │                                                               │ │
│  │  - Standard LSP: diagnostics, code actions                    │ │
│  │  - Custom requests: ownership graph, borrow scopes            │ │
│  │  - Custom notifications: push updates on file change          │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

**Key design decision:** This is a standalone language server, not a plugin to rust-analyzer. rust-analyzer does not support third-party extensions. However, it uses the same `ra_ap_*` crates and the same workspace loading logic, so it benefits from all rust-analyzer improvements automatically when the crate versions are bumped.

---

## Milestone 1: Language Server Scaffold

### 1.1 Rust Binary Project Setup (borrowscope-lsp)

### 1.2 LSP Protocol Implementation (tower-lsp or lsp-server crate)

### 1.3 Workspace Loading with ra_ap_* (RootDatabase, sysroot discovery)

### 1.4 Server Lifecycle (initialize, initialized, shutdown)

### 1.5 Text Document Synchronization (open, change, save, close)

### 1.6 Incremental Re-analysis via Salsa (only re-analyze changed files)

### 1.7 Startup Performance (background loading, progress notifications)

### 1.T Testing: Server starts, loads workspace, responds to initialize

---

## Milestone 2: Ownership Analysis Engine

### 2.1 Exhaustive Type Extraction (all 55 hir::Type methods per variable)

### 2.2 Method Call Resolution (resolve_method_call, self_borrow, canonical path)

### 2.3 Borrow Scope Computation (start line, end line, mutability)

### 2.4 Move Detection (ownership transfers between variables)

### 2.5 Closure Capture Analysis (captured variables, capture mode)

### 2.6 Rc/Arc Clone Tracking (reference count at each point)

### 2.7 Conflict Detection (overlapping mutable/shared borrows)

### 2.8 Per-Function Ownership Summary (variables, borrows, moves, drops)

### 2.T Testing: Analysis produces correct results for known patterns

---

## Milestone 3: LSP Custom Requests and Notifications

### 3.1 Custom Request: `borrowscope/ownershipGraph` (returns graph for a function)

### 3.2 Custom Request: `borrowscope/borrowScopes` (returns borrow ranges for a file)

### 3.3 Custom Request: `borrowscope/variableInfo` (returns full type info for a position)

### 3.4 Custom Notification: `borrowscope/analysisUpdated` (pushed on file change)

### 3.5 Standard LSP: `textDocument/publishDiagnostics` (borrow conflicts)

### 3.6 Standard LSP: `textDocument/codeLens` (borrow counts above functions)

### 3.7 Standard LSP: `textDocument/inlayHints` (ownership annotations inline)

### 3.T Testing: Requests return correct JSON, notifications fire on change

---

## Milestone 4: VS Code Extension Frontend

### 4.1 Extension Project Setup (TypeScript, package.json, activation)

### 4.2 Language Client Configuration (connect to borrowscope-lsp binary)

### 4.3 Server Binary Management (bundled binary, auto-download, version check)

### 4.4 Inline Decorations from Inlay Hints (ownership category next to variables)

### 4.5 Gutter Icons (borrow start/end, move indicators)

### 4.6 Borrow Scope Highlighting (colored background regions)

### 4.7 Diagnostics Display (conflicts in Problems panel)

### 4.8 CodeLens Rendering (borrow counts, click to expand)

### 4.T Testing: Extension connects to server, decorations render correctly

---

## Milestone 5: Ownership Graph WebView

### 5.1 WebView Panel Registration and Lifecycle

### 5.2 Graph Data Model (nodes, edges from LSP response)

### 5.3 Rendering Engine (D3.js force-directed or dagre hierarchical)

### 5.4 Node Styling (colored by type: Rc=purple, &=blue, &mut=red, move=green)

### 5.5 Edge Styling (dashed borrows, solid moves, dotted clones)

### 5.6 Interaction: Click Node to Navigate to Source

### 5.7 Interaction: Hover for Full Type Info

### 5.8 Interaction: Filter by Scope / Type Category

### 5.9 Live Update (graph re-renders when server pushes analysisUpdated)

### 5.10 Function Selector (dropdown to switch between functions)

### 5.T Testing: Graph renders, updates live, navigation works

---

## Milestone 6: Real-Time Incremental Updates

### 6.1 Salsa Incremental Computation (only re-analyze touched functions)

### 6.2 Debounced Analysis (wait 300ms after last keystroke before re-analyzing)

### 6.3 Partial Results (show what's available while analysis is in progress)

### 6.4 Diff-Based UI Updates (only re-render changed nodes/edges)

### 6.5 Performance Budget (analysis must complete in < 100ms for single-file changes)

### 6.6 Memory Management (evict analysis for closed files)

### 6.T Testing: Typing in editor updates graph within 500ms, no flicker

---

## Milestone 7: Advanced Visualizations

### 7.1 Temporal View: Borrow Lifetime Timeline (Gantt-chart style per function)

### 7.2 Scope Nesting View (nested boxes showing variable containment)

### 7.3 Reference Count History (line chart for Rc/Arc over function body)

### 7.4 Move Chain View (trace value through ownership transfers)

### 7.5 Conflict Highlight Mode (red overlay on conflicting borrow regions)

### 7.6 Comparison View (side-by-side before/after for refactoring)

### 7.T Testing: Each visualization renders correctly for sample functions

---

## Milestone 8: Configuration and Polish

### 8.1 Extension Settings (colors, layout, enabled features, performance tuning)

### 8.2 Keyboard Shortcuts (toggle panel, jump to next conflict, focus variable)

### 8.3 Command Palette (full command list with descriptions)

### 8.4 Theme Integration (respect VS Code light/dark/high-contrast themes)

### 8.5 Welcome View and Onboarding (first-time setup, Rust toolchain detection)

### 8.6 Performance Profiling and Optimization

### 8.7 Accessibility (screen reader support, keyboard navigation in graph)

### 8.T Testing: Settings apply, themes work, accessible navigation

---

## Milestone 9: Coexistence with rust-analyzer

### 9.1 Shared Workspace Loading (avoid duplicate sysroot/dependency resolution)

### 9.2 Complementary Diagnostics (don't duplicate rust-analyzer's borrow checker errors)

### 9.3 Hover Integration (extend rust-analyzer hover with ownership info)

### 9.4 Go-to-Definition Awareness (ownership graph follows navigation)

### 9.5 Semantic Token Coordination (don't conflict with RA's syntax highlighting)

### 9.6 Resource Sharing Strategy (memory budget when both servers run)

### 9.T Testing: Both extensions run simultaneously without conflicts

---

## Milestone 10: Publishing and Distribution

### 10.1 VS Code Marketplace Listing (icon, screenshots, demo GIF)

### 10.2 Extension Bundling (esbuild for TypeScript, binary per platform)

### 10.3 Platform Binaries (Linux x64, macOS arm64/x64, Windows x64)

### 10.4 Auto-Update for Server Binary

### 10.5 Minimum Rust Toolchain Detection

### 10.6 Documentation (README, user guide, troubleshooting)

### 10.7 Release Pipeline (GitHub Actions: build, test, package, publish)

### 10.T Testing: Installs from marketplace on all platforms, binary downloads work

---

## Dependencies

### Server (Rust)

| Crate | Purpose |
|-------|---------|
| `ra_ap_hir` | Semantic type information (hir::Type, 55 methods) |
| `ra_ap_ide` | High-level analysis APIs |
| `ra_ap_project_model` | Cargo.toml / workspace loading |
| `ra_ap_vfs` | Virtual file system (file watching) |
| `ra_ap_load_cargo` | Workspace + sysroot loading |
| `lsp-server` or `tower-lsp` | LSP protocol implementation |
| `serde` / `serde_json` | JSON-RPC serialization |
| `crossbeam` | Concurrent message passing |

### Frontend (TypeScript)

| Package | Purpose |
|---------|---------|
| `vscode-languageclient` | LSP client |
| `d3` or `dagre-d3` | Graph rendering in WebView |
| `@vscode/vsce` | Extension packaging |

---

## Phasing Strategy

| Phase | Milestones | User Experience |
|-------|-----------|-----------------|
| Phase 1: Working Server | M1 + M2 | Server loads workspace, extracts ownership data |
| Phase 2: Basic UI | M3 + M4 | Graph panel + inline decorations (static, on-demand) |
| Phase 3: Live | M5 + M6 | Graph updates in real-time as user types |
| Phase 4: Rich Viz | M7 | Timeline, scope nesting, ref count charts |
| Phase 5: Polish | M8 + M9 | Settings, themes, RA coexistence |
| Phase 6: Ship | M10 | Marketplace, cross-platform, auto-update |

---

## Key Design Decisions

### Why a separate server, not a rust-analyzer plugin?

rust-analyzer has no plugin system. It's a monolithic binary. The only way to extend it is to fork it or build a companion server. A companion server is cleaner: independent release cycle, independent memory budget, no risk of breaking RA updates.

### Why ra_ap_* crates, not LSP queries to rust-analyzer?

The standard LSP protocol does not expose `hir::Type` methods. You can get hover text (a string) but not structured type information (is_copy, impls_trait, as_adt). Using `ra_ap_*` directly gives access to all 55 methods with full fidelity.

### Why not share rust-analyzer's database?

rust-analyzer's `RootDatabase` is in-process and not shareable across processes. Two separate servers must each load the workspace independently. The cost is ~500MB-1GB additional memory. This is the same trade-off that other companion tools (rust-clippy, miri) make.

### Can we reduce the duplicate workspace loading?

Potentially, by loading only the sysroot and reusing rust-analyzer's cached dependency metadata. This is a Milestone 9 optimization, not a launch blocker.
