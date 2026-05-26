<div align="center">
  <img src="logo.png" alt="BorrowScope Logo" width="400"/>
  
  > Real-time ownership visualization for Rust — static analysis + runtime tracking + VS Code integration

  [![CI](https://github.com/mehmet-ylcnky/BorrowScope/actions/workflows/ci.yml/badge.svg)](https://github.com/mehmet-ylcnky/BorrowScope/actions)
  [![codecov](https://codecov.io/gh/mehmet-ylcnky/BorrowScope/branch/main/graph/badge.svg)](https://codecov.io/gh/mehmet-ylcnky/BorrowScope)
  [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
  [![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
  [![Tests](https://img.shields.io/badge/tests-2100%2B%20passing-brightgreen.svg)](https://github.com/mehmet-ylcnky/BorrowScope)
  
  📄 [Read the Technical Whitepapers](https://mehmet-ylcnky.github.io/BorrowScope/)
</div>

---

## What is BorrowScope?

BorrowScope is a comprehensive Rust ownership visualization platform that combines **static analysis**, **runtime tracking**, and a **VS Code extension** to make Rust's ownership and borrowing system visible. It provides:

- **Real-time inline annotations** in your editor showing ownership categories (`[&]`, `[&mut]`, `[Rc]`, `[Arc]`)
- **Interactive visualization panel** with 11 different views (Graph, Timeline, Memory, Runtime, etc.)
- **Cross-function borrow tracking** across call boundaries and files
- **Memory layout visualization** with field-level detail (ptr/len/cap for String/Vec)
- **Runtime event overlay** showing actual execution behavior alongside static analysis
- **Reference count timeline** for Rc/Arc with leak detection

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        VS Code Extension (borrowscope-vscode)            │
│                                                                          │
│  ┌──────────────────────┐    ┌──────────────────────────────────────┐   │
│  │   Editor Decorations  │    │   WebView Panel (11 views)            │   │
│  │   • Inline hints      │    │   • Force-directed graph              │   │
│  │   • Lifeline flows    │    │   • Timeline, Scopes, RefCount        │   │
│  │   • CodeLens stats    │    │   • Moves, Conflicts, Compare         │   │
│  │   • Borrow highlights │    │   • CrossRefs, Memory, Runtime        │   │
│  └──────────┬────────────┘    └──────────────┬────────────────────────┘   │
│             │                                 │                           │
├─────────────┼─────────────────────────────────┼───────────────────────────┤
│             ▼                                 ▼                           │
│  ┌─────────────────────┐    ┌────────────────────────────────────────┐   │
│  │  borrowscope-lsp     │    │  borrowscope-runtime (optional)         │   │
│  │  (Language Server)   │    │  (Runtime Event Tracking)               │   │
│  │                      │    │                                         │   │
│  │  • Ownership graph   │    │  • 88 event types                       │   │
│  │  • Borrow scopes     │    │  • Timing & drop order                  │   │
│  │  • Memory layout     │    │  • Ref count tracking                   │   │
│  │  • Cross-function    │    │  • Memory addresses                     │   │
│  │  • Conflict detect   │    │  • Async borrow tracking                │   │
│  └─────────────────────┘    └────────────────────────────────────────┘   │
│         ▲                              ▲                                  │
│         │ ra_ap_* (rust-analyzer)      │ #[trace_borrow] macro            │
└─────────┼──────────────────────────────┼──────────────────────────────────┘
          │                              │
    ┌─────┴──────────┐           ┌──────┴───────────┐
    │ borrowscope-   │           │ borrowscope-      │
    │ analyzer       │           │ macro             │
    │ (Static types) │           │ (Auto-instrument) │
    └────────────────┘           └───────────────────┘
```

## Components

| Component | Description | Lines | Tests |
|-----------|-------------|-------|-------|
| **borrowscope-runtime** | Runtime tracking library (88 event types, JSON export) | ~8,800 | 775 |
| **borrowscope-macro** | `#[trace_borrow]` proc macro for automatic instrumentation | ~8,800 | 533 |
| **borrowscope-analyzer** | Static analysis tool (type extraction for macro) | ~2,000 | — |
| **borrowscope-lsp** | Language server (ownership analysis via ra_ap_* APIs) | ~2,800 | 107 |
| **borrowscope-vscode** | VS Code extension (decorations, panel, runtime overlay) | ~6,900 | 689 |

**Total: ~29,000 lines of code, 2,100+ tests**

## VS Code Extension Features

### Editor Decorations
- Colored inline annotations: `[&]` `[&mut]` `[Rc]` `[Arc]` `[Cell]`
- Lifeline flow lines: `├─` `│` `╰─` with emoji labels (👁 🔒 💧 ❄ ↦ ─┘)
- Background highlights for borrow regions (blue/red tint)
- Hover tooltips with ownership details
- CodeLens stats above functions: `▸ 5 vars, 2 borrows, 1 move`
- Memory CodeLens: `🧠 Stack: 72B | Heap: ~28B | 3 ptrs`
- Cross-function annotations: `──→ 👁 &var enters fn(param)`

### Visualization Panel (11 Views)
| View | Icon | Description |
|------|------|-------------|
| Graph | 🕸️ | Force-directed ownership graph (D3.js) |
| Table | ▦ | Tabular view of all variables |
| Timeline | ⏱️ | Chronological ownership events |
| Scopes | 🔍 | Nested borrow scope visualization |
| RefCount | 🔗 | Rc/Arc reference count over time |
| Moves | ↦ | Ownership transfer chains |
| Conflicts | ⚠️ | Borrow conflict detection |
| Compare | 🪞 | Side-by-side function comparison |
| CrossRefs | 🔀 | Cross-function borrow tracking |
| Memory | 🧠 | Stack/heap layout with field details |
| Runtime | 🔬 | Runtime events, drop order, divergences |

### Runtime Integration (Optional)
When `borrowscope-runtime` is used, the extension shows:
- Green timing annotations: `⏱ 1.2ms (3×&)`
- Red divergence highlights: `⚡ Rc never dropped`
- Drop order visualization: `💀 #1, #2, #3` (LIFO)
- Ref count chart with leak detection
- Async borrow tracking across await points
- Actual hex addresses in Memory tab

## Quick Start

### 1. VS Code Extension (Static Analysis Only)

No instrumentation needed — works immediately on any Rust project:

```bash
# Build the language server
cd borrowscope-lsp
cargo build --release

# Open VS Code, set the server path in settings:
# "borrowscope.server.path": "/path/to/target/release/borrowscope-lsp"
```

Open any `.rs` file → see inline annotations, CodeLens, and the visualization panel.

### 2. With Runtime Tracking (Optional)

Add to your `Cargo.toml`:
```toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }
borrowscope-macro = "0.1"  # Optional: for automatic instrumentation
```

#### Manual Tracking
```rust
use borrowscope_runtime::*;

fn main() {
    reset();
    let data = track_new("data", vec![1, 2, 3]);
    let r = track_borrow("r", &data);
    println!("{:?}", r);
    track_drop("r");
    track_drop("data");
    
    // Export events for VS Code
    std::fs::create_dir_all(".borrowscope").ok();
    let events = get_events();
    std::fs::write(".borrowscope/events.json",
        serde_json::to_string_pretty(&events).unwrap()).unwrap();
}
```

#### Automatic Instrumentation
```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let data = vec![1, 2, 3];  // Automatically tracked
    let r = &data;              // Borrow tracked
    println!("{:?}", r);
}                               // Drops tracked
```

> **Note:** The macro requires running `borrowscope-analyzer` first. See [Static Analysis](#static-analysis) section.

### 3. Enable Runtime Overlay in VS Code

In `.vscode/settings.json`:
```json
{
  "borrowscope.runtime.enabled": true,
  "borrowscope.runtime.source": "file",
  "borrowscope.runtime.filePath": ".borrowscope/events.json"
}
```

Run your program → VS Code detects the events file → runtime overlay appears.

## Runtime Event Types (88 total)

| Category | Events |
|----------|--------|
| **Ownership** | New, Drop, Borrow, Move, Clone |
| **Smart Pointers** | RcNew, RcClone, ArcNew, ArcClone, WeakNew, WeakClone, WeakUpgrade, BoxNew, BoxIntoRaw, BoxFromRaw |
| **Interior Mutability** | RefCellNew, RefCellBorrow, RefCellDrop, CellNew, CellGet, CellSet |
| **Unsafe** | RawPtrCreated, RawPtrDeref, UnsafeBlockEnter/Exit, UnsafeFnCall, FfiCall, Transmute |
| **Async** | AsyncBlockEnter/Exit, AwaitStart/End, FutureCreate, FuturePoll |
| **Control Flow** | FnEnter/Exit, LoopEnter/Iteration/Exit, MatchEnter/Arm/Exit, Branch, Return, Break, Continue |
| **Concurrency** | ThreadSpawn/Join, ChannelSend/Recv, LockGuardAcquire/Drop, PinNew/IntoInner |
| **Memory** | StackAddr, StackField, HeapAddr, HeapRealloc, StackPadding |
| **Other** | StructCreate, TupleCreate, ArrayCreate, ClosureCreate/Capture, Deref, IndexAccess, FieldAccess |

## Memory Layout Visualization

The Memory tab shows field-level detail for all types — no runtime needed:

```
STACK (72B)                          HEAP
┌─────────────────────────┐
│ name: String      24B    │         ┌──────────────┐
│  .ptr  *const u8  (8B)  │────────▶│ "Alice" (5B) │
│  .len  usize     (8B)   │         │ cap: 8       │
│  .cap  usize     (8B)   │         └──────────────┘
├─────────────────────────┤
│ scores: Vec<f64>  24B    │         ┌──────────────┐
│  .ptr  *const T   (8B)  │────────▶│ [9.5, 8.7]   │
│  .len  usize     (8B)   │         │ cap: 4       │
│  .cap  usize     (8B)   │         └──────────────┘
└─────────────────────────┘
```

Supports: String, Vec, Box, Rc, Arc, Option, Result, RefCell, Cell, HashMap, user structs (via `ty.fields(db)`).

## Static Analysis (borrowscope-analyzer)

Required for the `#[trace_borrow]` macro:

```bash
cargo run -p borrowscope-analyzer -- /path/to/your/project
```

Generates `.borrowscope/type-info.json` with semantic type data that the macro uses for accurate instrumentation.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+O` | Show Ownership Graph |
| `Ctrl+Shift+I` | Inspect Variable at Cursor |
| `Ctrl+Shift+D` | Toggle Decorations |
| `Alt+Shift+N` | Next Borrow Conflict |
| `Alt+Shift+P` | Previous Borrow Conflict |
| `Ctrl+Shift+G` | Focus Graph Panel |

## Configuration

33 settings organized into groups:

| Group | Key Settings |
|-------|-------------|
| **Server** | `server.path`, `server.extraArgs` |
| **Analysis** | `analysis.debounceMs` (0-2000ms) |
| **Decorations** | `enabled`, `borrowScopes`, `gutterIcons`, `inlayHints`, `codeLens`, `lifelines` |
| **Graph** | `layout` (force/hierarchical/radial), `showTypes`, `animateUpdates` |
| **Colors** | `sharedBorrow`, `mutableBorrow`, `move`, `rcArc`, `owned`, `drop` |
| **Cross-Function** | `enabled`, `maxDepth`, `showInline` |
| **Memory** | `enabled`, `showAlignment`, `animationSpeed` |
| **Runtime** | `enabled`, `source` (file/websocket), `showTimings`, `showDropOrder`, `showRefCounts`, `highlightDivergences` |
| **Diagnostics** | `enabled`, `severity` (information/hint/warning) |

## Project Structure

```
BorrowScope/
├── borrowscope-runtime/     # Core tracking library (88 event types)
│   └── src/tracker/         # Modules: core, smart_pointers, async, unsafe, memory...
│
├── borrowscope-macro/       # #[trace_borrow] proc macro (133 instrumentation points)
│   └── src/                 # transform_visitor.rs, config.rs, type_info.rs
│
├── borrowscope-analyzer/    # Static analysis tool (type extraction)
│   └── src/                 # main.rs, analysis.rs, output.rs
│
├── borrowscope-lsp/         # Language server (ra_ap_* based)
│   ├── src/
│   │   ├── analysis.rs      # Ownership analysis, memory layout, cross-function
│   │   ├── handlers/        # LSP request/notification handlers
│   │   ├── server.rs        # Main loop, debounce, dispatch
│   │   └── state.rs         # Global state, caching
│   └── tests/               # 107 protocol tests
│
├── borrowscope-vscode/      # VS Code extension
│   ├── src/
│   │   ├── extension.ts     # Activation, commands, runtime wiring
│   │   ├── client.ts        # LSP client, decorations, lifelines
│   │   ├── graph/panel.ts   # WebView panel (11 views, D3.js)
│   │   ├── config.ts        # Typed configuration accessor
│   │   ├── runtime-*.ts     # Runtime integration (watcher, parser, mapper, socket)
│   │   ├── merge-views.ts   # Static + runtime merge with divergence detection
│   │   ├── performance.ts   # Performance monitoring
│   │   └── welcome.ts       # Onboarding flow
│   └── src/test/suite/      # 689 tests
│
├── examples/                # Standalone example projects
│   ├── ownership-patterns/
│   ├── smart-pointers/
│   ├── borrow-conflicts/
│   ├── async-ownership/
│   ├── graph-visualization/
│   └── allocator-sim/
│
└── logo.png                 # Project logo
```

## Performance

**Runtime library:**
- ~75-80ns per tracking call
- ~80 bytes per event
- Zero overhead without `track` feature

**VS Code extension:**
- < 5ms per ownership graph request (typical)
- < 100ms for cross-function analysis
- Debounced updates (configurable, default 300ms)
- Performance report via Command Palette

## Testing

```bash
# Runtime library
cargo test -p borrowscope-runtime --features track

# Macro
cargo test -p borrowscope-macro

# Language server
cargo test -p borrowscope-lsp

# VS Code extension
cd borrowscope-vscode && npm run build && npx tsc --noEmit false && npx mocha --require ./test-setup.js
```

## Related Work

| Project | Description |
|---------|-------------|
| [Aquascope](https://github.com/cognitive-engineering-lab/aquascope) | Compile-time and runtime visualizations for Rust |
| [Boris](https://github.com/ChristianSchott/boris) | Standalone ownership and borrowing visualizer |
| [Flowistry](https://github.com/willcrichton/flowistry) | Information flow analysis for Rust in VS Code |
| [REVIS](https://github.com/weirane/vscode-revis) | Lifetime-related error visualization |

BorrowScope differs by combining **static analysis** (LSP-based, no instrumentation needed) with **runtime tracking** (actual execution data) in a unified VS Code experience.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

<div align="center">
  <strong>Making Rust's ownership system visible — from source to execution.</strong>
</div>
