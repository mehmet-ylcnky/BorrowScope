# BorrowScope Macro Integration Implementation Plan

This document outlines the implementation phases for integrating `borrowscope-analyzer` type information with `borrowscope-macro`.

> **Schema Version**: This plan is based on type-info.json **v2.0** schema, which uses fully semantic type analysis (no string heuristics). See `borrowscope-analyzer/README.md` for schema details.

## Overview

The goal is to enable `borrowscope-macro` to consume the type information produced by `borrowscope-analyzer`, replacing heuristic-based type detection with accurate semantic type information.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         INTEGRATION ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Phase 1: Analysis                    Phase 2: Compilation                  │
│  ════════════════════                 ════════════════════                  │
│                                                                             │
│  ┌──────────────────┐                 ┌──────────────────┐                  │
│  │ borrowscope-     │                 │ borrowscope-     │                  │
│  │ analyzer         │                 │ macro            │                  │
│  └────────┬─────────┘                 └────────┬─────────┘                  │
│           │                                    │                            │
│           ▼                                    │                            │
│  ┌──────────────────┐                          │                            │
│  │ .borrowscope/    │◄─────────────────────────┘                            │
│  │ type-info.json   │     reads at expansion time                           │
│  └──────────────────┘                                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Type Info Loading in borrowscope-macro

### Objective

Enable `borrowscope-macro` to read and cache type information from `.borrowscope/type-info.json` at macro expansion time.

### Files to Create/Modify

```
borrowscope-macro/
├── Cargo.toml              # Add serde, serde_json dependencies
└── src/
    ├── lib.rs              # Add mod type_info
    ├── type_info.rs        # NEW: Type info loading and lookup
    └── transform_visitor.rs # Modify to use type_info lookup
```

### 1.1 Add Dependencies

Update `borrowscope-macro/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 1.2 Create type_info.rs Module

```rust
//! Type information loading and lookup for borrowscope-macro
//!
//! This module loads type-info.json (v2.0) produced by borrowscope-analyzer
//! and provides O(1) lookup by source location.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached type information - loaded once per compilation
static TYPE_INFO_CACHE: OnceLock<Option<TypeInfoCache>> = OnceLock::new();

/// Deserialized variable type info (v2.0 schema - semantic analysis)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct VariableTypeInfo {
    pub name: String,
    pub ty: String,
    
    // === Trait implementations (semantic via impls_trait) ===
    pub is_copy: bool,
    pub is_clone: bool,
    pub is_send: bool,
    pub is_sync: bool,
    pub is_drop: bool,
    pub is_sized: bool,
    pub is_future: bool,
    pub is_iterator: bool,
    
    // === Type structure (semantic via Type methods) ===
    pub is_primitive: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_raw_ptr: bool,
    pub is_slice: bool,
    pub is_str: bool,
    pub is_closure: bool,
    pub is_fn_ptr: bool,
    pub is_dyn_trait: bool,
    pub is_union: bool,
    
    // === ADT classification (semantic via canonical path) ===
    pub is_rc: bool,
    pub is_arc: bool,
    pub is_box: bool,
    pub is_weak: bool,
    pub is_refcell: bool,
    pub is_cell: bool,
    pub is_mutex: bool,
    pub is_rwlock: bool,
    pub is_guard: bool,
    pub is_vec: bool,
    pub is_string: bool,
    pub is_option: bool,
    pub is_result: bool,
    pub is_pin: bool,
    pub is_cow: bool,
    pub is_extern_type: bool,
    
    // === Declaration type ===
    pub is_static: bool,
    pub is_const: bool,
    
    // === Binding patterns ===
    pub is_tuple_binding: bool,
    pub is_mut_binding: bool,
    pub is_impl_trait: bool,
    
    // === Source location ===
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Deserialize)]
struct ProjectTypeInfo {
    version: String,
    files: HashMap<String, Vec<VariableTypeInfo>>,
}

/// Lookup key: (file, line, column)
type LocationKey = (String, u32, u32);

/// Cached type info with O(1) lookup
pub struct TypeInfoCache {
    by_location: HashMap<LocationKey, VariableTypeInfo>,
    project_root: PathBuf,
}

impl TypeInfoCache {
    /// Load type info from .borrowscope/type-info.json
    fn load(project_root: &Path) -> Option<Self> {
        let json_path = project_root.join(".borrowscope/type-info.json");
        
        if !json_path.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&json_path).ok()?;
        let info: ProjectTypeInfo = serde_json::from_str(&content).ok()?;
        
        // Build location index
        let mut by_location = HashMap::new();
        for (file, variables) in info.files {
            for var in variables {
                let key = (file.clone(), var.line, var.column);
                by_location.insert(key, var);
            }
        }
        
        Some(Self {
            by_location,
            project_root: project_root.to_path_buf(),
        })
    }
    
    /// Lookup type info by source location
    pub fn lookup(&self, file: &str, line: u32, column: u32) -> Option<&VariableTypeInfo> {
        // Normalize file path to relative
        let relative = self.normalize_path(file);
        self.by_location.get(&(relative, line, column))
    }
    
    fn normalize_path(&self, file: &str) -> String {
        // Convert absolute path to relative if needed
        let path = Path::new(file);
        if path.is_absolute() {
            path.strip_prefix(&self.project_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file.to_string())
        } else {
            file.to_string()
        }
    }
}

/// Find project root by walking up to find Cargo.toml
fn find_project_root() -> Option<PathBuf> {
    // Try CARGO_MANIFEST_DIR first (set during cargo build)
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return Some(PathBuf::from(dir));
    }
    
    // Fallback: walk up from current dir
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Get or initialize the type info cache
pub fn get_type_info() -> Option<&'static TypeInfoCache> {
    TYPE_INFO_CACHE.get_or_init(|| {
        let project_root = find_project_root()?;
        TypeInfoCache::load(&project_root)
    }).as_ref()
}

/// Lookup type info for a variable at the given location
pub fn lookup_type(file: &str, line: u32, column: u32) -> Option<&'static VariableTypeInfo> {
    get_type_info()?.lookup(file, line, column)
}
```

### 1.3 Modify transform_visitor.rs

Update the `visit_local_mut` function to use type info lookup:

```rust
// At the top of the file
use crate::type_info::{lookup_type, VariableTypeInfo};

// In visit_local_mut, before the current smart pointer detection:
fn visit_local_mut(&mut self, local: &mut Local) {
    // ... existing code to get var_name, location, etc ...
    
    // NEW: Try type info lookup first
    if let Some(type_info) = self.lookup_type_info(local) {
        if let Some(new_expr) = self.transform_with_type_info(local, type_info) {
            *init.expr = new_expr;
            visit_mut::visit_local_mut(self, local);
            return;
        }
    }
    
    // EXISTING: Fallback to heuristic detection
    if self.config.track_smart_pointers {
        if let Some(sp_type) = detect_smart_pointer_new(original_expr) {
            // ... existing heuristic code ...
        }
    }
}

/// Lookup type info for a local binding
fn lookup_type_info(&self, local: &Local) -> Option<&'static VariableTypeInfo> {
    let span = local.pat.span();
    
    // Get file path from span
    let file = span.source_file().path().to_string_lossy().to_string();
    let line = span.start().line as u32;
    let column = span.start().column as u32;
    
    lookup_type(&file, line, column)
}

/// Transform using semantic type information (v2.0 schema)
fn transform_with_type_info(
    &mut self,
    local: &mut Local,
    type_info: &VariableTypeInfo,
) -> Option<syn::Expr> {
    let var_name = /* extract from local */;
    let var_id = /* generate id */;
    let location = /* format location */;
    let original_expr = &local.init.as_ref()?.expr;
    let ty_str = &type_info.ty;
    
    // Use semantic type flags for accurate tracking
    
    // Smart pointers (ADT classification)
    if type_info.is_rc {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_rc_new_with_id(
                #var_id, #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    if type_info.is_arc {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_arc_new_with_id(
                #var_id, #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    if type_info.is_box {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_box_new_with_id(
                #var_id, #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    // Interior mutability (ADT classification)
    if type_info.is_refcell {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_refcell_new_with_id(
                #var_id, #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    if type_info.is_cell {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_cell_new_with_id(
                #var_id, #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    // Guards (ADT classification)
    if type_info.is_guard {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_guard_enter(
                #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    // References (Type structure)
    if type_info.is_mutable_reference {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_borrow_mut_with_id(
                #var_id, #var_name, #location, #original_expr
            )
        });
    }
    
    if type_info.is_reference {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_borrow_with_id(
                #var_id, #var_name, #location, #original_expr
            )
        });
    }
    
    // Raw pointers (Type structure)
    if type_info.is_raw_ptr {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_raw_ptr_create_with_id(
                #var_id, #var_name, #location, #original_expr
            )
        });
    }
    
    // Futures (Trait implementation)
    if type_info.is_future {
        return Some(syn::parse_quote! {
            borrowscope_runtime::track_future_create(
                #var_name, #ty_str, #location, #original_expr
            )
        });
    }
    
    // Default: use generic tracking with is_copy for move/copy semantics
    let is_copy = type_info.is_copy;
    Some(syn::parse_quote! {
        borrowscope_runtime::__track_new_with_id_helper(
            #var_id, #var_name, #location, #is_copy, #original_expr
        )
    })
}
```

### 1.4 Challenges and Solutions

| Challenge | Solution |
|-----------|----------|
| Span to file path | Use `span.source_file().path()` (requires `proc_macro` feature) |
| Line/column from Span | Use `span.start().line` and `span.start().column` |
| Path normalization | Strip project root prefix to get relative path |
| Cache invalidation | No runtime invalidation; users re-run analyzer when code changes |
| Missing type info | Fallback to existing heuristic detection |

### 1.5 Testing Strategy

1. Create test project with `.borrowscope/type-info.json`
2. Verify macro reads and uses type info
3. Verify fallback works when type info missing
4. Verify path normalization handles absolute/relative paths

---

## Phase 2: Enhanced Tracking with Semantic Information

### Objective

Leverage the new semantic type information (v2.0) to provide richer tracking capabilities.

### New Capabilities from Semantic Analysis

The v2.0 schema provides trait implementation data that enables new tracking features:

| Semantic Field | Tracking Enhancement |
|----------------|---------------------|
| `is_send` | Track thread-safety violations at runtime |
| `is_sync` | Detect unsafe shared access patterns |
| `is_drop` | Track custom destructor calls |
| `is_clone` | Distinguish clone vs move operations |
| `is_copy` | Accurate copy vs move semantics |
| `is_dyn_trait` | Track trait object usage |
| `is_slice` | Track slice operations (now detects `&[T]`, `Box<[T]>`) |

### New Tracking Functions to Add

```rust
// Thread-safety tracking (using is_send/is_sync)
pub fn track_send_violation(name: &str, ty: &str, location: &str);
pub fn track_sync_violation(name: &str, ty: &str, location: &str);

// Clone tracking (using is_clone)
pub fn track_clone<T: Clone>(name: &str, ty: &str, location: &str, value: &T) -> T;

// Drop tracking (using is_drop)  
pub fn track_drop_with_destructor(name: &str, ty: &str, location: &str);

// Guards - track borrow scope entry/exit
pub fn track_guard_enter<T>(name: &str, ty: &str, location: &str, guard: T) -> T;
pub fn track_guard_exit(name: &str);

// Weak references
pub fn track_weak_new<T>(name: &str, weak: Weak<T>) -> Weak<T>;
pub fn track_weak_upgrade<T>(name: &str, result: Option<Rc<T>>) -> Option<Rc<T>>;

// Futures (using is_future trait detection)
pub fn track_future_create<F: Future>(name: &str, ty: &str, future: F) -> impl Future;
pub fn track_future_poll(name: &str, state: &str);

// Iterators (using is_iterator trait detection)
pub fn track_iterator_create<I: Iterator>(name: &str, ty: &str, iter: I) -> impl Iterator;
pub fn track_iterator_next<T>(name: &str, item: Option<T>) -> Option<T>;

// Trait objects (using is_dyn_trait)
pub fn track_dyn_trait_create(name: &str, ty: &str, location: &str);
```

### Event Types to Add

```rust
pub enum EventType {
    // Existing...
    
    // New semantic-based events
    Clone,              // is_clone: true
    DropWithDestructor, // is_drop: true
    SendViolation,      // is_send: false but crossed thread
    SyncViolation,      // is_sync: false but shared
    GuardEnter,
    GuardExit,
    WeakNew,
    WeakUpgrade,
    WeakUpgradeFailed,
    FutureCreate,
    FuturePoll,
    IteratorCreate,
    IteratorNext,
    IteratorExhausted,
    DynTraitCreate,
}
```

### Runtime Event Enhancement

With semantic type info, events become more informative:

```json
{
  "event": "new",
  "name": "data",
  "type": "Rc<RefCell<Vec<i32, Global>>, Global>",
  "is_copy": false,
  "is_send": false,
  "is_sync": false,
  "is_drop": true,
  "location": "src/main.rs:15:8"
}
```

---

## Phase 3: CLI Improvements for borrowscope-analyzer

### Objective

Improve analyzer usability with CLI options.

### New CLI Options

```
borrowscope-analyzer [OPTIONS] <PROJECT_PATH>

OPTIONS:
    -o, --output <PATH>     Output path (default: .borrowscope/type-info.json)
    -w, --watch             Watch mode - re-analyze on file changes
    -q, --quiet             Suppress progress output
    -v, --verbose           Verbose output (debug logging)
    --format <FORMAT>       Output format: json (default), compact
    --include <PATTERN>     Only analyze files matching pattern
    --exclude <PATTERN>     Exclude files matching pattern
```

### Implementation

Add `clap` dependency and argument parsing in `main.rs`.

---

## Phase 4: Testing and Validation

### Unit Tests

- `type_info.rs`: Test loading, lookup, path normalization
- `transform_visitor.rs`: Test type-info-based transformation

### Integration Tests

- End-to-end: analyzer → macro → runtime
- Verify correct tracking functions selected
- Verify fallback behavior

### Test Projects

- `examples/type-coverage/`: Comprehensive type coverage
- `examples/integration-test/`: Macro + analyzer integration

---

## Phase 5: Documentation Updates

### Files to Update

1. `README.md` (root): Add analyzer to workflow
2. `borrowscope-macro/README.md`: Document type info integration
3. `borrowscope-analyzer/README.md`: Already comprehensive

### User Workflow Documentation

```bash
# Step 1: Analyze project
borrowscope-analyzer .

# Step 2: Build with instrumentation
cargo build

# Step 3: Run and collect events
cargo run
```

---

## Implementation Priority

| Phase | Priority | Effort | Dependencies |
|-------|----------|--------|--------------|
| Phase 1 | High | Medium | None |
| Phase 2 | Medium | Medium | Phase 1 |
| Phase 3 | Low | Low | None |
| Phase 4 | High | Medium | Phase 1, 2 |
| Phase 5 | Medium | Low | Phase 1 |

Recommended order: Phase 1 → Phase 4 (testing) → Phase 2 → Phase 5 → Phase 3

---

## Open Questions

1. **Span stability**: Do line/column numbers from `proc_macro::Span` match rust-analyzer's?
2. **Workspace support**: How to handle multi-crate workspaces?
3. **Incremental builds**: Should macro check if type-info.json is stale?
4. **Error handling**: How to report when type info lookup fails?
