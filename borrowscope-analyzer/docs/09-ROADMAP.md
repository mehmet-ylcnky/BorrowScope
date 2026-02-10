## 9. Development Roadmap - COMPLETED ✅

This section documents the evolution of borrowscope-analyzer from initializer-focused analysis to a complete semantic analysis system. **All phases are now complete**, achieving 100% semantic coverage.

### 9.1 Final Coverage Status

| Category | Patterns | Status |
|----------|----------|--------|
| Variable initializers | 78 | ✅ 100% semantic |
| Method calls on variables | 47+ | ✅ 100% semantic |
| Standalone expressions | 14 | ✅ 100% semantic |
| Self-borrow inference | 47 | ✅ 100% semantic |
| Closure traits | 6 | ✅ 100% semantic |
| **TOTAL** | **192+** | **✅ 100% semantic** |

**Zero heuristic pattern matching required.** All operations classified via rust-analyzer's type system.

### 9.1.1 Async Closure Traits (ra_ap_* 0.0.318+)

The upgrade to rust-analyzer 0.0.318 enabled detection of async closure traits:
- `AsyncFn` - async closure with immutable borrows
- `AsyncFnMut` - async closure with mutable borrows
- `AsyncFnOnce` - async closure that consumes captures

These complement the existing `Fn`/`FnMut`/`FnOnce` detection.

### 9.2 Implemented Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    IMPLEMENTED: FULL EXPRESSION ANALYSIS                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                    Analysis Output                             │
│  ───────────                    ───────────────                             │
│  let x = Rc::new(1);    ───▶    { name: "x", ty: "Rc<i32>",                │
│                                   initializer_kind: "rc_new",               │
│                                   method_calls: [...] }                     │
│                                                                             │
│  cell.set(42);          ───▶    method_calls on "cell": [{                 │
│                                   method: "set",                            │
│                                   operation: "core::cell::set",             │
│                                   self_borrow: "immutable" }]               │
│                                                                             │
│  drop(x);               ───▶    expressions: [{                            │
│                                   operation: "core::mem::drop",             │
│                                   argument: "x" }]                          │
│                                                                             │
│  thread::spawn(|| {});  ───▶    expressions: [{                            │
│                                   operation: "std::thread::spawn",          │
│                                   argument: "<closure captures: ...>" }]    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.3 Implementation Phases - All Complete

#### Phase 1: Method Call Tracking ✅
- `analyze_method_calls()` function
- `extract_receiver_name()` for variable identification
- `resolve_method_path()` for semantic operation paths
- Handles tuple bindings, shadowed variables

#### Phase 1.5: Smart Pointer Methods ✅
- Rc/Arc: clone, downgrade, upgrade
- RefCell: borrow, borrow_mut
- Mutex/RwLock: lock, read, write, try_lock
- Cell: get, set, replace

#### Phase 2: Standalone Expressions ✅
- `TrackedFunctions` struct with semantic `FunctionId` lookup
- Memory: drop, forget, transmute, replace, swap, take
- Threading: spawn with closure capture extraction
- Pointers: read, write, copy, copy_nonoverlapping

#### Phase 3: Self-Borrow Inference ✅
- `resolve_self_borrow()` using `func.self_param(db).access(db)`
- Returns: "immutable" (&self), "mutable" (&mut self), "consuming" (self)

#### Phase 4: Option/Result Methods ✅
- unwrap, expect, unwrap_or, unwrap_or_else
- ok, err, map, and_then, or_else

#### Phase 5: Generic Clone Tracking ✅
- Clone trait method detection via semantic resolution

### 9.4 Schema Evolution

| Version | Changes |
|---------|---------|
| 2.0 | Initial type classification |
| 2.1 | Added initializer_kind |
| 2.2 | Added disambiguation (scope_id, function_name, decl_index) |
| 2.3 | Added 78 semantic initializer categories |
| 2.4 | Added method_calls array |
| 2.5 | Added expressions, semantic operation paths |

### 9.5 Key Implementation Details

#### Semantic Method Resolution
```rust
fn resolve_method_path(sema, method_call, db) -> Option<String> {
    let func = sema.resolve_method_call(method_call)?;
    // Build canonical path from module + function name
    // Returns: "core::cell::set", "alloc::vec::push", etc.
}
```

#### Semantic Function Lookup (Zero Heuristics)
```rust
struct TrackedFunctions {
    functions: HashMap<Function, String>,  // FunctionId -> canonical path
}

impl TrackedFunctions {
    fn new(db) -> Self {
        // Look up functions by semantic identity via import_map::Query
        // Compare FunctionId, not strings
    }
}
```

#### Self-Borrow Detection
```rust
fn resolve_self_borrow(sema, method_call, db) -> Option<String> {
    let func = sema.resolve_method_call(method_call)?;
    let self_param = func.self_param(db)?;
    match self_param.access(db) {
        Access::Shared => "immutable",
        Access::Exclusive => "mutable", 
        Access::Owned => "consuming",
    }
}
```

### 9.6 Success Criteria - All Met ✅

| Criterion | Status |
|-----------|--------|
| Zero null operations in method_calls | ✅ Verified |
| Zero null self_borrow values | ✅ Verified |
| All 14 tracked functions detected | ✅ Verified |
| Chained calls not mis-attributed | ✅ Verified |
| Reference types properly stripped | ✅ Verified |
| Type aliases resolve correctly | ✅ Verified |
| impl Trait methods resolve | ✅ Verified |

### 9.7 Test Coverage

10 integration tests covering:
- Cell, Cow, OnceCell, Channel, JoinHandle methods
- MaybeUninit methods (write, assume_init, assume_init_read, assume_init_drop)
- Self-borrow types (immutable, mutable, consuming)
- Standalone expressions (drop, forget, spawn, transmute, ptr::read/write)
- No null values validation
- Chained call attribution

---

*This roadmap is now historical documentation. See [SEMANTIC_EXPANSION_PLAN.md](SEMANTIC_EXPANSION_PLAN.md) for detailed implementation notes.*
