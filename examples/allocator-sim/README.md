# Allocator Simulator

A comprehensive example demonstrating all 38 tracking functions in `borrowscope-runtime` through a memory allocator simulation.

## Purpose

This example showcases how BorrowScope can track complex ownership patterns found in real-world systems programming:

- **Memory allocators** with raw pointer manipulation
- **Thread-safe pools** using Arc for concurrent access
- **Caches** with interior mutability (RefCell/Cell)
- **FFI boundaries** simulating C library calls
- **Unsafe code** including transmute and union access

## Tracking Functions Used

### Basic Ownership

| Function | Description |
|----------|-------------|
| `track_new(name, value)` | Track variable creation |
| `track_move(name, value)` | Track ownership transfer |
| `track_drop(name)` | Track value destruction |
| `track_borrow(name, ref)` | Track immutable borrow (`&T`) |
| `track_borrow_mut(name, ref)` | Track mutable borrow (`&mut T`) |

### Smart Pointers

| Function | Description |
|----------|-------------|
| `track_rc_new(name, rc)` | Track `Rc<T>` creation |
| `track_rc_clone(name, source, rc)` | Track `Rc::clone()` with source |
| `track_arc_new(name, arc)` | Track `Arc<T>` creation |
| `track_arc_clone(name, source, arc)` | Track `Arc::clone()` with source |

### Interior Mutability

| Function | Description |
|----------|-------------|
| `track_refcell_new(name, cell)` | Track `RefCell<T>` creation |
| `track_refcell_borrow(name, id, loc, guard)` | Track `RefCell::borrow()` |
| `track_refcell_borrow_mut(name, id, loc, guard)` | Track `RefCell::borrow_mut()` |
| `track_refcell_drop(name, id, loc)` | Track borrow guard drop |
| `track_cell_new(name, cell)` | Track `Cell<T>` creation |
| `track_cell_set(name, loc)` | Track `Cell::set()` |
| `track_cell_get(name, loc)` | Track `Cell::get()` |

### Unsafe Operations

| Function | Description |
|----------|-------------|
| `track_raw_ptr(name, id, type, loc, ptr)` | Track `*const T` creation |
| `track_raw_ptr_mut(name, id, type, loc, ptr)` | Track `*mut T` creation |
| `track_raw_ptr_deref(id, loc, is_write)` | Track pointer dereference |
| `track_unsafe_block_enter(id, loc)` | Track entering unsafe block |
| `track_unsafe_block_exit(id, loc)` | Track exiting unsafe block |
| `track_unsafe_fn_call(name, loc)` | Track unsafe function call |
| `track_transmute(from, to, loc)` | Track `std::mem::transmute` |
| `track_union_field_access(union, field, loc)` | Track union field access |

### FFI

| Function | Description |
|----------|-------------|
| `track_ffi_call(name, loc)` | Track foreign function call |

### Statics & Constants

| Function | Description |
|----------|-------------|
| `track_static_init(name, loc)` | Track static variable initialization |
| `track_static_access(name, loc, is_write)` | Track static variable access |
| `track_const_eval(name, loc)` | Track const evaluation |

## Scenarios Demonstrated

1. **Basic Allocation** - Box-like heap allocation with move semantics
2. **Borrowed Slices** - Multiple concurrent immutable borrows
3. **Shared Blocks (Rc)** - Reference-counted sharing with clone tracking
4. **Thread-Safe Pool (Arc)** - Atomic reference counting for threads
5. **Interior Mutable Cache** - RefCell + Cell for runtime borrowing
6. **Raw Memory Operations** - Pointer creation and dereferencing
7. **Unsafe Allocator** - Custom allocator with static counters
8. **FFI Interop** - Simulated C library calls (malloc, free, mmap)
9. **Type Punning** - Transmute between types
10. **Union Field Access** - Unsafe union field reads

## Running

```bash
cd examples/allocator-sim
cargo run
```

## Sample Output

```
╔══════════════════════════════════════════════════════════════╗
║        Allocator Simulator - Full Runtime Demo               ║
╚══════════════════════════════════════════════════════════════╝

Config: BLOCK_SIZE=64, POOL_SIZE=8

━━━ 1. Basic Allocation ━━━

  Memory Map:
  ┌────────┬────────┬────────┬────────┐
  │ block1 │ block2 │
  └────────┴────────┴────────┴────────┘
  block1 moved to moved_block
  ✓ Basic allocation complete

━━━ 2. Borrowed Slices ━━━

  Three concurrent read slices:
  ┌─────────────────────────────────────────────────────────┐
  │ 0        64       128      192      256                 │
  │ [slice1][slice2][slice3]                 │
  └─────────────────────────────────────────────────────────┘
  Mutable slice write: buffer[0] = 0xFF
  ✓ Borrowed slices complete

━━━ 3. Shared Blocks (Rc) ━━━

  shared created, count: 1
  Rc Sharing (count=4):
       ┌─────────┐
       │ shared  │◄─┬─ reader1
       │  block  │◄─┼─ reader2
       │         │◄─┴─ reader3
       └─────────┘
  reader3 dropped, count: 3
  ✓ Shared blocks complete

━━━ 4. Thread-Safe Pool (Arc) ━━━

  Pool created with 8 blocks
  Arc Pool (count=3):
       ┌─────────────────────┐
       │    Thread-Safe      │
       │       Pool          │
       └──────────┬──────────┘
            ┌─────┴─────┐
         thread1     thread2
  Thread1 reading pool: 8 blocks
  ✓ Thread-safe pool complete

━━━ 5. Interior Mutable Cache ━━━

  Cache populated: 2 entries
  Cache read: Some((0, [1, 2, 3]))
  Hit count: 2
  ✓ Interior mutable cache complete

━━━ 6. Raw Memory Operations ━━━

  Raw Pointers:
    *const → 0x00007FFE7111DBD0
    *mut   → 0x00007FFE7111DBD0
  Read via *const: 0
  Write via *mut: 0xAB
  ✓ Raw memory ops complete

━━━ 7. Unsafe Allocator ━━━

  Allocations: 1
  Frees: 1
  ✓ Unsafe allocator complete

━━━ 8. FFI Interop ━━━

  Simulating C allocator calls:
    → malloc(64) called
    → realloc(ptr, 128) called
    → free(ptr) called
    → mmap(NULL, 4096, ...) called
    → munmap(ptr, 4096) called
  ✓ FFI interop complete

━━━ 9. Type Punning ━━━

  [0x01, 0x02, 0x03, 0x04] → u32: 0x04030201
  Pointer → usize: 0x7FFE7111DC78
  ✓ Type punning complete

━━━ 10. Union Field Access ━━━

  header.size = 64
  header.flags = [64, 0, 0, 0, 0, 0, 0, 0]
  ✓ Union access complete

╔══════════════════════════════════════════════════════════════╗
║                      Event Summary                           ║
╠══════════════════════════════════════════════════════════════╣
║  Drop....................................    21 ║
║  New.....................................     7 ║
║  Borrow..................................     5 ║
║  UnsafeBlockEnter........................     5 ║
║  UnsafeBlockExit.........................     5 ║
║  FfiCall.................................     5 ║
║  StaticAccess............................     4 ║
║  RcClone.................................     3 ║
║  ConstEval...............................     2 ║
║  CellSet.................................     2 ║
║  RawPtrDeref.............................     2 ║
║  StaticInit..............................     2 ║
║  ArcClone................................     2 ║
║  Transmute...............................     2 ║
║  RefCellBorrow...........................     2 ║
║  RawPtrCreated...........................     2 ║
║  UnsafeFnCall............................     2 ║
║  UnionFieldAccess........................     2 ║
║  RefCellDrop.............................     2 ║
║  RcNew...................................     1 ║
║  CellNew.................................     1 ║
║  CellGet.................................     1 ║
║  Move....................................     1 ║
║  ArcNew..................................     1 ║
║  RefCellNew..............................     1 ║
╠══════════════════════════════════════════════════════════════╣
║  TOTAL EVENTS                                         83 ║
╚══════════════════════════════════════════════════════════════╝

Exported to: /tmp/allocator-sim.json
```
