# Async Ownership

A comprehensive example demonstrating **all** BorrowScope runtime tracking features, including async/await patterns.

## Features Demonstrated

| Category | Functions | Description |
|----------|-----------|-------------|
| **Basic Ownership** | `track_new`, `track_move`, `track_drop` | Variable creation, ownership transfer, destruction |
| **Borrowing** | `track_borrow`, `track_borrow_mut` | Immutable and mutable references |
| **Rc (Single-threaded)** | `track_rc_new`, `track_rc_clone` | Reference counting with strong/weak counts |
| **Arc (Thread-safe)** | `track_arc_new`, `track_arc_clone` | Atomic reference counting for concurrency |
| **RefCell** | `track_refcell_new`, `refcell_borrow!`, `refcell_borrow_mut!`, `refcell_drop!` | Runtime borrow checking |
| **Cell** | `track_cell_new`, `track_cell_get`, `track_cell_set` | Copy-type interior mutability |
| **Static** | `track_static_init`, `track_static_access` | Global mutable state tracking |
| **Const** | `track_const_eval` | Compile-time constant evaluation |
| **Raw Pointers** | `track_raw_ptr`, `track_raw_ptr_mut`, `track_raw_ptr_deref` | Unsafe pointer operations |
| **Unsafe Blocks** | `track_unsafe_block_enter`, `track_unsafe_block_exit` | Unsafe scope boundaries |
| **Unsafe Functions** | `track_unsafe_fn_call` | Unsafe function invocations |
| **FFI** | `track_ffi_call` | Foreign function interface calls |
| **Transmute** | `track_transmute` | Type reinterpretation |
| **Unions** | `track_union_field_access` | Union field access |
| **Async** | Arc sharing across `tokio::spawn` | Ownership across await points |

## Run

```bash
cargo run
```

## Output Structure

The example runs 7 demo sections:

1. **Basic Ownership** - Create, move, drop a String
2. **Borrowing** - Multiple immutable borrows, then mutable borrow
3. **Smart Pointers** - Rc and Arc with clone tracking
4. **Interior Mutability** - RefCell and Cell operations
5. **Static and Const** - Global variable access patterns
6. **Unsafe Operations** - Raw pointers, transmute, unions, FFI
7. **Async Ownership** - Arc shared across spawned tasks

## Sample Event Output

```
New { timestamp: 0, var_name: "x", var_id: "x_0", type_name: "String" }
Move { timestamp: 1, from_id: "x", to_name: "y", to_id: "y_1" }
RcClone { timestamp: 12, var_name: "rc2", source_id: "rc1", strong_count: 2 }
RefCellBorrow { borrow_id: "ref_imm", is_mutable: false, location: "src/main.rs:117" }
RawPtrCreated { var_name: "ptr", ptr_type: "*const i32", address: 140734755963732 }
UnsafeBlockEnter { block_id: "0", location: "main.rs:unsafe_start" }
Transmute { from_type: "i32", to_type: "[u8; 4]" }
ArcClone { var_name: "task_data", source_id: "shared_data", strong_count: 2 }
```

## Event Types Captured

The example captures **53 events** across all categories:

| Event Type | Count | Description |
|------------|-------|-------------|
| New | 3 | Variable creation |
| Move | 1 | Ownership transfer |
| Drop | 17 | Variable destruction |
| Borrow | 5 | Immutable borrows |
| RcNew/RcClone | 2 | Rc operations |
| ArcNew/ArcClone | 4 | Arc operations |
| RefCellNew/Borrow/Drop | 5 | RefCell operations |
| CellNew/Get/Set | 3 | Cell operations |
| StaticInit/Access | 3 | Static variable ops |
| ConstEval | 1 | Const evaluation |
| RawPtrCreated/Deref | 4 | Raw pointer ops |
| UnsafeBlock Enter/Exit | 2 | Unsafe scope |
| UnsafeFnCall | 1 | Unsafe function |
| FfiCall | 1 | FFI call |
| Transmute | 1 | Type transmute |
| UnionFieldAccess | 1 | Union access |

## Exported JSON

Tracking data is exported to `/tmp/async-ownership.json` for visualization or analysis with `borrowscope-graph`.
