# Section 36: Planning the Transformation Strategy

## Learning Objectives

By the end of this section, you will:
- Understand what code transformations are needed
- Plan where to inject tracking calls
- Preserve original program semantics
- Handle edge cases systematically
- Design a robust transformation pipeline

## Prerequisites

- Completed Chapter 3 (Runtime Tracker)
- Understanding of Rust syntax and semantics
- Familiarity with AST concepts from Chapter 2

---

## The Transformation Goal

We need to transform this:

```rust
#[track_ownership]
fn example() {
    let x = 42;
    let r = &x;
}
```

Into this:

```rust
fn example() {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "example.rs:3:9", 42);
    let r = borrowscope_runtime::track_borrow(2, 1, false, "example.rs:4:13", &x);
    borrowscope_runtime::track_drop(2, "example.rs:5:1");
    borrowscope_runtime::track_drop(1, "example.rs:5:1");
}
```

---

## Transformation Rules

### Rule 1: Variable Creation

**Pattern:** `let <pattern> = <expr>;`

**Transform:**
```rust
// Before
let x = 42;

// After
let x = borrowscope_runtime::track_new(
    1,              // unique ID
    "x",            // variable name
    "i32",          // type name
    "file.rs:1:9",  // location
    42              // original value
);
```

**Key insight:** Wrap the initializer expression, return value unchanged.

### Rule 2: Immutable Borrow

**Pattern:** `&<expr>`

**Transform:**
```rust
// Before
let r = &x;

// After
let r = borrowscope_runtime::track_borrow(
    2,              // borrow ID
    1,              // borrowed variable ID
    false,          // is_mutable
    "file.rs:2:9",  // location
    &x              // original reference
);
```

### Rule 3: Mutable Borrow

**Pattern:** `&mut <expr>`

**Transform:**
```rust
// Before
let r = &mut x;

// After
let r = borrowscope_runtime::track_borrow_mut(
    2,              // borrow ID
    1,              // borrowed variable ID
    true,           // is_mutable
    "file.rs:3:9",  // location
    &mut x          // original reference
);
```

### Rule 4: Move

**Pattern:** Assignment or function call consuming a value

**Transform:**
```rust
// Before
let y = x;

// After
borrowscope_runtime::track_move(1, 2, "file.rs:4:9");
let y = x;
```

**Note:** Insert tracking call *before* the move.

### Rule 5: Drop

**Pattern:** End of scope

**Transform:**
```rust
// Before
{
    let x = 42;
    let y = 100;
} // x and y dropped here

// After
{
    let x = borrowscope_runtime::track_new(1, "x", "i32", "file.rs:1:9", 42);
    let y = borrowscope_runtime::track_new(2, "y", "i32", "file.rs:2:9", 100);
    borrowscope_runtime::track_drop(2, "file.rs:3:1");
    borrowscope_runtime::track_drop(1, "file.rs:3:1");
}
```

**Key insight:** Insert drops in LIFO order (reverse of creation).

---

## Semantic Preservation

### Principle 1: Zero Runtime Impact

The transformed code must behave **identically** to the original:

```rust
// Original
let x = expensive_computation();

// Transformed - still only calls expensive_computation() once
let x = borrowscope_runtime::track_new(
    1, "x", "Result", "file.rs:1:9",
    expensive_computation()  // Called exactly once
);
```

### Principle 2: Type Preservation

Tracking functions return their input unchanged:

```rust
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    // ... tracking logic ...
    value  // Return unchanged
}
```

This ensures type inference still works.

### Principle 3: Lifetime Preservation

References must maintain their lifetimes:

```rust
// Original
fn get_ref(x: &i32) -> &i32 { x }

// Transformed - lifetime preserved
fn get_ref(x: &i32) -> &i32 {
    borrowscope_runtime::track_borrow(1, 0, false, "file.rs:1:1", x)
}
```

---

## Edge Cases to Handle

### Case 1: Complex Patterns

```rust
// Tuple destructuring
let (x, y) = (1, 2);

// Struct destructuring
let Point { x, y } = point;

// Nested patterns
let ((a, b), c) = ((1, 2), 3);
```

**Strategy:** Extract each variable from the pattern and track individually.

### Case 2: Borrows in Expressions

```rust
// Borrow in function call
foo(&x);

// Borrow in method call
x.method(&y);

// Borrow in match
match &x {
    ref r => { /* ... */ }
}
```

**Strategy:** Wrap the borrow expression with tracking call.

### Case 3: Implicit Drops

```rust
// Early drop with explicit drop()
drop(x);

// Conditional drop
if condition {
    drop(x);
}

// Drop in loop
for item in vec {
    // item dropped at end of each iteration
}
```

**Strategy:** Track explicit drops, infer implicit drops from scope analysis.

### Case 4: Temporary Values

```rust
// Temporary reference
foo(&42);

// Temporary in expression
let x = String::from("hello").len();
```

**Strategy:** Don't track temporaries (they have no name).

### Case 5: Shadowing

```rust
let x = 1;
let x = 2;  // Shadows previous x
```

**Strategy:** Treat as drop of first `x` followed by new variable.

---

## Transformation Pipeline

### Phase 1: Analysis

1. **Collect variables** - Find all `let` statements
2. **Assign IDs** - Generate unique ID for each variable
3. **Build scope tree** - Track variable lifetimes
4. **Identify borrows** - Find all `&` and `&mut` expressions
5. **Detect moves** - Find ownership transfers

### Phase 2: Transformation

1. **Transform let statements** - Wrap initializers with `track_new`
2. **Transform borrows** - Wrap references with `track_borrow`
3. **Insert move tracking** - Add `track_move` calls
4. **Insert drop tracking** - Add `track_drop` at scope ends

### Phase 3: Validation

1. **Verify syntax** - Ensure transformed code parses
2. **Check semantics** - Verify types still match
3. **Test compilation** - Ensure code compiles

---

## Implementation Strategy

We'll build the transformation in layers:

### Layer 1: Simple Variables (Section 37-38)
```rust
let x = 42;
```

### Layer 2: Simple Borrows (Section 39)
```rust
let r = &x;
```

### Layer 3: Scope Management (Section 41)
```rust
{
    let x = 42;
}  // Insert drop here
```

### Layer 4: Complex Patterns (Section 42)
```rust
let (x, y) = (1, 2);
```

### Layer 5: Control Flow (Section 43)
```rust
if condition {
    let x = 42;
}
```

### Layer 6: Advanced Features (Sections 44-50)
- Method calls
- Closures
- Async/await
- Generics

---

## Code Organization

We'll organize the transformation code as:

```
borrowscope-macro/src/
├── lib.rs              - Entry point
├── visitor.rs          - AST visitor (VisitMut)
├── transform.rs        - Main transformation logic
├── scope.rs            - Scope tracking
├── id_generator.rs     - Unique ID generation
├── pattern.rs          - Pattern analysis
├── borrow_detection.rs - Borrow detection
└── codegen.rs          - Code generation helpers
```

---

## Example: Complete Transformation

**Input:**
```rust
#[track_ownership]
fn example() {
    let x = 42;
    {
        let r = &x;
        println!("{}", r);
    }
    println!("{}", x);
}
```

**Output:**
```rust
fn example() {
    let x = borrowscope_runtime::track_new(1, "x", "i32", "example.rs:3:9", 42);
    {
        let r = borrowscope_runtime::track_borrow(2, 1, false, "example.rs:5:17", &x);
        println!("{}", r);
        borrowscope_runtime::track_drop(2, "example.rs:7:5");
    }
    println!("{}", x);
    borrowscope_runtime::track_drop(1, "example.rs:10:1");
}
```

**Analysis:**
1. Variable `x` created with ID 1
2. Inner scope creates borrow `r` with ID 2
3. Borrow `r` dropped at end of inner scope
4. Variable `x` dropped at end of function

---

## Testing Strategy

For each transformation rule, we'll write tests:

### Compile Tests
```rust
#[test]
fn test_simple_variable() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/simple_variable.rs");
}
```

### Snapshot Tests
```rust
#[test]
fn test_transformation_output() {
    let input = quote! {
        let x = 42;
    };
    let output = transform(input);
    insta::assert_snapshot!(output.to_string());
}
```

### Integration Tests
```rust
#[test]
fn test_runtime_integration() {
    #[track_ownership]
    fn example() {
        let x = 42;
    }
    
    example();
    let events = get_events();
    assert_eq!(events.len(), 2); // New + Drop
}
```

---

## Common Pitfalls

### Pitfall 1: Double Wrapping

**Problem:**
```rust
// Wrong - wraps twice
let x = track_new(1, "x", "i32", "file.rs:1:1",
    track_new(1, "x", "i32", "file.rs:1:1", 42)
);
```

**Solution:** Track which expressions have already been transformed.

### Pitfall 2: Breaking Type Inference

**Problem:**
```rust
// Original - type inferred
let x = vec![1, 2, 3];

// Wrong - type annotation needed
let x = track_new(1, "x", "???", "file.rs:1:1", vec![1, 2, 3]);
```

**Solution:** Use `std::any::type_name::<T>()` or stringify the type.

### Pitfall 3: Incorrect Drop Order

**Problem:**
```rust
// Wrong - drops in creation order
track_drop(1, "file.rs:3:1");
track_drop(2, "file.rs:3:1");
```

**Solution:** Reverse the order (LIFO).

---

## Key Takeaways

✅ **Wrap, don't replace** - Preserve original expressions  
✅ **Zero-cost** - Tracking functions return values unchanged  
✅ **Preserve semantics** - Types, lifetimes, behavior identical  
✅ **Handle edge cases** - Patterns, control flow, temporaries  
✅ **Test thoroughly** - Compile tests, snapshots, integration  
✅ **Build incrementally** - Start simple, add complexity  

---

## Further Reading

- [syn VisitMut trait](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html)
- [Rust reference - Expressions](https://doc.rust-lang.org/reference/expressions.html)
- [Rust reference - Patterns](https://doc.rust-lang.org/reference/patterns.html)
- [Drop order in Rust](https://doc.rust-lang.org/reference/destructors.html)

---

**Previous:** Chapter 3 Summary  
**Next:** [37-implementing-the-ast-visitor.md](./37-implementing-the-ast-visitor.md)

**Progress:** 1/15 ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
