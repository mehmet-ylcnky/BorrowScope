# Complete Mapping: 35 Heuristics → 17 Verified rust-analyzer APIs

**Status:** ✅ ALL 35 HEURISTICS ELIMINATED — Implementation complete as of `dd805336b`

---

## Heuristics #1-23: String Matching → Canonical Path Matching

**Problem:** Macro does string matching on operation/method names
```rust
// HEURISTIC (current):
if op.contains("Rc") && op.contains("clone") { ... }
if method_name == "clone" { ... }
```

**Solution:** Use canonical path from rust-analyzer
```rust
// SEMANTIC (target):
match canonical_path.as_deref() {
    Some("alloc::rc::Rc::clone") => { ... },
    Some("alloc::sync::Arc::clone") => { ... },
    Some("core::cell::RefCell::borrow") => { ... },
    _ => {}
}
```

**APIs Used:**
1. `Semantics::resolve_method_call()` → `Function`
2. `Function::module()` → `Module`
3. `Module::path_to_root()` → `Iterator<Module>`
4. `Module::krate()` → `Crate`
5. `Crate::display_name()` → crate name
6. `Function::name()` → function name

**Eliminates Heuristics:**
- #1: `op.contains("Rc") && op.contains("clone")`
- #2: `op.contains("Weak") && op.contains("clone")`
- #3: `op.contains("Weak") && op.contains("upgrade")`
- #4: `op.contains("RefCell") && op.contains("borrow")`
- #5: `op.contains("Cell") && op.contains("get/set")`
- #6: `op.contains("Mutex::lock")`
- #7: `op.contains("RwLock::read/write")`
- #9: `op.contains("Cow") && op.contains("to_mut")`
- #10: `op.contains("JoinHandle") && op.contains("join")`
- #11: `op.contains("Sender") && op.contains("send")`
- #12: `op.contains("Receiver") && op.contains("recv")`
- #15: `method_name == "clone"`
- #16: `method_name == "upgrade"`
- #17: `method_name == "to_mut"`
- #18: `method_name == "join"`
- #19: `method_name == "send"`
- #20: `match method_name.as_str() { "unwrap" | "expect" | ... }`
- #21: `match method_name.as_str() { "lock" | "read" | "write" }`
- #22: `match method_name.as_str() { "get" | "set" }`
- #23: `match method_name.as_str() { "borrow" | "borrow_mut" }`
- #24: `path_str.contains("transmute")`
- #25: `fn_name.contains("transmute")`

---

## Heuristics #8, #13-14, #31-32: Type Name Matching → ADT Name

**Problem:** Checking receiver type by string matching
```rust
// HEURISTIC (current):
if op.contains("option") || op.contains("result") { ... }
if op.contains("OnceCell") || op.contains("OnceLock") { ... }
if op.contains("MaybeUninit") { ... }
```

**Solution:** Get ADT name from type
```rust
// SEMANTIC (target):
let receiver_type = sema.type_of_expr(&method_call.receiver())?.original;
if let Some(adt) = receiver_type.as_adt(db) {
    let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
    match adt_name.as_str() {
        "Option" => { ... },
        "Result" => { ... },
        "OnceCell" | "OnceLock" => { ... },
        "MaybeUninit" => { ... },
        _ => {}
    }
}
```

**APIs Used:**
7. `Semantics::type_of_expr()` → `TypeInfo`
8. `Type::as_adt()` → `Option<Adt>`
9. `Adt::name()` → `Name`

**Eliminates Heuristics:**
- #8: `op.contains("option") || op.contains("result")`
- #13: `op.contains("OnceCell") || op.contains("OnceLock")`
- #14: `op.contains("MaybeUninit")`
- #31: `unwrap_or_else(|| self.maybe_uninit_vars.contains())`
- #32: `unwrap_or_else(|| !self.once_cell_vars.contains())`

---

## Heuristic #29: Union Name Matching → ADT Type Check

**Problem:** Checking if type is union by name pattern
```rust
// HEURISTIC (current):
fn looks_like_union(name: &str) -> bool {
    name.contains("Union") || name.contains("Raw")
}
```

**Solution:** Check ADT variant
```rust
// SEMANTIC (target):
let receiver_type = sema.type_of_expr(&expr)?.original;
if let Some(adt) = receiver_type.as_adt(db) {
    let is_union = matches!(adt, Adt::Union(_));
}
```

**APIs Used:**
7. `Semantics::type_of_expr()` → `TypeInfo`
8. `Type::as_adt()` → `Option<Adt>`
10. Pattern match on `Adt` enum

**Eliminates Heuristics:**
- #29: `looks_like_union()` - checks name contains "Union", "Raw"

---

## Heuristic #30: Trait Method Detection (ALREADY SEMANTIC ✅)

**Status:** Already has `is_trait_method` field in `MethodBorrowInfo`

**APIs Used:**
11. `Function::as_assoc_item()` → `Option<AssocItem>`
12. `AssocItem::containing_trait()` → `Option<Trait>`
13. `Trait::name()` → `Name`

**Eliminates Heuristics:**
- #30: `is_clone_trait.unwrap_or(true)` - already semantic

---

## Heuristics #26-27: FFI Detection → extern_block()

**Problem:** Checking if function is FFI by name patterns
```rust
// HEURISTIC (current):
fn looks_like_ffi(name: &str) -> bool {
    name.starts_with("c_") || name.starts_with("ffi_") || 
    matches!(name, "malloc" | "free" | "printf" | ...)
}
```

**Solution:** Check if function is from extern block
```rust
// SEMANTIC (target):
let func = sema.resolve_path(&path)?;
let is_ffi = func.extern_block(db).is_some();
```

**APIs Used:**
14. `Function::extern_block()` → `Option<ExternBlock>`

**Eliminates Heuristics:**
- #26: `looks_like_ffi()` - checks prefixes `c_`, `ffi_`
- #27: `looks_like_ffi()` - checks names `malloc`, `printf`, etc.

---

## Heuristic #28: Static Detection → PathResolution

**Problem:** Checking if variable is static by SCREAMING_SNAKE_CASE
```rust
// HEURISTIC (current):
fn looks_like_static(name: &str) -> bool {
    name.chars().all(|c| c.is_uppercase() || c == '_')
}
```

**Solution:** Check path resolution
```rust
// SEMANTIC (target):
use ra_ap_hir::PathResolution;

if let Some(PathResolution::Def(ModuleDef::Static(_))) = sema.resolve_path(&path) {
    // This is a static variable
}
```

**APIs Used:**
15. `Semantics::resolve_path()` → `Option<PathResolution>`
16. Pattern match on `PathResolution::Def(ModuleDef::Static)`

**Eliminates Heuristics:**
- #28: `looks_like_static()` - checks SCREAMING_SNAKE_CASE

---

## Heuristic #33: Tracking Function Detection → Crate Check

**Problem:** Checking if function is from borrowscope_runtime
```rust
// HEURISTIC (current):
if fn_name.contains("track_") { ... }
```

**Solution:** Check function's crate
```rust
// SEMANTIC (target):
let func = sema.resolve_path(&path)?;
let module = func.module(db);
let krate = module.krate(db);
let crate_name = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();

if crate_name == "borrowscope_runtime" {
    // Skip tracking this function
}
```

**APIs Used:**
2. `Function::module()` → `Module`
4. `Module::krate()` → `Crate`
5. `Crate::display_name()` → crate name

**Eliminates Heuristics:**
- #33: `fn_name.contains("track_")` - check if from borrowscope_runtime crate

---

## Heuristic #34: Underscore Prefix (OK TO KEEP)

**Status:** This is a Rust convention, not a heuristic

```rust
// This is fine - Rust convention for unused variables
if ident.to_string().starts_with('_') {
    // Variable is intentionally unused
}
```

**No API needed** - This is semantic by Rust language convention

---

## Heuristic #35: Guard Method Detection → Return Type Check

**Problem:** Hardcoded list of guard-returning methods
```rust
// HEURISTIC (current):
let guard_methods = ["lock", "read", "write", "borrow", "borrow_mut"];
if guard_methods.contains(&method_name) { ... }
```

**Solution:** Check return type name
```rust
// SEMANTIC (target):
let func = sema.resolve_method_call(method_call)?;
let ret_type = func.ret_type(db);

if let Some(adt) = ret_type.as_adt(db) {
    let type_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
    let is_guard = type_name.ends_with("Guard");
    // Matches: MutexGuard, RwLockReadGuard, RwLockWriteGuard, etc.
}
```

**APIs Used:**
17. `Function::ret_type()` → `Type`
8. `Type::as_adt()` → `Option<Adt>`
9. `Adt::name()` → `Name`

**Eliminates Heuristics:**
- #35: `guard_methods.contains(&method_name)` - hardcoded list

---

## Summary: 17 APIs Eliminate 34 Heuristics

| API # | API Name | Heuristics Eliminated |
|-------|----------|----------------------|
| 1 | `Semantics::resolve_method_call()` | #1-25 (method resolution) |
| 2 | `Function::module()` | #1-25, #33 (path building, crate check) |
| 3 | `Module::path_to_root()` | #1-25 (path building) |
| 4 | `Module::krate()` | #1-25, #33 (path building, crate check) |
| 5 | `Crate::display_name()` | #1-25, #33 (path building, crate check) |
| 6 | `Function::name()` | #1-25 (path building) |
| 7 | `Semantics::type_of_expr()` | #8, #13-14, #29, #31-32 (type resolution) |
| 8 | `Type::as_adt()` | #8, #13-14, #29, #31-32, #35 (ADT check) |
| 9 | `Adt::name()` | #8, #13-14, #31-32, #35 (ADT name) |
| 10 | `Adt` enum matching | #29 (union check) |
| 11 | `Function::as_assoc_item()` | #30 (trait method) |
| 12 | `AssocItem::containing_trait()` | #30 (trait method) |
| 13 | `Trait::name()` | #30 (trait name) |
| 14 | `Function::extern_block()` | #26-27 (FFI detection) |
| 15 | `Semantics::resolve_path()` | #24-25, #28 (path resolution) |
| 16 | `PathResolution` pattern match | #28 (static detection) |
| 17 | `Function::ret_type()` | #35 (guard detection) |

**Total:** 34 heuristics eliminated (35 total - 1 kept as Rust convention)

---

## Implementation Checklist

- [x] Update analyzer to provide `canonical_path` field (via `operation` in `MethodCallInfo`)
- [x] Update analyzer to provide `receiver_adt` field (via `is_union`, `is_extern_type`, `is_static` in `VariableTypeInfo`)
- [x] Update analyzer to detect FFI functions (`is_extern_type` field)
- [x] Update analyzer to detect static variables (`is_static` field)
- [x] Update analyzer to detect union types (`is_union` field)
- [x] Update analyzer to check return types for guards (replaced with canonical path matching)
- [x] Update macro to use exact path matching (all `op.contains()` → `semantic_op` canonical paths)
- [x] Remove ALL `.contains()` checks from macro (zero remain)
- [x] Remove ALL string pattern matching from macro (zero heuristics remain)
- [x] Remove `diagnostics.rs` heuristic functions (`looks_like_ffi/static/union` deleted)
- [x] Verify 0 heuristics remain with exhaustive search

### Commits
- `258c9f3ca` — Category 1: all 19 `op.contains()` → exact canonical paths
- `dd805336b` — Categories 2-7: method name, function path, diagnostics, tracking sets, guard methods
