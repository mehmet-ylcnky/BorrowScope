# 2. Technical Architecture

## System Overview

BorrowScope operates as a multi-layer system that instruments Rust code, captures ownership events, and visualizes them interactively.

```
┌─────────────────────────────────────────────────────────────┐
│                      User's Rust Code                        │
│                   (annotated or auto-detected)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              INSTRUMENTATION LAYER                           │
│  ┌──────────────────┐         ┌─────────────────────┐      │
│  │ Procedural Macro │   OR    │  Cargo Subcommand   │      │
│  │ #[trace_borrow]  │         │  cargo borrowscope  │      │
│  └──────────────────┘         └─────────────────────┘      │
│         (syn + quote)              (rustc_driver)           │
└────────────────────────┬───────────────────────────────────┘
                         │ Injects tracking calls
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 RUNTIME TRACKER                              │
│  - Captures ownership events (new, borrow, move, drop)      │
│  - Builds ownership graph in memory                         │
│  - Exports to JSON/MessagePack                              │
│  - Optional: WebSocket streaming for live updates           │
└────────────────────────┬───────────────────────────────────┘
                         │ Event stream
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              VISUALIZATION ENGINE                            │
│  ┌─────────────────┐    ┌──────────────────────┐           │
│  │   Graph View    │    │    Timeline View     │           │
│  │  (D3.js/Cyto)   │    │  (Event sequence)    │           │
│  └─────────────────┘    └──────────────────────┘           │
│                                                              │
│  Frontend: Tauri (Rust + WebView) or Dioxus (WASM)         │
└─────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 2.1 borrowscope-macro (Procedural Macro Crate)
**Purpose:** Instrument Rust code at compile time to inject tracking logic.

**Responsibilities:**
- Parse function bodies using `syn`
- Identify variable declarations, borrows, moves, and drops
- Generate tracking calls via `quote!` macro
- Preserve original program semantics

**Input:** Rust source code with `#[trace_borrow]` attribute  
**Output:** Expanded code with embedded tracking calls

### 2.2 borrowscope-runtime (Core Library)
**Purpose:** Collect and manage ownership events during program execution.

**Responsibilities:**
- Provide tracking API: `track_new()`, `track_borrow()`, `track_move()`, `track_drop()`
- Build in-memory ownership graph (nodes = variables, edges = relationships)
- Serialize graph to JSON for visualization
- Optional: Stream events via WebSocket for real-time visualization

**Key data structures:**
- `Event`: Enum representing ownership operations
- `OwnershipGraph`: Graph structure with variables and relationships
- `Tracker`: Singleton managing event collection

### 2.3 borrowscope-cli (Cargo Subcommand)
**Purpose:** Provide seamless integration with Cargo workflow.

**Responsibilities:**
- Parse command-line arguments
- Instrument target source files automatically
- Compile and run instrumented code
- Launch visualization UI with generated data
- Clean up temporary files

**Commands:**
```bash
cargo borrowscope visualize <file>     # Visualize single file
cargo borrowscope run                  # Instrument and run entire project
cargo borrowscope export --format json # Export graph data only
```

### 2.4 borrowscope-ui (Visualization Frontend)
**Purpose:** Interactive visual representation of ownership and borrowing.

**Responsibilities:**
- Render ownership graph with interactive nodes/edges
- Display timeline of variable lifecycles
- Highlight borrow conflicts and errors
- Support zoom, pan, filtering
- Replay execution step-by-step

**Views:**
1. **Graph View:** Network diagram showing ownership relationships
2. **Timeline View:** Horizontal timeline with variable lifespans
3. **Code View:** Source code with highlighted active borrows
4. **Error Mode:** Simulate and visualize borrow checker errors

## Data Flow

1. **Compile Time:**
   - User annotates function with `#[trace_borrow]` or runs `cargo borrowscope`
   - Macro/CLI parses AST and injects tracking calls
   - Code compiles with instrumentation embedded

2. **Runtime:**
   - Instrumented code executes
   - Each ownership operation calls runtime tracker
   - Tracker builds ownership graph incrementally
   - Graph exported to JSON file or streamed via WebSocket

3. **Visualization:**
   - UI loads JSON data or connects to WebSocket
   - Renders initial graph state
   - User interacts: steps through timeline, explores relationships
   - UI updates dynamically based on event sequence

## Technology Choices

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Parsing | `syn`, `quote` | Industry standard for Rust macros |
| Graph | `petgraph` | Mature, efficient graph algorithms |
| Serialization | `serde_json` | Universal format, easy debugging |
| CLI | `clap` | Ergonomic, feature-rich argument parsing |
| UI Framework | Tauri or Dioxus | Native performance, Rust-first |
| Visualization | D3.js or Cytoscape.js | Powerful graph rendering libraries |
| Communication | WebSocket (optional) | Real-time streaming for live mode |
