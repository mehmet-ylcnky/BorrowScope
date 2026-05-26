# borrowscope-macro

> `#[trace_borrow]` - automatic instrumentation of Rust code for ownership tracking

## Overview

`borrowscope-macro` is a procedural macro that automatically transforms Rust functions to emit runtime tracking calls. It reads semantic type information from `borrowscope-analyzer` and generates precise `borrowscope-runtime` calls for every ownership operation - variable creation, borrows, moves, drops, smart pointer operations, control flow, and more.

**133 instrumentation points** covering the full Rust ownership model.

## Quick Start

```rust
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn example() {
    let data = vec![1, 2, 3];   // → track_new("data", vec![...])
    let r = &data;               // → track_borrow("r", &data)
    let moved = data;            // → track_move("data", "moved", data)
}                                // → track_drop("moved"), track_drop("r")

fn main() {
    reset();
    example();
    println!("{} events", get_events().len());
}
```

## Attribute Options

### Presets

| Attribute | Description |
|-----------|-------------|
| `#[trace_borrow]` | Standard tracking (recommended) |
| `#[trace_borrow(quiet)]` | Ownership only (new, move, drop, borrow) |
| `#[trace_borrow(verbose)]` | All tracking including control flow |

### Feature Selection

```rust
#[trace_borrow(skip = "loops,branches")]   // Skip specific groups
#[trace_borrow(only = "ownership")]        // Only specific groups
```

### Filtering & Sampling

```rust
#[trace_borrow(filter = "data*")]   // Only track vars matching glob
#[trace_borrow(sample = 0.1)]       // Track ~10% of operations
```

### Conditional Compilation

```rust
#[trace_borrow(debug_only)]           // Only in debug builds
#[trace_borrow(release_only)]         // Only in release builds
#[trace_borrow(feature = "tracing")]  // Only when feature enabled
```

## Feature Groups

| Group | Config Fields | What It Tracks |
|-------|--------------|----------------|
| `ownership` | track_new, track_move, track_drop, track_borrow | Variable creation, moves, drops, borrows |
| `smart_pointers` | track_smart_pointers | Rc/Arc new/clone, RefCell borrow/borrow_mut, Cell get/set, Weak |
| `loops` | track_loops | for/while/loop enter, iteration count, exit |
| `branches` | track_branches | if/else, match arm selection |
| `control_flow` | track_control_flow | break, continue, return |
| `try` | track_try | ? operator |
| `methods` | track_methods | clone, lock, unwrap |
| `async` | track_async | async blocks, await with live variables |
| `unsafe` | track_unsafe | unsafe blocks, raw pointer ops, transmute, FFI |
| `expressions` | track_expressions | struct/tuple/array/range creation, type casts |
| `functions` | track_functions | Function entry/exit (disabled by default) |

## What Gets Instrumented (133 points)

### Ownership (core)
| Source Code | Generated Tracking |
|---|---|
| `let x = value;` | `track_new_with_id(id, "x", location, value)` |
| `let y = x;` (move) | `track_move_with_id(src_id, dst_id, "y", location, x)` |
| `let y = x;` (copy) | `track_new_with_id(id, "y", location, x)` |
| `let r = &x;` | `track_borrow_with_id(id, owner_id, "borrow", location, false, &x)` |
| `let m = &mut x;` | `track_borrow_mut_with_id(id, owner_id, "borrow", location, &mut x)` |
| `}` (scope end) | `track_drop("x")` (LIFO order) |

### Smart Pointers
| Source Code | Generated Tracking |
|---|---|
| `Rc::new(v)` | `track_rc_new_with_id(id, "rc", "Rc<T>", location, Rc::new(v))` |
| `rc.clone()` | `track_rc_clone_with_id(id, src_id, "rc2", location, rc.clone())` |
| `Arc::new(v)` | `track_arc_new_with_id(...)` |
| `RefCell::new(v)` | `track_refcell_new("cell", RefCell::new(v))` |
| `Cell::new(v)` | `track_cell_new("cell", Cell::new(v))` |
| `Weak::downgrade(&rc)` | `track_weak_new(...)` |
| `weak.upgrade()` | `track_weak_upgrade(...)` |

### Control Flow
| Source Code | Generated Tracking |
|---|---|
| `for x in iter { }` | `track_loop_enter/iteration/exit` |
| `while cond { }` | `track_loop_enter/iteration/exit` |
| `if cond { } else { }` | `track_branch(id, "then"/"else", location)` |
| `match x { arm => }` | `track_match_enter/arm/exit` |
| `break` | `track_break(id, label, location)` |
| `continue` | `track_continue(id, label, location)` |
| `return val` | `track_return(id, has_value, location)` |
| `expr?` | `track_try(id, location)` |

### Async
| Source Code | Generated Tracking |
|---|---|
| `async { }` | `track_async_block_enter/exit` |
| `future.await` | `track_await_start_with_live_vars/track_await_end` |

### Unsafe
| Source Code | Generated Tracking |
|---|---|
| `unsafe { }` | `track_unsafe_block_enter_enriched/exit` |
| `*raw_ptr` | (tracked via unsafe block context) |
| `transmute(x)` | (tracked via expression analysis) |

### Expressions
| Source Code | Generated Tracking |
|---|---|
| `MyStruct { field }` | `track_struct_create(id, "MyStruct", location)` |
| `(a, b, c)` | `track_tuple_create(id, 3, location)` |
| `[1, 2, 3]` | `track_array_create(id, 3, location)` |
| `0..10` | `track_range(id, "Range", location)` |
| `x as u64` | `track_type_cast(id, "u64", location)` |
| `closure` | `track_closure_create + track_closure_capture` |

### Functions & Methods
| Source Code | Generated Tracking |
|---|---|
| `fn foo() { }` | `track_fn_enter/exit` (when enabled) |
| `x.clone()` | `track_clone(id, "x", location)` |
| `mutex.lock()` | `track_lock(id, "Mutex", "m", location)` |
| `opt.unwrap()` | `track_unwrap(id, "unwrap", "opt", location)` |
| `*deref_expr` | `track_deref(id, "x", location)` |
| `arr[i]` | `track_index_access(id, "arr", location)` |
| `obj.field` | `track_field_access(id, "obj", "field", location)` |

### Channels
| Source Code | Generated Tracking |
|---|---|
| `mpsc::channel()` | `track_channel(id, location, tx, rx)` |

## How It Works

```
1. borrowscope-analyzer runs → generates .borrowscope/type-info.json
2. #[trace_borrow] reads type-info.json at compile time (via OnceLock cache)
3. For each variable, looks up: is_copy? is_rc? is_move? initializer_kind?
4. Transforms the AST to wrap expressions with appropriate track_*() calls
5. Emits the transformed function with tracking code
```

### Type Info Lookup

The macro uses `type-info.json` to make semantic decisions:

- **Copy vs Move:** If `is_copy == true`, emit `track_new` instead of `track_move`
- **Rc/Arc detection:** If `is_rc == true`, emit `track_rc_new` instead of `track_new`
- **Initializer classification:** Uses `initializer_kind` to choose the right tracking call
- **Drop points:** Uses `drop_line` from analyzer for accurate scope-end drops
- **Closure captures:** Uses `closure_captures` for `track_closure_capture` calls

### Stable Rust Compatibility

On stable Rust, `proc_macro::Span` doesn't expose file/line info. The macro uses variable name lookup (`by_name` index) instead of span-based lookup. When names are ambiguous, it falls back to `by_function` index.

## Code Structure

```
src/
├── lib.rs                  (724 lines)   Macro entry point, attribute parsing
├── transform_visitor.rs    (4104 lines)  Core AST transformation (133 instrumentation points)
├── type_info.rs            (1185 lines)  Type info loading from JSON, lookup API
├── config.rs               (754 lines)   TraceConfig, presets, feature groups
├── visitor.rs              (262 lines)   Syn VisitMut implementation
├── diagnostics.rs          (221 lines)   Compile-time warnings for ambiguous patterns
├── generic_handler.rs      (218 lines)   Generic function handling
├── parser.rs               (156 lines)   Attribute argument parsing
├── optimized_transform.rs  (148 lines)   Optimized code generation paths
├── best_practices.rs       (142 lines)   Best practice suggestions
├── examples.rs             (140 lines)   Example code generation
├── borrow_detection.rs     (140 lines)   Borrow expression detection
├── validation.rs           (123 lines)   Input validation
├── pattern.rs              (107 lines)   Pattern analysis (tuple, struct, etc.)
├── codegen.rs              (106 lines)   Code generation utilities
├── span_utils.rs           (83 lines)    Span manipulation
├── formatting.rs           (72 lines)    Output formatting
└── hygiene.rs              (70 lines)    Macro hygiene utilities

tests/
├── 30 test files (89 .rs files total)
├── 533 tests covering all instrumentation points
└── compile/ (trybuild compile-fail tests)
```

**Total: ~8,800 lines**

## Dependencies

| Crate | Purpose |
|-------|---------|
| `syn` | Rust AST parsing and manipulation |
| `quote` | Code generation (TokenStream construction) |
| `proc-macro2` | Proc macro token types |
| `proc-macro-error` | Better error reporting in proc macros |
| `proc-macro-warning` | Compile-time warnings |
| `serde` / `serde_json` | Reading type-info.json |

## Testing

```bash
# Run all 533 tests
cargo test -p borrowscope-macro

# Run specific test category
cargo test -p borrowscope-macro --test rc_arc_tests
cargo test -p borrowscope-macro --test async_tests
cargo test -p borrowscope-macro --test closure_tests
```

Test categories:
- `track_new_tests` / `track_move_tests` / `track_borrow_tests` - core ownership
- `rc_arc_tests` / `box_tests` - smart pointers
- `closure_tests` - closure captures and traits
- `async_tests` - async/await instrumentation
- `control_flow_tests` - loops, branches, break/continue
- `pattern_tests` - destructuring, match bindings
- `config_tests` - presets, skip/only, sampling
- `semantic_integration` / `semantic_lookup_tests` - type info integration
- `compile/` - trybuild compile-fail tests

## Performance Impact

- **Compile time:** +5-15% (type info loading is cached via OnceLock)
- **Runtime overhead:** ~75-80ns per tracking call (when `track` feature enabled)
- **Zero cost without feature:** All tracking calls are no-ops without `features = ["track"]`
- **Sampling mode:** Reduces overhead proportionally (10% sample = ~8ns average)

## License

Apache-2.0
