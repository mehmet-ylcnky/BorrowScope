# 5. Technical Stack

## Core Dependencies

### borrowscope-macro

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `syn` | 2.0 | AST parsing | Industry standard, feature-rich, well-maintained |
| `quote` | 1.0 | Code generation | Seamless integration with syn, ergonomic macros |
| `proc-macro2` | 1.0 | Proc macro support | Required for procedural macros |

**Features needed:**
```toml
syn = { version = "2.0", features = ["full", "visit-mut", "extra-traits"] }
```
- `full`: Parse complete Rust syntax
- `visit-mut`: Modify AST in-place
- `extra-traits`: Debug implementations for testing

---

### borrowscope-runtime

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `serde` | 1.0 | Serialization | Universal standard for Rust serialization |
| `serde_json` | 1.0 | JSON export | Human-readable, debuggable output format |
| `petgraph` | 0.6 | Graph algorithms | Mature, efficient graph data structures |
| `parking_lot` | 0.12 | Thread-safe locks | Faster than std::sync::Mutex, better performance |
| `lazy_static` | 1.4 | Global state | Simple global tracker initialization |

**Optional dependencies:**
```toml
[dependencies]
# For WebSocket streaming (Phase 4)
tokio = { version = "1.35", features = ["rt", "net"], optional = true }
tokio-tungstenite = { version = "0.21", optional = true }

# For MessagePack (alternative to JSON)
rmp-serde = { version = "1.1", optional = true }

[features]
default = []
websocket = ["tokio", "tokio-tungstenite"]
msgpack = ["rmp-serde"]
```

---

### borrowscope-cli

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `clap` | 4.4 | CLI parsing | Modern, derive-based API, excellent UX |
| `syn` | 2.0 | Code parsing | Reuse for file instrumentation |
| `quote` | 1.0 | Code generation | Inject attributes programmatically |
| `toml` | 0.8 | Config files | Standard Rust config format |
| `serde` | 1.0 | Config deserialization | Parse .borrowscope.toml |
| `anyhow` | 1.0 | Error handling | Ergonomic error propagation |
| `colored` | 2.1 | Terminal colors | Better CLI output readability |

**Features:**
```toml
clap = { version = "4.4", features = ["derive", "cargo"] }
```

---

### borrowscope-ui (Tauri)

#### Rust Backend

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `tauri` | 1.5 | Desktop framework | Lightweight, secure, Rust-native |
| `serde` | 1.0 | Data serialization | Bridge Rust ↔ JavaScript |
| `serde_json` | 1.0 | JSON handling | Load graph data |

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### JavaScript Frontend

| Package | Version | Purpose | Justification |
|---------|---------|---------|---------------|
| `d3` | ^7.8.5 | Data visualization | Powerful, flexible, standard for graphs |
| `cytoscape` | ^3.26.0 | Graph rendering | Specialized for network graphs |
| `@tauri-apps/api` | ^1.5.0 | Tauri bindings | Call Rust from JavaScript |

**Alternative (if using Dioxus):**
```toml
dioxus = "0.4"
dioxus-web = "0.4"
```

---

## Development Tools

### Build Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Compiler (stable channel) |
| Cargo | 1.75+ | Build system |
| Node.js | 18+ | Frontend tooling (if using Tauri) |
| npm/pnpm | Latest | JavaScript package manager |

### Testing

| Crate | Version | Purpose |
|-------|---------|---------|
| `trybuild` | 1.0 | Macro compile tests |
| `insta` | 1.34 | Snapshot testing |
| `criterion` | 0.5 | Benchmarking |
| `proptest` | 1.4 | Property-based testing |

```toml
[dev-dependencies]
trybuild = "1.0"
insta = "1.34"
criterion = "0.5"
proptest = "1.4"
```

### Code Quality

| Tool | Purpose |
|------|---------|
| `rustfmt` | Code formatting |
| `clippy` | Linting |
| `cargo-audit` | Security audits |
| `cargo-deny` | Dependency validation |
| `cargo-outdated` | Dependency updates |

### Documentation

| Tool | Purpose |
|------|---------|
| `rustdoc` | API documentation |
| `mdbook` | User guide |
| `cargo-readme` | Generate README from docs |

---

## Version Requirements

### Minimum Supported Rust Version (MSRV)
**1.75.0** (released January 2024)

**Rationale:**
- Stable procedural macro features
- Modern async/await support
- Required by Tauri 1.5

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Full | Primary development platform |
| macOS | ✅ Full | Tauri fully supported |
| Windows | ✅ Full | Tauri fully supported |
| WASM | 🟡 Partial | UI only (if using Dioxus) |

### Browser Support (for Tauri WebView)

| Browser Engine | Platform | Version |
|----------------|----------|---------|
| WebKit | macOS | System default |
| WebView2 | Windows | Edge 90+ |
| WebKitGTK | Linux | 2.36+ |

---

## Dependency Justification

### Why `syn` + `quote`?
- **Standard:** Used by 99% of procedural macros in ecosystem
- **Mature:** Battle-tested, stable API
- **Feature-rich:** Handles all Rust syntax edge cases
- **Alternatives considered:** `proc-macro-hack` (deprecated), manual parsing (too complex)

### Why `petgraph`?
- **Performance:** Optimized graph algorithms
- **Features:** Supports directed graphs, traversal, serialization
- **Alternatives considered:** Custom graph (reinventing wheel), `graphlib` (less mature)

### Why Tauri over Electron?
- **Size:** ~3MB vs ~100MB binary
- **Performance:** Native Rust backend, no Node.js overhead
- **Security:** Rust memory safety, smaller attack surface
- **Rust-first:** Seamless integration with our codebase

### Why D3.js + Cytoscape.js?
- **D3.js:** Timeline and custom visualizations
- **Cytoscape.js:** Specialized for graph layouts and interactions
- **Alternatives considered:** 
  - Three.js (overkill for 2D graphs)
  - Vis.js (less flexible)
  - Pure Canvas (too much manual work)

### Why `clap` v4?
- **Modern API:** Derive-based, less boilerplate
- **Features:** Subcommands, validation, help generation
- **Alternatives considered:** `structopt` (merged into clap), `argh` (less features)

---

## Optional Features & Extensions

### Phase 4+ (Advanced Features)

| Feature | Dependencies | Purpose |
|---------|--------------|---------|
| MIR Analysis | `rustc_middle`, `rustc_driver` | Compiler integration |
| WebSocket Streaming | `tokio`, `tokio-tungstenite` | Real-time updates |
| MessagePack Export | `rmp-serde` | Compact binary format |
| VS Code Extension | `vscode-languageserver` | IDE integration |

### Future Considerations

| Technology | Use Case | Priority |
|------------|----------|----------|
| WASM | Browser-based UI | Medium |
| LSP | Editor integration | High |
| Tree-sitter | Better parsing | Low |
| LLVM IR | Deeper analysis | Low |

---

## Dependency Update Policy

### Stability Tiers

**Tier 1 (Critical):** Update cautiously, test thoroughly
- `syn`, `quote`, `serde`, `tauri`

**Tier 2 (Important):** Update regularly, follow semver
- `clap`, `petgraph`, `tokio`

**Tier 3 (Nice-to-have):** Update as needed
- `colored`, `anyhow`, dev dependencies

### Update Schedule
- **Security patches:** Immediate
- **Minor versions:** Monthly review
- **Major versions:** Quarterly evaluation

---

## License Compatibility

All dependencies use permissive licenses compatible with MIT/Apache-2.0:
- MIT: `syn`, `quote`, `serde`, `clap`
- Apache-2.0: `petgraph`, `tokio`
- Dual MIT/Apache-2.0: Most Rust crates

**BorrowScope License:** MIT OR Apache-2.0 (standard Rust dual-license)
