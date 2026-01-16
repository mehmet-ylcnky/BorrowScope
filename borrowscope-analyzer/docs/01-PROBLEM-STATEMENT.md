## 1. Problem Statement

BorrowScope's `#[trace_borrow]` procedural macro automatically instruments Rust functions to track ownership events at runtime. The macro transforms source code by injecting tracking calls around variable bindings, borrows, and drops. However, procedural macros in Rust operate under a fundamental constraint: they execute during the early stages of compilation, before type inference and resolution have occurred.

### The Proc-Macro Type Blindness Problem

When `rustc` invokes a procedural macro, it provides only the raw token stream of the annotated item. At this point in the compilation pipeline, the compiler has not yet performed type checking. Consider the following function:

```rust
#[trace_borrow]
fn example() {
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));
    let borrowed = data.borrow();
    process(&borrowed);
}
```

The `#[trace_borrow]` macro receives tokens representing `Rc::new(RefCell::new(vec![1, 2, 3]))` but has no way to determine:

1. That `data` has type `Rc<RefCell<Vec<i32>>>`
2. That `Rc<T>` is a reference-counted smart pointer requiring `track_rc_new`
3. That the inner `RefCell<T>` provides interior mutability
4. That `borrowed` is a `Ref<Vec<i32>>` guard from `RefCell::borrow()`
5. Whether any of these types implement `Copy`

The macro can only perform syntactic pattern matching on the token stream. It can recognize `Rc::new(...)` by matching the literal tokens, but this approach fails for:

```rust
let data = std::rc::Rc::new(value);     // Different path
let data = MyRc::new(value);            // Type alias
let data = create_shared(value);        // Factory function returning Rc
let data = if cond { Rc::new(a) } else { Rc::new(b) };  // Conditional
```

### Compilation Pipeline and Type Resolution Timing

The Rust compilation process follows a strict ordering where macro expansion precedes type resolution:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RUST COMPILATION PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   PARSING   │───▶│    MACRO    │───▶│    NAME     │───▶│    TYPE     │  │
│  │             │    │  EXPANSION  │    │ RESOLUTION  │    │  CHECKING   │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│        │                  │                  │                  │          │
│        ▼                  ▼                  ▼                  ▼          │
│   Token Stream      Expanded AST        Resolved        Typed HIR         │
│                                          Names                             │
│                                                                             │
│                     ▲                                                       │
│                     │                                                       │
│              #[trace_borrow]                                                │
│              EXECUTES HERE                                                  │
│                                                                             │
│              ✗ No type information available                                │
│              ✗ No trait implementation data                                 │
│              ✗ No generic instantiation info                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

This architectural constraint is not a limitation of BorrowScope's implementation but rather a fundamental property of Rust's compilation model. Procedural macros are intentionally isolated from type information to maintain compilation determinism and enable parallel macro expansion.

### Consequences for BorrowScope

Without type information, `borrowscope-macro` must rely on heuristic pattern matching to select appropriate tracking functions. The current implementation uses syntactic patterns:

```rust
// In borrowscope-macro's transform_visitor.rs
if text.contains("Rc::new") {
    // Assume this is Rc<T> and use track_rc_new
}
```

This heuristic approach leads to several failure modes:

**False Negatives**: The macro fails to recognize smart pointers created through non-standard patterns, resulting in generic `track_new` calls instead of specialized `track_rc_new` or `track_arc_new` calls. This loses semantic information about reference counting behavior.

**False Positives**: A variable named `Rc_new_value` or a comment containing `Rc::new` could theoretically trigger incorrect classification.

**Missing Copy Semantics**: The `Copy` trait fundamentally changes ownership semantics—copying instead of moving. Without knowing whether a type implements `Copy`, the macro cannot accurately represent whether an assignment transfers ownership or creates a copy.

**Incomplete Smart Pointer Coverage**: Types like `Weak<T>`, `MutexGuard<T>`, `RwLockReadGuard<T>`, and user-defined smart pointers cannot be detected through syntax alone.

The borrowscope-analyzer addresses these limitations by performing semantic analysis as a separate build step, extracting complete type information that the macro can consume at expansion time.

---

