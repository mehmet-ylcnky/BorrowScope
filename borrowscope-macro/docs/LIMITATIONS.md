# BorrowScope Macro Limitations

## The Type Information Barrier

The `#[trace_borrow]` procedural macro operates at the syntactic level during Rust's compilation pipeline. It runs **before** type checking occurs, which means it can only see tokens and syntax patterns—not the actual types of expressions.

This is a fundamental architectural constraint of Rust's compilation model, not a bug or missing feature. For a comprehensive technical analysis, see our whitepaper: [The Type Information Barrier](../../docs/whitepaper-type-information-challenge.md).

## What the Macro Cannot Auto-Detect

The following patterns require **type information** that procedural macros cannot access:

### 1. FFI Function Calls

```rust
extern "C" {
    fn external_func(ptr: *mut i32);
}

fn regular_func(ptr: *mut i32) {}

fn process(ptr: *mut i32) {
    external_func(ptr);  // FFI call - macro cannot detect
    regular_func(ptr);   // Regular call - looks identical syntactically
}
```

Both calls have identical syntax: `identifier(args)`. The macro cannot know that `external_func` was declared in an `extern` block.

### 2. Union Field Access

```rust
union MyUnion { int_val: i32, float_val: f32 }
struct MyStruct { x: i32, y: i32 }

fn access(u: MyUnion, s: MyStruct) {
    let a = u.int_val;  // Union access - macro cannot detect
    let b = s.x;        // Struct access - looks identical syntactically
}
```

Both are `expr.field` syntax. The macro cannot know that `u` is a union type.

### 3. Static Variable Access

```rust
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn increment() {
    let local = AtomicU32::new(0);
    
    COUNTER.fetch_add(1, Ordering::SeqCst);  // Static - macro cannot detect
    local.fetch_add(1, Ordering::SeqCst);    // Local - looks identical
}
```

Both are `identifier.method(args)`. The macro cannot resolve whether `COUNTER` is a static.

### 4. Type-Dependent Clone Behavior

```rust
fn example<T: Clone>(rc: Rc<i32>, arc: Arc<i32>, generic: T) {
    let a = rc.clone();      // Rc::clone - macro detects via variable tracking
    let b = arc.clone();     // Arc::clone - macro detects via variable tracking
    let c = generic.clone(); // Unknown type - macro cannot determine
}
```

For generic parameters, the macro cannot know what concrete type `T` will be.

## Manual Instrumentation

For these patterns, use the tracking functions from `borrowscope-runtime` directly:

### FFI Calls

```rust
use borrowscope_runtime::track_ffi_call;

extern "C" {
    fn c_function(data: *const u8, len: usize) -> i32;
}

fn call_ffi(data: &[u8]) -> i32 {
    track_ffi_call("c_function", "my_module.rs:15");
    unsafe { c_function(data.as_ptr(), data.len()) }
}
```

### Union Field Access

```rust
use borrowscope_runtime::track_union_field_access;

union Data {
    int_val: i32,
    float_val: f32,
}

fn read_as_int(data: &Data) -> i32 {
    track_union_field_access("Data", "int_val", "my_module.rs:25");
    unsafe { data.int_val }
}
```

### Static Variable Access

```rust
use borrowscope_runtime::{track_static_init, track_static_access};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn init_counter() {
    // Track initialization (call once at program start if needed)
    track_static_init("COUNTER", 1, "AtomicU32", false, ());
}

fn increment() {
    track_static_access(1, "COUNTER", true, "my_module.rs:35");
    COUNTER.fetch_add(1, Ordering::SeqCst);
}

fn read() -> u32 {
    track_static_access(1, "COUNTER", false, "my_module.rs:40");
    COUNTER.load(Ordering::SeqCst)
}
```

## Function Reference

| Function | Purpose | Parameters |
|----------|---------|------------|
| `track_ffi_call(fn_name, location)` | Track FFI boundary crossing | Function name, source location |
| `track_union_field_access(union_name, field_name, location)` | Track union field read | Union type name, field name, location |
| `track_static_init(var_name, var_id, type_name, is_mutable, value)` | Track static initialization | Variable name, unique ID, type, mutability, value |
| `track_static_access(var_id, var_name, is_write, location)` | Track static read/write | Variable ID, name, write flag, location |
| `track_transmute(from_type, to_type, location)` | Track transmute operations | Source type, dest type, location |

## Future Plans

We are exploring solutions to reduce manual instrumentation:

1. **Annotation System** — `#[borrowscope(ffi = ["func1"], unions = ["MyUnion"])]`
2. **Validation Tool** — `cargo borrowscope check` to identify uninstrumented patterns
3. **Compiler Integration** — Full semantic analysis (requires nightly Rust)

See the [whitepaper](../../docs/whitepaper-type-information-challenge.md) for detailed analysis of these approaches.

## What the Macro CAN Detect

The macro successfully auto-instruments these patterns through syntactic analysis:

- Variable creation, moves, drops
- Immutable and mutable borrows
- `Rc::new()`, `Rc::clone()`, `Arc::new()`, `Arc::clone()`
- `Box::new()`, `Box::into_raw()`, `Box::from_raw()`
- `RefCell::borrow()`, `RefCell::borrow_mut()`
- `Cell::get()`, `Cell::set()`
- `Mutex::lock()`, `RwLock::read()`, `RwLock::write()`
- `Cow::to_mut()`
- `Weak::upgrade()`, `Weak::clone()`
- Channel `send()` and `recv()` operations
- Thread `JoinHandle::join()`
- Closure creation and captures
- `unsafe` block entry/exit
- Raw pointer creation and dereference

For most Rust code, these patterns cover the vast majority of ownership events.
