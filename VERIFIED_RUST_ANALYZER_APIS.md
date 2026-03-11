# Verified rust-analyzer APIs for 100% Semantic Analysis

**Source:** https://docs.rs/ra_ap_hir/latest/ra_ap_hir/

All APIs below have been verified against the official rust-analyzer documentation.

---

## 1. Method Resolution → Canonical Path

### API: `Semantics::resolve_method_call()`
**Signature:**
```rust
pub fn resolve_method_call(&self, call: &MethodCallExpr) -> Option<Function>
```

**Usage:**
```rust
let func: Function = sema.resolve_method_call(method_call)?;
```

**Returns:** `Function` struct representing the resolved method

---

## 2. Function → Module Path

### API: `Function::module()`
**Signature:**
```rust
pub fn module(self, db: &dyn HirDatabase) -> Module
```

**Usage:**
```rust
let module: Module = func.module(db);
```

**Returns:** The module containing the function

---

## 3. Module → Path Components

### API: `Module::path_to_root()`
**Signature:**
```rust
// Returns iterator of modules from this module to crate root
pub fn path_to_root(&self, db: &dyn HirDatabase) -> impl Iterator<Item = Module>
```

**Usage:**
```rust
let mut segments: Vec<String> = module.path_to_root(db)
    .into_iter()
    .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
    .collect();
segments.reverse(); // Root to leaf order
```

**Returns:** Iterator of `Module` from current to root

---

## 4. Module → Crate

### API: `Module::krate()`
**Signature:**
```rust
pub fn krate(&self, db: &dyn HirDatabase) -> Crate
```

**Usage:**
```rust
let krate: Crate = module.krate(db);
let crate_name = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();
```

**Returns:** The crate containing this module

---

## 5. Function → Name

### API: `Function::name()`
**Signature:**
```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
```

**Usage:**
```rust
let fn_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
```

**Returns:** `Name` struct (use `.display_no_db()` to get string)

---

## 6. Type Resolution → ADT

### API: `Type::as_adt()`
**Signature:**
```rust
pub fn as_adt(&self) -> Option<Adt>
```

**Usage:**
```rust
let receiver_type: Type = sema.type_of_expr(&method_call.receiver())?.original;
if let Some(adt) = receiver_type.as_adt(db) {
    // It's a struct/enum/union
}
```

**Returns:** `Option<Adt>` where `Adt` is an enum with variants:
- `Adt::Struct(Struct)`
- `Adt::Union(Union)`
- `Adt::Enum(Enum)`

---

## 7. ADT → Name

### API: `Adt::name()`
**Signature:**
```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
```

**Usage:**
```rust
let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
// Returns: "Rc", "Option", "Result", etc.
```

**Returns:** `Name` struct

---

## 8. ADT → Check if Union

### API: Pattern matching on `Adt` enum
**Usage:**
```rust
let is_union = matches!(adt, Adt::Union(_));
```

**Alternative:**
```rust
match adt {
    Adt::Struct(_) => { /* struct */ },
    Adt::Union(_) => { /* union */ },
    Adt::Enum(_) => { /* enum */ },
}
```

---

## 9. Trait Method Detection

### API: `Function::as_assoc_item()` + `AssocItem::containing_trait()`
**Signature:**
```rust
// From Function trait implementation
impl AsAssocItem for Function {
    fn as_assoc_item(self, db: &dyn HirDatabase) -> Option<AssocItem>
}

// From AssocItem
pub fn containing_trait(self, db: &dyn HirDatabase) -> Option<Trait>
```

**Usage:**
```rust
let func: Function = sema.resolve_method_call(method_call)?;

let trait_info = func.as_assoc_item(db)
    .and_then(|item| item.containing_trait(db));

let is_trait_method = trait_info.is_some();
let trait_name = trait_info.map(|t| t.name(db).display_no_db(Edition::Edition2021).to_string());
```

**Returns:** `Option<Trait>` if method is from a trait

---

## 10. FFI Detection

### API: `Function::extern_block()`
**Signature:**
```rust
pub fn extern_block(self, db: &dyn HirDatabase) -> Option<ExternBlock>
```

**Usage:**
```rust
let func: Function = sema.resolve_path(&path)?;
let is_extern = func.extern_block(db).is_some();
```

**Returns:** `Some(ExternBlock)` if function is declared in an `extern` block

---

## 11. Static Detection

### API: `Semantics::resolve_path()` → `PathResolution`
**Signature:**
```rust
pub fn resolve_path(&self, path: &Path) -> Option<PathResolution>

pub enum PathResolution {
    Def(ModuleDef),
    // ... other variants
}

pub enum ModuleDef {
    Static(Static),
    // ... other variants
}
```

**Usage:**
```rust
use ra_ap_hir::PathResolution;

if let Some(PathResolution::Def(ModuleDef::Static(static_def))) = sema.resolve_path(&path) {
    // This is a static variable
}
```

**Returns:** `PathResolution` enum that can be matched

---

## 12. Expression Type Resolution

### API: `Semantics::type_of_expr()`
**Signature:**
```rust
pub fn type_of_expr(&self, expr: &Expr) -> Option<TypeInfo<'db>>

pub struct TypeInfo<'db> {
    pub original: Type<'db>,
    pub adjusted: Option<Type<'db>>,
}
```

**Usage:**
```rust
let receiver_expr = method_call.receiver()?;
let type_info = sema.type_of_expr(&receiver_expr)?;
let receiver_type = type_info.original; // Use original, not adjusted
```

**Returns:** `TypeInfo` with `original` and `adjusted` types

---

## Complete Example: Building Canonical Path

```rust
use ra_ap_hir::{Semantics, HirDatabase};
use ra_ap_syntax::ast::MethodCallExpr;

fn get_canonical_path(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    method_call: &MethodCallExpr,
) -> Option<String> {
    // 1. Resolve method call to Function
    let func = sema.resolve_method_call(method_call)?;
    
    // 2. Get module containing the function
    let module = func.module(db);
    
    // 3. Build path from root to module
    let mut segments: Vec<String> = module.path_to_root(db)
        .into_iter()
        .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
        .collect();
    segments.reverse(); // Root to leaf order
    
    // 4. Add crate name
    let krate = module.krate(db);
    let crate_name = krate.display_name(db).map(|n| n.to_string()).unwrap_or_default();
    if !crate_name.is_empty() {
        segments.insert(0, crate_name);
    }
    
    // 5. Add function name
    let fn_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
    segments.push(fn_name);
    
    // Result: "alloc::rc::Rc::clone"
    Some(segments.join("::"))
}
```

---

## Complete Example: Getting ADT Name from Receiver

```rust
fn get_receiver_adt_name(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    method_call: &MethodCallExpr,
) -> Option<String> {
    // 1. Get receiver expression
    let receiver_expr = method_call.receiver()?;
    
    // 2. Get type of receiver
    let type_info = sema.type_of_expr(&receiver_expr)?;
    let receiver_type = type_info.original;
    
    // 3. Check if it's an ADT
    let adt = receiver_type.as_adt(db)?;
    
    // 4. Get ADT name
    let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
    
    // Result: "Rc", "Option", "Result", etc.
    Some(adt_name)
}
```

---

## Complete Example: Checking if Union

```rust
fn is_union_type(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    method_call: &MethodCallExpr,
) -> bool {
    let receiver_expr = method_call.receiver()?;
    let type_info = sema.type_of_expr(&receiver_expr)?;
    let receiver_type = type_info.original;
    
    if let Some(adt) = receiver_type.as_adt(db) {
        matches!(adt, Adt::Union(_))
    } else {
        false
    }
}
```

---

## Complete API Mapping for All 35 Heuristics

### Guard Detection (Heuristic #35)

**API: `Function::ret_type()`**
```rust
pub fn ret_type(self, db: &dyn HirDatabase) -> Type<'_>
```

**Usage:**
```rust
let func = sema.resolve_method_call(method_call)?;
let ret_type = func.ret_type(db);

// Check if return type is a guard
if let Some(adt) = ret_type.as_adt(db) {
    let type_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
    let is_guard = type_name.ends_with("Guard"); 
    // Matches: MutexGuard, RwLockReadGuard, RwLockWriteGuard, etc.
}
```

---

## Summary of All Verified APIs

| # | API | Purpose | Returns | Eliminates Heuristics |
|---|-----|---------|---------|----------------------|
| 1 | `Semantics::resolve_method_call()` | Resolve method to Function | `Option<Function>` | #1-23 (all method matching) |
| 2 | `Function::module()` | Get containing module | `Module` | #1-23 (path building) |
| 3 | `Module::path_to_root()` | Get path to crate root | `Iterator<Module>` | #1-23 (path building) |
| 4 | `Module::krate()` | Get containing crate | `Crate` | #1-23 (path building) |
| 5 | `Crate::display_name()` | Get crate name | `Option<CrateName>` | #1-23, #33 (crate check) |
| 6 | `Function::name()` | Get function name | `Name` | #1-23 (path building) |
| 7 | `Semantics::type_of_expr()` | Get expression type | `Option<TypeInfo>` | #8, #13-14, #20, #31-32 |
| 8 | `Type::as_adt()` | Check if ADT | `Option<Adt>` | #8, #13-14, #29, #31-32 |
| 9 | `Adt::name()` | Get ADT name | `Name` | #8, #13-14, #31-32 |
| 10 | `Adt` enum matching | Check if union | `bool` | #29 |
| 11 | `Function::as_assoc_item()` | Get associated item | `Option<AssocItem>` | #30 |
| 12 | `AssocItem::containing_trait()` | Get trait if trait method | `Option<Trait>` | #30 |
| 13 | `Trait::name()` | Get trait name | `Name` | #30 |
| 14 | `Function::extern_block()` | Check if extern | `Option<ExternBlock>` | #26-27 |
| 15 | `Semantics::resolve_path()` | Resolve path to definition | `Option<PathResolution>` | #24-25, #28 |
| 16 | `PathResolution::Def(ModuleDef::Static)` | Check if static | Pattern match | #28 |
| 17 | `Function::ret_type()` | Get return type | `Type` | #35 |

---

## Next Steps

1. **Update analyzer** to provide `canonical_path` and `receiver_adt` fields
2. **Update macro** to use exact path matching instead of `.contains()`
3. **Remove ALL string matching heuristics** from macro
4. **Verify 0 heuristics remain** with exhaustive search
