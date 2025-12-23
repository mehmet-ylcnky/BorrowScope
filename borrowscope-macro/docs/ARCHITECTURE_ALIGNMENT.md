# BorrowScope Architecture: Macro ↔ Runtime Alignment

## Overview

```
┌─────────────────────────────────────────┐     ┌─────────────────────────────────────────┐
│         borrowscope-macro               │     │         borrowscope-runtime             │
│    (Compile-time Instrumentation)       │     │      (Runtime Event Recording)          │
├─────────────────────────────────────────┤     ├─────────────────────────────────────────┤
│                                         │     │                                         │
│  #[trace_borrow]                        │     │  pub fn track_*() functions             │
│       │                                 │     │       ▲                                 │
│       ▼                                 │     │       │                                 │
│  ┌─────────────────────────────────┐    │     │  ┌─────────────────────────────────┐    │
│  │   Syntactic Pattern Matching    │    │     │  │   Global Event Tracker          │    │
│  │   (syn crate - AST only)        │    │     │  │   (Thread-safe storage)         │    │
│  └─────────────────────────────────┘    │     │  └─────────────────────────────────┘    │
│                                         │     │                                         │
└─────────────────────────────────────────┘     └─────────────────────────────────────────┘
                    │                                           ▲
                    │         GENERATES CALLS TO                │
                    └───────────────────────────────────────────┘
```

## Alignment Status by Category

### ✅ Fully Aligned (Macro auto-generates → Runtime receives)

| Pattern | Macro Generates | Runtime Function |
|---------|-----------------|------------------|
| `let x = value` | `track_new_with_id(...)` | `track_new_with_id()` |
| `&x` | `track_borrow_with_id(...)` | `track_borrow_with_id()` |
| `&mut x` | `track_borrow_mut_with_id(...)` | `track_borrow_mut_with_id()` |
| `x = y` (move) | `track_move_with_id(...)` | `track_move_with_id()` |
| `}` (scope end) | `track_drop(...)` | `track_drop()` |

#### Smart Pointers

| Pattern | Macro Generates | Runtime Function |
|---------|-----------------|------------------|
| `Rc::new(v)` | `track_rc_new_with_id(...)` | `track_rc_new_with_id()` |
| `rc.clone()` | `track_rc_clone_with_id(...)` | `track_rc_clone_with_id()` |
| `Arc::new(v)` | `track_arc_new_with_id(...)` | `track_arc_new_with_id()` |
| `arc.clone()` | `track_arc_clone_with_id(...)` | `track_arc_clone_with_id()` |

#### Interior Mutability

| Pattern | Macro Generates | Runtime Function |
|---------|-----------------|------------------|
| `RefCell::new(v)` | `track_refcell_new(...)` | `track_refcell_new()` |
| `refcell.borrow()` | `track_refcell_borrow(...)` | `track_refcell_borrow()` |
| `refcell.borrow_mut()` | `track_refcell_borrow_mut(...)` | `track_refcell_borrow_mut()` |
| `Cell::new/get/set` | `track_cell_*()` | `track_cell_*()` |

#### Unsafe & Raw Pointers

| Pattern | Macro Generates | Runtime Function |
|---------|-----------------|------------------|
| `unsafe { }` | `track_unsafe_block_enter/exit()` | `track_unsafe_block_*()` |
| `*raw_ptr` | `track_raw_ptr_deref(...)` | `track_raw_ptr_deref()` |
| `ptr as *const T` | `track_raw_ptr(...)` | `track_raw_ptr()` |

#### Async & Control Flow

| Pattern | Macro Generates | Runtime Function |
|---------|-----------------|------------------|
| `async { }` | `track_async_block_*()` | `track_async_block_*()` |
| `.await` | `track_await_start/end()` | `track_await_*()` |
| `loop/while/for` | `track_loop_*()` | `track_loop_*()` |
| `match` | `track_match_*()` | `track_match_*()` |
| `return` | `track_return(...)` | `track_return()` |

### ⚠️ Gap: Type-Dependent Patterns

Runtime has functions, but Macro **CANNOT** auto-detect due to the type information barrier:

| Pattern | Macro Status | Runtime Function |
|---------|--------------|------------------|
| `ffi_func(args)` | ❌ Cannot detect extern | `track_ffi_call()` |
| `union.field` | ❌ Cannot detect union type | `track_union_field_access()` |
| `STATIC_VAR.method()` | ❌ Cannot detect static | `track_static_access()` |
| `static X: T = v` | ❌ Cannot detect static init | `track_static_init()` |
| `generic.clone()` | ❌ Cannot resolve T | `track_clone()` |

**Solution: Manual Instrumentation**

```rust
// User must call runtime functions directly:
track_ffi_call("func", "location");
track_union_field_access("UnionName", "field", "location");
track_static_access(id, "VAR_NAME", is_write, "location");
```

## Data Flow

```
    User Code                    Macro Transform                Runtime Execution
    ─────────                    ───────────────                ─────────────────
        │                              │                              │
        ▼                              ▼                              ▼
┌───────────────┐            ┌───────────────────┐           ┌───────────────────┐
│ #[trace_borrow]│            │ fn example() {    │           │ Events recorded:  │
│ fn example() { │  ──────▶  │   track_new_with_ │  ──────▶  │ [New, Borrow,     │
│   let x = 5;   │            │     id("x",...);  │           │  BorrowMut, Drop] │
│   let r = &x;  │            │   let x = 5;      │           │                   │
│   ...          │            │   track_borrow_   │           │ Export to JSON,   │
│ }              │            │     with_id("r"); │           │ DOT, or analyze   │
└───────────────┘            │   let r = &x;     │           └───────────────────┘
                              │   ...             │
                              │   track_drop("x");│
                              │ }                 │
                              └───────────────────┘
```

## Coverage Estimate

```
┌────────────────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Auto-instrumented by macro:  ~95% of typical Rust ownership patterns    │
│   ████████████████████████████████████████████████████████████████░░░░░   │
│                                                                            │
│   Requires manual tracking:    ~5% (FFI, unions, statics, some generics)  │
│   ░░░░░                                                                    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## Summary

- **Fully Aligned**: ~50+ tracking functions where macro auto-generates calls that runtime receives
- **Gap**: 4 patterns (FFI, union, static, generic clone) where runtime has functions but macro cannot auto-detect
- **Root Cause**: Rust's compilation pipeline runs macros before type checking (the "type information barrier")
- **Workaround**: Manual instrumentation using runtime functions directly
