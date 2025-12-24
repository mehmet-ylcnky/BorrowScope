# borrowscope-lsp

> Real-time ownership and borrowing visualization in your IDE

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../LICENSE)

## Vision

**borrowscope-lsp** brings Rust's ownership model to life directly in your editor. Instead of mentally tracing ownership flows or deciphering borrow checker errors, developers see ownership transfers, borrow lifetimes, and potential conflicts visualized in real-time as they type.

This is the "killer feature" of the BorrowScope ecosystem—making the invisible mechanics of Rust's memory model visible where developers need it most: in their IDE.

---

## Table of Contents

- [Core Features](#core-features)
- [End-User Scenarios](#end-user-scenarios)
- [Visualization Features](#visualization-features)
- [UX & Interactivity](#ux--interactivity)
- [IDE Integration](#ide-integration)
- [Technical Architecture](#technical-architecture)
- [Protocol Design](#protocol-design)
- [Performance Requirements](#performance-requirements)
- [Implementation Phases](#implementation-phases)
- [Dependencies](#dependencies)

---

## Core Features

### 1. Ownership Flow Visualization
- **Inline annotations** showing where ownership transfers occur
- **Color-coded lifetimes** for variables and references
- **Move semantics highlighting** when values are moved
- **Drop point indicators** showing where destructors run

### 2. Borrow Lifetime Visualization
- **Lifetime spans** rendered as colored underlines or sidebars
- **Overlapping borrow detection** with conflict highlighting
- **Mutable vs immutable** borrow distinction (different colors/styles)
- **Reborrow chains** showing nested borrow relationships

### 3. Smart Pointer Tracking
- **Rc/Arc reference counts** displayed inline
- **RefCell borrow state** (borrowed/borrowed_mut/available)
- **Weak reference validity** indicators
- **Clone propagation** visualization

### 4. Real-time Diagnostics
- **Pre-emptive conflict warnings** before compilation
- **Suggested fixes** with quick-action code modifications
- **"Why does this fail?"** explanations for borrow errors
- **Alternative patterns** suggestions (e.g., "Consider using Rc<RefCell<T>>")

---

## End-User Scenarios

### Scenario 1: Learning Rust
**User:** A developer new to Rust, struggling with ownership concepts.

**Experience:**
```rust
fn main() {
    let s = String::from("hello");  // [OWNER: s] created
    let r1 = &s;                     // [BORROW: r1 ← s] immutable
    let r2 = &s;                     // [BORROW: r2 ← s] immutable  
    println!("{} {}", r1, r2);       // [USE: r1, r2] last use
    let r3 = &mut s;                 // ⚠️ Error: s still borrowed
}
```

The IDE shows:
- Green underline under `s` spanning its lifetime
- Blue underlines under `r1` and `r2` showing borrow spans
- Red squiggle on `&mut s` with hover explanation
- Sidebar timeline showing overlapping lifetimes

### Scenario 2: Debugging Complex Ownership
**User:** Experienced developer debugging a borrow checker error in async code.

**Experience:**
- Click on error → IDE highlights the conflicting borrow
- "Show ownership flow" command traces the value back to creation
- Timeline view shows when each borrow starts/ends
- Async boundary crossings are marked with special indicators

### Scenario 3: Code Review
**User:** Reviewing a PR with complex lifetime annotations.

**Experience:**
- Hover over `'a` → see all variables bound to this lifetime
- Lifetime relationships visualized as a graph in sidebar
- "Simplify lifetimes" suggestion when annotations are redundant

### Scenario 4: Refactoring
**User:** Extracting a function and needs to understand what to borrow vs move.

**Experience:**
- Select code block → "Analyze ownership" command
- IDE shows which variables are:
  - Only read (can borrow immutably)
  - Mutated (need mutable borrow)
  - Consumed (must move or clone)
- Auto-generates function signature with correct borrowing

---

## Visualization Features

### Inline Decorations

| Decoration | Meaning | Style |
|------------|---------|-------|
| `→` | Ownership transfer (move) | Orange arrow |
| `&` | Immutable borrow active | Blue underline |
| `&mut` | Mutable borrow active | Red underline |
| `†` | Drop point | Gray marker |
| `⚡` | Async boundary | Yellow lightning |
| `🔒` | Lock guard active | Lock icon |
| `#n` | Reference count (Rc/Arc) | Superscript number |

### Gutter Icons

```
│ 1 │ ○  let data = vec![1, 2, 3];     // ○ = owner created
│ 2 │ ├─ let r1 = &data;               // ├─ = borrow starts
│ 3 │ ├─ let r2 = &data;               // ├─ = another borrow
│ 4 │ │  process(r1, r2);              // │ = borrows active
│ 5 │ └─                               // └─ = borrows end
│ 6 │ ●  drop(data);                   // ● = owner dropped
```

### Sidebar Timeline

```
Timeline: data                    Line
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
█████████████████████████████████  1-6  owner: data
    ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓        2-5  borrow: r1
    ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓        3-5  borrow: r2
                              ×   6    dropped
```

### Hover Information

Hovering over a variable shows:
```
┌─────────────────────────────────────────┐
│ data: Vec<i32>                          │
│ ─────────────────────────────────────── │
│ Owner: data (line 1)                    │
│ Status: Borrowed immutably (2 active)   │
│ Lifetime: lines 1-6                     │
│ Drop: line 6 (explicit)                 │
│                                         │
│ Active borrows:                         │
│   • r1: &Vec<i32> (line 2-5)           │
│   • r2: &Vec<i32> (line 3-5)           │
│                                         │
│ [Show Flow] [Show Timeline] [Explain]   │
└─────────────────────────────────────────┘
```

---

## UX & Interactivity

### Commands (Command Palette)

| Command | Description |
|---------|-------------|
| `BorrowScope: Toggle Visualization` | Enable/disable all visualizations |
| `BorrowScope: Show Ownership Flow` | Trace ownership from cursor position |
| `BorrowScope: Show Lifetime Graph` | Open lifetime relationship diagram |
| `BorrowScope: Explain Error` | Detailed explanation of borrow error |
| `BorrowScope: Suggest Fix` | AI-powered fix suggestions |
| `BorrowScope: Analyze Selection` | Ownership analysis of selected code |
| `BorrowScope: Show Timeline` | Open sidebar timeline view |
| `BorrowScope: Find Conflicts` | Highlight all potential borrow conflicts |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+O` | Toggle ownership visualization |
| `Ctrl+Shift+B` | Show borrows for symbol under cursor |
| `Ctrl+Shift+L` | Show lifetime for symbol under cursor |
| `F8` | Next borrow conflict |
| `Shift+F8` | Previous borrow conflict |

### Interactive Elements

1. **Clickable Annotations**
   - Click on borrow marker → jump to borrowed-from location
   - Click on move marker → jump to new owner
   - Click on drop marker → show what triggered drop

2. **Drag-to-Trace**
   - Drag from variable → highlight all uses
   - Drag between variables → show relationship

3. **Context Menu**
   - Right-click variable → "Track ownership"
   - Right-click error → "Explain" / "Fix" / "Show alternatives"

4. **Minimap Integration**
   - Lifetime spans shown as colored regions in minimap
   - Conflicts highlighted in red

### Configuration Options

```json
{
  "borrowscope.visualization.enabled": true,
  "borrowscope.visualization.style": "underline", // "underline" | "background" | "gutter"
  "borrowscope.visualization.opacity": 0.7,
  "borrowscope.colors.owner": "#4CAF50",
  "borrowscope.colors.immutableBorrow": "#2196F3",
  "borrowscope.colors.mutableBorrow": "#F44336",
  "borrowscope.colors.move": "#FF9800",
  "borrowscope.colors.drop": "#9E9E9E",
  "borrowscope.timeline.enabled": true,
  "borrowscope.timeline.position": "right", // "right" | "bottom" | "floating"
  "borrowscope.hover.delay": 300,
  "borrowscope.hover.showLifetime": true,
  "borrowscope.hover.showBorrows": true,
  "borrowscope.diagnostics.preemptive": true,
  "borrowscope.performance.debounceMs": 150,
  "borrowscope.performance.maxFileSize": 100000
}
```

---

## IDE Integration

### VS Code Extension

Primary target. Full feature support including:
- Custom decorations API for inline visualization
- Webview panels for timeline/graph views
- CodeLens for ownership summaries
- Tree view for ownership hierarchy
- Debug adapter integration for runtime tracking

**Extension Structure:**
```
borrowscope-vscode/
├── src/
│   ├── extension.ts          # Entry point
│   ├── client.ts             # LSP client
│   ├── decorations.ts        # Visual decorations
│   ├── timeline.ts           # Timeline webview
│   ├── graph.ts              # Ownership graph webview
│   ├── commands.ts           # Command handlers
│   └── config.ts             # Configuration
├── media/
│   ├── timeline.js           # Timeline visualization
│   └── graph.js              # D3.js graph rendering
└── package.json
```

### JetBrains (IntelliJ/CLion/RustRover)

Plugin using IntelliJ Platform SDK:
- External annotator for inline decorations
- Tool window for timeline view
- Gutter icons for ownership markers
- Intention actions for fixes

### Neovim/Vim

Via nvim-lspconfig + custom UI:
- Virtual text for inline annotations
- Floating windows for hover info
- Telescope integration for navigation
- Custom highlights for lifetime spans

### Emacs

Via lsp-mode + custom rendering:
- Overlays for inline visualization
- Dedicated buffer for timeline
- Integration with rust-analyzer

### Helix

Native LSP support with:
- Inline hints
- Diagnostic integration
- Custom picker for ownership navigation

---

## Technical Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         IDE / Editor                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Decorations │  │  Timeline   │  │     Graph View          │  │
│  │   Engine    │  │   Panel     │  │   (D3.js/Canvas)        │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                      │                │
│         └────────────────┼──────────────────────┘                │
│                          │                                       │
│                    ┌─────┴─────┐                                 │
│                    │ LSP Client│                                 │
│                    └─────┬─────┘                                 │
└──────────────────────────┼───────────────────────────────────────┘
                           │ JSON-RPC (stdio/tcp)
┌──────────────────────────┼───────────────────────────────────────┐
│                    ┌─────┴─────┐                                 │
│                    │LSP Server │  borrowscope-lsp                │
│                    └─────┬─────┘                                 │
│         ┌────────────────┼────────────────┐                      │
│         │                │                │                      │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐              │
│  │  Analyzer   │  │  Visualizer │  │  Diagnostics│              │
│  │   Engine    │  │   Engine    │  │   Engine    │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                    ┌─────┴─────┐                                 │
│                    │ borrowscope│                                │
│                    │   -graph   │                                │
│                    └─────┬─────┘                                 │
│                          │                                       │
│         ┌────────────────┼────────────────┐                      │
│         │                │                │                      │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐              │
│  │rust-analyzer│  │    HIR      │  │    MIR      │              │
│  │ integration │  │  Analysis   │  │  Analysis   │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└──────────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. LSP Server (`borrowscope-lsp`)

The main server implementing Language Server Protocol:

```rust
pub struct BorrowScopeLspServer {
    /// Connection to the client
    connection: Connection,
    /// Document manager
    documents: DocumentStore,
    /// Ownership analyzer
    analyzer: OwnershipAnalyzer,
    /// Visualization generator
    visualizer: Visualizer,
    /// Configuration
    config: ServerConfig,
}
```

#### 2. Ownership Analyzer

Analyzes Rust source code to extract ownership information:

```rust
pub struct OwnershipAnalyzer {
    /// Integration with rust-analyzer
    ra_client: RustAnalyzerClient,
    /// Cached analysis results
    cache: AnalysisCache,
}

pub struct OwnershipInfo {
    /// All variables and their ownership spans
    pub variables: Vec<VariableInfo>,
    /// All borrows and their lifetimes
    pub borrows: Vec<BorrowInfo>,
    /// Ownership transfers (moves)
    pub moves: Vec<MoveInfo>,
    /// Drop points
    pub drops: Vec<DropInfo>,
    /// Detected conflicts
    pub conflicts: Vec<ConflictInfo>,
}
```

#### 3. Visualizer

Converts analysis results to visual representations:

```rust
pub struct Visualizer {
    config: VisualizationConfig,
}

impl Visualizer {
    /// Generate inline decorations
    pub fn generate_decorations(&self, info: &OwnershipInfo) -> Vec<Decoration>;
    
    /// Generate timeline data
    pub fn generate_timeline(&self, info: &OwnershipInfo) -> TimelineData;
    
    /// Generate ownership graph
    pub fn generate_graph(&self, info: &OwnershipInfo) -> GraphData;
}
```

#### 4. rust-analyzer Integration

Two integration strategies:

**Option A: Companion Server**
- Run alongside rust-analyzer
- Query rust-analyzer for semantic information via LSP
- Augment with BorrowScope-specific analysis

**Option B: rust-analyzer Plugin** (future)
- Implement as rust-analyzer extension
- Direct access to HIR/MIR
- Better performance, tighter integration

---

## Protocol Design

### Custom LSP Extensions

Beyond standard LSP, we define custom methods:

#### `borrowscope/ownershipInfo`

Request ownership information for a document:

```typescript
interface OwnershipInfoParams {
  textDocument: TextDocumentIdentifier;
  range?: Range;  // Optional: specific range
}

interface OwnershipInfoResponse {
  variables: VariableInfo[];
  borrows: BorrowInfo[];
  moves: MoveInfo[];
  drops: DropInfo[];
  conflicts: ConflictInfo[];
}

interface VariableInfo {
  name: string;
  type: string;
  range: Range;
  lifetime: Range;
  owner: boolean;
}

interface BorrowInfo {
  name: string;
  borrowedFrom: string;
  mutable: boolean;
  range: Range;
  lifetime: Range;
}
```

#### `borrowscope/timeline`

Request timeline visualization data:

```typescript
interface TimelineParams {
  textDocument: TextDocumentIdentifier;
  variables?: string[];  // Filter to specific variables
}

interface TimelineResponse {
  spans: TimelineSpan[];
  conflicts: TimelineConflict[];
}

interface TimelineSpan {
  variable: string;
  kind: "owner" | "borrow" | "borrow_mut";
  startLine: number;
  endLine: number;
  color: string;
}
```

#### `borrowscope/explainError`

Get detailed explanation for a borrow error:

```typescript
interface ExplainErrorParams {
  textDocument: TextDocumentIdentifier;
  position: Position;
  errorCode?: string;
}

interface ExplainErrorResponse {
  explanation: string;
  involvedVariables: VariableInfo[];
  conflictingBorrows: BorrowInfo[];
  suggestions: Suggestion[];
  visualizations: {
    timeline?: TimelineResponse;
    graph?: GraphResponse;
  };
}
```

#### `borrowscope/decorations`

Push decorations to client (server → client notification):

```typescript
interface DecorationsNotification {
  textDocument: TextDocumentIdentifier;
  decorations: Decoration[];
}

interface Decoration {
  range: Range;
  kind: "owner" | "borrow" | "borrow_mut" | "move" | "drop" | "conflict";
  text?: string;  // Inline text
  hoverMessage?: string;
  color?: string;
}
```

---

## Performance Requirements

### Latency Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Keystroke to decoration update | <100ms | 200ms |
| Hover information | <50ms | 150ms |
| Full file analysis | <500ms | 2s |
| Timeline generation | <100ms | 300ms |
| Graph generation | <200ms | 500ms |

### Optimization Strategies

1. **Incremental Analysis**
   - Only re-analyze changed regions
   - Cache unchanged results
   - Use tree-diffing for minimal updates

2. **Debouncing**
   - Debounce rapid keystrokes (150ms default)
   - Cancel in-flight requests on new input

3. **Lazy Loading**
   - Load visible range first
   - Background-load rest of file
   - Prioritize cursor vicinity

4. **Caching**
   - Cache analysis per-function
   - Invalidate only affected functions on edit
   - Persist cache across sessions

5. **Streaming**
   - Stream decorations as computed
   - Progressive timeline rendering

### Resource Limits

```rust
pub struct ResourceLimits {
    /// Maximum file size to analyze (bytes)
    pub max_file_size: usize,  // default: 100KB
    /// Maximum functions per file
    pub max_functions: usize,  // default: 500
    /// Maximum variables to track
    pub max_variables: usize,  // default: 1000
    /// Analysis timeout (ms)
    pub timeout_ms: u64,       // default: 5000
    /// Cache size (entries)
    pub cache_size: usize,     // default: 100
}
```

---

## Implementation Phases

### Phase 1: Foundation (MVP)
**Goal:** Basic ownership visualization in VS Code

- [ ] LSP server skeleton with tower-lsp
- [ ] Basic document synchronization
- [ ] Simple ownership analysis (local variables only)
- [ ] Inline decorations for owner/borrow/drop
- [ ] VS Code extension with decoration rendering
- [ ] Basic hover information

**Deliverable:** Can visualize ownership in simple functions

### Phase 2: Enhanced Analysis
**Goal:** Comprehensive ownership tracking

- [ ] rust-analyzer integration for semantic info
- [ ] Cross-function ownership tracking
- [ ] Smart pointer support (Rc, Arc, RefCell)
- [ ] Lifetime parameter visualization
- [ ] Move semantics highlighting
- [ ] Conflict detection and highlighting

**Deliverable:** Full ownership visualization for real-world code

### Phase 3: Interactive Features
**Goal:** Rich interactivity and diagnostics

- [ ] Timeline sidebar view
- [ ] Ownership graph visualization
- [ ] "Explain error" feature
- [ ] Quick-fix suggestions
- [ ] Click-to-navigate between related items
- [ ] Command palette integration

**Deliverable:** Interactive exploration of ownership

### Phase 4: Multi-IDE Support
**Goal:** Broad editor support

- [ ] JetBrains plugin
- [ ] Neovim integration
- [ ] Emacs integration
- [ ] Helix support
- [ ] Standardized configuration

**Deliverable:** Works in all major Rust IDEs

### Phase 5: Advanced Features
**Goal:** Power-user features

- [ ] Async/await ownership tracking
- [ ] Unsafe code visualization
- [ ] Runtime integration (connect to borrowscope-runtime)
- [ ] AI-powered suggestions
- [ ] Custom visualization themes
- [ ] Performance profiling integration

**Deliverable:** Complete ownership visualization platform

---

## Dependencies

### Rust Crates

```toml
[dependencies]
# LSP implementation
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }

# Rust analysis
ra_ap_syntax = "0.0.x"        # rust-analyzer syntax
ra_ap_hir = "0.0.x"           # rust-analyzer HIR (optional)
syn = { version = "2", features = ["full", "visit"] }

# BorrowScope ecosystem
borrowscope-graph = { path = "../borrowscope-graph" }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utilities
dashmap = "5"                  # Concurrent cache
tracing = "0.1"               # Logging
anyhow = "1"                  # Error handling
```

### VS Code Extension

```json
{
  "dependencies": {
    "vscode-languageclient": "^9.0.0",
    "d3": "^7.0.0"
  }
}
```

---

## Related Projects

- [rust-analyzer](https://rust-analyzer.github.io/) - Rust LSP server we integrate with
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - LSP server framework
- [borrowscope-runtime](../borrowscope-runtime/) - Runtime tracking library
- [borrowscope-graph](../borrowscope-graph/) - Ownership graph algorithms

---

## Contributing

This project is in early design phase. Contributions welcome for:
- Architecture feedback
- IDE-specific expertise
- rust-analyzer integration knowledge
- UX/visualization design

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## License

Apache 2.0 - See [LICENSE](../LICENSE)
