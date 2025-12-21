# BorrowScope Transformation Strategy

## Overview

This document defines the complete strategy for transforming Rust code to inject runtime tracking calls while preserving program semantics.

## Transformation Rules

### Rule 1: Variable Creation

**Pattern**: `let <ident> = <expr>;`

**Transformation**:
```rust
// Before
let x = 42;

// After  
let x = borrowscope_runtime::track_new("x", 42);
```

**Implementation**: Wrap the initializer expression with `track_new()`, which returns the value unchanged.

### Rule 2: Immutable Borrow

**Pattern**: `let <ident> = &<expr>;`

**Transformation**:
```rust
// Before
let r = &x;

// After
let r = borrowscope_runtime::track_borrow("r", &x);
```

**Implementation**: Wrap the reference expression with `track_borrow()`.

### Rule 3: Mutable Borrow

**Pattern**: `let <ident> = &mut <expr>;`

**Transformation**:
```rust
// Before
let r = &mut x;

// After
let r = borrowscope_runtime::track_borrow_mut("r", &mut x);
```

**Implementation**: Wrap the mutable reference with `track_borrow_mut()`.

### Rule 4: Move (Future)

**Pattern**: Assignment consuming a value

**Transformation**:
```rust
// Before
let y = x;

// After
let y = borrowscope_runtime::track_move("x", "y", x);
```

**Status**: Not yet implemented - requires ownership analysis.

### Rule 5: Drop (Future)

**Pattern**: End of scope

**Transformation**:
```rust
// Before
{
    let x = 42;
} // x dropped here

// After
{
    let x = borrowscope_runtime::track_new("x", 42);
    borrowscope_runtime::track_drop("x");
}
```

**Status**: Not yet implemented - requires scope tracking.

## Semantic Preservation Principles

### Principle 1: Zero Runtime Impact

Tracking functions must return their input unchanged:

```rust
pub fn track_new<T>(name: &str, value: T) -> T {
    // ... tracking logic ...
    value  // Return unchanged
}
```

This ensures:
- No performance impact when tracking is disabled
- Type inference still works
- Lifetimes are preserved

### Principle 2: Type Preservation

The transformed code must have identical types:

```rust
// Original type: i32
let x = 42;

// Transformed type: still i32
let x = track_new("x", 42);
```

### Principle 3: Lifetime Preservation

References must maintain their lifetimes:

```rust
// Original: 'a lifetime
fn get_ref<'a>(x: &'a i32) -> &'a i32 { x }

// Transformed: 'a lifetime preserved
fn get_ref<'a>(x: &'a i32) -> &'a i32 {
    track_borrow("x", x)
}
```

## Current Implementation Status

### ✅ Implemented

1. **Simple variable tracking**
   - `let x = <expr>` → `track_new("x", <expr>)`
   - Works with any type
   - Preserves type inference

2. **Immutable borrow tracking**
   - `let r = &x` → `track_borrow("r", &x)`
   - Preserves reference lifetime
   - Works in any context

3. **Mutable borrow tracking**
   - `let r = &mut x` → `track_borrow_mut("r", &mut x)`
   - Preserves mutable reference semantics
   - Maintains exclusivity

### 🚧 Planned

1. **Move tracking**
   - Detect ownership transfers
   - Insert `track_move()` calls
   - Handle complex patterns

2. **Drop tracking**
   - Track scope boundaries
   - Insert `track_drop()` in LIFO order
   - Handle early returns

3. **Pattern destructuring**
   - Tuple patterns: `let (x, y) = tuple`
   - Struct patterns: `let Point { x, y } = point`
   - Nested patterns

4. **Control flow**
   - If/else expressions
   - Match expressions
   - Loop expressions

5. **Method calls**
   - Detect self borrows
   - Track receiver
   - Handle method chains

## Edge Cases

### Case 1: Uninitialized Variables

```rust
let x;
x = 5;
```

**Strategy**: Don't track uninitialized variables (no initializer to wrap).

### Case 2: Pattern Matching

```rust
let (x, y) = (1, 2);
```

**Strategy**: Track the tuple, then track individual bindings (future work).

### Case 3: Temporaries

```rust
foo(&42);
```

**Strategy**: Don't track temporaries (they have no variable name).

### Case 4: Shadowing

```rust
let x = 1;
let x = 2;
```

**Strategy**: Track as two separate variables with the same name.

## Testing Strategy

### Unit Tests

Test individual transformation rules:

```rust
#[test]
fn test_transform_simple_variable() {
    let input = quote! { let x = 5; };
    let output = transform(input);
    assert!(output.contains("track_new"));
}
```

### Integration Tests

Test with runtime:

```rust
#[test]
fn test_tracking_works() {
    #[trace_borrow]
    fn example() {
        let x = 5;
    }
    
    borrowscope_runtime::reset();
    example();
    let events = borrowscope_runtime::get_events();
    assert_eq!(events.len(), 1);
}
```

### Compile Tests

Test that transformed code compiles:

```rust
#[test]
fn test_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/simple.rs");
}
```

## Performance Considerations

### Transformation Time

- Target: <1ms per function
- Current: ~100μs for simple functions
- Bottleneck: AST traversal

### Runtime Overhead

- Target: <100ns per operation
- Current: ~75ns (from Chapter 3 benchmarks)
- Achieved through inlining

### Binary Size

- Target: <5% increase
- Strategy: Feature flags to disable tracking
- Optimization: Dead code elimination

## Known Limitations

### Cannot Track

1. **Temporaries**: No variable name to track
2. **Macro expansions**: Transformed before our macro runs
3. **External crates**: Can't modify their code
4. **FFI boundaries**: Crosses language barrier

### Best Effort

1. **Generic types**: Use `std::any::type_name::<T>()` at runtime
2. **Closure captures**: Simplified analysis
3. **Method self borrows**: Heuristic-based detection

## Future Enhancements

### Phase 1 (Current)
- ✅ Simple variables
- ✅ Basic borrows
- 🚧 Documentation

### Phase 2 (Next)
- Move tracking
- Drop tracking
- Scope management

### Phase 3 (Advanced)
- Pattern destructuring
- Control flow
- Method calls

### Phase 4 (Polish)
- Error messages
- Optimization
- Comprehensive testing

## References

- [Rust Reference - Expressions](https://doc.rust-lang.org/reference/expressions.html)
- [syn VisitMut](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html)
- [quote macro](https://docs.rs/quote/latest/quote/)
- Chapter 3: Runtime Tracker implementation
