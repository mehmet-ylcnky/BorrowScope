# Milestone 11: Cross-Function Borrow Tracking - Detailed Specification

## 11.1 Inter-Procedural Borrow Analysis

**Objective:** Track borrow lifetimes across function boundaries. When a reference is passed from a caller to a callee, show the full borrow path spanning both functions. This enables visualizing how ownership flows through the call graph.

**Current limitation:** Each function is analyzed independently. A borrow created in `main()` and passed to `process(&data)` shows as two disconnected scopes — one in `main` (the `&data` argument) and one in `process` (the `r` parameter). The user cannot see that they are the same borrow.

**Goal:** Show a unified lifeline that spans:
```
main():
  ├─ 👁 &data created
  │   process(&data)  ──────────┐
  │                              │  (borrow flows into callee)
  │   process():                 │
  │     ├─ r receives &data  ◄──┘
  │     │  r.len()
  │     ╰─ r released
  │
  ╰─ &data released (after process returns)
```

## 11.2 Call Graph Resolution

**Steps:**
1. For each function call in the analyzed function, resolve the call target using `hir::Function`
2. Identify which arguments are references (borrows being passed)
3. Map caller argument positions to callee parameter positions
4. Analyze the callee to determine how long the parameter lives

**Server endpoint:**
```
Request:
  method: "borrowscope/crossFunctionBorrows"
  params: { "textDocument": { "uri": "..." } }

Response:
  result: {
    "cross_borrows": [
      {
        "origin_variable": "data",
        "origin_function": "main",
        "origin_line": 5,
        "path": [
          { "function": "main", "variable": "&data", "start_line": 5, "end_line": 12 },
          { "function": "process", "variable": "r", "start_line": 1, "end_line": 5 }
        ]
      }
    ]
  }
```

## 11.3 Implementation Approach

**Phase 1: Direct calls only (no trait dispatch)**
- Resolve `fn foo(&data)` → find `fn foo(r: &Vec<i32>)`
- Map argument 0 → parameter 0
- Analyze callee's use of that parameter
- Report combined span

**Phase 2: One level deep**
- Only trace caller → direct callee (not callee → callee's callee)
- Skip recursive calls
- Skip trait objects (`&dyn Trait`) and dynamic dispatch

**Phase 3: Full call chain (future)**
- Multi-level tracing with cycle detection
- Trait resolution for known concrete types
- Generic monomorphization

## 11.4 Visualization in VS Code

**Option A: Expanded inline view**
```rust
fn main() {
  ├─ 👁 &data ⟵ data
  │   process(&data);        // ──→ enters process()
  │   │ 👁 r active in process()
  │   │ r.len()
  │   │ 💧 r released in process()
  │   // ◄── returns from process()
  ╰─ 💧 &data released
```

**Option B: Collapsed with expand-on-click**
```rust
fn main() {
  ├─ 👁 &data ⟵ data
  │   process(&data);        // [+] click to expand borrow path
  ╰─ 💧 &data released
```

**Option C: Separate panel view**
- Show full call-chain borrow graph in the WebView panel (Milestone 5)
- Click a borrow to see its full cross-function path

## 11.5 Complexity and Limitations

| Scenario | Supported | Notes |
|----------|-----------|-------|
| Direct function call | ✅ Phase 1 | `foo(&data)` |
| Method call on concrete type | ✅ Phase 1 | `data.process()` |
| Trait method (known impl) | ⚠️ Phase 3 | Requires monomorphization |
| Dynamic dispatch (`&dyn`) | ❌ | Cannot resolve at compile time |
| Closures capturing references | ⚠️ Phase 2 | Closure body analysis |
| Async functions | ⚠️ Phase 2 | Future polling complicates lifetimes |
| Recursive calls | ❌ | Cycle detection, show warning |

## 11.6 Tests

- Direct call: `foo(&data)` shows combined span across caller and callee
- Method call: `data.process()` traces into the method
- Multiple arguments: only reference arguments are tracked
- Return value: `let r = get_ref(&data)` traces through the callee
- Nested calls: `foo(bar(&data))` shows two-level chain
- Non-reference arguments are not tracked (owned values are moved, not borrowed)
- Recursive call shows warning, not infinite loop
- Performance: cross-function analysis completes in < 500ms for typical call depth

---

**Priority:** After Milestone 10 (all current milestones complete)
**Estimated effort:** 2-3 weeks
**Dependencies:** Milestone 2 (borrow scope computation), Milestone 3 (LSP endpoints)
