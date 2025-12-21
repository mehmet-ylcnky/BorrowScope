# BorrowScope Macro Implementation Plan

## Completed Phases

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Complete | Location, RefCell/Cell |
| Phase 2 | ✅ Complete | Unsafe blocks, raw ptr casts, transmute |
| Phase 3 | ❌ Not Implementable | Static/const (requires type info) |
| Phase 4 | ✅ Complete | Async blocks and await expressions |
| Phase 5 | ✅ Complete | Loops, match, branches, return, try, clone, lock, unwrap |
| Phase 6 | ✅ Complete | Break/continue, struct/tuple, range, array, cast, closure, region |
| Phase 7 | ✅ Complete | Attribute parameters for configurable tracking |

---

## Phase 6: Additional Tracking Features (COMPLETED)

### 6.1 Break/Continue Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_break(break_id: usize, loop_id: Option<&str>, location: &str)
pub fn track_continue(continue_id: usize, loop_id: Option<&str>, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `Break { timestamp, break_id, loop_label, location }`
- `Continue { timestamp, continue_id, loop_label, location }`

**Macro transformation:**
```rust
// Input:
loop {
    if done { break; }
    if skip { continue; }
}

// Output:
loop {
    if done {
        track_break(ID, None, "loc");
        break;
    }
    if skip {
        track_continue(ID, None, "loc");
        continue;
    }
}

// With labels:
'outer: loop { break 'outer; }
// Output:
'outer: loop {
    track_break(ID, Some("outer"), "loc");
    break 'outer;
}
```

**Detectable patterns:** `Expr::Break`, `Expr::Continue`

---

### 6.2 Closure Capture Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_closure_create(closure_id: usize, capture_mode: &str, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `ClosureCreate { timestamp, closure_id, capture_mode, location }`

**Macro transformation:**
```rust
// Input:
let c = || x + 1;
let c = move || x + 1;

// Output:
let c = {
    track_closure_create(ID, "ref", "loc");
    || x + 1
};
let c = {
    track_closure_create(ID, "move", "loc");
    move || x + 1
};
```

**Detectable patterns:** `Expr::Closure` (check `capture` field for `move` keyword)

**Note:** Runtime function implemented, macro transformation not yet added (closures are visited but not tracked with new event).

---

### 6.3 Struct/Tuple Construction Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_struct_create(struct_id: usize, type_name: &str, location: &str)
pub fn track_tuple_create(tuple_id: usize, len: usize, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `StructCreate { timestamp, struct_id, type_name, location }`
- `TupleCreate { timestamp, tuple_id, len, location }`

**Macro transformation:**
```rust
// Input:
Point { x: 1, y: 2 }
(a, b, c)

// Output:
{
    track_struct_create(ID, "Point", "loc");
    Point { x: 1, y: 2 }
}
{
    track_tuple_create(ID, 3, "loc");
    (a, b, c)
}
```

**Detectable patterns:** `Expr::Struct`, `Expr::Tuple`

---

### 6.4 Let-Else Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_let_else(let_id: usize, pattern: &str, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `LetElse { timestamp, let_id, pattern, location }`

**Macro transformation:**
```rust
// Input:
let Some(x) = opt else { return };

// Output:
let Some(x) = opt else {
    track_let_else(ID, "Some(x)", "loc");
    return
};
```

**Detectable patterns:** `Stmt::Local` with `else` branch (syn: `local.init.diverge`)

**Note:** Runtime function implemented, macro transformation not yet added.

---

### 6.5 Range Expression Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_range(range_id: usize, range_type: &str, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `Range { timestamp, range_id, range_type, location }`

**Macro transformation:**
```rust
// Input:
0..10
0..=10
..10
0..

// Output:
{ track_range(ID, "Range", "loc"); 0..10 }
{ track_range(ID, "RangeInclusive", "loc"); 0..=10 }
{ track_range(ID, "RangeTo", "loc"); ..10 }
{ track_range(ID, "RangeFrom", "loc"); 0.. }
```

**Detectable patterns:** `Expr::Range`

---

### 6.6 Binary Operation Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_binary_op(op_id: usize, operator: &str, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `BinaryOp { timestamp, op_id, operator, location }`

**Macro transformation:**
```rust
// Input:
x + y
a && b

// Output:
{ track_binary_op(ID, "+", "loc"); x + y }
{ track_binary_op(ID, "&&", "loc"); a && b }
```

**Detectable patterns:** `Expr::Binary`

**Note:** Runtime function implemented, macro transformation not added (too noisy).

---

### 6.7 Array Literal Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_array_create(array_id: usize, len: usize, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `ArrayCreate { timestamp, array_id, len, location }`

**Macro transformation:**
```rust
// Input:
[1, 2, 3]
[0; 10]

// Output:
{ track_array_create(ID, 3, "loc"); [1, 2, 3] }
{ track_array_create(ID, 10, "loc"); [0; 10] }
```

**Detectable patterns:** `Expr::Array`, `Expr::Repeat`

---

### 6.8 Type Cast Tracking (non-pointer) ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_type_cast(cast_id: usize, to_type: &str, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `TypeCast { timestamp, cast_id, to_type, location }`

**Macro transformation:**
```rust
// Input:
x as i64
y as f32

// Output (skip if already handled as ptr cast):
{ track_type_cast(ID, "i64", "loc"); x as i64 }
```

**Detectable patterns:** `Expr::Cast` (exclude pointer casts already handled)

---

## Phase 7: Usability Improvements (COMPLETED)

### 7.1 Attribute Parameters ✅

**No runtime changes needed.**

**Macro enhancement:** ✅ IMPLEMENTED
```rust
#[trace_borrow]                           // default: standard tracking
#[trace_borrow(skip = "loops,branches")]  // skip specific features
#[trace_borrow(only = "ownership")]       // only new/move/drop/borrow
#[trace_borrow(verbose)]                  // enable all tracking
#[trace_borrow(quiet)]                    // minimal tracking (ownership only)
```

**Toggleable feature groups:**
- `ownership` - new, move, drop, borrow
- `smart_pointers` - Rc, Arc, RefCell, Cell
- `loops` - for, while, loop tracking
- `branches` - if/else, match tracking
- `control_flow` - break, continue, return
- `try` - ? operator tracking
- `methods` - clone, lock, unwrap
- `async` - async blocks, await
- `unsafe` - unsafe blocks, ptr casts
- `expressions` - struct, tuple, array, range, cast

**Implementation:** `config.rs` module with `TraceConfig` and `TraceArgs` parsing.

---

### 7.2 Region/Span Tracking ✅

**Runtime functions needed:** ✅ IMPLEMENTED
```rust
pub fn track_region_enter(region_id: usize, name: &str, location: &str)
pub fn track_region_exit(region_id: usize, location: &str)
```

**Events:** ✅ IMPLEMENTED
- `RegionEnter { timestamp, region_id, name, location }`
- `RegionExit { timestamp, region_id, location }`

**Usage (manual, not auto-transformed):**
```rust
// User code:
borrowscope_runtime::region!("initialization", {
    // setup code
});

// Expands to:
{
    track_region_enter(ID, "initialization", "loc");
    let __result = { /* setup code */ };
    track_region_exit(ID, "loc");
    __result
}
```

---

## Summary: New Runtime Functions Needed

| Feature | Functions | Events |
|---------|-----------|--------|
| 6.1 Break/Continue | `track_break`, `track_continue` | `Break`, `Continue` |
| 6.2 Closure | `track_closure_create` | `ClosureCreate` |
| 6.3 Struct/Tuple | `track_struct_create`, `track_tuple_create` | `StructCreate`, `TupleCreate` |
| 6.4 Let-Else | `track_let_else` | `LetElse` |
| 6.5 Range | `track_range` | `Range` |
| 6.6 Binary Op | `track_binary_op` | `BinaryOp` |
| 6.7 Array | `track_array_create` | `ArrayCreate` |
| 6.8 Type Cast | `track_type_cast` | `TypeCast` |
| 7.2 Region | `track_region_enter`, `track_region_exit` | `RegionEnter`, `RegionExit` |

**Total: 12 new functions, 11 new event types**

---

## Implementation Priority

| Priority | Feature | Effort | Value | Runtime Changes |
|----------|---------|--------|-------|-----------------|
| 1 | 6.1 Break/Continue | Low | High | 2 functions, 2 events |
| 2 | 6.2 Closure capture | Low | High | 1 function, 1 event |
| 3 | 6.3 Struct/Tuple | Low | Medium | 2 functions, 2 events |
| 4 | 6.4 Let-Else | Low | Medium | 1 function, 1 event |
| 5 | 7.1 Attribute params | Medium | High | None |
| 6 | 6.5 Range | Low | Low | 1 function, 1 event |
| 7 | 6.7 Array | Low | Low | 1 function, 1 event |
| 8 | 6.6 Binary Op | Low | Low | 1 function, 1 event |
| 9 | 6.8 Type Cast | Low | Low | 1 function, 1 event |
| 10 | 7.2 Region | Medium | Medium | 2 functions, 2 events |

---

## Phase 5: Extended Tracking Features (COMPLETED)

### 5.1 Loop Tracking

**Runtime functions needed:**
```rust
pub fn track_loop_enter(loop_id: usize, loop_type: &str, location: &str)
pub fn track_loop_iteration(loop_id: usize, iteration: usize, location: &str)
pub fn track_loop_exit(loop_id: usize, location: &str)
```

**Events:**
- `LoopEnter { timestamp, loop_id, loop_type, location }`
- `LoopIteration { timestamp, loop_id, iteration, location }`
- `LoopExit { timestamp, loop_id, location }`

**Macro transformation:**
```rust
// Input:
for item in collection { body }

// Output:
{
    track_loop_enter(ID, "for", "loc");
    let mut __iter_count = 0usize;
    for item in collection {
        track_loop_iteration(ID, __iter_count, "loc");
        __iter_count += 1;
        body
    }
    track_loop_exit(ID, "loc");
}
```

**Detectable patterns:** `Expr::ForLoop`, `Expr::While`, `Expr::Loop`

---

### 5.2 Match Arm Tracking

**Runtime functions needed:**
```rust
pub fn track_match_enter(match_id: usize, location: &str)
pub fn track_match_arm(match_id: usize, arm_index: usize, pattern: &str, location: &str)
pub fn track_match_exit(match_id: usize, location: &str)
```

**Events:**
- `MatchEnter { timestamp, match_id, location }`
- `MatchArm { timestamp, match_id, arm_index, pattern, location }`
- `MatchExit { timestamp, match_id, location }`

**Macro transformation:**
```rust
// Input:
match value { A => a, B => b }

// Output:
{
    track_match_enter(ID, "loc");
    let __match_result = match value {
        A => { track_match_arm(ID, 0, "A", "loc"); a }
        B => { track_match_arm(ID, 1, "B", "loc"); b }
    };
    track_match_exit(ID, "loc");
    __match_result
}
```

**Detectable patterns:** `Expr::Match`

---

### 5.3 If/Else Branch Tracking

**Runtime functions needed:**
```rust
pub fn track_branch(branch_id: usize, branch_type: &str, taken: bool, location: &str)
```

**Events:**
- `Branch { timestamp, branch_id, branch_type, taken, location }`

**Macro transformation:**
```rust
// Input:
if cond { then } else { else_branch }

// Output:
if cond {
    track_branch(ID, "if_then", true, "loc");
    then
} else {
    track_branch(ID, "if_else", true, "loc");
    else_branch
}
```

**Detectable patterns:** `Expr::If`

---

### 5.4 Return Tracking

**Runtime functions needed:**
```rust
pub fn track_return(return_id: usize, has_value: bool, location: &str)
```

**Events:**
- `Return { timestamp, return_id, has_value, location }`

**Macro transformation:**
```rust
// Input:
return value;

// Output:
{
    track_return(ID, true, "loc");
    return value;
}
```

**Detectable patterns:** `Expr::Return`

---

### 5.5 Try/? Operator Tracking

**Runtime functions needed:**
```rust
pub fn track_try(try_id: usize, location: &str)
```

**Events:**
- `Try { timestamp, try_id, location }`

**Macro transformation:**
```rust
// Input:
expr?

// Output:
{
    track_try(ID, "loc");
    expr?
}
```

**Detectable patterns:** `Expr::Try`

---

### 5.6 Index Access Tracking

**Runtime functions needed:**
```rust
pub fn track_index_access(access_id: usize, container: &str, location: &str)
```

**Events:**
- `IndexAccess { timestamp, access_id, container, location }`

**Macro transformation:**
```rust
// Input:
arr[i]

// Output:
{
    track_index_access(ID, "arr", "loc");
    arr[i]
}
```

**Detectable patterns:** `Expr::Index`

---

### 5.7 Field Access Tracking

**Runtime functions needed:**
```rust
pub fn track_field_access(access_id: usize, base: &str, field: &str, location: &str)
```

**Events:**
- `FieldAccess { timestamp, access_id, base, field, location }`

**Macro transformation:**
```rust
// Input:
obj.field

// Output:
{
    track_field_access(ID, "obj", "field", "loc");
    obj.field
}
```

**Detectable patterns:** `Expr::Field`

---

### 5.8 Function Call Tracking

**Runtime functions needed:**
```rust
pub fn track_call(call_id: usize, fn_name: &str, location: &str)
```

**Events:**
- `Call { timestamp, call_id, fn_name, location }`

**Macro transformation:**
```rust
// Input:
some_fn(args)

// Output:
{
    track_call(ID, "some_fn", "loc");
    some_fn(args)
}
```

**Detectable patterns:** `Expr::Call` (excluding already-handled cases like transmute)

---

### 5.9 Mutex/RwLock Tracking

**Runtime functions needed:**
```rust
pub fn track_lock(lock_id: usize, lock_type: &str, var_name: &str, location: &str)
```

**Events:**
- `Lock { timestamp, lock_id, lock_type, var_name, location }`

**Macro transformation:**
```rust
// Input:
mutex.lock()

// Output:
{
    track_lock(ID, "mutex_lock", "mutex", "loc");
    mutex.lock()
}
```

**Detectable patterns:** `Expr::MethodCall` where method is `lock`, `read`, `write`, `try_lock`, `try_read`, `try_write`

---

### 5.10 Option/Result Unwrap Tracking

**Runtime functions needed:**
```rust
pub fn track_unwrap(unwrap_id: usize, method: &str, var_name: &str, location: &str)
```

**Events:**
- `Unwrap { timestamp, unwrap_id, method, var_name, location }`

**Macro transformation:**
```rust
// Input:
option.unwrap()

// Output:
{
    track_unwrap(ID, "unwrap", "option", "loc");
    option.unwrap()
}
```

**Detectable patterns:** `Expr::MethodCall` where method is `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`

---

### 5.11 Clone Tracking

**Runtime functions needed:**
```rust
pub fn track_clone(clone_id: usize, var_name: &str, location: &str)
```

**Events:**
- `Clone { timestamp, clone_id, var_name, location }`

**Macro transformation:**
```rust
// Input:
data.clone()

// Output:
{
    track_clone(ID, "data", "loc");
    data.clone()
}
```

**Detectable patterns:** `Expr::MethodCall` where method is `clone`

---

### 5.12 Deref Tracking

**Runtime functions needed:**
```rust
pub fn track_deref(deref_id: usize, var_name: &str, location: &str)
```

**Events:**
- `Deref { timestamp, deref_id, var_name, location }`

**Macro transformation:**
```rust
// Input:
*reference

// Output:
{
    track_deref(ID, "reference", "loc");
    *reference
}
```

**Detectable patterns:** `Expr::Unary` where op is `Deref` (`*`)

---

## Implementation Order

1. **5.1 Loop Tracking** ✅ - for, while, loop with iteration counting
2. **5.5 Try/? Tracking** ✅ - Error propagation points
3. **5.11 Clone Tracking** ✅ - .clone() method calls
4. **5.9 Mutex/RwLock Tracking** ✅ - lock, read, write, try_* methods
5. **5.10 Unwrap Tracking** ✅ - unwrap, expect, unwrap_or, etc.
6. **5.12 Deref Tracking** ❌ - DISABLED (breaks assignment expressions)
7. **5.2 Match Arm Tracking** ✅ - Match enter, arm taken, exit
8. **5.3 Branch Tracking** ✅ - If/else branch taken
9. **5.4 Return Tracking** ✅ - Early returns
10. **5.6 Index Access Tracking** ❌ - DISABLED (breaks assignment expressions)
11. **5.7 Field Access Tracking** ❌ - DISABLED (breaks assignment expressions)
12. **5.8 Function Call Tracking** ❌ - DISABLED (too noisy, available but not enabled)

---

## Disabled Features - Detailed Explanations

### 5.12 Deref Tracking (`*expr`) - DISABLED

**Problem:** The dereference operator `*` can appear in both lvalue (left-hand side of assignment) and rvalue (right-hand side) positions. Our transformation wraps expressions in blocks, which is invalid for lvalues.

**Example of the problem:**
```rust
// Original code:
*ptr = 42;

// Our transformation would produce:
{ track_deref(1, "ptr", "loc"); *ptr } = 42;
//                                      ^^^ ERROR: cannot assign to block expression
```

**Why it cannot be fixed:**

1. **Rust's grammar restriction**: The left-hand side of an assignment must be a "place expression" (lvalue). Block expressions `{ ... }` are "value expressions" (rvalues) and cannot appear on the left side of `=`, `+=`, etc.

2. **Context-awareness required**: To fix this, the macro would need to determine whether `*expr` appears in an lvalue or rvalue position. This requires analyzing the parent expression:
   - `*x = y` → lvalue (cannot wrap)
   - `let z = *x` → rvalue (can wrap)
   - `*x += 1` → lvalue (cannot wrap)
   - `foo(*x)` → rvalue (can wrap)

3. **Proc macro limitations**: While syn provides the AST, determining lvalue vs rvalue context requires walking up the tree from the current expression, which is complex and error-prone in the visitor pattern we use.

4. **Compound assignments**: Even if we detected simple assignments, compound assignments (`+=`, `-=`, `*=`, etc.) and other mutating operations would also break.

**Alternative approaches (not implemented):**
- Only track derefs in known rvalue positions (function arguments, let initializers)
- Require explicit opt-in via a separate attribute
- Use a post-processing pass to remove invalid transformations

---

### 5.6 Index Access Tracking (`arr[i]`) - DISABLED

**Problem:** Same as deref - index expressions can be lvalues.

**Example of the problem:**
```rust
// Original code:
arr[0] = 42;
vec[i] += 1;

// Our transformation would produce:
{ track_index_access(1, "arr", "loc"); arr[0] } = 42;
//                                              ^^^ ERROR: cannot assign to block expression
```

**Why it cannot be fixed:**

1. **Index expressions are place expressions**: In Rust, `arr[i]` can be assigned to if `arr` implements `IndexMut`. The expression `arr[i]` evaluates to a place (memory location), not a value.

2. **Same lvalue/rvalue problem**: We cannot distinguish:
   - `arr[i] = x` → lvalue (cannot wrap)
   - `let x = arr[i]` → rvalue (can wrap)
   - `arr[i] += 1` → lvalue (cannot wrap)
   - `foo(arr[i])` → rvalue (can wrap)

3. **Chained indexing**: Expressions like `matrix[i][j] = x` compound the problem - both index operations are lvalues.

4. **Slice patterns**: Index expressions in patterns (`let [a, b] = arr`) have different semantics entirely.

---

### 5.7 Field Access Tracking (`obj.field`) - DISABLED

**Problem:** Same as deref and index - field access can be lvalues.

**Example of the problem:**
```rust
// Original code:
point.x = 10;
self.counter += 1;

// Our transformation would produce:
{ track_field_access(1, "point", "x", "loc"); point.x } = 10;
//                                                      ^^^ ERROR: cannot assign to block expression
```

**Why it cannot be fixed:**

1. **Field access is a place expression**: `obj.field` refers to a memory location within the struct, which can be assigned to if the struct is mutable.

2. **Extremely common pattern**: Field assignment is one of the most common operations in Rust. Disabling it for lvalue positions would miss most field accesses.

3. **Method call confusion**: The syntax `obj.field` is visually similar to `obj.method()`, but they have completely different semantics. We already handle method calls separately, but field access tracking would interfere.

4. **Tuple field access**: Tuple fields (`tuple.0`, `tuple.1`) have the same problem.

---

### 5.8 Function Call Tracking - DISABLED (by choice)

**Problem:** Unlike the above, function call tracking *can* be implemented correctly. It is disabled because it would generate excessive noise in the event stream.

**Why it's disabled by choice:**

1. **Volume of events**: Every function call would generate an event. In typical Rust code, this includes:
   - Standard library functions (`println!`, `format!`, `Vec::new`)
   - Iterator methods (`.map()`, `.filter()`, `.collect()`)
   - Trait method calls
   - Operator overloads (which are function calls)

2. **Low signal-to-noise ratio**: Most function calls are not relevant to ownership analysis. Tracking `vec.push(x)` is useful, but tracking `x.to_string()` or `format!("{}", x)` adds noise without insight.

3. **Performance impact**: The sheer number of function calls in typical code would significantly increase:
   - Event storage memory usage
   - JSON export size
   - Analysis/visualization processing time

4. **Already covered cases**: The important function calls are already tracked:
   - `Rc::new`, `Arc::new`, `Rc::clone`, `Arc::clone` → smart pointer tracking
   - `RefCell::borrow`, `RefCell::borrow_mut` → interior mutability tracking
   - `.clone()` → clone tracking
   - `.lock()`, `.read()`, `.write()` → lock tracking
   - `.unwrap()`, `.expect()` → unwrap tracking
   - `std::mem::transmute` → transmute tracking

**The code exists but is not called:** The `transform_fn_call` method is implemented and can be enabled if needed for specific debugging scenarios. It could be exposed via a feature flag or attribute parameter in the future.

---

## Summary of Lvalue Problem

The fundamental issue with deref, index, and field access tracking is Rust's distinction between **place expressions** (lvalues) and **value expressions** (rvalues):

| Expression | Can be lvalue? | Can wrap in block? |
|------------|---------------|-------------------|
| `*ptr` | Yes (if ptr is `*mut T`) | No (when lvalue) |
| `arr[i]` | Yes (if arr is `IndexMut`) | No (when lvalue) |
| `obj.field` | Yes (if obj is `&mut`) | No (when lvalue) |
| `x` (variable) | Yes | No (when lvalue) |
| `foo()` | No | Yes |
| `x + y` | No | Yes |

Our transformation `expr` → `{ track(...); expr }` converts a place expression into a value expression, which breaks assignment semantics. This is a fundamental limitation of the block-wrapping approach.

**Possible future solutions:**
1. **Statement-level tracking**: Instead of wrapping expressions, insert tracking statements before/after the containing statement
2. **Compiler plugin**: Use rustc's MIR or HIR where lvalue/rvalue information is available
3. **Selective tracking**: Only track in positions known to be rvalues (function arguments, return values, let initializers)

---

## Test Coverage

- 155+ macro unit tests (including 14 new Phase 5 tests)
- 7 async tracking integration tests
- All tests passing


---

## Phase 8: Enhanced Tracking Features

### 8.1 Function Entry/Exit Tracking

**Runtime functions needed:** NEW
```rust
pub fn track_fn_enter(fn_id: usize, fn_name: &str, location: &str)
pub fn track_fn_exit(fn_id: usize, fn_name: &str, location: &str)
```

**Events:** NEW
- `FnEnter { timestamp, fn_id, fn_name, location }`
- `FnExit { timestamp, fn_id, fn_name, location }`

**Macro transformation:**
```rust
// Input:
#[trace_borrow]
fn example(x: i32) -> i32 {
    x + 1
}

// Output:
fn example(x: i32) -> i32 {
    track_fn_enter(ID, "example", "loc");
    let __result = { x + 1 };
    track_fn_exit(ID, "example", "loc");
    __result
}
```

**Status:** TODO

---

### 8.2 Parameter Tracking

**Runtime functions needed:** Uses existing `track_new`

**Macro transformation:**
```rust
// Input:
#[trace_borrow]
fn example(x: i32, name: String) {
    // ...
}

// Output:
fn example(x: i32, name: String) {
    track_new("x", &x);  // or track_new_with_id
    track_new("name", &name);
    // ...
}
```

**Status:** TODO

---

### 8.3 Drop Order Tracking

**Runtime functions needed:** Uses existing `track_drop`

**Macro transformation:**
```rust
// Input:
{
    let a = 1;
    let b = 2;
    let c = 3;
}

// Output:
{
    let a = track_new("a", 1);
    let b = track_new("b", 2);
    let c = track_new("c", 3);
    // At scope exit, drops in reverse order:
    track_drop("c");
    track_drop("b");
    track_drop("a");
}
```

**Note:** Currently drops are tracked via `TrackGuard`. This would add explicit drop order tracking.

**Status:** TODO

---

### 8.4 Let-Else Transformation

**Runtime functions needed:** Uses existing `track_let_else`

**Macro transformation:**
```rust
// Input:
let Some(x) = opt else { return };

// Output:
let Some(x) = opt else {
    track_let_else(ID, "Some(x)", "loc");
    return
};
```

**Detectable patterns:** `Local` with `diverge` in init

**Status:** TODO (runtime exists)

---

### 8.5 Repeat Array Tracking

**Runtime functions needed:** Uses existing `track_array_create`

**Macro transformation:**
```rust
// Input:
[0; 100]

// Output:
{
    track_array_create(ID, 100, "loc");
    [0; 100]
}
```

**Detectable patterns:** `Expr::Repeat`

**Status:** TODO

---

### 8.6 Closure Capture Details

**Runtime functions needed:** NEW
```rust
pub fn track_closure_capture(closure_id: usize, var_name: &str, capture_mode: &str, location: &str)
```

**Events:** NEW
- `ClosureCapture { timestamp, closure_id, var_name, capture_mode, location }`

**Macro transformation:**
```rust
// Input:
let x = 1;
let y = String::new();
let c = move || x + y.len();

// Output:
let c = {
    track_closure_create(ID, "move", "loc");
    track_closure_capture(ID, "x", "copy", "loc");
    track_closure_capture(ID, "y", "move", "loc");
    move || x + y.len()
};
```

**Note:** Requires analyzing closure body to detect captured variables.

**Status:** TODO

---

## Phase 8 Summary

| Feature | Runtime Changes | Macro Changes | Effort |
|---------|-----------------|---------------|--------|
| 8.1 Fn Entry/Exit | 2 functions, 2 events | Wrap fn body | Low |
| 8.2 Parameters | None | Add track_new for params | Low |
| 8.3 Drop Order | None | Insert drops at scope exit | Medium |
| 8.4 Let-Else | None (exists) | Transform diverge branch | Low |
| 8.5 Repeat Array | None (exists) | Handle Expr::Repeat | Low |
| 8.6 Closure Capture | 1 function, 1 event | Analyze closure body | Medium |

**Total new runtime additions:**
- 3 new functions: `track_fn_enter`, `track_fn_exit`, `track_closure_capture`
- 3 new events: `FnEnter`, `FnExit`, `ClosureCapture`
