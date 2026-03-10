# Semantic Category Implementation Specification

> Implementation guide for achieving 100% semantic coverage (109/109 patterns) in borrowscope-analyzer.

## Table of Contents

- [1. Coverage Overview](#1-coverage-overview)
  - [1.1 Summary Table](#11-summary-table)
  - [1.2 Complete Pattern Registry (109 Patterns)](#12-complete-pattern-registry-109-patterns)
- [2. Semantic Patterns (36) — Already Complete](#2-semantic-patterns-36--already-complete)
  - [2.1 Smart Pointer Creation (5)](#21-smart-pointer-creation-5)
  - [2.2 Smart Pointer Clone (2)](#22-smart-pointer-clone-2)
  - [2.3 Box Operations (3)](#23-box-operations-3)
  - [2.4 Weak Reference Operations (3)](#24-weak-reference-operations-3)
  - [2.5 RefCell/Cell Creation & Guards (3 of 4)](#25-refcellcell-creation--guards-3-of-4)
  - [2.6 Pin Creation (1 of 2)](#26-pin-creation-1-of-2)
  - [2.7 Cow Creation (2 of 3)](#27-cow-creation-2-of-3)
  - [2.8 OnceCell/OnceLock Creation (2 of 5)](#28-onceceloncelock-creation-2-of-5)
  - [2.9 MaybeUninit Creation (2 of 6)](#29-maybeuninit-creation-2-of-6)
  - [2.10 Concurrency Creation (5 of 12)](#210-concurrency-creation-5-of-12)
  - [2.11 Guard Creation (8 of 9)](#211-guard-creation-8-of-9)
- [3. Partial Patterns (7) — Implementation Spec](#3-partial-patterns-7--implementation-spec)
  - [3.1 Pin::as_ref / Pin::as_mut (1 pattern)](#31-pinas_ref--pinas_mut-1-pattern)
  - [3.2 OnceCell::get / OnceCell::get_or_init (2 patterns)](#32-oncecellget--oncecellget_or_init-2-patterns)
  - [3.3 MaybeUninit::assume_init / MaybeUninit::write (2 patterns)](#33-maybeuninitassume_init--maybeuninitwrite-2-patterns)
  - [3.4 Guard::map / Guard::try_map (1 pattern)](#34-guardmap--guardtry_map-1-pattern)
  - [3.5 Clone Trait Detection (1 pattern)](#35-clone-trait-detection-1-pattern)
- [4. Syntactic Patterns (66) — Implementation Spec](#4-syntactic-patterns-66--implementation-spec)
  - [4.1 Phase 1: Method Call Tracking (13 patterns)](#41-phase-1-method-call-tracking-13-patterns)
    - [4.1.1 Cell::set (1)](#411-cellset-1)
    - [4.1.2 Cow::to_mut (1)](#412-cowto_mut-1)
    - [4.1.3 OnceCell::set (1)](#413-oncecellset-1)
    - [4.1.4 MaybeUninit methods (2)](#414-maybeuninit-methods-2)
    - [4.1.5 Channel operations (3)](#415-channel-operations-3)
    - [4.1.6 JoinHandle::join (1)](#416-joinhandlejoin-1)
    - [4.1.7 Concurrency remaining (4)](#417-concurrency-remaining-4)
  - [4.2 Phase 2: Standalone Expression Tracking (4 patterns)](#42-phase-2-standalone-expression-tracking-4-patterns)
    - [4.2.1 thread::spawn (2)](#421-threadspawn-2)
    - [4.2.2 transmute (2)](#422-transmute-2)
  - [4.3 Phase 3: Self-Borrow Type Resolution (47 patterns)](#43-phase-3-self-borrow-type-resolution-47-patterns)
    - [4.3.1 Immutable self-borrow (19)](#431-immutable-self-borrow-19)
    - [4.3.2 Mutable self-borrow (25)](#432-mutable-self-borrow-25)
    - [4.3.3 Consuming self-borrow (3)](#433-consuming-self-borrow-3)
  - [4.4 Phase 4: Unwrap Method Tracking (5 patterns)](#44-phase-4-unwrap-method-tracking-5-patterns)
    - [4.4.1 unwrap (1)](#441-unwrap-1)
    - [4.4.2 expect (1)](#442-expect-1)
    - [4.4.3 unwrap_or (1)](#443-unwrap_or-1)
    - [4.4.4 unwrap_or_else (1)](#444-unwrap_or_else-1)
    - [4.4.5 unwrap_or_default (1)](#445-unwrap_or_default-1)
  - [4.5 Phase 5: Clone Trait Verification (1 pattern)](#45-phase-5-clone-trait-verification-1-pattern)
- [5. Analyzer-Side Implementation](#5-analyzer-side-implementation)
  - [5.1 Current State Assessment](#51-current-state-assessment)
  - [5.2 Macro ↔ Analyzer Data Flow](#52-macro--analyzer-data-flow-whats-connected-vs-whats-ignored)
  - [5.3 Remaining Analyzer Changes](#53-remaining-analyzer-changes)
  - [5.4 Output Schema Changes](#54-output-schema-changes-v30--v31)
  - [5.5 No New Classification Functions Needed](#55-no-new-classification-functions-needed)
- [6. Macro-Side Refactoring](#6-macro-side-refactoring)
  - [6.1 Files to Modify](#61-files-to-modify)
  - [6.2 Functions to Delete](#62-functions-to-delete)
  - [6.3 New Lookup Logic](#63-new-lookup-logic)
- [7. Testing Strategy](#7-testing-strategy)
  - [7.1 Per-Phase Test Cases](#71-per-phase-test-cases)
  - [7.2 Regression Tests](#72-regression-tests)
  - [7.3 Success Criteria](#73-success-criteria)

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

## 2. Semantic Patterns (36) — Already Complete

These 36 patterns are fully implemented in `classify_by_resolved_type_semantic()` (analysis.rs:1203). Each uses ADT identity comparison via `KnownTypes.classify(&adt)` — zero string matching. The function receives the resolved `ra_ap_hir::Type` and an `expr_kind` string derived from expression structure (not from user code), then matches `(type_class, expr_kind)` tuples.

**No changes needed for any pattern in this section.**

### 2.1 Smart Pointer Creation (5)

| Pattern ID | Match Arm | KnownTypes Field | Lang Item / Import Path |
|------------|-----------|------------------|------------------------|
| 1 | `("rc", "call") => "rc_new"` | `rc` | `alloc::rc::Rc` via import_map |
| 2 | `("arc", "call") => "arc_new"` | `arc` | `alloc::sync::Arc` via import_map |
| 3 | `("box", "call") => "box_new"` | `box_` | lang item `OwnedBox` |
| 4 | `("weak", "call") => "weak_new"` | `weak_rc` / `weak_arc` | `alloc::rc::Weak` / `alloc::sync::Weak` |
| 5 | `("weak", "downgrade") => "weak_downgrade"` | (same) | expr_kind set when call is `Rc::downgrade()` |

`expr_kind` is determined by expression structure analysis before the match:
- `"call"` = `Struct::method()` or `Struct::new()` pattern
- `"clone"` = `.clone()` call where result type matches
- `"downgrade"` / `"upgrade"` = specific associated function patterns

### 2.2 Smart Pointer Clone (2)

| Pattern ID | Match Arm | How expr_kind Becomes "clone" |
|------------|-----------|-------------------------------|
| 6 | `("rc", "clone") => "rc_clone"` | Result type of `.clone()` resolves to `Rc<T>` ADT |
| 7 | `("arc", "clone") => "arc_clone"` | Result type of `.clone()` resolves to `Arc<T>` ADT |

### 2.3 Box Operations (3)

| Pattern ID | Match Arm | Notes |
|------------|-----------|-------|
| 12 | `("box", "call") => "box_new"` | Covers `Box::new(v)` and `Box::from_raw(p)` — both produce `Box<T>` ADT |
| 13 | Result type is `*mut T` → `"raw_ptr"` | `Box::into_raw()` — classified by result type being raw pointer |
| 14 | `("box", "call") => "box_new"` | `Box::from_raw(p)` — result is `Box<T>`, same match arm as ID 12 |

### 2.4 Weak Reference Operations (3)

| Pattern ID | Match Arm |
|------------|-----------|
| 20 | `("weak", "call") => "weak_new"` |
| 21 | `("weak", "downgrade") => "weak_downgrade"` |
| 22 | `("weak", "upgrade") => "weak_upgrade"` |

### 2.5 RefCell/Cell Creation & Guards (3 of 4)

| Pattern ID | Match Arm | Notes |
|------------|-----------|-------|
| 8 | `("refcell", "call") => "refcell_new"` | |
| 9 | `("ref_guard", "borrow") => "refcell_borrow"` | Result type is `Ref<T>` guard |
| 10 | `("refmut_guard", "borrow_mut") => "refcell_borrow_mut"` | Result type is `RefMut<T>` guard |

Cell::new is also semantic: `("cell", "call") => "cell_new"`. Cell::set (ID 11) is syntactic — see Section 4.

### 2.6 Pin Creation (1 of 2)

| Pattern ID | Match Arm | Notes |
|------------|-----------|-------|
| 15 | `("pin", "call") => "pin_new"` | `KnownTypes.pin` resolved via lang item `Pin` |

Pin::as_ref/as_mut (ID 16) is partial — see Section 3.

### 2.7 Cow Creation (2 of 3)

| Pattern ID | Match Arm |
|------------|-----------|
| 17 | `("cow", "call") => "cow_new"` |
| 18 | `("cow", "path") => "cow_variant"` |

Cow::to_mut (ID 19) is syntactic — see Section 4.

### 2.8 OnceCell/OnceLock Creation (2 of 5)

| Pattern ID | Match Arm |
|------------|-----------|
| 23 | `("once_cell", "call") => "once_cell_new"` |
| 24 | `("once_lock", "call") => "once_lock_new"` |

### 2.9 MaybeUninit Creation (2 of 6)

| Pattern ID | Match Arm | Notes |
|------------|-----------|-------|
| 28 | `("maybe_uninit", "call") => "maybe_uninit_new"` | Covers `MaybeUninit::uninit()` |
| 29 | `("maybe_uninit", "call") => "maybe_uninit_new"` | Covers `MaybeUninit::zeroed()` — same ADT + call |

Also: `("maybe_uninit", _) => "maybe_uninit"` catches any other MaybeUninit expression as generic.

### 2.10 Concurrency Creation (5 of 12)

| Pattern ID | Match Arm | Notes |
|------------|-----------|-------|
| 34 | `("mutex", "call") => "mutex_new"` | |
| 35 | `("rwlock", "call") => "rwlock_new"` | |
| 36 | `("channel_sender", _) \| ("channel_receiver", _) => "channel_new"` | Tuple destructuring `(tx, rx)` — both elements classified |
| 37 | `("mutex_guard", "lock") => "mutex_lock"` | Result type is `MutexGuard<T>` |
| 38 | `("rwlock_read_guard", "read") => "rwlock_read"` | Result type is `RwLockReadGuard<T>` |

### 2.11 Guard Creation (8 of 9)

| Pattern ID | Match Arm | Guard Type |
|------------|-----------|------------|
| 46 | `("mutex_guard", "lock") => "mutex_lock"` | `MutexGuard<T>` |
| 47 | `("rwlock_read_guard", "read") => "rwlock_read"` | `RwLockReadGuard<T>` |
| 48 | `("rwlock_write_guard", "write") => "rwlock_write"` | `RwLockWriteGuard<T>` |
| 49 | `("ref_guard", "borrow") => "refcell_borrow"` | `Ref<T>` |
| 50 | `("refmut_guard", "borrow_mut") => "refcell_borrow_mut"` | `RefMut<T>` |
| 51 | (same as 46) | |
| 52 | (same as 47) | |
| 53 | `("rwlock_write_guard", "write") => "rwlock_write"` | `RwLockWriteGuard<T>` |

Guard::map (ID 54) is partial — see Section 3.

---

## 3. Partial Patterns (7) — Implementation Spec

These patterns have semantic type information available (the receiver ADT is known) but the operation classification still falls back to method name string matching in the macro. Each needs the analyzer to emit the operation in `MethodCallInfo.operation` so the macro can look it up instead of guessing.

**Common fix for all 7**: The analyzer's `analyze_method_calls()` already resolves `receiver_type` semantically and populates `MethodCallInfo.operation` with the canonical path (e.g., `core::cell::Cell::set`). The macro needs to read `method_calls[]` from `type-info.json` and use `operation` instead of `infer_self_borrow_type()`.

### 3.1 Pin::as_ref / Pin::as_mut (ID 16)

**Current state**: Initializer classified as `"pin_new"` via `("pin", "call")`. But `p.as_ref()` / `p.as_mut()` are method calls on an existing `Pin` variable — not initializers. The macro falls back to `infer_self_borrow_type("as_ref")` → `Immutable` (correct by accident for `as_ref`, wrong for `as_mut`).

**What's semantic**: Receiver type resolves to `Pin<&mut T>` ADT. `resolve_self_borrow()` returns `Access::Shared` for `as_ref`, `Access::Exclusive` for `as_mut`.

**What's missing**: Macro doesn't read `self_borrow` from analyzer output for this method call. It uses name-based heuristic instead.

**Fix**: Macro reads `method_calls[].self_borrow` from type-info.json for the variable. If present, use it directly. No analyzer changes needed — data is already emitted.

### 3.2 OnceCell::get / OnceCell::get_or_init (IDs 26–27)

**Current state**: `OnceCell::new()` classified semantically as `"once_cell_new"`. But `.get()` and `.get_or_init(|| v)` are method calls. Macro uses `detect_once_cell_method()` which matches `method_name == "get"` / `method_name == "get_or_init"`.

**What's semantic**: `analyze_method_calls()` resolves receiver to `OnceCell<T>` ADT. `operation` field contains `core::cell::OnceCell::get` or `core::cell::OnceCell::get_or_init`.

**What's missing**: Same as 3.1 — macro doesn't consume the analyzer's method call data.

**Fix**: Macro looks up `method_calls[]` by line/method name, reads `operation` field. Maps `"...::OnceCell::get"` → `track_cell_get`, `"...::OnceCell::get_or_init"` → `track_cell_get`.

### 3.3 MaybeUninit::assume_init / MaybeUninit::write (IDs 30–31)

**Current state**: `MaybeUninit::uninit()` classified as `"maybe_uninit_new"`. But `.write(v)` and `.assume_init()` are method calls. Macro uses `detect_maybe_uninit_method()` matching by name.

**What's semantic**: Receiver resolves to `MaybeUninit<T>` ADT. `operation` = `core::mem::MaybeUninit::write` / `core::mem::MaybeUninit::assume_init`. `self_borrow` = `"mutable"` for write, `"consuming"` for assume_init.

**Fix**: Same pattern — macro reads `operation` from method_calls.

### 3.4 Guard::map / Guard::try_map (ID 54)

**Current state**: Guard types (MutexGuard, RwLockReadGuard, etc.) are classified semantically when created via `.lock()`, `.read()`, `.write()`. But `MutexGuard::map(g, |d| &d.field)` is a static method call, not a method on a variable. The macro skips it entirely.

**What's semantic**: The result type of `MutexGuard::map()` is `MappedMutexGuard<T>` — a distinct ADT. The analyzer can classify this via `classify_by_resolved_type_semantic()` if we add the mapped guard types to `KnownTypes`.

**What's missing**: `KnownTypes` doesn't include `MappedMutexGuard`, `MappedRwLockReadGuard`, `MappedRwLockWriteGuard`.

**Fix (analyzer-side)**:
```rust
// Add to KnownTypes struct:
pub mapped_mutex_guard: Option<AdtId>,
pub mapped_rwlock_read_guard: Option<AdtId>,
pub mapped_rwlock_write_guard: Option<AdtId>,

// Add to classify():
if Some(id) == self.mapped_mutex_guard { return Some("mapped_mutex_guard"); }
// ...

// Add match arms:
("mapped_mutex_guard", _) => "mutex_guard_map",
("mapped_rwlock_read_guard", _) => "rwlock_read_guard_map",
("mapped_rwlock_write_guard", _) => "rwlock_write_guard_map",
```

### 3.5 Clone Trait Detection (ID 107)

**Current state**: Macro detects `.clone()` by method name. Cannot distinguish `Clone::clone` trait impl from an inherent method named `clone`.

**What's semantic**: `analyze_method_calls()` already populates `is_trait_method: Some(true)` and `trait_name: Some("Clone")` via `resolve_trait_info()`. This data is in the JSON output.

**What's missing**: Macro doesn't read `is_trait_method` / `trait_name` from method_calls.

**Fix**: Macro looks up method call, checks `trait_name == "Clone"`. If true → `track_borrow("clone", &x)`. If false (inherent method) → skip or track differently.

---

## 4. Syntactic Patterns (66) — Implementation Spec

These patterns currently rely on string matching in the macro. The fix is uniform: the analyzer already emits `MethodCallInfo` with semantic `operation`, `self_borrow`, `receiver_type`, `is_trait_method`, and `trait_name` for every method call. The macro must read this data from `type-info.json` instead of using `infer_self_borrow_type()` and `detect_*()` functions.

### 4.1 Phase 1: Method Call Tracking (13 patterns)

These are method calls on known types where the macro currently matches by method name string. The analyzer already resolves all of them semantically via `analyze_method_calls()` → `resolve_method_path()`.

**Unified mechanism**: For each method call `receiver.method(args)`, the analyzer emits:
```json
{
  "method": "set",
  "operation": "core::cell::Cell::set",
  "self_borrow": "mutable",
  "receiver_type": "Cell<i32>",
  "is_trait_method": false
}
```

The macro looks up `method_calls[]` for the variable by name + line proximity, reads `operation`, and maps it to the correct `track_*` call.

#### 4.1.1 Cell::set (ID 11)

| Field | Value |
|-------|-------|
| **Macro today** | `detect_cell_operation()`: `method.to_string() == "set"` |
| **Analyzer emits** | `operation: "core::cell::Cell::set"`, `self_borrow: "mutable"` |
| **Macro replacement** | Match `operation` ending in `::Cell::set` → `track_cell_set(name)` |

#### 4.1.2 Cow::to_mut (ID 19)

| Field | Value |
|-------|-------|
| **Macro today** | `detect_cow_to_mut()`: `method.to_string() == "to_mut"` |
| **Analyzer emits** | `operation: "alloc::borrow::Cow::to_mut"`, `self_borrow: "mutable"` |
| **Macro replacement** | Match `::Cow::to_mut` → `track_borrow_mut("cow_to_mut", &mut receiver)` |

#### 4.1.3 OnceCell::set (ID 25)

| Field | Value |
|-------|-------|
| **Macro today** | `detect_once_cell_method()`: `method_name == "set"` — ambiguous with `Cell::set` |
| **Analyzer emits** | `operation: "core::cell::OnceCell::set"`, `self_borrow: "mutable"` |
| **Macro replacement** | Match `::OnceCell::set` → `track_cell_set(name)` |

#### 4.1.4 MaybeUninit methods (IDs 32–33)

| ID | Method | Analyzer Operation | self_borrow |
|----|--------|--------------------|-------------|
| 32 | `assume_init_read` | `core::mem::MaybeUninit::assume_init_read` | `consuming` |
| 33 | `assume_init_drop` | `core::mem::MaybeUninit::assume_init_drop` | `mutable` |

**Macro today**: `detect_maybe_uninit_method()` matching by name.
**Macro replacement**: Match `::MaybeUninit::assume_init_read` → `track_move`, `::MaybeUninit::assume_init_drop` → `track_drop`.

#### 4.1.5 Channel operations (IDs 39–41)

| ID | Method | Analyzer Operation | self_borrow |
|----|--------|--------------------|-------------|
| 39 | `send` | `std::sync::mpsc::Sender::send` | `immutable` |
| 40 | `recv` | `std::sync::mpsc::Receiver::recv` | `mutable` |
| 41 | `try_recv` | `std::sync::mpsc::Receiver::try_recv` | `mutable` |

**Macro today**: `detect_concurrency_op()` matching `method_name == "send"` etc.
**Macro replacement**: Match `::Sender::send` / `::Receiver::recv` / `::Receiver::try_recv` from `operation` field.

#### 4.1.6 JoinHandle::join (ID 42)

| Field | Value |
|-------|-------|
| **Macro today** | `method_name == "join"` — ambiguous with `str::join`, `Vec::join`, etc. |
| **Analyzer emits** | `operation: "std::thread::JoinHandle::join"`, `self_borrow: "consuming"` |
| **Macro replacement** | Match `::JoinHandle::join` → `track_move("join", handle)` |

This is the most important disambiguation — `"join"` is extremely common as a method name.

#### 4.1.7 Concurrency remaining (IDs 43–45 + rwlock_try_write)

| ID | Method | Analyzer Operation | self_borrow |
|----|--------|--------------------|-------------|
| 43 | `try_lock` | `std::sync::Mutex::try_lock` | `immutable` |
| 44 | `try_read` | `std::sync::RwLock::try_read` | `immutable` |
| 45 | `try_write` | `std::sync::RwLock::try_write` | `immutable` |

**Macro today**: These are in the `guard_methods` skip-list in `transform_method_call()` — the macro skips wrapping them to avoid lifetime issues with guard temporaries.
**Macro replacement**: Still skip wrapping (guard lifetime issue remains), but emit a standalone `track_borrow("lock_attempt", &receiver)` before the call using analyzer data.

### 4.2 Phase 2: Standalone Expression Tracking (4 patterns)

These are free function calls (not method calls). The analyzer tracks them via `analyze_expressions()` using `TrackedFunctions` (FunctionId comparison).

#### 4.2.1 thread::spawn (IDs not in registry — covered by ExpressionInfo)

Already fully semantic in the analyzer. `TrackedFunctions` resolves `std::thread::spawn` by FunctionId. Both `thread::spawn(|| {})` and `std::thread::spawn(|| {})` resolve to the same `FunctionId`.

**Analyzer emits**: `ExpressionInfo { kind: "function_call", path: "std::thread::spawn", closure_captures: [...] }`

**Macro today**: `path_str.contains("spawn")` — matches any function with "spawn" in the path.
**Macro replacement**: Read `expressions[]` from type-info.json. Match `path == "std::thread::spawn"`.

#### 4.2.2 transmute (IDs 108–109)

Already fully semantic in the analyzer. `TrackedFunctions` resolves `std::mem::transmute` by FunctionId.

**Analyzer emits**: `ExpressionInfo { kind: "function_call", path: "core::mem::transmute", is_unsafe: true }`

**Macro today**: `path_str.contains("transmute")`.
**Macro replacement**: Read `expressions[]`, match `path == "core::mem::transmute"` → `track_transmute(name, from_type, to_type)`.

### 4.3 Phase 3: Self-Borrow Type Resolution (47 patterns)

This is the largest group. All 47 patterns share the same problem: the macro's `infer_self_borrow_type()` uses method name prefixes/matches to guess `&self` vs `&mut self` vs `self`. The analyzer's `resolve_self_borrow()` already computes this exactly via `func.self_param(db).access(db)` → `Access::Shared/Exclusive/Owned`.

**Single fix for all 47**: Replace `infer_self_borrow_type(method_name)` with a lookup into `method_calls[].self_borrow` from type-info.json.

```rust
// BEFORE (transform_visitor.rs:578):
let borrow_type = Self::infer_self_borrow_type(&method_name);

// AFTER:
let borrow_type = self.lookup_method_call_borrow(&receiver_name, &method_name, call_line)
    .unwrap_or_else(|| Self::infer_self_borrow_type(&method_name)); // fallback if no analyzer data
```

The lookup function:
```rust
fn lookup_method_call_borrow(&self, var_name: &str, method: &str, line: u32) -> Option<SelfBorrowType> {
    let type_info = self.type_info_cache.as_ref()?;
    let var_info = type_info.lookup_in_function(&self.current_fn_name, var_name, None)?;
    let mc = var_info.method_calls.iter()
        .find(|mc| mc.method == method && mc.line == line)?;
    match mc.self_borrow.as_deref()? {
        "immutable" => Some(SelfBorrowType::Immutable),
        "mutable" => Some(SelfBorrowType::Mutable),
        "consuming" => Some(SelfBorrowType::Consuming),
        _ => None,
    }
}
```

#### 4.3.1 Immutable self-borrow (19 patterns, IDs 55–73)

All resolve to `Access::Shared` → `self_borrow: "immutable"`. The macro currently gets these right by accident (name heuristics happen to match), but fails on user-defined methods with the same names.

| ID | Pattern | Example False Positive |
|----|---------|----------------------|
| 55 | `as_*` | `my_struct.as_thing()` taking `&mut self` |
| 58 | `get*` | `cache.get_mut()` taking `&mut self` |
| 71 | `clone` | Inherent `.clone()` taking `self` (consuming) |

With semantic data, all 19 are correct regardless of naming.

#### 4.3.2 Mutable self-borrow (25 patterns, IDs 74–98)

All resolve to `Access::Exclusive` → `self_borrow: "mutable"`. Current heuristics are mostly correct for std types but wrong for user types.

| ID | Pattern | Example False Positive |
|----|---------|----------------------|
| 80 | `set*` | `config.settings()` — not a setter |
| 93 | `send` | `my_logger.send()` taking `&self` |
| 97 | `lock` | `my_cache.lock()` returning non-guard |

#### 4.3.3 Consuming self-borrow (3 patterns, IDs 99–101)

All resolve to `Access::Owned` → `self_borrow: "consuming"`.

| ID | Pattern | Notes |
|----|---------|-------|
| 99 | `into_*` | Always consuming by convention, but semantic confirms it |
| 100 | `unwrap` | Consuming on `Option`/`Result`, but could be `&self` on user types |
| 101 | `expect` | Same as unwrap |

### 4.4 Phase 4: Unwrap Method Tracking (5 patterns)

These are method calls on `Option<T>` or `Result<T, E>` that extract the inner value. The macro currently detects them by name. The analyzer provides both the receiver type (Option/Result ADT) and the operation path.

**Unified mechanism**: Macro reads `operation` from method_calls. If it matches `core::option::Option::unwrap` (or `core::result::Result::unwrap`), emit the appropriate tracking call.

#### 4.4.1–4.4.5 (IDs 102–106)

| ID | Method | Analyzer Operation | self_borrow | Tracking Call |
|----|--------|--------------------|-------------|---------------|
| 102 | `unwrap` | `core::option::Option::unwrap` | `consuming` | `track_move("unwrap", v)` |
| 103 | `expect` | `core::option::Option::expect` | `consuming` | `track_move("expect", v)` |
| 104 | `unwrap_or` | `core::option::Option::unwrap_or` | `consuming` | `track_move("unwrap_or", v)` |
| 105 | `unwrap_or_else` | `core::option::Option::unwrap_or_else` | `consuming` | `track_move("unwrap_or_else", v)` |
| 106 | `unwrap_or_default` | `core::option::Option::unwrap_or_default` | `consuming` | `track_move("unwrap_or_default", v)` |

All five also exist on `Result<T, E>` with paths like `core::result::Result::unwrap`. The macro matches the suffix (`::unwrap`, `::expect`, etc.) regardless of whether it's Option or Result.

### 4.5 Phase 5: Clone Trait Verification (1 pattern)

#### ID 107: `.clone()` → verify `Clone::clone`

**Macro today**: `method_name == "clone"` → always emits `track_borrow("clone", &x)`.

**Analyzer emits**: `is_trait_method: true, trait_name: "Clone"` when `.clone()` resolves to the `Clone::clone` trait impl.

**Macro replacement**:
```rust
if method_name == "clone" {
    if let Some(mc) = self.lookup_method_call(var, "clone", line) {
        if mc.trait_name.as_deref() == Some("Clone") {
            // Confirmed Clone::clone — emit track_borrow
        } else {
            // Inherent method named "clone" — use self_borrow from analyzer
        }
    }
}
```

---

## 5. Analyzer-Side Implementation

### 5.1 Current State Assessment

The analyzer already implements everything needed for 66 of 73 remaining patterns. No new analyzer code is required for those — the macro just needs to consume the existing output.

**What already works (analysis.rs on `feature/analyzer-method-call-tracking`):**

| Function | Line | What It Does | Covers |
|----------|------|-------------|--------|
| `analyze_method_calls()` | 1943 | Iterates all `MethodCallExpr` nodes, resolves receiver type, method path, self_borrow, trait info, unsafe flag. Populates `MethodCallInfo` on each variable. | P1 (13), P3 (47), P4 (5), P5 (1) |
| `resolve_self_borrow()` | 2982 | `func.self_param(db).access(db)` → `Shared/Exclusive/Owned` | All 47 self-borrow patterns |
| `resolve_method_path()` | 3000 | Builds canonical path `crate::module::Type::method` | All method-based patterns |
| `resolve_trait_info()` | 2041 | `func.container(db)` → `Trait(t)` or `Impl(_)` | Clone verification (P5) |
| `analyze_expressions()` | 2069 | Iterates `CallExpr` nodes, resolves via `TrackedFunctions` (FunctionId) | P2 (4 patterns: spawn, transmute) |
| `extract_closure_captures_semantic()` | 2170 | `closure_hir.captured_items(db)` with `CaptureKind` | thread::spawn closure captures |

**Gap analysis — what's missing:**

| Gap | Patterns Affected | Fix |
|-----|-------------------|-----|
| `method_calls` not serialized to type-info.json | All 66 | Add `method_calls: Vec<MethodCallInfo>` to output schema |
| `expressions` not serialized to type-info.json | 4 (P2) | Add `expressions: Vec<ExpressionInfo>` to output schema |
| Mapped guard types not in `KnownTypes` | 1 (ID 54) | Add 3 ADTs (see 5.3) |

### 5.2 Macro ↔ Analyzer Data Flow: What's Connected vs What's Ignored

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

**Fields the analyzer writes but macro IGNORES (31):**

| Field | What It Contains | Potential Use |
|-------|-----------------|---------------|
| `method_calls` | `Vec<MethodCallInfo>` — every method call with semantic operation, self_borrow, trait info | **CRITICAL**: Replaces all 73 heuristic patterns |
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

### 5.3 Remaining Analyzer Changes

Only one actual code change is needed on the analyzer side: adding mapped guard types to `KnownTypes` for Guard::map (ID 54).

```rust
// In KnownTypes::new(), add after existing guard lookups:
let mapped_mutex_guard = find_adt(db, &import_map, "std::sync::MappedMutexGuard");
let mapped_rwlock_read_guard = find_adt(db, &import_map, "std::sync::MappedRwLockReadGuard");
let mapped_rwlock_write_guard = find_adt(db, &import_map, "std::sync::MappedRwLockWriteGuard");

// In KnownTypes::classify(), add:
if Some(id) == self.mapped_mutex_guard { return Some("mapped_mutex_guard"); }
if Some(id) == self.mapped_rwlock_read_guard { return Some("mapped_rwlock_read_guard"); }
if Some(id) == self.mapped_rwlock_write_guard { return Some("mapped_rwlock_write_guard"); }

// In classify_by_resolved_type_semantic(), add match arms:
("mapped_mutex_guard", _) => "mutex_guard_map",
("mapped_rwlock_read_guard", _) => "rwlock_read_guard_map",
("mapped_rwlock_write_guard", _) => "rwlock_write_guard_map",
```

### 5.4 Output Schema Changes (v3.0 → v3.1)

The `VariableTypeInfo` in `output.rs` already has `method_calls: Vec<MethodCallInfo>`. The JSON serialization already includes it. The macro's `type_info.rs` deserialization struct needs to add the field:

```rust
// Add to type_info.rs VariableTypeInfo:
#[serde(default)]
pub method_calls: Vec<MethodCallInfoCompact>,

// Compact version (macro only needs these fields):
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MethodCallInfoCompact {
    pub method: String,
    pub line: u32,
    pub operation: Option<String>,
    pub self_borrow: Option<String>,
    pub is_trait_method: Option<bool>,
    pub trait_name: Option<String>,
}
```

Similarly for expressions (needed for P2):
```rust
#[serde(default)]
pub expressions: Vec<ExpressionInfoCompact>,

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExpressionInfoCompact {
    pub line: u32,
    pub kind: String,
    pub path: Option<String>,
    pub is_unsafe: Option<bool>,
}
```

### 5.5 No New Classification Functions Needed

The original plan called for `classify_unwrap_method()` and `is_clone_trait_call()` as new analyzer functions. These are **not needed** because:

- **Unwrap**: `resolve_method_path()` already returns `core::option::Option::unwrap` etc. The macro matches the path suffix.
- **Clone**: `resolve_trait_info()` already returns `(Some(true), Some("Clone"))`. The macro checks `trait_name`.

Both are already in the `MethodCallInfo` output. The classification happens on the macro side via simple string matching on semantic paths (which is correct — matching on canonical paths from the compiler is not heuristic).

---

## 6. Macro-Side Refactoring

### 6.1 Files to Modify

| File | Change | Scope |
|------|--------|-------|
| `type_info.rs` | Add `method_calls` and `expressions` deserialization (see §5.4) | ~20 lines |
| `type_info.rs` | Add `lookup_method_call()` and `lookup_expression()` functions | ~30 lines |
| `transform_visitor.rs` | Replace `infer_self_borrow_type()` call with semantic lookup + fallback | ~5 lines changed |
| `transform_visitor.rs` | Replace `detect_*()` calls in `visit_expr_mut()` with operation-based dispatch | ~40 lines changed |
| `smart_pointer.rs` | Delete 14 detection functions (see §6.2), keep `SmartPointerType` enum | ~250 lines deleted |

### 6.2 Functions to Delete from smart_pointer.rs

These 14 functions use string matching on AST paths/method names. All are replaced by reading `operation` from analyzer output.

| # | Function | Line | Patterns It Covers | Replacement |
|---|----------|------|--------------------|-------------|
| 1 | `detect_smart_pointer_new()` | 156 | IDs 1–5 | `initializer_kind` field already in type-info.json |
| 2 | `detect_rc_clone()` | 189 | IDs 6–7 | `initializer_kind == "rc_clone"` / `"arc_clone"` |
| 3 | `detect_refcell_borrow()` | 207 | IDs 9–10 | `initializer_kind == "refcell_borrow"` / `"refcell_borrow_mut"` |
| 4 | `detect_cell_operation()` | 222 | ID 11 | `method_calls[].operation` ending `::Cell::set` |
| 5 | `detect_box_pin()` | 237 | ID 15 | `initializer_kind == "pin_new"` |
| 6 | `detect_box_raw_op()` | 248 | IDs 13–14 | `initializer_kind == "box_new"` / raw_ptr type |
| 7 | `detect_pin_operation()` | 264 | ID 16 | `method_calls[].operation` ending `::Pin::as_ref` |
| 8 | `detect_cow_creation()` | 280 | IDs 17–18 | `initializer_kind == "cow_new"` / `"cow_variant"` |
| 9 | `detect_cow_to_mut()` | 296 | ID 19 | `method_calls[].operation` ending `::Cow::to_mut` |
| 10 | `detect_downgrade()` | 304 | ID 21 | `initializer_kind == "weak_downgrade"` |
| 11 | `detect_weak_upgrade()` | 320 | ID 22 | `initializer_kind == "weak_upgrade"` |
| 12 | `detect_once_cell_new()` | 328 | IDs 23–24 | `initializer_kind == "once_cell_new"` / `"once_lock_new"` |
| 13 | `detect_once_cell_method()` | 344 | IDs 25–27 | `method_calls[].operation` |
| 14 | `detect_maybe_uninit_new()` | 358 | IDs 28–29 | `initializer_kind == "maybe_uninit_new"` |

Also delete from smart_pointer.rs:
- `detect_maybe_uninit_method()` (line 374) — replaced by method_calls lookup
- `detect_concurrency_op()` (line 389) — replaced by method_calls lookup

Also delete from transform_visitor.rs:
- `infer_self_borrow_type()` (line 462) — replaced by `lookup_method_call_borrow()` with fallback

**Keep**: `SmartPointerType` enum, `SmartPointerOp` enum, `ConcurrencyOp` enum, `is_smart_pointer_operation()` — these are still used for the fallback path when no analyzer data is available.

### 6.3 New Lookup Logic

#### type_info.rs additions

```rust
impl TypeInfoCache {
    /// Lookup a method call on a variable by function context + variable name + method + line
    pub fn lookup_method_call(
        &self, fn_name: &str, var_name: &str, method: &str, line: u32,
    ) -> Option<&MethodCallInfoCompact> {
        let var = self.lookup_in_function(fn_name, var_name, None)?;
        var.method_calls.iter()
            .find(|mc| mc.method == method && mc.line == line)
    }

    /// Lookup a standalone expression by function context + line
    pub fn lookup_expression(
        &self, fn_name: &str, line: u32,
    ) -> Option<&ExpressionInfoCompact> {
        // Expressions are stored at file level, not per-variable
        // Search all variables in the function for the matching line
        let fn_vars = self.by_function.get(fn_name)?;
        for entries in fn_vars.values() {
            for var in entries {
                if let Some(expr) = var.expressions.iter().find(|e| e.line == line) {
                    return Some(expr);
                }
            }
        }
        None
    }
}
```

#### transform_visitor.rs — transform_method_call() refactored

```rust
fn transform_method_call(&mut self, method_call: &mut ExprMethodCall) {
    if !Self::is_simple_variable(&method_call.receiver) {
        self.visit_expr_mut(&mut method_call.receiver);
        for arg in &mut method_call.args { self.visit_expr_mut(arg); }
        return;
    }

    let method_name = method_call.method.to_string();
    let receiver_name = Self::extract_receiver_name(&method_call.receiver);

    // Try semantic lookup first
    let semantic_borrow = receiver_name.as_ref().and_then(|name| {
        let cache = self.type_info_cache.as_ref()?;
        let mc = cache.lookup_method_call(&self.current_fn_name, name, &method_name, self.current_line)?;
        mc.self_borrow.as_deref().map(|sb| match sb {
            "mutable" => SelfBorrowType::Mutable,
            "consuming" => SelfBorrowType::Consuming,
            _ => SelfBorrowType::Immutable,
        })
    });

    let borrow_type = semantic_borrow
        .unwrap_or_else(|| Self::infer_self_borrow_type(&method_name));

    // Guard methods — skip wrapping (lifetime issue)
    let guard_methods = [
        "lock", "try_lock", "read", "try_read", "write", "try_write",
        "borrow", "borrow_mut", "get_mut",
    ];
    if guard_methods.contains(&method_name.as_str()) {
        self.visit_expr_mut(&mut method_call.receiver);
        for arg in &mut method_call.args { self.visit_expr_mut(arg); }
        return;
    }

    // Consuming — visit normally
    if borrow_type == SelfBorrowType::Consuming {
        self.visit_expr_mut(&mut method_call.receiver);
        for arg in &mut method_call.args { self.visit_expr_mut(arg); }
        return;
    }

    // Wrap receiver with tracking
    if let Some(receiver_name) = receiver_name {
        if self.ref_vars.contains(&receiver_name) {
            for arg in &mut method_call.args { self.visit_expr_mut(arg); }
            return;
        }
        let receiver_expr = method_call.receiver.clone();
        method_call.receiver = Box::new(match borrow_type {
            SelfBorrowType::Immutable => syn::parse_quote! {
                borrowscope_runtime::track_borrow("method_borrow", &#receiver_expr)
            },
            SelfBorrowType::Mutable => syn::parse_quote! {
                borrowscope_runtime::track_borrow_mut("method_borrow", &mut #receiver_expr)
            },
            SelfBorrowType::Consuming => unreachable!(),
        });
    }

    for arg in &mut method_call.args { self.visit_expr_mut(arg); }
}
```

The key change is 3 lines: the `semantic_borrow` lookup and the `unwrap_or_else` fallback. Everything else stays the same.

---

## 7. Testing Strategy

### 7.1 Per-Phase Test Cases

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

### 7.2 Regression Tests

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

### 7.3 Success Criteria

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

