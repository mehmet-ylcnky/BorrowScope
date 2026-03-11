# Heuristic to Semantic Conversion Analysis

## Current Heuristics in Macro

This document systematically analyzes **ALL** remaining heuristics in the macro and identifies how to make them fully semantic using rust-analyzer APIs.

### Category 1: String Matching on Operation Paths

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|-------------------|----------------------------|
| 1 | `op.contains("Rc") && op.contains("clone")` | Is this `Rc::clone` or `Arc::clone`? | ❌ Provides string | `sema.resolve_method_call()` → `Function::module()` → full path `alloc::rc::Rc::clone` |
| 2 | `op.contains("Weak") && op.contains("clone")` | Is this `Weak::clone`? | ❌ Provides string | `sema.resolve_method_call()` → full path `alloc::rc::Weak::clone` |
| 3 | `op.contains("Weak") && op.contains("upgrade")` | Is this `Weak::upgrade`? | ❌ Provides string | `sema.resolve_method_call()` → full path `alloc::rc::Weak::upgrade` |
| 4 | `op.contains("RefCell") && op.contains("borrow")` | Is this `RefCell::borrow` or `RefCell::borrow_mut`? | ❌ Provides string | `sema.resolve_method_call()` → full path `core::cell::RefCell::borrow` |
| 5 | `op.contains("Cell") && op.contains("get/set")` | Is this `Cell::get` or `Cell::set`? | ❌ Provides string | `sema.resolve_method_call()` → full path `core::cell::Cell::get` |
| 6 | `op.contains("Mutex::lock")` | Is this `Mutex::lock`? | ❌ Provides string | `sema.resolve_method_call()` → full path `std::sync::Mutex::lock` |
| 7 | `op.contains("RwLock::read/write")` | Is this `RwLock::read` or `RwLock::write`? | ❌ Provides string | `sema.resolve_method_call()` → full path `std::sync::RwLock::read` |
| 8 | `op.contains("option") \|\| op.contains("result")` | Is receiver type `Option<T>` or `Result<T,E>`? | ❌ Provides string | `sema.type_of_expr()` → `Type::as_adt()` → check if `Option` or `Result` |
| 9 | `op.contains("Cow") && op.contains("to_mut")` | Is this `Cow::to_mut`? | ❌ Provides string | `sema.resolve_method_call()` → full path `alloc::borrow::Cow::to_mut` |
| 10 | `op.contains("JoinHandle") && op.contains("join")` | Is this `JoinHandle::join`? | ❌ Provides string | `sema.resolve_method_call()` → full path `std::thread::JoinHandle::join` |
| 11 | `op.contains("Sender") && op.contains("send")` | Is this `Sender::send`? | ❌ Provides string | `sema.resolve_method_call()` → full path `std::sync::mpsc::Sender::send` |
| 12 | `op.contains("Receiver") && op.contains("recv")` | Is this `Receiver::recv`? | ❌ Provides string | `sema.resolve_method_call()` → full path `std::sync::mpsc::Receiver::recv` |
| 13 | `op.contains("OnceCell") \|\| op.contains("OnceLock")` | Is this `OnceCell` or `OnceLock`? | ❌ Provides string | `sema.type_of_expr()` → `Type::as_adt()` → check ADT name |
| 14 | `op.contains("MaybeUninit")` | Is this `MaybeUninit`? | ❌ Provides string | `sema.type_of_expr()` → `Type::as_adt()` → check ADT name |

### Category 2: Method Name String Matching

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|-------------------|----------------------------|
| 15 | `method_name == "clone"` | Is this a clone method? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 16 | `method_name == "upgrade"` | Is this Weak::upgrade? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 17 | `method_name == "to_mut"` | Is this Cow::to_mut? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 18 | `method_name == "join"` | Is this JoinHandle::join? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 19 | `method_name == "send"` | Is this Sender::send? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 20 | `match method_name.as_str() { "unwrap" \| "expect" \| ... }` | Is this Option/Result unwrap? | ❌ Just method name | `sema.type_of_expr()` → check receiver type |
| 21 | `match method_name.as_str() { "lock" \| "read" \| "write" }` | Is this Mutex/RwLock? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 22 | `match method_name.as_str() { "get" \| "set" }` | Is this Cell::get/set? | ❌ Just method name | `sema.resolve_method_call()` → full path |
| 23 | `match method_name.as_str() { "borrow" \| "borrow_mut" }` | Is this RefCell? | ❌ Just method name | `sema.resolve_method_call()` → full path |

### Category 3: Function Path String Matching

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|----------------------------|----------------------------|
| 24 | `path_str.contains("transmute")` | Is this `std::mem::transmute`? | ❌ Builds path string | `sema.resolve_path()` → `PathResolution::Def()` → check function path |
| 25 | `fn_name.contains("transmute")` | Is function transmute? | ❌ Just function name | `sema.resolve_path()` → full path |

### Category 4: Name Pattern Heuristics (diagnostics.rs)

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|----------------------------|----------------------------|
| 26 | `looks_like_ffi()` - checks prefixes `c_`, `ffi_`, etc. | Is this an FFI function? | ❌ No | `Function::is_unsafe()` + `Function::is_extern()` |
| 27 | `looks_like_ffi()` - checks names `malloc`, `printf`, etc. | Is this a C stdlib function? | ❌ No | `Function::module()` → check if from `extern` block |
| 28 | `looks_like_static()` - checks SCREAMING_SNAKE_CASE | Is this a static variable? | ❌ No | `sema.resolve_path()` → `PathResolution::Def(ModuleDef::Static)` |
| 29 | `looks_like_union()` - checks name contains "Union", "Raw", etc. | Is this a union type? | ❌ No | `Type::as_adt()` → `Adt::is_union()` |

### Category 5: Trait Method Detection

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|----------------------------|----------------------------|
| 30 | `is_clone_trait.unwrap_or(true)` | Is this `Clone::clone` trait method? | ✅ Provides `is_trait_method` | Already available in `MethodBorrowInfo::is_trait_method` |

### Category 6: Fallback to Tracking Sets

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|----------------------------|----------------------------|
| 31 | `unwrap_or_else(\|\| self.maybe_uninit_vars.contains())` | Is this MaybeUninit? | ❌ Provides string | `sema.type_of_expr()` → `Type::as_adt()` → check ADT name |
| 32 | `unwrap_or_else(\|\| !self.once_cell_vars.contains())` | Is this NOT OnceCell? | ❌ Provides string | `sema.type_of_expr()` → `Type::as_adt()` → check ADT name |

### Category 7: Miscellaneous Heuristics

| # | Heuristic in Macro | Semantic Info Needed | Analyzer Provides? | rust-analyzer API to Get It |
|---|-------------------|---------------------|----------------------------|----------------------------|
| 33 | `fn_name.contains("track_")` | Is this a tracking function? | ❌ No | Should skip by checking if function is from `borrowscope_runtime` crate |
| 34 | `ident.to_string().starts_with('_')` | Is variable name prefixed with underscore? | ❌ No | This is OK - Rust convention for unused variables |
| 35 | `guard_methods.contains(&method_name)` | Is this a guard-returning method? | ❌ Hardcoded list | `sema.resolve_method_call()` → check return type is a guard (MutexGuard, etc.) |

## Total Heuristics Found: 35

## Key rust-analyzer APIs (VERIFIED)

### 1. Method Resolution - Get Full Canonical Path
```rust
// Resolve method call to Function
let func: Function = sema.resolve_method_call(method_call)?;

// Get module containing the function
let module: Module = func.module(db);

// Build full path: crate → module path → function name
let mut segments: Vec<String> = module.path_to_root(db)
    .into_iter()
    .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
    .collect();
segments.reverse();

// Add crate name
let krate = module.krate(db);
let crate_name = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();
if !crate_name.is_empty() {
    segments.insert(0, crate_name);
}

// Add function name
let fn_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
segments.push(fn_name);

let canonical_path = segments.join("::"); // e.g., "alloc::rc::Rc::clone"
```

### 2. Type Resolution - Get ADT Name
```rust
// Get receiver type from method call
let receiver_expr = method_call.receiver()?;
let receiver_type: Type = sema.type_of_expr(&receiver_expr)?.original;

// Check if it's an ADT (struct/enum/union)
if let Some(adt) = receiver_type.as_adt(db) {
    let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string(); // "Rc", "Option", "Result"
    let is_union = adt.is_union(db); // Check if it's a union
}
```

### 3. Trait Method Detection
```rust
// Check if method is from a trait
let func: Function = sema.resolve_method_call(method_call)?;

// Get trait if this is a trait method
let trait_info = func.as_assoc_item(db)
    .and_then(|item| item.containing_trait(db));

let is_trait_method = trait_info.is_some();
let trait_name = trait_info.map(|t| t.name(db).display_no_db(Edition::Edition2021).to_string());
```

### 4. FFI Detection
```rust
// Check if function is extern
let func: Function = sema.resolve_path(&path)?;
let is_extern = func.extern_block(db).is_some(); // Returns Some(ExternBlock) if extern
```

### 5. Static Detection
```rust
// Resolve path to check if it's a static
use ra_ap_hir::PathResolution;

if let Some(PathResolution::Def(ModuleDef::Static(static_def))) = sema.resolve_path(&path) {
    // This is a static variable
}
```

### 6. Guard Type Detection
```rust
// Check if return type is a guard (MutexGuard, RwLockReadGuard, etc.)
let func: Function = sema.resolve_method_call(method_call)?;
let ret_type: Type = func.ret_type(db);

// Check if return type name contains "Guard"
if let Some(adt) = ret_type.as_adt(db) {
    let type_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
    let is_guard = type_name.contains("Guard"); // MutexGuard, RwLockReadGuard, etc.
}
```

## Solution: Store Full Canonical Paths

Instead of storing operation strings like `"Rc::clone"`, the analyzer should store:

```rust
pub struct MethodBorrowInfo {
    pub method: String,                    // "clone"
    pub canonical_path: Option<String>,    // "alloc::rc::Rc::clone"
    pub receiver_type: String,             // "Rc<i32>"
    pub receiver_adt: Option<String>,      // "Rc" (just the ADT name)
    pub self_borrow: Option<String>,       // "immutable"
    pub is_trait_method: Option<bool>,     // true for Clone::clone
    pub trait_name: Option<String>,        // "Clone"
}
```

Then the macro can do **exact path matching**:
```rust
match canonical_path.as_deref() {
    Some("alloc::rc::Rc::clone") => handle_rc_clone(),
    Some("alloc::sync::Arc::clone") => handle_arc_clone(),
    Some("core::cell::RefCell::borrow") => handle_refcell_borrow(),
    // etc.
}
```

## Implementation Plan

1. **Update analyzer output schema** - Add `canonical_path` and `receiver_adt` fields
2. **Use `resolve_method_path()` function** - Already exists in analyzer, just need to store result
3. **Extract ADT name from receiver type** - Use `Type::as_adt()` API
4. **Add FFI/static/union detection** - Use proper rust-analyzer APIs
5. **Update macro to use exact matching** - Replace all `.contains()` with `match` statements
6. **Remove all string parsing heuristics** - No more `.contains()`, `starts_with()`, pattern matching

## Benefits

- ✅ 100% semantic - no string parsing
- ✅ Exact matching - no false positives
- ✅ Type-safe - compiler checks paths
- ✅ Maintainable - clear match statements
- ✅ Fast - no string operations at runtime
