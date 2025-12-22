# Changelog

## [0.1.2] - 2024-12-22

### Summary
Major feature release adding comprehensive tracking for extended smart pointers, concurrency primitives, and advanced memory operations. This release nearly doubles the number of tracking functions and event types.

### Statistics
- **+7,776 lines added**, -1,161 lines removed
- **33 files changed** in borrowscope-runtime
- **Tracking functions**: 41 → 70+ (71% increase)
- **Event types**: ~20 → 45+ (125% increase)
- **Tracking categories**: 5 → 14 (180% increase)
- **New examples**: 13 example files
- **New tests**: 10 test files with comprehensive coverage

---

### Added

#### New Smart Pointer Tracking

| Type | Functions | Events |
|------|-----------|--------|
| **Box** | `track_box_new`, `track_box_into_raw`, `track_box_from_raw` | `BoxNew`, `BoxIntoRaw`, `BoxFromRaw` |
| **Weak** | `track_weak_new`, `track_weak_new_sync`, `track_weak_upgrade`, `track_weak_upgrade_sync`, `track_weak_clone`, `track_weak_clone_sync` | `WeakNew`, `WeakClone`, `WeakUpgrade` |
| **Pin** | `track_pin_new`, `track_pin_into_inner` | `PinNew`, `PinIntoInner` |
| **Cow** | `track_cow_borrowed`, `track_cow_owned`, `track_cow_to_mut` | `CowBorrowed`, `CowOwned`, `CowToMut` |

#### New Concurrency Tracking

| Type | Functions | Events |
|------|-----------|--------|
| **Threads** | `track_thread_spawn`, `track_thread_join` | `ThreadSpawn`, `ThreadJoin` |
| **Channels** | `track_channel`, `track_channel_send`, `track_channel_recv`, `track_channel_try_recv` | `ChannelNew`, `ChannelSend`, `ChannelRecv` |
| **Lock Guards** | `track_lock_guard_acquire`, `track_lock_guard_drop` | `LockGuardNew`, `LockGuardDrop` |

#### New Interior Mutability Tracking

| Type | Functions | Events |
|------|-----------|--------|
| **OnceCell/OnceLock** | `track_once_cell_new`, `track_once_lock_new`, `track_once_cell_set`, `track_once_cell_get`, `track_once_cell_get_or_init` | `OnceCellNew`, `OnceCellSet`, `OnceCellGet`, `OnceCellGetOrInit` |
| **MaybeUninit** | `track_maybe_uninit_uninit`, `track_maybe_uninit_new`, `track_maybe_uninit_write`, `track_maybe_uninit_assume_init`, `track_maybe_uninit_assume_init_read`, `track_maybe_uninit_assume_init_drop` | `MaybeUninitNew`, `MaybeUninitWrite`, `MaybeUninitAssumeInit`, `MaybeUninitAssumeInitRead`, `MaybeUninitAssumeInitDrop` |

#### New Control Flow & Expression Tracking (Phase 5-6)

| Category | Functions |
|----------|-----------|
| **Async** | `track_async_block_enter`, `track_async_block_exit`, `track_await_start`, `track_await_end` |
| **Loops** | `track_loop_enter`, `track_loop_exit`, `track_loop_iteration` |
| **Branches** | `track_branch`, `track_match_enter`, `track_match_arm`, `track_match_exit` |
| **Control Flow** | `track_return`, `track_break`, `track_continue`, `track_try`, `track_let_else` |
| **Expressions** | `track_struct_create`, `track_tuple_create`, `track_array_create`, `track_range`, `track_binary_op`, `track_type_cast` |
| **Methods** | `track_clone`, `track_deref`, `track_unwrap`, `track_lock` |
| **Functions** | `track_fn_enter`, `track_fn_exit`, `track_call` |
| **Closures** | `track_closure_create`, `track_closure_capture` |
| **Access** | `track_index_access`, `track_field_access` |
| **Regions** | `track_region_enter`, `track_region_exit` |

#### New Event Helper Methods

Filter events by category using new helper methods on `Event`:
- `is_box()` - Box events
- `is_weak()` - Weak reference events
- `is_pin()` - Pin events
- `is_cow()` - Cow events
- `is_thread()` - Thread events
- `is_channel()` - Channel events
- `is_lock_guard()` - Lock guard events
- `is_once_cell()` - OnceCell/OnceLock events
- `is_maybe_uninit()` - MaybeUninit events

#### New Examples

- `box_tracking.rs` - Box heap allocation tracking
- `weak_references.rs` - Weak reference lifecycle
- `pin_tracking.rs` - Pin memory pinning
- `cow_tracking.rs` - Clone-on-write patterns
- `thread_tracking.rs` - Thread spawn/join
- `channel_tracking.rs` - MPSC channel operations
- `lock_guard_tracking.rs` - Mutex/RwLock guards
- `once_cell_tracking.rs` - Lazy initialization
- `maybe_uninit_tracking.rs` - Uninitialized memory
- `async_tracking.rs` - Async/await patterns
- `control_flow_tracking.rs` - Loops, branches, control flow
- `method_call_tracking.rs` - Method call tracking
- `phase6_tracking.rs` - Expression tracking

#### New Test Files

- `box_tracking_tests.rs` - 282 lines
- `weak_tracking_tests.rs` - 298 lines
- `pin_tracking_tests.rs` - 289 lines
- `cow_tracking_tests.rs` - 323 lines
- `thread_tracking_tests.rs` - 307 lines
- `channel_tracking_tests.rs` - 399 lines
- `lock_guard_tracking_tests.rs` - 289 lines
- `once_cell_tracking_tests.rs` - 218 lines
- `maybe_uninit_tracking_tests.rs` - 197 lines
- `async_tracking_tests.rs` - 228 lines

---

### Changed

#### Documentation
- Updated lib.rs with comprehensive documentation for all new features
- Expanded tracking categories table from 5 to 14 categories
- Added code examples for all new tracking categories
- Updated function count from 41 to 70+
- Added event filtering examples with helper methods

#### Event Module
- Extended `Event` enum with 25+ new variants
- Added `timestamp()` and `var_name()` methods for all new events
- Added category helper methods for filtering

---

### Comparison with v0.1.1 (Published on docs.rs)

| Feature | v0.1.1 | v0.1.2 |
|---------|--------|--------|
| Tracking functions | 41 | 70+ |
| Event types | ~20 | 45+ |
| Tracking categories | 5 | 14 |
| Smart pointer types | Rc, Arc, RefCell, Cell | + Box, Weak, Pin, Cow |
| Concurrency | None | Threads, Channels, Lock Guards |
| Lazy init | None | OnceCell, OnceLock |
| Unsafe memory | Raw pointers only | + MaybeUninit |
| Control flow | None | Loops, branches, returns |
| Expressions | None | Structs, tuples, arrays, casts |
| Async | None | Async blocks, await |
| Examples | 0 | 13 |
| Test files | Base tests | + 10 new test files |

---

### Migration Guide

No breaking changes. All existing code continues to work. New features are additive.

To use new features:
```rust
use borrowscope_runtime::*;

// Box tracking
let boxed = track_box_new("data", "loc", Box::new(42));

// Weak reference tracking
let weak = track_weak_new("weak", "rc", "loc", Rc::downgrade(&rc));

// Channel tracking
let (tx, rx) = mpsc::channel();
let (tx, rx) = track_channel("chan", "loc", tx, rx);

// OnceCell tracking
let cell: OnceCell<i32> = track_once_cell_new("config", "loc", OnceCell::new());

// MaybeUninit tracking
let uninit: MaybeUninit<i32> = track_maybe_uninit_uninit("data", "loc", MaybeUninit::uninit());

// Filter events by category
let events = get_events();
let box_events: Vec<_> = events.iter().filter(|e| e.is_box()).collect();
```
