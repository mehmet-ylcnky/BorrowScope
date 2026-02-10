# borrowscope-analyzer

> Static type analyzer for BorrowScope using rust-analyzer

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Solution Architecture](#2-solution-architecture)
3. [Implementation Details](#3-implementation-details)
   - [3.1 rust-analyzer Integration](#31-rust-analyzer-integration)
   - [3.2 Workspace Loading](#32-workspace-loading)
   - [3.3 Type Extraction](#33-type-extraction)
4. [Output Format](#4-output-format)
5. [Integration with borrowscope-macro](#5-integration-with-borrowscope-macro)
   - [5.1 Type Information Lookup](#type-information-lookup)
   - [5.2 Enhanced Tracking Function Selection](#enhanced-tracking-function-selection)
   - [5.3 The Complete BorrowScope Pipeline](#the-complete-borrowscope-pipeline)
6. [Usage Guide](#6-usage-guide)
   - [6.1 Running the Analyzer](#61-running-the-analyzer)
   - [6.2 Workflow for Users](#62-workflow-for-users)
7. [Performance Characteristics](#7-performance-characteristics)
8. [Limitations & Future Work](#8-limitations--future-work)
9. [Future Development Roadmap](#9-future-development-roadmap)
   - [9.1 Current Coverage Analysis](#91-current-coverage-analysis)
   - [9.2 Architecture Evolution](#92-architecture-evolution)
   - [9.3 Implementation Phases](#93-implementation-phases)
   - [9.4 Schema Evolution](#94-schema-evolution)
   - [9.5 Macro Refactoring](#95-macro-refactoring)
   - [9.6 Success Criteria](#96-success-criteria)
10. [Dependencies](#10-dependencies)

---

## 1. Problem Statement

BorrowScope's `#[trace_borrow]` procedural macro automatically instruments Rust functions to track ownership events at runtime. The macro transforms source code by injecting tracking calls around variable bindings, borrows, and drops. However, procedural macros in Rust operate under a fundamental constraint: they execute during the early stages of compilation, before type inference and resolution have occurred.

### The Proc-Macro Type Blindness Problem

When `rustc` invokes a procedural macro, it provides only the raw token stream of the annotated item. At this point in the compilation pipeline, the compiler has not yet performed type checking. Consider the following function:

```rust
#[trace_borrow]
fn example() {
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));
    let borrowed = data.borrow();
    process(&borrowed);
}
```

The `#[trace_borrow]` macro receives tokens representing `Rc::new(RefCell::new(vec![1, 2, 3]))` but has no way to determine:

1. That `data` has type `Rc<RefCell<Vec<i32>>>`
2. That `Rc<T>` is a reference-counted smart pointer requiring `track_rc_new`
3. That the inner `RefCell<T>` provides interior mutability
4. That `borrowed` is a `Ref<Vec<i32>>` guard from `RefCell::borrow()`
5. Whether any of these types implement `Copy`

The macro can only perform syntactic pattern matching on the token stream. It can recognize `Rc::new(...)` by matching the literal tokens, but this approach fails for:

```rust
let data = std::rc::Rc::new(value);     // Different path
let data = MyRc::new(value);            // Type alias
let data = create_shared(value);        // Factory function returning Rc
let data = if cond { Rc::new(a) } else { Rc::new(b) };  // Conditional
```

### Compilation Pipeline and Type Resolution Timing

The Rust compilation process follows a strict ordering where macro expansion precedes type resolution:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RUST COMPILATION PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   PARSING   │───▶│    MACRO    │───▶│    NAME     │───▶│    TYPE     │  │
│  │             │    │  EXPANSION  │    │ RESOLUTION  │    │  CHECKING   │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│        │                  │                  │                  │          │
│        ▼                  ▼                  ▼                  ▼          │
│   Token Stream      Expanded AST        Resolved        Typed HIR         │
│                                          Names                             │
│                                                                             │
│                     ▲                                                       │
│                     │                                                       │
│              #[trace_borrow]                                                │
│              EXECUTES HERE                                                  │
│                                                                             │
│              ✗ No type information available                                │
│              ✗ No trait implementation data                                 │
│              ✗ No generic instantiation info                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

This architectural constraint is not a limitation of BorrowScope's implementation but rather a fundamental property of Rust's compilation model. Procedural macros are intentionally isolated from type information to maintain compilation determinism and enable parallel macro expansion.

### Consequences for BorrowScope

Without type information, `borrowscope-macro` must rely on heuristic pattern matching to select appropriate tracking functions. The current implementation uses syntactic patterns:

```rust
// In borrowscope-macro's transform_visitor.rs
if text.contains("Rc::new") {
    // Assume this is Rc<T> and use track_rc_new
}
```

This heuristic approach leads to several failure modes:

**False Negatives**: The macro fails to recognize smart pointers created through non-standard patterns, resulting in generic `track_new` calls instead of specialized `track_rc_new` or `track_arc_new` calls. This loses semantic information about reference counting behavior.

**False Positives**: A variable named `Rc_new_value` or a comment containing `Rc::new` could theoretically trigger incorrect classification.

**Missing Copy Semantics**: The `Copy` trait fundamentally changes ownership semantics—copying instead of moving. Without knowing whether a type implements `Copy`, the macro cannot accurately represent whether an assignment transfers ownership or creates a copy.

**Incomplete Smart Pointer Coverage**: Types like `Weak<T>`, `MutexGuard<T>`, `RwLockReadGuard<T>`, and user-defined smart pointers cannot be detected through syntax alone.

The borrowscope-analyzer addresses these limitations by performing semantic analysis as a separate build step, extracting complete type information that the macro can consume at expansion time.

---

## 2. Solution Architecture

The borrowscope-analyzer implements a two-phase compilation strategy that decouples type analysis from macro expansion. By running semantic analysis as a pre-build step, we extract type information into a structured format that the procedural macro can consume during its execution. This approach works within Rust's compilation model rather than against it.

### Two-Phase Build Strategy

The solution introduces an explicit analysis phase before the standard Cargo build:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      TWO-PHASE BUILD STRATEGY                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PHASE 1: STATIC ANALYSIS (borrowscope-analyzer)                            │
│  ════════════════════════════════════════════════                           │
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │  Cargo.toml  │───▶│rust-analyzer │───▶│ type-info.json│                  │
│  │  src/*.rs    │    │   engine     │    │              │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│                                                                             │
│        User's                Full semantic          Extracted type          │
│        project               analysis with          metadata for            │
│        source                type resolution        all variables           │
│                                                                             │
│                                    │                                        │
│                                    ▼                                        │
│  PHASE 2: INSTRUMENTED BUILD (cargo build)                                  │
│  ═════════════════════════════════════════                                  │
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │  src/*.rs    │───▶│#[trace_borrow]───▶│ Instrumented │                   │
│  │              │    │    macro     │    │    binary    │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│                             │                                               │
│                             │ reads                                         │
│                             ▼                                               │
│                      ┌──────────────┐                                       │
│                      │type-info.json│                                       │
│                      └──────────────┘                                       │
│                                                                             │
│        Macro now has complete type information for accurate tracking        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

This architecture preserves the standard Rust compilation model while augmenting it with pre-computed type information. The analyzer runs independently of `rustc`, using the same semantic analysis engine that powers rust-analyzer IDE features.

### Component Overview

The solution consists of three interconnected components:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMPONENT ARCHITECTURE                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     borrowscope-analyzer                            │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │    │
│  │  │   main.rs   │  │ analysis.rs │  │  output.rs  │                  │    │
│  │  │             │  │             │  │             │                  │    │
│  │  │ CLI entry   │  │ Semantic    │  │ JSON        │                  │    │
│  │  │ point       │  │ analysis    │  │ serialization│                 │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                  │    │
│  │         │                │                │                         │    │
│  │         └────────────────┴────────────────┘                         │    │
│  │                          │                                          │    │
│  │                          ▼                                          │    │
│  │              ┌───────────────────────┐                              │    │
│  │              │  ra_ap_* crates       │                              │    │
│  │              │  (rust-analyzer libs) │                              │    │
│  │              └───────────────────────┘                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                          │                                                  │
│                          │ writes                                           │
│                          ▼                                                  │
│                ┌───────────────────────┐                                    │
│                │  .borrowscope/        │                                    │
│                │    type-info.json     │                                    │
│                └───────────────────────┘                                    │
│                          │                                                  │
│                          │ reads                                            │
│                          ▼                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     borrowscope-macro                               │    │
│  │                                                                     │    │
│  │  #[trace_borrow] ──▶ lookup type by file:line:col ──▶ emit correct │    │
│  │                      from type-info.json              tracking call │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**borrowscope-analyzer** is a standalone binary that loads a Rust project using rust-analyzer's workspace loading infrastructure. It performs full semantic analysis, extracting type information for every variable binding, and writes the results to a JSON file in the project's `.borrowscope/` directory.

**type-info.json** serves as the bridge between static analysis and macro expansion. It contains a structured representation of every variable's type, including classification flags for smart pointers, interior mutability types, and Copy semantics. The file is keyed by source location (file path, line, column) enabling precise lookup during macro expansion.

**borrowscope-macro** (to be enhanced) will read the type information file at macro expansion time. When transforming a `let` binding, it looks up the variable's location in the JSON file to retrieve complete type information, enabling accurate selection of tracking functions.

### Design Rationale

Several alternative approaches were considered before settling on this architecture:

**Compiler Plugin**: A rustc plugin could access type information directly during compilation. However, compiler plugins are unstable, require nightly Rust, and couple tightly to rustc internals that change frequently.

**Build Script Integration**: A `build.rs` script could theoretically perform analysis, but build scripts execute before compilation and cannot access the type-checked HIR. They also cannot easily invoke rust-analyzer's analysis infrastructure.

**Runtime Reflection**: Rust intentionally lacks runtime reflection. While `std::any::TypeId` exists, it cannot be used at compile time and provides only type identity, not structural information.

**Separate Analysis Tool**: The chosen approach uses rust-analyzer's published crates (`ra_ap_*`) which provide stable, well-maintained APIs for semantic analysis. These crates are the same ones powering the rust-analyzer IDE, ensuring correctness and compatibility with Rust's evolving type system.

The two-phase approach adds a build step but provides complete type information without requiring unstable features or compiler modifications. It integrates cleanly with existing Cargo workflows and can be automated through build scripts or CI pipelines.

---

## 3. Implementation Details

### 3.1 rust-analyzer Integration

The borrowscope-analyzer leverages rust-analyzer's semantic analysis engine through its published crate ecosystem. These crates, prefixed with `ra_ap_`, provide the same analysis capabilities that power IDE features like go-to-definition, type hints, and refactoring. By using these crates directly, we obtain production-grade type resolution without reimplementing Rust's complex type system.

The analyzer depends on six core crates from the rust-analyzer project:

```toml
ra_ap_hir = "0.0.232"          # High-level intermediate representation
ra_ap_ide_db = "0.0.232"       # IDE database infrastructure  
ra_ap_load-cargo = "0.0.232"   # Cargo workspace loading
ra_ap_project_model = "0.0.232" # Project structure modeling
ra_ap_syntax = "0.0.232"       # Syntax tree representation
ra_ap_vfs = "0.0.232"          # Virtual file system
```

The `ra_ap_hir` crate provides the `Semantics` struct, which serves as the primary API for semantic queries. Given a syntax node, `Semantics` can resolve its type, determine trait implementations, and navigate semantic relationships. The key methods used by the analyzer include:

```rust
use ra_ap_hir::{Semantics, HirDisplay};
use ra_ap_ide_db::RootDatabase;

let sema = Semantics::new(&db);

// Get the type of a pattern (variable binding)
if let Some(type_info) = sema.type_of_pat(&pattern) {
    let ty = type_info.original;
    
    // Display the type as a string
    let type_string = ty.display(db, Edition::Edition2021).to_string();
    
    // Query type properties
    let is_copy = ty.is_copy(db);           // Does it implement Copy?
    let is_reference = ty.is_reference();    // Is it &T or &mut T?
    let is_mutable_ref = ty.is_mutable_reference();
    let is_raw_ptr = ty.is_raw_ptr();       // Is it *const T or *mut T?
}
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
            var_info.ty = ty.display(db, Edition::Edition2021).to_string();
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
    if let Some(clone_trait) = db.lang_item(krate_id, LangItem::Clone).and_then(|li| li.as_trait()) {
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

## 4. Output Format

The analyzer produces a JSON file containing type information for all variable bindings in the project. This file is written to `.borrowscope/type-info.json` relative to the project root, creating the directory if it does not exist.

### Schema Structure

The output follows a hierarchical structure with project-level metadata and per-file variable information:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         type-info.json SCHEMA (v2.3)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  {                                                                          │
│    "version": "2.3",              ◄─── Schema version for compatibility     │
│    "analyzer_version": "0.1.0",   ◄─── Analyzer binary version              │
│    "files": {                     ◄─── Map: relative path → variables       │
│      "src/main.rs": [                                                       │
│        {                                                                    │
│          "name": "data",          ◄─── Variable name from source            │
│          "ty": "Rc<RefCell<Vec<i32>>>",  ◄─── Fully resolved type          │
│                                                                             │
│          // === Trait implementations (semantic via impls_trait) ===        │
│          "is_copy": false,        ◄─── Copy trait (ty.is_copy)              │
│          "is_clone": true,        ◄─── Clone trait (impls_trait)            │
│          "is_send": false,        ◄─── Send trait (import_map lookup)       │
│          "is_sync": false,        ◄─── Sync trait (impls_trait)             │
│          "is_drop": true,         ◄─── Drop trait (impls_trait)             │
│          "is_sized": true,        ◄─── Sized trait (impls_trait)            │
│          "is_future": false,      ◄─── Future trait (impls_trait)           │
│          "is_iterator": false,    ◄─── Iterator trait (impls_trait)         │
│                                                                             │
│          // === Type structure (semantic via Type methods) ===              │
│          "is_primitive": false,   ◄─── i32, bool, char, etc. (as_builtin)   │
│          "is_reference": false,   ◄─── &T or &mut T (is_reference)          │
│          "is_mutable_reference": false,  ◄─── &mut T (is_mutable_reference) │
│          "is_raw_ptr": false,     ◄─── *const T or *mut T (is_raw_ptr)      │
│          "is_slice": false,       ◄─── [T], &[T], Box<[T]> (is_slice)       │
│          "is_str": false,         ◄─── str type (as_builtin.is_str)         │
│          "is_closure": false,     ◄─── Closure type (is_closure)            │
│          "is_fn_ptr": false,      ◄─── fn(...) -> ... (is_fn)               │
│          "is_dyn_trait": false,   ◄─── dyn Trait, &dyn T, Box<dyn T>        │
│          "is_union": false,       ◄─── Union type (as_adt + Adt::Union)     │
│                                                                             │
│          // === ADT classification (semantic via canonical path) ===        │
│          "is_rc": true,           ◄─── alloc::rc::Rc                        │
│          "is_arc": false,         ◄─── alloc::sync::Arc                     │
│          "is_box": false,         ◄─── alloc::boxed::Box                    │
│          "is_weak": false,        ◄─── alloc::rc::Weak or alloc::sync::Weak │
│          "is_refcell": true,      ◄─── core::cell::RefCell                  │
│          "is_cell": false,        ◄─── core::cell::Cell                     │
│          "is_mutex": false,       ◄─── std::sync::Mutex                     │
│          "is_rwlock": false,      ◄─── std::sync::RwLock                    │
│          "is_guard": false,       ◄─── MutexGuard, Ref, RefMut, etc.        │
│          "is_vec": true,          ◄─── alloc::vec::Vec                      │
│          "is_string": false,      ◄─── alloc::string::String                │
│          "is_option": false,      ◄─── core::option::Option                 │
│          "is_result": false,      ◄─── core::result::Result                 │
│          "is_pin": false,         ◄─── core::pin::Pin                       │
│          "is_cow": false,         ◄─── alloc::borrow::Cow                   │
│          "is_once_cell": false,   ◄─── core::cell::OnceCell (v2.1+)         │
│          "is_maybe_uninit": false,◄─── core::mem::MaybeUninit (v2.1+)       │
│          "is_channel": false,     ◄─── mpsc::Sender/Receiver (v2.1+)        │
│          "is_extern_type": false, ◄─── c_void, CStr, CString, OsStr, etc.   │
│                                                                             │
│          // === Declaration type ===                                        │
│          "is_static": false,      ◄─── static declaration                   │
│          "is_const": false,       ◄─── const declaration                    │
│                                                                             │
│          // === Binding patterns for macro transformation ===               │
│          "is_tuple_binding": false,  ◄─── let (a, b) = ...                  │
│          "is_mut_binding": false,    ◄─── let mut x = ...                   │
│          "is_impl_trait": false,     ◄─── impl Trait type                   │
│                                                                             │
│          // === Initializer pattern (v2.1+) ===                             │
│          "initializer_kind": "rc_new",  ◄─── Semantic init pattern          │
│                                                                             │
│          // === Source location ===                                         │
│          "file": "src/main.rs",                                             │
│          "line": 15,                                                        │
│          "column": 8,                                                       │
│          "span_start": 1234,      ◄─── Byte offset start                    │
│          "span_end": 1238,        ◄─── Byte offset end                      │
│                                                                             │
│          // === Disambiguation (v2.2+) ===                                  │
│          "scope_id": 5,           ◄─── Scope identifier                     │
│          "function_name": "example",  ◄─── Containing function name         │
│          "decl_index": 2          ◄─── Declaration order in function        │
│        },                                                                   │
│        ...                                                                  │
│      ]                                                                      │
│    },                                                                       │
│    "by_name": {                   ◄─── Index by variable name (v2.1+)       │
│      "data": [ ... ],             ◄─── All variables named "data"           │
│      ...                                                                    │
│    },                                                                       │
│    "by_function": {               ◄─── Index by function+name (v2.2+)       │
│      "example": {                 ◄─── Function name                        │
│        "data": [ ... ],           ◄─── Variables in that function           │
│        ...                                                                  │
│      },                                                                     │
│      ...                                                                    │
│    }                                                                        │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Field Descriptions

The `VariableTypeInfo` structure captures comprehensive type metadata for each variable binding:

**Identity Fields**

The `name` field contains the variable name exactly as it appears in source code. For simple bindings like `let x = ...`, this is `"x"`. For pattern bindings like `let (a, b) = ...`, the current implementation captures the entire pattern text; future versions may decompose patterns into individual bindings.

The `file`, `line`, and `column` fields provide the precise source location of the binding. Line numbers are 1-indexed to match editor conventions. Column numbers indicate the start of the pattern within the line. These coordinates enable the macro to look up type information by source location.

**Type String**

The `ty` field contains the fully resolved type as a string. This includes generic parameters with their concrete types and any allocator parameters. Examples:

| Source Expression | Resolved Type |
|-------------------|---------------|
| `let x = 42;` | `i32` |
| `let s = String::from("hello");` | `String` |
| `let v = vec![1, 2, 3];` | `Vec<i32, Global>` |
| `let rc = Rc::new(value);` | `Rc<MyStruct, Global>` |
| `let nested = Rc::new(RefCell::new(vec![]));` | `Rc<RefCell<Vec<i32, Global>>, Global>` |
| `let ptr: *const i32 = &x;` | `*const i32` |
| `let closure = \|x\| x + 1;` | `impl Fn(i32) -> i32` |
| `let guard = mutex.lock().unwrap();` | `MutexGuard<'_, i32>` |
| `let future = async { 42 };` | `impl Future<Output = i32>` |

The `Global` allocator parameter appears because rust-analyzer displays the full type including default generic parameters. This verbosity is intentional—it provides complete type information without ambiguity.

**Classification Flags**

Boolean flags provide quick classification using semantic analysis (no string parsing):

| Flag | Detection Method | Purpose |
|------|------------------|---------|
| `is_copy` | `ty.is_copy(db)` | Copy trait - determines move vs copy semantics |
| `is_clone` | `ty.impls_trait(Clone)` | Clone trait implementation |
| `is_send` | `ty.impls_trait(Send)` via import_map | Thread-safe ownership transfer |
| `is_sync` | `ty.impls_trait(Sync)` | Thread-safe shared reference |
| `is_drop` | `ty.impls_trait(Drop)` | Custom destructor |
| `is_sized` | `ty.impls_trait(Sized)` | Compile-time known size |
| `is_future` | `ty.impls_trait(Future)` | Async/Future type |
| `is_iterator` | `ty.impls_trait(Iterator)` | Iterator type |
| `is_primitive` | `ty.as_builtin()` | i32, bool, char, f64, etc. |
| `is_reference` | `ty.is_reference()` | &T or &mut T |
| `is_mutable_reference` | `ty.is_mutable_reference()` | &mut T |
| `is_raw_ptr` | `ty.is_raw_ptr()` | *const T or *mut T |
| `is_slice` | `ty.is_slice()` + inner type check | [T], &[T], Box<[T]> |
| `is_str` | `ty.as_builtin().is_str()` | str type |
| `is_closure` | `ty.is_closure()` | Closure type |
| `is_fn_ptr` | `ty.is_fn()` | fn(...) -> ... |
| `is_dyn_trait` | `ty.as_dyn_trait()` + inner type check | dyn Trait, &dyn T, Box<dyn T> |
| `is_union` | `ty.as_adt()` + `Adt::Union` | Union types including MaybeUninit |
| `is_rc` | ADT path == `alloc::rc::Rc` | Reference-counted pointer |
| `is_arc` | ADT path == `alloc::sync::Arc` | Atomic reference-counted pointer |
| `is_box` | ADT path == `alloc::boxed::Box` | Heap allocation |
| `is_weak` | ADT path == `alloc::rc::Weak` or `alloc::sync::Weak` | Weak reference |
| `is_refcell` | ADT path == `core::cell::RefCell` | Runtime borrow checking |
| `is_cell` | ADT path == `core::cell::Cell` | Copy-based interior mutability |
| `is_mutex` | ADT path == `std::sync::Mutex` | Thread-safe lock |
| `is_rwlock` | ADT path == `std::sync::RwLock` | Reader-writer lock |
| `is_guard` | ADT path matches guard types | MutexGuard, Ref, RefMut, etc. |
| `is_vec` | ADT path == `alloc::vec::Vec` | Dynamic array |
| `is_string` | ADT path == `alloc::string::String` | Owned string |
| `is_option` | ADT path == `core::option::Option` | Optional value |
| `is_result` | ADT path == `core::result::Result` | Result type |
| `is_pin` | ADT path == `core::pin::Pin` | Pinned pointer |
| `is_cow` | ADT path == `alloc::borrow::Cow` | Clone-on-write |
| `is_once_cell` | ADT path == `core::cell::OnceCell` or `std::sync::OnceLock` | Lazy initialization |
| `is_maybe_uninit` | ADT path == `core::mem::MaybeUninit` | Uninitialized memory |
| `is_channel` | ADT path matches mpsc Sender/Receiver | Channel endpoints |
| `is_extern_type` | ADT path matches FFI types | c_void, CStr, CString, OsStr, etc. |
| `is_static` | Declaration syntax | static declaration |
| `is_const` | Declaration syntax | const declaration |
| `is_tuple_binding` | Pattern syntax | let (a, b) = ... |
| `is_mut_binding` | Pattern syntax | let mut x = ... |
| `is_impl_trait` | Type annotation syntax | impl Trait type |

**Initializer Pattern (v2.1+)**

The `initializer_kind` field captures the semantic pattern of the variable's initializer expression. This enables the macro to select the most appropriate tracking function based on how the variable was created, not just its type. The analyzer uses fully semantic type resolution to classify initializers into 78 distinct categories.

#### Expression-Level Classification

The top-level expression type determines the initial classification:

| Expression Type | `initializer_kind` | Example | Description |
|-----------------|-------------------|---------|-------------|
| `Literal` | `literal` | `let x = 42;` | Numeric, string, bool, char literals |
| `CallExpr` | (see Function Calls) | `let x = foo();` | Function or constructor call |
| `MethodCallExpr` | (see Method Calls) | `let x = y.bar();` | Method invocation |
| `BlockExpr` | `block` | `let x = { ... };` | Block expression |
| `IfExpr` | `if` | `let x = if c { a } else { b };` | Conditional expression |
| `MatchExpr` | `match` | `let x = match v { ... };` | Pattern matching |
| `ClosureExpr` | `closure` | `let f = \|x\| x + 1;` | Closure definition |
| `RefExpr` | `ref` / `ref_mut` | `let r = &x;` / `let r = &mut x;` | Reference creation |
| `PathExpr` | `path` | `let x = some_var;` | Variable or constant reference |
| `MacroExpr` | (see Macros) | `let v = vec![1,2,3];` | Macro invocation |
| `AwaitExpr` | `await` | `let x = fut.await;` | Async await |
| `TryExpr` | `try` | `let x = fallible()?;` | Try operator |
| `TupleExpr` | `tuple` | `let t = (1, 2, 3);` | Tuple construction |
| `ArrayExpr` | `array` | `let a = [1, 2, 3];` | Array construction |
| `IndexExpr` | `index` | `let x = arr[0];` | Index operation |
| `FieldExpr` | `field` | `let x = s.field;` | Field access |
| `CastExpr` | `cast` | `let x = y as i32;` | Type cast |
| `RecordExpr` | `struct_literal` | `let s = Struct { ... };` | Struct literal |
| `RangeExpr` | `range` | `let r = 0..10;` | Range expression |
| `BinExpr` | `binary` | `let x = a + b;` | Binary operation |
| `PrefixExpr` (deref) | `deref` | `let x = *ptr;` | Dereference |
| `PrefixExpr` (not) | `not` | `let x = !flag;` | Logical not |
| `PrefixExpr` (neg) | `neg` | `let x = -val;` | Negation |
| `LoopExpr` | `loop` | `let x = loop { break 42; };` | Loop expression |
| `WhileExpr` | `while` | `let x = while c { ... };` | While loop |
| `ForExpr` | `for` | `let x = for i in iter { ... };` | For loop |
| `ReturnExpr` | `return` | `let x = return;` | Return expression |
| `BreakExpr` | `break` | `let x = break;` | Break expression |
| `ContinueExpr` | `continue` | `let x = continue;` | Continue expression |
| `YieldExpr` | `yield` | `let x = yield val;` | Generator yield |
| `YeetExpr` | `yeet` | `let x = yeet err;` | Yeet expression |
| `AsmExpr` | `asm` | `let x = asm!(...);` | Inline assembly |
| `FormatArgsExpr` | `format_args` | `let x = format_args!(...);` | Format arguments |
| `OffsetOfExpr` | `offset_of` | `let x = offset_of!(...);` | Offset of field |

#### Function Call Classification

When the initializer is a function call (`CallExpr`), the analyzer examines the callee path to identify specific patterns:

##### Smart Pointer Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Rc creation | `rc_new` | `Rc::new`, `std::rc::Rc::new`, `alloc::rc::Rc::new` |
| Arc creation | `arc_new` | `Arc::new`, `std::sync::Arc::new`, `alloc::sync::Arc::new` |
| Box creation | `box_new` | `Box::new`, `std::boxed::Box::new`, `alloc::boxed::Box::new` |
| Box::pin | `box_pin` | `Box::pin`, `std::boxed::Box::pin` |
| Rc clone | `rc_clone` | `Rc::clone`, `std::rc::Rc::clone` |
| Arc clone | `arc_clone` | `Arc::clone`, `std::sync::Arc::clone` |
| Weak creation | `weak_new` | `Weak::new`, `std::rc::Weak::new`, `std::sync::Weak::new` |

##### Interior Mutability Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| RefCell creation | `refcell_new` | `RefCell::new`, `std::cell::RefCell::new`, `core::cell::RefCell::new` |
| Cell creation | `cell_new` | `Cell::new`, `std::cell::Cell::new`, `core::cell::Cell::new` |
| Mutex creation | `mutex_new` | `Mutex::new`, `std::sync::Mutex::new` |
| RwLock creation | `rwlock_new` | `RwLock::new`, `std::sync::RwLock::new` |

##### Lazy Initialization

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| OnceCell creation | `once_cell_new` | `OnceCell::new`, `std::cell::OnceCell::new`, `core::cell::OnceCell::new` |
| OnceLock creation | `once_lock_new` | `OnceLock::new`, `std::sync::OnceLock::new` |

##### Uninitialized Memory

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| MaybeUninit uninit | `maybe_uninit_uninit` | `MaybeUninit::uninit`, `std::mem::MaybeUninit::uninit`, `core::mem::MaybeUninit::uninit` |
| MaybeUninit new | `maybe_uninit_new` | `MaybeUninit::new`, `std::mem::MaybeUninit::new`, `core::mem::MaybeUninit::new` |
| MaybeUninit zeroed | `maybe_uninit_zeroed` | `MaybeUninit::zeroed`, `std::mem::MaybeUninit::zeroed`, `core::mem::MaybeUninit::zeroed` |

##### Channels

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Channel creation | `channel_new` | `channel`, `std::sync::mpsc::channel` |
| Sync channel | `sync_channel_new` | `sync_channel`, `std::sync::mpsc::sync_channel` |

##### Pin

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Pin creation | `pin_new` | `Pin::new`, `std::pin::Pin::new`, `core::pin::Pin::new` |
| Pin unchecked | `pin_new_unchecked` | `Pin::new_unchecked`, `std::pin::Pin::new_unchecked` |

##### Cow (Clone-on-Write)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Cow borrowed | `cow_borrowed` | `Cow::Borrowed`, `std::borrow::Cow::Borrowed` |
| Cow owned | `cow_owned` | `Cow::Owned`, `std::borrow::Cow::Owned` |

##### Option/Result Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Some variant | `option_some` | `Some`, `core::option::Option::Some`, `std::option::Option::Some` |
| None variant (call) | `option_none` | `None()`, `core::option::Option::None()` (as function call) |
| None variant (path) | `none` | `None`, `Option::None` (as path expression) |
| Ok variant | `result_ok` | `Ok`, `core::result::Result::Ok`, `std::result::Result::Ok` |
| Err variant | `result_err` | `Err`, `core::result::Result::Err`, `std::result::Result::Err` |

##### String Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| String::new | `string_new` | `String::new`, `std::string::String::new`, `alloc::string::String::new` |
| String::from | `string_from` | `String::from`, `std::string::String::from`, `alloc::string::String::from` |
| String::with_capacity | `string_with_capacity` | `String::with_capacity` |

##### Vec Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Vec::new | `vec_new` | `Vec::new`, `std::vec::Vec::new`, `alloc::vec::Vec::new` |
| Vec::with_capacity | `vec_with_capacity` | `Vec::with_capacity`, `std::vec::Vec::with_capacity` |

##### Collection Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| HashMap::new | `hashmap_new` | `HashMap::new`, `std::collections::HashMap::new` |
| HashSet::new | `hashset_new` | `HashSet::new`, `std::collections::HashSet::new` |
| BTreeMap::new | `btreemap_new` | `BTreeMap::new`, `std::collections::BTreeMap::new` |
| BTreeSet::new | `btreeset_new` | `BTreeSet::new`, `std::collections::BTreeSet::new` |
| VecDeque::new | `vecdeque_new` | `VecDeque::new`, `std::collections::VecDeque::new` |
| LinkedList::new | `linkedlist_new` | `LinkedList::new`, `std::collections::LinkedList::new` |
| BinaryHeap::new | `binaryheap_new` | `BinaryHeap::new`, `std::collections::BinaryHeap::new` |

##### Path and FFI Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| PathBuf::new | `pathbuf_new` | `PathBuf::new`, `std::path::PathBuf::new` |
| PathBuf::from | `pathbuf_from` | `PathBuf::from`, `std::path::PathBuf::from` |
| OsString::new | `osstring_new` | `OsString::new`, `std::ffi::OsString::new` |
| OsString::from | `osstring_from` | `OsString::from`, `std::ffi::OsString::from` |
| CString::new | `cstring_new` | `CString::new`, `std::ffi::CString::new` |

##### Raw Pointer Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Null pointer | `ptr_null` | `ptr::null`, `std::ptr::null`, `core::ptr::null` |
| Null mut pointer | `ptr_null_mut` | `ptr::null_mut`, `std::ptr::null_mut`, `core::ptr::null_mut` |
| NonNull::new | `nonnull_new` | `NonNull::new`, `std::ptr::NonNull::new`, `core::ptr::NonNull::new` |
| NonNull::dangling | `nonnull_dangling` | `NonNull::dangling`, `std::ptr::NonNull::dangling` |

##### Box Raw Pointer Operations

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Box::into_raw | `box_into_raw` | `Box::into_raw`, `std::boxed::Box::into_raw` |
| Box::from_raw | `box_from_raw` | `Box::from_raw`, `std::boxed::Box::from_raw` |

##### ManuallyDrop

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| ManuallyDrop::new | `manually_drop_new` | `ManuallyDrop::new`, `std::mem::ManuallyDrop::new`, `core::mem::ManuallyDrop::new` |
| ManuallyDrop::into_inner | `manually_drop_into_inner` | `ManuallyDrop::into_inner`, `std::mem::ManuallyDrop::into_inner` |

##### Atomics

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| AtomicBool::new | `atomic_bool_new` | `AtomicBool::new`, `std::sync::atomic::AtomicBool::new`, `core::sync::atomic::AtomicBool::new` |
| AtomicI8::new | `atomic_i8_new` | `AtomicI8::new`, `std::sync::atomic::AtomicI8::new` |
| AtomicI16::new | `atomic_i16_new` | `AtomicI16::new`, `std::sync::atomic::AtomicI16::new` |
| AtomicI32::new | `atomic_i32_new` | `AtomicI32::new`, `std::sync::atomic::AtomicI32::new` |
| AtomicI64::new | `atomic_i64_new` | `AtomicI64::new`, `std::sync::atomic::AtomicI64::new` |
| AtomicIsize::new | `atomic_isize_new` | `AtomicIsize::new`, `std::sync::atomic::AtomicIsize::new` |
| AtomicU8::new | `atomic_u8_new` | `AtomicU8::new`, `std::sync::atomic::AtomicU8::new` |
| AtomicU16::new | `atomic_u16_new` | `AtomicU16::new`, `std::sync::atomic::AtomicU16::new` |
| AtomicU32::new | `atomic_u32_new` | `AtomicU32::new`, `std::sync::atomic::AtomicU32::new` |
| AtomicU64::new | `atomic_u64_new` | `AtomicU64::new`, `std::sync::atomic::AtomicU64::new` |
| AtomicUsize::new | `atomic_usize_new` | `AtomicUsize::new`, `std::sync::atomic::AtomicUsize::new` |
| AtomicPtr::new | `atomic_ptr_new` | `AtomicPtr::new`, `std::sync::atomic::AtomicPtr::new` |

##### Time

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Duration::new | `duration_new` | `Duration::new`, `std::time::Duration::new`, `core::time::Duration::new` |
| Duration::from_secs | `duration_from_secs` | `Duration::from_secs`, `std::time::Duration::from_secs` |
| Duration::from_millis | `duration_from_millis` | `Duration::from_millis`, `std::time::Duration::from_millis` |
| Duration::from_micros | `duration_from_micros` | `Duration::from_micros`, `std::time::Duration::from_micros` |
| Duration::from_nanos | `duration_from_nanos` | `Duration::from_nanos`, `std::time::Duration::from_nanos` |
| Duration::from_secs_f32/f64 | `duration_from_secs_f` | `Duration::from_secs_f32`, `Duration::from_secs_f64` |
| Instant::now | `instant_now` | `Instant::now`, `std::time::Instant::now` |
| SystemTime::now | `system_time_now` | `SystemTime::now`, `std::time::SystemTime::now` |

##### IO

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Cursor::new | `cursor_new` | `Cursor::new`, `std::io::Cursor::new` |
| BufReader::new | `bufreader_new` | `BufReader::new`, `std::io::BufReader::new` |
| BufReader::with_capacity | `bufreader_with_capacity` | `BufReader::with_capacity`, `std::io::BufReader::with_capacity` |
| BufWriter::new | `bufwriter_new` | `BufWriter::new`, `std::io::BufWriter::new` |
| BufWriter::with_capacity | `bufwriter_with_capacity` | `BufWriter::with_capacity`, `std::io::BufWriter::with_capacity` |
| File::open | `file_open` | `File::open`, `std::fs::File::open` |
| File::create | `file_create` | `File::create`, `std::fs::File::create` |

##### Ordering (Comparison Result)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Ordering::Less | `ordering_less` | `Ordering::Less`, `std::cmp::Ordering::Less` |
| Ordering::Equal | `ordering_equal` | `Ordering::Equal`, `std::cmp::Ordering::Equal` |
| Ordering::Greater | `ordering_greater` | `Ordering::Greater`, `std::cmp::Ordering::Greater` |

##### Poll (Async Support)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Poll::Ready | `poll_ready` | `Poll::Ready`, `std::task::Poll::Ready` |
| Poll::Pending | `poll_pending` | `Poll::Pending`, `std::task::Poll::Pending` |

##### Panic Support

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Location::caller | `location_caller` | `Location::caller`, `std::panic::Location::caller` |

##### UnsafeCell

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| UnsafeCell::new | `unsafe_cell_new` | `UnsafeCell::new`, `std::cell::UnsafeCell::new`, `core::cell::UnsafeCell::new` |

##### Trait Methods

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Default::default | `default` | `Default::default`, `std::default::Default::default`, `core::default::Default::default` |
| Clone (generic) | `clone` | Any path ending in `::clone` |

##### Semantic Type Classification (v2.3+)

The analyzer applies **semantic type classification** to ALL expressions using rust-analyzer's type resolution. This approach examines the resolved type to classify the initializer, with expression structure (call, method, macro, etc.) used only as context:

###### ADT Classification by Canonical Path

For types that resolve to an ADT (struct, enum, or union), the analyzer extracts the canonical module path and classifies accordingly:

| Resolved Type Path | `initializer_kind` | Example |
|-------------------|-------------------|---------|
| `alloc::rc::Rc` | `rc_new` | `let x = create_rc();` where return type is `Rc<T>` |
| `alloc::sync::Arc` | `arc_new` | Factory function returning `Arc<T>` |
| `core::option::Option` | `option_variant` | Any function returning `Option<T>` |
| `core::result::Result` | `result_variant` | Any function returning `Result<T, E>` |
| (70+ standard library types) | (type-specific) | See full list in source |

###### User-Defined Types

When the resolved type is a user-defined ADT (not from std/core/alloc), classification is by ADT kind:

| ADT Kind | `initializer_kind` | Example |
|----------|-------------------|---------|
| Struct | `user_struct` | `let p = Point::new(1, 2);` |
| Enum | `user_enum` | `let s = Status::Active;` |
| Union | `user_union` | `let u = MyUnion::new();` |

###### Tuple Types

| Type | `initializer_kind` | Example |
|------|-------------------|---------|
| Tuple | `tuple` | `let (tx, rx) = channel();` |

###### impl Trait Types

For opaque `impl Trait` return types, the analyzer extracts the primary trait bound:

| Trait Bound | `initializer_kind` | Example |
|-------------|-------------------|---------|
| `Future` | `impl_future` | `let f = async_fn();` returning `impl Future<Output = T>` |
| `Iterator` | `impl_iterator` | `let i = get_iter();` returning `impl Iterator<Item = T>` |
| `Fn`/`FnMut`/`FnOnce` | `impl_fn` | `let f = get_closure();` returning `impl Fn(T) -> U` |
| Other traits | `impl_{trait_name}` | Lowercase trait name |

###### Other Semantic Classifications

| Type Category | `initializer_kind` | Detection Method |
|---------------|-------------------|------------------|
| Primitives | `primitive` | `ty.as_builtin()` - i32, bool, char, etc. |
| str | `str` | `ty.as_builtin().is_str()` |
| Closures | `closure` | `ty.is_closure()` |
| Function pointers | `fn_ptr` | `ty.is_fn()` |
| References | `ref` / `ref_mut` | `ty.is_reference()` |
| Raw pointers | `raw_ptr` | `ty.is_raw_ptr()` |

##### Fallback

With semantic classification, the `call` fallback is now rare and only occurs when:
- The type cannot be resolved (e.g., in files outside the crate graph)
- The type is truly unknown to rust-analyzer

| Pattern | `initializer_kind` | Description |
|---------|-------------------|-------------|
| Unresolved call | `call` | Function call with unresolvable return type |

#### Method Call Classification

When the initializer is a method call (`MethodCallExpr`), the analyzer examines the method name:

##### RefCell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `borrow` | `refcell_borrow` | `let r = cell.borrow();` |
| `borrow_mut` | `refcell_borrow_mut` | `let r = cell.borrow_mut();` |
| `try_borrow` | `refcell_try_borrow` | `let r = cell.try_borrow();` |
| `try_borrow_mut` | `refcell_try_borrow_mut` | `let r = cell.try_borrow_mut();` |

##### Cell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `get` | `cell_get` | `let v = cell.get();` |
| `set` | `cell_set` | `let _ = cell.set(v);` |
| `replace` | `cell_replace` | `let old = cell.replace(new);` |
| `take` | `cell_take` | `let v = cell.take();` |

##### Mutex/RwLock Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `lock` | `mutex_lock` | `let guard = mutex.lock().unwrap();` |
| `try_lock` | `mutex_try_lock` | `let guard = mutex.try_lock();` |
| `read` | `rwlock_read` | `let guard = rwlock.read().unwrap();` |
| `write` | `rwlock_write` | `let guard = rwlock.write().unwrap();` |
| `try_read` | `rwlock_try_read` | `let guard = rwlock.try_read();` |
| `try_write` | `rwlock_try_write` | `let guard = rwlock.try_write();` |

##### OnceCell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `get_or_init` | `once_cell_get_or_init` | `let v = cell.get_or_init(\|\| 42);` |
| `get_or_try_init` | `once_cell_get_or_try_init` | `let v = cell.get_or_try_init(\|\| Ok(42));` |

##### MaybeUninit Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `assume_init` | `maybe_uninit_assume_init` | `let v = uninit.assume_init();` |
| `assume_init_read` | `maybe_uninit_assume_init_read` | `let v = uninit.assume_init_read();` |
| `assume_init_ref` | `maybe_uninit_assume_init_ref` | `let r = uninit.assume_init_ref();` |
| `assume_init_mut` | `maybe_uninit_assume_init_mut` | `let r = uninit.assume_init_mut();` |

##### Weak Pointer Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `downgrade` | `weak_downgrade` | `let weak = Rc::downgrade(&rc);` |
| `upgrade` | `weak_upgrade` | `let strong = weak.upgrade();` |

##### Cow Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `to_mut` | `cow_to_mut` | `let m = cow.to_mut();` |
| `into_owned` | `cow_into_owned` | `let owned = cow.into_owned();` |

##### Pin Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `as_ref` | `pin_as_ref` | `let r = pin.as_ref();` |
| `as_mut` | `pin_as_mut` | `let r = pin.as_mut();` |
| `into_inner` | `into_inner` | `let v = pin.into_inner();` |

##### Atomic Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `load` | `atomic_load` | `let v = atomic.load(Ordering::SeqCst);` |
| `store` | `atomic_store` | `atomic.store(v, Ordering::SeqCst);` |
| `swap` | `atomic_swap` | `let old = atomic.swap(new, Ordering::SeqCst);` |
| `compare_exchange` | `atomic_compare_exchange` | `let r = atomic.compare_exchange(...);` |
| `compare_exchange_weak` | `atomic_compare_exchange_weak` | `let r = atomic.compare_exchange_weak(...);` |
| `fetch_add` | `atomic_fetch_add` | `let old = atomic.fetch_add(1, Ordering::SeqCst);` |
| `fetch_sub` | `atomic_fetch_sub` | `let old = atomic.fetch_sub(1, Ordering::SeqCst);` |
| `fetch_and` | `atomic_fetch_and` | `let old = atomic.fetch_and(mask, Ordering::SeqCst);` |
| `fetch_or` | `atomic_fetch_or` | `let old = atomic.fetch_or(mask, Ordering::SeqCst);` |
| `fetch_xor` | `atomic_fetch_xor` | `let old = atomic.fetch_xor(mask, Ordering::SeqCst);` |
| `fetch_max` | `atomic_fetch_max` | `let old = atomic.fetch_max(val, Ordering::SeqCst);` |
| `fetch_min` | `atomic_fetch_min` | `let old = atomic.fetch_min(val, Ordering::SeqCst);` |
| `fetch_update` | `atomic_fetch_update` | `let r = atomic.fetch_update(...);` |

##### Duration/Instant Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `as_secs` | `duration_as_secs` | `let s = duration.as_secs();` |
| `as_millis` | `duration_as_millis` | `let ms = duration.as_millis();` |
| `as_micros` | `duration_as_micros` | `let us = duration.as_micros();` |
| `as_nanos` | `duration_as_nanos` | `let ns = duration.as_nanos();` |
| `as_secs_f32`/`as_secs_f64` | `duration_as_secs_f` | `let s = duration.as_secs_f64();` |
| `elapsed` | `instant_elapsed` | `let d = instant.elapsed();` |
| `duration_since` | `instant_duration_since` | `let d = instant.duration_since(earlier);` |

##### Iterator Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `iter` | `iter` | `let it = vec.iter();` |
| `iter_mut` | `iter_mut` | `let it = vec.iter_mut();` |
| `into_iter` | `into_iter` | `let it = vec.into_iter();` |

##### Common Combinator Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `unwrap` | `unwrap` | `let v = opt.unwrap();` |
| `expect` | `expect` | `let v = opt.expect("msg");` |
| `map` | `map` | `let v = opt.map(\|x\| x + 1);` |
| `and_then` | `and_then` | `let v = opt.and_then(\|x\| Some(x));` |
| `ok` | `ok` | `let opt = result.ok();` |
| `err` | `err` | `let opt = result.err();` |
| `clone` | `clone` | `let c = val.clone();` |

##### Fallback

For method calls not matching known patterns, the analyzer applies semantic type classification (see above) based on the resolved return type. This means methods like `.iter().map().filter()` chains are classified by their final return type rather than falling back to a generic `method` classification.

| Method | `initializer_kind` | Description |
|--------|-------------------|-------------|
| Unknown method | (semantic) | Classified by resolved return type |
| Unresolved method | `method` | Method with unresolvable return type |

#### Macro Classification

When the initializer is a macro invocation (`MacroExpr`), the analyzer examines the macro name:

| Macro | `initializer_kind` | Example |
|-------|-------------------|---------|
| `vec!` | `vec_macro` | `let v = vec![1, 2, 3];` |
| `format!` | `format_macro` | `let s = format!("{}", x);` |
| `println!`/`print!`/`eprintln!`/`eprint!` | `print_macro` | `let _ = println!("hi");` |
| `panic!` | `panic_macro` | `let _ = panic!("error");` |
| `assert!`/`assert_eq!`/`assert_ne!` | `assert_macro` | `let _ = assert!(true);` |
| `pin!` | `pin_macro` | `let p = pin!(future);` |
| Unknown macro | `macro` | Any macro not matching above |

#### Design Rationale

The `initializer_kind` classification serves several purposes:

1. **Precise Tracking Selection**: The macro can select the exact tracking function based on how a value was created, not just its type. For example, `Rc::clone` should use `track_rc_clone` (which records the source reference count) rather than `track_rc_new`.

2. **Type Alias Handling**: When users define type aliases like `type MyRc<T> = Rc<T>`, the call `MyRc::new(x)` won't match the `Rc::new` pattern. However, the type flags (`is_rc: true`) still enable correct tracking via the fallback path.

3. **Guard Tracking**: Methods like `borrow()`, `lock()`, and `read()` create guard types that require special tracking to monitor their lifetime and detect potential deadlocks or borrow violations.

4. **Unsafe Operation Tracking**: Patterns like `MaybeUninit::assume_init()` and `Box::from_raw()` indicate unsafe operations that warrant special attention in ownership visualization.

5. **Performance Optimization**: By classifying at analysis time, the macro avoids runtime pattern matching and can generate optimal tracking code directly.

**Disambiguation Fields (v2.2+)**

These fields enable precise variable lookup when multiple variables share the same name:

| Field | Purpose |
|-------|---------|
| `function_name` | Name of the containing function (null for module-level) |
| `decl_index` | 0-based declaration order within the function |
| `scope_id` | Unique scope identifier |
| `span_start` | Byte offset of pattern start |
| `span_end` | Byte offset of pattern end |

The macro uses these fields to disambiguate variables with the same name in different functions or shadowed within the same function:

```rust
fn foo() {
    let x = Rc::new(1);  // function_name: "foo", decl_index: 0
}

fn bar() {
    let x = Arc::new(1); // function_name: "bar", decl_index: 0
    let x = x.clone();   // function_name: "bar", decl_index: 1 (shadowing)
}
```

These flags are not mutually exclusive. A type like `Rc<RefCell<Vec<String>>>` will have `is_rc`, `is_refcell`, `is_vec`, and `is_string` all set to `true`, reflecting the nested structure.

**Binding Pattern Flags (for Macro Transformation)**

The following flags help the macro make better transformation decisions based on the [battle test whitepaper](https://mehmet-ylcnky.github.io/BorrowScope/battle-test-whitepaper/) error taxonomy:

| Flag | Battle Test Error | Macro Action |
|------|-------------------|--------------|
| `is_tuple_binding` | ERR-002: Tuple destructuring | Skip tracking or handle specially |
| `is_mut_binding` | Pattern syntax | let mut x = ... |
| `is_impl_trait` | Type annotation syntax | impl Trait type |

These flags are not mutually exclusive. A type like `Rc<RefCell<Vec<String>>>` will have `is_rc`, `is_refcell`, `is_vec`, and `is_string` all set to `true`, reflecting the nested structure. Additionally, it will have `is_clone: true`, `is_drop: true`, `is_sized: true` from trait detection.

### Example Output

For a source file containing:

```rust
fn example() {
    let count = 42;
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    let guard = shared.borrow();
    let future = async { 42 };
}
```

The analyzer produces:

```json
{
  "version": "2.3",
  "analyzer_version": "0.1.0",
  "files": {
    "src/main.rs": [
      {
        "name": "count",
        "ty": "i32",
        "is_copy": true,
        "is_clone": true,
        "is_send": true,
        "is_sync": true,
        "is_drop": false,
        "is_sized": true,
        "is_future": false,
        "is_iterator": false,
        "is_primitive": true,
        "is_reference": false,
        "is_mutable_reference": false,
        "is_raw_ptr": false,
        "is_slice": false,
        "is_str": false,
        "is_closure": false,
        "is_fn_ptr": false,
        "is_dyn_trait": false,
        "is_union": false,
        "is_rc": false,
        "is_arc": false,
        "is_box": false,
        "is_vec": false,
        "is_string": false,
        "file": "src/main.rs",
        "line": 2,
        "column": 8
      },
      {
        "name": "shared",
        "ty": "Rc<RefCell<Vec<i32, Global>>, Global>",
        "is_copy": false,
        "is_clone": true,
        "is_send": false,
        "is_sync": false,
        "is_drop": true,
        "is_sized": true,
        "is_future": false,
        "is_iterator": false,
        "is_primitive": false,
        "is_reference": false,
        "is_mutable_reference": false,
        "is_raw_ptr": false,
        "is_slice": false,
        "is_str": false,
        "is_closure": false,
        "is_fn_ptr": false,
        "is_dyn_trait": false,
        "is_union": false,
        "is_rc": true,
        "is_arc": false,
        "is_box": false,
        "is_refcell": true,
        "is_vec": true,
        "is_string": false,
        "file": "src/main.rs",
        "line": 3,
        "column": 8
      },
      {
        "name": "guard",
        "ty": "Ref<'_, Vec<i32, Global>>",
        "is_copy": false,
        "is_clone": false,
        "is_send": false,
        "is_sync": true,
        "is_drop": true,
        "is_sized": true,
        "is_guard": true,
        "is_vec": true,
        "file": "src/main.rs",
        "line": 4,
        "column": 8
      },
      {
        "name": "future",
        "ty": "impl Future<Output = i32>",
        "is_copy": false,
        "is_clone": false,
        "is_send": true,
        "is_sync": false,
        "is_drop": false,
        "is_sized": true,
        "is_future": true,
        "is_closure": true,
        "file": "src/main.rs",
        "line": 5,
        "column": 8
      }
    ]
  }
}
```

Key observations:
- `count` has `is_primitive: true`, `is_copy: true`, `is_send: true`, `is_sync: true` - all detected semantically
- `shared` has `is_send: false`, `is_sync: false` because `Rc` is not thread-safe
- `guard` has `is_guard: true` and `is_sync: true` (guards are Sync but not Send)
- `future` has `is_future: true` detected via `impls_trait(Future)`
- All trait flags are determined by actual trait implementations, not string matching

---

## 5. Integration with borrowscope-macro

The type information produced by the analyzer enables `borrowscope-macro` to make informed decisions during code transformation. This section describes the integration architecture and the enhanced tracking capabilities it enables.

### Type Information Lookup

When the `#[trace_borrow]` macro processes a function, it needs to determine the appropriate tracking function for each variable binding. With the analyzer's output available, the macro can perform precise lookups using function context and declaration order:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MACRO TYPE LOOKUP FLOW (v2.2)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                    Macro Processing                            │
│  ───────────                    ────────────────                            │
│                                                                             │
│  #[trace_borrow]                                                            │
│  fn example() {                 ┌─────────────────────────────────┐         │
│      let data = Rc::new(x); ──▶ │ 1. Set function context: "example"│        │
│      ...                        │ 2. Lookup in by_function index   │         │
│  }                              │    key: ("example", "data", 0)   │         │
│                                 │ 3. Find: initializer_kind="rc_new"│        │
│                                 │ 4. Emit: track_rc_new_with_id(   │         │
│                                 │          id, "data", type, loc,  │         │
│                                 │          Rc::new(x))             │         │
│                                 └─────────────────────────────────┘         │
│                                                                             │
│  type-info.json                                                             │
│  ──────────────                                                             │
│  {                                                                          │
│    "by_function": {             ◄─── Primary lookup index (v2.2)            │
│      "example": {                                                           │
│        "data": [{                                                           │
│          "name": "data",                                                    │
│          "function_name": "example",                                        │
│          "decl_index": 0,       ◄─── Disambiguates shadowed vars            │
│          "initializer_kind": "rc_new",  ◄─── Determines tracking fn         │
│          "is_rc": true                                                      │
│        }]                                                                   │
│      }                                                                      │
│    },                                                                       │
│    "by_name": {                 ◄─── Fallback index                         │
│      "data": [...]                                                          │
│    }                                                                        │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The macro uses a two-tier lookup strategy:

1. **Primary**: `lookup_in_function(fn_name, var_name, decl_index)` - Uses the `by_function` index for precise matching
2. **Fallback**: `lookup_by_name(var_name)` - Uses the `by_name` index when function context is unavailable

This approach handles variable shadowing correctly:

```rust
#[trace_borrow]
fn shadowing_example() {
    let x = 1;           // decl_index: 0, type: i32
    let x = "hello";     // decl_index: 1, type: &str  
    let x = vec![1, 2];  // decl_index: 2, type: Vec<i32>
    let x = Rc::new(x);  // decl_index: 3, type: Rc<Vec<i32>>
}
```

Each `x` is correctly identified by its `decl_index`, allowing the macro to select the appropriate tracking function for each.

### Enhanced Tracking Function Selection

With complete type information including `initializer_kind`, the macro can select the most appropriate tracking function for each variable. The decision is now based on semantic analysis rather than heuristics:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 TRACKING FUNCTION SELECTION LOGIC (v2.2)                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Given: VariableTypeInfo for binding                                        │
│                                                                             │
│  // === Primary: Use initializer_kind for precise tracking ===              │
│                                                                             │
│  match initializer_kind:                                                    │
│      "rc_new"           → track_rc_new_with_id(...)                         │
│      "rc_clone"         → track_rc_clone_with_id(...)                       │
│      "arc_new"          → track_arc_new_with_id(...)                        │
│      "arc_clone"        → track_arc_clone_with_id(...)                      │
│      "box_new"          → track_box_new(...)                                │
│      "refcell_new"      → track_refcell_new(...)                            │
│      "refcell_borrow"   → track_refcell_borrow(...)                         │
│      "refcell_borrow_mut" → track_refcell_borrow_mut(...)                   │
│      "cell_new"         → track_cell_new(...)                               │
│      "mutex_lock"       → track_lock_guard_acquire(...)                     │
│      "channel_new"      → track_channel(...)                                │
│      "weak_new"         → track_weak_new(...)                               │
│      "pin_new"          → track_pin_new(...)                                │
│      "cow_borrowed"     → track_cow_borrowed(...)                           │
│      "cow_owned"        → track_cow_owned(...)                              │
│      "ref"              → track_borrow_with_id(...)                         │
│      "ref_mut"          → track_borrow_mut_with_id(...)                     │
│                                                                             │
│  // === Fallback: Use type flags for generic initializers ===               │
│                                                                             │
│  if is_rc:              → track_rc_new_with_id(...)                         │
│  else if is_arc:        → track_arc_new_with_id(...)                        │
│  else if is_box:        → track_box_new(...)                                │
│  else if is_refcell:    → track_refcell_new(...)                            │
│  else if is_cell:       → track_cell_new(...)                               │
│  else if is_raw_ptr:    → track_raw_ptr_create(...)                         │
│                                                                             │
│  // === Default: Generic tracking ===                                       │
│                                                                             │
│  else:                  → track_new_with_id(...)                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The `initializer_kind` field enables precise tracking even for type aliases and factory functions:

```rust
type MyRc<T> = Rc<T>;

fn example() {
    // Without initializer_kind: macro sees "MyRc::new" - doesn't match "Rc::new"
    // With initializer_kind: analyzer resolves type to Rc, sets is_rc=true
    let x = MyRc::new(42);  // Correctly tracked as Rc
    
    // Factory function returning Rc
    let y = create_shared(42);  // initializer_kind="call", but is_rc=true
                                // Falls back to type-based tracking
}
```

### Fallback Behavior

The macro must handle cases where type information is unavailable. This occurs when:

1. The analyzer has not been run on the project
2. The source file was modified after analysis
3. The binding location doesn't match any entry (line number drift)

The fallback strategy preserves backward compatibility:

```rust
fn get_tracking_call(binding: &LetBinding, type_info: Option<&VariableTypeInfo>) -> TokenStream {
    match type_info {
        Some(info) => {
            // Use precise type information
            select_tracking_function(info)
        }
        None => {
            // Fall back to syntactic heuristics (current behavior)
            infer_from_syntax(binding)
        }
    }
}
```

When falling back, the macro logs a warning indicating that type information was not found, encouraging users to run the analyzer for complete tracking accuracy.

### Benefits Over Syntactic Analysis

The integration provides several concrete improvements:

**Accurate Smart Pointer Detection**: Any expression that evaluates to `Rc<T>` is correctly identified, regardless of how it was constructed. Factory functions, conditional expressions, and match arms all resolve to their actual types.

**Copy Semantics**: The runtime can now distinguish between ownership transfers and copies. This is essential for accurate visualization—a copy creates a new independent value, while a move transfers the original.

**Nested Type Awareness**: A type like `Arc<Mutex<Vec<String>>>` has multiple classification flags set, allowing the runtime to track all relevant aspects: atomic reference counting, mutex locking, vector operations, and string allocations.

**User-Defined Types**: While the analyzer cannot automatically classify user-defined smart pointers, the full type string is available. Future versions could support user-provided classification rules or trait-based detection.

**Closure Types**: Closures have opaque types like `impl Fn(i32) -> i32`. The analyzer captures these types, enabling tracking of closure creation and potential capture analysis.

### Runtime Event Enhancement

With type information available, the runtime events become more informative:

```json
// Without type info
{"event": "new", "name": "data", "type": "unknown"}

// With type info
{"event": "rc_new", "name": "data", "type": "Rc<RefCell<Vec<i32, Global>>, Global>", 
 "is_copy": false, "ref_count": 1}
```

The enhanced events enable richer visualization and analysis. A visualization tool can render reference-counted pointers differently from owned values, show interior mutability boundaries, and accurately depict copy vs move semantics.

### The Complete BorrowScope Pipeline

BorrowScope consists of three interconnected projects that form a complete ownership visualization pipeline. Each project addresses a specific limitation in Rust's compilation model, and together they transform invisible ownership semantics into observable runtime events.

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           BORROWSCOPE ARCHITECTURE                                  │
│                                                                                     │
│    ┌───────────────────┐      ┌───────────────────┐      ┌───────────────────┐     │
│    │                   │      │                   │      │                   │     │
│    │  borrowscope-     │      │  borrowscope-     │      │  borrowscope-     │     │
│    │    analyzer       │─────▶│     macro         │─────▶│    runtime        │     │
│    │                   │      │                   │      │                   │     │
│    │  Static Analysis  │      │  Code Transform   │      │  Event Capture    │     │
│    │                   │      │                   │      │                   │     │
│    └───────────────────┘      └───────────────────┘      └───────────────────┘     │
│            │                          │                          │                 │
│            ▼                          ▼                          ▼                 │
│    ┌───────────────────┐      ┌───────────────────┐      ┌───────────────────┐     │
│    │                   │      │                   │      │                   │     │
│    │  type-info.json   │      │  Instrumented     │      │  Event Stream     │     │
│    │                   │      │  Source Code      │      │  + JSON Export    │     │
│    │                   │      │                   │      │                   │     │
│    └───────────────────┘      └───────────────────┘      └───────────────────┘     │
│                                                                                     │
│    PHASE 1: Pre-Build         PHASE 2: Compile-Time      PHASE 3: Runtime          │
│    ─────────────────          ──────────────────         ───────────────           │
│    Extracts semantic          Transforms AST to          Records timestamped       │
│    type information           inject tracking calls      ownership events          │
│    using rust-analyzer        using type-info.json       to global tracker         │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

The pipeline solves a fundamental problem: Rust's ownership system operates entirely at compile time, leaving no trace at runtime. The borrow checker enforces rules, rejects invalid programs, and then disappears—the compiled binary contains no record of ownership transfers, borrow scopes, or drop ordering. BorrowScope reconstructs this invisible layer by combining static analysis with runtime instrumentation.

#### Phase 1: Static Analysis with borrowscope-analyzer

The analyzer runs as a pre-build step, leveraging rust-analyzer's semantic analysis infrastructure to extract type information that procedural macros cannot access. It processes the entire project and produces a JSON file containing detailed type classifications for every variable binding.

```rust
// User's source code
fn process_data() {
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    let guard = shared.borrow();
    let cloned = Rc::clone(&shared);
}
```

The analyzer resolves each binding to its concrete type and classifies the initializer expression:

```json
{
  "by_function": {
    "process_data": {
      "shared": [{
        "name": "shared",
        "ty": "Rc<RefCell<Vec<i32>>>",
        "initializer_kind": "rc_new",
        "is_rc": true,
        "is_refcell": false,
        "decl_index": 0
      }],
      "guard": [{
        "name": "guard",
        "ty": "Ref<Vec<i32>>",
        "initializer_kind": "refcell_borrow",
        "is_guard": true,
        "decl_index": 0
      }],
      "cloned": [{
        "name": "cloned",
        "ty": "Rc<RefCell<Vec<i32>>>",
        "initializer_kind": "rc_clone",
        "is_rc": true,
        "decl_index": 0
      }]
    }
  }
}
```

The `initializer_kind` field captures the semantic pattern of how each variable was created—not just its type, but the specific operation that produced it. This distinction is critical: `Rc::new()` and `Rc::clone()` both produce `Rc<T>`, but they represent fundamentally different ownership operations (allocation vs. shared reference).

#### Phase 2: Code Transformation with borrowscope-macro

The `#[trace_borrow]` procedural macro executes during compilation, transforming annotated functions to inject tracking calls. When type-info.json exists, the macro loads it at compile time and uses the semantic classifications to select precise tracking functions.

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                        MACRO TRANSFORMATION PROCESS                                 │
│                                                                                     │
│  INPUT: Original Source                OUTPUT: Instrumented Source                  │
│  ─────────────────────                 ───────────────────────────                  │
│                                                                                     │
│  #[trace_borrow]                       fn process_data() {                          │
│  fn process_data() {                       let shared = borrowscope_runtime::       │
│      let shared = Rc::new(                     track_rc_new_with_id(                │
│          RefCell::new(                             1,           // var_id           │
│              vec![1, 2, 3]                         "shared",    // name             │
│          )                                         "Rc<RefCell<Vec<i32>>>",         │
│      );                        ───▶                "src/lib.rs:5",                  │
│      let guard = shared                            Rc::new(RefCell::new(            │
│          .borrow();                                    vec![1, 2, 3]                │
│      let cloned = Rc::clone(                       ))                               │
│          &shared                                   );                               │
│      );                                        let guard = borrowscope_runtime::    │
│  }                                                 track_refcell_borrow(            │
│                                                        2, 1,    // guard_id, cell_id│
│                                                        "src/lib.rs:6",              │
│                                                        shared.borrow()              │
│                                                    );                               │
│                                                // ... continues with cloned         │
│                                            }                                        │
│                                                                                     │
│  LOOKUP PROCESS:                                                                    │
│  ───────────────                                                                    │
│  1. Macro sets function context: "process_data"                                     │
│  2. For each let binding, lookup in type-info.json:                                 │
│     - Key: (function_name, var_name, decl_index)                                    │
│     - Returns: VariableTypeInfo with initializer_kind                               │
│  3. Match initializer_kind to tracking function:                                    │
│     - "rc_new"         → track_rc_new_with_id()                                     │
│     - "refcell_borrow" → track_refcell_borrow()                                     │
│     - "rc_clone"       → track_rc_clone_with_id()                                   │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

The macro maintains internal state during transformation: a map of variable names to unique IDs, a scope stack for tracking drop order, and type context sets for special handling of guards, weak references, and other stateful types. This state enables correlation between related events—a borrow event references its owner's ID, a clone event references its source's ID.

When type-info.json is unavailable, the macro falls back to syntactic pattern matching. It recognizes literal patterns like `Rc::new(...)` and `Arc::clone(&x)` but cannot handle type aliases, factory functions, or conditional expressions. The fallback ensures backward compatibility while the analyzer provides enhanced precision.

#### Phase 3: Runtime Event Capture with borrowscope-runtime

The runtime library provides the tracking functions that the macro injects. Each function follows a consistent pattern: acquire a lock on the global tracker, generate a monotonic timestamp, record an event with all relevant metadata, and return the original value unchanged.

```rust
// Simplified implementation of track_rc_new_with_id
pub fn track_rc_new_with_id<T>(
    var_id: usize,
    name: &str,
    type_name: &str,
    location: &str,
    value: Rc<T>,
) -> Rc<T> {
    TRACKER.lock().record_event(Event::RcNew {
        timestamp: TIMESTAMP.fetch_add(1, Ordering::SeqCst),
        var_id: format!("var_{}", var_id),
        var_name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        strong_count: Rc::strong_count(&value),
        weak_count: Rc::weak_count(&value),
    });
    value  // Pass-through: tracking is transparent
}
```

The pass-through design is essential—tracking functions must not alter program behavior. They observe and record, then return the value exactly as received. This transparency allows instrumentation of production code paths without changing semantics.

The runtime accumulates events in a thread-safe global tracker using `parking_lot::Mutex` for efficient locking and `AtomicU64` for lock-free timestamp generation. Events can be queried, exported to JSON, or transformed into an ownership graph for visualization:

```rust
// After program execution
let events = borrowscope_runtime::get_events();
let graph = borrowscope_runtime::get_graph();

// Export for external visualization tools
borrowscope_runtime::export_json("ownership_trace.json")?;
```

#### Data Flow Through the Pipeline

The following diagram illustrates how information flows through all three projects during a complete analysis cycle:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              DATA FLOW DIAGRAM                                      │
│                                                                                     │
│  ┌─────────────┐                                                                    │
│  │   User's    │                                                                    │
│  │   Source    │                                                                    │
│  │    Code     │                                                                    │
│  └──────┬──────┘                                                                    │
│         │                                                                           │
│         ▼                                                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                         ANALYZER (Pre-Build)                                │   │
│  │                                                                             │   │
│  │   rust-analyzer APIs:                                                       │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │   │
│  │   │  ra_ap_ide  │  │ ra_ap_hir   │  │ra_ap_hir_ty │  │ra_ap_syntax │       │   │
│  │   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │   │
│  │          │                │                │                │              │   │
│  │          └────────────────┴────────────────┴────────────────┘              │   │
│  │                                    │                                        │   │
│  │                                    ▼                                        │   │
│  │                         Semantic Analysis:                                  │   │
│  │                         - Type resolution                                   │   │
│  │                         - Trait implementation                              │   │
│  │                         - ADT classification                                │   │
│  │                         - Initializer pattern                               │   │
│  │                                    │                                        │   │
│  └────────────────────────────────────┼────────────────────────────────────────┘   │
│                                       │                                             │
│                                       ▼                                             │
│                          ┌─────────────────────────┐                                │
│                          │    type-info.json       │                                │
│                          │    (.borrowscope/)      │                                │
│                          └────────────┬────────────┘                                │
│                                       │                                             │
│         ┌─────────────────────────────┼─────────────────────────────┐               │
│         │                             │                             │               │
│         ▼                             ▼                             ▼               │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                          MACRO (Compile-Time)                               │   │
│  │                                                                             │   │
│  │   TypeInfoCache                    OwnershipVisitor                         │   │
│  │   ┌─────────────────┐              ┌─────────────────────────────────┐      │   │
│  │   │ Load JSON once  │              │ Walk AST, for each let binding: │      │   │
│  │   │ via OnceLock    │─────────────▶│ 1. Lookup type info             │      │   │
│  │   │                 │              │ 2. Match initializer_kind       │      │   │
│  │   │ by_function:    │              │ 3. Generate tracking call       │      │   │
│  │   │   HashMap<...>  │              │ 4. Assign unique var_id         │      │   │
│  │   │                 │              │ 5. Track scope for drops        │      │   │
│  │   │ by_name:        │              └─────────────────────────────────┘      │   │
│  │   │   HashMap<...>  │                              │                        │   │
│  │   └─────────────────┘                              │                        │   │
│  │                                                    ▼                        │   │
│  │                                    Instrumented TokenStream                 │   │
│  │                                                                             │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                       │                                             │
│                                       ▼                                             │
│                          ┌─────────────────────────┐                                │
│                          │   Compiled Binary       │                                │
│                          │   (with tracking calls) │                                │
│                          └────────────┬────────────┘                                │
│                                       │                                             │
│                                       ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                          RUNTIME (Execution)                                │   │
│  │                                                                             │   │
│  │   Global State                     Event Recording                          │   │
│  │   ┌─────────────────┐              ┌─────────────────────────────────┐      │   │
│  │   │ TRACKER:        │              │ Each track_* call:              │      │   │
│  │   │   Mutex<Tracker>│◀─────────────│ 1. Lock TRACKER                 │      │   │
│  │   │                 │              │ 2. Generate timestamp           │      │   │
│  │   │ TIMESTAMP:      │              │ 3. Create Event variant         │      │   │
│  │   │   AtomicU64     │              │ 4. Push to events vector        │      │   │
│  │   └─────────────────┘              │ 5. Return value unchanged       │      │   │
│  │          │                         └─────────────────────────────────┘      │   │
│  │          ▼                                                                  │   │
│  │   ┌─────────────────┐                                                       │   │
│  │   │ Vec<Event>      │──────▶ get_events() ──────▶ JSON Export               │   │
│  │   │ (40+ variants)  │──────▶ get_graph()  ──────▶ OwnershipGraph            │   │
│  │   └─────────────────┘                                                       │   │
│  │                                                                             │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

#### Why Three Separate Projects?

The separation into three projects is not arbitrary—it reflects fundamental constraints in Rust's compilation model and the need for different execution contexts.

The analyzer must run before compilation because it needs access to rust-analyzer's semantic analysis, which requires a fully-configured workspace with resolved dependencies. This analysis cannot happen inside a procedural macro because macros execute in a sandboxed environment without access to external tools or the filesystem (in a portable way).

The macro must run during compilation because it transforms source code. It cannot run before compilation (the code doesn't exist in transformed form yet) or after (the binary is already built). The macro bridges the gap between static analysis and runtime by encoding type information into the generated tracking calls.

The runtime must execute with the program because ownership events occur during execution. Static analysis can determine types, but it cannot observe actual control flow, loop iterations, or conditional branches. The runtime captures what actually happens, not what might happen.

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    WHY THREE PROJECTS ARE NECESSARY                                 │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                                                                             │   │
│  │   CONSTRAINT                        SOLUTION                                │   │
│  │   ──────────                        ────────                                │   │
│  │                                                                             │   │
│  │   Proc macros cannot access         borrowscope-analyzer                    │   │
│  │   type information                  Runs pre-build with rust-analyzer       │   │
│  │                                                                             │   │
│  │   Type info must reach the          type-info.json                          │   │
│  │   macro somehow                     File-based communication                │   │
│  │                                                                             │   │
│  │   Code transformation must          borrowscope-macro                       │   │
│  │   happen at compile time            Procedural macro with JSON loading      │   │
│  │                                                                             │   │
│  │   Ownership events occur            borrowscope-runtime                     │   │
│  │   during execution                  Library linked into final binary        │   │
│  │                                                                             │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                     │
│  The three projects form a pipeline that crosses compilation boundaries:            │
│                                                                                     │
│       Pre-Build          Compile-Time           Runtime                            │
│       ─────────          ────────────           ───────                            │
│       analyzer ────JSON────▶ macro ────Binary────▶ runtime                         │
│                                                                                     │
│  Each boundary requires a different communication mechanism:                        │
│  - Analyzer → Macro: JSON file (survives process boundary)                         │
│  - Macro → Runtime: Generated code (embedded in binary)                            │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

#### Integration Example: Complete Workflow

The following example demonstrates the complete pipeline from source code to runtime events:

```rust
// src/lib.rs - User's source code
use std::rc::Rc;
use std::cell::RefCell;
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn demonstrate_ownership() {
    // Create a reference-counted, interior-mutable container
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    
    // Clone the Rc (shared ownership)
    let also_shared = Rc::clone(&shared);
    
    // Borrow the inner value
    {
        let borrowed = shared.borrow();
        println!("Values: {:?}", *borrowed);
    } // borrowed dropped here, releasing the borrow
    
    // Mutably borrow and modify
    shared.borrow_mut().push(4);
}

fn main() {
    reset();
    demonstrate_ownership();
    
    // Export the ownership trace
    export_json("ownership_trace.json").unwrap();
}
```

**Step 1: Run the analyzer**

```bash
$ borrowscope-analyzer .

BorrowScope Analyzer v0.1.0
═══════════════════════════════════════════
Project: /home/user/demo

  Loading workspace...
  Analyzing: src/lib.rs

═══════════════════════════════════════════
Summary:
  Files analyzed: 1
  Variables found: 4
  Types resolved: 4 (100.0%)

Output: /home/user/demo/.borrowscope/type-info.json
```

The analyzer produces type-info.json with semantic classifications:

```json
{
  "version": "2.2",
  "by_function": {
    "demonstrate_ownership": {
      "shared": [{
        "name": "shared",
        "ty": "Rc<RefCell<Vec<i32>>>",
        "initializer_kind": "rc_new",
        "is_rc": true,
        "decl_index": 0
      }],
      "also_shared": [{
        "name": "also_shared", 
        "ty": "Rc<RefCell<Vec<i32>>>",
        "initializer_kind": "rc_clone",
        "is_rc": true,
        "decl_index": 0
      }],
      "borrowed": [{
        "name": "borrowed",
        "ty": "Ref<Vec<i32>>",
        "initializer_kind": "refcell_borrow",
        "is_guard": true,
        "decl_index": 0
      }]
    }
  }
}
```

**Step 2: Build the project**

```bash
$ cargo build
```

During compilation, the `#[trace_borrow]` macro loads type-info.json and transforms the function. The macro's internal processing:

```
Processing function: demonstrate_ownership
  Binding: shared
    Lookup: ("demonstrate_ownership", "shared", 0)
    Found: initializer_kind = "rc_new"
    Emit: track_rc_new_with_id(1, "shared", "Rc<RefCell<Vec<i32>>>", ...)
  
  Binding: also_shared  
    Lookup: ("demonstrate_ownership", "also_shared", 0)
    Found: initializer_kind = "rc_clone"
    Emit: track_rc_clone_with_id(2, 1, "also_shared", ...)
  
  Binding: borrowed
    Lookup: ("demonstrate_ownership", "borrowed", 0)
    Found: initializer_kind = "refcell_borrow"
    Emit: track_refcell_borrow(3, 1, ...)
```

**Step 3: Run the program**

```bash
$ cargo run
Values: [1, 2, 3]
```

The runtime captures events as the program executes. The exported ownership_trace.json contains:

```json
[
  {
    "type": "RcNew",
    "timestamp": 1,
    "var_name": "shared",
    "var_id": "var_1",
    "type_name": "Rc<RefCell<Vec<i32>>>",
    "strong_count": 1,
    "weak_count": 0
  },
  {
    "type": "RcClone", 
    "timestamp": 2,
    "var_name": "also_shared",
    "var_id": "var_2",
    "source_id": "var_1",
    "strong_count": 2,
    "weak_count": 0
  },
  {
    "type": "RefCellBorrow",
    "timestamp": 3,
    "borrow_id": "var_3",
    "refcell_id": "var_1",
    "is_mutable": false
  },
  {
    "type": "RefCellDrop",
    "timestamp": 4,
    "borrow_id": "var_3"
  },
  {
    "type": "RefCellBorrow",
    "timestamp": 5,
    "borrow_id": "anon_4",
    "refcell_id": "var_1", 
    "is_mutable": true
  },
  {
    "type": "RefCellDrop",
    "timestamp": 6,
    "borrow_id": "anon_4"
  },
  {
    "type": "Drop",
    "timestamp": 7,
    "var_id": "var_2"
  },
  {
    "type": "Drop",
    "timestamp": 8,
    "var_id": "var_1"
  }
]
```

The event stream captures the complete ownership lifecycle: Rc allocation, shared cloning, RefCell borrowing and release, and finally the drops in reverse declaration order. Each event includes correlation IDs that link related operations—the `RcClone` references its source, the `RefCellBorrow` references its container, and drops reference the original allocations.

---

## 6. Usage Guide

### 6.1 Running the Analyzer

The borrowscope-analyzer is a standalone binary that analyzes a Rust project and produces type information. It requires the project path as its only argument:

```bash
# From the BorrowScope workspace
cargo run -p borrowscope-analyzer -- /path/to/your/project

# Or if installed
borrowscope-analyzer /path/to/your/project
```

The analyzer expects a valid Cargo project with a `Cargo.toml` at the specified path. It will:

1. Load the workspace using rust-analyzer's infrastructure
2. Discover the Rust sysroot for standard library type resolution
3. Analyze all `.rs` files in the project (excluding dependencies and target/)
4. Write results to `.borrowscope/type-info.json`

Example output:

```
BorrowScope Analyzer v0.1.0
═══════════════════════════════════════════
Project: /home/user/my-project

  Loading workspace...
  Analyzing: src/main.rs
  Analyzing: src/lib.rs
  Analyzing: src/utils.rs

═══════════════════════════════════════════
Summary:
  Files analyzed: 3
  Variables found: 47
  Types resolved: 47 (100.0%)

Output: /home/user/my-project/.borrowscope/type-info.json
```

The analyzer logs progress to stderr and can be configured with the `RUST_LOG` environment variable for detailed debugging:

```bash
RUST_LOG=debug cargo run -p borrowscope-analyzer -- /path/to/project
```

### 6.2 Workflow for Users

The complete workflow for using BorrowScope with full type information involves three steps:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      COMPLETE USER WORKFLOW                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  STEP 1: Analyze                                                            │
│  ───────────────                                                            │
│                                                                             │
│  $ borrowscope-analyzer .                                                   │
│                                                                             │
│  Creates: .borrowscope/type-info.json                                       │
│                                                                             │
│                         │                                                   │
│                         ▼                                                   │
│                                                                             │
│  STEP 2: Build                                                              │
│  ────────────                                                               │
│                                                                             │
│  $ cargo build                                                              │
│                                                                             │
│  The #[trace_borrow] macro reads type-info.json during expansion            │
│  and generates accurate tracking calls.                                     │
│                                                                             │
│                         │                                                   │
│                         ▼                                                   │
│                                                                             │
│  STEP 3: Run                                                                │
│  ─────────                                                                  │
│                                                                             │
│  $ cargo run                                                                │
│                                                                             │
│  Runtime tracking captures ownership events with full type information.     │
│  Export to JSON for visualization.                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 1: Analyze**

Run the analyzer whenever your code changes significantly. The analyzer needs to re-run when:

- New variables are added
- Variable types change
- Files are added or renamed
- Line numbers shift significantly (due to added/removed code)

For development workflows, consider running the analyzer as part of your build process or IDE save hook.

**Step 2: Build**

Build your project normally with `cargo build` or `cargo run`. The `#[trace_borrow]` macro will automatically detect and use the type information file if present. No code changes are required—the macro transparently upgrades its tracking precision when type information is available.

**Step 3: Run and Analyze**

Execute your instrumented program. The runtime tracking will now include accurate type information in its events. Export the events for analysis:

```rust
use borrowscope_runtime::*;

fn main() {
    reset();
    
    // Your instrumented code runs here
    my_function();
    
    // Export events with full type information
    let events = get_events();
    std::fs::write(
        "trace.json",
        serde_json::to_string_pretty(&events).unwrap()
    ).unwrap();
}
```

### Automation with Build Scripts

For projects that want automatic analysis, a `build.rs` script can invoke the analyzer:

```rust
// build.rs
use std::process::Command;

fn main() {
    // Re-run if any Rust source changes
    println!("cargo:rerun-if-changed=src/");
    
    // Run the analyzer
    let status = Command::new("borrowscope-analyzer")
        .arg(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .status();
    
    if let Err(e) = status {
        println!("cargo:warning=borrowscope-analyzer not found: {}", e);
        println!("cargo:warning=Type information will not be available");
    }
}
```

This ensures type information is always up-to-date, though it adds to build time. For large projects, consider running the analyzer only in CI or as a separate development step.

### Verifying Type Information

To verify the analyzer is working correctly, inspect the generated JSON:

```bash
# Check that the file was created
cat .borrowscope/type-info.json | head -50

# Count variables by type
cat .borrowscope/type-info.json | jq '.files[].[] | .ty' | sort | uniq -c | sort -rn

# Find all Rc variables
cat .borrowscope/type-info.json | jq '.files[][] | select(.is_rc == true) | .name'

# Check resolution rate
cat .borrowscope/type-info.json | jq '[.files[][]] | length as $total | [.files[][] | select(.ty != "unknown")] | length as $resolved | "\($resolved)/\($total) types resolved"'
```

A 100% resolution rate indicates the analyzer successfully determined types for all variables. Lower rates may indicate files outside the crate graph or analysis errors—check the analyzer's stderr output for warnings.

---

## 7. Performance Characteristics

The analyzer's performance is dominated by workspace loading rather than the actual type extraction. Understanding this breakdown helps set appropriate expectations and identify optimization opportunities.

### Timing Breakdown

Analysis of a small project (single file, ~100 variables) shows the following timing distribution:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PERFORMANCE BREAKDOWN                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Total Time: ~45-50 seconds                                                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│    │
│  │            Workspace Loading (~32s, 65%)                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐                                    │
│  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│                                    │
│  │    Type Analysis (~12s, 25%)       │                                    │
│  └─────────────────────────────────────┘                                    │
│  ┌──────────────┐                                                           │
│  │░░░░░░░░░░░░░░│                                                           │
│  │ Compile (~5s)│                                                           │
│  └──────────────┘                                                           │
│                                                                             │
│  Workspace Loading includes:                                                │
│    • Sysroot discovery (rustc --print sysroot)                              │
│    • Standard library metadata loading                                      │
│    • Cargo.toml parsing and dependency resolution                           │
│    • Building the semantic database                                         │
│                                                                             │
│  Type Analysis includes:                                                    │
│    • Parsing source files                                                   │
│    • Walking syntax trees                                                   │
│    • Resolving types via Semantics API                                      │
│    • JSON serialization                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The workspace loading phase is expensive because rust-analyzer must build a complete semantic model of the project and its dependencies. This includes loading metadata for the entire standard library, which contains thousands of types and trait implementations.

### Scaling Characteristics

Analysis time scales primarily with project complexity rather than line count:

| Project Size | Dependencies | Variables | Load Time | Analysis Time | Total |
|--------------|--------------|-----------|-----------|---------------|-------|
| Small (1 file) | std only | ~100 | ~32s | ~12s | ~45s |
| Medium (10 files) | 5-10 crates | ~500 | ~35s | ~15s | ~50s |
| Large (100 files) | 50+ crates | ~2000 | ~60s | ~30s | ~90s |

The workspace loading time increases modestly with dependency count because rust-analyzer must resolve and load metadata for each crate. The analysis time scales roughly linearly with the number of variables.

### Optimization Opportunities

Several strategies can improve performance for different use cases:

**Incremental Analysis**: The current implementation performs full analysis on every run. A future version could cache the loaded workspace and only re-analyze changed files. rust-analyzer's internal architecture supports incremental updates, but exposing this through the `ra_ap_*` crates requires additional implementation work.

**Parallel File Processing**: Type extraction for different files is independent and could be parallelized. The current implementation processes files sequentially, but the `Semantics` API is thread-safe for read operations.

**Selective Analysis**: For large projects, analyzing only files containing `#[trace_borrow]` annotations would reduce work. This requires a two-pass approach: first scan for annotations, then analyze only relevant files.

**Workspace Caching**: The loaded `RootDatabase` could theoretically be serialized and reused across runs. However, rust-analyzer's database structures are not designed for serialization, making this approach impractical without significant upstream changes.

### When to Run the Analyzer

Given the analysis cost, consider these usage patterns:

**Development**: Run the analyzer once when starting work on a feature, then re-run only when adding new variables or changing types significantly. Minor edits that don't affect types don't require re-analysis.

**CI/CD**: Include analysis in the CI pipeline to ensure type information is always current for release builds. The ~1 minute overhead is acceptable for CI but may be too slow for rapid local iteration.

**IDE Integration**: A future rust-analyzer extension could provide type information directly to the macro, eliminating the need for a separate analysis step. This would leverage rust-analyzer's existing incremental analysis and caching.

### Memory Usage

The analyzer's memory footprint is dominated by rust-analyzer's semantic database:

| Phase | Memory Usage |
|-------|--------------|
| Startup | ~50 MB |
| After workspace load | ~500-800 MB |
| During analysis | ~600-900 MB |
| Peak (large projects) | ~1-2 GB |

The memory usage reflects rust-analyzer's design for IDE responsiveness—it caches extensively to enable fast queries. For the analyzer's batch processing use case, this caching is less beneficial but unavoidable given the current API design.

---

## 8. Limitations & Future Work

### Current Limitations

The borrowscope-analyzer represents a working solution to the proc-macro type blindness problem, but several limitations remain that warrant acknowledgment.

**Pattern Binding Decomposition**: The analyzer currently captures pattern bindings as a single unit. For destructuring patterns like `let (a, b, c) = tuple;`, the entire pattern `(a, b, c)` is recorded as one entry rather than three separate variables. This means the macro cannot look up individual components by their names. A future version should decompose patterns into their constituent bindings, each with its own type information.

**Line Number Drift**: The type information file contains absolute line numbers. If the source file is modified between analysis and compilation—even by adding a comment—line numbers may no longer match. The macro's lookup will fail, triggering fallback to syntactic heuristics. Robust solutions include content-based hashing or relative positioning within functions.

**Macro-Generated Code**: Variables created by other procedural macros are not visible to the analyzer because it runs before macro expansion. If a macro generates `let` bindings, those bindings will not have type information. This is an inherent limitation of the two-phase approach.

**Workspace-External Files**: Files not included in the Cargo workspace (standalone scripts, files excluded via `Cargo.toml`) receive only syntax-based analysis. The analyzer cannot resolve types for code outside the crate graph because rust-analyzer requires a complete project model for semantic analysis.

**Generic Type Parameters**: While the analyzer captures fully instantiated types like `Vec<i32, Global>`, it cannot provide information about generic type parameters in isolation. A function `fn foo<T>(x: T)` will have `x` typed as the generic parameter `T`, not a concrete type. This is correct behavior but limits tracking precision for generic code.

**Closure Capture Analysis**: The analyzer identifies closure types but does not currently extract information about captured variables. Knowing that a closure captures `x` by reference versus by move would enable more precise tracking of ownership flow into closures.

**Build Script Outputs**: Projects with complex build scripts that generate Rust code may not have that generated code analyzed. The analyzer runs `cargo check` to obtain build script outputs, but timing and caching issues can cause inconsistencies.

### Future Work

Several enhancements would improve the analyzer's utility and integration:

**Macro Integration**: The `borrowscope-macro` crate needs modification to read and utilize the type information file. This involves:
- Loading `type-info.json` at macro expansion time
- Building a lookup index by source location
- Modifying the transformation logic to query type information
- Implementing graceful fallback when information is unavailable

**Incremental Analysis**: Implementing incremental analysis would dramatically improve performance for iterative development. This requires:
- Tracking file modification times or content hashes
- Persisting the rust-analyzer database between runs
- Updating only changed files while preserving cached analysis

**IDE Extension**: A rust-analyzer extension could provide type information directly to the macro through a language server protocol extension or shared memory mechanism. This would eliminate the separate analysis step entirely and provide always-current type information.

**User-Defined Type Classification**: Supporting user-provided rules for classifying custom smart pointer types would extend tracking to domain-specific abstractions. A configuration file could map type patterns to tracking categories:

```toml
# .borrowscope/config.toml
[classifications]
"MySmartPtr<" = "smart_pointer"
"CustomCell<" = "interior_mutability"
```

**Trait-Based Detection**: Rather than string matching on type names, future versions could query whether types implement specific traits (`Deref`, `DerefMut`, `Clone`, `Copy`). This would provide more robust classification that works with type aliases and newtype wrappers.

**Cross-Crate Analysis**: Analyzing dependencies to understand their type structures would enable tracking of ownership across crate boundaries. Currently, types from dependencies are resolved but not deeply analyzed for their ownership semantics.

**Parallel Analysis**: Implementing parallel file processing would improve performance on multi-core systems. The semantic queries are thread-safe for reading, enabling concurrent analysis of independent files.

**Watch Mode**: A file-watching mode that re-analyzes on source changes would integrate better with development workflows. Combined with incremental analysis, this could provide near-instant updates as code is edited.

### Research Directions

Beyond immediate improvements, several research directions could advance BorrowScope's capabilities:

**Borrow Checker Integration**: Deeper integration with rust-analyzer's borrow checking analysis could provide information about lifetime constraints and borrow conflicts, enabling visualization of why certain code patterns are rejected by the compiler.

**Data Flow Analysis**: Tracking how values flow through a program—not just where they are created—would enable richer visualizations showing the complete lifecycle of owned data.

**Async Ownership Tracking**: Rust's async/await introduces complex ownership patterns where values are captured by futures and may move between threads. Specialized analysis for async code would improve tracking accuracy in concurrent programs.

---

## 9. Future Development Roadmap

This section presents a comprehensive technical roadmap for evolving the borrowscope-analyzer from its current initializer-focused analysis to a complete semantic analysis system capable of eliminating all heuristic pattern matching from the macro. The goal is to achieve 100% semantic coverage, where every ownership-related operation is classified through rust-analyzer's type system rather than syntactic string matching.

### 9.1 Current Coverage Analysis

The analyzer currently tracks **variable initializers**—the expressions that create new variables. This covers 36 of 109 patterns (33%) used by the macro for ownership tracking. The remaining patterns involve method calls on existing variables, standalone expressions, and self-borrow type inference, none of which produce new variable bindings.

#### Coverage by Category

| Category | Total Patterns | ✅ Semantic | ⚠️ Partial | ❌ Syntactic |
|----------|---------------|-------------|------------|--------------|
| Smart pointer creation | 5 | 5 | 0 | 0 |
| Smart pointer clone | 2 | 2 | 0 | 0 |
| RefCell/Cell methods | 4 | 3 | 0 | 1 |
| Box operations | 3 | 3 | 0 | 0 |
| Pin operations | 2 | 1 | 1 | 0 |
| Cow operations | 3 | 2 | 0 | 1 |
| Weak reference operations | 3 | 3 | 0 | 0 |
| OnceCell/OnceLock operations | 5 | 2 | 2 | 1 |
| MaybeUninit operations | 6 | 2 | 2 | 2 |
| Concurrency operations | 12 | 5 | 0 | 7 |
| Guard method patterns | 9 | 8 | 1 | 0 |
| Self borrow inference (immutable) | 19 | 0 | 0 | 19 |
| Self borrow inference (mutable) | 25 | 0 | 0 | 25 |
| Self borrow inference (consuming) | 3 | 0 | 0 | 3 |
| Unwrap methods | 5 | 0 | 0 | 5 |
| Clone method | 1 | 0 | 1 | 0 |
| Transmute detection | 2 | 0 | 0 | 2 |
| **TOTAL** | **109** | **36** | **7** | **66** |

#### The Fundamental Gap

The analyzer's current architecture creates a fundamental gap: it tracks what types variables **have**, not what operations are **performed on** those variables. Consider:

```rust
let x = Rc::new(1);  // ✅ Tracked: initializer_kind = "rc_new"
let y = x.clone();   // ✅ Tracked: initializer_kind = "rc_clone" (via result type)
x.some_method();     // ❌ Not tracked: method call, no new variable
```

The third line represents the gap—method calls that don't create new variables but still represent significant ownership operations (like `Cell::set`, `Sender::send`, or `JoinHandle::join`).

### 9.2 Architecture Evolution

To achieve 100% semantic coverage, the analyzer must evolve from initializer-only analysis to full expression analysis. This requires tracking three additional categories of operations.

#### Current Architecture: Initializer Analysis

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CURRENT: INITIALIZER-ONLY ANALYSIS                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                    Analysis Output                             │
│  ───────────                    ───────────────                             │
│  let x = Rc::new(1);    ───▶    { name: "x", ty: "Rc<i32>",                │
│                                   initializer_kind: "rc_new" }              │
│                                                                             │
│  let y = x.clone();     ───▶    { name: "y", ty: "Rc<i32>",                │
│                                   initializer_kind: "rc_clone" }            │
│                                                                             │
│  cell.set(42);          ───▶    (not tracked - no new variable)            │
│                                                                             │
│  thread::spawn(|| {});  ───▶    (not tracked - no new variable)            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Target Architecture: Full Expression Analysis

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TARGET: FULL EXPRESSION ANALYSIS                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                    Analysis Output                             │
│  ───────────                    ───────────────                             │
│  let x = Rc::new(1);    ───▶    { name: "x", ty: "Rc<i32>",                │
│                                   initializer_kind: "rc_new",               │
│                                   method_calls: [...] }                     │
│                                                                             │
│  let y = x.clone();     ───▶    { name: "y", ty: "Rc<i32>",                │
│                                   initializer_kind: "rc_clone" }            │
│                                                                             │
│  cell.set(42);          ───▶    method_calls on "cell": [{                 │
│                                   method: "set", line: 5,                   │
│                                   operation: "cell_set",                    │
│                                   self_borrow: "mutable" }]                 │
│                                                                             │
│  thread::spawn(|| {});  ───▶    expressions: [{                            │
│                                   kind: "function_call",                    │
│                                   operation: "thread_spawn",                │
│                                   line: 7 }]                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### New Analysis Capabilities Required

1. **Method Call Tracking**: For each tracked variable, record all method calls made on it, including the method name, resolved receiver type, and semantic operation classification.

2. **Standalone Expression Tracking**: Track function calls and expressions that don't create variables but represent significant operations (e.g., `thread::spawn`, `transmute`).

3. **Self-Borrow Type Resolution**: Use rust-analyzer's method resolution to determine whether methods take `&self`, `&mut self`, or `self`, eliminating 47 heuristic patterns.

### 9.3 Implementation Phases

The expansion is organized into five phases, each targeting a specific category of patterns. Phases are ordered by dependency and complexity, allowing incremental progress with testable milestones.

#### Phase 1: Method Call Tracking on Known Variables

**Objective**: Track method calls where the receiver is a variable with a known tracked type.

**Patterns Eliminated**: 13 syntactic patterns

**Implementation Strategy**:

After collecting variables in `analyze_file()`, walk all method call expressions in the syntax tree. For each method call, check if the receiver resolves to a tracked variable. If so, classify the method call based on the receiver type and method name.

```rust
// Pseudocode for Phase 1 implementation
fn analyze_method_calls(
    sema: &Semantics<'_, RootDatabase>,
    source_file: &ast::SourceFile,
    variables: &mut HashMap<String, VariableTypeInfo>,
    db: &RootDatabase,
) {
    for node in source_file.syntax().descendants() {
        if let Some(method_call) = ast::MethodCallExpr::cast(node) {
            // Extract receiver name (e.g., "cell" from "cell.set(42)")
            let receiver_name = extract_receiver_name(&method_call)?;
            
            // Check if receiver is a tracked variable
            if let Some(var_info) = variables.get_mut(&receiver_name) {
                let method_name = method_call.name_ref()?.text().to_string();
                let receiver_type = sema.type_of_expr(&method_call.receiver()?)?;
                
                // Classify based on receiver type and method
                if let Some(operation) = classify_method_call(&receiver_type, &method_name, db) {
                    var_info.method_calls.push(MethodCallInfo {
                        method: method_name,
                        operation,
                        line: line_number(&method_call),
                        self_borrow: resolve_self_borrow(sema, &method_call, db),
                    });
                }
            }
        }
    }
}
```

**Specific Patterns**:

| Method | Receiver Type | Operation | Current Detection |
|--------|---------------|-----------|-------------------|
| `.set(v)` | `Cell<T>` | `cell_set` | `method_name == "set"` |
| `.to_mut()` | `Cow<T>` | `cow_to_mut` | `method.to_string() == "to_mut"` |
| `.set(v)` | `OnceCell<T>` | `once_cell_set` | `method_name == "set"` |
| `.get()` | `OnceCell<T>` | `once_cell_get` | `method_name == "get"` |
| `.get_or_init(f)` | `OnceCell<T>` | `once_cell_get_or_init` | `method_name == "get_or_init"` |
| `.write(v)` | `MaybeUninit<T>` | `maybe_uninit_write` | `method_name == "write"` |
| `.assume_init()` | `MaybeUninit<T>` | `maybe_uninit_assume_init` | `method_name == "assume_init"` |
| `.assume_init_read()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_read` | `method_name == "assume_init_read"` |
| `.assume_init_drop()` | `MaybeUninit<T>` | `maybe_uninit_assume_init_drop` | `method_name == "assume_init_drop"` |
| `.send(v)` | `Sender<T>` | `channel_send` | `method_name == "send"` |
| `.recv()` | `Receiver<T>` | `channel_recv` | `method_name == "recv"` |
| `.try_recv()` | `Receiver<T>` | `channel_try_recv` | `method_name == "try_recv"` |
| `.join()` | `JoinHandle<T>` | `thread_join` | `method_name == "join"` |

**Estimated Effort**: 2-3 hours

#### Phase 2: Standalone Expression Tracking

**Objective**: Track function calls and expressions that don't create variables but represent significant ownership operations.

**Patterns Eliminated**: 4 syntactic patterns

**Implementation Strategy**:

Add a new output field `expressions` that captures standalone calls. Walk all call expressions and check if they resolve to known functions like `thread::spawn` or `std::mem::transmute`.

```rust
// Pseudocode for Phase 2 implementation
fn analyze_expressions(
    sema: &Semantics<'_, RootDatabase>,
    source_file: &ast::SourceFile,
    db: &RootDatabase,
) -> Vec<ExpressionInfo> {
    let mut expressions = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(call) = ast::CallExpr::cast(node) {
            // Resolve the called function
            if let Some(path_expr) = call.expr().and_then(ast::PathExpr::cast) {
                if let Some(resolved) = sema.resolve_path(&path_expr.path()?) {
                    let path = get_function_canonical_path(&resolved, db);
                    
                    let operation = match path.as_str() {
                        "std::thread::spawn" | "core::thread::spawn" => Some("thread_spawn"),
                        "std::mem::transmute" | "core::mem::transmute" => Some("transmute"),
                        _ => None,
                    };
                    
                    if let Some(op) = operation {
                        expressions.push(ExpressionInfo {
                            kind: "function_call".to_string(),
                            operation: op.to_string(),
                            line: line_number(&call),
                            path: Some(path),
                        });
                    }
                }
            }
        }
    }
    
    expressions
}
```

**Specific Patterns**:

| Pattern | Operation | Current Detection |
|---------|-----------|-------------------|
| `thread::spawn(...)` | `thread_spawn` | `path_str.contains("thread::spawn")` |
| `std::thread::spawn(...)` | `thread_spawn` | `path_str.contains("std::thread::spawn")` |
| `transmute(...)` | `transmute` | `path_str.contains("transmute")` |
| `std::mem::transmute(...)` | `transmute` | `fn_name.contains("transmute")` |

**Estimated Effort**: 1-2 hours

#### Phase 3: Self-Borrow Type Inference Elimination

**Objective**: Replace all heuristic self-borrow inference with semantic method resolution.

**Patterns Eliminated**: 47 syntactic patterns (the largest category)

**The Problem**:

The macro currently infers whether a method takes `&self`, `&mut self`, or `self` based on method name patterns:

```rust
// Current heuristic approach in borrowscope-macro
fn infer_self_borrow_type(method_name: &str) -> SelfBorrowType {
    // Immutable patterns
    if method_name.starts_with("as_") || method_name.starts_with("to_") ||
       method_name.starts_with("is_") || method_name.starts_with("get") ||
       matches!(method_name, "len" | "capacity" | "iter" | "clone" | ...) {
        return SelfBorrowType::Immutable;
    }
    
    // Mutable patterns
    if method_name.starts_with("push") || method_name.starts_with("pop") ||
       method_name.starts_with("insert") || method_name.starts_with("set") ||
       matches!(method_name, "clear" | "sort" | "reverse" | ...) {
        return SelfBorrowType::Mutable;
    }
    
    // Consuming patterns
    if method_name.starts_with("into_") ||
       matches!(method_name, "unwrap" | "expect") {
        return SelfBorrowType::Consuming;
    }
    
    SelfBorrowType::Unknown
}
```

This heuristic approach fails for:
- Methods that don't follow naming conventions
- User-defined methods with arbitrary names
- Trait methods with non-standard signatures

**The Solution**:

Use rust-analyzer's `resolve_method_call` to get the actual method definition, then inspect its self parameter:

```rust
fn resolve_self_borrow(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> Option<String> {
    // Resolve method call to its definition
    let func = sema.resolve_method_call(method_call)?;
    
    // Get the self parameter from the function signature
    let self_param = func.self_param(db)?;
    
    // Determine access type
    match self_param.access(db) {
        ra_ap_hir::Access::Shared => Some("immutable".to_string()),
        ra_ap_hir::Access::Exclusive => Some("mutable".to_string()),
        ra_ap_hir::Access::Owned => Some("consuming".to_string()),
    }
}
```

**Patterns Eliminated**:

*Immutable (19 patterns)*: `as_*`, `to_*`, `is_*`, `get*`, `len`, `capacity`, `iter`, `chars`, `bytes`, `lines`, `split`, `trim`, `contains`, `starts_with`, `ends_with`, `find`, `clone`, `first`, `last`

*Mutable (25 patterns)*: `push*`, `pop*`, `insert*`, `remove*`, `append*`, `add*`, `set*`, `update*`, `modify*`, `clear`, `truncate`, `extend`, `drain`, `sort`, `reverse`, `dedup`, `retain`, `tick`, `recv`, `send`, `changed`, `wait`, `acquire`, `lock`, `write`

*Consuming (3 patterns)*: `into_*`, `unwrap`, `expect`

**Estimated Effort**: 3-4 hours

#### Phase 4: Unwrap Method Tracking

**Objective**: Track unwrap operations semantically by detecting method calls on `Option<T>` and `Result<T, E>`.

**Patterns Eliminated**: 5 syntactic patterns

**Implementation Strategy**:

When processing method calls, check if the receiver type is `Option` or `Result`, then classify unwrap-family methods:

```rust
fn classify_unwrap_method(
    receiver_type: &ra_ap_hir::Type,
    method_name: &str,
    db: &RootDatabase,
) -> Option<String> {
    // Check if receiver is Option<T> or Result<T, E>
    if !is_option_or_result(receiver_type, db) {
        return None;
    }
    
    match method_name {
        "unwrap" => Some("unwrap".to_string()),
        "expect" => Some("expect".to_string()),
        "unwrap_or" => Some("unwrap_or".to_string()),
        "unwrap_or_else" => Some("unwrap_or_else".to_string()),
        "unwrap_or_default" => Some("unwrap_or_default".to_string()),
        _ => None,
    }
}

fn is_option_or_result(ty: &ra_ap_hir::Type, db: &RootDatabase) -> bool {
    let display = ty.display(db).to_string();
    display.starts_with("Option<") || display.starts_with("Result<")
}
```

**Specific Patterns**:

| Method | Operation | Current Detection |
|--------|-----------|-------------------|
| `.unwrap()` | `unwrap` | `method_name == "unwrap"` |
| `.expect(msg)` | `expect` | `method_name == "expect"` |
| `.unwrap_or(v)` | `unwrap_or` | `method_name == "unwrap_or"` |
| `.unwrap_or_else(f)` | `unwrap_or_else` | `method_name == "unwrap_or_else"` |
| `.unwrap_or_default()` | `unwrap_or_default` | `method_name == "unwrap_or_default"` |

**Estimated Effort**: 1 hour

#### Phase 5: Clone Tracking

**Objective**: Track `.clone()` calls semantically by verifying the method resolves to `Clone::clone`.

**Patterns Eliminated**: 1 syntactic pattern

**Implementation Strategy**:

When encountering a `.clone()` method call, verify it's actually `Clone::clone` (not some other method named `clone`) by checking the trait:

```rust
fn is_clone_trait_call(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> bool {
    let method_name = method_call.name_ref()?.text().to_string();
    if method_name != "clone" {
        return false;
    }
    
    // Resolve to the actual method definition
    let func = sema.resolve_method_call(method_call)?;
    
    // Check if it's a trait method
    let assoc_item = func.as_assoc_item(db)?;
    let container_trait = assoc_item.container_trait(db)?;
    
    // Get the trait's canonical path
    let trait_path = get_trait_canonical_path(&container_trait, db);
    
    trait_path == "core::clone::Clone" || trait_path == "std::clone::Clone"
}
```

**Estimated Effort**: 30 minutes

### 9.4 Schema Evolution

The expanded analysis capabilities require evolving the output schema from v2.2 to v2.4. The new schema adds two major components: method call tracking per variable and standalone expression tracking per file.

#### Current Schema (v2.2)

```json
{
  "version": "2.2",
  "files": {
    "src/main.rs": [
      {
        "name": "x",
        "ty": "Rc<i32>",
        "is_copy": false,
        "is_rc": true,
        "is_arc": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "is_box": false,
        "is_send": true,
        "is_sync": true,
        "initializer_kind": "rc_new",
        "source_location": { "line": 5, "column": 9 },
        "function_context": "main"
      }
    ]
  },
  "by_name": { ... },
  "by_function": { ... }
}
```

#### Target Schema (v2.4)

```json
{
  "version": "2.4",
  "files": {
    "src/main.rs": [
      {
        "name": "x",
        "ty": "Rc<i32>",
        "is_copy": false,
        "is_rc": true,
        "is_arc": false,
        "is_refcell": false,
        "is_cell": false,
        "is_mutex": false,
        "is_rwlock": false,
        "is_box": false,
        "is_send": true,
        "is_sync": true,
        "initializer_kind": "rc_new",
        "source_location": { "line": 5, "column": 9 },
        "function_context": "main",
        "method_calls": [
          {
            "method": "clone",
            "line": 8,
            "operation": "rc_clone",
            "self_borrow": "immutable",
            "receiver_type": "Rc<i32>",
            "result_type": "Rc<i32>"
          },
          {
            "method": "as_ref",
            "line": 10,
            "operation": null,
            "self_borrow": "immutable",
            "receiver_type": "Rc<i32>",
            "result_type": "&i32"
          }
        ]
      },
      {
        "name": "cell",
        "ty": "Cell<i32>",
        "is_cell": true,
        "initializer_kind": "cell_new",
        "source_location": { "line": 12, "column": 9 },
        "function_context": "main",
        "method_calls": [
          {
            "method": "set",
            "line": 15,
            "operation": "cell_set",
            "self_borrow": "immutable",
            "receiver_type": "Cell<i32>",
            "result_type": "()"
          },
          {
            "method": "get",
            "line": 16,
            "operation": "cell_get",
            "self_borrow": "immutable",
            "receiver_type": "Cell<i32>",
            "result_type": "i32"
          }
        ]
      }
    ]
  },
  "expressions": {
    "src/main.rs": [
      {
        "kind": "function_call",
        "line": 20,
        "operation": "thread_spawn",
        "path": "std::thread::spawn",
        "result_type": "JoinHandle<()>"
      },
      {
        "kind": "function_call",
        "line": 25,
        "operation": "transmute",
        "path": "std::mem::transmute",
        "from_type": "u32",
        "to_type": "i32"
      }
    ]
  },
  "by_name": { ... },
  "by_function": { ... }
}
```

#### New Data Structures

```rust
/// Information about a method call on a tracked variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCallInfo {
    /// Method name (e.g., "clone", "set", "send")
    pub method: String,
    
    /// Line number where the call occurs
    pub line: u32,
    
    /// Semantic operation classification (e.g., "cell_set", "channel_send")
    /// None if the method doesn't map to a tracked operation
    pub operation: Option<String>,
    
    /// How the method borrows self: "immutable", "mutable", or "consuming"
    pub self_borrow: Option<String>,
    
    /// Fully qualified receiver type
    pub receiver_type: String,
    
    /// Fully qualified result type (if any)
    pub result_type: Option<String>,
}

/// Information about a standalone expression requiring tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionInfo {
    /// Expression kind: "function_call", "method_call", "unsafe_block"
    pub kind: String,
    
    /// Line number where the expression occurs
    pub line: u32,
    
    /// Semantic operation classification
    pub operation: String,
    
    /// Function path for function calls (e.g., "std::thread::spawn")
    pub path: Option<String>,
    
    /// Additional type information for specific operations
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// Updated variable type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTypeInfo {
    // ... existing fields ...
    
    /// Method calls made on this variable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_calls: Vec<MethodCallInfo>,
}

/// Updated project type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeInfo {
    // ... existing fields ...
    
    /// Standalone expressions requiring tracking, keyed by file path
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub expressions: HashMap<String, Vec<ExpressionInfo>>,
}
```

#### Backward Compatibility

The schema evolution maintains backward compatibility:

1. **New fields are optional**: `method_calls` and `expressions` default to empty collections
2. **Existing fields unchanged**: All v2.2 fields retain their meaning and format
3. **Version field updated**: Consumers can check version to determine available features
4. **Graceful degradation**: Macros using v2.2 data continue working; they simply don't benefit from new fields

### 9.5 Macro Refactoring

After the analyzer expansion, the macro requires significant refactoring to consume the new semantic data and eliminate syntactic detection.

#### Current Macro Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CURRENT MACRO DETECTION FLOW                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Input: AST Node                                                            │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────┐                                                    │
│  │ Check Analyzer Data │──── Found? ───▶ Use semantic classification       │
│  └─────────────────────┘                                                    │
│           │                                                                 │
│           │ Not found                                                       │
│           ▼                                                                 │
│  ┌─────────────────────┐                                                    │
│  │ Syntactic Detection │                                                    │
│  │ (109 patterns)      │                                                    │
│  └─────────────────────┘                                                    │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────┐                                                    │
│  │ Emit Tracking Call  │                                                    │
│  └─────────────────────┘                                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Target Macro Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TARGET MACRO DETECTION FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Input: AST Node                                                            │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────┐                                                    │
│  │ Query Analyzer Data │                                                    │
│  │ (type-info.json)    │                                                    │
│  └─────────────────────┘                                                    │
│           │                                                                 │
│           ├── Variable binding? ──▶ Look up by (function, name, occurrence) │
│           │                              │                                  │
│           │                              ▼                                  │
│           │                        Use initializer_kind                     │
│           │                                                                 │
│           ├── Method call? ──────▶ Look up receiver variable                │
│           │                              │                                  │
│           │                              ▼                                  │
│           │                        Find matching method_call by line        │
│           │                              │                                  │
│           │                              ▼                                  │
│           │                        Use operation + self_borrow              │
│           │                                                                 │
│           └── Standalone expr? ──▶ Look up in expressions by line           │
│                                          │                                  │
│                                          ▼                                  │
│                                    Use operation                            │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────┐                                                    │
│  │ Emit Tracking Call  │                                                    │
│  │ (semantic only)     │                                                    │
│  └─────────────────────┘                                                    │
│                                                                             │
│  NO SYNTACTIC FALLBACK - analyzer data is authoritative                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Files to Modify

**`borrowscope-macro/src/smart_pointer.rs`**:
- Delete `detect_smart_pointer_new()` - replaced by `initializer_kind`
- Delete `detect_rc_clone()` - replaced by `initializer_kind`
- Delete `detect_refcell_borrow()` - replaced by `initializer_kind`
- Delete `detect_cell_op()` - replaced by `method_calls`
- Delete `detect_cow_creation()` - replaced by `initializer_kind`
- Delete `detect_cow_to_mut()` - replaced by `method_calls`
- Delete `detect_once_cell_new()` - replaced by `initializer_kind`
- Delete `detect_once_cell_method()` - replaced by `method_calls`
- Delete `detect_maybe_uninit_new()` - replaced by `initializer_kind`
- Delete `detect_maybe_uninit_method()` - replaced by `method_calls`
- Delete `detect_concurrency_op()` - replaced by `initializer_kind` + `method_calls` + `expressions`
- Delete `detect_pin_operation()` - replaced by `initializer_kind`
- Delete `detect_downgrade()` - replaced by `initializer_kind`
- Delete `detect_weak_upgrade()` - replaced by `initializer_kind`

**`borrowscope-macro/src/transform_visitor.rs`**:
- Delete `infer_self_borrow_type()` - replaced by `self_borrow` field
- Modify `visit_local_mut()` to query `initializer_kind` directly
- Modify `visit_expr_mut()` to query `method_calls` and `expressions`
- Remove all `method_name ==` comparisons
- Remove all `contains()` string matching

**`borrowscope-macro/src/type_info.rs`**:
- Add `MethodCallInfo` and `ExpressionInfo` structs
- Add lookup methods for method calls by line number
- Add lookup methods for expressions by line number
- Update `VariableTypeInfo` to include `method_calls`

#### Code Transformation Example

**Before (Syntactic)**:

```rust
// In transform_visitor.rs
fn transform_method_call(&mut self, expr: &ExprMethodCall) -> Option<Expr> {
    let method_name = expr.method.to_string();
    
    // Heuristic: check method name patterns
    if method_name == "clone" {
        return Some(self.emit_clone_tracking(expr));
    }
    
    if method_name == "set" {
        // Could be Cell::set, OnceCell::set, or something else
        // We guess based on context
        return Some(self.emit_cell_set_tracking(expr));
    }
    
    // Infer self borrow type from method name
    let borrow_type = infer_self_borrow_type(&method_name);
    self.emit_method_borrow_tracking(expr, borrow_type)
}
```

**After (Semantic)**:

```rust
// In transform_visitor.rs
fn transform_method_call(&mut self, expr: &ExprMethodCall) -> Option<Expr> {
    let line = expr.span().start().line as u32;
    
    // Look up receiver variable
    let receiver_name = extract_receiver_name(expr)?;
    let var_info = self.lookup_type_info(&receiver_name)?;
    
    // Find the method call info for this line
    let method_info = var_info.method_calls.iter()
        .find(|m| m.line == line)?;
    
    // Use semantic classification directly
    match method_info.operation.as_deref() {
        Some("cell_set") => self.emit_cell_set_tracking(expr),
        Some("channel_send") => self.emit_channel_send_tracking(expr),
        Some("rc_clone") => self.emit_rc_clone_tracking(expr),
        Some("clone") => self.emit_clone_tracking(expr),
        _ => {
            // Use semantic self_borrow instead of heuristic
            let borrow_type = match method_info.self_borrow.as_deref() {
                Some("immutable") => SelfBorrowType::Immutable,
                Some("mutable") => SelfBorrowType::Mutable,
                Some("consuming") => SelfBorrowType::Consuming,
                _ => SelfBorrowType::Unknown,
            };
            self.emit_method_borrow_tracking(expr, borrow_type)
        }
    }
}
```

#### Estimated Refactoring Effort

| Component | Changes | Estimate |
|-----------|---------|----------|
| `smart_pointer.rs` | Delete 14 detection functions | 1 hour |
| `transform_visitor.rs` | Rewrite method call handling | 2 hours |
| `type_info.rs` | Add new structs and lookups | 1 hour |
| Testing | Update tests for new behavior | 1 hour |
| **Total** | | **5 hours** |

### 9.6 Success Criteria

The expansion is complete when all of the following criteria are met:

#### Code Quality Metrics

1. **Zero syntactic detection in `smart_pointer.rs`**:
   ```bash
   # Should return 0 matches
   grep -c "contains\|starts_with\|ends_with\|== \"" borrowscope-macro/src/smart_pointer.rs
   ```

2. **Zero method name comparisons in `transform_visitor.rs`**:
   ```bash
   # Should return 0 matches
   grep -c "method_name ==" borrowscope-macro/src/transform_visitor.rs
   ```

3. **`infer_self_borrow_type()` function deleted**:
   ```bash
   # Should return 0 matches
   grep -c "fn infer_self_borrow_type" borrowscope-macro/src/transform_visitor.rs
   ```

4. **All 109 patterns marked as covered**:
   - Update `SYNTACTIC_PATTERNS.md` to show ✅ for all patterns
   - No ❌ or ⚠️ remaining

#### Functional Verification

5. **All tests pass**:
   ```bash
   cargo test -p borrowscope-analyzer
   cargo test -p borrowscope-macro
   ```

6. **Example projects produce identical output**:
   - Run all examples with and without analyzer data
   - Verify tracking events are semantically equivalent
   - No regression in tracking accuracy

7. **Type coverage example achieves 100%**:
   ```bash
   cargo run -p borrowscope-analyzer -- examples/type-coverage
   # Verify all variables have initializer_kind
   # Verify all method calls have operation or self_borrow
   # Verify all standalone expressions are captured
   ```

#### Documentation Updates

8. **Schema documentation updated** to v2.4
9. **README updated** with new capabilities
10. **SYNTACTIC_PATTERNS.md archived** (no longer needed)

### 9.7 Timeline and Milestones

| Phase | Patterns | Complexity | Estimate | Cumulative |
|-------|----------|------------|----------|------------|
| Phase 1: Method calls | 13 | Medium | 2-3 hours | 3 hours |
| Phase 2: Expressions | 4 | Medium | 1-2 hours | 5 hours |
| Phase 3: Self-borrow | 47 | High | 3-4 hours | 9 hours |
| Phase 4: Unwrap | 5 | Low | 1 hour | 10 hours |
| Phase 5: Clone | 1 | Low | 30 min | 10.5 hours |
| Testing & validation | - | Medium | 2 hours | 12.5 hours |
| Macro refactoring | - | High | 5 hours | 17.5 hours |
| Documentation | - | Low | 1 hour | 18.5 hours |
| **Total** | **70** | | | **~19 hours** |

#### Milestone Checkpoints

**M1: Method Call Infrastructure** (Phase 1 complete)
- New `method_calls` field in output
- 13 patterns eliminated
- Macro consumes method call data

**M2: Expression Tracking** (Phase 2 complete)
- New `expressions` field in output
- 17 patterns eliminated (cumulative)
- `thread::spawn` and `transmute` tracked semantically

**M3: Self-Borrow Resolution** (Phase 3 complete)
- `self_borrow` field populated for all method calls
- 64 patterns eliminated (cumulative)
- `infer_self_borrow_type()` deleted from macro

**M4: Complete Semantic Coverage** (Phases 4-5 complete)
- 70 patterns eliminated (all non-initializer patterns)
- All success criteria met
- Ready for release

---

## 10. Dependencies

The borrowscope-analyzer relies on rust-analyzer's published crate ecosystem for semantic analysis. These crates are versioned together and should be kept in sync to avoid compatibility issues.

### rust-analyzer Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `ra_ap_hir` | 0.0.232 | High-level intermediate representation and semantic queries |
| `ra_ap_ide_db` | 0.0.232 | IDE database infrastructure and root database type |
| `ra_ap_load-cargo` | 0.0.232 | Cargo workspace loading and sysroot discovery |
| `ra_ap_project_model` | 0.0.232 | Project structure modeling and configuration |
| `ra_ap_syntax` | 0.0.232 | Syntax tree representation and AST types |
| `ra_ap_vfs` | 0.0.232 | Virtual file system for source file management |

These crates are published to crates.io with each rust-analyzer release. The version number `0.0.232` corresponds to a specific rust-analyzer release. All `ra_ap_*` crates must use the same version to ensure ABI compatibility.

**Version Pinning Rationale**: The `ra_ap_*` crates follow rust-analyzer's rapid release cycle and do not maintain semver compatibility between versions. Internal APIs change frequently as rust-analyzer evolves. Pinning to a specific version ensures reproducible builds and avoids unexpected breakage from API changes.

**Updating Dependencies**: When updating to a newer rust-analyzer version, all `ra_ap_*` crates must be updated together. API changes may require code modifications. The rust-analyzer changelog documents breaking changes between versions.

### Utility Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON serialization for output |
| `anyhow` | 1.0 | Error handling with context |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output formatting |

These utility crates follow semver and can be updated independently.

### Rust Version Requirements

The analyzer requires Rust 1.75 or later due to dependencies on recent language features used by rust-analyzer. The `rust-version` field in `Cargo.toml` enforces this minimum.

### Dependency Tree Considerations

The `ra_ap_*` crates bring a substantial dependency tree, including:

- `salsa` - Incremental computation framework
- `rowan` - Syntax tree library
- `chalk` - Trait solving
- `rustc_lexer` - Rust lexer (from rustc)
- Numerous utility crates

This results in a large dependency footprint (~300 crates) and extended initial compile times (~3-5 minutes for a clean build). Subsequent incremental builds are fast (~5-10 seconds).

### Compatibility Notes

The `ra_ap_*` crates are designed for use within rust-analyzer and may have rough edges when used as a library:

- Some APIs assume IDE usage patterns and may be awkward for batch processing
- Error handling sometimes uses panics rather than Results
- Documentation is sparse; reading rust-analyzer source code is often necessary
- Breaking changes occur without deprecation warnings

Despite these challenges, the `ra_ap_*` crates provide the most complete and correct Rust semantic analysis available outside of rustc itself. The alternative—reimplementing type resolution—would be a multi-year effort with ongoing maintenance burden as Rust evolves.
