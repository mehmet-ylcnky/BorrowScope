# borrowscope-lsp

> Language server for real-time Rust ownership visualization — powered by rust-analyzer's semantic engine

## Overview

`borrowscope-lsp` is a Language Server Protocol (LSP) implementation that provides real-time ownership analysis for Rust code. It uses the same `ra_ap_*` compiler infrastructure as rust-analyzer to perform deep semantic analysis — resolving types, borrow scopes, moves, conflicts, cross-function borrows, and memory layouts — then serves this data to the VS Code extension via custom LSP requests.

**No instrumentation needed.** Works immediately on any Rust project.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      borrowscope-lsp                          │
│                                                              │
│  ┌──────────────┐    ┌──────────────────────────────────┐   │
│  │  LSP Server   │    │  Analysis Engine                  │   │
│  │  (lsp-server) │    │                                   │   │
│  │               │    │  • Ownership graph                │   │
│  │  • stdio I/O  │    │  • Borrow scopes (NLL)           │   │
│  │  • Debounce   │    │  • Move detection                │   │
│  │  • Caching    │    │  • Conflict detection            │   │
│  │  • Background │    │  • Cross-function borrows        │   │
│  │    loading    │    │  • Memory layout (field-level)   │   │
│  └──────┬────────┘    │  • Rc/Arc clone tracking         │   │
│         │             │  • Closure capture analysis       │   │
│         ▼             └──────────────────────────────────┘   │
│  ┌──────────────┐                                            │
│  │  State        │    ┌──────────────────────────────────┐   │
│  │               │    │  Workspace (ra_ap_*)              │   │
│  │  • Open files │    │                                   │   │
│  │  • Cache (LRU)│    │  • RootDatabase (Salsa)          │   │
│  │  • Debounce   │    │  • Semantics (type inference)    │   │
│  │  • Workspace  │    │  • VFS (virtual file system)     │   │
│  └──────────────┘    └──────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         ▲ stdio (JSON-RPC)
         │
    VS Code Extension
```

## Custom LSP Requests

| Request | Description | Response |
|---------|-------------|----------|
| `borrowscope/ownershipGraph` | Full ownership analysis for a function | Variables, borrows, moves, conflicts, Rc clones, closures |
| `borrowscope/borrowScopes` | Borrow scope regions with NLL | Start/end lines, borrower/owner, kind |
| `borrowscope/variableInfo` | Detailed info for variable at cursor | Type, ownership category, traits, lifecycle |
| `borrowscope/crossFunctionBorrows` | Inter-procedural borrow tracking | Borrow paths across call boundaries + file resolution |
| `borrowscope/memoryLayout` | Stack/heap layout with field detail | Sizes, offsets, alignment, ptr/len/cap fields |

## Standard LSP Features

| Feature | Description |
|---------|-------------|
| **Hover** | Ownership category + type info on hover |
| **CodeLens** | `▸ N vars, N borrows, N moves` + `🧠 Stack: NB \| Heap: ~NB` |
| **Inlay Hints** | `[&]`, `[&mut]`, `[Rc]`, `[Arc]`, `[Cell]` inline |
| **Text Sync** | Full document sync with debounced re-analysis |

## Analysis Capabilities

### Ownership Graph (`analyze_function`)

For each function, extracts:
- **Variables**: name, type, line, ownership category, is_copy, traits
- **Borrow Scopes**: borrower → owner, kind (shared/mutable), start/end lines (NLL)
- **Moves**: source → destination, line, type
- **Conflicts**: overlapping mutable + shared borrows
- **Rc/Arc Clones**: clone chains with source tracking
- **Closures**: captured variables with capture mode (move/ref/ref_mut)
- **Function Stats**: var count, borrow count, move count, clone count

### Cross-Function Borrows (`analyze_cross_function_borrows`)

Tracks borrows that flow across function call boundaries:
- Resolves call targets via `sema.resolve_method_call` / `sema.resolve_path`
- Follows borrow paths through multiple call levels (configurable depth)
- Resolves actual file paths via `HasSource` for cross-file navigation
- Performance guards: max depth, max borrows per function

### Memory Layout (`analyze_memory_layout`)

Field-level memory visualization using `ty.layout(db)`:
- Stack frame: variable sizes, offsets, alignment
- Heap allocations: estimated sizes for String/Vec/Box/Rc/Arc
- Pointer relationships: which stack vars point to heap
- **Field decomposition** for all types:
  - String/Vec: ptr, len, cap (8B each)
  - Box/Rc/Arc: ptr
  - References: ptr
  - Option/Result: discriminant + value
  - RefCell/Cell: internal structure
  - HashMap: ctrl, bucket_mask, items, growth_left
  - User structs: all fields via `ty.fields(db)` with computed offsets
- Variable end-of-life via `find_last_use` (NLL)
- Timeline slider support (line-based appear/disappear)

### Borrow Scope Computation

Uses Non-Lexical Lifetimes (NLL) semantics:
- `Definition::usages()` to find last use of each borrow
- Scopes end at last use, not at lexical scope boundary
- Handles nested borrows and reborrowing

### Move Detection

Semantic move detection via:
- `ty.is_copy(db)` — Copy types don't move
- Assignment analysis — `let y = x` where x is non-Copy
- Function call arguments — passing by value
- Pattern matching — destructuring moves

### Conflict Detection

Finds overlapping borrow scopes:
- Shared + mutable borrow of same owner overlapping
- Multiple mutable borrows overlapping
- Reports overlap region (start_line, end_line)

## Server Features

### Background Loading
- Workspace loaded in a background thread
- Server responds to requests immediately (returns empty during loading)
- Progress notifications sent to client

### Debounced Analysis
- Configurable debounce (default 300ms, reads from client settings)
- `debounce_ms = 0` means immediate analysis
- Pending changes flushed after debounce timer expires

### Analysis Caching
- Per-file, per-function LRU cache
- Results marked stale on file change (still served while re-analyzing)
- Memory-bounded with eviction
- Cache cleared on file close

## Code Structure

```
src/
├── main.rs              (61 lines)    Entry point, connection setup
├── server.rs            (161 lines)   Main loop, debounce, background loading
├── state.rs             (297 lines)   GlobalState, AnalysisCache, OpenFile
├── capabilities.rs      (24 lines)    LSP capability declaration
├── workspace.rs         (35 lines)    WorkspaceData (db + vfs)
├── handlers/
│   ├── mod.rs           (30 lines)    Handler dispatch
│   ├── requests.rs      (627 lines)   Custom request handlers (5 endpoints)
│   └── notifications.rs (348 lines)   didOpen/didChange/didClose/didSave
├── analysis.rs          (2227 lines)  Core analysis engine
│   ├── extract_full_type_info()       Variable ownership extraction
│   ├── compute_borrow_scopes()        NLL borrow scope computation
│   ├── detect_moves()                 Semantic move detection
│   ├── detect_conflicts()             Borrow conflict detection
│   ├── analyze_closures()             Closure capture analysis
│   ├── track_rc_clones()              Rc/Arc clone chain tracking
│   ├── analyze_cross_function_borrows() Inter-procedural analysis
│   ├── analyze_memory_layout()        Field-level memory layout
│   └── extract_type_fields()          Type field decomposition
└── lib.rs               (3 lines)     Library re-exports

tests/
├── lsp_protocol.rs      (1850 lines)  107 protocol-level tests
└── analysis.rs          (1802 lines)  Integration tests
```

**Total: ~3,800 lines of server code + 3,650 lines of tests**

## Dependencies

| Crate | Purpose |
|-------|---------|
| `lsp-server` | LSP protocol implementation (stdio JSON-RPC) |
| `lsp-types` | LSP type definitions |
| `ra_ap_hir` | Semantic analysis (types, traits, methods) |
| `ra_ap_ide_db` | Definitions, usages, references |
| `ra_ap_load_cargo` | Workspace loading |
| `ra_ap_project_model` | Cargo project model |
| `ra_ap_vfs` / `ra_ap_vfs-notify` | Virtual file system with file watching |
| `ra_ap_syntax` | AST parsing and traversal |
| `ra_ap_hir_ty` | Type layout computation |
| `serde` / `serde_json` | JSON serialization |
| `crossbeam-channel` | Message passing (from lsp-server) |
| `tracing` | Structured logging |

## Building & Running

```bash
# Debug build
cargo build -p borrowscope-lsp

# Release build (recommended for VS Code)
cargo build -p borrowscope-lsp --release

# Run directly
./target/release/borrowscope-lsp

# Version check
./target/release/borrowscope-lsp --version
```

The server communicates via stdio (stdin/stdout) using JSON-RPC. It's launched by the VS Code extension automatically.

## Testing

```bash
# Run all 107 protocol tests
cargo test -p borrowscope-lsp --test lsp_protocol -- --test-threads=1

# Run integration tests
cargo test -p borrowscope-lsp --test analysis

# Run all
cargo test -p borrowscope-lsp -- --test-threads=1
```

Tests use a real `RootDatabase` with synthetic Rust code to verify:
- All 5 custom requests return correct data
- CodeLens generation (ownership stats + memory)
- Inlay hints (ownership categories)
- Hover information
- Cross-function borrow resolution
- Memory layout field extraction
- Cache behavior (fresh/stale/eviction)
- Debounce timing

## Performance

| Operation | Typical Time |
|-----------|-------------|
| Workspace loading | 3-10s (first time, cached after) |
| Ownership graph (single function) | 2-5ms |
| Cross-function borrows | 20-100ms |
| Memory layout | 5-15ms |
| CodeLens (all functions in file) | 10-50ms |
| Debounce (configurable) | 0-2000ms |

## License

Apache-2.0
