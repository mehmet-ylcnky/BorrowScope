# BorrowScope Macro

> Procedural macros for automatic instrumentation of Rust ownership and borrowing

[![Crates.io](https://img.shields.io/crates/v/borrowscope-macro.svg)](https://crates.io/crates/borrowscope-macro)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## Introduction

BorrowScope Macro is a procedural macro crate that automatically instruments Rust code to track ownership transfers, borrows, and memory operations at runtime. It works in conjunction with [borrowscope-runtime](../borrowscope-runtime) to provide visibility into Rust's ownership system without requiring manual instrumentation.

The `#[trace_borrow]` attribute macro transforms your functions by injecting tracking calls at key points: variable creation, borrowing, moves, drops, and smart pointer operations. This enables runtime analysis of ownership flow, which is invaluable for learning Rust, debugging complex ownership scenarios, and understanding how your code interacts with Rust's memory model.

## Purpose and Motivation

Rust's ownership and borrowing system is powerful but operates entirely at compile time, making it invisible during execution. While the borrow checker prevents memory errors, developers often struggle to understand *why* certain patterns work or fail, especially when dealing with:

- Complex ownership chains across function boundaries
- Smart pointer interactions (`Rc`, `Arc`, `RefCell`, `Cell`)
- Interior mutability patterns
- Unsafe code blocks and raw pointer operations

BorrowScope Macro addresses this by making ownership operations observable. Instead of manually adding tracking calls throughout your code, you simply annotate functions with `#[trace_borrow]`, and the macro handles the instrumentation automatically. This approach:

1. **Reduces boilerplate** - No need to wrap every variable creation or borrow manually
2. **Ensures consistency** - All trackable operations are instrumented uniformly
3. **Preserves semantics** - The transformed code behaves identically to the original
4. **Enables tooling** - The generated events can be visualized, analyzed, or exported

## Features

### Basic Ownership Tracking

| Operation | Description |
|-----------|-------------|
| Variable creation | Tracks `let x = value` statements |
| Immutable borrows | Tracks `&x` references |
| Mutable borrows | Tracks `&mut x` references |
| Moves | Tracks ownership transfers |
| Drops | Tracks variables going out of scope (LIFO order) |

### Smart Pointer Support

| Type | Operations Tracked |
|------|-------------------|
| `Rc<T>` | Creation (`Rc::new`), cloning (`Rc::clone`) |
| `Arc<T>` | Creation (`Arc::new`), cloning (`Arc::clone`) |
| `Box<T>` | Creation (`Box::new`) |
| `RefCell<T>` | Creation, `borrow()`, `borrow_mut()` |
| `Cell<T>` | Creation, `get()`, `set()` |

### Unsafe Code Tracking

| Operation | Description |
|-----------|-------------|
| Unsafe blocks | Entry and exit tracking with unique block IDs |
| Raw pointer casts | Tracks `as *const T` and `as *mut T` conversions |
| `transmute` calls | Detects `std::mem::transmute` usage |

### Additional Features

- **Accurate source locations** - Uses `file!()` and `line!()` macros for precise location reporting
- **Scope-aware drop ordering** - Maintains correct LIFO drop order across nested scopes
- **Closure support** - Tracks captured variables in closures
- **Generic function support** - Works with generic type parameters and lifetimes
- **Async function support** - Compatible with async functions (with limitations)

## Usage

Add both crates to your `Cargo.toml`:

```toml
[dependencies]
borrowscope-runtime = { version = "0.1", features = ["track"] }
borrowscope-macro = "0.1"
```

Annotate functions you want to trace:

```rust
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn example() {
    let data = vec![1, 2, 3];      // track_new called
    let reference = &data;          // track_borrow called
    println!("{:?}", reference);
}                                   // track_drop called for data

fn main() {
    reset();  // Clear previous tracking data
    example();
    
    // Export events as JSON
    let events = get_events();
    println!("{}", serde_json::to_string_pretty(&events).unwrap());
}
```

### Smart Pointer Example

```rust
use borrowscope_macro::trace_borrow;
use std::rc::Rc;
use std::cell::RefCell;

#[trace_borrow]
fn smart_pointer_example() {
    // Rc tracking
    let shared = Rc::new(42);           // track_rc_new
    let clone1 = Rc::clone(&shared);    // track_rc_clone
    
    // RefCell tracking
    let cell = RefCell::new(100);       // track_refcell_new
    let guard = cell.borrow();          // track_refcell_borrow
    let mut_guard = cell.borrow_mut();  // track_refcell_borrow_mut
}
```

### Unsafe Code Example

```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn unsafe_example() {
    let x = 42;
    let ptr = &x as *const i32;  // track_raw_ptr
    
    unsafe {                      // track_unsafe_block_enter
        let _val = *ptr;
    }                             // track_unsafe_block_exit
}
```

## Limitations

### Const Functions Cannot Be Traced

Const functions are evaluated at compile time by the Rust compiler, which fundamentally conflicts with runtime tracking. When a function is marked `const`, the compiler may evaluate it during compilation rather than at runtime, meaning any tracking calls we inject would never execute. Furthermore, const contexts have strict restrictions on what operations are permitted—they cannot call non-const functions, and our tracking functions are inherently non-const as they modify global state.

The macro will emit a compile-time error if you attempt to use `#[trace_borrow]` on a const function, with a helpful message explaining that tracking requires runtime operations.

### Extern Functions Cannot Be Traced

Functions with non-Rust ABIs (such as `extern "C"`) cannot be traced because they must conform to foreign calling conventions. Injecting tracking calls would alter the function's behavior and potentially break FFI compatibility. These functions are often called from C code or other languages that expect specific memory layouts and calling semantics that our instrumentation would violate.

### Raw Pointer Dereference Tracking

While the macro can track raw pointer *creation* (the `as *const T` and `as *mut T` cast operations), it cannot track raw pointer *dereferences* (`*ptr`). This limitation exists because Rust's dereference operator (`*`) is syntactically identical for raw pointers and types implementing the `Deref` trait. At macro expansion time, we only have access to the Abstract Syntax Tree (AST), not type information. When we see `*x`, we cannot determine whether `x` is a raw pointer requiring unsafe dereference tracking, or a smart pointer like `Box<T>` or `Rc<T>` that safely implements `Deref`.

Distinguishing between these cases would require type information from the compiler, which is not available to procedural macros. This is a fundamental limitation of Rust's macro system, which operates purely on syntax before type checking occurs.

### FFI Call Tracking

The macro cannot automatically detect and track calls to foreign functions (FFI). When you call a function like `libc::malloc()` or any other extern function, the macro sees only a path expression followed by arguments—syntactically identical to any other function call. Determining whether a function is declared as `extern "C"` requires access to the function's declaration, which may be in a different crate, a system library, or generated by a build script.

Procedural macros operate on a single item at a time (in our case, a function body) and have no mechanism to query declarations from other modules or crates. This information is only available during later compilation stages when the compiler has resolved all names and types.

### Union Field Access Tracking

Accessing fields of a union type is an unsafe operation in Rust because the compiler cannot guarantee which variant is currently valid. However, the macro cannot detect union field access because the syntax `value.field` is identical for structs and unions. Without type information, we cannot distinguish between a safe struct field access and an unsafe union field access.

This limitation means that while we track entry and exit from `unsafe` blocks (where union access must occur), we cannot specifically identify which operations within those blocks are union field accesses versus other unsafe operations.

### Unsafe Function Call Tracking

Similar to FFI calls, the macro cannot detect calls to functions declared as `unsafe fn`. The call syntax `some_function(args)` is identical whether the function is safe or unsafe. Determining the safety requirement of a function requires access to its signature, which may be defined anywhere in the dependency graph.

While all calls to unsafe functions must occur within `unsafe` blocks (which we do track), we cannot distinguish an unsafe function call from a safe function call that happens to be inside an unsafe block for other reasons.

### Static and Const Variable Tracking

Static variables (`static` and `static mut`) and const items (`const`) cannot be tracked for two distinct reasons:

**Declaration tracking is impossible:** Static and const declarations are module-level items, not local variables within function bodies. The `#[trace_borrow]` attribute is designed for function instrumentation and does not have visibility into module-level declarations. Tracking static initialization would require a separate macro approach, such as a `#[trace_static]` attribute for static declarations or a module-level `#[trace_module]` macro.

**Access tracking is impossible:** Even when code inside a traced function accesses a static variable, the macro cannot detect this. When we see an expression like `SOME_STATIC`, it is syntactically identical to accessing a local variable, a const, or even calling a function. Without type information, we cannot determine that `SOME_STATIC` refers to a static variable rather than any other kind of binding.

The runtime library provides `track_static_init`, `track_static_access`, and `track_const_eval` functions, but these cannot be automatically invoked by the macro. Users who need static tracking must manually instrument their code using these runtime functions directly.

### Async Function Limitations

While async functions can be annotated with `#[trace_borrow]` and basic ownership tracking works, the macro does not track async-specific operations like future creation, polling, or await points.

Async functions in Rust are transformed by the compiler into state machines after macro expansion. The `#[trace_borrow]` macro sees the original async function syntax, but the actual execution involves compiler-generated code that creates futures, implements `Poll`, and manages state transitions. This transformation happens in a later compilation phase that procedural macros cannot observe or influence.

What the macro can track in async functions:
- Variable creation and drops within the async function body
- Borrows and moves (though lifetimes across await points are complex)
- Smart pointer operations

What the macro cannot track:
- Future creation (the implicit `impl Future` generated by the compiler)
- Poll invocations (handled by the async runtime, not user code)
- State transitions across await points
- Waker and context interactions

### Type-Dependent Behavior Detection

Several Rust patterns have behavior that depends on types rather than syntax:

- **Drop order for struct fields** - The order fields are dropped depends on declaration order, not usage
- **Implicit dereferencing** - Method calls may auto-deref through multiple layers
- **Deref coercions** - `&String` automatically coerces to `&str` in many contexts
- **Move vs Copy semantics** - Whether assignment moves or copies depends on whether the type implements `Copy`

The macro cannot detect or track these behaviors because they are determined by the type system after macro expansion. We track explicit operations visible in the syntax, but implicit compiler-inserted operations remain invisible to our instrumentation.

## Technical Background

Procedural macros in Rust operate during an early phase of compilation, after parsing but before type checking. At this stage, the compiler has constructed an Abstract Syntax Tree (AST) representing the syntactic structure of the code, but has not yet:

1. Resolved names to their definitions
2. Inferred or checked types
3. Determined trait implementations
4. Validated borrow checker rules

This means procedural macros can see *what* code looks like syntactically, but not *what it means* semantically. A macro sees that you wrote `x.foo()`, but cannot know whether `foo` is a method on `x`'s type, a method from a trait, or will fail to compile entirely.

BorrowScope Macro works within these constraints by focusing on syntactic patterns that reliably indicate ownership operations:

- `let` bindings always create new variables
- `&` and `&mut` always create references
- `unsafe { }` blocks are syntactically distinct
- `as *const T` casts are syntactically identifiable
- Known function names like `Rc::new` or `transmute` can be pattern-matched

For operations that require type information, the only solutions would be:
- **Compiler plugins** (unstable, nightly-only)
- **External analysis tools** (like rust-analyzer integration)
- **Explicit user annotations** (additional attributes marking specific operations)

The current design prioritizes stability and usability on stable Rust, accepting these limitations in exchange for a tool that works reliably across the Rust ecosystem.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.
