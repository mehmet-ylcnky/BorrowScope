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

#### Phase 1 Patterns Implemented (24 patterns)

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

### Phase 1.5: Smart Pointer Method Patterns ✅ COMPLETE

**Status:** Implemented in commit `0a17f278e`

| Method | Receiver Type | Operation |
|--------|---------------|-----------|
| `.clone()` | `Rc<T>` | `rc_clone` |
| `.downgrade()` | `Rc<T>` | `rc_downgrade` |
| `.clone()` | `Arc<T>` | `arc_clone` |
| `.downgrade()` | `Arc<T>` | `arc_downgrade` |
| `.upgrade()` | `Weak<T>` | `weak_upgrade` |
| `.clone()` | `Weak<T>` | `weak_clone` |
| `.borrow()` | `RefCell<T>` | `refcell_borrow` |
| `.borrow_mut()` | `RefCell<T>` | `refcell_borrow_mut` |
| `.try_borrow()` | `RefCell<T>` | `refcell_try_borrow` |
| `.try_borrow_mut()` | `RefCell<T>` | `refcell_try_borrow_mut` |
| `.into_inner()` | `RefCell<T>` | `refcell_into_inner` |
| `.replace()` | `RefCell<T>` | `refcell_replace` |
| `.lock()` | `Mutex<T>` | `mutex_lock` |
| `.try_lock()` | `Mutex<T>` | `mutex_try_lock` |
| `.into_inner()` | `Mutex<T>` | `mutex_into_inner` |
| `.read()` | `RwLock<T>` | `rwlock_read` |
| `.write()` | `RwLock<T>` | `rwlock_write` |
| `.try_read()` | `RwLock<T>` | `rwlock_try_read` |
| `.try_write()` | `RwLock<T>` | `rwlock_try_write` |
| `.into_inner()` | `RwLock<T>` | `rwlock_into_inner` |

---

### Phase 2: Standalone Expression Tracking ✅ COMPLETE

**Status:** Implemented in commit `0c9b7de00`

**Files Modified:**
- `output.rs`: Added `ExpressionInfo` struct, `expressions` field to `ProjectTypeInfo`
- `analysis.rs`: Added `analyze_expressions()`, `analyze_call_expr()`, `get_resolved_path()`, `classify_function_call()`

**Schema Version:** 2.5

#### Implemented Functions

##### `analyze_expressions(sema, db, source_file) -> Vec<ExpressionInfo>`
Iterates over all `CallExpr` nodes and classifies standalone function calls.

##### `classify_function_call(path) -> Option<String>`
Maps canonical function paths to operation names:

| Function | Operation |
|----------|-----------|
| `std::thread::spawn` | `thread_spawn` |
| `std::mem::drop` | `drop` |
| `std::mem::forget` | `forget` |
| `std::mem::transmute` | `transmute` |
| `std::mem::transmute_copy` | `transmute_copy` |
| `std::mem::replace` | `mem_replace` |
| `std::mem::swap` | `mem_swap` |
| `std::mem::take` | `mem_take` |
| `std::ptr::read` | `ptr_read` |
| `std::ptr::write` | `ptr_write` |
| `std::ptr::read_volatile` | `ptr_read_volatile` |
| `std::ptr::write_volatile` | `ptr_write_volatile` |
| `std::ptr::copy` | `ptr_copy` |
| `std::ptr::copy_nonoverlapping` | `ptr_copy_nonoverlapping` |

#### Output Schema

```json
{
  "expressions": {
    "src/main.rs": [
      {
        "line": 10,
        "column": 4,
        "kind": "function_call",
        "path": "core::mem::drop",
        "operation": "drop",
        "argument": "x",
        "result_type": "()"
      }
    ]
  }
}
```

---

### Phase 3: Self-Borrow Type Inference ✅ COMPLETE

**Status:** Already implemented as part of Phase 1 via `resolve_self_borrow()`

All method calls now include semantic `self_borrow` detection using `sema.resolve_method_call()` instead of heuristic pattern matching.

---

### Phase 4: Option/Result Methods ✅ COMPLETE

**Status:** Implemented in commit `65abf6489`

#### Option Methods

| Method | Operation |
|--------|-----------|
| `.unwrap()` | `option_unwrap` |
| `.expect()` | `option_expect` |
| `.unwrap_or()` | `option_unwrap_or` |
| `.unwrap_or_else()` | `option_unwrap_or_else` |
| `.unwrap_or_default()` | `option_unwrap_or_default` |
| `.map()` | `option_map` |
| `.and_then()` | `option_and_then` |
| `.ok_or()` | `option_ok_or` |
| `.take()` | `option_take` |
| `.replace()` | `option_replace` |

#### Result Methods

| Method | Operation |
|--------|-----------|
| `.unwrap()` | `result_unwrap` |
| `.expect()` | `result_expect` |
| `.unwrap_or()` | `result_unwrap_or` |
| `.unwrap_or_else()` | `result_unwrap_or_else` |
| `.unwrap_or_default()` | `result_unwrap_or_default` |
| `.unwrap_err()` | `result_unwrap_err` |
| `.expect_err()` | `result_expect_err` |
| `.map()` | `result_map` |
| `.map_err()` | `result_map_err` |
| `.and_then()` | `result_and_then` |
| `.ok()` | `result_ok` |
| `.err()` | `result_err` |

---

### Phase 5: Generic Clone Tracking ✅ COMPLETE

**Status:** Implemented in commit `65abf6489`

Any `.clone()` call not already covered by type-specific patterns (Rc, Arc, Weak) is tracked as operation `clone`.

---

## Summary

All phases complete. Schema version 2.5.

| Phase | Patterns | Status |
|-------|----------|--------|
| Phase 1 | 24 | ✅ Complete |
| Phase 1.5 | 20 | ✅ Complete |
| Phase 2 | 14 | ✅ Complete |
| Phase 3 | 47 (via resolve_self_borrow) | ✅ Complete |
| Phase 4 | 22 | ✅ Complete |
| Phase 5 | 1 | ✅ Complete |
| **Total** | **128** | ✅ Complete |
