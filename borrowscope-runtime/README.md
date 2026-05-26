# borrowscope-runtime

> Runtime tracking library for Rust ownership visualization - 93 event types, zero-cost when disabled

[![Crates.io](https://img.shields.io/crates/v/borrowscope-runtime.svg)](https://crates.io/crates/borrowscope-runtime)

## Overview

`borrowscope-runtime` captures ownership transfers, borrows, smart pointer operations, concurrency primitives, and unsafe code as they happen at runtime. It generates structured event data that can be exported to JSON for analysis, visualization in VS Code, or debugging.

**93 event types** covering the full Rust ownership model. Zero overhead without the `track` feature.

## Installation

```toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }
```

Without `features = ["track"]`, all tracking functions compile to no-ops.

## Quick Start

```rust
use borrowscope_runtime::*;

fn main() {
    reset();

    let data = track_new("data", vec![1, 2, 3]);
    let r = track_borrow("r", &data);
    println!("{:?}", r);
    track_drop("r");

    let moved = track_move("data", "moved", data);
    track_drop("moved");

    // Export
    let events = get_events();
    println!("{} events captured", events.len());
    export_json(".borrowscope/events.json").unwrap();
}
```

## Tracking Functions (88+ functions)

### Basic Ownership

| Function | Description |
|----------|-------------|
| `track_new(name, value)` | Track variable creation, returns value |
| `track_borrow(name, &value)` | Track shared borrow |
| `track_borrow_mut(name, &mut value)` | Track mutable borrow |
| `track_move(from, to, value)` | Track ownership transfer |
| `track_drop(name)` | Track variable going out of scope |
| `track_drop_batch(&[names])` | Track multiple drops efficiently |

### Smart Pointers

| Function | Description |
|----------|-------------|
| `track_rc_new(name, Rc::new(v))` | Track Rc creation |
| `track_rc_clone(name, source, rc.clone())` | Track Rc clone |
| `track_arc_new(name, Arc::new(v))` | Track Arc creation |
| `track_arc_clone(name, source, arc.clone())` | Track Arc clone |
| `track_weak_new(name, Rc::downgrade(&rc))` | Track Weak creation |
| `track_weak_upgrade(weak_id, loc, weak.upgrade())` | Track Weak upgrade |
| `track_box_new(name, Box::new(v))` | Track Box allocation |
| `track_box_into_raw(box_id, loc)` | Track Box::into_raw |
| `track_box_from_raw(name, id, loc)` | Track Box::from_raw |
| `track_pin_new(name, Pin::new(v))` | Track Pin creation |
| `track_cow_borrowed/owned/to_mut` | Track Cow operations |

### Interior Mutability

| Function | Description |
|----------|-------------|
| `track_refcell_new(name, RefCell::new(v))` | Track RefCell creation |
| `track_refcell_borrow(name, &refcell)` | Track RefCell::borrow |
| `track_refcell_borrow_mut(name, &refcell)` | Track RefCell::borrow_mut |
| `track_refcell_drop(borrow_id, loc)` | Track guard drop |
| `track_cell_new(name, Cell::new(v))` | Track Cell creation |
| `track_cell_get/set(cell_id, loc)` | Track Cell access |
| `track_once_cell_new/set/get/get_or_init` | Track OnceCell |
| `track_maybe_uninit_new/write/assume_init` | Track MaybeUninit |

### Unsafe Code

| Function | Description |
|----------|-------------|
| `track_raw_ptr(name, ptr)` | Track raw pointer creation |
| `track_raw_ptr_deref(ptr_id, loc)` | Track pointer dereference |
| `track_unsafe_block_enter/exit(id, loc)` | Track unsafe blocks |
| `track_unsafe_fn_call(name, loc)` | Track unsafe function calls |
| `track_ffi_call(name, loc)` | Track FFI calls |
| `track_transmute(from, to, loc)` | Track transmute |

### Concurrency

| Function | Description |
|----------|-------------|
| `track_thread_spawn/join(id, loc)` | Track thread lifecycle |
| `track_channel(id, loc, tx, rx)` | Track channel creation |
| `track_channel_send/recv(id, loc)` | Track channel operations |
| `track_lock(id, type, name, loc)` | Track lock acquisition |
| `track_lock_guard_acquire/drop(id, loc)` | Track guard lifecycle |

### Async/Await

| Function | Description |
|----------|-------------|
| `track_async_block_enter/exit(id, loc)` | Track async blocks |
| `track_await_start(id, future, loc)` | Track await start |
| `track_await_start_with_live_vars(...)` | Track with captured vars |
| `track_await_end(id, loc)` | Track await completion |

### Control Flow

| Function | Description |
|----------|-------------|
| `track_fn_enter/exit(id, name, loc)` | Track function boundaries |
| `track_loop_enter/iteration/exit` | Track loops |
| `track_match_enter/arm/exit` | Track match expressions |
| `track_branch(id, type, loc)` | Track if/else |
| `track_return/break/continue` | Track control flow |
| `track_try(id, loc)` | Track ? operator |
| `track_region_enter/exit(id, name, loc)` | Track scopes |

### Expressions

| Function | Description |
|----------|-------------|
| `track_struct_create(id, type, loc)` | Track struct instantiation |
| `track_tuple_create(id, len, loc)` | Track tuple creation |
| `track_array_create(id, len, loc)` | Track array creation |
| `track_closure_create/capture` | Track closures |
| `track_clone/deref/unwrap` | Track common operations |
| `track_index_access/field_access` | Track access patterns |
| `track_range/binary_op/type_cast` | Track expressions |

### Memory Layout

| Function | Description |
|----------|-------------|
| `track_stack_addr(name, &value)` | Record stack address |
| `track_string_layout(name, &string)` | Record String ptr/len/cap + heap |
| `track_vec_layout(name, &vec)` | Record Vec ptr/len/cap + heap |
| `track_heap_addr(owner, addr, size, cap, content)` | Record heap allocation |
| `track_heap_realloc(id, old, new, old_size, new_size)` | Record reallocation |
| `track_stack_padding(after, addr, bytes)` | Record alignment padding |
| `export_memory_json(path, fn_name)` | Export memory layout JSON |

### Sampling (Performance)

| Function | Description |
|----------|-------------|
| `should_sample(rate)` | Check if call should be tracked |
| `track_new_sampled(name, value, rate)` | Track with probability |
| `track_borrow_sampled(name, &v, rate)` | Borrow with probability |
| `track_move_sampled(from, to, v, rate)` | Move with probability |
| `track_drop_sampled(name, rate)` | Drop with probability |

### Query & Export

| Function | Description |
|----------|-------------|
| `get_events()` | Get all recorded events |
| `get_events_filtered(predicate)` | Filter events |
| `get_events_for_var(name)` | Events for one variable |
| `get_new_events/borrow_events/drop_events/move_events` | By category |
| `get_event_counts()` | (new, borrow, move, drop) tuple |
| `get_summary()` | Full TrackingSummary |
| `print_summary()` | Print to stdout |
| `reset()` | Clear all events |
| `export_json(path)` | Export graph + events to JSON |
| `export_memory_json(path, fn)` | Export memory layout |

## Event Types (93 total)

| Category | Count | Events |
|----------|-------|--------|
| Ownership | 4 | New, Borrow, Move, Drop |
| Smart Pointers | 11 | RcNew, RcClone, ArcNew, ArcClone, WeakNew, WeakClone, WeakUpgrade, BoxNew, BoxIntoRaw, BoxFromRaw, PinNew/IntoInner |
| Interior Mutability | 13 | RefCellNew/Borrow/Drop, CellNew/Get/Set, OnceCellNew/Set/Get/GetOrInit, MaybeUninit×5 |
| Unsafe | 7 | RawPtrCreated, RawPtrDeref, UnsafeBlockEnter/Exit, UnsafeFnCall, FfiCall, Transmute, UnionFieldAccess |
| Async | 4 | AsyncBlockEnter/Exit, AwaitStart, AwaitEnd |
| Control Flow | 15 | FnEnter/Exit, LoopEnter/Iteration/Exit, MatchEnter/Arm/Exit, Branch, Return, Try, Break, Continue, RegionEnter/Exit |
| Concurrency | 8 | ThreadSpawn/Join, ChannelSenderNew/ReceiverNew/Send/Recv, LockGuardAcquire/Drop |
| Expressions | 14 | Call, Clone, Deref, IndexAccess, FieldAccess, ClosureCreate/Capture, StructCreate, TupleCreate, ArrayCreate, LetElse, Range, BinaryOp, TypeCast |
| Memory | 5 | StackAddr, StackField, HeapAddr, HeapRealloc, StackPadding |
| Static | 3 | StaticInit, StaticAccess, ConstEval |
| Other | 9 | Lock, Unwrap, CowBorrowed/Owned/ToMut, OnceLockNew, PinIntoInner, Ordering, FmtArguments |

## Modules

```
src/
├── lib.rs              (573 lines)   Public API, re-exports, documentation
├── event.rs            (1343 lines)  Event enum (93 variants) + helper methods
├── tracker/
│   ├── mod.rs          (2106 lines)  Tracker struct, event recording, main API
│   ├── core.rs         (526 lines)   Core tracking (new, borrow, move, drop)
│   ├── smart_pointers.rs (543 lines) Rc, Arc, Weak, Box, Pin, Cow
│   ├── interior_mut.rs (348 lines)   RefCell, Cell, OnceCell, MaybeUninit
│   ├── control_flow.rs (242 lines)   Loops, match, branch, return, break
│   ├── expressions.rs  (243 lines)   Struct, tuple, array, closure, deref
│   ├── unsafe_code.rs  (226 lines)   Raw pointers, unsafe blocks, FFI
│   ├── memory.rs       (165 lines)   Stack/heap address tracking
│   ├── sampling.rs     (149 lines)   Probabilistic tracking (xorshift64)
│   ├── concurrency.rs  (95 lines)    Threads, channels, locks
│   ├── async_tracking.rs (91 lines)  Async blocks, await points
│   ├── maybe_uninit.rs (89 lines)    MaybeUninit operations
│   ├── statics.rs      (73 lines)    Static/const tracking
│   └── query.rs        (274 lines)   Event filtering and summary
├── graph.rs            (487 lines)   Ownership graph construction (petgraph)
├── lifetime.rs         (532 lines)   Lifetime analysis and timeline
├── export.rs           (278 lines)   JSON export (events + graph)
├── guard.rs            (306 lines)   RAII guards for automatic drop tracking
└── error.rs            (94 lines)    Error types
```

**Total: ~8,800 lines**

## Performance

| Metric | Value |
|--------|-------|
| Per-call overhead (with `track`) | ~75-80ns |
| Per-event memory | ~80 bytes |
| Without `track` feature | Zero (compiled away) |
| Sampling at 10% | ~8ns average |
| Thread safety | `parking_lot::Mutex` (fast) |

### Benchmarks

```bash
cargo bench -p borrowscope-runtime
```

Three benchmark suites:
- `performance` - core tracking call overhead
- `optimization` - sampling and batch operations
- `overhead_analysis` - real-world scenario overhead

## Testing

```bash
# Run all 775 tests
cargo test -p borrowscope-runtime --features track

# Run specific category
cargo test -p borrowscope-runtime --test rc_arc_integration_tests
cargo test -p borrowscope-runtime --test async_tracking_tests
cargo test -p borrowscope-runtime --test property_based_tests
```

32 test files covering:
- Core ownership (new, borrow, move, drop)
- Smart pointers (Rc, Arc, Weak, Box, Pin, Cow)
- Interior mutability (RefCell, Cell, OnceCell, MaybeUninit)
- Unsafe code, concurrency, async
- Performance edge cases
- Property-based testing (proptest + quickcheck)
- Sampling correctness

## Integration with VS Code

The extension reads events via two mechanisms:

1. **File-based:** `export_json(".borrowscope/events.json")` → extension watches file
2. **Memory layout:** `export_memory_json(".borrowscope/memory-events.json", "fn_name")` → Memory tab

Events use serde's internally-tagged format: `{"type": "New", "var_name": "x", ...}`

## License

Apache-2.0
