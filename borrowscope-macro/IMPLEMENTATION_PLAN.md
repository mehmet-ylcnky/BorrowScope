# BorrowScope Macro Implementation Plan

## Completed Phases

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ Complete | Location, RefCell/Cell |
| Phase 2 | ✅ Complete | Unsafe blocks, raw ptr casts, transmute |
| Phase 3 | ❌ Not Implementable | Static/const (requires type info) |
| Phase 4 | ✅ Complete | Async blocks and await expressions |

---

## Phase 5: Extended Tracking Features

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
