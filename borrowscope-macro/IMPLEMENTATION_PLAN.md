# BorrowScope Macro Implementation Plan

## Current State

### Aligned with Runtime
- `track_new`, `track_borrow`, `track_borrow_mut`, `track_move`, `track_drop`
- `track_rc_new`, `track_rc_clone`, `track_arc_new`, `track_arc_clone`
- `track_refcell_new`, `track_refcell_borrow`, `track_refcell_borrow_mut` ✅
- `track_cell_new`, `track_cell_get`, `track_cell_set` ✅
- `track_unsafe_block_enter/exit` ✅
- `track_raw_ptr`, `track_raw_ptr_mut` ✅
- `track_transmute` ✅
- `track_async_block_enter/exit`, `track_await_start/end` ✅

### Implementation Status

| Runtime Feature | Macro Support | Status |
|-----------------|---------------|--------|
| `track_refcell_new` | Transformed | ✅ DONE |
| `track_refcell_borrow` | Transformed | ✅ DONE |
| `track_refcell_borrow_mut` | Transformed | ✅ DONE |
| `track_refcell_drop` | Not implemented | N/A (uses track_drop) |
| `track_cell_new` | Transformed | ✅ DONE |
| `track_cell_get` | Transformed | ✅ DONE |
| `track_cell_set` | Transformed | ✅ DONE |
| `track_raw_ptr` | Transformed | ✅ DONE |
| `track_raw_ptr_mut` | Transformed | ✅ DONE |
| `track_raw_ptr_deref` | Not possible | ❌ Requires type info |
| `track_unsafe_block_enter/exit` | Transformed | ✅ DONE |
| `track_unsafe_fn_call` | Not possible | ❌ Requires type info |
| `track_ffi_call` | Not possible | ❌ Requires type info |
| `track_transmute` | Transformed | ✅ DONE |
| `track_union_field_access` | Not possible | ❌ Requires type info |
| `track_static_init` | Not possible | ❌ Requires type info |
| `track_static_access` | Not possible | ❌ Requires type info |
| `track_const_eval` | Not possible | ❌ Requires type info |
| `track_async_block_enter/exit` | Transformed | ✅ DONE |
| `track_await_start/end` | Transformed | ✅ DONE |

---

## Phase 1: Fix Inconsistencies & Low-Hanging Fruit ✅ COMPLETE

### 1.1 Fix Location Extraction ✅
- Replaced placeholder `extract_location()` with `location_tokens()` 
- Uses `concat!(file!(), ":", line!())` macros evaluated at call site

### 1.2 Fix async/unsafe Inconsistency ✅
- Removed rejection in `best_practices.rs` for async/unsafe functions
- Only const functions are now rejected (cannot have runtime tracking)

### 1.3 Complete RefCell Transformation ✅
| Pattern | Transform To |
|---------|-------------|
| `RefCell::new(x)` | `track_refcell_new("name", RefCell::new(x))` |
| `cell.borrow()` | `track_refcell_borrow("id", "cell_id", "loc", cell.borrow())` |
| `cell.borrow_mut()` | `track_refcell_borrow_mut("id", "cell_id", "loc", cell.borrow_mut())` |

### 1.4 Complete Cell Transformation ✅
| Pattern | Transform To |
|---------|-------------|
| `Cell::new(x)` | `track_cell_new("name", Cell::new(x))` |
| `cell.get()` | `track_cell_get("cell_id", "loc", cell.get())` |
| `cell.set(v)` | `{ track_cell_set("cell_id", "loc"); cell.set(v) }` |

---

## Phase 2: Unsafe Code Tracking ✅ COMPLETE

### 2.1 Unsafe Block Tracking ✅
Transform:
```rust
unsafe { ... }
```
To:
```rust
unsafe {
    borrowscope_runtime::track_unsafe_block_enter(ID, "loc");
    let __unsafe_result = { ... };
    borrowscope_runtime::track_unsafe_block_exit(ID, "loc");
    __unsafe_result
}
```

### 2.2 Raw Pointer Tracking ✅
| Pattern | Transform To |
|---------|-------------|
| `&x as *const T` | `track_raw_ptr("name", id, "*const T", "loc", &x as *const T)` |
| `&mut x as *mut T` | `track_raw_ptr_mut("name", id, "*mut T", "loc", &mut x as *mut T)` |

### 2.3 Transmute Tracking ✅
| Pattern | Transform To |
|---------|-------------|
| `std::mem::transmute(x)` | `{ track_transmute(...); std::mem::transmute(x) }` |

### Known Limitations (Require Type Information)

The following cannot be implemented in a proc macro because they require type
information that is not available during macro expansion:

| Operation | Why It's Not Possible |
|-----------|----------------------|
| `*ptr` dereference tracking | Cannot distinguish raw pointer deref from regular `Deref` trait |
| FFI call tracking | Cannot know if a function is `extern "C"` without seeing its declaration |
| Union field access tracking | Cannot know if a type is `union` vs `struct` |
| Unsafe fn call tracking | Cannot know if a function is `unsafe fn` without seeing its signature |

---

## Phase 3: Static/Const Tracking ❌ NOT IMPLEMENTABLE

### Analysis

Static and const tracking cannot be implemented in `#[trace_borrow]` for two reasons:

1. **Scope mismatch**: `#[trace_borrow]` is a function-level attribute. Static and const declarations are module-level items that exist outside function bodies.

2. **No type information**: When code accesses a static variable (e.g., `SOME_STATIC`), the macro sees only a path expression. It cannot distinguish between:
   - A static variable (`static SOME_STATIC: i32 = 0`)
   - A const item (`const SOME_CONST: i32 = 0`)
   - A local variable (`let some_static = 0`)
   - A function call (`some_static()`)

### Conclusion

Phase 3 is **not implementable** within the current `#[trace_borrow]` design. The runtime API exists but cannot be automatically invoked without type information.

---

## Phase 4: Async Tracking ✅ COMPLETE

### 4.1 Runtime API ✅
Added to borrowscope-runtime:
- `track_async_block_enter(block_id, location)`
- `track_async_block_exit(block_id, location)`
- `track_await_start(await_id, future_name, location)`
- `track_await_end(await_id, location)`

New event types:
- `AsyncBlockEnter { timestamp, block_id, location }`
- `AsyncBlockExit { timestamp, block_id, location }`
- `AwaitStart { timestamp, await_id, future_name, location }`
- `AwaitEnd { timestamp, await_id, location }`

### 4.2 Macro Transformations ✅

**Async blocks:**
```rust
async { expr }
```
Transforms to:
```rust
async {
    track_async_block_enter(ID, "loc");
    let __async_result = { expr };
    track_async_block_exit(ID, "loc");
    __async_result
}
```

**Await expressions:**
```rust
future.await
```
Transforms to:
```rust
{
    track_await_start(ID, "future", "loc");
    let __await_result = future.await;
    track_await_end(ID, "loc");
    __await_result
}
```

### What Cannot Be Tracked
- Future polling (happens in executor, not user code)
- Pin/Unpin semantics (type-dependent)
- Waker interactions (runtime internals)
- State machine transitions (compiler-generated)

---

## Summary

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Complete | Location, async/unsafe consistency, RefCell/Cell |
| Phase 2 | ✅ Complete | Unsafe blocks, raw ptr casts, transmute |
| Phase 3 | ❌ Not Implementable | Requires type info or different macro approach |
| Phase 4 | ✅ Complete | Async blocks and await expressions |

---

## Test Coverage

- 143+ macro unit tests
- 7 async tracking integration tests
- All tests passing
