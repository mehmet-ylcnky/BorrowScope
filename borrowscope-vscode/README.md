# borrowscope-vscode

> VS Code extension for real-time Rust ownership visualization — 11 interactive views, inline annotations, and runtime overlay

## Overview

The BorrowScope VS Code extension provides a rich visual experience for understanding Rust's ownership system. It combines **static analysis** (from `borrowscope-lsp`) with optional **runtime data** (from `borrowscope-runtime`) to show ownership flow directly in your editor and in an interactive visualization panel.

**No configuration needed** — install, point to the LSP binary, and open a Rust file.

## Features at a Glance

### Editor Decorations

```rust
fn example() {
    let owned = String::from("hello");     // [Owned]  ← colored inline hint
    let reference = &owned;                 // [&]      ← blue
    let mut_ref = &mut owned;              // [&mut]   ← red
    let rc = Rc::new(42);                  // [Rc]     ← purple
    //  ├─ 👁 shared borrow ──────────┐   ← lifeline flow
    //  │                              │
    //  ╰─ 🔒 mutable borrow ─────────┘   ← lifeline end
}
// ▸ 4 vars, 2 borrows, 0 moves          ← CodeLens stats
// 🧠 Stack: 56B | Heap: ~5B | 2 ptrs    ← Memory CodeLens
```

### Visualization Panel (11 Views)

The panel opens when you click a CodeLens or press `Ctrl+Shift+O`:

| View | Icon | What It Shows |
|------|------|---------------|
| **Graph** | 🕸️ | Force-directed ownership graph (D3.js) — nodes are variables, edges are borrows/moves |
| **Table** | ▦ | All variables with types, categories, and ownership info |
| **Timeline** | ⏱️ | Gantt-chart of variable lifetimes with borrow regions |
| **Scopes** | 🔍 | Nested borrow scopes showing active borrows at each line |
| **RefCount** | 🔗 | Rc/Arc reference count changes over time |
| **Moves** | ↦ | Ownership transfer chains (source → destination) |
| **Conflicts** | ⚠️ | Borrow conflicts with overlapping regions highlighted |
| **Compare** | 🪞 | Side-by-side comparison of two functions |
| **CrossRefs** | 🔀 | Cross-function borrow tracking with file navigation |
| **Memory** | 🧠 | Stack/heap layout with field-level detail (ptr/len/cap) |
| **Runtime** | 🔬 | Runtime events: timeline, drop order, ref count chart, event stream |

### Landing Page

On first open, the panel shows a landing page with:
- Project logo
- Icon grid (click to open any view)
- Source Code + Research links
- Function name and stats

### Runtime Overlay (Optional)

When `borrowscope-runtime` events are available:
- **Green annotations**: `⏱ 1.2ms (3×&)` — actual variable lifetime
- **Red divergences**: `⚡ Rc never dropped` — static vs runtime disagreement
- **Purple ref counts**: `🔗 peak:4 (3 clones)` — Rc/Arc tracking
- **Gray drop order**: `💀 #3` — actual LIFO sequence
- **Status bar**: `BorrowScope: Static ✓ | Runtime ✓ (103 events, just now)`

## Installation

1. Build the language server:
   ```bash
   cd borrowscope-lsp && cargo build --release
   ```

2. Open the extension in VS Code:
   ```bash
   cd borrowscope-vscode && npm install && npm run build
   ```

3. Press F5 to launch the Extension Development Host

4. Set the server path in settings:
   ```json
   { "borrowscope.server.path": "/path/to/target/release/borrowscope-lsp" }
   ```

## Configuration (33 settings)

| Group | Settings |
|-------|----------|
| **Server** | `path`, `extraArgs` |
| **Analysis** | `debounceMs` (0-2000) |
| **Decorations** | `enabled`, `borrowScopes`, `gutterIcons`, `inlayHints`, `codeLens`, `lifelines` |
| **Graph** | `layout` (force/hierarchical/radial), `showTypes`, `animateUpdates` |
| **Colors** | `sharedBorrow`, `mutableBorrow`, `move`, `rcArc`, `owned`, `drop` |
| **Cross-Function** | `enabled`, `maxDepth`, `showInline` |
| **Memory** | `enabled`, `showAlignment`, `animationSpeed` |
| **Runtime** | `enabled`, `source` (file/websocket), `filePath`, `websocketPort`, `showTimings`, `showDropOrder`, `showRefCounts`, `highlightDivergences` |
| **Diagnostics** | `enabled`, `severity` |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+O` | Show Ownership Graph |
| `Ctrl+Shift+I` | Inspect Variable at Cursor |
| `Ctrl+Shift+D` | Toggle Decorations |
| `Alt+Shift+N` | Next Borrow Conflict |
| `Alt+Shift+P` | Previous Borrow Conflict |
| `Ctrl+Shift+G` | Focus Graph Panel |

## Commands (27 total)

All accessible via `Ctrl+Shift+P`:

- Show Graph / Timeline / Scopes / RefCount / Moves
- Toggle Decorations / Borrow Scopes / Gutter Icons / Lifelines / CodeLens
- Next/Previous Conflict
- Restart Server / Show Server Output
- Export Graph as DOT / SVG
- Describe Graph (Accessibility)
- Show Performance Report
- Show Welcome
- Toggle Runtime / Set Source / Toggle Timings / Drop Order / Ref Counts / Divergences

## Architecture

```
src/
├── extension.ts              Entry point, activation, command registration
├── client.ts                 LSP client, decorations, lifelines application
├── commands.ts               CodeLens click handler, graph panel orchestration
├── config.ts                 Typed configuration accessor (BorrowScopeConfig)
├── decorations.ts            Inline ownership hints ([&], [&mut], [Rc])
├── lifelines.ts              Flow line decorations (├─ │ ╰─)
├── cross-lifelines.ts        Cross-function borrow annotations
├── highlights.ts             Background borrow region highlights
├── conflicts.ts              Conflict detection decorations
│
├── graph/
│   ├── panel.ts              WebView panel (11 views, D3.js, ~1500 lines)
│   ├── model.ts              Graph data model (nodes, edges)
│   ├── renderer.ts           D3.js force-directed graph
│   ├── timeline.ts           Timeline/Gantt view
│   ├── scopes.ts             Scope nesting view
│   ├── refcount.ts           Ref count chart
│   ├── movechain.ts          Move chain visualization
│   ├── comparison.ts         Side-by-side comparison
│   ├── cross-panel.ts        Cross-function panel
│   ├── diff.ts               Graph diff computation
│   └── messages.ts           WebView ↔ extension messaging
│
├── runtime-types.ts          88 event type definitions (from borrowscope-runtime)
├── runtime-watcher.ts        File watcher for .borrowscope/events.json
├── runtime-parser.ts         Event parsing, validation, filtering
├── runtime-mapper.ts         Map runtime events to static variables
├── runtime-socket.ts         WebSocket live connection
├── runtime-decorations.ts    Green timing / red divergence overlays
├── runtime-status.ts         Status bar indicator + toggle commands
├── merge-views.ts            Static + runtime merge with divergence detection
├── divergence-detector.ts    16 divergence kinds with suggestions
├── refcount-timeline.ts      Rc/Arc ref count timeline builder
├── drop-order.ts             Drop order analysis (LIFO detection)
├── async-borrow-tracker.ts   Borrow tracking across await points
│
├── performance.ts            PerformanceMonitor, Debouncer
├── welcome.ts                First-run onboarding panel
├── server-manager.ts         LSP server lifecycle
└── server-path.ts            Server binary resolution
```

**Total: ~6,900 lines of extension code**

## Testing

```bash
# Build
npm run build

# Compile TypeScript (for test output)
npx tsc --noEmit false

# Run all 689 tests
npx mocha --require ./test-setup.js

# Run specific test suite
npx mocha --require ./test-setup.js --grep "12.7"
```

### Test Suites (44 files, 689 tests)

| Suite | Tests | Covers |
|-------|-------|--------|
| `client.test.ts` | LSP client, server path, decorations |
| `commands.test.ts` | CodeLens click, graph panel creation |
| `decorations.test.ts` | Inline hints, colors |
| `lifelines.test.ts` | Flow lines, symbols |
| `cross-lifelines.test.ts` | Cross-function annotations |
| `panel.test.ts` | WebView panel lifecycle |
| `model.test.ts` | Graph data model |
| `live-update.test.ts` | Real-time updates |
| `memory.test.ts` | Memory layout visualization |
| `runtime-watcher.test.ts` | File watching |
| `runtime-parser.test.ts` | Event parsing (all 88 types) |
| `runtime-mapper.test.ts` | Variable mapping |
| `merge-views.test.ts` | Static + runtime merge |
| `runtime-decorations.test.ts` | Timing/divergence overlays |
| `divergence-detector.test.ts` | 16 divergence kinds |
| `refcount-timeline.test.ts` | Rc/Arc timeline |
| `drop-order.test.ts` | LIFO analysis |
| `async-borrow-tracker.test.ts` | Await crossing detection |
| `runtime-socket.test.ts` | WebSocket connection |
| `runtime-status.test.ts` | Status bar |
| `config.test.ts` | 33 settings |
| `keybindings.test.ts` | 6 shortcuts |
| `command-palette.test.ts` | 27 commands |
| `theme.test.ts` | 10 theme colors |
| `welcome.test.ts` | Onboarding |
| `performance.test.ts` | Monitor + debouncer |
| `accessibility.test.ts` | ARIA, keyboard nav |

## Theme Support

- **10 registered theme colors** with dark/light/high-contrast defaults
- Panel uses `--vscode-*` CSS variables throughout
- `.vscode-light` rules for light themes
- `.vscode-high-contrast` rules (thicker borders, higher opacity)
- Decoration colors configurable via settings

## Accessibility

- Graph nodes have `role="button"`, `tabindex="0"`, `aria-label`
- Enter key navigates to source line from focused node
- `aria-live` region with text description of graph
- "Describe Graph" command reads structure aloud
- All information supplemented with text (not color-only)

## Runtime Integration

Two modes for receiving runtime events:

### File-based (default)
```json
{ "borrowscope.runtime.source": "file" }
```
Extension watches `.borrowscope/events.json`. Run your instrumented program → file updates → overlay appears.

### WebSocket (live)
```json
{ "borrowscope.runtime.source": "websocket" }
```
Extension connects to `ws://localhost:9876`. Events stream in real-time as program runs.

### Divergence Detection (16 kinds)

| Kind | Severity | Description |
|------|----------|-------------|
| `rc_leak` | error | Rc/Arc never dropped |
| `rc_cycle` | error | Reference cycle detected |
| `missing_drop` | warning | Non-Copy var never dropped |
| `async_borrow_held` | warning | Borrow held across await |
| `weak_upgrade_fail` | warning | Weak::upgrade returned None |
| `channel_recv_fail` | warning | Channel receive failed |
| `use_after_move` | error | Events after move |
| `unsafe_hidden` | info | Unsafe hides ownership info |
| `conditional_move` | info | Move in untaken branch |

## Performance

- Extension activation: < 1s
- Graph render (50 nodes): < 100ms
- Debounced updates: configurable (default 300ms)
- Performance report via Command Palette
- Slow operations (>100ms) logged to output channel

## License

Apache-2.0
