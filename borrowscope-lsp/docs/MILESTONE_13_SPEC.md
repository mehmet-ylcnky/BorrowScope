# Milestone 13: Memory Layout Visualization

## Overview

Visualize the physical memory layout of Rust variables (stack and heap) as an interactive panel in VS Code. Shows where each variable lives in memory, its size, pointer relationships, and lifecycle — all within the editor alongside the code.

**Two modes:**
- **Static mode** (always available): Shows predicted layout from type information (sizes, stack/heap classification, pointer relationships)
- **Runtime mode** (after execution): Shows actual addresses, real allocation sizes, heap fragmentation, and drop order from `borrowscope-runtime` event capture

**Trigger:** New CodeLens action "Show Memory Layout" above each function, or command palette "BorrowScope: Show Memory Layout".

---

## 13.1 Static Memory Layout Analysis (Server-Side)

**Objective:** Extract memory layout information from `hir::Type::layout()` for all variables in a function.

**New LSP endpoint:**
```
Request:
  method: "borrowscope/memoryLayout"
  params: {
    "textDocument": { "uri": "file:///..." },
    "position": { "line": 5, "character": 0 }
  }

Response:
  result: {
    "function_name": "main",
    "stack_frame": {
      "total_size": 96,
      "alignment": 8,
      "variables": [
        {
          "name": "x",
          "type": "i32",
          "offset": 0,
          "size": 4,
          "alignment": 4,
          "region": "stack",
          "lifetime": { "start_line": 2, "end_line": 20 },
          "category": "Copy"
        },
        {
          "name": "v",
          "type": "Vec<i32>",
          "offset": 8,
          "size": 24,
          "alignment": 8,
          "region": "stack",
          "heap_allocation": {
            "element_size": 4,
            "capacity": "dynamic",
            "estimated_bytes": "len * 4"
          },
          "lifetime": { "start_line": 3, "end_line": 18 },
          "category": "Owned",
          "points_to_heap": true
        },
        {
          "name": "r",
          "type": "&Vec<i32>",
          "offset": 32,
          "size": 8,
          "alignment": 8,
          "region": "stack",
          "lifetime": { "start_line": 4, "end_line": 6 },
          "category": "SharedRef",
          "points_to": "v"
        }
      ]
    },
    "heap_allocations": [
      {
        "owner": "v",
        "type": "[i32]",
        "estimated_size": 12,
        "lifetime": { "start_line": 3, "end_line": 18 },
        "kind": "Vec backing storage"
      }
    ]
  }
```

**Server implementation (analysis.rs):**
```rust
#[derive(Debug, Serialize)]
pub struct MemoryLayoutInfo {
    pub function_name: String,
    pub stack_frame: StackFrame,
    pub heap_allocations: Vec<HeapAllocation>,
}

#[derive(Debug, Serialize)]
pub struct StackFrame {
    pub total_size: u64,
    pub alignment: u64,
    pub variables: Vec<StackVariable>,
}

#[derive(Debug, Serialize)]
pub struct StackVariable {
    pub name: String,
    pub type_display: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u64,
    pub region: String,  // "stack"
    pub lifetime: Lifetime,
    pub category: String,
    pub points_to_heap: bool,
    pub points_to: Option<String>,  // name of another stack variable
}

#[derive(Debug, Serialize)]
pub struct HeapAllocation {
    pub owner: String,
    pub type_display: String,
    pub estimated_size: u64,
    pub lifetime: Lifetime,
    pub kind: String,  // "Vec backing", "String buffer", "Box content", "Rc inner"
}

#[derive(Debug, Serialize)]
pub struct Lifetime {
    pub start_line: u32,
    pub end_line: u32,
}

pub fn analyze_memory_layout(
    db: &RootDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
    line_index: &dyn Fn(TextSize) -> (u32, u32),
) -> MemoryLayoutInfo {
    // For each variable:
    // 1. ty.layout(db) → size, alignment
    // 2. Classify: stack-only (Copy, fixed arrays) vs heap-allocating (Vec, String, Box)
    // 3. Determine pointer relationships
    // 4. Compute lifetime from borrow scopes
    todo!()
}
```

**Type → Memory classification:**

| Type | Stack Size | Heap | Kind |
|------|-----------|------|------|
| `i32`, `bool`, `f64` | 4, 1, 8 | None | Copy, stack-only |
| `[T; N]` | N * size_of::<T>() | None | Fixed array, stack |
| `Vec<T>` | 24 (ptr+len+cap) | len * size_of::<T>() | Dynamic, heap-backed |
| `String` | 24 (ptr+len+cap) | len bytes | Dynamic, heap-backed |
| `Box<T>` | 8 (ptr) | size_of::<T>() | Heap-allocated |
| `Rc<T>` | 8 (ptr) | size_of::<T>() + 16 (strong+weak) | Ref-counted, heap |
| `Arc<T>` | 8 (ptr) | size_of::<T>() + 16 (atomic counts) | Thread-safe, heap |
| `&T` | 8 (ptr) | None | Pointer to existing |
| `&mut T` | 8 (ptr) | None | Exclusive pointer |
| `RefCell<T>` | size_of::<T>() + 8 (borrow flag) | None | Stack with flag |

---

## 13.2 VS Code Panel: Memory Layout View

**Trigger options:**
1. **CodeLens:** "🧠 Show Memory Layout" above each function (alongside existing stats CodeLens)
2. **Command palette:** "BorrowScope: Show Memory Layout"
3. **Context menu:** Right-click inside a function → "Show Memory Layout"

**Panel layout (WebView):**

```
┌─────────────────────────────────────────────────────────────────┐
│  🧠 Memory Layout: main()                    [Static] [Runtime] │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─── STACK (96 bytes) ──────────┐  ┌─── HEAP ──────────────┐  │
│  │                                │  │                        │  │
│  │  ┌────────────────────────┐   │  │                        │  │
│  │  │ x: i32        4B  ████│   │  │                        │  │
│  │  └────────────────────────┘   │  │                        │  │
│  │  ┌────────────────────────┐   │  │  ┌──────────────────┐ │  │
│  │  │ v: Vec<i32>  24B      │───┼──┼─▶│ [1,2,3]  12B     │ │  │
│  │  │  ptr | len:3 | cap:4  │   │  │  └──────────────────┘ │  │
│  │  └────────────────────────┘   │  │                        │  │
│  │  ┌────────────────────────┐   │  │  ┌──────────────────┐ │  │
│  │  │ s: String    24B      │───┼──┼─▶│ "hello"   5B     │ │  │
│  │  └────────────────────────┘   │  │  └──────────────────┘ │  │
│  │  ┌────────────────────────┐   │  │                        │  │
│  │  │ r: &Vec       8B  ─┐  │   │  │                        │  │
│  │  └───────────────────┼──┘   │  │                        │  │
│  │         points to v ──┘      │  │                        │  │
│  │                                │  │                        │  │
│  └────────────────────────────────┘  └────────────────────────┘  │
│                                                                  │
│  ▶ Play  ⏭ Step  ⏮ Reset   ═══════════════════  Line 5/20     │
└─────────────────────────────────────────────────────────────────┘
```

**Interactive features:**
- **Timeline slider:** Step through source lines, variables appear/disappear
- **Play button:** Animate the lifecycle automatically
- **Hover:** Show full type info, exact size, alignment
- **Click variable:** Navigate to declaration in editor
- **Arrows:** Animated pointer connections (stack→heap, ref→target)
- **Color coding:** Green=alive, Gray=dropped, Red=moved, Blue=borrowed

**Implementation (panel-memory.ts):**
```typescript
export class MemoryPanel {
  public static createOrShow(extensionUri: vscode.Uri, layout: MemoryLayoutInfo): void {
    // Similar to GraphPanel but renders memory layout SVG
    // Uses D3.js for the stack/heap visualization
    // Timeline slider controls which line is "current"
  }
}
```

---

## 13.3 Runtime Mode (with borrowscope-runtime)

**When runtime data is available** (user ran the program with instrumentation), the panel switches to show actual addresses:

**Additional data from runtime:**
```json
{
  "runtime_events": [
    { "type": "stack_alloc", "name": "x", "addr": "0x7ffd3a2b1c00", "size": 4, "timestamp_ns": 1000 },
    { "type": "heap_alloc", "name": "v.data", "addr": "0x55a1b2c3d000", "size": 16, "timestamp_ns": 2000 },
    { "type": "heap_realloc", "name": "s.data", "old_addr": "0x55a1b2c3d020", "new_addr": "0x55a1b2c3d040", "new_size": 11, "timestamp_ns": 5000 },
    { "type": "heap_free", "name": "v.data", "addr": "0x55a1b2c3d000", "size": 16, "timestamp_ns": 9000 }
  ]
}
```

**Runtime-only features:**
- Actual hex addresses displayed
- Heap fragmentation visible (gaps between allocations)
- Reallocation animation (old block freed, new block allocated elsewhere)
- Actual drop order (may differ from source order due to compiler optimization)
- Stack pointer movement visible
- Cache line boundaries (64-byte aligned markers)

**Toggle:** [Static] / [Runtime] button in panel header switches between modes.

---

## 13.4 CodeLens Integration

**Add memory layout CodeLens alongside existing ownership stats:**

```rust
  ▸ 5 vars, 2 borrows, 1 move          ← existing (click → graph panel)
  🧠 Stack: 72B | Heap: ~28B            ← NEW (click → memory panel)
  fn process_data(input: &[u8]) -> Result<Output> {
```

**Server-side (requests.rs):**
```rust
// In handle_code_lens, add a second lens per function:
lenses.push(CodeLens {
    range: fn_line_range,
    command: Command {
        title: format!("🧠 Stack: {}B | Heap: ~{}B", stack_size, heap_estimate),
        command: "borrowscope.showMemoryLayout",
        arguments: vec![uri, fn_name],
    },
});
```

---

## 13.5 Animated Lifecycle

**The timeline shows memory state at each source line:**

```
Line 1:  [x]
Line 2:  [x][v─────────]  →  [v.data]
Line 3:  [x][v─────────][r]  →  [v.data]    r points to v
Line 4:  [x][v─────────]  →  [v.data]       r released
Line 5:  [x][v─────────][s─────────]  →  [v.data]["hello"]
Line 6:  [x][v─────────][s─────────][m]  →  [v.data]["hello"]  m→s
Line 7:  [x][v─────────][s─────────]  →  [v.data]["hello world"]  (realloc!)
...
Line 20: []  →  []   (all freed)
```

**Animation events:**
- Variable appears: fade in + slide down on stack
- Variable dropped: fade out + slide up
- Heap allocation: block appears on heap side with arrow from stack
- Heap free: block turns red, fades out
- Reallocation: old block fades, new block appears at different position
- Borrow created: dashed arrow appears between stack vars
- Borrow released: arrow fades out

---

## 13.6 Implementation Steps

| Step | Description | Effort |
|------|-------------|--------|
| 13.1 | Server: `analyze_memory_layout()` using `ty.layout(db)` | 2 days |
| 13.2 | Server: `borrowscope/memoryLayout` LSP endpoint | 1 day |
| 13.3 | Server: Memory CodeLens (stack/heap sizes) | 0.5 days |
| 13.4 | Extension: `MemoryPanel` WebView (static mode) | 3 days |
| 13.5 | Extension: D3.js SVG rendering (stack + heap columns) | 2 days |
| 13.6 | Extension: Timeline slider + play animation | 1 day |
| 13.7 | Extension: Pointer arrows (stack→heap, ref→var) | 1 day |
| 13.8 | Extension: Runtime mode (load events from file) | 2 days |
| 13.9 | Extension: Toggle Static/Runtime in panel | 0.5 days |
| 13.10 | Tests | 2 days |

**Total estimated effort:** 2-3 weeks

---

## 13.7 Tests

### Server-side (Rust):
```rust
#[test]
fn test_memory_layout_i32_is_4_bytes_stack() { }
#[test]
fn test_memory_layout_vec_is_24_bytes_stack_plus_heap() { }
#[test]
fn test_memory_layout_reference_is_8_bytes() { }
#[test]
fn test_memory_layout_box_is_8_bytes_stack_plus_heap() { }
#[test]
fn test_memory_layout_rc_shows_refcount_overhead() { }
#[test]
fn test_memory_layout_string_shows_heap_allocation() { }
#[test]
fn test_memory_layout_lifetime_matches_borrow_scope() { }
#[test]
fn test_memory_layout_points_to_correct_target() { }
```

### Extension-side (TypeScript):
```typescript
describe("13. Memory Layout Visualization", () => {
  it("panel opens on command");
  it("shows stack variables with sizes");
  it("shows heap allocations with owners");
  it("arrows connect stack pointers to heap");
  it("timeline slider updates display");
  it("dropped variables fade out");
  it("runtime mode shows actual addresses");
  it("static mode shows predicted layout");
  it("CodeLens shows stack/heap summary");
  it("click variable navigates to source");
});
```

---

## 13.8 Configuration

```json
{
  "borrowscope.memoryLayout.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show memory layout CodeLens and panel"
  },
  "borrowscope.memoryLayout.showAddresses": {
    "type": "boolean",
    "default": false,
    "description": "Show hex addresses (runtime mode only)"
  },
  "borrowscope.memoryLayout.animationSpeed": {
    "type": "number",
    "default": 600,
    "description": "Animation speed in ms per step"
  }
}
```

---

**Priority:** After Milestone 12 (runtime integration provides the event data)
**Dependencies:** Milestone 5 (WebView panel infrastructure), Milestone 12 (runtime events for runtime mode)
**Estimated effort:** 2-3 weeks
