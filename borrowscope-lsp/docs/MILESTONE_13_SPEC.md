# Milestone 13: Memory Layout Visualization

## Overview

Visualize the physical memory layout of Rust variables (stack and heap) as an interactive panel in VS Code. The user sees exactly where each variable lives in memory, how much space it occupies, which pointers connect stack to heap, and how the memory state evolves line-by-line as the program executes.

**Two modes:**
- **Static mode** (always available, no execution needed): Predicted layout from compile-time type information
- **Runtime mode** (after running with `borrowscope-runtime`): Actual addresses, real sizes, heap fragmentation

**Trigger:** CodeLens "🧠 Stack: 72B | Heap: ~28B" above each function, or command palette "BorrowScope: Show Memory Layout"

---

## 13.1 Motivating Example

**Given this Rust code:**
```rust
fn process_data() {
    let id: u32 = 42;                          // 4 bytes, stack
    let name = String::from("Alice");          // 24 bytes stack + 5 bytes heap
    let scores: Vec<f64> = vec![9.5, 8.7];    // 24 bytes stack + 16 bytes heap
    let name_ref = &name;                      // 8 bytes, stack (pointer to name)
    
    let shared = Rc::new(scores);              // 8 bytes stack + 16+24 bytes heap
    let shared2 = Rc::clone(&shared);          // 8 bytes stack (points to same heap)
    
    drop(shared2);                             // stack freed, refcount decremented
    drop(shared);                              // stack freed, heap freed (refcount=0)
    
    let boxed = Box::new([0u8; 1024]);         // 8 bytes stack + 1024 bytes heap
    drop(boxed);                               // stack freed, heap freed
}
```

**The visualization shows:**

```
Line 1: fn process_data()
═══════════════════════════════════════════════════════════════════

STACK (grows ↓)                          HEAP
┌──────────────────────────────┐         ┌──────────────────────────┐
│ [empty - function just       │         │ [empty]                  │
│  entered]                    │         │                          │
└──────────────────────────────┘         └──────────────────────────┘

Line 2: let id: u32 = 42;
═══════════════════════════════════════════════════════════════════

STACK                                    HEAP
┌──────────────────────────────┐         ┌──────────────────────────┐
│ ┌──────────────────────────┐ │         │ [empty]                  │
│ │ id: u32 = 42             │ │         │                          │
│ │ offset: 0  size: 4B      │ │         │                          │
│ │ align: 4   ████          │ │         │                          │
│ └──────────────────────────┘ │         │                          │
└──────────────────────────────┘         └──────────────────────────┘
Stack used: 4B                           Heap used: 0B

Line 3: let name = String::from("Alice");
═══════════════════════════════════════════════════════════════════

STACK                                    HEAP
┌──────────────────────────────┐         ┌──────────────────────────┐
│ ┌──────────────────────────┐ │         │ ┌──────────────────────┐ │
│ │ id: u32 = 42       4B   │ │         │ │ "Alice"    5B        │ │
│ └──────────────────────────┘ │         │ │ owned by: name       │ │
│ ┌──────────────────────────┐ │         │ └──────────────────────┘ │
│ │ name: String      24B   │─┼────────▶│                          │
│ │  ptr ─────────────────── │ │         │                          │
│ │  len: 5                  │ │         │                          │
│ │  cap: 5                  │ │         │                          │
│ └──────────────────────────┘ │         │                          │
└──────────────────────────────┘         └──────────────────────────┘
Stack used: 28B                          Heap used: 5B

Line 5: let name_ref = &name;
═══════════════════════════════════════════════════════════════════

STACK                                    HEAP
┌──────────────────────────────┐         ┌──────────────────────────┐
│ ┌──────────────────────────┐ │         │ ┌──────────────────────┐ │
│ │ id: u32 = 42       4B   │ │         │ │ "Alice"    5B        │ │
│ └──────────────────────────┘ │         │ └──────────────────────┘ │
│ ┌──────────────────────────┐ │         │ ┌──────────────────────┐ │
│ │ name: String      24B   │─┼────────▶│ │ [9.5, 8.7]  16B     │ │
│ └──────────────────────────┘ │         │ │ owned by: scores     │ │
│ ┌──────────────────────────┐ │         │ └──────────────────────┘ │
│ │ scores: Vec<f64>  24B   │─┼────────▶│                          │
│ └──────────────────────────┘ │         │                          │
│ ┌──────────────────────────┐ │         │                          │
│ │ name_ref: &String  8B   │ │         │                          │
│ │  ──▶ points to: name    │ │         │                          │
│ └──────────────────────────┘ │         │                          │
└──────────────────────────────┘         └──────────────────────────┘
Stack used: 60B                          Heap used: 21B
Borrows: name_ref → name (shared)
```

---

## 13.2 Server-Side: Memory Layout Analysis

**New endpoint: `borrowscope/memoryLayout`**

```rust
use ra_ap_hir as hir;
use ra_ap_ide_db::RootDatabase;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryLayoutInfo {
    pub function_name: String,
    pub stack_frame: StackFrame,
    pub heap_allocations: Vec<HeapAllocation>,
    pub pointer_relationships: Vec<PointerRelation>,
    pub timeline: Vec<MemoryEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StackFrame {
    pub total_size: u64,
    pub alignment: u64,
    pub variables: Vec<StackVariable>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StackVariable {
    pub name: String,
    pub type_display: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u64,
    pub lifetime: LineRange,
    pub category: MemoryCategory,
    pub internal_layout: Option<StructLayout>,
}

#[derive(Debug, Clone, Serialize)]
pub enum MemoryCategory {
    /// Fixed-size, stack-only (i32, bool, [T; N])
    StackOnly,
    /// Stack metadata + heap data (Vec, String, Box)
    HeapBacked,
    /// Pointer to another stack variable (&T, &mut T)
    Reference { target: String, mutable: bool },
    /// Reference-counted heap allocation (Rc, Arc)
    RefCounted { clone_count: usize },
    /// Interior mutability wrapper (RefCell, Mutex)
    InteriorMut,
}

/// Internal layout of compound types (Vec, String, etc.)
#[derive(Debug, Clone, Serialize)]
pub struct StructLayout {
    pub fields: Vec<FieldLayout>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub value_hint: String,  // "ptr", "len: 3", "cap: 4"
}

#[derive(Debug, Clone, Serialize)]
pub struct HeapAllocation {
    pub id: String,
    pub owner: String,
    pub type_display: String,
    pub size: u64,
    pub lifetime: LineRange,
    pub kind: HeapKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum HeapKind {
    VecBacking { element_size: u64, capacity: u64 },
    StringBuffer { capacity: u64 },
    BoxContent,
    RcInner { strong_count: usize, weak_count: usize },
    ArcInner { strong_count: usize, weak_count: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct PointerRelation {
    pub from: String,       // stack variable name
    pub to: String,         // heap allocation id or stack variable name
    pub kind: PointerKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum PointerKind {
    OwnsHeap,           // Vec.ptr → heap data
    BorrowsStack,       // &x → x
    BorrowsMutStack,    // &mut x → x
    SharesHeap,         // Rc clone → same Rc inner
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryEvent {
    pub line: u32,
    pub kind: MemoryEventKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum MemoryEventKind {
    StackAlloc { name: String, size: u64 },
    StackFree { name: String },
    HeapAlloc { id: String, size: u64, owner: String },
    HeapFree { id: String },
    HeapRealloc { id: String, old_size: u64, new_size: u64 },
    BorrowStart { borrower: String, target: String, mutable: bool },
    BorrowEnd { borrower: String },
    RcIncrement { id: String, new_count: usize },
    RcDecrement { id: String, new_count: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}
```

**Implementation using `hir::Type::layout()`:**

```rust
pub fn analyze_memory_layout(
    db: &RootDatabase,
    sema: &Semantics<'_, RootDatabase>,
    display_target: &DisplayTarget,
    function: &ast::Fn,
    line_index: &dyn Fn(TextSize) -> (u32, u32),
) -> MemoryLayoutInfo {
    let mut stack_vars = Vec::new();
    let mut heap_allocs = Vec::new();
    let mut pointers = Vec::new();
    let mut timeline = Vec::new();
    let mut current_offset: u64 = 0;

    let body = match function.body() { Some(b) => b, None => return empty_layout() };

    for let_stmt in body.syntax().descendants().filter_map(ast::LetStmt::cast) {
        let pat = match let_stmt.pat() { Some(p) => p, None => continue };
        let name = pat.syntax().text().to_string().trim().to_string();

        // Get the type
        let ty = match let_stmt.initializer().and_then(|e| sema.type_of_expr(&e)) {
            Some(t) => t.original,
            None => continue,
        };

        // Get layout from compiler
        let (size, align) = match ty.layout(db) {
            Ok(layout) => (layout.size(), layout.align()),
            Err(_) => (0, 1),
        };

        // Align the offset
        current_offset = (current_offset + align - 1) & !(align - 1);

        let (start_line, _) = line_index(let_stmt.syntax().text_range().start());
        let end_line = find_last_use_line(sema, &pat, &body, line_index);

        // Classify memory category
        let category = classify_memory_category(db, &ty, &name);

        // If heap-backed, create heap allocation
        if let MemoryCategory::HeapBacked = &category {
            let heap_size = estimate_heap_size(db, &ty);
            let heap_id = format!("{}.data", name);
            heap_allocs.push(HeapAllocation {
                id: heap_id.clone(),
                owner: name.clone(),
                type_display: format_heap_type(&ty, db, display_target),
                size: heap_size,
                lifetime: LineRange { start: start_line, end: end_line },
                kind: classify_heap_kind(db, &ty),
            });
            pointers.push(PointerRelation {
                from: name.clone(),
                to: heap_id.clone(),
                kind: PointerKind::OwnsHeap,
            });
            timeline.push(MemoryEvent {
                line: start_line,
                kind: MemoryEventKind::HeapAlloc { id: heap_id, size: heap_size, owner: name.clone() },
            });
        }

        // Build internal layout for compound types
        let internal_layout = build_internal_layout(db, &ty, display_target);

        stack_vars.push(StackVariable {
            name: name.clone(),
            type_display: ty.display(db, *display_target).to_string(),
            offset: current_offset,
            size,
            alignment: align,
            lifetime: LineRange { start: start_line, end: end_line },
            category,
            internal_layout,
        });

        timeline.push(MemoryEvent {
            line: start_line,
            kind: MemoryEventKind::StackAlloc { name: name.clone(), size },
        });
        timeline.push(MemoryEvent {
            line: end_line,
            kind: MemoryEventKind::StackFree { name },
        });

        current_offset += size;
    }

    MemoryLayoutInfo {
        function_name: function.name().map(|n| n.text().to_string()).unwrap_or_default(),
        stack_frame: StackFrame {
            total_size: current_offset,
            alignment: 8, // typical function frame alignment
            variables: stack_vars,
        },
        heap_allocations: heap_allocs,
        pointer_relationships: pointers,
        timeline,
    }
}

fn classify_memory_category(db: &RootDatabase, ty: &hir::Type<'_>, name: &str) -> MemoryCategory {
    if ty.is_reference() {
        let mutable = ty.is_mutable_reference();
        return MemoryCategory::Reference { target: String::new(), mutable };
    }

    if let Some(adt) = ty.as_adt() {
        let path = get_adt_path(db, &adt);
        if path.contains("vec") || path.contains("string") || path.contains("box") {
            return MemoryCategory::HeapBacked;
        }
        if path.contains("rc::rc") || path.contains("sync::arc") {
            return MemoryCategory::RefCounted { clone_count: 1 };
        }
        if path.contains("refcell") || path.contains("mutex") {
            return MemoryCategory::InteriorMut;
        }
    }

    MemoryCategory::StackOnly
}

fn estimate_heap_size(db: &RootDatabase, ty: &hir::Type<'_>) -> u64 {
    // For Vec<T>: estimate as capacity * size_of::<T>()
    // For String: estimate as capacity bytes
    // For Box<T>: size_of::<T>()
    // For Rc<T>: size_of::<T>() + 16 (strong + weak counts)
    // Default: 0
    if let Some(args) = ty.type_arguments().next() {
        if let Ok(layout) = args.layout(db) {
            return layout.size() * 4; // assume initial capacity of 4
        }
    }
    8 // default estimate
}

fn build_internal_layout(db: &RootDatabase, ty: &hir::Type<'_>, dt: &DisplayTarget) -> Option<StructLayout> {
    // For Vec<T>: show ptr (8B) + len (8B) + cap (8B)
    // For String: show ptr (8B) + len (8B) + cap (8B)
    // For &T: show ptr (8B)
    let type_str = ty.display(db, *dt).to_string();
    if type_str.starts_with("Vec<") || type_str == "String" {
        return Some(StructLayout {
            fields: vec![
                FieldLayout { name: "ptr".into(), offset: 0, size: 8, value_hint: "→ heap".into() },
                FieldLayout { name: "len".into(), offset: 8, size: 8, value_hint: "usize".into() },
                FieldLayout { name: "cap".into(), offset: 16, size: 8, value_hint: "usize".into() },
            ],
        });
    }
    None
}
```

---

## 13.3 VS Code Panel: Memory Layout WebView

**File: `src/graph/panel-memory.ts`**

```typescript
import * as vscode from "vscode";

export class MemoryPanel {
  public static currentPanel: MemoryPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;

  public static createOrShow(extensionUri: vscode.Uri, layout: any): void {
    const column = vscode.ViewColumn.Beside;

    if (MemoryPanel.currentPanel) {
      MemoryPanel.currentPanel._panel.reveal(column);
      MemoryPanel.currentPanel.update(layout);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      "borrowscopeMemory",
      "BorrowScope: Memory Layout",
      column,
      { enableScripts: true, retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(extensionUri, "media")] }
    );

    MemoryPanel.currentPanel = new MemoryPanel(panel, extensionUri, layout);
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri, layout: any) {
    this._panel = panel;
    this.update(layout);
    this._panel.onDidDispose(() => { MemoryPanel.currentPanel = undefined; });
  }

  public update(layout: any): void {
    this._panel.webview.html = this._buildHtml(layout);
  }

  private _buildHtml(layout: any): string {
    // Renders the full memory visualization with:
    // - Stack column (left)
    // - Heap column (right)
    // - Pointer arrows (SVG paths)
    // - Timeline slider (bottom)
    // - Stats bar (top)
    return `<!DOCTYPE html>...`; // Full implementation below
  }
}
```

**Command registration:**
```typescript
// In commands.ts
vscode.commands.registerCommand("borrowscope.showMemoryLayout", async (uri?: string, fn?: string) => {
  const client = getClient();
  if (!client) return;

  const editor = vscode.window.activeTextEditor;
  if (!editor) return;

  // Find function line
  let line = editor.selection.active.line;
  if (fn) {
    for (let i = 0; i < editor.document.lineCount; i++) {
      if (new RegExp(`\\bfn\\s+${fn}\\b`).test(editor.document.lineAt(i).text)) {
        line = i; break;
      }
    }
  }

  const layout = await client.sendRequest("borrowscope/memoryLayout", {
    textDocument: { uri: uri || editor.document.uri.toString() },
    position: { line, character: 4 },
  });

  if (layout) MemoryPanel.createOrShow(extensionUri, layout);
});
```

---

## 13.4 WebView HTML Structure

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin: 0; font-family: var(--vscode-font-family); background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); }
    
    #header { padding: 8px 16px; border-bottom: 1px solid var(--vscode-panel-border); display: flex; justify-content: space-between; }
    #mode-toggle { display: flex; gap: 4px; }
    #mode-toggle button { padding: 3px 10px; border: 1px solid var(--vscode-button-border); background: transparent; color: var(--vscode-foreground); border-radius: 3px; cursor: pointer; }
    #mode-toggle button.active { background: var(--vscode-button-background); color: var(--vscode-button-foreground); }
    
    #memory-view { display: flex; height: calc(100vh - 100px); gap: 2px; padding: 8px; }
    
    .column { flex: 1; border: 1px solid var(--vscode-panel-border); border-radius: 6px; padding: 8px; overflow-y: auto; }
    .column h3 { font-size: 12px; margin: 0 0 8px; padding-bottom: 6px; border-bottom: 1px solid var(--vscode-panel-border); }
    .column.stack h3 { color: #58a6ff; }
    .column.heap h3 { color: #3fb950; }
    
    .var-block { margin: 4px 0; padding: 6px 8px; border-radius: 4px; border-left: 3px solid; font-size: 11px; transition: all 0.3s; }
    .var-block.alive { opacity: 1; }
    .var-block.dead { opacity: 0.25; }
    .var-block .name { font-weight: bold; }
    .var-block .meta { color: var(--vscode-descriptionForeground); font-size: 10px; }
    .var-block .fields { margin-top: 4px; padding-left: 8px; border-left: 1px solid var(--vscode-panel-border); }
    .var-block .field { font-size: 10px; color: var(--vscode-descriptionForeground); }
    
    .pointer-svg { position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; }
    
    #timeline { padding: 8px 16px; border-top: 1px solid var(--vscode-panel-border); display: flex; align-items: center; gap: 12px; }
    #timeline button { background: var(--vscode-button-background); border: none; color: var(--vscode-button-foreground); padding: 4px 10px; border-radius: 3px; cursor: pointer; }
    #timeline input[type=range] { flex: 1; }
    
    #stats { display: flex; gap: 16px; font-size: 11px; padding: 4px 16px; }
    .stat { display: flex; align-items: center; gap: 4px; }
    .stat-dot { width: 8px; height: 8px; border-radius: 50%; }
  </style>
</head>
<body>
  <div id="header">
    <div>
      <strong>🧠 Memory Layout:</strong> <span id="fn-name">process_data()</span>
    </div>
    <div id="mode-toggle">
      <button class="active" id="btn-static">Static</button>
      <button id="btn-runtime">Runtime</button>
    </div>
  </div>
  
  <div id="stats">
    <div class="stat"><div class="stat-dot" style="background:#58a6ff"></div> Stack: <span id="s-stack">0B</span></div>
    <div class="stat"><div class="stat-dot" style="background:#3fb950"></div> Heap: <span id="s-heap">0B</span></div>
    <div class="stat"><div class="stat-dot" style="background:#d2a8ff"></div> Pointers: <span id="s-ptrs">0</span></div>
    <div class="stat"><div class="stat-dot" style="background:#f85149"></div> Freed: <span id="s-freed">0B</span></div>
  </div>
  
  <div id="memory-view">
    <div class="column stack">
      <h3>📦 STACK (frame size: <span id="frame-size">0</span>B)</h3>
      <div id="stack-content"></div>
    </div>
    <div class="column heap">
      <h3>🌐 HEAP</h3>
      <div id="heap-content"></div>
    </div>
  </div>
  
  <div id="timeline">
    <button id="btn-play">▶ Play</button>
    <button id="btn-step">⏭</button>
    <button id="btn-reset">⏮</button>
    <input type="range" id="slider" min="1" max="20" value="1">
    <span id="line-info">Line 1</span>
  </div>
  
  <script src="${d3Uri}"></script>
  <script>
    const layout = ${JSON.stringify(layout)};
    // ... D3.js rendering code ...
  </script>
</body>
</html>
```

---

## 13.5 Color Coding and Visual Language

| Element | Color | Meaning |
|---------|-------|---------|
| Stack variable (alive) | `#58a6ff` (blue) border | Currently on stack |
| Stack variable (dropped) | `#484f58` (gray) border, 25% opacity | Freed from stack |
| Heap block (alive) | `#3fb950` (green) border | Currently allocated |
| Heap block (freed) | `#f85149` (red) border, dashed | Returned to allocator |
| Pointer arrow (owns) | `#58a6ff` solid | Stack owns heap data |
| Pointer arrow (borrows) | `#d2a8ff` dashed | Reference to another var |
| Pointer arrow (Rc/Arc) | `#cba6f7` dotted | Shared ownership |
| Reallocation | `#f9e2af` flash | Old freed, new allocated |
| Borrow badge (shared) | `#58a6ff` background | `& shared` |
| Borrow badge (exclusive) | `#f85149` background | `&mut exclusive` |

**Variable block internal layout (for Vec, String):**
```
┌─────────────────────────────────────┐
│ name: String                   24B  │
│ ┌─────────┬─────────┬─────────┐    │
│ │ ptr: 8B │ len: 8B │ cap: 8B │    │
│ │ → heap  │ = 5     │ = 8     │    │
│ └─────────┴─────────┴─────────┘    │
│ lifetime: lines 3-15                │
└─────────────────────────────────────┘
```

---

## 13.6 Runtime Mode: Actual Addresses

**When runtime data is available, the panel shows:**

```
STACK (SP: 0x7ffd3a2b1000)               HEAP (brk: 0x55a1b2c30000)

0x7ffd3a2b1c00 ┌──────────────────┐     0x55a1b2c3d000 ┌──────────────┐
               │ id: u32 = 42     │                     │ "Alice" (5B) │
               │ 4 bytes          │                     │ owned: name  │
0x7ffd3a2b1c04 ├──────────────────┤     0x55a1b2c3d005 ├──────────────┤
               │ [padding: 4B]    │                     │ [gap: 11B]   │
0x7ffd3a2b1c08 ├──────────────────┤     0x55a1b2c3d010 ├──────────────┤
               │ name: String     │──────────────────▶  │ [9.5, 8.7]   │
               │ ptr: 0x55..d000  │                     │ (16B)        │
               │ len: 5           │                     │ owned: scores│
               │ cap: 8           │     0x55a1b2c3d020 └──────────────┘
0x7ffd3a2b1c20 ├──────────────────┤
               │ scores: Vec<f64> │──────────────────▶
               │ ptr: 0x55..d010  │
               │ len: 2           │
               │ cap: 4           │
0x7ffd3a2b1c38 ├──────────────────┤
               │ name_ref: &String│
               │ ptr: 0x7f..1c08  │─┐ (points to name on stack)
0x7ffd3a2b1c40 └──────────────────┘ │
                         ▲           │
                         └───────────┘
```

**Runtime-only information:**
- Exact hex addresses
- Padding bytes between variables (alignment gaps)
- Heap fragmentation (gaps between allocations)
- Actual capacity vs length (Vec may allocate more than needed)
- Allocator metadata overhead

**How runtime data is obtained:**
```rust
// In user's code (with borrowscope-runtime):
use borrowscope_runtime::*;

#[trace_borrow]
fn process_data() {
    let id: u32 = 42;
    track_stack_addr("id", &id as *const _ as usize, std::mem::size_of_val(&id));
    
    let name = String::from("Alice");
    track_stack_addr("name", &name as *const _ as usize, std::mem::size_of_val(&name));
    track_heap_addr("name.data", name.as_ptr() as usize, name.capacity());
    // ...
}
```

**Output file (`.borrowscope/memory-events.json`):**
```json
{
  "events": [
    {"type": "stack_alloc", "name": "id", "addr": "0x7ffd3a2b1c00", "size": 4, "line": 2, "timestamp_ns": 1000},
    {"type": "stack_alloc", "name": "name", "addr": "0x7ffd3a2b1c08", "size": 24, "line": 3, "timestamp_ns": 2000},
    {"type": "heap_alloc", "name": "name.data", "addr": "0x55a1b2c3d000", "size": 5, "line": 3, "timestamp_ns": 2100},
    {"type": "heap_realloc", "name": "name.data", "old_addr": "0x55a1b2c3d000", "new_addr": "0x55a1b2c3d020", "new_size": 11, "line": 7, "timestamp_ns": 5000},
    {"type": "heap_free", "name": "name.data", "addr": "0x55a1b2c3d020", "size": 11, "line": 15, "timestamp_ns": 9000}
  ]
}
```

---

## 13.7 CodeLens Integration

**Two CodeLens lines per function:**
```rust
  ▸ 5 vars, 2 borrows, 1 move              ← existing (→ graph panel)
  🧠 Stack: 72B | Heap: ~28B | 3 ptrs      ← NEW (→ memory panel)
  fn process_data(input: &[u8]) -> Result<Output> {
```

**Server code for memory CodeLens:**
```rust
// In handle_code_lens, after the existing ownership stats lens:
let layout = analyze_memory_layout(db, sema, display_target, &function, &line_index);
let heap_estimate: u64 = layout.heap_allocations.iter().map(|h| h.size).sum();
let ptr_count = layout.pointer_relationships.len();

lenses.push(serde_json::json!({
    "range": {"start": {"line": fn_line.saturating_sub(1), "character": 0}, "end": {"line": fn_line.saturating_sub(1), "character": 0}},
    "command": {
        "title": format!("🧠 Stack: {}B | Heap: ~{}B | {} ptrs", layout.stack_frame.total_size, heap_estimate, ptr_count),
        "command": "borrowscope.showMemoryLayout",
        "arguments": [uri_str, fn_name]
    }
}));
```

---

## 13.8 Animation Timeline

**Events are ordered by source line. The slider steps through them:**

```
Line 1:  fn entered → empty frame
Line 2:  StackAlloc(id, 4B) → id appears on stack
Line 3:  StackAlloc(name, 24B) + HeapAlloc(name.data, 5B) → name + arrow to heap
Line 4:  StackAlloc(scores, 24B) + HeapAlloc(scores.data, 16B) → scores + arrow
Line 5:  StackAlloc(name_ref, 8B) + BorrowStart(name_ref→name) → ref + dashed arrow
Line 6:  BorrowEnd(name_ref) + StackFree(name_ref) → ref fades out
Line 7:  HeapRealloc(name.data, 5→11B) → old block red, new block appears
...
Line 20: StackFree(all) → everything fades, frame empty
```

**Animation code (D3.js):**
```javascript
function animateToLine(targetLine) {
  const activeVars = layout.stack_frame.variables.filter(v => 
    targetLine >= v.lifetime.start && targetLine <= v.lifetime.end
  );
  const activeHeap = layout.heap_allocations.filter(h =>
    targetLine >= h.lifetime.start && targetLine <= h.lifetime.end
  );

  // Stack: enter/exit with transition
  const stackBlocks = d3.select('#stack-content').selectAll('.var-block')
    .data(activeVars, d => d.name);

  stackBlocks.enter().append('div')
    .attr('class', 'var-block alive')
    .style('opacity', 0)
    .html(d => renderStackBlock(d))
    .transition().duration(300)
    .style('opacity', 1);

  stackBlocks.exit()
    .classed('dead', true)
    .transition().duration(300)
    .style('opacity', 0)
    .remove();

  // Heap: similar enter/exit
  // Arrows: redraw SVG paths for active pointers
  updatePointerArrows(activeVars, activeHeap);
}
```

---

## 13.9 Type → Size Reference Table

| Rust Type | Stack Bytes | Heap Bytes | Notes |
|-----------|------------|------------|-------|
| `bool` | 1 | 0 | |
| `u8` / `i8` | 1 | 0 | |
| `u16` / `i16` | 2 | 0 | |
| `u32` / `i32` / `f32` | 4 | 0 | |
| `u64` / `i64` / `f64` | 8 | 0 | |
| `u128` / `i128` | 16 | 0 | |
| `usize` / `isize` | 8 | 0 | 64-bit |
| `char` | 4 | 0 | Unicode scalar |
| `&T` / `&mut T` | 8 | 0 | Thin pointer |
| `&dyn Trait` | 16 | 0 | Fat pointer (ptr + vtable) |
| `&[T]` | 16 | 0 | Fat pointer (ptr + len) |
| `&str` | 16 | 0 | Fat pointer (ptr + len) |
| `[T; N]` | N × size(T) | 0 | Inline array |
| `Vec<T>` | 24 | cap × size(T) | ptr + len + cap |
| `String` | 24 | cap | ptr + len + cap |
| `Box<T>` | 8 | size(T) | Thin pointer |
| `Rc<T>` | 8 | size(T) + 16 | ptr → {strong, weak, T} |
| `Arc<T>` | 8 | size(T) + 16 | ptr → {strong, weak, T} |
| `RefCell<T>` | size(T) + 8 | 0 | T + borrow flag |
| `Mutex<T>` | size(T) + 8 | 0 | T + lock state |
| `Option<T>` | size(T) + align(T) | 0 | Niche optimization may reduce |
| `Result<T, E>` | max(size(T), size(E)) + 8 | 0 | Tag + payload |

---

## 13.10 Implementation Steps

| Step | Description | Effort |
|------|-------------|--------|
| 13.1 | Server: `analyze_memory_layout()` using `ty.layout(db)` | 2 days |
| 13.2 | Server: `borrowscope/memoryLayout` LSP endpoint | 1 day |
| 13.3 | Server: Memory CodeLens (stack/heap sizes) | 0.5 days |
| 13.4 | Extension: `MemoryPanel` WebView class | 1 day |
| 13.5 | Extension: D3.js SVG rendering (stack + heap columns) | 3 days |
| 13.6 | Extension: Variable blocks with internal layout | 1 day |
| 13.7 | Extension: Pointer arrows (SVG curved paths) | 1 day |
| 13.8 | Extension: Timeline slider + play animation | 1 day |
| 13.9 | Extension: Runtime mode (load events, show addresses) | 2 days |
| 13.10 | Extension: Static/Runtime toggle | 0.5 days |
| 13.11 | Tests (server + extension) | 2 days |

**Total estimated effort:** 3 weeks

---

## 13.11 Tests

### Server-side (Rust):
```rust
#[test] fn test_layout_i32_is_4_bytes_stack_only() { }
#[test] fn test_layout_vec_is_24_stack_plus_heap() { }
#[test] fn test_layout_string_is_24_stack_plus_heap() { }
#[test] fn test_layout_box_is_8_stack_plus_heap() { }
#[test] fn test_layout_reference_is_8_bytes_no_heap() { }
#[test] fn test_layout_rc_is_8_stack_plus_refcount_heap() { }
#[test] fn test_layout_array_is_inline_stack() { }
#[test] fn test_layout_pointer_relationship_vec_to_heap() { }
#[test] fn test_layout_pointer_relationship_ref_to_stack() { }
#[test] fn test_layout_timeline_events_ordered_by_line() { }
#[test] fn test_layout_lifetime_matches_borrow_scope() { }
#[test] fn test_layout_alignment_padding_correct() { }
#[test] fn test_layout_internal_fields_for_vec() { }
#[test] fn test_layout_total_frame_size_correct() { }
```

### Extension-side (TypeScript):
```typescript
describe("13. Memory Layout Visualization", () => {
  it("MemoryPanel opens on command");
  it("shows stack variables with correct sizes");
  it("shows heap allocations with owners");
  it("pointer arrows connect stack to heap");
  it("timeline slider updates visible variables");
  it("dropped variables fade out");
  it("runtime mode shows hex addresses");
  it("static mode shows predicted layout");
  it("CodeLens shows stack/heap summary");
  it("click variable navigates to source");
  it("internal layout shows ptr/len/cap for Vec");
  it("reallocation shows old freed + new allocated");
});
```

---

## 13.12 Configuration

```json
{
  "borrowscope.memoryLayout.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show memory layout CodeLens and panel"
  },
  "borrowscope.memoryLayout.showInternalLayout": {
    "type": "boolean",
    "default": true,
    "description": "Show internal field layout (ptr/len/cap) for compound types"
  },
  "borrowscope.memoryLayout.showPadding": {
    "type": "boolean",
    "default": false,
    "description": "Show alignment padding bytes between variables"
  },
  "borrowscope.memoryLayout.animationSpeed": {
    "type": "number",
    "default": 600,
    "minimum": 100,
    "maximum": 2000,
    "description": "Animation speed in ms per line step"
  },
  "borrowscope.memoryLayout.runtimeEventsPath": {
    "type": "string",
    "default": ".borrowscope/memory-events.json",
    "description": "Path to runtime memory events file"
  }
}
```

---

**Priority:** After Milestone 12 (runtime integration)
**Dependencies:** Milestone 5 (WebView panel), Milestone 12 (runtime events for runtime mode)
**Estimated effort:** 3 weeks
