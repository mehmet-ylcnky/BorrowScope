# Syntactic/Heuristic Detection Patterns in borrowscope-macro

This document lists all patterns detected using string matching and syntactic analysis (AST-based) in the macro, and indicates whether each pattern is covered semantically by `borrowscope-analyzer`.

**Legend:**
- ✅ = Covered semantically by analyzer (no heuristics needed)
- ❌ = Not covered by analyzer (requires syntactic detection)
- ⚠️ = Partially covered (type detected, but not the specific operation)

---

## 1. Smart Pointer Creation (`smart_pointer.rs` - `detect_smart_pointer_new`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Box::new(...)` | `path_str.contains("Box::new")` | ✅ `box_new` |
| `Rc::new(...)` | `path_str.contains("Rc::new")` | ✅ `rc_new` |
| `Arc::new(...)` | `path_str.contains("Arc::new")` | ✅ `arc_new` |
| `RefCell::new(...)` | `path_str.contains("RefCell::new")` | ✅ `refcell_new` |
| `Cell::new(...)` | `path_str.contains("Cell::new")` | ✅ `cell_new` |

## 2. Smart Pointer Cloning (`smart_pointer.rs` - `detect_rc_clone`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Rc::clone(&x)` | `path_str.contains("Rc::clone")` | ✅ `rc_clone` |
| `Arc::clone(&x)` | `path_str.contains("Arc::clone")` | ✅ `arc_clone` |

## 3. RefCell/Cell Methods (`smart_pointer.rs` - `detect_refcell_borrow`, `detect_cell_op`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `.borrow()` | `method_name == "borrow"` | ✅ `refcell_borrow` (via guard type) |
| `.borrow_mut()` | `method_name == "borrow_mut"` | ✅ `refcell_borrow_mut` (via guard type) |
| `.get()` | `method_name == "get"` | ✅ `cell_get` (when result assigned) |
| `.set()` | `method_name == "set"` | ❌ No return value to track |

## 4. Box Operations (`smart_pointer.rs`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Box::pin(...)` | `path_str.contains("Box::pin")` | ✅ `pin_new` (via Pin type) |
| `Box::into_raw(...)` | `path_str.contains("Box::into_raw")` | ✅ `raw_ptr` (via ptr type) |
| `Box::from_raw(...)` | `path_str.contains("Box::from_raw")` | ✅ `box_new` (via Box type) |

## 5. Pin Operations (`smart_pointer.rs` - `detect_pin_operation`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Pin::new(...)` | `path_str.contains("Pin::new")` | ✅ `pin_new` |
| `Pin::into_inner(...)` | `path_str.contains("Pin::into_inner")` | ⚠️ Inner type detected |

## 6. Cow Operations (`smart_pointer.rs` - `detect_cow_creation`, `detect_cow_to_mut`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Cow::Borrowed(...)` | `path_str.contains("Cow::Borrowed")` | ✅ `cow_variant` |
| `Cow::Owned(...)` | `path_str.contains("Cow::Owned")` | ✅ `cow_variant` |
| `.to_mut()` | `method.to_string() == "to_mut"` | ❌ Mutates in place |

## 7. Weak Reference Operations (`smart_pointer.rs` - `detect_downgrade`, `detect_weak_upgrade`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `Rc::downgrade(&x)` | `path_str.contains("Rc::downgrade")` | ✅ `weak_downgrade` |
| `Arc::downgrade(&x)` | `path_str.contains("Arc::downgrade")` | ✅ `weak_downgrade` |
| `.upgrade()` | `method.to_string() == "upgrade"` | ✅ `weak_upgrade` (via Option type) |

## 8. OnceCell/OnceLock Operations (`smart_pointer.rs` - `detect_once_cell_new`, `detect_once_cell_method`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `OnceCell::new()` | `path_str.contains("OnceCell::new")` | ✅ `once_cell_new` |
| `OnceLock::new()` | `path_str.contains("OnceLock::new")` | ✅ `once_lock_new` |
| `.set(...)` | `method_name == "set"` | ❌ Returns Result, not tracked |
| `.get()` | `method_name == "get"` | ⚠️ Returns Option<&T> |
| `.get_or_init(...)` | `method_name == "get_or_init"` | ⚠️ Returns &T |

## 9. MaybeUninit Operations (`smart_pointer.rs` - `detect_maybe_uninit_new`, `detect_maybe_uninit_method`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `MaybeUninit::uninit()` | `path_str.contains("MaybeUninit::uninit")` | ✅ `maybe_uninit_new` |
| `MaybeUninit::new(...)` | `path_str.contains("MaybeUninit::new")` | ✅ `maybe_uninit_new` |
| `.write(...)` | `method_name == "write"` | ❌ Returns &mut T |
| `.assume_init()` | `method_name == "assume_init"` | ⚠️ Inner type detected |
| `.assume_init_read()` | `method_name == "assume_init_read"` | ⚠️ Inner type detected |
| `.assume_init_drop()` | `method_name == "assume_init_drop"` | ❌ Returns () |

## 10. Concurrency Operations (`smart_pointer.rs` - `detect_concurrency_op`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `thread::spawn(...)` | `path_str.contains("thread::spawn")` | ❌ Not tracked |
| `std::thread::spawn(...)` | `path_str.contains("std::thread::spawn")` | ❌ Not tracked |
| `mpsc::channel()` | `path_str.contains("mpsc::channel")` | ✅ `channel_sender`/`channel_receiver` (tuple) |
| `sync::mpsc::channel()` | `path_str.contains("sync::mpsc::channel")` | ✅ `channel_sender`/`channel_receiver` |
| `.join()` | `method_name == "join"` | ❌ Method on JoinHandle |
| `.send(...)` | `method_name == "send"` | ❌ Method on Sender |
| `.recv()` | `method_name == "recv"` | ❌ Method on Receiver |
| `.try_recv()` | `method_name == "try_recv"` | ❌ Method on Receiver |
| `.lock()` | `method_name == "lock"` | ✅ `mutex_lock` (via guard type) |
| `.try_lock()` | `method_name == "try_lock"` | ✅ `mutex_try_lock` (via Result<Guard>) |
| `.read()` | `method_name == "read"` | ✅ `rwlock_read` (via guard type) |
| `.write()` | `method_name == "write"` | ✅ `rwlock_write` (via guard type) |

## 11. Guard Methods Skipped (`transform_visitor.rs`)

These methods return guards/borrows and are skipped to avoid lifetime issues:

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `.lock()` | `guard_methods.contains("lock")` | ✅ Guard type detected |
| `.try_lock()` | `guard_methods.contains("try_lock")` | ✅ Guard type detected |
| `.read()` | `guard_methods.contains("read")` | ✅ Guard type detected |
| `.try_read()` | `guard_methods.contains("try_read")` | ✅ Guard type detected |
| `.write()` | `guard_methods.contains("write")` | ✅ Guard type detected |
| `.try_write()` | `guard_methods.contains("try_write")` | ✅ Guard type detected |
| `.borrow()` | `guard_methods.contains("borrow")` | ✅ Guard type detected |
| `.borrow_mut()` | `guard_methods.contains("borrow_mut")` | ✅ Guard type detected |
| `.get_mut()` | `guard_methods.contains("get_mut")` | ⚠️ Returns &mut T |

## 12. Self Borrow Type Inference (`transform_visitor.rs` - `infer_self_borrow_type`)

### Immutable Borrow Patterns

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `as_*` methods | `method_name.starts_with("as_")` | ❌ Method calls not tracked |
| `to_*` methods | `method_name.starts_with("to_")` | ❌ Method calls not tracked |
| `is_*` methods | `method_name.starts_with("is_")` | ❌ Method calls not tracked |
| `get*` methods | `method_name.starts_with("get")` | ❌ Method calls not tracked |
| `len` | Exact match | ❌ Method calls not tracked |
| `capacity` | Exact match | ❌ Method calls not tracked |
| `iter` | Exact match | ❌ Method calls not tracked |
| `chars` | Exact match | ❌ Method calls not tracked |
| `bytes` | Exact match | ❌ Method calls not tracked |
| `lines` | Exact match | ❌ Method calls not tracked |
| `split` | Exact match | ❌ Method calls not tracked |
| `trim` | Exact match | ❌ Method calls not tracked |
| `contains` | Exact match | ❌ Method calls not tracked |
| `starts_with` | Exact match | ❌ Method calls not tracked |
| `ends_with` | Exact match | ❌ Method calls not tracked |
| `find` | Exact match | ❌ Method calls not tracked |
| `clone` | Exact match | ❌ Method calls not tracked |
| `first` | Exact match | ❌ Method calls not tracked |
| `last` | Exact match | ❌ Method calls not tracked |

### Mutable Borrow Patterns

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `push*` methods | `method_name.starts_with("push")` | ❌ Method calls not tracked |
| `pop*` methods | `method_name.starts_with("pop")` | ❌ Method calls not tracked |
| `insert*` methods | `method_name.starts_with("insert")` | ❌ Method calls not tracked |
| `remove*` methods | `method_name.starts_with("remove")` | ❌ Method calls not tracked |
| `append*` methods | `method_name.starts_with("append")` | ❌ Method calls not tracked |
| `add*` methods | `method_name.starts_with("add")` | ❌ Method calls not tracked |
| `set*` methods | `method_name.starts_with("set")` | ❌ Method calls not tracked |
| `update*` methods | `method_name.starts_with("update")` | ❌ Method calls not tracked |
| `modify*` methods | `method_name.starts_with("modify")` | ❌ Method calls not tracked |
| `clear` | Exact match | ❌ Method calls not tracked |
| `truncate` | Exact match | ❌ Method calls not tracked |
| `extend` | Exact match | ❌ Method calls not tracked |
| `drain` | Exact match | ❌ Method calls not tracked |
| `sort` | Exact match | ❌ Method calls not tracked |
| `reverse` | Exact match | ❌ Method calls not tracked |
| `dedup` | Exact match | ❌ Method calls not tracked |
| `retain` | Exact match | ❌ Method calls not tracked |
| `tick` | Exact match | ❌ Method calls not tracked |
| `recv` | Exact match | ❌ Method calls not tracked |
| `send` | Exact match | ❌ Method calls not tracked |
| `changed` | Exact match | ❌ Method calls not tracked |
| `wait` | Exact match | ❌ Method calls not tracked |
| `acquire` | Exact match | ❌ Method calls not tracked |
| `lock` | Exact match | ❌ Method calls not tracked |
| `write` | Exact match | ❌ Method calls not tracked |

### Consuming Patterns

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `into_*` methods | `method_name.starts_with("into_")` | ❌ Method calls not tracked |
| `unwrap` | Exact match | ❌ Method calls not tracked |
| `expect` | Exact match | ❌ Method calls not tracked |

## 13. Unwrap Methods (`transform_visitor.rs`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `.unwrap()` | `method_name == "unwrap"` | ❌ Method calls not tracked |
| `.expect(...)` | `method_name == "expect"` | ❌ Method calls not tracked |
| `.unwrap_or(...)` | `method_name == "unwrap_or"` | ❌ Method calls not tracked |
| `.unwrap_or_else(...)` | `method_name == "unwrap_or_else"` | ❌ Method calls not tracked |
| `.unwrap_or_default()` | `method_name == "unwrap_or_default"` | ❌ Method calls not tracked |

## 14. Clone Method (`transform_visitor.rs`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `.clone()` | `method_name == "clone"` | ⚠️ Result type detected, not the operation |

## 15. Transmute Detection (`transform_visitor.rs`)

| Pattern | Detection Method | Analyzer |
|---------|------------------|----------|
| `transmute(...)` | `path_str.contains("transmute")` | ❌ Not tracked |
| `std::mem::transmute(...)` | `fn_name.contains("transmute")` | ❌ Not tracked |

---

## Summary

| Category | Total | ✅ Covered | ⚠️ Partial | ❌ Not Covered |
|----------|-------|-----------|------------|----------------|
| Smart pointer creation | 5 | 5 | 0 | 0 |
| Smart pointer clone | 2 | 2 | 0 | 0 |
| RefCell/Cell methods | 4 | 3 | 0 | 1 |
| Box operations | 3 | 3 | 0 | 0 |
| Pin operations | 2 | 1 | 1 | 0 |
| Cow operations | 3 | 2 | 0 | 1 |
| Weak reference operations | 3 | 3 | 0 | 0 |
| OnceCell/OnceLock operations | 5 | 2 | 2 | 1 |
| MaybeUninit operations | 6 | 2 | 2 | 2 |
| Concurrency operations | 12 | 5 | 0 | 7 |
| Guard method patterns | 9 | 8 | 1 | 0 |
| Self borrow inference (immutable) | 19 | 0 | 0 | 19 |
| Self borrow inference (mutable) | 25 | 0 | 0 | 25 |
| Self borrow inference (consuming) | 3 | 0 | 0 | 3 |
| Unwrap methods | 5 | 0 | 0 | 5 |
| Clone method | 1 | 0 | 1 | 0 |
| Transmute detection | 2 | 0 | 0 | 2 |
| **TOTAL** | **109** | **36** | **7** | **66** |

## Key Insight

The analyzer tracks **variable initializers** (what type a variable has when created), not **method calls on existing variables**. This is why:

- ✅ **36 patterns** are fully covered - these create new variables with trackable types
- ⚠️ **7 patterns** are partially covered - the result type is detected but not the specific operation
- ❌ **66 patterns** are not covered - these are method calls that don't create new variables

To achieve 100% semantic coverage, the analyzer would need to track **all expressions**, not just `let` bindings. This would require:
1. Walking all method call expressions
2. Resolving the receiver type
3. Matching method names to operations
4. Emitting operation-specific classifications

This is a significant expansion of scope beyond initializer classification.
