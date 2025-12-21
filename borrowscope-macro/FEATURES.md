# BorrowScope Macro Features Reference

This document lists all features implemented in `borrowscope-macro` and their corresponding runtime dependencies.

## Configuration Options

The `#[trace_borrow]` attribute supports these configuration modes:

| Attribute | Description |
|-----------|-------------|
| `#[trace_borrow]` | Default: all standard tracking enabled |
| `#[trace_borrow(quiet)]` | Ownership only (new, move, drop, borrow) |
| `#[trace_borrow(verbose)]` | All tracking including noisy features |
| `#[trace_borrow(skip = "loops,branches")]` | Skip specific feature groups |
| `#[trace_borrow(only = "ownership")]` | Enable only specified feature groups |

### Feature Groups

| Group | Config Field | Features Included |
|-------|--------------|-------------------|
| `ownership` | `track_new`, `track_move`, `track_drop`, `track_borrow` | Variable creation, moves, drops, borrows |
| `smart_pointers` | `track_smart_pointers` | Rc, Arc, RefCell, Cell operations |
| `loops` | `track_loops` | for, while, loop tracking |
| `branches` | `track_branches` | if/else, match tracking |
| `control_flow` | `track_control_flow` | break, continue, return |
| `try` | `track_try` | ? operator |
| `methods` | `track_methods` | clone, lock, unwrap |
| `async` | `track_async` | async blocks, await |
| `unsafe` | `track_unsafe` | unsafe blocks, raw pointers, transmute |
| `expressions` | `track_expressions` | struct, tuple, array, range, cast |
| `functions` | `track_functions` | Function entry/exit (disabled by default) |

---

## Implemented Transformations

### Phase 1: Basic Ownership

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `let x = value;` | `track_new(name, value)` | `New` | Tracks variable creation with type info |
| `let y = &x;` | `track_borrow(name, ref)` | `Borrow` | Tracks immutable borrow creation |
| `let y = &mut x;` | `track_borrow_mut(name, ref)` | `Borrow` | Tracks mutable borrow creation |
| `let y = x;` (move) | `track_move(from, to, value)` | `Move` | Tracks ownership transfer between variables |
| Scope exit | `track_drop(name)` | `Drop` | Tracks variable going out of scope |

### Phase 1: Interior Mutability

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `RefCell::new(v)` | `track_refcell_new(name, refcell)` | `RefCellNew` | Tracks RefCell creation |
| `refcell.borrow()` | `track_refcell_borrow(name, guard)` | `RefCellBorrow` | Tracks RefCell immutable borrow |
| `refcell.borrow_mut()` | `track_refcell_borrow_mut(name, guard)` | `RefCellBorrow` | Tracks RefCell mutable borrow |
| `Cell::new(v)` | `track_cell_new(name, cell)` | `CellNew` | Tracks Cell creation |
| `cell.get()` | `track_cell_get(name, value)` | `CellGet` | Tracks Cell value read |
| `cell.set(v)` | `track_cell_set(name)` | `CellSet` | Tracks Cell value write |

### Phase 2: Unsafe Operations

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `unsafe { ... }` | `track_unsafe_block_enter/exit(id, loc)` | `UnsafeBlockEnter/Exit` | Tracks entering/exiting unsafe blocks |
| `&raw const x` / `&raw mut x` | `track_raw_ptr(name, ptr)` | `RawPtrCreated` | Tracks raw pointer creation |
| `*ptr` (in unsafe) | `track_raw_ptr_deref(name)` | `RawPtrDeref` | Tracks raw pointer dereference |
| `std::mem::transmute(v)` | `track_transmute(name, from, to)` | `Transmute` | Tracks type transmutation |

### Phase 4: Async Operations

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `async { ... }` | `track_async_block_enter/exit(id, loc)` | `AsyncBlockEnter/Exit` | Tracks async block boundaries |
| `expr.await` | `track_await_start/end(id, future, loc)` | `AwaitStart/End` | Tracks await point suspension/resumption |

### Phase 5: Control Flow

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `for x in iter { }` | `track_loop_enter/exit/iteration(id, type, loc)` | `LoopEnter/Exit/Iteration` | Tracks for loop with iteration count |
| `while cond { }` | `track_loop_enter/exit/iteration(id, type, loc)` | `LoopEnter/Exit/Iteration` | Tracks while loop with iteration count |
| `loop { }` | `track_loop_enter/exit/iteration(id, type, loc)` | `LoopEnter/Exit/Iteration` | Tracks infinite loop with iteration count |
| `match expr { }` | `track_match_enter/arm/exit(id, arm, pattern, loc)` | `MatchEnter/Arm/Exit` | Tracks match expression and which arm taken |
| `if cond { } else { }` | `track_branch(id, type, loc)` | `Branch` | Tracks which branch of if/else taken |
| `return expr;` | `track_return(id, has_value, loc)` | `Return` | Tracks return statements |
| `expr?` | `track_try(id, loc)` | `Try` | Tracks ? operator usage |

### Phase 5: Method Calls

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `.clone()` | `track_clone(id, var, loc)` | `Clone` | Tracks clone method calls |
| `.lock()` / `.read()` / `.write()` | `track_lock(id, type, var, loc)` | `Lock` | Tracks mutex/rwlock acquisitions |
| `.unwrap()` / `.expect()` | `track_unwrap(id, method, var, loc)` | `Unwrap` | Tracks Option/Result unwrapping |

### Phase 6: Additional Expressions

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `break;` / `break 'label;` | `track_break(id, label, loc)` | `Break` | Tracks break statements with optional label |
| `continue;` / `continue 'label;` | `track_continue(id, label, loc)` | `Continue` | Tracks continue statements with optional label |
| `|| expr` / `move || expr` | `track_closure_create(id, mode, loc)` | `ClosureCreate` | Tracks closure creation with capture mode |
| `Point { x, y }` | `track_struct_create(id, type, loc)` | `StructCreate` | Tracks struct literal construction |
| `(a, b, c)` | `track_tuple_create(id, len, loc)` | `TupleCreate` | Tracks tuple construction |
| `0..10` / `0..=10` | `track_range(id, type, loc)` | `Range` | Tracks range expression creation |
| `[1, 2, 3]` | `track_array_create(id, len, loc)` | `ArrayCreate` | Tracks array literal creation |
| `x as i64` | `track_type_cast(id, to_type, loc)` | `TypeCast` | Tracks non-pointer type casts |

### Phase 7: Configuration

No runtime changes. Macro-only feature for attribute parameter parsing.

### Phase 8: Function Tracking

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| Function entry | `track_fn_enter(id, name, loc)` | `FnEnter` | Tracks function entry point |
| Function exit | `track_fn_exit(id, name, loc)` | `FnExit` | Tracks function exit point |
| Closure capture | `track_closure_capture(id, var, mode, loc)` | `ClosureCapture` | Tracks individual variable captures in closures |

---

## Smart Pointer Tracking

| Macro Transformation | Runtime Function | Event Type | Description |
|---------------------|------------------|------------|-------------|
| `Rc::new(v)` | `track_rc_new(name, rc)` | `RcNew` | Tracks Rc creation with strong/weak counts |
| `Rc::clone(&rc)` | `track_rc_clone(name, source, rc)` | `RcClone` | Tracks Rc clone with updated counts |
| `Arc::new(v)` | `track_arc_new(name, arc)` | `ArcNew` | Tracks Arc creation with strong/weak counts |
| `Arc::clone(&arc)` | `track_arc_clone(name, source, arc)` | `ArcClone` | Tracks Arc clone with updated counts |

---

## Runtime Event Summary

| Event Category | Events | Count |
|----------------|--------|-------|
| Basic Ownership | `New`, `Borrow`, `Move`, `Drop` | 4 |
| Smart Pointers | `RcNew`, `RcClone`, `ArcNew`, `ArcClone` | 4 |
| Interior Mutability | `RefCellNew`, `RefCellBorrow`, `RefCellDrop`, `CellNew`, `CellGet`, `CellSet` | 6 |
| Unsafe | `RawPtrCreated`, `RawPtrDeref`, `UnsafeBlockEnter`, `UnsafeBlockExit`, `UnsafeFnCall`, `Transmute`, `UnionFieldAccess`, `FfiCall` | 8 |
| Async | `AsyncBlockEnter`, `AsyncBlockExit`, `AwaitStart`, `AwaitEnd` | 4 |
| Loops | `LoopEnter`, `LoopIteration`, `LoopExit` | 3 |
| Branches | `MatchEnter`, `MatchArm`, `MatchExit`, `Branch` | 4 |
| Control Flow | `Return`, `Break`, `Continue` | 3 |
| Try | `Try` | 1 |
| Methods | `Clone`, `Lock`, `Unwrap`, `Deref`, `Call`, `IndexAccess`, `FieldAccess` | 7 |
| Expressions | `ClosureCreate`, `StructCreate`, `TupleCreate`, `Range`, `BinaryOp`, `ArrayCreate`, `TypeCast`, `LetElse` | 8 |
| Regions | `RegionEnter`, `RegionExit` | 2 |
| Functions | `FnEnter`, `FnExit`, `ClosureCapture` | 3 |
| Static/Const | `StaticInit`, `StaticAccess`, `ConstEval` | 3 |
| **Total** | | **60** |

---

## Usage Example

```rust
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn example() {
    let x = String::from("hello");  // New event
    let y = &x;                      // Borrow event
    let z = x;                       // Move event
}                                    // Drop events

fn main() {
    reset();
    example();
    let events = get_events();
    println!("{}", serde_json::to_string_pretty(&events).unwrap());
}
```

```rust
// With function tracking enabled
#[trace_borrow(only = "ownership,functions")]
fn tracked_fn(x: i32) -> i32 {
    // FnEnter event
    let y = x + 1;  // New event
    y               // FnExit event
}
```
