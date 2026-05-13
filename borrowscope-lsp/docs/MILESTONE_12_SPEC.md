# Milestone 12: Runtime + Static Analysis Integration

## Overview

Combine the static analysis (borrowscope-lsp, compile-time) with runtime tracking (borrowscope-runtime, execution-time) into a unified VS Code experience. The user sees both what the compiler knows AND what actually happens at runtime — in the same editor, same visualization.

**Why this matters:**
- Static analysis shows what the compiler *can* prove — but can't show dynamic dispatch, async timing, or conditional paths
- Runtime tracking shows what *actually* happens — but requires instrumentation and running the code
- Together: complete picture of ownership flow, from source to execution

---

## 12.1 Architecture: Two Data Sources, One UI

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          VS Code Extension                               │
│                                                                          │
│  ┌──────────────────────┐         ┌──────────────────────────────────┐  │
│  │   Static Layer        │         │   Runtime Layer                   │  │
│  │   (always active)     │         │   (active during/after execution) │  │
│  │                       │         │                                   │  │
│  │  • Inline annotations │         │  • Execution path highlighting    │  │
│  │  • Lifeline flow      │         │  • Actual borrow durations        │  │
│  │  • Hover tooltips     │         │  • Dynamic dispatch resolution    │  │
│  │  • CodeLens stats     │         │  • Async task flow                │  │
│  │  • Conflict detection │         │  • Reference count history        │  │
│  └──────────┬────────────┘         └──────────────┬────────────────────┘  │
│             │                                      │                      │
│             │         ┌────────────────────┐       │                      │
│             └────────►│   Unified Renderer  │◄─────┘                      │
│                       │                    │                              │
│                       │  Merges both views │                              │
│                       │  Shows agreements  │                              │
│                       │  Highlights diffs  │                              │
│                       └────────────────────┘                              │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                          Data Sources                                     │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────┐              ┌────────────────────────────────┐ │
│  │  borrowscope-lsp     │              │  borrowscope-runtime            │ │
│  │  (LSP over stdio)    │              │  (JSON event file / WebSocket)  │ │
│  │                      │              │                                 │ │
│  │  Provides:           │              │  Provides:                      │ │
│  │  • OwnershipGraph    │              │  • Event stream                 │ │
│  │  • BorrowScopes      │              │  • Actual lifetimes             │ │
│  │  • VariableInfo      │              │  • Rc/Arc ref counts            │ │
│  │  • Conflicts         │              │  • Drop order                   │ │
│  │  • InlayHints        │              │  • Async poll sequence          │ │
│  └─────────────────────┘              └────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 12.2 Runtime Event Ingestion

**How runtime data gets into VS Code:**

### Option A: File-based (simple, offline)

```
User workflow:
1. Add #[trace_borrow] to functions
2. Run: cargo run > /tmp/borrowscope-events.json
3. VS Code detects the file, loads events
4. Overlays runtime data on the static view
```

**Extension watches for event files:**

```typescript
// src/runtime-watcher.ts
import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

const EVENT_FILE = ".borrowscope/events.json";

export class RuntimeWatcher {
  private watcher: vscode.FileSystemWatcher | undefined;
  private events: RuntimeEvent[] = [];

  start(workspaceRoot: string): void {
    const pattern = new vscode.RelativePattern(workspaceRoot, EVENT_FILE);
    this.watcher = vscode.workspace.createFileSystemWatcher(pattern);

    this.watcher.onDidChange((uri) => this.loadEvents(uri.fsPath));
    this.watcher.onDidCreate((uri) => this.loadEvents(uri.fsPath));

    // Load existing file
    const existing = path.join(workspaceRoot, EVENT_FILE);
    if (fs.existsSync(existing)) {
      this.loadEvents(existing);
    }
  }

  private loadEvents(filePath: string): void {
    const content = fs.readFileSync(filePath, "utf8");
    this.events = JSON.parse(content);
    this.onEventsUpdated(this.events);
  }

  onEventsUpdated: (events: RuntimeEvent[]) => void = () => {};

  getEvents(): RuntimeEvent[] {
    return this.events;
  }

  dispose(): void {
    this.watcher?.dispose();
  }
}
```

### Option B: WebSocket (live, real-time)

```
User workflow:
1. Add borrowscope-runtime with "websocket" feature
2. Run the program — it streams events to ws://localhost:9876
3. VS Code connects to WebSocket, receives events live
4. Decorations update in real-time as the program runs
```

**Extension connects to runtime WebSocket:**

```typescript
// src/runtime-socket.ts
import * as WebSocket from "ws";

export class RuntimeSocket {
  private ws: WebSocket | undefined;
  private events: RuntimeEvent[] = [];

  connect(port: number = 9876): void {
    this.ws = new WebSocket(`ws://localhost:${port}`);

    this.ws.on("message", (data: string) => {
      const event: RuntimeEvent = JSON.parse(data);
      this.events.push(event);
      this.onEvent(event);
    });

    this.ws.on("close", () => {
      this.onDisconnect();
    });
  }

  onEvent: (event: RuntimeEvent) => void = () => {};
  onDisconnect: () => void = () => {};

  getEvents(): RuntimeEvent[] {
    return this.events;
  }

  disconnect(): void {
    this.ws?.close();
  }
}
```

### Option C: Debug Adapter Protocol integration

```
User workflow:
1. Start debugging (F5) with borrowscope-runtime instrumented
2. DAP adapter captures events alongside breakpoints
3. Events flow into VS Code automatically
4. No manual file management
```

---

## 12.3 Runtime Event Types (from borrowscope-runtime)

```typescript
// src/runtime-types.ts

interface RuntimeEvent {
  timestamp_ns: number;
  thread_id: number;
  kind: RuntimeEventKind;
}

type RuntimeEventKind =
  | { type: "New"; name: string; type_name: string; location: SourceLocation }
  | { type: "Drop"; name: string; location: SourceLocation }
  | { type: "Borrow"; name: string; target: string; mutable: boolean; location: SourceLocation }
  | { type: "BorrowEnd"; name: string; location: SourceLocation }
  | { type: "Move"; from: string; to: string; location: SourceLocation }
  | { type: "RcNew"; name: string; location: SourceLocation }
  | { type: "RcClone"; name: string; source: string; ref_count: number; location: SourceLocation }
  | { type: "RcDrop"; name: string; ref_count: number; location: SourceLocation }
  | { type: "ArcNew"; name: string; location: SourceLocation }
  | { type: "ArcClone"; name: string; source: string; ref_count: number; location: SourceLocation }
  | { type: "ArcDrop"; name: string; ref_count: number; location: SourceLocation }
  | { type: "RefCellBorrow"; name: string; target: string; mutable: boolean; location: SourceLocation }
  | { type: "RefCellRelease"; name: string; location: SourceLocation }
  | { type: "FutureCreate"; name: string; location: SourceLocation }
  | { type: "FuturePoll"; name: string; state: "Pending" | "Ready"; location: SourceLocation }
  | { type: "UnsafeEnter"; name: string; location: SourceLocation }
  | { type: "UnsafeExit"; name: string; location: SourceLocation };

interface SourceLocation {
  file: string;
  line: number;
  column: number;
}
```

---

## 12.4 Merging Static and Runtime Views

**The core challenge:** Map runtime events (which reference variable names and source locations) to the static analysis (which has semantic type information and borrow scopes).

### Mapping strategy:

```typescript
// src/merge-views.ts

interface MergedVariable {
  name: string;
  file: string;
  line: number;

  // From static analysis (borrowscope-lsp)
  static: {
    type_display: string;
    ownership_category: string;
    borrow_scope?: { start_line: number; end_line: number };
    is_copy: boolean;
  };

  // From runtime (borrowscope-runtime)
  runtime?: {
    actual_lifetime_ns: number;
    actual_borrow_count: number;
    actual_ref_count_peak: number;  // for Rc/Arc
    was_actually_moved: boolean;
    drop_order: number;
    thread_id: number;
  };

  // Comparison
  agreement: "match" | "diverge" | "runtime_only" | "static_only";
  divergence_reason?: string;
}

function mergeViews(
  staticGraph: OwnershipGraph,
  runtimeEvents: RuntimeEvent[],
  file: string
): MergedVariable[] {
  const merged: MergedVariable[] = [];

  for (const staticVar of staticGraph.variables) {
    const runtimeNew = runtimeEvents.find(
      (e) => e.kind.type === "New" && e.kind.name === staticVar.name
        && e.kind.location.file.endsWith(file)
        && e.kind.location.line === staticVar.line
    );

    const runtimeDrop = runtimeEvents.find(
      (e) => e.kind.type === "Drop" && e.kind.name === staticVar.name
    );

    const runtimeBorrows = runtimeEvents.filter(
      (e) => e.kind.type === "Borrow" && e.kind.target === staticVar.name
    );

    merged.push({
      name: staticVar.name,
      file,
      line: staticVar.line,
      static: {
        type_display: staticVar.type_display,
        ownership_category: staticVar.ownership_category,
        is_copy: staticVar.is_copy,
      },
      runtime: runtimeNew ? {
        actual_lifetime_ns: runtimeDrop
          ? runtimeDrop.timestamp_ns - runtimeNew.timestamp_ns
          : -1,
        actual_borrow_count: runtimeBorrows.length,
        actual_ref_count_peak: getRefCountPeak(runtimeEvents, staticVar.name),
        was_actually_moved: runtimeEvents.some(
          (e) => e.kind.type === "Move" && e.kind.from === staticVar.name
        ),
        drop_order: runtimeDrop ? runtimeEvents.indexOf(runtimeDrop) : -1,
        thread_id: runtimeNew.thread_id,
      } : undefined,
      agreement: computeAgreement(staticVar, runtimeNew, runtimeBorrows),
    });
  }

  return merged;
}

function computeAgreement(staticVar: any, runtimeNew: any, runtimeBorrows: any[]): string {
  if (!runtimeNew) return "static_only";

  // Check if static prediction matches runtime reality
  const staticSaysMoved = staticVar.ownership_category === "Owned"; // simplified
  const runtimeMoved = runtimeBorrows.length === 0; // simplified

  return "match"; // detailed comparison logic here
}

function getRefCountPeak(events: RuntimeEvent[], name: string): number {
  let count = 0;
  let peak = 0;
  for (const e of events) {
    if (e.kind.type === "RcClone" && e.kind.source === name) count++;
    if (e.kind.type === "RcDrop" && e.kind.name === name) count--;
    peak = Math.max(peak, count);
  }
  return peak;
}
```

---

## 12.5 Unified Visualization

### Inline annotations (combined):

```rust
fn example() {
    let data = vec![1, 2, 3];     // [Owned] ⏱ 1.2ms alive, dropped #3
    let r = &data;                 // [&] ⏱ 0.8ms, borrowed 2x at runtime
    let rc = Rc::new(42);         // [Rc] peak refs: 4, final drop at line 20
    process(&data);                // ──→ static: enters process() | runtime: 0.3ms in callee
}
```

### Decoration types for runtime overlay:

```typescript
// src/runtime-decorations.ts

const runtimeTimingDecoration = vscode.window.createTextEditorDecorationType({
  after: {
    color: "rgba(46, 204, 113, 0.6)",  // green for runtime data
    fontStyle: "italic",
    margin: "0 0 0 2em",
  },
});

const divergenceDecoration = vscode.window.createTextEditorDecorationType({
  backgroundColor: "rgba(231, 76, 60, 0.1)",  // red tint for divergences
  after: {
    color: "#e74c3c",
    fontStyle: "italic",
    margin: "0 0 0 1em",
  },
});

export function applyRuntimeOverlay(
  editor: vscode.TextEditor,
  merged: MergedVariable[]
): void {
  const timings: vscode.DecorationOptions[] = [];
  const divergences: vscode.DecorationOptions[] = [];

  for (const v of merged) {
    if (!v.runtime) continue;

    const line = v.line - 1;

    if (v.agreement === "match") {
      // Show timing info in green
      const lifetime = formatDuration(v.runtime.actual_lifetime_ns);
      timings.push({
        range: new vscode.Range(line, 0, line, 0),
        renderOptions: {
          after: {
            contentText: ` ⏱ ${lifetime}`,
            color: "rgba(46, 204, 113, 0.6)",
          },
        },
        hoverMessage: buildRuntimeHover(v),
      });
    } else if (v.agreement === "diverge") {
      // Show divergence in red
      divergences.push({
        range: new vscode.Range(line, 0, line, 0),
        renderOptions: {
          after: {
            contentText: ` ⚡ ${v.divergence_reason}`,
            color: "#e74c3c",
          },
        },
        hoverMessage: buildDivergenceHover(v),
      });
    }
  }

  editor.setDecorations(runtimeTimingDecoration, timings);
  editor.setDecorations(divergenceDecoration, divergences);
}

function formatDuration(ns: number): string {
  if (ns < 1000) return `${ns}ns`;
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(1)}μs`;
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(1)}ms`;
  return `${(ns / 1_000_000_000).toFixed(2)}s`;
}

function buildRuntimeHover(v: MergedVariable): vscode.MarkdownString {
  const md = new vscode.MarkdownString();
  md.appendMarkdown(`**${v.name}** — Runtime Data\n\n`);
  md.appendMarkdown(`| Property | Value |\n|---|---|\n`);
  md.appendMarkdown(`| Lifetime | ${formatDuration(v.runtime!.actual_lifetime_ns)} |\n`);
  md.appendMarkdown(`| Borrow count | ${v.runtime!.actual_borrow_count} |\n`);
  md.appendMarkdown(`| Thread | ${v.runtime!.thread_id} |\n`);
  md.appendMarkdown(`| Drop order | #${v.runtime!.drop_order} |\n`);
  if (v.runtime!.actual_ref_count_peak > 0) {
    md.appendMarkdown(`| Peak ref count | ${v.runtime!.actual_ref_count_peak} |\n`);
  }
  return md;
}

function buildDivergenceHover(v: MergedVariable): vscode.MarkdownString {
  const md = new vscode.MarkdownString();
  md.appendMarkdown(`**⚡ Static/Runtime Divergence:** \`${v.name}\`\n\n`);
  md.appendMarkdown(`**Static says:** ${v.static.ownership_category}\n\n`);
  md.appendMarkdown(`**Runtime shows:** ${v.divergence_reason}\n\n`);
  md.appendMarkdown(`This can happen with dynamic dispatch, conditional paths, or unsafe code.\n`);
  return md;
}
```

---

## 12.6 Divergence Detection

**When static and runtime disagree:**

| Scenario | Static says | Runtime shows | Divergence |
|----------|-------------|---------------|------------|
| Conditional move | "Owned (may move)" | "Not moved (else branch taken)" | Path-dependent |
| Dynamic dispatch | "Unknown callee" | "Called `Vec::push`" | Resolved at runtime |
| Unsafe code | "Raw pointer" | "Actually borrowed safely" | Unsafe hides info |
| Rc cycle | "Rc (will drop)" | "Leaked (ref count never 0)" | Memory leak |
| Async timing | "Borrow scope lines 5-10" | "Borrow held across await (2.3s)" | Async suspension |
| Interior mutability | "RefCell (shared)" | "Panicked: already borrowed" | Runtime borrow violation |

**Divergence decoration examples:**

```rust
fn example() {
    let data = vec![1, 2, 3];

    // Static: "data may be moved on line 8"
    // Runtime: "data was NOT moved (else branch taken)"
    if condition {
        consume(data);              // ⚡ Static: move here | Runtime: branch not taken
    } else {
        println!("{:?}", data);     // ⏱ 0.1ms — data still alive here
    }

    // Static: "rc will be dropped at end of scope"
    // Runtime: "rc leaked — ref count never reached 0"
    let rc = Rc::new(42);
    create_cycle(&rc);              // ⚡ Rc cycle detected: peak refs=3, never dropped
}
```

---

## 12.7 Rc/Arc Reference Count Timeline

**Show how reference counts change over time:**

```typescript
// src/refcount-timeline.ts

interface RefCountPoint {
  line: number;
  count: number;
  event: "new" | "clone" | "drop";
  variable: string;
}

function buildRefCountTimeline(events: RuntimeEvent[], varName: string): RefCountPoint[] {
  const points: RefCountPoint[] = [];
  let count = 0;

  for (const e of events) {
    if (e.kind.type === "RcNew" && e.kind.name === varName) {
      count = 1;
      points.push({ line: e.kind.location.line, count, event: "new", variable: varName });
    } else if (e.kind.type === "RcClone" && e.kind.source === varName) {
      count++;
      points.push({ line: e.kind.location.line, count, event: "clone", variable: e.kind.name });
    } else if (e.kind.type === "RcDrop" && e.kind.name === varName) {
      count--;
      points.push({ line: e.kind.location.line, count, event: "drop", variable: varName });
    }
  }

  return points;
}
```

**Visualization in editor:**

```rust
fn rc_example() {
    let rc1 = Rc::new(data);       // 🔗 refs: 1
    let rc2 = rc1.clone();         // 🔗 refs: 2
    let rc3 = rc1.clone();         // 🔗 refs: 3
    drop(rc2);                     // 🔗 refs: 2
    drop(rc3);                     // 🔗 refs: 1
}                                  // 🔗 refs: 0 → dropped
```

---

## 12.8 Async Ownership Flow

**Track borrows across await points:**

```rust
async fn fetch_and_process(url: &str) -> Result<()> {
    let data = fetch(url).await;        // ⏱ awaited 230ms
    let parsed = parse(&data);          // 👁 &data — held across next await!
    let result = validate(parsed).await; // ⚡ &data alive across await (230ms + 45ms)
    println!("{:?}", result);
}
```

**Runtime events for async:**

```typescript
interface AsyncBorrowInfo {
  variable: string;
  borrow_start_line: number;
  await_points_crossed: Array<{
    line: number;
    duration_ms: number;
    poll_count: number;
  }>;
  total_held_duration_ms: number;
}
```

**Why this matters:** A borrow held across an await point can cause issues in multi-threaded runtimes. Static analysis knows the borrow exists, but only runtime knows how long it's actually held.

---

## 12.9 Drop Order Visualization

**Show the actual order variables are dropped:**

```rust
fn example() {
    let a = String::from("first");    // drop order: #3
    let b = String::from("second");   // drop order: #2
    let c = String::from("third");    // drop order: #1 (LIFO)
}
// Runtime shows: c dropped, then b, then a
```

**Decoration:**

```typescript
const dropOrderDecoration = vscode.window.createTextEditorDecorationType({
  after: {
    color: "rgba(149, 165, 166, 0.5)",
    fontStyle: "italic",
  },
});

function applyDropOrder(editor: vscode.TextEditor, events: RuntimeEvent[]): void {
  const drops = events
    .filter((e) => e.kind.type === "Drop")
    .map((e, idx) => ({ ...e, order: idx + 1 }));

  const decorations = drops.map((d) => ({
    range: new vscode.Range(d.kind.location.line - 1, 0, d.kind.location.line - 1, 0),
    renderOptions: {
      after: { contentText: ` 💀 #${d.order}` },
    },
  }));

  editor.setDecorations(dropOrderDecoration, decorations);
}
```

---

## 12.10 Configuration

```json
{
  "borrowscope.runtime.enabled": {
    "type": "boolean",
    "default": false,
    "description": "Enable runtime data overlay (requires borrowscope-runtime instrumentation)"
  },
  "borrowscope.runtime.source": {
    "type": "string",
    "enum": ["file", "websocket", "dap"],
    "default": "file",
    "description": "How to receive runtime events"
  },
  "borrowscope.runtime.filePath": {
    "type": "string",
    "default": ".borrowscope/events.json",
    "description": "Path to runtime events file (relative to workspace)"
  },
  "borrowscope.runtime.websocketPort": {
    "type": "number",
    "default": 9876,
    "description": "WebSocket port for live runtime events"
  },
  "borrowscope.runtime.showTimings": {
    "type": "boolean",
    "default": true,
    "description": "Show variable lifetime durations"
  },
  "borrowscope.runtime.showDropOrder": {
    "type": "boolean",
    "default": false,
    "description": "Show drop order numbers"
  },
  "borrowscope.runtime.showRefCounts": {
    "type": "boolean",
    "default": true,
    "description": "Show Rc/Arc reference count changes"
  },
  "borrowscope.runtime.highlightDivergences": {
    "type": "boolean",
    "default": true,
    "description": "Highlight where static and runtime analysis disagree"
  }
}
```

---

## 12.11 User Workflow

### Setup (one-time):

```toml
# Cargo.toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track", "json-export"] }
borrowscope-macro = "0.1"
```

```rust
// main.rs
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn main() {
    // ... your code ...
}

// At end of main:
fn main() {
    // ... code ...
    borrowscope_runtime::export_json(".borrowscope/events.json");
}
```

### Daily use:

```
1. Write Rust code
   → See static analysis (inline annotations, lifelines, hover)

2. Run the program (cargo run)
   → .borrowscope/events.json is generated

3. VS Code detects the file
   → Runtime overlay appears (green timings, red divergences)

4. Edit code
   → Static updates instantly
   → Runtime overlay shows "stale" indicator until next run

5. Run again
   → Runtime overlay refreshes with new data
```

### Status bar indicator:

```
┌─────────────────────────────────────────────────────────────┐
│  BorrowScope: Static ✓ | Runtime ✓ (events: 1,247, 0.3s ago) │
└─────────────────────────────────────────────────────────────┘
```

---

## 12.12 Implementation Steps

| Step | Description | Effort |
|------|-------------|--------|
| 12.1 | Runtime event file watcher | 1 day |
| 12.2 | Event parser (JSON → TypeScript types) | 1 day |
| 12.3 | Variable mapping (runtime name+line → static variable) | 2 days |
| 12.4 | Merge logic (combine static + runtime into MergedVariable) | 2 days |
| 12.5 | Runtime timing decorations (green overlay) | 1 day |
| 12.6 | Divergence detection and highlighting | 2 days |
| 12.7 | Rc/Arc ref count timeline | 1 day |
| 12.8 | Drop order visualization | 1 day |
| 12.9 | WebSocket live connection (Option B) | 2 days |
| 12.10 | Async borrow tracking across await points | 3 days |
| 12.11 | Configuration and toggle UI | 1 day |
| 12.12 | Status bar indicator | 0.5 days |

**Total estimated effort:** 3-4 weeks

---

## 12.13 Tests

### Server-side (Rust — borrowscope-runtime integration):

```rust
#[test]
fn test_runtime_events_export_json() {
    // Run instrumented code, verify JSON output format
}

#[test]
fn test_event_source_locations_match_source() {
    // Verify line numbers in events match actual source
}

#[test]
fn test_rc_ref_count_tracking() {
    // Clone 3 times, drop 2, verify count sequence
}

#[test]
fn test_async_borrow_across_await() {
    // Borrow held across .await, verify timing captured
}
```

### Extension-side (TypeScript):

```typescript
describe("12. Runtime + Static Integration", () => {
  it("loads events from JSON file");
  it("maps runtime events to static variables by name+line");
  it("shows timing decoration for matched variables");
  it("detects divergence when move doesn't happen at runtime");
  it("shows ref count peak for Rc variables");
  it("marks stale runtime data after file edit");
  it("clears runtime overlay when file is deleted");
  it("handles empty events file gracefully");
  it("WebSocket connection receives live events");
  it("drop order numbers are correct (LIFO)");
  it("async borrow duration includes await time");
  it("divergence hover explains the difference");
});
```

---

## 12.14 Limitations

| Limitation | Reason | Mitigation |
|-----------|--------|------------|
| Requires instrumentation | `#[trace_borrow]` must be added | Provide CLI tool to auto-instrument |
| Runtime overhead (~80ns/event) | Tracking has cost | Sampling mode for hot paths |
| Name matching is fragile | Variable renamed → events don't match | Use source location as primary key |
| Optimized builds may differ | Release mode may elide drops | Warn user if running in release |
| Multi-threaded timing | Events from different threads interleave | Group by thread_id |
| Macro-generated code | Source locations may not match | Use span info from proc macro |

---

## 12.15 Future Extensions

- **Time-travel debugging:** Scrub through events timeline, see ownership state at any point
- **Flame graph of ownership:** Visualize which variables consume the most lifetime
- **Leak detection:** Rc cycles that never reach ref count 0
- **Performance profiling:** Which borrows are held longest? Which drops are expensive?
- **Test coverage for ownership:** "This borrow path was never exercised at runtime"

---

**Priority:** After Milestone 11 (cross-function tracking)
**Estimated effort:** 3-4 weeks
**Dependencies:** 
- borrowscope-runtime (existing, provides events)
- borrowscope-macro (existing, provides instrumentation)
- Milestone 4 (VS Code extension infrastructure)
- Milestone 5 (WebView panel for timeline visualization)
