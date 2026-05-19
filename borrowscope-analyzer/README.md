# borrowscope-analyzer

> Static type analysis tool for Rust projects — powers the `#[trace_borrow]` macro with semantic type information

## Overview

`borrowscope-analyzer` performs deep semantic analysis of Rust projects using the same compiler infrastructure as rust-analyzer (`ra_ap_*` crates). It extracts comprehensive type information for every variable, expression, and operation in your code, then outputs a structured JSON file (`.borrowscope/type-info.json`) that the `borrowscope-macro` reads at compile time to emit accurate tracking calls.

**Key principle:** Zero heuristics. All analysis is fully semantic — types are resolved via the compiler's type system, not string matching.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    borrowscope-analyzer                       │
│                                                              │
│  ┌──────────────────┐    ┌────────────────────────────────┐ │
│  │  Workspace Loader │    │  Semantic Analysis Engine       │ │
│  │  (ra_ap_load_cargo)│    │                                │ │
│  │                    │    │  • KnownTypes (100+ ADTs)      │ │
│  │  • Cargo.toml     │    │  • KnownMacros (30+ macros)    │ │
│  │  • Dependencies   │    │  • TrackedFunctions (14 fns)   │ │
│  │  • Sysroot        │    │  • Variable analysis           │ │
│  └────────┬───────────┘    │  • Expression analysis         │ │
│           │                │  • Trait impl detection        │ │
│           ▼                │  • Closure capture analysis    │ │
│  ┌──────────────────┐    │  • Unsafe operation tracking   │ │
│  │  RootDatabase     │    │  • Borrow span analysis        │ │
│  │  (Salsa DB)       │────▶│  • Method call resolution      │ │
│  │                    │    │  • Lifetime extraction         │ │
│  │  • Type inference  │    └────────────────────────────────┘ │
│  │  • Name resolution │                    │                  │
│  │  • Trait solving   │                    ▼                  │
│  └──────────────────┘    ┌────────────────────────────────┐ │
│                           │  Output: type-info.json (v2.1)  │ │
│                           │                                  │ │
│                           │  • Per-file variable info        │ │
│                           │  • Expression classifications    │ │
│                           │  • Trait implementations         │ │
│                           │  • Closure captures              │ │
│                           │  • Unsafe operations             │ │
│                           └────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Usage

```bash
# Analyze a Rust project
cargo run -p borrowscope-analyzer -- /path/to/your/project

# Output is written to:
# /path/to/your/project/.borrowscope/type-info.json
```

The generated `type-info.json` is then read by `#[trace_borrow]` at compile time to determine which tracking calls to emit for each variable.

## What It Analyzes

### Per-Variable Information (~50 fields)

| Category | Fields |
|----------|--------|
| **Identity** | name, file, line, column, span_start/end, function_name, decl_index |
| **Type** | ty (display string), type_arguments, layout (size/align) |
| **Ownership** | is_copy, is_clone, is_drop, copy_semantics, binding_mode |
| **Smart Pointers** | is_rc, is_arc, is_box, is_weak, is_pin, is_cow |
| **Interior Mutability** | is_cell, is_refcell, is_mutex, is_rwlock, is_once_cell, is_guard |
| **References** | is_reference, is_mutable_reference, is_raw_ptr, contains_reference |
| **Collections** | is_vec, is_string, is_channel, is_atomic |
| **Traits** | is_send, is_sync, is_sized, is_future, is_iterator, is_callable |
| **Async** | is_future, future_output_type, is_impl_trait |
| **Patterns** | is_mut_binding, is_ref_binding, is_tuple_binding, pattern_adjustments |
| **Lifecycle** | drop_line, drop_column, scope_id, lifetime |
| **Initializer** | initializer_kind (semantic classification) |
| **Usages** | usages[] (line, column, kind: read/write/read_write) |
| **Methods** | method_calls[] (method, operation, self_borrow, trait_name) |
| **Closures** | closure_captures[] (name, capture_kind, type) |
| **Adjustments** | adjustments[] (kind: deref/borrow_shared/borrow_mut/pointer_cast) |

### Initializer Classification (100+ categories)

The analyzer semantically classifies every variable initializer:

| Category | Examples |
|----------|----------|
| `rc_new` | `Rc::new(x)` |
| `arc_clone` | `arc.clone()` |
| `refcell_borrow_mut` | `cell.borrow_mut()` |
| `mutex_lock` | `mutex.lock().unwrap()` |
| `vec_macro` | `vec![1, 2, 3]` |
| `string_new` | `String::from("hello")` |
| `channel_new` | `mpsc::channel()` |
| `weak_upgrade` | `weak.upgrade()` |
| `closure` | `\|x\| x + 1` |
| `user_struct` | `MyStruct { field: value }` |

### Expression Tracking

Standalone expressions that affect ownership:

| Function | Classification |
|----------|---------------|
| `std::mem::drop(x)` | Explicit drop |
| `std::mem::forget(x)` | Leak (no destructor) |
| `std::mem::replace(&mut x, y)` | Swap ownership |
| `std::mem::transmute(x)` | Unsafe type cast |
| `std::thread::spawn(\|\| {})` | Thread with captures |
| `std::ptr::read(p)` | Unsafe read |
| `std::ptr::write(p, v)` | Unsafe write |

### Trait Implementation Detection (40+ traits)

For every type encountered, the analyzer checks:

- **Access:** Deref, DerefMut, Index, IndexMut
- **Conversion:** From, Into, AsRef, AsMut, Borrow, BorrowMut, ToOwned
- **Comparison:** PartialEq, Eq, PartialOrd, Ord
- **Arithmetic:** Add, Sub, Mul, Div, Rem, Neg + Assign variants
- **Bitwise:** BitAnd, BitOr, BitXor, Shl, Shr, Not + Assign variants
- **Other:** RangeBounds, Termination, UnwindSafe, RefUnwindSafe

### Additional Analysis

| Feature | Description |
|---------|-------------|
| **Await points** | Type of awaited future + result type |
| **Unsafe operations** | Raw pointer derefs, unsafe fn calls, inside_unsafe_block |
| **Borrow spans** | Start/end of borrow lifetimes via Definition::usages() |
| **Destructuring** | Tuple/struct pattern bindings with types |
| **Match bindings** | Pattern variables in match/if-let/while-let arms |
| **Field accesses** | Partial borrow tracking (e.g., `self.field`) |
| **Closure traits** | Fn/FnMut/FnOnce classification per closure |
| **Enum variants** | Variant construction with field types |
| **Lifetimes** | Explicit lifetime parameters |
| **Labels** | Loop labels for break/continue |
| **Const patterns** | Constants used in pattern matching |
| **Record fields** | Named field expressions and patterns |

## Type Resolution Strategy

### Phase 1: Lang Items (fully semantic, zero string matching)
Types that are Rust lang items are resolved via `lang_items()`:
- `Box`, `UnsafeCell`, `Pin`, `Option`, `String`, `ManuallyDrop`, `MaybeUninit`, `PhantomData`, `Poll`, `Context`, ranges, `CStr`, `Layout`

### Phase 2: Import Map (semantic with module path verification)
Types without lang items are found via `import_map::Query`:
- `Rc`, `Arc`, `Weak`, `Cell`, `RefCell`, `Mutex`, `RwLock`, guards, `Vec`, `HashMap`, channels, `PathBuf`, atomics, etc.

### Phase 3: Trait Lookup (semantic)
40+ traits resolved via import map with module path filtering.

### Fallback: (name, crate) Map
For re-exported types where ADT identity differs from the type resolver's ADT, a `(TypeName, CrateName)` map provides classification.

## Output Format (type-info.json v3.0)

```json
{
  "version": "3.0",
  "analyzer_version": "0.1.0",
  "files": {
    "src/main.rs": [
      {
        "name": "data",
        "ty": "Vec<i32>",
        "file": "src/main.rs",
        "line": 5,
        "column": 8,
        "span_start": 42,
        "span_end": 46,
        "function_name": "main",
        "decl_index": 0,
        "is_vec": true,
        "is_copy": false,
        "is_clone": true,
        "is_drop": true,
        "is_send": true,
        "is_sync": true,
        "is_sized": true,
        "initializer_kind": "vec_macro",
        "is_mut_binding": true,
        "drop_line": 20,
        "drop_column": 1,
        "scope_id": 3,
        "layout": { "size": 24, "align": 8 },
        "type_arguments": ["i32"],
        "method_calls": [
          { "method": "push", "line": 8, "column": 4, "operation": "vec_push", "self_borrow": "mutable", "receiver_type": "Vec<i32>", "result_type": "()" }
        ],
        "usages": [
          { "line": 8, "column": 4, "kind": "write" },
          { "line": 12, "column": 14, "kind": "read" }
        ]
      }
    ]
  },
  "expressions": { "src/main.rs": [...] },
  "await_points": { "src/async.rs": [...] },
  "unsafe_operations": { "src/ffi.rs": [...] },
  "borrow_spans": { "src/main.rs": [...] },
  "destructuring": { "src/main.rs": [...] },
  "match_bindings": { "src/main.rs": [...] },
  "field_accesses": { "src/main.rs": [...] },
  "method_borrows": { "src/main.rs": [...] },
  "function_calls": { "src/main.rs": [...] },
  "trait_impls": { "Vec<i32>": { "implements_deref": true, "implements_index": true, ... } },
  "closure_traits": { "src/main.rs": [...] },
  "variants": { "src/main.rs": [...] },
  "lifetimes": { "src/lib.rs": [...] },
  "labels": { "src/main.rs": [...] },
  "const_patterns": { "src/main.rs": [...] },
  "callables": { "src/main.rs": [...] },
  "record_field_exprs": { "src/main.rs": [...] },
  "record_field_pats": { "src/main.rs": [...] },
  "by_name": { "data": [...] },
  "by_function": { "main": { "data": [...] } }
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ra_ap_hir` | High-level semantic API (types, traits, functions) |
| `ra_ap_hir_def` | Definition-level data (lang items) |
| `ra_ap_hir_ty` | Type inference engine |
| `ra_ap_ide_db` | IDE database (definitions, usages, references) |
| `ra_ap_load_cargo` | Workspace loading from Cargo.toml |
| `ra_ap_project_model` | Cargo project model |
| `ra_ap_syntax` | Syntax tree (AST) |
| `ra_ap_vfs` | Virtual file system |

All `ra_ap_*` crates are pinned to version `0.0.318` for compatibility.

## Code Structure

```
src/
├── main.rs        (91 lines)   Entry point, CLI argument parsing
├── analysis.rs    (4615 lines) Core semantic analysis engine
│   ├── KnownTypes              100+ ADT lookups via lang items + import map
│   ├── KnownMacros             30+ macro lookups
│   ├── TrackedFunctions        14 ownership-relevant functions
│   ├── analyze_project()       Top-level project analysis
│   ├── analyze_file()          Per-file analysis orchestrator
│   ├── analyze_let_stmt()      Variable declaration analysis
│   ├── populate_type_info()    Type → 50+ boolean flags
│   ├── classify_initializer()  Semantic initializer classification
│   ├── analyze_method_calls()  Method resolution + self borrow
│   ├── analyze_expressions()   Standalone expression tracking
│   ├── collect_trait_impls()   40+ trait implementation checks
│   └── ...                     20+ more analysis functions
└── output.rs      (858 lines)  Serializable output structures
    ├── ProjectTypeInfo         Top-level output container
    ├── VariableTypeInfo        Per-variable information (~50 fields)
    ├── ExpressionInfo          Tracked expression data
    ├── TraitImplInfo           40+ trait implementation flags
    └── ...                     15+ more output structures
```

**Total: ~5,500 lines of pure semantic analysis code**

## How It Integrates

```
1. User runs:  cargo run -p borrowscope-analyzer -- ./my-project
2. Analyzer:   Loads workspace → resolves all types → writes type-info.json
3. User builds: cargo build
4. Macro:      #[trace_borrow] reads type-info.json → emits correct track_*() calls
5. Runtime:    Program runs with tracking → exports events.json
6. VS Code:    Extension reads events.json → shows runtime overlay
```

## Performance

- Workspace loading: 5-15s (depends on dependency count)
- Per-file analysis: 50-200ms
- Total for medium project (~50 files): 15-30s
- Output JSON size: 100KB-2MB (depends on project size)

## License

Apache-2.0
