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

### Disabled Features Explanation

Features 5.6, 5.7, and 5.12 (index access, field access, deref) were disabled because
they break assignment expressions. The transformation:

```rust
*x = y;  // becomes { track_deref(...); *x } = y;  // INVALID!
```

The left-hand side of an assignment cannot be a block expression. Fixing this would
require context-aware transformation that distinguishes lvalue from rvalue positions,
which significantly increases complexity.

Feature 5.8 (function call tracking) is implemented but disabled by default as it
would generate too many events for most use cases.

---

## Test Coverage

- 155+ macro unit tests (including 14 new Phase 5 tests)
- 7 async tracking integration tests
- All tests passing
