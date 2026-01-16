# Semantic Analysis Expansion Plan

## Goal

Eliminate ALL heuristic and syntactic detection in `borrowscope-macro` by expanding `borrowscope-analyzer` to provide semantic classification for every pattern currently detected via string matching.

**Current State:**
- Analyzer covers 36/109 patterns (33%) - initializer classification only
- Macro uses 66 syntactic patterns for method calls on existing variables
- 7 patterns partially covered

**Target State:**
- Analyzer covers 100% of patterns semantically
- Macro uses ZERO string matching for type/operation detection
- All classification based on rust-analyzer's resolved types

---

## Architecture Change Required

### Current: Initializer-Only Analysis

```
let x = Rc::new(1);  // ✅ Tracked: initializer_kind = "rc_new"
x.clone();           // ❌ Not tracked: method call on existing variable
```

### Target: Full Expression Analysis

```
let x = Rc::new(1);  // ✅ initializer_kind = "rc_new"
let y = x.clone();   // ✅ initializer_kind = "rc_clone" (via result type)
x.some_method();     // ✅ NEW: method_calls[] = [{receiver: "x", method: "some_method", receiver_type: "Rc<i32>"}]
```

### New Output Schema (v2.4)

```json
{
  "version": "2.4",
  "files": {
    "src/main.rs": [
      {
        "name": "x",
        "ty": "Rc<i32>",
        "initializer_kind": "rc_new",
        "method_calls": [
          {
            "method": "clone",
            "line": 5,
            "receiver_type": "Rc<i32>",
            "result_type": "Rc<i32>",
            "operation": "rc_clone"
          }
        ]
      }
    ]
  },
  "expressions": {
    "src/main.rs": [
      {
        "line": 10,
        "kind": "method_call",
        "receiver": "sender",
        "receiver_type": "Sender<i32>",
        "method": "send",
        "operation": "channel_send"
      },
      {
        "line": 15,
        "kind": "function_call",
        "path": "std::mem::transmute",
        "operation": "transmute"
      }
    ]
  }
}
```

---

## Implementation Plan

### Phase 1: Method Call Tracking on Known Variables ✅ COMPLETE

**Status:** Implemented in commit `42b01190b` and `eb879f159`

**Files Modified:**
- `output.rs`: Added `MethodCallInfo` struct, `method_calls` field to `VariableTypeInfo`
- `analysis.rs`: Added `analyze_method_calls()`, `resolve_self_borrow()`, `classify_method_operation()`, `extract_tuple_elements()`

**Schema Version:** 2.4

#### Implemented Functions

##### `analyze_method_calls(sema, db, source_file, variables)`
Iterates over all `MethodCallExpr` nodes in the source file, extracts the receiver variable name, and associates method calls with their corresponding `VariableTypeInfo`.

Key features:
- Handles shadowed variables by matching the most recent declaration before the call (by line number)
- Supports tuple destructuring: `(tx, rx)` elements are indexed separately
- Extracts `mut` bindings correctly: `let mut cow` → variable name `cow`

##### `resolve_self_borrow(sema, method_call, db) -> Option<String>`
Uses `sema.resolve_method_call()` to get the actual function definition, then inspects `self_param.access()` to determine:
- `"immutable"` for `&self`
- `"mutable"` for `&mut self`
- `"consuming"` for `self`

##### `classify_method_operation(receiver_type, method_name) -> Option<String>`
Pattern matches (receiver_type, method_name) pairs to semantic operation names.

##### `extract_tuple_elements(tuple_pat) -> Vec<String>`
Parses tuple pattern strings like `"(tx, rx)"` into individual element names for method call matching.

#### Phase 1 Patterns Implemented (13 patterns)

| Method | Receiver Type | Operation | Self Borrow |
|--------|---------------|-----------|-------------|
| `.set(v)` | `Cell<T>` | `cell_set` | immutable |
| `.get()` | `Cell<T>` | `cell_get` | immutable |
| `.replace()` | `Cell<T>` | `cell_replace` | immutable |
| `.take()` | `Cell<T>` | `cell_take` | immutable |
| `.to_mut()` | `Cow<T>` | `cow_to_mut` | mutable |
| `.into_owned()` | `Cow<T>` | `cow_into_owned` | consuming |
| `.set(v)` | `OnceCell<T>` | `once_cell_set` | immutable |
| `.get()` | `OnceCell<T>` | `once_cell_get` | immutable |
| `.get_or_init(f)` | `OnceCell<T>` | `once_cell_get_or_init` | immutable |
| `.get_or_try_init(f)` | `OnceCell<T>` | `once_cell_get_or_try_init` | immutable |
| `.write(v)` | `MaybeUninit<T>` | `maybe_uninit_write` | mutable |
| `.assume_init()` | `MaybeUninit<T>` | `maybe_uninit_assume_init` | consuming |
| `.assume_init_read()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_read` | immutable |
| `.assume_init_drop()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_drop` | mutable |
| `.assume_init_ref()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_ref` | immutable |
| `.assume_init_mut()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_mut` | mutable |
| `.send(v)` | `Sender<T>` | `channel_send` | immutable |
| `.try_send(v)` | `SyncSender<T>` | `channel_try_send` | immutable |
| `.recv()` | `Receiver<T>` | `channel_recv` | immutable |
| `.try_recv()` | `Receiver<T>` | `channel_try_recv` | immutable |
| `.recv_timeout()` | `Receiver<T>` | `channel_recv_timeout` | immutable |
| `.iter()` | `Receiver<T>` | `channel_iter` | immutable |
| `.join()` | `JoinHandle<T>` | `thread_join` | consuming |
| `.is_finished()` | `JoinHandle<T>` | `thread_is_finished` | immutable |

#### Tests

Integration tests in `borrowscope-analyzer/tests/method_call_tracking.rs`:
- `test_cell_method_tracking`
- `test_cow_method_tracking`
- `test_once_cell_method_tracking`
- `test_channel_method_tracking`
- `test_join_handle_method_tracking`
- `test_self_borrow_detection`

---

### Phase 1.5: Smart Pointer Method Patterns (TODO)

Extend `classify_method_operation()` to cover smart pointer methods not yet tracked.

| Method | Receiver Type | Operation | Priority |
|--------|---------------|-----------|----------|
| `.clone()` | `Rc<T>` | `rc_clone` | High |
| `.downgrade()` | `Rc<T>` | `rc_downgrade` | Medium |
| `.upgrade()` | `Weak<T>` (Rc) | `rc_weak_upgrade` | Medium |
| `.clone()` | `Arc<T>` | `arc_clone` | High |
| `.downgrade()` | `Arc<T>` | `arc_downgrade` | Medium |
| `.upgrade()` | `Weak<T>` (Arc) | `arc_weak_upgrade` | Medium |
| `.borrow()` | `RefCell<T>` | `refcell_borrow` | High |
| `.borrow_mut()` | `RefCell<T>` | `refcell_borrow_mut` | High |
| `.lock()` | `Mutex<T>` | `mutex_lock` | High |
| `.try_lock()` | `Mutex<T>` | `mutex_try_lock` | Medium |
| `.read()` | `RwLock<T>` | `rwlock_read` | High |
| `.write()` | `RwLock<T>` | `rwlock_write` | High |
| `.try_read()` | `RwLock<T>` | `rwlock_try_read` | Medium |
| `.try_write()` | `RwLock<T>` | `rwlock_try_write` | Medium |
| `.into_inner()` | Various | `into_inner` | Low |

---

### Phase 2: Standalone Expression Tracking

Track expressions that aren't variable initializers but need semantic classification.

**File:** `analysis.rs`
**New Function:** `analyze_expressions()`
**New Output Field:** `expressions: Vec<ExpressionInfo>`

#### 2.1 Thread Spawn (Eliminates 2 syntactic patterns)

| Pattern | Operation | Macro Pattern Eliminated |
|---------|-----------|--------------------------|
| `thread::spawn(...)` | `thread_spawn` | `path_str.contains("thread::spawn")` |
| `std::thread::spawn(...)` | `thread_spawn` | `path_str.contains("std::thread::spawn")` |

**Implementation:**
```rust
// Detect function calls where resolved function is std::thread::spawn
if let ast::Expr::CallExpr(call) = expr {
    if let Some(func_type) = sema.type_of_expr(&call.expr()?) {
        // Check if it resolves to thread::spawn
        if is_thread_spawn_fn(&func_type, db) {
            expressions.push(ExpressionInfo {
                kind: "function_call",
                operation: "thread_spawn",
                line: call.syntax().text_range().start().into(),
            });
        }
    }
}
```

#### 2.2 Transmute (Eliminates 2 syntactic patterns)

| Pattern | Operation | Macro Pattern Eliminated |
|---------|-----------|--------------------------|
| `transmute(...)` | `transmute` | `path_str.contains("transmute")` |
| `std::mem::transmute(...)` | `transmute` | `fn_name.contains("transmute")` |

**Implementation:**
```rust
// Check if function call resolves to std::mem::transmute
fn is_transmute_call(sema: &Semantics, call: &ast::CallExpr, db: &RootDatabase) -> bool {
    if let Some(path) = call.expr().and_then(|e| ast::PathExpr::cast(e.syntax().clone())) {
        if let Some(resolved) = sema.resolve_path(&path.path()?) {
            // Check canonical path
            let path_str = get_function_path(&resolved, db);
            return path_str == "core::mem::transmute" || path_str == "std::mem::transmute";
        }
    }
    false
}
```

---

### Phase 3: Self-Borrow Type Inference Elimination

The macro infers whether a method takes `&self`, `&mut self`, or `self` based on method name patterns. This is the largest category of heuristics (47 patterns).

**Solution:** Use rust-analyzer's method resolution to get the actual signature.

**File:** `analysis.rs`
**New Function:** `get_method_self_type()`

#### 3.1 Implementation

```rust
fn get_method_self_type(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> Option<SelfBorrowType> {
    // Resolve the method to its definition
    let func = sema.resolve_method_call(method_call)?;
    
    // Get the self parameter type from the function signature
    let self_param = func.self_param(db)?;
    
    match self_param.access(db) {
        ra_ap_hir::Access::Shared => Some(SelfBorrowType::Immutable),
        ra_ap_hir::Access::Exclusive => Some(SelfBorrowType::Mutable),
        ra_ap_hir::Access::Owned => Some(SelfBorrowType::Consuming),
    }
}
```

#### 3.2 Output Schema Addition

```json
{
  "method_calls": [
    {
      "method": "push",
      "self_borrow": "mutable",  // NEW: semantic, not heuristic
      "line": 10
    }
  ]
}
```

#### 3.3 Patterns Eliminated (47 total)

**Immutable patterns (19):**
- `as_*`, `to_*`, `is_*`, `get*` prefixes
- `len`, `capacity`, `iter`, `chars`, `bytes`, `lines`, `split`, `trim`
- `contains`, `starts_with`, `ends_with`, `find`, `clone`, `first`, `last`

**Mutable patterns (25):**
- `push*`, `pop*`, `insert*`, `remove*`, `append*`, `add*`, `set*`, `update*`, `modify*` prefixes
- `clear`, `truncate`, `extend`, `drain`, `sort`, `reverse`, `dedup`, `retain`
- `tick`, `recv`, `send`, `changed`, `wait`, `acquire`, `lock`, `write`

**Consuming patterns (3):**
- `into_*` prefix
- `unwrap`, `expect`

---

### Phase 4: Unwrap/Expect Tracking

Track unwrap operations semantically by detecting method calls on `Option<T>` and `Result<T, E>`.

#### 4.1 Patterns to Track (Eliminates 5 syntactic patterns)

| Method | Receiver Type | Operation | Macro Pattern Eliminated |
|--------|---------------|-----------|--------------------------|
| `.unwrap()` | `Option<T>` / `Result<T,E>` | `unwrap` | `method_name == "unwrap"` |
| `.expect(msg)` | `Option<T>` / `Result<T,E>` | `expect` | `method_name == "expect"` |
| `.unwrap_or(v)` | `Option<T>` / `Result<T,E>` | `unwrap_or` | `method_name == "unwrap_or"` |
| `.unwrap_or_else(f)` | `Option<T>` / `Result<T,E>` | `unwrap_or_else` | `method_name == "unwrap_or_else"` |
| `.unwrap_or_default()` | `Option<T>` / `Result<T,E>` | `unwrap_or_default` | `method_name == "unwrap_or_default"` |

**Implementation:**
```rust
fn classify_unwrap_method(
    receiver_type: &ra_ap_hir::Type,
    method_name: &str,
    db: &RootDatabase,
) -> Option<String> {
    if !is_option_or_result(receiver_type, db) {
        return None;
    }
    
    match method_name {
        "unwrap" => Some("unwrap"),
        "expect" => Some("expect"),
        "unwrap_or" => Some("unwrap_or"),
        "unwrap_or_else" => Some("unwrap_or_else"),
        "unwrap_or_default" => Some("unwrap_or_default"),
        _ => None,
    }
}
```

---

### Phase 5: Clone Tracking

Track `.clone()` calls semantically by checking if the receiver type implements `Clone`.

#### 5.1 Pattern (Eliminates 1 syntactic pattern)

| Method | Condition | Operation | Macro Pattern Eliminated |
|--------|-----------|-----------|--------------------------|
| `.clone()` | Receiver implements `Clone` | `clone` | `method_name == "clone"` |

**Implementation:**
```rust
fn is_clone_call(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> bool {
    let method_name = method_call.name_ref()?.text().to_string();
    if method_name != "clone" {
        return false;
    }
    
    // Verify it's actually Clone::clone, not some other clone method
    if let Some(func) = sema.resolve_method_call(method_call) {
        let trait_id = func.as_assoc_item(db)?.container_trait(db)?;
        // Check if trait is core::clone::Clone
        let trait_path = get_trait_path(&trait_id, db);
        return trait_path == "core::clone::Clone" || trait_path == "std::clone::Clone";
    }
    false
}
```

---

## Summary: Patterns to Implement

### Phase 1: Method Calls on Known Variables (13 patterns)

| # | Method | Receiver Type | Operation |
|---|--------|---------------|-----------|
| 1 | `.set(v)` | `Cell<T>` | `cell_set` |
| 2 | `.to_mut()` | `Cow<T>` | `cow_to_mut` |
| 3 | `.set(v)` | `OnceCell<T>` | `once_cell_set` |
| 4 | `.get()` | `OnceCell<T>` | `once_cell_get` |
| 5 | `.get_or_init(f)` | `OnceCell<T>` | `once_cell_get_or_init` |
| 6 | `.write(v)` | `MaybeUninit<T>` | `maybe_uninit_write` |
| 7 | `.assume_init()` | `MaybeUninit<T>` | `maybe_uninit_assume_init` |
| 8 | `.assume_init_read()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_read` |
| 9 | `.assume_init_drop()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_drop` |
| 10 | `.send(v)` | `Sender<T>` | `channel_send` |
| 11 | `.recv()` | `Receiver<T>` | `channel_recv` |
| 12 | `.try_recv()` | `Receiver<T>` | `channel_try_recv` |
| 13 | `.join()` | `JoinHandle<T>` | `thread_join` |

### Phase 2: Standalone Expressions (4 patterns)

| # | Pattern | Operation |
|---|---------|-----------|
| 1 | `thread::spawn(...)` | `thread_spawn` |
| 2 | `std::thread::spawn(...)` | `thread_spawn` |
| 3 | `transmute(...)` | `transmute` |
| 4 | `std::mem::transmute(...)` | `transmute` |

### Phase 3: Self-Borrow Inference (47 patterns)

All method self-borrow type inference replaced with semantic resolution via `sema.resolve_method_call()`.

### Phase 4: Unwrap Methods (5 patterns)

| # | Method | Operation |
|---|--------|-----------|
| 1 | `.unwrap()` | `unwrap` |
| 2 | `.expect(msg)` | `expect` |
| 3 | `.unwrap_or(v)` | `unwrap_or` |
| 4 | `.unwrap_or_else(f)` | `unwrap_or_else` |
| 5 | `.unwrap_or_default()` | `unwrap_or_default` |

### Phase 5: Clone (1 pattern)

| # | Method | Operation |
|---|--------|-----------|
| 1 | `.clone()` | `clone` |

---

## New Types to Add to `classify_by_path`

```rust
// Thread
"std::thread::JoinHandle" => "join_handle",

// These are already covered but verify:
// "std::sync::mpsc::Sender" => "channel_sender",
// "std::sync::mpsc::Receiver" => "channel_receiver",
```

---

## New Output Structures

### MethodCallInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCallInfo {
    pub method: String,
    pub line: u32,
    pub operation: String,
    pub self_borrow: Option<String>,  // "immutable", "mutable", "consuming"
    pub receiver_type: String,
}
```

### ExpressionInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionInfo {
    pub kind: String,  // "function_call", "method_call"
    pub line: u32,
    pub operation: String,
    pub details: Option<String>,
}
```

### Updated VariableTypeInfo

```rust
pub struct VariableTypeInfo {
    // ... existing fields ...
    
    /// Method calls on this variable (NEW)
    pub method_calls: Vec<MethodCallInfo>,
}
```

### Updated ProjectTypeInfo

```rust
pub struct ProjectTypeInfo {
    // ... existing fields ...
    
    /// Standalone expressions that need tracking (NEW)
    pub expressions: HashMap<String, Vec<ExpressionInfo>>,
}
```

---

## Macro Changes Required

After analyzer expansion, the macro should:

1. **Load method_calls** from type-info.json for each variable
2. **Load expressions** for standalone tracking needs
3. **Remove all syntactic detection** functions from `smart_pointer.rs`
4. **Remove `infer_self_borrow_type`** - use analyzer's `self_borrow` field
5. **Simplify `transform_visitor.rs`** - just emit tracking calls based on analyzer data

### Before (Syntactic)

```rust
if method_name == "clone" {
    // Heuristic: assume it's Clone::clone
    emit_clone_tracking();
}
```

### After (Semantic)

```rust
if let Some(method_info) = type_info.method_calls.iter().find(|m| m.line == current_line) {
    match method_info.operation.as_str() {
        "clone" => emit_clone_tracking(),
        "channel_send" => emit_channel_send_tracking(),
        // ... etc
    }
}
```

---

## Testing Strategy

### 1. Expand `examples/type-coverage/src/main.rs`

Add test cases for all new patterns:

```rust
// Phase 1: Method calls
fn test_cell_methods() {
    let cell = Cell::new(42);
    cell.set(100);  // Should be tracked as cell_set
}

fn test_channel_methods() {
    let (tx, rx) = mpsc::channel();
    tx.send(42).unwrap();  // Should be tracked as channel_send
    let _ = rx.recv();     // Should be tracked as channel_recv
}

// Phase 2: Standalone expressions
fn test_thread_spawn() {
    let handle = thread::spawn(|| {});  // Should be tracked as thread_spawn
    handle.join().unwrap();              // Should be tracked as thread_join
}

fn test_transmute() {
    let x: i32 = unsafe { std::mem::transmute(42u32) };  // Should be tracked as transmute
}

// Phase 4: Unwrap methods
fn test_unwrap_methods() {
    let opt: Option<i32> = Some(42);
    let _ = opt.unwrap();           // unwrap
    let _ = opt.expect("msg");      // expect
    let _ = opt.unwrap_or(0);       // unwrap_or
}
```

### 2. Verification Script

```bash
# Run analyzer
cargo run -p borrowscope-analyzer -- examples/type-coverage

# Check all operations are detected
jq '.files[][] | select(.method_calls != null) | .method_calls[].operation' \
    examples/type-coverage/.borrowscope/type-info.json | sort | uniq -c

# Check expressions
jq '.expressions[][] | .operation' \
    examples/type-coverage/.borrowscope/type-info.json | sort | uniq -c
```

---

## Timeline Estimate

| Phase | Patterns | Complexity | Estimate |
|-------|----------|------------|----------|
| Phase 1 | 13 | Medium | 2-3 hours |
| Phase 2 | 4 | Medium | 1-2 hours |
| Phase 3 | 47 | High | 3-4 hours |
| Phase 4 | 5 | Low | 1 hour |
| Phase 5 | 1 | Low | 30 min |
| Testing | - | Medium | 2 hours |
| Macro refactor | - | High | 3-4 hours |
| **Total** | **70** | - | **~15 hours** |

---

## Success Criteria

1. ✅ `borrowscope-macro/src/smart_pointer.rs` has ZERO `contains()` or string matching
2. ✅ `transform_visitor.rs` has ZERO `method_name ==` comparisons
3. ✅ `infer_self_borrow_type()` function is deleted
4. ✅ All 109 patterns from SYNTACTIC_PATTERNS.md show ✅ in Analyzer column
5. ✅ `cargo test` passes for both analyzer and macro
6. ✅ Example projects produce identical tracking output
