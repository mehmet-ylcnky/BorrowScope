# 4. Component Specifications

## 4.1 borrowscope-macro

### Overview
Procedural macro crate that instruments Rust code to inject ownership tracking calls.

### Crate Type
```toml
[lib]
proc-macro = true
```

### Dependencies
```toml
syn = { version = "2.0", features = ["full", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### Public API

#### Attribute Macro
```rust
#[trace_borrow]
fn my_function() { ... }
```

### Implementation Details

#### AST Transformation Strategy
1. Parse function body into AST using `syn::parse_macro_input!`
2. Visit each statement and expression
3. Identify patterns:
   - `let` bindings → inject `track_new()`
   - `&expr` → inject `track_borrow()`
   - `&mut expr` → inject `track_borrow_mut()`
   - Function calls that consume values → inject `track_move()`
   - Scope exits → inject `track_drop()`
4. Generate modified AST with tracking calls
5. Return expanded code via `quote!`

#### Example Transformation
**Input:**
```rust
#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r = &s;
    println!("{}", r);
}
```

**Output (conceptual):**
```rust
fn example() {
    let s = borrowscope_runtime::track_new("s", String::from("hello"));
    let r = borrowscope_runtime::track_borrow("r", &s);
    println!("{}", r);
    borrowscope_runtime::track_drop("r");
    borrowscope_runtime::track_drop("s");
}
```

### Key Challenges
- Preserving original semantics (no behavior changes)
- Handling complex expressions (nested borrows, method chains)
- Determining when moves occur (requires type information)
- Inserting drop calls at correct scope boundaries

### Testing Strategy
- Unit tests for each AST pattern
- Integration tests with `trybuild` for compile-time validation
- Snapshot tests comparing expanded output

---

## 4.2 borrowscope-runtime

### Overview
Core library that collects ownership events and builds the ownership graph.

### Crate Type
```toml
[lib]
```

### Dependencies
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
petgraph = "0.6"
lazy_static = "1.4"
parking_lot = "0.12"  # For thread-safe global tracker
```

### Public API

#### Tracking Functions
```rust
pub fn track_new<T>(name: &str, value: T) -> T;
pub fn track_borrow<T>(name: &str, value: &T) -> &T;
pub fn track_borrow_mut<T>(name: &str, value: &mut T) -> &mut T;
pub fn track_move<T>(from: &str, to: &str, value: T) -> T;
pub fn track_drop(name: &str);
```

#### Export Functions
```rust
pub fn export_json(path: &str) -> Result<(), Error>;
pub fn get_events() -> Vec<Event>;
pub fn get_graph() -> OwnershipGraph;
pub fn reset();
```

### Data Structures

#### Event
```rust
#[derive(Debug, Clone, Serialize)]
pub enum Event {
    New { 
        timestamp: u64,
        var_name: String,
        type_name: String,
    },
    Borrow { 
        timestamp: u64,
        borrower: String,
        owner: String,
        mutable: bool,
    },
    Move { 
        timestamp: u64,
        from: String,
        to: String,
    },
    Drop { 
        timestamp: u64,
        var_name: String,
    },
}
```

#### OwnershipGraph
```rust
#[derive(Debug, Serialize)]
pub struct OwnershipGraph {
    nodes: Vec<Variable>,
    edges: Vec<Relationship>,
}

#[derive(Debug, Serialize)]
pub struct Variable {
    id: String,
    name: String,
    type_name: String,
    created_at: u64,
    dropped_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub enum Relationship {
    Owns { from: String, to: String },
    BorrowsImmut { from: String, to: String, start: u64, end: u64 },
    BorrowsMut { from: String, to: String, start: u64, end: u64 },
}
```

#### Global Tracker
```rust
struct Tracker {
    events: Vec<Event>,
    graph: Graph<Variable, Relationship>,
    timestamp: AtomicU64,
}

lazy_static! {
    static ref GLOBAL_TRACKER: Mutex<Tracker> = Mutex::new(Tracker::new());
}
```

### Implementation Details

#### Event Collection
- Each tracking function acquires lock on `GLOBAL_TRACKER`
- Increments timestamp
- Records event
- Updates graph structure
- Returns value unchanged (zero-cost abstraction goal)

#### Graph Building
- Use `petgraph::Graph` for efficient graph operations
- Nodes added on `track_new()`
- Edges added on `track_borrow()` and `track_move()`
- Edges removed on `track_drop()`

#### JSON Export Format
```json
{
  "events": [
    { "type": "New", "timestamp": 1, "var_name": "s", "type_name": "String" },
    { "type": "Borrow", "timestamp": 2, "borrower": "r", "owner": "s", "mutable": false },
    { "type": "Drop", "timestamp": 3, "var_name": "r" },
    { "type": "Drop", "timestamp": 4, "var_name": "s" }
  ],
  "graph": {
    "nodes": [...],
    "edges": [...]
  }
}
```

### Performance Considerations
- Minimize lock contention (use `parking_lot::Mutex`)
- Lazy graph construction (build on export, not per-event)
- Optional: compile-time feature flag to disable tracking in release builds

---

## 4.3 borrowscope-cli

### Overview
Cargo subcommand for seamless integration with Rust projects.

### Crate Type
```toml
[[bin]]
name = "cargo-borrowscope"
```

### Dependencies
```toml
clap = { version = "4.0", features = ["derive"] }
syn = "2.0"
quote = "1.0"
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

### Command Structure

```bash
cargo borrowscope <SUBCOMMAND> [OPTIONS]

Subcommands:
  visualize <file>    Instrument, run, and visualize a single file
  run                 Instrument and run entire project
  export              Generate JSON without opening UI
  init                Create .borrowscope.toml config file
```

### Command Specifications

#### `visualize <file>`
```bash
cargo borrowscope visualize src/main.rs
```
**Steps:**
1. Parse target file
2. Inject `#[trace_borrow]` on all functions (or specified functions)
3. Create temporary instrumented copy
4. Compile with `borrowscope-runtime` dependency
5. Execute binary
6. Collect JSON output
7. Launch UI with data
8. Clean up temporary files

**Options:**
- `--function <name>` - Only instrument specific function
- `--output <path>` - Save JSON to custom location
- `--no-ui` - Skip launching visualization

#### `run`
```bash
cargo borrowscope run
```
**Steps:**
1. Read `Cargo.toml` to find all source files
2. Instrument all files in `src/`
3. Add `borrowscope-runtime` to dependencies temporarily
4. Run `cargo build && cargo run`
5. Collect output
6. Launch UI
7. Restore original files

**Options:**
- `--release` - Build in release mode
- `--example <name>` - Run specific example

#### `export`
```bash
cargo borrowscope export --output graph.json
```
**Steps:**
1. Instrument code
2. Run and collect data
3. Export JSON
4. Exit (no UI)

#### `init`
```bash
cargo borrowscope init
```
Creates `.borrowscope.toml`:
```toml
[instrumentation]
auto_instrument = true
exclude_functions = ["main", "test_*"]

[visualization]
auto_open = true
port = 8080

[export]
format = "json"
output_dir = "./borrowscope-output"
```

### Implementation Details

#### File Instrumentation
```rust
fn instrument_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut ast = syn::parse_file(&content)?;
    
    // Visit all functions and add #[trace_borrow]
    for item in &mut ast.items {
        if let syn::Item::Fn(func) = item {
            func.attrs.push(parse_quote!(#[trace_borrow]));
        }
    }
    
    Ok(quote!(#ast).to_string())
}
```

#### Temporary Project Management
- Create `.borrowscope-temp/` directory
- Copy instrumented files
- Modify `Cargo.toml` to add runtime dependency
- Build and run
- Clean up on exit or error

### Error Handling
- Graceful failures with helpful messages
- Automatic rollback on errors
- Preserve original files (never modify in-place)

---

## 4.4 borrowscope-ui

### Overview
Interactive visualization frontend for ownership graphs and timelines.

### Technology Choice
**Recommended:** Tauri (Rust backend + web frontend)
- Native performance
- Small binary size
- Rust-first development
- Cross-platform (Linux, macOS, Windows)

**Alternative:** Dioxus (pure Rust WASM)
- Fully Rust codebase
- Web-based, no installation
- Easier deployment

### Project Structure
```
borrowscope-ui/
├── src-tauri/          # Rust backend
│   ├── main.rs
│   └── Cargo.toml
├── src/                # Frontend (HTML/JS/CSS)
│   ├── index.html
│   ├── main.js
│   ├── graph.js
│   ├── timeline.js
│   └── styles.css
└── tauri.conf.json
```

### Dependencies

#### Rust (Tauri backend)
```toml
tauri = "1.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### JavaScript (Frontend)
```json
{
  "dependencies": {
    "d3": "^7.8.5",
    "cytoscape": "^3.26.0"
  }
}
```

### UI Components

#### 1. Graph View
**Library:** Cytoscape.js or D3.js force-directed graph

**Features:**
- Nodes: Variables (circles)
- Edges: Relationships (arrows)
- Colors:
  - Blue: Owner
  - Green: Immutable borrow
  - Red: Mutable borrow
  - Gray: Dropped
- Interactions:
  - Hover: Show variable details
  - Click: Highlight related nodes
  - Drag: Reposition nodes
  - Zoom/Pan: Navigate large graphs

**Implementation:**
```javascript
const cy = cytoscape({
  container: document.getElementById('graph'),
  elements: loadGraphData(),
  style: [
    {
      selector: 'node',
      style: {
        'background-color': 'data(color)',
        'label': 'data(name)'
      }
    },
    {
      selector: 'edge',
      style: {
        'line-color': 'data(color)',
        'target-arrow-color': 'data(color)',
        'target-arrow-shape': 'triangle'
      }
    }
  ]
});
```

#### 2. Timeline View
**Library:** Custom D3.js implementation

**Features:**
- Horizontal timeline (x-axis = time)
- Variable lifespans as horizontal bars
- Borrow periods as overlapping segments
- Scrubber to step through events
- Play/pause animation

**Layout:**
```
Time →
0────────────────────────────────────────→ 100

s  ████████████████████████████████████
r1      ████████████
r2           ████████████
```

#### 3. Code View
**Library:** Monaco Editor or CodeMirror

**Features:**
- Display original source code
- Highlight active variables at current timestamp
- Show borrow annotations inline
- Sync with graph and timeline

#### 4. Control Panel
**Elements:**
- Load JSON button
- Play/Pause button
- Step forward/backward buttons
- Speed slider
- Reset button
- Export image button

### Tauri Commands (Rust ↔ JS Bridge)

```rust
#[tauri::command]
fn load_graph_data(path: String) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_image(format: String) -> Result<(), String> {
    // Generate PNG/SVG of current graph
    Ok(())
}
```

```javascript
// Call from frontend
const data = await invoke('load_graph_data', { path: 'graph.json' });
```

### Responsive Design
- Minimum window size: 1024x768
- Resizable panels (graph | timeline | code)
- Dark/light theme toggle
- Keyboard shortcuts:
  - Space: Play/Pause
  - ←/→: Step backward/forward
  - R: Reset
  - E: Export

### Accessibility
- ARIA labels for all interactive elements
- Keyboard navigation support
- High contrast mode
- Screen reader compatible

---

## Integration Between Components

```
┌─────────────────┐
│  User's Code    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌──────────────────┐
│ borrowscope-cli │─────▶│ borrowscope-macro│
└────────┬────────┘      └──────────────────┘
         │                         │
         │                         ▼
         │               ┌──────────────────┐
         │               │borrowscope-runtime│
         │               └─────────┬─────────┘
         │                         │
         │                    (JSON file)
         │                         │
         ▼                         ▼
┌─────────────────────────────────────────┐
│          borrowscope-ui                 │
│  (loads JSON, renders visualization)    │
└─────────────────────────────────────────┘
```

### Workspace Configuration
```toml
[workspace]
members = [
    "borrowscope-macro",
    "borrowscope-runtime",
    "borrowscope-cli",
    "borrowscope-ui/src-tauri"
]
```
