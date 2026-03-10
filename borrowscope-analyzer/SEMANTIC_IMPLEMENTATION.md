# Semantic Category Implementation Specification

> Implementation guide for achieving 100% semantic coverage (109/109 patterns) in borrowscope-analyzer.

## Table of Contents

- [1. Coverage Overview](#1-coverage-overview)
  - [1.1 Summary Table](#11-summary-table)
  - [1.2 Complete Pattern Registry (109 Patterns)](#12-complete-pattern-registry-109-patterns)
- [2. Semantic Implementation — All 109 Patterns ✅](#2-semantic-implementation--all-109-patterns-)
  - [2.1 ADT-Based Initializer Classification (36 patterns)](#21-adt-based-initializer-classification-36-patterns)
  - [2.2 Method Call Semantic Dispatch (66 patterns)](#22-method-call-semantic-dispatch-66-patterns)
    - [2.2.1 semantic_op Dispatch (18 patterns)](#221-semantic_op-dispatch-18-patterns)
    - [2.2.2 Self-Borrow Inference (47 patterns)](#222-self-borrow-inference-47-patterns)
    - [2.2.3 Clone Trait Verification (1 pattern)](#223-clone-trait-verification-1-pattern)
  - [2.3 Standalone Expression Tracking (6 patterns)](#23-standalone-expression-tracking-6-patterns)
- [3. Analyzer-Side Implementation](#3-analyzer-side-implementation)
  - [3.1 Current State Assessment](#31-current-state-assessment)
  - [3.2 Macro ↔ Analyzer Data Flow](#32-macro--analyzer-data-flow)
  - [3.3 Key Functions](#33-key-functions)
  - [3.4 Output Schema](#34-output-schema)
  - [3.5 No New Classification Functions Needed](#35-no-new-classification-functions-needed)
- [4. Macro-Side Refactoring](#4-macro-side-refactoring)
  - [4.1 Files Modified](#41-files-modified)
  - [4.2 Functions Deleted](#42-functions-deleted)
  - [4.3 Lookup Logic](#43-lookup-logic)
- [5. Testing Strategy](#5-testing-strategy)
  - [5.1 Test Cases](#51-test-cases)
  - [5.2 Regression Tests](#52-regression-tests)
  - [5.3 Success Criteria](#53-success-criteria)

---

## 1. Coverage Overview

### 1.1 Summary Table

| Category | Total | ✅ Semantic | ⚠️ Partial | ❌ Syntactic | Notes |
|----------|-------|-------------|------------|--------------|-------|
| Smart pointer creation | 5 | 5 | 0 | 0 | |
| Smart pointer clone | 2 | 2 | 0 | 0 | |
| RefCell/Cell methods | 4 | **4** | 0 | 0 | Cell::set now uses semantic_op |
| Box operations | 3 | 3 | 0 | 0 | |
| Pin operations | 2 | **2** | 0 | 0 | Pin::as_ref/as_mut use semantic self_borrow |
| Cow operations | 3 | **3** | 0 | 0 | Cow::to_mut now uses semantic_op |
| Weak reference operations | 3 | 3 | 0 | 0 | |
| OnceCell/OnceLock operations | 5 | **5** | 0 | 0 | set/get/get_or_init now use semantic_op |
| MaybeUninit operations | 6 | **6** | 0 | 0 | write/assume_init* now use semantic_op |
| Concurrency operations | 12 | **12** | 0 | 0 | send/recv/try_recv/join/try_lock/try_read/try_write now use semantic_op |
| Guard method patterns | 9 | **9** | 0 | 0 | Guard::map uses crate-verified ADT fallback |
| Self borrow inference (immutable) | 19 | **19** | 0 | 0 | semantic_op self_borrow lookup (Step 5+7) |
| Self borrow inference (mutable) | 25 | **25** | 0 | 0 | semantic_op self_borrow lookup (Step 5+7) |
| Self borrow inference (consuming) | 3 | **3** | 0 | 0 | semantic_op self_borrow lookup (Step 5+7) |
| Unwrap methods | 5 | **5** | 0 | 0 | Semantic: verifies Option/Result via semantic_op |
| Clone method | 1 | **1** | 0 | 0 | Uses is_trait_method/trait_name from analyzer |
| Transmute detection | 2 | **2** | 0 | 0 | semantic expression lookup (Step 8) |
| **TOTAL** | **109** | **109** | **0** | **0** |

### 1.2 Complete Pattern Registry (109 Patterns)

**Status legend:**
- ✅ = Fully semantic (ADT identity or FunctionId comparison, zero string matching)
- ⚠️ = Partial (type known semantically, but operation classification still uses heuristics)
- ❌ = Syntactic (macro uses `method_name == "..."` or `path.contains("...")`)

**Phase legend:**
- `—` = Already complete, no work needed
- `P1` = Phase 1: Method Call Tracking
- `P2` = Phase 2: Standalone Expression Tracking
- `P3` = Phase 3: Self-Borrow Type Resolution
- `P4` = Phase 4: Unwrap Method Tracking
- `P5` = Phase 5: Clone Trait Verification

#### Smart Pointer Creation (5 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 1 | `rc_new` | `let x = Rc::new(1)` | ✅ | — | `KnownTypes.rc` ADT + expr_kind `"call"` → `"rc_new"` |
| 2 | `arc_new` | `let x = Arc::new(1)` | ✅ | — | `KnownTypes.arc` ADT + expr_kind `"call"` → `"arc_new"` |
| 3 | `box_new` | `let x = Box::new(1)` | ✅ | — | `KnownTypes.box_` ADT (lang item `OwnedBox`) + `"call"` → `"box_new"` |
| 4 | `weak_new` | `let w = Weak::new()` | ✅ | — | `KnownTypes.weak_rc`/`weak_arc` ADT + `"call"` → `"weak_new"` |
| 5 | `weak_downgrade` | `let w = Rc::downgrade(&x)` | ✅ | — | Weak ADT + expr_kind `"downgrade"` → `"weak_downgrade"` |

#### Smart Pointer Clone (2 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 6 | `rc_clone` | `let y = x.clone()` where `x: Rc<T>` | ✅ | — | Result type resolves to `Rc<T>` ADT + expr_kind `"clone"` → `"rc_clone"` |
| 7 | `arc_clone` | `let y = x.clone()` where `x: Arc<T>` | ✅ | — | Result type resolves to `Arc<T>` ADT + expr_kind `"clone"` → `"arc_clone"` |

#### RefCell/Cell Methods (4 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 8 | `refcell_new` | `let r = RefCell::new(1)` | ✅ | — | `KnownTypes.refcell` ADT + `"call"` → `"refcell_new"` |
| 9 | `refcell_borrow` | `let g = r.borrow()` | ✅ | — | Result type is `Ref<T>` (`KnownTypes.ref_guard`) + expr_kind `"borrow"` → `"refcell_borrow"` |
| 10 | `refcell_borrow_mut` | `let g = r.borrow_mut()` | ✅ | — | Result type is `RefMut<T>` (`KnownTypes.refmut_guard`) + expr_kind `"borrow_mut"` → `"refcell_borrow_mut"` |
| 11 | `cell_set` | `cell.set(42)` | ✅ | — | Macro: `semantic_op` contains `::Cell::set` → `track_cell_set`. Fallback: method name match when not OnceCell. |

#### Box Operations (3 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 12 | `box_new` | `let b = Box::new(1)` | ✅ | — | (same as ID 3) |
| 13 | `box_into_raw` | `let p = Box::into_raw(b)` | ✅ | — | Result type is `*mut T` (raw_ptr) + consuming call → classified by type |
| 14 | `box_from_raw` | `let b = unsafe { Box::from_raw(p) }` | ✅ | — | Result type is `Box<T>` ADT + `"call"` → `"box_new"` |

#### Pin Operations (2 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 15 | `pin_new` | `let p = Pin::new(&mut x)` | ✅ | — | `KnownTypes.pin` ADT (lang item `Pin`) + `"call"` → `"pin_new"` |
| 16 | `pin_as_ref` | `p.as_ref()` / `p.as_mut()` | ✅ | — | Type known (Pin ADT), `semantic_op` + `self_borrow` from analyzer. Covered by self-borrow patterns (IDs 55-73). |

#### Cow Operations (3 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 17 | `cow_new` | `let c = Cow::Owned(s)` | ✅ | — | `KnownTypes.cow` ADT + `"call"` → `"cow_new"` |
| 18 | `cow_variant` | `let c = Cow::Borrowed(&s)` | ✅ | — | Cow ADT + expr_kind `"path"` → `"cow_variant"` |
| 19 | `cow_to_mut` | `c.to_mut()` | ✅ | — | Macro: `semantic_op` contains `Cow` + `to_mut` → `track_cow_to_mut`. Fallback: cow_vars set + method name. |

#### Weak Reference Operations (3 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 20 | `weak_new` | `let w = Weak::new()` | ✅ | — | (same as ID 4) |
| 21 | `weak_downgrade` | `let w = Rc::downgrade(&x)` | ✅ | — | (same as ID 5) |
| 22 | `weak_upgrade` | `let x = w.upgrade()` | ✅ | — | Result type is `Option<Rc<T>>` — Option ADT, but expr_kind `"upgrade"` on Weak ADT → `"weak_upgrade"` |

#### OnceCell/OnceLock Operations (5 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 23 | `once_cell_new` | `let c = OnceCell::new()` | ✅ | — | `KnownTypes.once_cell` ADT + `"call"` → `"once_cell_new"` |
| 24 | `once_lock_new` | `let c = OnceLock::new()` | ✅ | — | `KnownTypes.once_lock` ADT + `"call"` → `"once_lock_new"` |
| 25 | `once_cell_set` | `c.set(value)` | ✅ | — | Macro: `semantic_op` contains `::OnceCell::set` or `::OnceLock::set`. Fallback: once_cell_vars + method name. |
| 26 | `once_cell_get` | `c.get()` | ✅ | — | Macro: `semantic_op` contains `::get`. Fallback: method name. |
| 27 | `once_cell_get_or_init` | `c.get_or_init(\|\| v)` | ✅ | — | Macro: `semantic_op` contains `::get_or_init`. Fallback: method name. |

#### MaybeUninit Operations (6 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 28 | `maybe_uninit_new` | `let m = MaybeUninit::uninit()` | ✅ | — | `KnownTypes.maybe_uninit` ADT (lang item `MaybeUninit`) + `"call"` → `"maybe_uninit_new"` |
| 29 | `maybe_uninit_zeroed` | `let m = MaybeUninit::zeroed()` | ✅ | — | Same ADT + `"call"` → `"maybe_uninit_new"` |
| 30 | `maybe_uninit_write` | `m.write(value)` | ✅ | — | Macro: `semantic_op` contains `::MaybeUninit::write`. Fallback: maybe_uninit_vars + method name. |
| 31 | `maybe_uninit_assume_init` | `unsafe { m.assume_init() }` | ✅ | — | Macro: `semantic_op` contains `::assume_init`. Fallback: method name. |
| 32 | `maybe_uninit_assume_init_read` | `unsafe { m.assume_init_read() }` | ✅ | — | Macro: `semantic_op` contains `::assume_init_read`. Fallback: method name. |
| 33 | `maybe_uninit_assume_init_drop` | `unsafe { m.assume_init_drop() }` | ✅ | — | Macro: `semantic_op` contains `::assume_init_drop`. Fallback: method name. |

#### Concurrency Operations (12 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 34 | `mutex_new` | `let m = Mutex::new(1)` | ✅ | — | `KnownTypes.mutex` ADT + `"call"` → `"mutex_new"` |
| 35 | `rwlock_new` | `let r = RwLock::new(1)` | ✅ | — | `KnownTypes.rwlock` ADT + `"call"` → `"rwlock_new"` |
| 36 | `channel_new` | `let (tx, rx) = mpsc::channel()` | ✅ | — | Result type contains `Sender`/`Receiver` ADT → `"channel_new"` |
| 37 | `mutex_lock` | `let g = m.lock().unwrap()` | ✅ | — | Result type is `MutexGuard<T>` ADT + expr_kind `"lock"` → `"mutex_lock"` |
| 38 | `rwlock_read` | `let g = r.read().unwrap()` | ✅ | — | Result type is `RwLockReadGuard<T>` ADT + `"read"` → `"rwlock_read"` |
| 39 | `channel_send` | `tx.send(value)` | ✅ | — | Macro: `semantic_op` contains `Sender` + `send`. Fallback: sender_vars + method name. |
| 40 | `channel_recv` | `rx.recv()` | ✅ | — | Macro: `semantic_op` contains `Receiver` + `recv`. Fallback: receiver_vars + method name. |
| 41 | `channel_try_recv` | `rx.try_recv()` | ✅ | — | Macro: `semantic_op` contains `Receiver` + `try_recv`. Fallback: receiver_vars + method name. |
| 42 | `thread_join` | `handle.join()` | ✅ | — | Macro: `semantic_op` contains `JoinHandle` + `join`. Fallback: join_handle_vars + method name. |
| 43 | `mutex_try_lock` | `m.try_lock()` | ✅ | — | Macro: `semantic_op` disambiguates, `transform_lock` handles `try_lock` → "mutex". |
| 44 | `rwlock_try_read` | `r.try_read()` | ✅ | — | Macro: `semantic_op` disambiguates, `transform_lock` handles `try_read` → "rwlock_read". |
| 45 | `rwlock_try_write` | `r.try_write()` | ✅ | — | Macro: `semantic_op` disambiguates, `transform_lock` handles `try_write` → "rwlock_write". |

#### Guard Method Patterns (9 patterns)

| ID | Pattern | Example | Status | Phase | How It Works Today |
|----|---------|---------|--------|-------|--------------------|
| 46 | `mutex_guard` | `let g = m.lock().unwrap()` | ✅ | — | `KnownTypes.mutex_guard` ADT via initializer |
| 47 | `rwlock_read_guard` | `let g = r.read().unwrap()` | ✅ | — | `KnownTypes.rwlock_read_guard` ADT |
| 48 | `rwlock_write_guard` | `let g = r.write().unwrap()` | ✅ | — | `KnownTypes.rwlock_write_guard` ADT |
| 49 | `ref_guard` | `let g = r.borrow()` | ✅ | — | `KnownTypes.ref_guard` ADT |
| 50 | `refmut_guard` | `let g = r.borrow_mut()` | ✅ | — | `KnownTypes.refmut_guard` ADT |
| 51 | `mutex_lock` | `m.lock()` | ✅ | — | (same as ID 37) |
| 52 | `rwlock_read` | `r.read()` | ✅ | — | (same as ID 38) |
| 53 | `rwlock_write` | `r.write()` | ✅ | — | Result type is `RwLockWriteGuard<T>` ADT + `"write"` → `"rwlock_write"` |
| 54 | `guard_map` | `MutexGuard::map(g, \|d\| &d.field)` | ✅ | — | Mapped guard ADTs classified via crate-verified fallback; macro dispatches `mutex_guard_mapped`/`rwlock_*_guard_mapped` initializer kinds. |

#### Self Borrow Inference — Immutable (19 patterns)

| ID | Pattern | Current Detection | Status | Phase |
|----|---------|-------------------|--------|-------|
| 55 | `as_*` | `semantic_op` self_borrow lookup, fallback: `starts_with("as_")` | ✅ | — |
| 56 | `to_*` | `semantic_op` self_borrow lookup, fallback: `starts_with("to_")` | ✅ | — |
| 57 | `is_*` | `semantic_op` self_borrow lookup, fallback: `starts_with("is_")` | ✅ | — |
| 58 | `get*` | `semantic_op` self_borrow lookup, fallback: `starts_with("get")` | ✅ | — |
| 59 | `len` | `semantic_op` self_borrow lookup, fallback: `== "len"` | ✅ | — |
| 60 | `capacity` | `semantic_op` self_borrow lookup, fallback: `== "capacity"` | ✅ | — |
| 61 | `iter` | `semantic_op` self_borrow lookup, fallback: `== "iter"` | ✅ | — |
| 62 | `chars` | `semantic_op` self_borrow lookup, fallback: `== "chars"` | ✅ | — |
| 63 | `bytes` | `semantic_op` self_borrow lookup, fallback: `== "bytes"` | ✅ | — |
| 64 | `lines` | `semantic_op` self_borrow lookup, fallback: `== "lines"` | ✅ | — |
| 65 | `split` | `semantic_op` self_borrow lookup, fallback: `== "split"` | ✅ | — |
| 66 | `trim` | `semantic_op` self_borrow lookup, fallback: `== "trim"` | ✅ | — |
| 67 | `contains` | `semantic_op` self_borrow lookup, fallback: `== "contains"` | ✅ | — |
| 68 | `starts_with` | `semantic_op` self_borrow lookup, fallback: `== "starts_with"` | ✅ | — |
| 69 | `ends_with` | `semantic_op` self_borrow lookup, fallback: `== "ends_with"` | ✅ | — |
| 70 | `find` | `semantic_op` self_borrow lookup, fallback: `== "find"` | ✅ | — |
| 71 | `clone` | `semantic_op` self_borrow lookup, fallback: `== "clone"` | ✅ | — |
| 72 | `first` | `semantic_op` self_borrow lookup, fallback: `== "first"` | ✅ | — |
| 73 | `last` | `semantic_op` self_borrow lookup, fallback: `== "last"` | ✅ | — |

#### Self Borrow Inference — Mutable (25 patterns)

| ID | Pattern | Current Detection | Status | Phase |
|----|---------|-------------------|--------|-------|
| 74 | `push*` | `semantic_op` self_borrow lookup, fallback: `starts_with("push")` | ✅ | — |
| 75 | `pop*` | `semantic_op` self_borrow lookup, fallback: `starts_with("pop")` | ✅ | — |
| 76 | `insert*` | `semantic_op` self_borrow lookup, fallback: `starts_with("insert")` | ✅ | — |
| 77 | `remove*` | `semantic_op` self_borrow lookup, fallback: `starts_with("remove")` | ✅ | — |
| 78 | `append*` | `semantic_op` self_borrow lookup, fallback: `starts_with("append")` | ✅ | — |
| 79 | `add*` | `semantic_op` self_borrow lookup, fallback: `starts_with("add")` | ✅ | — |
| 80 | `set*` | `semantic_op` self_borrow lookup, fallback: `starts_with("set")` | ✅ | — |
| 81 | `update*` | `semantic_op` self_borrow lookup, fallback: `starts_with("update")` | ✅ | — |
| 82 | `modify*` | `semantic_op` self_borrow lookup, fallback: `starts_with("modify")` | ✅ | — |
| 83 | `clear` | `semantic_op` self_borrow lookup, fallback: `== "clear"` | ✅ | — |
| 84 | `truncate` | `semantic_op` self_borrow lookup, fallback: `== "truncate"` | ✅ | — |
| 85 | `extend` | `semantic_op` self_borrow lookup, fallback: `== "extend"` | ✅ | — |
| 86 | `drain` | `semantic_op` self_borrow lookup, fallback: `== "drain"` | ✅ | — |
| 87 | `sort` | `semantic_op` self_borrow lookup, fallback: `== "sort"` | ✅ | — |
| 88 | `reverse` | `semantic_op` self_borrow lookup, fallback: `== "reverse"` | ✅ | — |
| 89 | `dedup` | `semantic_op` self_borrow lookup, fallback: `== "dedup"` | ✅ | — |
| 90 | `retain` | `semantic_op` self_borrow lookup, fallback: `== "retain"` | ✅ | — |
| 91 | `tick` | `semantic_op` self_borrow lookup, fallback: `== "tick"` | ✅ | — |
| 92 | `recv` | `semantic_op` self_borrow lookup, fallback: `== "recv"` | ✅ | — |
| 93 | `send` | `semantic_op` self_borrow lookup, fallback: `== "send"` | ✅ | — |
| 94 | `changed` | `semantic_op` self_borrow lookup, fallback: `== "changed"` | ✅ | — |
| 95 | `wait` | `semantic_op` self_borrow lookup, fallback: `== "wait"` | ✅ | — |
| 96 | `acquire` | `semantic_op` self_borrow lookup, fallback: `== "acquire"` | ✅ | — |
| 97 | `lock` | `semantic_op` self_borrow lookup, fallback: `== "lock"` | ✅ | — |
| 98 | `write` | `semantic_op` self_borrow lookup, fallback: `== "write"` | ✅ | — |

#### Self Borrow Inference — Consuming (3 patterns)

| ID | Pattern | Current Detection | Status | Phase |
|----|---------|-------------------|--------|-------|
| 99 | `into_*` | `semantic_op` self_borrow lookup, fallback: `starts_with("into_")` | ✅ | — |
| 100 | `unwrap` (consuming) | `semantic_op` self_borrow lookup, fallback: `== "unwrap"` | ✅ | — |
| 101 | `expect` (consuming) | `semantic_op` self_borrow lookup, fallback: `== "expect"` | ✅ | — |

#### Unwrap Methods (5 patterns)

| ID | Pattern | Example | Current Detection | Status | Phase |
|----|---------|---------|-------------------|--------|-------|
| 102 | `unwrap` | `opt.unwrap()` | ✅ | — | Semantic: verifies `core::option::unwrap` or `core::result::unwrap` via `semantic_op`. |
| 103 | `expect` | `opt.expect("msg")` | ✅ | — | Semantic: verifies via `semantic_op`. |
| 104 | `unwrap_or` | `opt.unwrap_or(default)` | ✅ | — | Semantic: verifies via `semantic_op`. |
| 105 | `unwrap_or_else` | `opt.unwrap_or_else(\|\| v)` | ✅ | — | Semantic: verifies via `semantic_op`. |
| 106 | `unwrap_or_default` | `opt.unwrap_or_default()` | ✅ | — | Semantic: verifies via `semantic_op`. |

#### Clone Method (1 pattern)

| ID | Pattern | Example | Current Detection | Status | Phase |
|----|---------|---------|-------------------|--------|-------|
| 107 | `clone_trait` | `x.clone()` | ✅ | — | Uses `is_trait_method`/`trait_name` from analyzer to verify `Clone::clone` vs inherent `.clone()`. |

> Note: `.clone()` is partial because the macro detects the method name but cannot verify it resolves to `Clone::clone` trait impl vs an inherent method named `clone`.

#### Transmute Detection (2 patterns)

| ID | Pattern | Example | Current Detection | Status | Phase |
|----|---------|---------|-------------------|--------|-------|
| 108 | `transmute` | `transmute(x)` | `semantic expression lookup` + `find_transmute_types()` | ✅ | — |
| 109 | `std_transmute` | `std::mem::transmute(x)` | `semantic expression lookup` + `find_transmute_types()` | ✅ | — |

> Note: Both resolve to the same `FunctionId` semantically. Transmute detection now uses `ExpressionInfo` from analyzer with `argument`/`result_type` extraction. Returns `None` when multiple transmutes exist in the same function (can't disambiguate without span line access).

---

## 2. Semantic Implementation — All 109 Patterns ✅

All 109 patterns are now fully semantic. This section documents the three mechanisms used.

### 2.1 ADT-Based Initializer Classification (36 patterns)

Implemented in `classify_by_resolved_type_semantic()` (analysis.rs). Uses ADT identity comparison via `KnownTypes.classify(&adt)` — zero string matching. Matches `(type_class, expr_kind)` tuples.

For unstable std types not found via `import_map` (e.g., `MappedMutexGuard`), a crate-verified fallback checks `adt.name()` + confirms the crate is `std`/`core`/`alloc`.

| Category | Count | Pattern IDs | Mechanism |
|----------|-------|-------------|-----------|
| Smart pointer creation | 5 | 1–5 | `("rc"/"arc"/"box"/"weak", "call"/"downgrade")` |
| Smart pointer clone | 2 | 6–7 | `("rc"/"arc", "clone")` |
| Box operations | 3 | 12–14 | `("box", "call")` + raw ptr result type |
| Weak reference ops | 3 | 20–22 | `("weak", "call"/"downgrade"/"upgrade")` |
| RefCell/Cell creation & guards | 4 | 8–10, 11* | `("refcell"/"cell"/"ref_guard"/"refmut_guard", ...)` |
| Pin creation | 1 | 15 | `("pin", "call")` via lang item |
| Cow creation | 2 | 17–18 | `("cow", "call"/"path")` |
| OnceCell/OnceLock creation | 2 | 23–24 | `("once_cell"/"once_lock", "call")` |
| MaybeUninit creation | 2 | 28–29 | `("maybe_uninit", "call")` |
| Concurrency creation | 5 | 34–38 | `("mutex"/"rwlock"/"channel_*"/"mutex_guard"/"rwlock_read_guard", ...)` |
| Guard creation | 8 | 46–53 | Guard ADT match arms |
| Guard::map (mapped guards) | 1 | 54 | `("mapped_mutex_guard"/"mapped_rwlock_*", _)` via crate-verified fallback |

*Cell::set (ID 11) also uses `semantic_op` dispatch — see §2.2.

### 2.2 Method Call Semantic Dispatch (66 patterns)

The macro reads `method_calls[]` from `type-info.json` for each variable. The analyzer populates `operation`, `self_borrow`, `is_trait_method`, and `trait_name` via `analyze_method_calls()`.

#### 2.2.1 `semantic_op` Dispatch (18 patterns)

All 18 method dispatch points in `visit_expr_mut()` use `semantic_op` lookup. The macro reads `method_calls[].operation` and matches against canonical paths.

| Category | Count | Pattern IDs | Example `operation` |
|----------|-------|-------------|---------------------|
| Cell::set | 1 | 11 | `core::cell::Cell::set` |
| Cow::to_mut | 1 | 19 | `alloc::borrow::Cow::to_mut` |
| OnceCell/OnceLock methods | 3 | 25–27 | `core::cell::OnceCell::set/get/get_or_init` |
| MaybeUninit methods | 4 | 30–33 | `core::mem::MaybeUninit::write/assume_init/...` |
| Channel operations | 3 | 39–41 | `std::sync::mpsc::Sender::send` etc. |
| JoinHandle::join | 1 | 42 | `std::thread::JoinHandle::join` |
| Concurrency try_* | 3 | 43–45 | `std::sync::Mutex::try_lock` etc. |
| Unwrap methods | 5 | 102–106 | `core::option::Option::unwrap` / `core::result::Result::unwrap` etc. |

Unwrap methods (IDs 102–106) verify `operation` contains `"option"` or `"result"` before dispatching.

Falls back to name-based matching only when no analyzer data is available.

#### 2.2.2 Self-Borrow Inference (47 patterns)

`infer_self_borrow_type()` does semantic lookup first via `method_calls[].self_borrow`, heuristic fallback only when no analyzer data exists.

| Category | Count | Pattern IDs | `self_borrow` value |
|----------|-------|-------------|---------------------|
| Immutable (`&self`) | 19 | 55–73 | `"immutable"` |
| Mutable (`&mut self`) | 25 | 74–98 | `"mutable"` |
| Consuming (`self`) | 3 | 99–101 | `"consuming"` |

Covers `as_*`, `get*`, `set*`, `push*`, `insert*`, `remove*`, `into_*`, `send`, `recv`, `lock`, `read`, `write`, `clone`, `unwrap`, `expect`, and all other method name patterns.

#### 2.2.3 Clone Trait Verification (1 pattern)

| Pattern ID | Mechanism |
|------------|-----------|
| 107 | Macro reads `is_trait_method` + `trait_name` from `method_calls[]`. Only emits `track_clone` if confirmed `Clone::clone` (or no analyzer data). Inherent `.clone()` methods fall through to `transform_method_call`. |

Analyzer fix: `resolve_trait_info()` now checks `i.trait_(db)` on `ItemContainer::Impl` blocks to detect trait impls (e.g., `impl Clone for Rc<T>`).

### 2.3 Standalone Expression Tracking (6 patterns)

#### 2.3.1 Pin::as_ref / Pin::as_mut (1 pattern, ID 16)

Covered by self-borrow inference (§2.2.2). Pin::as_ref gets `self_borrow: "immutable"`, Pin::as_mut gets `self_borrow: "mutable"` from analyzer.

#### 2.3.2 Transmute Detection (2 patterns, IDs 108–109)

Macro reads `expressions[]` from `type-info.json`. Matches `kind: "function_call"` + `path: "core::mem::transmute"`. Analyzer resolves via `TrackedFunctions` (FunctionId comparison).

#### 2.3.3 thread::spawn (covered by ExpressionInfo)

Macro reads `expressions[]`, matches `path: "std::thread::spawn"`. Analyzer resolves via `TrackedFunctions`.


## 3. Analyzer-Side Implementation

### 3.1 Current State — Complete

All analyzer-side work is done. The analyzer emits all data needed for 109/109 semantic patterns.

**What already works (analysis.rs on `feature/analyzer-method-call-tracking`):**

| Function | Line | What It Does | Covers |
|----------|------|-------------|--------|
| `analyze_method_calls()` | 1943 | Iterates all `MethodCallExpr` nodes, resolves receiver type, method path, self_borrow, trait info, unsafe flag. Populates `MethodCallInfo` on each variable. | P1 (13), P3 (47), P4 (5), P5 (1) |
| `resolve_self_borrow()` | 2982 | `func.self_param(db).access(db)` → `Shared/Exclusive/Owned` | All 47 self-borrow patterns |
| `resolve_method_path()` | 3000 | Builds canonical path `crate::module::Type::method` | All method-based patterns |
| `resolve_trait_info()` | 2041 | `func.container(db)` → `Trait(t)` or `Impl(_)` | Clone verification (P5) |
| `analyze_expressions()` | 2069 | Iterates `CallExpr` nodes, resolves via `TrackedFunctions` (FunctionId) | P2 (4 patterns: spawn, transmute) |
| `extract_closure_captures_semantic()` | 2170 | `closure_hir.captured_items(db)` with `CaptureKind` | thread::spawn closure captures |

All gaps have been resolved:
- `method_calls` serialized with `operation`, `self_borrow`, `is_trait_method`, `trait_name`
- `expressions` serialized for standalone calls (transmute, thread::spawn)
- Mapped guard ADTs in `KnownTypes` + crate-verified fallback for unstable types
- `resolve_trait_info()` detects trait impls via `i.trait_(db)` on `Impl` blocks

### 3.2 Macro ↔ Analyzer Data Flow

The macro reads `type-info.json` via `type_info.rs`. Today it only consumes **variable-level** fields (creation/initialization). It completely ignores **operation-level** fields (what happens to variables after creation), and also ignores many variable-level fields the analyzer provides.

#### A. Per-Variable Fields: Macro Reads vs Ignores

The analyzer's `VariableTypeInfo` has **78 fields**. The macro's `VariableTypeInfo` deserializes **47 fields**. That leaves **31 fields** the analyzer writes but the macro never reads.

**Fields the macro READS (47) — driving the 36 semantic patterns:**

| Category | Fields | Used For |
|----------|--------|----------|
| Identity | `name`, `ty` | Variable lookup, type display |
| Initializer | `initializer_kind` | Dispatch to `track_*` call (the core semantic path) |
| Trait flags | `is_copy`, `is_clone`, `is_send`, `is_sync`, `is_drop`, `is_sized`, `is_future`, `is_iterator` | Move vs copy semantics, async detection |
| Type structure | `is_primitive`, `is_reference`, `is_mutable_reference`, `is_raw_ptr`, `is_slice`, `is_str`, `is_closure`, `is_fn_ptr`, `is_dyn_trait`, `is_union` | Skip tracking for refs, detect closures |
| ADT classification | `is_rc`, `is_arc`, `is_box`, `is_weak`, `is_refcell`, `is_cell`, `is_mutex`, `is_rwlock`, `is_guard`, `is_vec`, `is_string`, `is_option`, `is_result`, `is_pin`, `is_cow`, `is_once_cell`, `is_maybe_uninit`, `is_channel`, `is_extern_type` | Type-specific tracking dispatch |
| Declaration | `is_static`, `is_const` | Skip tracking for statics/consts |
| Binding | `is_tuple_binding`, `is_mut_binding`, `is_impl_trait` | Destructuring, mutability |
| Disambiguation | `function_name`, `decl_index` | Correct lookup for shadowed names |

**Fields the analyzer writes but macro does not yet consume (potential future use):**

| Field | What It Contains | Potential Use |
|-------|-----------------|---------------|
| `usages` | `Vec<VariableUsageInfo>` — every use site with line/column/kind | Could track variable usage flow |
| `closure_captures` | `Vec<ClosureCaptureInfo>` — capture kinds (shared_ref, move, etc.) | Could emit precise closure capture tracking |
| `line`, `column` | Declaration location | Could enable line-based lookup (not available on stable proc_macro) |
| `file` | Source file path | Multi-file disambiguation |
| `span_start`, `span_end` | Byte offsets | Precise span matching |
| `drop_line`, `drop_column` | Where the variable is dropped | Could emit `track_drop` at exact location |
| `scope_id` | Scope nesting identifier | Could track scope entry/exit |
| `fields` | `Vec<FieldInfo>` — struct field names and types | Could track field access patterns |
| `adjustments` | `Vec<AdjustmentInfo>` — autoref/autoderef chain | Could detect implicit borrows |
| `type_arguments` | Generic type parameters | Could display `Rc<String>` instead of `Rc<T>` |
| `layout` | `LayoutInfo { size, align }` | Could report memory layout |
| `deref_chain` | Autoderef sequence | Could track implicit deref borrows |
| `lifetime` | Named lifetime if present | Could annotate borrow lifetimes |
| `binding_mode` | `ref`, `ref mut`, `move` | Could distinguish binding patterns |
| `contains_reference` | Whether type contains references | Could warn about reference-containing moves |
| `reference_mutability` | `"shared"` or `"mutable"` | More precise than `is_mutable_reference` |
| `is_ref_binding` | `ref` or `ref mut` pattern | Could track ref bindings |
| `pattern_adjustments` | Pattern match adjustments | Could track match ergonomics |
| `future_output_type` | Output type of Future | Could annotate async tracking |
| `iterator_item_type` | Item type of Iterator | Could annotate iterator tracking |
| `is_atomic` | AtomicBool, AtomicUsize, etc. | Could track atomic operations |
| `is_join_handle` | `JoinHandle<T>` | Could track thread joins (currently uses var name set) |
| `is_duration`, `is_instant` | Time types | Could track timing operations |
| `is_callable` | Fn/FnMut/FnOnce impl | Could track callable invocations |

#### B. Per-Variable Nested Data: method_calls[] (the big gap)

Each `MethodCallInfo` in `method_calls[]` contains:

| Field | Type | What It Provides | Macro Heuristic It Replaces |
|-------|------|------------------|-----------------------------|
| `method` | `String` | Method name (e.g., `"push"`) | Already known from AST |
| `line` | `u32` | Call location | Enables precise matching |
| `column` | `u32` | Call location | Enables precise matching |
| `operation` | `Option<String>` | Canonical path: `"alloc::vec::Vec::push"` | `detect_*()` functions (16 functions, ~250 lines) |
| `self_borrow` | `Option<String>` | `"immutable"` / `"mutable"` / `"consuming"` | `infer_self_borrow_type()` (47 patterns) |
| `receiver_type` | `String` | `"Vec<i32>"` — fully qualified | Not available to macro at all |
| `result_type` | `Option<String>` | `"Option<i32>"` | Not available to macro at all |
| `is_trait_method` | `Option<bool>` | `true` for trait impls | `method_name == "clone"` can't distinguish |
| `trait_name` | `Option<String>` | `"Clone"`, `"Iterator"`, etc. | Not available to macro at all |
| `is_unsafe` | `Option<bool>` | `true` for unsafe calls | Not available to macro at all |

#### C. Top-Level Project Data: Macro Ignores Everything

The analyzer's `ProjectTypeInfo` has **18 top-level maps**. The macro reads only **4** (`version`, `files`, `by_name`, `by_function`). These **14 maps** are completely ignored:

| Top-Level Field | What It Contains | Potential Use |
|-----------------|-----------------|---------------|
| `expressions` | `HashMap<file, Vec<ExpressionInfo>>` — spawn, transmute, drop, etc. | **CRITICAL**: Replaces `path_str.contains("transmute")` etc. (4 patterns) |
| `await_points` | `HashMap<file, Vec<AwaitPointInfo>>` — every `.await` with live variables | Could track async ownership across await boundaries |
| `borrow_spans` | `HashMap<file, Vec<BorrowSpanInfo>>` — borrow start/end/use sites | Could emit precise borrow lifetime tracking |
| `unsafe_operations` | `HashMap<file, Vec<UnsafeOperationInfo>>` — unsafe blocks/calls | Could emit `track_unsafe_block_enter/exit` semantically |
| `closure_traits` | `HashMap<file, Vec<ClosureTraitInfo>>` — Fn/FnMut/FnOnce + captures | Could track closure borrow semantics |
| `field_accesses` | `HashMap<file, Vec<FieldAccessInfo>>` — field reads/writes | Could emit `track_field_access` semantically |
| `destructuring` | `HashMap<file, Vec<DestructuringInfo>>` — pattern destructuring | Could track ownership splits |
| `match_bindings` | `HashMap<file, Vec<MatchBindingInfo>>` — match arm bindings | Could track match ownership patterns |
| `variants` | `HashMap<file, Vec<VariantInfo>>` — enum variant usage | Could track enum variant construction |
| `lifetimes` | `HashMap<file, Vec<LifetimeInfo>>` — named lifetimes | Could annotate borrow tracking |
| `labels` | `HashMap<file, Vec<LabelInfo>>` — loop labels | Could track labeled break/continue |
| `const_patterns` | `HashMap<file, Vec<ConstPatternInfo>>` — const in patterns | Could track const pattern matching |
| `callables` | `HashMap<file, Vec<CallableInfo>>` — callable expressions | Could track function pointer calls |
| `record_field_exprs` | `HashMap<file, Vec<RecordFieldExprInfo>>` — struct literal fields | Could track struct construction |
| `record_field_pats` | `HashMap<file, Vec<RecordFieldPatInfo>>` — struct pattern fields | Could track struct destructuring |

#### D. Summary: What Matters for 109/109

To achieve 109/109 semantic patterns, only **2 fields** need to be consumed:

| Priority | Field | Patterns Fixed |
|----------|-------|---------------|
| **P0** | `method_calls[]` (per-variable) | 69 patterns (P1: 13, P3: 47, P4: 5, P5: 1, partial: 3) |
| **P0** | `expressions` (top-level) | 4 patterns (P2: spawn + transmute) |

The other 29 ignored variable fields and 12 ignored top-level maps are **nice-to-have** for richer tracking but not required for the 109/109 goal.

#### E. Complete Macro Method Audit

Every method in the macro classified by whether the analyzer can replace its heuristic logic.

**smart_pointer.rs — Detection Functions:**

| # | Method | Line | Detection Mechanism | Analyzer Equivalent | Can Replace? |
|---|--------|------|--------------------|--------------------|-------------|
| 1 | `detect_smart_pointer_new()` | 156 | Path string: `"Rc::new"`, `"Arc::new"`, etc. | `initializer_kind` field | ✅ Already replaced when analyzer data present |
| 2 | `detect_rc_clone()` | 189 | Path string: `"Rc::clone"`, `"Arc::clone"` | `initializer_kind: "rc_clone"/"arc_clone"` | ✅ Already replaced |
| 3 | `detect_refcell_borrow()` | 207 | Method name: `"borrow"`, `"borrow_mut"` | `initializer_kind: "refcell_borrow"/"refcell_borrow_mut"` | ✅ Already replaced |
| 4 | `detect_cell_operation()` | 222 | Method name: `"get"`, `"set"` | `method_calls[].operation: "core::cell::Cell::set"` | ✅ Yes — read `method_calls[]` |
| 5 | `detect_box_pin()` | 237 | Path string: `"Box::pin"` | `initializer_kind: "pin_new"` | ✅ Already replaced |
| 6 | `detect_box_raw_op()` | 248 | Path string: `"Box::into_raw"`, `"Box::from_raw"` | `initializer_kind: "box_new"` + result type | ✅ Already replaced |
| 7 | `detect_pin_operation()` | 264 | Method name: `"as_ref"`, `"as_mut"` on Pin | `method_calls[].operation: "core::pin::Pin::as_ref"` | ✅ Yes — read `method_calls[]` |
| 8 | `detect_cow_creation()` | 280 | Path string: `"Cow::Borrowed"`, `"Cow::Owned"` | `initializer_kind: "cow_new"/"cow_variant"` | ✅ Already replaced |
| 9 | `detect_cow_to_mut()` | 296 | Method name: `"to_mut"` | `method_calls[].operation: "alloc::borrow::Cow::to_mut"` | ✅ Yes — read `method_calls[]` |
| 10 | `detect_downgrade()` | 304 | Path string: `"Rc::downgrade"`, `"Arc::downgrade"` | `initializer_kind: "weak_downgrade"` | ✅ Already replaced |
| 11 | `detect_weak_upgrade()` | 320 | Method name: `"upgrade"` | `method_calls[].operation: "alloc::rc::Weak::upgrade"` | ✅ Yes — read `method_calls[]` |
| 12 | `detect_once_cell_new()` | 328 | Path string: `"OnceCell::new"`, `"OnceLock::new"` | `initializer_kind: "once_cell_new"/"once_lock_new"` | ✅ Already replaced |
| 13 | `detect_once_cell_method()` | 344 | Method name: `"set"`, `"get"`, `"get_or_init"` | `method_calls[].operation: "core::cell::OnceCell::get"` | ✅ Yes — read `method_calls[]` |
| 14 | `detect_maybe_uninit_new()` | 358 | Path string: `"MaybeUninit::uninit"`, `"MaybeUninit::zeroed"` | `initializer_kind: "maybe_uninit_new"` | ✅ Already replaced |
| 15 | `detect_maybe_uninit_method()` | 374 | Method name: `"write"`, `"assume_init"`, etc. | `method_calls[].operation: "core::mem::MaybeUninit::write"` | ✅ Yes — read `method_calls[]` |
| 16 | `detect_concurrency_op()` | 389 | Path/method string: `"thread::spawn"`, `"channel"`, `"send"`, `"recv"` | `expressions[].path` + `method_calls[].operation` | ✅ Yes — read both |
| 17 | `is_smart_pointer_operation()` | 425 | Aggregates detect_* calls | All of the above | ✅ Yes — replaced by lookup |

**transform_visitor.rs — Core Transform Methods:**

| # | Method | Line | What It Does | Heuristic? | Analyzer Can Replace? |
|---|--------|------|-------------|-----------|----------------------|
| 18 | `infer_self_borrow_type()` | 462 | Guesses `&self`/`&mut self`/`self` from method name | ❌ **Heuristic** | ✅ `method_calls[].self_borrow` |
| 19 | `transform_method_call()` | 548 | Wraps receiver with `track_borrow`/`track_borrow_mut` based on inferred borrow | ❌ **Heuristic** (uses #18) | ✅ Read `self_borrow` from `method_calls[]` |
| 20 | `transform_clone()` | 1619 | Emits `track_clone` for any `.clone()` call | ❌ **Heuristic** (can't verify Clone trait) | ✅ `method_calls[].trait_name == "Clone"` |
| 21 | `transform_lock()` | 1634 | Emits `track_lock` for `.lock()`/`.read()`/`.write()` | ❌ **Heuristic** (matches method name) | ✅ `method_calls[].operation` ending `::Mutex::lock` etc. |
| 22 | `transform_unwrap()` | 1653 | Emits `track_unwrap` for `.unwrap()`/`.expect()` etc. | ❌ **Heuristic** (matches method name) | ✅ `method_calls[].operation` ending `::Option::unwrap` etc. |
| 23 | `transform_call_expr()` | 1419 | Detects `transmute` by `path_str.contains("transmute")` | ❌ **Heuristic** | ✅ `expressions[].path == "core::mem::transmute"` |
| 24 | `transform_closure()` | 710 | Detects capture mode from `move` keyword, extracts captured vars by walking AST | ⚠️ **Partial** — `move` keyword is syntactic, captured vars are guessed | ✅ `closure_captures[]` has exact `CaptureKind` per variable |
| 25 | `transform_by_initializer_kind()` | 1129 | Dispatches on `initializer_kind` from analyzer | ✅ **Semantic** | — Already uses analyzer |
| 26 | `lookup_type_info()` | 120 | Looks up variable in type-info.json | ✅ **Semantic** | — Already uses analyzer |

**transform_visitor.rs — Structural Transforms (no type info needed):**

| # | Method | Line | What It Does | Needs Analyzer? |
|---|--------|------|-------------|----------------|
| 27 | `transform_local()` | 751 | Transforms `let` statements — wraps initializer with `track_new` | ⚠️ Partially — uses analyzer for `initializer_kind`, falls back to `detect_*()` |
| 28 | `transform_reference()` | 1319 | Wraps `&x`/`&mut x` with `track_borrow`/`track_borrow_mut` | ❌ No — reference expressions are syntactically unambiguous |
| 29 | `transform_unsafe_block()` | 1375 | Wraps unsafe blocks with enter/exit tracking | ⚠️ Could use `unsafe_operations` for richer context |
| 30 | `transform_ptr_cast()` | 1396 | Wraps `x as *const T` with `track_raw_ptr` | ❌ No — pointer casts are syntactically unambiguous |
| 31 | `transform_async_block()` | 1478 | Wraps async blocks with enter/exit tracking | ⚠️ Could use `await_points` for live variable info |
| 32 | `transform_await()` | 1498 | Wraps `.await` with start/end tracking | ⚠️ Could use `await_points[].live_variables` and `awaited_type` |
| 33 | `extract_future_name()` | 1515 | Guesses future name from AST | ⚠️ Could use `await_points[].awaited_type` |
| 34 | `transform_for_loop()` | 1540 | Wraps for loops with iteration tracking | ❌ No — loop structure is syntactic |
| 35 | `transform_while_loop()` | 1563 | Wraps while loops | ❌ No |
| 36 | `transform_loop()` | 1585 | Wraps loop expressions | ❌ No |
| 37 | `transform_try()` | 1605 | Wraps `?` operator | ❌ No — syntactically unambiguous |
| 38 | `transform_deref()` | 1702 | Wraps `*x` (DISABLED) | ⚠️ Could use `adjustments[]` for implicit deref |
| 39 | `transform_match()` | 1721 | Wraps match arms | ⚠️ Could use `match_bindings[]` for binding modes |
| 40 | `transform_if()` | 1761 | Wraps if/else branches | ❌ No |
| 41 | `transform_return()` | 1796 | Wraps return expressions | ❌ No |
| 42 | `transform_index()` | 1819 | Wraps `arr[i]` (DISABLED) | ❌ No |
| 43 | `transform_field()` | 1839 | Wraps `obj.field` (DISABLED) | ⚠️ Could use `field_accesses[]` for field type + access kind |
| 44 | `transform_fn_call()` | 1867 | Generic function call tracking (DISABLED) | ⚠️ Could use `callables[]` for param/return types |
| 45 | `transform_break()` | 1904 | Wraps break expressions | ❌ No |
| 46 | `transform_continue()` | 1944 | Wraps continue expressions | ❌ No |
| 47 | `transform_struct()` | 1968 | Wraps struct literal creation | ⚠️ Could use `record_field_exprs[]` for field types |
| 48 | `transform_tuple()` | 1983 | Wraps tuple creation | ❌ No |
| 49 | `transform_range()` | 1998 | Wraps range expressions | ❌ No |
| 50 | `transform_array()` | 2016 | Wraps array creation | ❌ No |
| 51 | `transform_cast()` | 2031 | Wraps non-pointer casts | ❌ No — cast target type is in the syntax |

**transform_visitor.rs — Helper Methods (no detection logic):**

| # | Method | Line | Purpose | Needs Analyzer? |
|---|--------|------|---------|----------------|
| 52 | `gen_id()` | 140 | Generate unique tracking IDs | ❌ No |
| 53 | `location_tokens()` | 148 | Generate location token stream | ❌ No |
| 54 | `extract_pattern_name()` | 156 | Get variable name from pattern | ❌ No |
| 55 | `matches_filter()` | 166 | Check if variable matches user filter | ❌ No |
| 56 | `glob_match()` | 182 | Glob pattern matching | ❌ No |
| 57 | `wrap_with_sampling()` | 222 | Wrap tracking call with sampling | ❌ No |
| 58 | `extract_borrowed_id()` | 237 | Get variable ID from borrow expression | ❌ No |
| 59 | `is_variable_path()` | 248 | Check if expr is a simple variable | ❌ No |
| 60 | `is_complex_pattern()` | 253 | Check if pattern needs destructuring | ❌ No |
| 61 | `get_simple_ident()` | 261 | Extract ident from simple pattern | ❌ No |
| 62 | `build_access_expr()` | 270 | Build tuple access expression | ❌ No |
| 63 | `generate_destructure_stmts()` | 286 | Generate destructuring statements | ❌ No |
| 64 | `transform_complex_pattern()` | 385 | Handle complex let patterns | ❌ No |
| 65 | `is_simple_variable()` | 533 | Check if expr is simple variable | ❌ No |
| 66 | `extract_receiver_name()` | 538 | Get receiver variable name | ❌ No |
| 67 | `is_move_closure()` | 626 | Check for `move` keyword | ❌ No |
| 68 | `extract_captured_vars()` | 631 | Walk AST to find captured variables | ⚠️ Could use `closure_captures[]` |
| 69 | `is_potential_move()` | 746 | Check if expr is a variable path | ❌ No |
| 70 | `extract_clone_source_id()` | 1276 | Get source var from `Rc::clone(&x)` | ❌ No — syntactically unambiguous |
| 71 | `extract_downgrade_source()` | 1291 | Get source var from `Rc::downgrade(&x)` | ❌ No |
| 72 | `extract_box_from_into_raw()` | 1305 | Get var from `Box::into_raw(b)` | ❌ No |

**visit_expr_mut dispatch (line 2199) — Heuristic Method-Name Matching:**

| # | Code Location | Heuristic | Analyzer Replacement |
|---|--------------|-----------|---------------------|
| 73 | `method_name == "clone"` (line 2325) | Weak::clone check by var name set | ✅ `method_calls[].operation` ending `::Weak::clone` |
| 74 | `method_name == "clone"` (line 2347) | Generic clone | ✅ `method_calls[].trait_name == "Clone"` |
| 75 | `method_name == "lock"` (line 2370) | Mutex::lock by name | ✅ `method_calls[].operation` ending `::Mutex::lock` |
| 76 | `method_name == "read"` (line 2375) | RwLock::read by name | ✅ `method_calls[].operation` ending `::RwLock::read` |
| 77 | `method_name == "write"` (line 2380) | RwLock::write by name — **AMBIGUOUS** with `io::Write::write`, `MaybeUninit::write` | ✅ `method_calls[].operation` disambiguates |
| 78 | `"unwrap" \| "expect" \| ...` (line 2393) | Unwrap methods by name | ✅ `method_calls[].operation` ending `::Option::unwrap` etc. |
| 79 | `cow_vars.contains() && "to_mut"` (line 2404) | Cow::to_mut by var set + name | ✅ `method_calls[].operation` |
| 80 | `weak_vars.get() && "upgrade"` (line 2419) | Weak::upgrade by var set + name | ✅ `method_calls[].operation` |
| 81 | `join_handle_vars.contains() && "join"` (line 2436) | JoinHandle::join by var set + name — **AMBIGUOUS** with `str::join` | ✅ `method_calls[].operation` disambiguates |
| 82 | `sender_vars.contains() && "send"` (line 2445) | Sender::send by var set + name | ✅ `method_calls[].operation` |
| 83 | `receiver_vars.contains() && "recv"` (line 2462) | Receiver::recv by var set + name | ✅ `method_calls[].operation` |
| 84 | `receiver_vars.contains() && "try_recv"` (line 2470) | Receiver::try_recv by var set + name | ✅ `method_calls[].operation` |
| 85 | `method_name == "borrow"` (line 2490) | RefCell::borrow by name — **AMBIGUOUS** with `Borrow::borrow` trait | ✅ `method_calls[].operation` disambiguates |
| 86 | `method_name == "borrow_mut"` (line 2498) | RefCell::borrow_mut by name | ✅ `method_calls[].operation` |
| 87 | `once_cell_vars.contains() + detect_once_cell_method()` (line 2507) | OnceCell methods by var set + name | ✅ `method_calls[].operation` |
| 88 | `maybe_uninit_vars.contains() + detect_maybe_uninit_method()` (line 2545) | MaybeUninit methods by var set + name | ✅ `method_calls[].operation` |
| 89 | `method_name == "get"` (line 2590) | Cell::get by name — **AMBIGUOUS** with `HashMap::get`, `Vec::get`, etc. | ✅ `method_calls[].operation` disambiguates |
| 90 | `method_name == "set"` (line 2600) | Cell::set by name — **AMBIGUOUS** with `OnceCell::set`, user `.set()` | ✅ `method_calls[].operation` disambiguates |

#### F. Summary Counts

| Category | Count | Analyzer Can Replace? |
|----------|-------|-----------------------|
| Detection functions (smart_pointer.rs) | 17 | ✅ All 17 — delete entirely |
| Heuristic transform methods | 6 (#18–23) | ✅ All 6 — read `method_calls[]`/`expressions[]` |
| Heuristic dispatch points in visit_expr_mut | 18 (#73–90) | ✅ All 18 — read `method_calls[].operation` |
| Partial transforms (could be enriched) | 10 (#24, 29, 31–33, 38–39, 43–44, 68) | ⚠️ Optional — analyzer has richer data |
| Structural transforms (no type info needed) | 17 (#28, 30, 34–37, 40–42, 45–51) | ❌ Not needed — syntactically unambiguous |
| Helper methods (no detection logic) | 21 (#52–72) | ❌ Not needed — pure AST manipulation |

**Total heuristic points: 41** (17 + 6 + 18) — all replaceable by reading `method_calls[]` and `expressions[]`.

**Ambiguous methods found** (most dangerous heuristics):
- `"write"` — could be `RwLock::write`, `io::Write::write`, `MaybeUninit::write`
- `"join"` — could be `JoinHandle::join`, `str::join`, `Vec::join`
- `"borrow"` — could be `RefCell::borrow`, `Borrow::borrow` trait
- `"get"` — could be `Cell::get`, `HashMap::get`, `Vec::get`, `OnceCell::get`
- `"set"` — could be `Cell::set`, `OnceCell::set`, user's `.set()`

### 3.3 Key Functions

| Function | Line | What It Does |
|----------|------|-------------|
| `analyze_method_calls()` | ~1943 | Resolves receiver type, method path, self_borrow, trait info for every method call |
| `resolve_self_borrow()` | ~2982 | `func.self_param(db).access(db)` → `Shared/Exclusive/Owned` |
| `resolve_method_path()` | ~3000 | Builds canonical path `crate::module::Type::method` |
| `resolve_trait_info()` | ~2041 | `func.container(db)` → detects trait impls via `i.trait_(db)` |
| `analyze_expressions()` | ~2069 | Resolves standalone calls via `TrackedFunctions` (FunctionId) |
| `classify_by_resolved_type_semantic()` | ~1232 | ADT classification with crate-verified fallback for unstable types |

### 3.4 Output Schema

`VariableTypeInfo` includes:
- `method_calls: Vec<MethodCallInfo>` — `operation`, `self_borrow`, `is_trait_method`, `trait_name`
- `expressions: Vec<ExpressionInfo>` — `kind`, `path`, `is_unsafe`

Macro-side structs (`type_info.rs`): `MethodCallInfo`, `ExpressionInfo` with `#[serde(default)]` for backward compatibility.

### 3.5 No New Classification Functions Needed

The original plan called for `classify_unwrap_method()` and `is_clone_trait_call()` as new analyzer functions. These are **not needed** because:

- **Unwrap**: `resolve_method_path()` already returns `core::option::Option::unwrap` etc. The macro matches the path suffix.
- **Clone**: `resolve_trait_info()` already returns `(Some(true), Some("Clone"))`. The macro checks `trait_name`.

Both are already in the `MethodCallInfo` output. The classification happens on the macro side via simple string matching on semantic paths (which is correct — matching on canonical paths from the compiler is not heuristic).

---

## 4. Macro-Side Refactoring — Complete

### 4.1 Files Modified

| File | Change |
|------|--------|
| `type_info.rs` | Added `MethodCallInfo` and `ExpressionInfo` structs with `#[serde(default)]` fields |
| `transform_visitor.rs` | `infer_self_borrow_type()` does semantic lookup first, heuristic fallback |
| `transform_visitor.rs` | All 18 method dispatch points use `semantic_op` lookup |
| `transform_visitor.rs` | Clone dispatch checks `is_trait_method`/`trait_name` |
| `transform_visitor.rs` | Unwrap dispatch verifies Option/Result via `semantic_op` |
| `transform_visitor.rs` | Transmute detection uses semantic expression data |
| `transform_visitor.rs` | Mapped guard initializer kinds dispatched |
| `smart_pointer.rs` | Deleted `detect_once_cell_method()` and `detect_maybe_uninit_method()` |

### 4.2 Functions Deleted

| Function | Was In | Replaced By |
|----------|--------|-------------|
| `detect_once_cell_method()` | smart_pointer.rs | `semantic_op` dispatch |
| `detect_maybe_uninit_method()` | smart_pointer.rs | `semantic_op` dispatch |

**Remaining `detect_*` functions** (10 in smart_pointer.rs) are used only in the initializer fallback path when no analyzer data is available:
`detect_smart_pointer_new`, `detect_rc_clone`, `detect_box_pin`, `detect_box_raw_op`, `detect_pin_operation`, `detect_cow_creation`, `detect_downgrade`, `detect_once_cell_new`, `detect_maybe_uninit_new`, `detect_concurrency_op`

### 4.3 Lookup Mechanism

The macro uses `crate::type_info::lookup_by_name(var_name)` to find analyzer data for a variable, then reads `method_calls[]` to find the relevant method call by name. The `operation`, `self_borrow`, `is_trait_method`, and `trait_name` fields drive dispatch decisions. When no analyzer data is available, all dispatch points fall back to name-based heuristics.

---

## 5. Testing Strategy

### 5.1 Per-Phase Test Cases

Each phase gets a standalone Rust file in `examples/type-coverage/src/` that exercises every pattern in that phase. The analyzer runs first, then the macro builds with the analyzer output.

#### Phase 1 test: `phase1_method_calls.rs`

```rust
use std::cell::Cell;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::mem::MaybeUninit;
use std::sync::mpsc;
use std::sync::{Mutex, RwLock};

#[borrowscope_macro::trace_borrow]
fn phase1_all_patterns() {
    // Cell::set (ID 11)
    let c = Cell::new(0);
    c.set(42);

    // Cow::to_mut (ID 19)
    let mut cow: Cow<str> = Cow::Borrowed("hello");
    cow.to_mut().push_str(" world");

    // OnceCell::set (ID 25)
    let once = OnceCell::new();
    once.set(42).ok();

    // MaybeUninit (IDs 32-33)
    let mut mu = MaybeUninit::<i32>::uninit();
    mu.write(42);
    unsafe { mu.assume_init_drop(); }

    // Channel ops (IDs 39-41)
    let (tx, rx) = mpsc::channel();
    tx.send(42).unwrap();
    let _ = rx.recv();
    let _ = rx.try_recv();

    // JoinHandle::join (ID 42)
    let h = std::thread::spawn(|| 42);
    h.join().unwrap();

    // Concurrency (IDs 43-45)
    let m = Mutex::new(0);
    let _ = m.try_lock();
    let r = RwLock::new(0);
    let _ = r.try_read();
    let _ = r.try_write();
}
```

#### Phase 2 test: `phase2_expressions.rs`

```rust
#[borrowscope_macro::trace_borrow]
fn phase2_all_patterns() {
    // thread::spawn (both paths → same FunctionId)
    let h1 = std::thread::spawn(|| { let x = 1; x });
    let h2 = { use std::thread; thread::spawn(|| 2) };

    // transmute (both paths → same FunctionId)
    let x: u32 = 42;
    let _y: f32 = unsafe { std::mem::transmute(x) };
    let _z: f32 = unsafe { core::mem::transmute(x) };
}
```

#### Phase 3 test: `phase3_self_borrow.rs`

```rust
#[borrowscope_macro::trace_borrow]
fn phase3_immutable() {
    let v = vec![1, 2, 3];
    let _ = v.as_slice();    // as_*
    let _ = v.len();         // len
    let _ = v.capacity();    // capacity
    let _ = v.iter();        // iter
    let _ = v.contains(&1);  // contains
    let _ = v.first();       // first
    let _ = v.last();        // last

    let s = String::from("hello");
    let _ = s.as_str();      // as_*
    let _ = s.to_uppercase();// to_*
    let _ = s.is_empty();    // is_*
    let _ = s.chars();       // chars
    let _ = s.bytes();       // bytes
    let _ = s.lines();       // lines (on &str)
    let _ = s.trim();        // trim
    let _ = s.contains("h"); // contains
    let _ = s.starts_with("h"); // starts_with
    let _ = s.ends_with("o");   // ends_with
    let _ = s.find("l");     // find
    let _ = s.clone();       // clone
}

#[borrowscope_macro::trace_borrow]
fn phase3_mutable() {
    let mut v = vec![1, 2, 3];
    v.push(4);       v.pop();        v.insert(0, 0);
    v.remove(0);     v.append(&mut vec![5]);
    v.clear();       v.truncate(0);  v.extend([1, 2]);
    v.drain(..);     v.sort();       v.reverse();
    v.dedup();       v.retain(|x| *x > 0);
}

#[borrowscope_macro::trace_borrow]
fn phase3_consuming() {
    let v = vec![1, 2, 3];
    let _ = v.into_iter();  // into_*

    let opt = Some(42);
    let _ = opt.unwrap();   // unwrap (consuming)

    let opt2 = Some(42);
    let _ = opt2.expect("msg"); // expect (consuming)
}
```

#### Phase 4 test: `phase4_unwrap.rs`

```rust
#[borrowscope_macro::trace_borrow]
fn phase4_all_patterns() {
    let a: Option<i32> = Some(1);
    let _ = a.unwrap();           // ID 102

    let b: Option<i32> = Some(2);
    let _ = b.expect("msg");      // ID 103

    let c: Option<i32> = None;
    let _ = c.unwrap_or(0);       // ID 104

    let d: Option<i32> = None;
    let _ = d.unwrap_or_else(|| 0); // ID 105

    let e: Option<i32> = None;
    let _ = e.unwrap_or_default(); // ID 106
}
```

#### Phase 5 test: `phase5_clone.rs`

```rust
struct MyType;
impl MyType {
    fn clone(&self) -> i32 { 42 } // inherent "clone" — NOT Clone::clone
}

#[borrowscope_macro::trace_borrow]
fn phase5_clone_disambiguation() {
    let x = vec![1, 2, 3];
    let _ = x.clone(); // Clone::clone → track_borrow("clone", &x)

    let m = MyType;
    let _ = m.clone(); // inherent method → track_borrow("method_borrow", &m), NOT "clone"
}
```

### 5.2 Regression Tests

Existing examples must produce identical tracking output before and after refactoring.

```bash
# Capture baseline (before refactoring)
cd examples/type-coverage && cargo run 2>&1 > baseline.json

# After refactoring, compare
cd examples/type-coverage && cargo run 2>&1 > refactored.json
diff baseline.json refactored.json
```

Also run the full test suite:
```bash
cargo test -p borrowscope-runtime --features track
cargo test -p borrowscope-macro
```

### 5.3 Success Criteria

1. **Zero syntactic detection in macro** — no `detect_*()` calls remain:
   ```bash
   grep -rn 'detect_smart_pointer_new\|detect_rc_clone\|detect_refcell_borrow\|detect_cell_operation\|detect_box_pin\|detect_box_raw_op\|detect_pin_operation\|detect_cow_creation\|detect_cow_to_mut\|detect_downgrade\|detect_weak_upgrade\|detect_once_cell_new\|detect_once_cell_method\|detect_maybe_uninit_new\|detect_maybe_uninit_method\|detect_concurrency_op' borrowscope-macro/src/
   # Expected: no output
   ```

2. **`infer_self_borrow_type()` only used as fallback**:
   ```bash
   grep -n 'infer_self_borrow_type' borrowscope-macro/src/transform_visitor.rs
   # Expected: only in unwrap_or_else fallback, not as primary path
   ```

3. **109/109 patterns semantic** in README:
   ```bash
   grep -c '✅' borrowscope-analyzer/README.md
   # Expected: 109
   grep -c '❌\|⚠️' borrowscope-analyzer/README.md
   # Expected: 0
   ```

4. **All tests pass**:
   ```bash
   cargo test -p borrowscope-runtime --features track && cargo test -p borrowscope-macro
   ```

5. **type-coverage example runs clean**:
   ```bash
   cd examples/type-coverage
   cargo run -p borrowscope-analyzer -- .
   cargo run 2>&1 | grep -c "ERROR"
   # Expected: 0
   ```

