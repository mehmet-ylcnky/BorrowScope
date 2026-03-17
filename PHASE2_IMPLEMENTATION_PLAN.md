# Phase 2: Eliminate All Heuristics — Implementation Plan

> **Goal:** Replace ALL 35 string-matching heuristics in `borrowscope-macro` with 100% semantic analysis using 17 verified rust-analyzer APIs.
>
> **Source documents:** `HEURISTIC_TO_SEMANTIC_ANALYSIS.md`, `VERIFIED_RUST_ANALYZER_APIS.md`, `COMPLETE_HEURISTIC_TO_API_MAPPING.md`
>
> **All APIs verified against:** https://docs.rs/ra_ap_hir/latest/ra_ap_hir/

---

## Table of Contents

1. [The Problem](#1-the-problem)
2. [All 35 Heuristics](#2-all-35-heuristics)
3. [All 17 Verified APIs](#3-all-17-verified-apis)
4. [Heuristic → API Mapping](#4-heuristic--api-mapping)
5. [Target Schema](#5-target-schema)
6. [Code Examples](#6-code-examples)
7. [Files to Modify](#7-files-to-modify)
8. [Implementation Checklist](#8-implementation-checklist)

---

## 1. The Problem

The macro identifies operations (Rc::clone, Mutex::lock, etc.) by string matching on operation names the analyzer provides:

```rust
// CURRENT (heuristic):
let is_rc_clone = op.contains("Rc") && op.contains("clone");  // false positive on "MyRcWrapper"
if method_name == "clone" { ... }  // ambiguous — many types have clone()
fn looks_like_ffi(name: &str) { name.starts_with("c_") || ... }  // guessing by naming convention
```

The fix: the analyzer must provide **canonical paths** and **ADT names** from rust-analyzer's semantic APIs, so the macro can do exact matching:

```rust
// TARGET (semantic):
match canonical_path.as_deref() {
    Some("alloc::rc::Rc::clone") => handle_rc_clone(),
    Some("std::sync::Mutex::lock") => handle_mutex_lock(),
    _ => {}
}
```

---

## 2. All 35 Heuristics

### Category 1: String Matching on Operation Paths (14 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 1 | `op.contains("Rc") && op.contains("clone")` | `Rc::clone` |
| 2 | `op.contains("Weak") && op.contains("clone")` | `Weak::clone` |
| 3 | `op.contains("Weak") && op.contains("upgrade")` | `Weak::upgrade` |
| 4 | `op.contains("RefCell") && op.contains("borrow")` | `RefCell::borrow` / `borrow_mut` |
| 5 | `op.contains("Cell") && op.contains("get/set")` | `Cell::get` / `Cell::set` |
| 6 | `op.contains("Mutex::lock")` | `Mutex::lock` |
| 7 | `op.contains("RwLock::read/write")` | `RwLock::read` / `write` |
| 8 | `op.contains("option") \|\| op.contains("result")` | Receiver is `Option<T>` or `Result<T,E>` |
| 9 | `op.contains("Cow") && op.contains("to_mut")` | `Cow::to_mut` |
| 10 | `op.contains("JoinHandle") && op.contains("join")` | `JoinHandle::join` |
| 11 | `op.contains("Sender") && op.contains("send")` | `Sender::send` |
| 12 | `op.contains("Receiver") && op.contains("recv")` | `Receiver::recv` |
| 13 | `op.contains("OnceCell") \|\| op.contains("OnceLock")` | Receiver is `OnceCell` / `OnceLock` |
| 14 | `op.contains("MaybeUninit")` | Receiver is `MaybeUninit` |

### Category 2: Method Name String Matching (9 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 15 | `method_name == "clone"` | Clone method (but which type?) |
| 16 | `method_name == "upgrade"` | Weak::upgrade (but could be anything) |
| 17 | `method_name == "to_mut"` | Cow::to_mut |
| 18 | `method_name == "join"` | JoinHandle::join |
| 19 | `method_name == "send"` | Sender::send |
| 20 | `match method_name { "unwrap" \| "expect" \| ... }` | Option/Result unwrapping |
| 21 | `match method_name { "lock" \| "read" \| "write" }` | Mutex/RwLock locking |
| 22 | `match method_name { "get" \| "set" }` | Cell::get/set |
| 23 | `match method_name { "borrow" \| "borrow_mut" }` | RefCell borrowing |

### Category 3: Function Path String Matching (2 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 24 | `path_str.contains("transmute")` | `std::mem::transmute` |
| 25 | `fn_name.contains("transmute")` | transmute function |

### Category 4: Name Pattern Heuristics in `diagnostics.rs` (4 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 26 | `looks_like_ffi()` — prefixes `c_`, `ffi_` | FFI function |
| 27 | `looks_like_ffi()` — names `malloc`, `printf`, etc. | C stdlib function |
| 28 | `looks_like_static()` — SCREAMING_SNAKE_CASE | Static variable |
| 29 | `looks_like_union()` — name contains "Union", "Raw" | Union type |

### Category 5: Trait Method Detection (1 heuristic)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 30 | `is_clone_trait.unwrap_or(true)` | Clone trait method — ✅ already has `is_trait_method` field |

### Category 6: Fallback to Tracking Sets (2 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 31 | `unwrap_or_else(\|\| self.maybe_uninit_vars.contains())` | Variable is `MaybeUninit` |
| 32 | `unwrap_or_else(\|\| !self.once_cell_vars.contains())` | Variable is NOT `OnceCell` |

### Category 7: Miscellaneous (3 heuristics)

| # | Heuristic | What it's trying to detect |
|---|-----------|---------------------------|
| 33 | `fn_name.contains("track_")` | Function from `borrowscope_runtime` crate |
| 34 | `ident.to_string().starts_with('_')` | Unused variable — **OK to keep** (Rust convention) |
| 35 | `guard_methods.contains(&method_name)` | Guard-returning method (hardcoded list) |

---

## 3. All 17 Verified APIs

Each API verified against https://docs.rs/ra_ap_hir/latest/ra_ap_hir/ with exact signatures.

### 3.1 Method Resolution → Canonical Path (APIs 1–6)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 1 | `Semantics::resolve_method_call()` | `fn resolve_method_call(&self, call: &MethodCallExpr) -> Option<Function>` | Resolved `Function` |
| 2 | `Function::module()` | `fn module(self, db: &dyn HirDatabase) -> Module` | Containing module |
| 3 | `Module::path_to_root()` | `fn path_to_root(&self, db: &dyn HirDatabase) -> impl Iterator<Item = Module>` | Module chain to root |
| 4 | `Module::krate()` | `fn krate(&self, db: &dyn HirDatabase) -> Crate` | Containing crate |
| 5 | `Crate::display_name()` | `fn display_name(&self, db: &dyn HirDatabase) -> Option<CrateName>` | Crate name |
| 6 | `Function::name()` | `fn name(self, db: &dyn HirDatabase) -> Name` | Function name |

**Combined, these build:** `"alloc::rc::Rc::clone"`, `"std::sync::Mutex::lock"`, etc.

### 3.2 Type Resolution → ADT Name (APIs 7–10)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 7 | `Semantics::type_of_expr()` | `fn type_of_expr(&self, expr: &Expr) -> Option<TypeInfo<'db>>` | `TypeInfo { original, adjusted }` |
| 8 | `Type::as_adt()` | `fn as_adt(&self) -> Option<Adt>` | `Adt` enum (`Struct`, `Union`, `Enum`) |
| 9 | `Adt::name()` | `fn name(self, db: &dyn HirDatabase) -> Name` | ADT name: `"Rc"`, `"Option"`, etc. |
| 10 | `Adt` enum matching | `matches!(adt, Adt::Union(_))` | `bool` — is it a union? |

**`TypeInfo` struct:**
```rust
pub struct TypeInfo<'db> {
    pub original: Type<'db>,   // Use this one — the declared type
    pub adjusted: Option<Type<'db>>,  // Coerced type (usually not needed)
}
```

**`Adt` enum variants:**
```rust
pub enum Adt {
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

// Usage:
match adt {
    Adt::Struct(_) => { /* struct */ },
    Adt::Union(_) => { /* union */ },
    Adt::Enum(_) => { /* enum */ },
}
```

### 3.3 Trait Method Detection (APIs 11–13)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 11 | `Function::as_assoc_item()` | `fn as_assoc_item(self, db: &dyn HirDatabase) -> Option<AssocItem>` | Associated item |
| 12 | `AssocItem::containing_trait()` | `fn containing_trait(self, db: &dyn HirDatabase) -> Option<Trait>` | Containing trait |
| 13 | `Trait::name()` | `fn name(self, db: &dyn HirDatabase) -> Name` | Trait name: `"Clone"`, `"Drop"`, etc. |

**Note:** `Function` implements the `AsAssocItem` trait:
```rust
impl AsAssocItem for Function {
    fn as_assoc_item(self, db: &dyn HirDatabase) -> Option<AssocItem>
}
```

### 3.4 FFI Detection (API 14)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 14 | `Function::extern_block()` | `fn extern_block(self, db: &dyn HirDatabase) -> Option<ExternBlock>` | `Some(ExternBlock)` if extern |

### 3.5 Path Resolution — Static & Transmute Detection (APIs 15–16)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 15 | `Semantics::resolve_path()` | `fn resolve_path(&self, path: &Path) -> Option<PathResolution>` | `PathResolution` enum |
| 16 | `PathResolution::Def(ModuleDef::Static)` | Pattern match | Confirms static variable |

**`PathResolution` enum:**
```rust
pub enum PathResolution {
    Def(ModuleDef),
    // ... other variants (Local, TypeParam, etc.)
}
```

**`ModuleDef` enum (relevant variants):**
```rust
pub enum ModuleDef {
    Static(Static),
    Function(Function),
    // ... other variants (Module, Adt, Variant, Const, Trait, etc.)
}
```

### 3.6 Return Type — Guard Detection (API 17)

| # | API | Signature | Returns |
|---|-----|-----------|---------|
| 17 | `Function::ret_type()` | `fn ret_type(self, db: &dyn HirDatabase) -> Type<'_>` | Return type |

**Combined with APIs 8–9:** check if return type ADT name ends with `"Guard"`.

---

## 4. Heuristic → API Mapping

### Heuristics #1–25: String Matching → Canonical Path

**APIs used:** 1–6 (resolve method → build canonical path)

**Before:**
```rust
if op.contains("Rc") && op.contains("clone") { ... }
if method_name == "clone" { ... }
```

**After:**
```rust
match canonical_path.as_deref() {
    Some("alloc::rc::Rc::clone") => { ... },
    Some("alloc::sync::Arc::clone") => { ... },
    Some("alloc::rc::Weak::clone") => { ... },
    Some("alloc::rc::Weak::upgrade") => { ... },
    Some("core::cell::RefCell::borrow") => { ... },
    Some("core::cell::RefCell::borrow_mut") => { ... },
    Some("core::cell::Cell::get") => { ... },
    Some("core::cell::Cell::set") => { ... },
    Some("std::sync::Mutex::lock") => { ... },
    Some("std::sync::RwLock::read") => { ... },
    Some("std::sync::RwLock::write") => { ... },
    Some("alloc::borrow::Cow::to_mut") => { ... },
    Some("std::thread::JoinHandle::join") => { ... },
    Some("std::sync::mpsc::Sender::send") => { ... },
    Some("std::sync::mpsc::Receiver::recv") => { ... },
    _ => {}
}
```

**Eliminates:** #1–7, #9–12, #15–19, #21–25
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

### Heuristics #8, #13–14, #31–32: Type Name Matching → ADT Name

**APIs used:** 7–9 (type_of_expr → as_adt → name)

**Before:**
```rust
if op.contains("option") || op.contains("result") { ... }
if op.contains("OnceCell") { ... }
if op.contains("MaybeUninit") { ... }
self.maybe_uninit_vars.contains(&var_name)  // tracking set fallback
```

**After:**
```rust
match receiver_adt.as_deref() {
    Some("Option") => { ... },
    Some("Result") => { ... },
    Some("OnceCell") | Some("OnceLock") => { ... },
    Some("MaybeUninit") => { ... },
    _ => {}
}
```

**Eliminates:** #8, #13, #14, #31, #32
- #8: `op.contains("option") || op.contains("result")`
- #13: `op.contains("OnceCell") || op.contains("OnceLock")`
- #14: `op.contains("MaybeUninit")`
- #31: `unwrap_or_else(|| self.maybe_uninit_vars.contains())`
- #32: `unwrap_or_else(|| !self.once_cell_vars.contains())`

### Heuristic #20: Option/Result Method Matching → Canonical Path + ADT

**APIs used:** 1–6 (canonical path) + 7–9 (receiver ADT)

**Before:**
```rust
match method_name.as_str() {
    "unwrap" | "expect" | "unwrap_or" | ... => { ... }
}
```

**After:** Match on canonical path (e.g., `"core::option::Option::unwrap"`) or check `receiver_adt == "Option"`.

**Eliminates:** #20

### Heuristics #26–27: FFI Detection → extern_block()

**APIs used:** 14

**Before:**
```rust
fn looks_like_ffi(name: &str) -> bool {
    name.starts_with("c_") || name.starts_with("ffi_") ||
    matches!(name, "malloc" | "free" | "printf" | ...)
}
```

**After:**
```rust
let is_ffi = func.extern_block(db).is_some();
```

**Eliminates:** #26, #27
- #26: `looks_like_ffi()` — checks prefixes `c_`, `ffi_`
- #27: `looks_like_ffi()` — checks names `malloc`, `printf`, etc.

### Heuristic #28: Static Detection → PathResolution

**APIs used:** 15–16

**Before:**
```rust
fn looks_like_static(name: &str) -> bool {
    name.chars().all(|c| c.is_uppercase() || c == '_')
}
```

**After:**
```rust
if let Some(PathResolution::Def(ModuleDef::Static(_))) = sema.resolve_path(&path) {
    // confirmed static
}
```

**Eliminates:** #28
- #28: `looks_like_static()` — checks SCREAMING_SNAKE_CASE

### Heuristic #29: Union Detection → Adt Enum

**APIs used:** 7–8, 10

**Before:**
```rust
fn looks_like_union(name: &str) -> bool {
    name.contains("Union") || name.contains("Raw")
}
```

**After:**
```rust
let is_union = matches!(adt, Adt::Union(_));
```

**Eliminates:** #29
- #29: `looks_like_union()` — checks name contains "Union", "Raw"

### Heuristic #30: Trait Method — Already Semantic ✅

Already has `is_trait_method` field in `MethodBorrowInfo`. Uses APIs 11–13.

### Heuristic #33: Tracking Function → Crate Check

**APIs used:** 2, 4, 5

**Before:**
```rust
if fn_name.contains("track_") { ... }
```

**After:**
```rust
let crate_name = func.module(db).krate(db).display_name(db);
if crate_name == "borrowscope_runtime" { /* skip */ }
```

**Eliminates:** #33
- #33: `fn_name.contains("track_")` — check if from `borrowscope_runtime` crate

### Heuristic #34: Underscore Prefix — OK to Keep

`ident.to_string().starts_with('_')` is a Rust language convention, not a heuristic. No API needed.

```rust
// This is fine — Rust convention for unused variables
if ident.to_string().starts_with('_') {
    // Variable is intentionally unused
}
```

### Heuristic #35: Guard Methods → Return Type Check

**APIs used:** 17, 8, 9

**Before:**
```rust
let guard_methods = ["lock", "read", "write", "borrow", "borrow_mut"];
if guard_methods.contains(&method_name) { ... }
```

**After:**
```rust
let ret_type = func.ret_type(db);
if let Some(adt) = ret_type.as_adt(db) {
    let is_guard = adt.name(db).display_no_db(Edition::Edition2021)
        .to_string().ends_with("Guard");
}
```

**Eliminates:** #35
- #35: `guard_methods.contains(&method_name)` — hardcoded list

---

## 5. Target Schema

### Updated `MethodBorrowInfo` (analyzer output)

```rust
pub struct MethodBorrowInfo {
    pub method: String,                    // "clone"
    pub canonical_path: Option<String>,    // "alloc::rc::Rc::clone"       ← NEW
    pub receiver_adt: Option<String>,      // "Rc"                         ← NEW
    pub trait_name: Option<String>,        // "Clone"                      ← NEW
    pub operation: Option<String>,         // "Rc::clone" (deprecated, keep for backward compat)
    pub receiver_type: String,             // "Rc<i32>"
    pub self_borrow: Option<String>,       // "immutable"
    pub is_trait_method: Option<bool>,     // true
    pub span: SpanInfo,
}
```

### New analyzer-provided fields for other heuristics

The analyzer must also provide (in `VariableTypeInfo` or new structures):
- `is_ffi: Option<bool>` — from `Function::extern_block()`
- `is_static: Option<bool>` — from `PathResolution::Def(ModuleDef::Static)`
- `is_union: Option<bool>` — from `matches!(adt, Adt::Union(_))`
- `returns_guard: Option<bool>` — from `Function::ret_type()` + ADT name check
- `source_crate: Option<String>` — from `Module::krate()` + `Crate::display_name()`

---

## 6. Code Examples

### Quick Reference — Individual API Usage

From `Semantics` (entry points):
```rust
let func: Function = sema.resolve_method_call(method_call)?;
let type_info: TypeInfo = sema.type_of_expr(&expr)?;
let resolution: PathResolution = sema.resolve_path(&path)?;
```

From `Function`:
```rust
let module: Module = func.module(db);
let fn_name: Name = func.name(db);
let ret_type: Type = func.ret_type(db);
let is_ffi: bool = func.extern_block(db).is_some();
let assoc: Option<AssocItem> = func.as_assoc_item(db);
```

From `Module` / `Crate`:
```rust
let krate: Crate = module.krate(db);
let crate_name: String = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();
let path_segments: Vec<Module> = module.path_to_root(db).collect();
```

From `Type` / `Adt`:
```rust
let receiver_type: Type = sema.type_of_expr(&receiver_expr)?.original;
let adt: Adt = receiver_type.as_adt(db)?;
let adt_name: String = adt.name(db).display_no_db(Edition::Edition2021).to_string();
let is_union: bool = matches!(adt, Adt::Union(_));
```

From `AssocItem` / `Trait`:
```rust
let trait_: Option<Trait> = func.as_assoc_item(db)?.containing_trait(db);
let trait_name: String = trait_.name(db).display_no_db(Edition::Edition2021).to_string();
```

### Building Canonical Path

```rust
fn get_canonical_path(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    method_call: &MethodCallExpr,
) -> Option<String> {
    let func = sema.resolve_method_call(method_call)?;
    let module = func.module(db);

    let mut segments: Vec<String> = module.path_to_root(db)
        .into_iter()
        .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
        .collect();
    segments.reverse();

    let krate = module.krate(db);
    let crate_name = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();
    if !crate_name.is_empty() {
        segments.insert(0, crate_name);
    }

    segments.push(func.name(db).display_no_db(Edition::Edition2021).to_string());
    Some(segments.join("::"))  // "alloc::rc::Rc::clone"
}
```

### Getting ADT Name from Receiver

```rust
fn get_receiver_adt_name(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    method_call: &MethodCallExpr,
) -> Option<String> {
    let receiver_expr = method_call.receiver()?;
    let type_info = sema.type_of_expr(&receiver_expr)?;
    let adt = type_info.original.as_adt(db)?;
    Some(adt.name(db).display_no_db(Edition::Edition2021).to_string())  // "Rc", "Option", etc.
}
```

### Checking if Union

```rust
fn is_union_type(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    expr: &ast::Expr,
) -> bool {
    sema.type_of_expr(expr)
        .and_then(|ti| ti.original.as_adt(db))
        .map(|adt| matches!(adt, Adt::Union(_)))
        .unwrap_or(false)
}
// Alternative: adt.is_union(db) also works
```

### Detecting FFI

```rust
let func = sema.resolve_method_call(method_call)?;
let is_ffi = func.extern_block(db).is_some();
```

### Detecting Static

```rust
use ra_ap_hir::PathResolution;

if let Some(PathResolution::Def(ModuleDef::Static(_))) = sema.resolve_path(&path) {
    // confirmed static variable
}
```

### Detecting Guard Return Type

```rust
let func = sema.resolve_method_call(method_call)?;
let ret_type = func.ret_type(db);
let is_guard = ret_type.as_adt(db)
    .map(|adt| adt.name(db).display_no_db(Edition::Edition2021).to_string().ends_with("Guard"))
    .unwrap_or(false);
```

### Getting Trait Name

```rust
let func = sema.resolve_method_call(method_call)?;
let trait_name = func.as_assoc_item(db)
    .and_then(|item| item.containing_trait(db))
    .map(|t| t.name(db).display_no_db(Edition::Edition2021).to_string());
```

### Checking Source Crate

```rust
let func = sema.resolve_method_call(method_call)?;
let crate_name = func.module(db).krate(db)
    .display_name(db).map(|n| n.to_string()).unwrap_or_default();
let is_borrowscope = crate_name == "borrowscope_runtime";
```

---

## 7. Files to Modify

### Analyzer

| File | Change |
|------|--------|
| `borrowscope-analyzer/src/output.rs` | Add `canonical_path`, `receiver_adt`, `trait_name` to `MethodBorrowInfo`. Add `is_ffi`, `is_static`, `is_union`, `returns_guard`, `source_crate` fields. |
| `borrowscope-analyzer/src/analysis.rs` | Populate new fields in `collect_method_borrows()` using APIs 1–17. Use existing `resolve_method_path()` function. |

### Macro

| File | Change |
|------|--------|
| `borrowscope-macro/src/transform_visitor.rs` | Replace ALL `.contains()` and `method_name ==` checks with `match canonical_path.as_deref()`. Replace tracking set fallbacks with `receiver_adt` checks. Replace `guard_methods` list with `returns_guard` field. |
| `borrowscope-macro/src/diagnostics.rs` | Remove `looks_like_ffi()`, `looks_like_static()`, `looks_like_union()`. Replace with `is_ffi`, `is_static`, `is_union` fields from analyzer. |

---

## 8. Implementation Checklist

### Implementation Steps (Overview)

1. **Update analyzer output schema** — Add `canonical_path` and `receiver_adt` fields
2. **Use `resolve_method_path()` function** — Already exists in analyzer, just need to store result
3. **Extract ADT name from receiver type** — Use `Type::as_adt()` API
4. **Add FFI/static/union detection** — Use proper rust-analyzer APIs
5. **Update macro to use exact matching** — Replace all `.contains()` with `match` statements
6. **Remove all string parsing heuristics** — No more `.contains()`, `starts_with()`, pattern matching

### Analyzer Changes
- [ ] Add `canonical_path: Option<String>` to `MethodBorrowInfo`
- [ ] Add `receiver_adt: Option<String>` to `MethodBorrowInfo`
- [ ] Add `trait_name: Option<String>` to `MethodBorrowInfo`
- [ ] Add `is_ffi`, `is_static`, `is_union`, `returns_guard`, `source_crate` fields
- [ ] Populate `canonical_path` using `resolve_method_call()` → module path → crate name
- [ ] Populate `receiver_adt` using `type_of_expr()` → `as_adt()` → `name()`
- [ ] Populate `trait_name` using `as_assoc_item()` → `containing_trait()` → `name()`
- [ ] Populate `is_ffi` using `extern_block()`
- [ ] Populate `is_static` using `resolve_path()` → `PathResolution::Def(ModuleDef::Static)`
- [ ] Populate `is_union` using `as_adt()` → `matches!(adt, Adt::Union(_))`
- [ ] Populate `returns_guard` using `ret_type()` → `as_adt()` → name ends with "Guard"
- [ ] Populate `source_crate` using `module()` → `krate()` → `display_name()`

### Macro Changes
- [ ] Replace ALL `op.contains()` checks with `match canonical_path.as_deref()`
- [ ] Replace ALL `method_name == "..."` checks with canonical path matching
- [ ] Replace `maybe_uninit_vars.contains()` fallback with `receiver_adt == "MaybeUninit"`
- [ ] Replace `once_cell_vars.contains()` fallback with `receiver_adt` check
- [ ] Replace `guard_methods.contains()` with `returns_guard` field
- [ ] Replace `fn_name.contains("track_")` with `source_crate == "borrowscope_runtime"`
- [ ] Remove `looks_like_ffi()` — use `is_ffi` field
- [ ] Remove `looks_like_static()` — use `is_static` field
- [ ] Remove `looks_like_union()` — use `is_union` field

### Verification
- [ ] Run analyzer on test project, verify `canonical_path` populated
- [ ] Run integration test — should pass
- [ ] Run unit tests — should pass
- [ ] Exhaustive search for remaining heuristics:
  ```bash
  grep -rn "\.contains\|starts_with\|ends_with" borrowscope-macro/src/*.rs | grep -v test
  grep -rn "unwrap_or\|unwrap_or_else" borrowscope-macro/src/*.rs | grep -v test
  grep -rn "looks_like\|might\|probably\|assume" borrowscope-macro/src/*.rs
  ```
- [ ] Confirm 0 heuristics remain (excluding #34 underscore convention)

### Summary Table

| API # | API | Eliminates |
|-------|-----|-----------|
| 1 | `Semantics::resolve_method_call()` | #1–25 |
| 2 | `Function::module()` | #1–25, #33 |
| 3 | `Module::path_to_root()` | #1–25 |
| 4 | `Module::krate()` | #1–25, #33 |
| 5 | `Crate::display_name()` | #1–25, #33 |
| 6 | `Function::name()` | #1–25 |
| 7 | `Semantics::type_of_expr()` | #8, #13–14, #20, #29, #31–32 |
| 8 | `Type::as_adt()` | #8, #13–14, #29, #31–32, #35 |
| 9 | `Adt::name()` | #8, #13–14, #31–32, #35 |
| 10 | `Adt` enum matching | #29 |
| 11 | `Function::as_assoc_item()` | #30 |
| 12 | `AssocItem::containing_trait()` | #30 |
| 13 | `Trait::name()` | #30 |
| 14 | `Function::extern_block()` | #26–27 |
| 15 | `Semantics::resolve_path()` | #24–25, #28 |
| 16 | `PathResolution` pattern match | #28 |
| 17 | `Function::ret_type()` | #35 |

**Result:** 17 APIs → 34 heuristics eliminated. 1 kept (#34, Rust `_` convention).

---

## Benefits

- ✅ 100% semantic — no string parsing
- ✅ Exact matching — no false positives
- ✅ Type-safe — compiler checks paths
- ✅ Maintainable — clear match statements
- ✅ Fast — no string operations at runtime
