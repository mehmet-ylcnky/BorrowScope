# BorrowScope: Publication Plan

## Published Papers

| # | Title | Component | Date | Link |
|---|-------|-----------|------|------|
| 1 | Technical White Paper: Architecture & Design | borrowscope-runtime | Dec 2025 | [Link](borrowscope-runtime/) |
| 2 | Automating Instrumentation with a Procedural Macro | borrowscope-macro | Jan 2026 | [Link](borrowscope-macro-intro/) |
| 3 | Rust's Type Information Barrier | borrowscope-macro | Feb 2026 | [Link](borrowscope-macro-whitepaper/) |
| 4 | Bridging the Type Information Gap | borrowscope-analyzer | Mar 2026 | [Link](borrowscope-analyzer-whitepaper/) |
| 5 | Eliminating Heuristics from Rust Procedural Macros (v3.0) | borrowscope-analyzer + macro | May 2026 | [Link](eliminating-heuristics-from-rust-proc-macros/) |
| 6 | Ownership Graph Construction from Runtime Event Streams | borrowscope-graph | May 2026 | [Link](ownership-graph-construction/) |
| 7 | Battle-Testing BorrowScope on Real-World Crates | All | May 2026 | [Link](battle-test-whitepaper/) |
| 8 | Real-Time Ownership Visualization via Language Server Protocol | borrowscope-lsp | Jun 2026 | [Link](lsp-based-ownership-visualization/) |
| 9 | Interactive Ownership Visualization for Rust in VS Code | borrowscope-vscode | Jun 2026 | [Link](vscode-ownership-visualization/) |

---

## Planned Papers

### Paper 10: Merging Static and Runtime Analysis for Ownership Visualization

**Component:** borrowscope-vscode (runtime integration)

**Abstract:** Deep dive into combining compile-time ownership analysis (from the LSP) with runtime execution data (from instrumented code) into a unified visualization. Covers the merge algorithm, 88 event types, variable mapping strategy, all 9 divergence detectors with formal definitions, and evaluation on real-world Rust programs.

**Key Contributions:**
- 88 runtime event types consumed from file or WebSocket
- Variable mapping: runtime IDs to static declarations by name + line
- 9 divergence detection kinds with severity levels and suggestions
- Rc/Arc reference count timeline with leak and cycle detection
- Drop order analysis with LIFO verification
- Async borrow tracking across await points
- Enhanced memory visualization via type-info.json integration

**Status:** Ready to write

---

### Paper 11: End-to-End Ownership Instrumentation Pipeline

**Component:** All (analyzer -> macro -> runtime -> LSP -> VS Code)

**Abstract:** Presents the complete BorrowScope pipeline as a unified system. The static analyzer extracts type information into type-info.json, the proc macro uses it to instrument code with tracking calls, the runtime generates events, the LSP provides real-time static analysis, and the VS Code extension merges both into interactive visualizations. Evaluates the full pipeline on real Rust projects.

**Key Contributions:**
- Complete pipeline: analyzer (207 variables) -> macro (563 tests) -> runtime (88 event types) -> LSP (5 endpoints) -> VS Code (11 views)
- type-info.json as the bridge between static analysis and runtime instrumentation
- Enhanced memory visualization: actual heap sizes, allocation origins, real addresses from runtime
- Performance budget across all components
- 2,100+ tests across the ecosystem
- Zero heuristics throughout the entire pipeline

**Status:** Ready to write

---

## Publication Timeline

| Paper | Status | Date |
|-------|--------|------|
| Papers 1-7 | Published | Dec 2025 - May 2026 |
| Paper 8: LSP Server | Published | Jun 2026 |
| Paper 9: VS Code Extension | Published | Jun 2026 |
| Paper 10: Runtime-Static Merge | Planned | TBD |
| Paper 11: End-to-End Pipeline | Planned | TBD |

---

## Total Publication Count

- **Published:** 9 papers
- **Planned:** 2 papers
- **Grand total:** 11 papers covering the complete BorrowScope ecosystem
