## 3. Implementation Details

### 3.1 rust-analyzer Integration

The borrowscope-analyzer leverages rust-analyzer's semantic analysis engine through its published crate ecosystem. These crates, prefixed with `ra_ap_`, provide the same analysis capabilities that power IDE features like go-to-definition, type hints, and refactoring. By using these crates directly, we obtain production-grade type resolution without reimplementing Rust's complex type system.

The analyzer depends on seven core crates from the rust-analyzer project:

```toml
ra_ap_hir = "0.0.318"          # High-level intermediate representation
ra_ap_hir_ty = "0.0.318"       # Type inference and attach_db for thread-local DB
ra_ap_ide_db = "0.0.318"       # IDE database infrastructure  
ra_ap_load-cargo = "0.0.318"   # Cargo workspace loading
ra_ap_project_model = "0.0.318" # Project structure modeling
ra_ap_syntax = "0.0.318"       # Syntax tree representation
ra_ap_vfs = "0.0.318"          # Virtual file system
```

The `ra_ap_hir` crate provides the `Semantics` struct, which serves as the primary API for semantic queries. Given a syntax node, `Semantics` can resolve its type, determine trait implementations, and navigate semantic relationships.

**Important (0.0.318+)**: Starting with version 0.0.318, rust-analyzer uses thread-local storage for database attachment. All code that calls `display()` or other methods requiring database access must be wrapped with `attach_db`:

```rust
use ra_ap_hir::{Semantics, HirDisplay};
use ra_ap_hir_ty::attach_db;
use ra_ap_ide_db::RootDatabase;

let sema = Semantics::new(&db);

// Wrap analysis code with attach_db
let results = attach_db(&db, || {
    // Get the type of a pattern (variable binding)
    if let Some(type_info) = sema.type_of_pat(&pattern) {
        let ty = type_info.original;
        
        // Display the type as a string (requires attached db)
        let type_string = ty.display(db, DisplayTarget::from_crate(db, krate)).to_string();
        
        // Query type properties
        let is_copy = ty.is_copy(db);           // Does it implement Copy?
        let is_reference = ty.is_reference();    // Is it &T or &mut T?
        let is_mutable_ref = ty.is_mutable_reference();
        let is_raw_ptr = ty.is_raw_ptr();       // Is it *const T or *mut T?
    }
});
```

Note the API changes from 0.0.232:
- `display(db, Edition::Edition2021)` → `display(db, DisplayTarget::from_crate(db, krate))`
- `db.lang_item(krate, LangItem::X)` → `lang_items(db, krate).X`
- `module.krate()` → `module.krate(db)`
```

The `type_of_pat` method is particularly important. An earlier implementation used `type_of_expr` on the initializer expression, but this returned the type of the expression before coercion. For example, in `let ptr: *const i32 = &value;`, the expression `&value` has type `&i32`, but the pattern `ptr` has type `*const i32` after implicit coercion. Using `type_of_pat` correctly captures the variable's actual type after all coercions are applied.

### 3.2 Workspace Loading

Before semantic analysis can occur, rust-analyzer must load the project's workspace. This involves parsing `Cargo.toml`, resolving dependencies, and crucially, locating the Rust standard library (sysroot). The sysroot contains pre-compiled metadata for `std`, `core`, `alloc`, and other standard crates.

Without sysroot discovery, types from the standard library resolve to `{unknown}`. This was a critical issue during development—initial tests showed only 10% type resolution because `String`, `Vec`, `Rc`, and other standard types could not be resolved.

The fix required enabling sysroot discovery in the cargo configuration:

```rust
use ra_ap_project_model::{CargoConfig, RustLibSource};
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};

let mut cargo_config = CargoConfig::default();
// Enable automatic sysroot discovery
cargo_config.sysroot = Some(RustLibSource::Discover);

let load_config = LoadCargoConfig {
    load_out_dirs_from_check: true,      // Load build script outputs
    with_proc_macro_server: ProcMacroServerChoice::None,  // Skip proc-macro expansion
    prefill_caches: true,                // Prefill analysis caches
};

let (db, vfs, _proc_macros) = load_workspace_at(
    project_path,
    &cargo_config,
    &load_config,
    &|msg| { /* progress callback */ },
)?;
```

The `RustLibSource::Discover` setting instructs rust-analyzer to locate the sysroot by querying `rustc --print sysroot`. This finds the standard library metadata regardless of how Rust was installed (rustup, system package, custom toolchain).

The `load_out_dirs_from_check` option is important for projects using build scripts. When enabled, rust-analyzer runs `cargo check` to obtain build script outputs, which may include generated code or environment variables that affect type resolution.

Proc-macro expansion is disabled (`ProcMacroServerChoice::None`) because we analyze the source before macro expansion. This is intentional—we want the types as they appear in the user's source code, not after transformation by other macros.

### 3.3 Type Extraction

With the workspace loaded, the analyzer walks each source file's syntax tree to find variable bindings. The extraction process operates on `let` statements, extracting the pattern (variable name), its resolved type, and source location.

```rust
use ra_ap_syntax::{ast, AstNode, SyntaxKind, SourceFile};

fn extract_with_semantics(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &SourceFile,
    file_path: &str,
    variables: &mut Vec<VariableTypeInfo>,
) {
    for node in source_file.syntax().descendants() {
        // Only process let statements
        if node.kind() != SyntaxKind::LET_STMT {
            continue;
        }
        
        let let_stmt = ast::LetStmt::cast(node)?;
        let pat = let_stmt.pat()?;
        
        // Calculate source location
        let range = pat.syntax().text_range();
        let (line, column) = calculate_position(source_file, range.start());
        
        let mut var_info = VariableTypeInfo::new(
            pat.syntax().text().to_string(),
            file_path.to_string(),
            line,
            column,
        );
        
        // Get type from semantic analysis
        if let Some(type_info) = sema.type_of_pat(&pat) {
            let ty = type_info.original;
            var_info.ty = ty.display(db, display_target.clone()).to_string();
            var_info.is_copy = ty.is_copy(db);
            var_info.is_reference = ty.is_reference();
            var_info.is_mutable_reference = ty.is_mutable_reference();
            var_info.is_raw_ptr = ty.is_raw_ptr();
            
            // Classify smart pointers and collections by type name
            classify_type(&mut var_info);
        }
        
        variables.push(var_info);
    }
}
```

The `classify_type` function uses **semantic analysis** via rust-analyzer's type system APIs to set classification flags. This approach is fully semantic—no string heuristics:

```rust
fn populate_type_info(var_info: &mut VariableTypeInfo, ty: &ra_ap_hir::Type, db: &RootDatabase) {
    // === Trait implementations (semantic via impls_trait) ===
    var_info.is_copy = ty.is_copy(db);  // Direct API
    
    // Lookup traits via lang items and check implementation
    if let Some(clone_trait) = lang_items(db, krate_id).clone_trait() {
        var_info.is_clone = ty.impls_trait(db, clone_trait.into(), &[]);
    }
    // Same pattern for: Drop, Sync, Sized, Future, Iterator
    
    // Send trait (not a lang item) - found via import_map search
    if let Some(send_trait) = find_send_trait(db, krate) {
        var_info.is_send = ty.impls_trait(db, send_trait, &[]);
    }
    
    // === Type structure (semantic via Type methods) ===
    var_info.is_reference = ty.is_reference();
    var_info.is_mutable_reference = ty.is_mutable_reference();
    var_info.is_raw_ptr = ty.is_raw_ptr();
    var_info.is_closure = ty.is_closure();
    var_info.is_fn_ptr = ty.is_fn();
    
    // Slice detection - checks inner type for &[T], Box<[T]>, etc.
    var_info.is_slice = ty.is_slice() || ty.strip_reference().is_slice()
        || ty.type_arguments().any(|inner| inner.is_slice());
    
    // Primitive detection via builtin type API
    if let Some(builtin) = ty.as_builtin() {
        var_info.is_primitive = builtin.is_int() || builtin.is_uint() || builtin.is_float() 
            || builtin.is_char() || builtin.is_bool() || builtin.is_str();
    }
    
    // === ADT classification (semantic via canonical path) ===
    if let Some(adt) = ty.as_adt() {
        var_info.is_union = matches!(adt, Adt::Union(_));
        
        // Get canonical path like "alloc::rc::Rc" or "std::sync::Mutex"
        if let Some(path) = get_adt_path(&adt, db) {
            classify_by_path(var_info, &path);
        }
    }
    
    // Trait object detection - checks inner type for &dyn T, Box<dyn T>, etc.
    var_info.is_dyn_trait = ty.as_dyn_trait().is_some() 
        || ty.strip_reference().as_dyn_trait().is_some()
        || ty.type_arguments().any(|inner| inner.as_dyn_trait().is_some());
}

fn classify_by_path(var_info: &mut VariableTypeInfo, path: &str) {
    // Exact path matching - no string heuristics
    var_info.is_rc = path == "alloc::rc::Rc" || path == "std::rc::Rc";
    var_info.is_arc = path == "alloc::sync::Arc" || path == "std::sync::Arc";
    var_info.is_mutex = path == "std::sync::Mutex" || path == "std::sync::poison::mutex::Mutex";
    // ... etc for all ADT types
}
```

This semantic classification is reliable because it uses rust-analyzer's type resolution APIs directly:

1. **Trait detection**: Uses `ty.impls_trait()` with traits looked up via `LangItem` or `import_map` search
2. **Type structure**: Uses `Type` methods like `is_reference()`, `is_closure()`, `is_slice()`, `as_dyn_trait()`
3. **Primitive detection**: Uses `ty.as_builtin()` methods
4. **ADT classification**: Uses exact canonical path matching from `get_adt_path()`

A variable initialized with `create_shared(value)` that returns `Rc<T>` will have its type resolved to `Rc<SomeType, Global>`, and the ADT path will be `alloc::rc::Rc`, correctly identifying it as an `Rc`.

The analyzer also handles files that are not part of the crate graph (e.g., standalone `.rs` files or files excluded from compilation). For these files, it falls back to syntax-only analysis using explicit type annotations when available:

```rust
if let Some(ty_annotation) = let_stmt.ty() {
    var_info.ty = ty_annotation.syntax().text().to_string();
    classify_type(&mut var_info);
}
```

This fallback ensures the analyzer produces useful output even for files that rust-analyzer cannot fully analyze, though the type information will be limited to what is explicitly annotated in the source.

---

