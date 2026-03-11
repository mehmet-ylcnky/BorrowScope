# Design Document: Make Analyzer Mandatory for Macro

**Status:** Draft  
**Created:** 2026-03-11  
**Goal:** Remove all heuristics from macro, make it 100% dependent on analyzer semantic data

---

## Executive Summary

Transform `borrowscope-macro` from a hybrid (semantic + heuristic fallback) system to a **fully semantic** system that requires `borrowscope-analyzer` as a mandatory preprocessing step. This eliminates all string-matching heuristics and ensures ownership tracking is based entirely on rust-analyzer's type system API.

**Key Principle:** The macro should be a "dumb transformer" that only reads and applies decisions made by the analyzer. All intelligence lives in the analyzer.

---

## Motivation

### Current Problems
1. **Dual code paths:** Semantic (analyzer-based) + heuristic (string-matching) fallbacks
2. **Incomplete semantic usage:** Macro has `is_copy`, `is_primitive` but doesn't use them
3. **Maintenance burden:** Every new pattern needs both semantic + heuristic implementations
4. **Accuracy gaps:** Heuristics can misclassify (e.g., treating Copy as move)

### Benefits of Analyzer-Only Approach
1. **Single source of truth:** rust-analyzer API for all type decisions
2. **Zero heuristics:** No string matching, no guessing
3. **Correctness:** Semantic analysis catches edge cases heuristics miss
4. **Maintainability:** Changes only in analyzer, macro just applies them
5. **Testability:** Analyzer output is deterministic and inspectable

---

## Architecture Overview

### Before (Current)
```
User Code
    ↓
#[trace_borrow] macro
    ↓
├─→ Try lookup_type_info() [semantic]
│   └─→ If None → detect_* functions [heuristic]
└─→ Generate tracking calls
```

### After (Proposed)
```
User Code
    ↓
borrowscope-analyzer (REQUIRED)
    ↓
.borrowscope/type-info.json (complete coverage)
    ↓
#[trace_borrow] macro (semantic only)
    ↓
├─→ lookup_type_info() [MUST succeed]
└─→ Generate tracking calls
```

---

## Implementation Plan

### Progress Tracker

#### Phase 1: Analyzer Enhancements (2-3 days) ✅ COMPLETE
- [x] **Step 1.1:** Add `copy_semantics` field (commit: af9ed8e3d)
- [x] **Step 1.2:** Add `method_borrows` array (commit: 497714823)
- [x] **Step 1.3:** Add `field_accesses` array (commit: 8ee8e282d - verified)
- [x] **Step 1.4:** Add `function_calls` array (commit: 2b7f80c58 - partial, 356de2c4a - traits added)
- [x] **Step 1.5:** Enhance `closure_captures` with location data (commit: 615c0df4d)

**Note:** Step 1.4 extended to include comprehensive trait tracking:
- Added 41 traits to KnownTypes (Deref, Index, From, Into, comparison, arithmetic, bitwise, etc.)
- Added TraitImplInfo struct with 41 boolean fields
- Added collect_trait_impls() function using Type::impls_trait() API
- 100% semantic, zero heuristics

**Phase 1 Summary:**
- All analyzer enhancements complete
- Ready for Phase 2: Macro Simplification

#### Phase 2: Macro Simplification (2-3 days)
- [x] **Step 2.1:** Delete `smart_pointer.rs` (commits: 15c7efe45, 3a06efb50, 5e1f01905, 8c2c7294d, 2639134df)
- [x] **Step 2.2:** Remove `infer_self_borrow_type_heuristic()` (commit: d33742006, verified: c98381af4)
- [x] **Step 2.3:** Make analyzer mandatory (commit: 36ed6834d)
- [ ] **Step 2.4:** Use `copy_semantics` for Copy vs Move
- [ ] **Step 2.5:** Skip drop tracking for Copy types
- [ ] **Step 2.6:** Remove smart pointer fallback
- [ ] **Step 2.7:** Update method call transformation
- [ ] **Step 2.8:** Remove heuristic sets

#### Phase 3: Schema Updates (1 day)
- [ ] **Step 3.1:** Update `type_info.rs` structs

#### Phase 4: Testing (2-3 days)
- [ ] **Step 4.1:** Generate fixtures for 567 tests
- [ ] **Step 4.2:** Add integration tests

#### Phase 5: Documentation (1 day)
- [ ] **Step 5.1:** Update README.md
- [ ] **Step 5.2:** Update borrowscope-macro README
- [ ] **Step 5.3:** Update SEMANTIC_IMPLEMENTATION.md

#### Phase 6: Migration Guide (1 day)
- [ ] **Step 6.1:** Create MIGRATION.md

**Current Status:** Phase 2, Step 2.3 complete (8/21 steps, 38.1%)

---

### Phase 1: Analyzer Enhancements (Expand Coverage)

#### Step 1.1: Add `copy_semantics` field to analyzer output
**File:** `borrowscope-analyzer/src/output.rs`

**Changes:**
```rust
// Line ~410, add to VariableInfo struct:
#[serde(default)]
pub copy_semantics: bool,  // true if assignment is copy, not move
```

**File:** `borrowscope-analyzer/src/analysis.rs`

**Changes:**
```rust
// Line ~1800, in analyze_variable_usage(), after is_copy detection:
var_info.copy_semantics = var_info.is_copy || var_info.is_primitive;

// This tells macro: "assignment from this variable is a copy, not a move"
```

**Rationale:** Macro needs explicit signal to distinguish `let y = x` (copy) vs move.

---

#### Step 1.2: Add `method_borrow_info` to track ALL method calls
**File:** `borrowscope-analyzer/src/output.rs`

**Changes:**
```rust
// Line ~200, add new struct:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodBorrowInfo {
    pub method_name: String,
    pub receiver_var: String,      // Variable method is called on
    pub borrow_kind: String,       // "none", "shared_ref", "mutable_ref"
    pub line: u32,
    pub column: u32,
}

// Line ~100, add to AnalysisOutput:
#[serde(default)]
pub method_borrows: Vec<MethodBorrowInfo>,
```

**File:** `borrowscope-analyzer/src/analysis.rs`

**Changes:**
```rust
// Line ~1500, add new function:
fn analyze_method_calls(
    &mut self,
    func: &hir::Function,
    body: &hir::Body,
    source_map: &SourceMap,
) {
    let def_map = func.module(self.db).def_map(self.db);
    
    for (expr_id, expr) in body.exprs.iter() {
        if let hir::Expr::MethodCall { receiver, method_name, .. } = expr {
            // Get receiver variable name
            let receiver_var = self.expr_to_var_name(body, *receiver);
            
            // Get method function
            let method_fn = match self.infer[expr_id].as_callable(self.db) {
                Some((hir::CallableDefId::FunctionId(fn_id), _)) => fn_id,
                _ => continue,
            };
            
            // Check self parameter type
            let fn_data = self.db.function_data(method_fn);
            let self_param = fn_data.params.first()?;
            let self_ty = &body.params[*self_param];
            
            let borrow_kind = if self_ty.is_mutable_reference() {
                "mutable_ref"
            } else if self_ty.is_reference() {
                "shared_ref"
            } else {
                "none"  // by-value or smart pointer
            };
            
            let location = self.get_expr_location(source_map, expr_id);
            
            self.output.method_borrows.push(MethodBorrowInfo {
                method_name: method_name.to_string(),
                receiver_var,
                borrow_kind: borrow_kind.to_string(),
                line: location.line,
                column: location.column,
            });
        }
    }
}

// Line ~1200, call from analyze_function():
self.analyze_method_calls(&func, &body, &source_map);
```

**Rationale:** Eliminates `infer_self_borrow_type_heuristic()` — all method borrow info comes from analyzer.

---

#### Step 1.3: Add `field_access_info` for field borrows
**File:** `borrowscope-analyzer/src/output.rs`

**Changes:**
```rust
// Line ~220, add new struct:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessInfo {
    pub base_var: String,          // Variable being accessed
    pub field_path: Vec<String>,   // ["field1", "field2"] for nested
    pub access_kind: String,       // "read", "write", "borrow_shared", "borrow_mut"
    pub line: u32,
    pub column: u32,
}

// Line ~100, add to AnalysisOutput:
#[serde(default)]
pub field_accesses: Vec<FieldAccessInfo>,
```

**File:** `borrowscope-analyzer/src/analysis.rs`

**Changes:**
```rust
// Line ~1600, add new function:
fn analyze_field_accesses(
    &mut self,
    body: &hir::Body,
    source_map: &SourceMap,
) {
    for (expr_id, expr) in body.exprs.iter() {
        if let hir::Expr::Field { expr: base, name } = expr {
            let base_var = self.expr_to_var_name(body, *base);
            let field_name = name.to_string();
            
            // Determine access kind from parent context
            let access_kind = self.infer_field_access_kind(expr_id, body);
            
            let location = self.get_expr_location(source_map, expr_id);
            
            self.output.field_accesses.push(FieldAccessInfo {
                base_var,
                field_path: vec![field_name],
                access_kind,
                line: location.line,
                column: location.column,
            });
        }
    }
}

fn infer_field_access_kind(&self, expr_id: ExprId, body: &hir::Body) -> String {
    // Check if field access is in borrow context
    // This requires walking up the expression tree
    // For MVP: return "read" (can enhance later)
    "read".to_string()
}

// Line ~1200, call from analyze_function():
self.analyze_field_accesses(&body, &source_map);
```

**Rationale:** Replaces heuristic field access detection in macro.

---

#### Step 1.4: Add `function_call_info` for return type tracking
**File:** `borrowscope-analyzer/src/output.rs`

**Changes:**
```rust
// Line ~240, add new struct:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallInfo {
    pub function_name: String,
    pub return_category: String,   // "rc_new", "arc_clone", "option_some", etc.
    pub is_copy_return: bool,
    pub line: u32,
    pub column: u32,
}

// Line ~100, add to AnalysisOutput:
#[serde(default)]
pub function_calls: Vec<FunctionCallInfo>,
```

**File:** `borrowscope-analyzer/src/analysis.rs`

**Changes:**
```rust
// Line ~1700, add new function:
fn analyze_function_calls(
    &mut self,
    body: &hir::Body,
    source_map: &SourceMap,
) {
    for (expr_id, expr) in body.exprs.iter() {
        if let hir::Expr::Call { callee, .. } = expr {
            // Get function being called
            let fn_id = match self.infer[*callee].as_callable(self.db) {
                Some((hir::CallableDefId::FunctionId(id), _)) => id,
                _ => continue,
            };
            
            let fn_name = self.db.function_data(fn_id).name.to_string();
            
            // Get return type
            let ret_ty = self.infer[expr_id].clone();
            
            // Classify return type
            let return_category = self.classify_type(&ret_ty);
            let is_copy_return = ret_ty.is_copy(self.db);
            
            let location = self.get_expr_location(source_map, expr_id);
            
            self.output.function_calls.push(FunctionCallInfo {
                function_name: fn_name,
                return_category,
                is_copy_return,
                line: location.line,
                column: location.column,
            });
        }
    }
}

// Line ~1200, call from analyze_function():
self.analyze_function_calls(&body, &source_map);
```

**Rationale:** Macro can look up function call return types instead of guessing from name.

---

#### Step 1.5: Enhance `closure_captures` with precise location info
**File:** `borrowscope-analyzer/src/output.rs`

**Changes:**
```rust
// Line ~88, update ClosureCaptureInfo:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureCaptureInfo {
    pub var_name: String,
    pub capture_kind: String,  // "shared_ref", "mutable_ref", "move"
    pub line: u32,             // ADD: closure definition line
    pub column: u32,           // ADD: closure definition column
}
```

**File:** `borrowscope-analyzer/src/analysis.rs`

**Changes:**
```rust
// Line ~1850, in closure capture analysis, add location:
let location = self.get_expr_location(source_map, closure_expr_id);

var_info.closure_captures.push(ClosureCaptureInfo {
    var_name: captured_var.clone(),
    capture_kind: kind.to_string(),
    line: location.line,      // ADD
    column: location.column,  // ADD
});
```

**Rationale:** Macro needs location to match closure AST node to analyzer data.

---

### Phase 2: Macro Simplification (Remove Heuristics)

#### Step 2.1: Remove all `detect_*` functions
**File:** `borrowscope-macro/src/smart_pointer.rs`

**Action:** DELETE entire file (10 functions, ~400 lines)

Functions to remove:
- `detect_smart_pointer_new()`
- `detect_smart_pointer_clone()`
- `detect_refcell_borrow()`
- `detect_mutex_lock()`
- `detect_rwlock_lock()`
- `detect_arc_operations()`
- `detect_rc_operations()`
- `detect_box_operations()`
- `detect_concurrency_op()`
- `detect_unsafe_op()`

**File:** `borrowscope-macro/src/lib.rs`

**Changes:**
```rust
// Line ~10, remove:
mod smart_pointer;  // DELETE this line
```

---

#### Step 2.2: Remove `infer_self_borrow_type_heuristic()`
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Action:** DELETE function at lines ~2417-2500

**Replace with:**
```rust
// Line ~2417, replace entire function:
fn lookup_method_borrow_info(
    &self,
    receiver_name: &str,
    method_name: &str,
    line: u32,
) -> Option<String> {
    // Load method_borrows from type_info.json
    let type_info_data = self.type_info_data.as_ref()?;
    
    type_info_data
        .method_borrows
        .iter()
        .find(|mb| {
            mb.receiver_var == receiver_name
                && mb.method_name == method_name
                && mb.line == line
        })
        .map(|mb| mb.borrow_kind.clone())
}
```

**Rationale:** All method borrow info now comes from analyzer's `method_borrows` array.

---

#### Step 2.3: Make analyzer mandatory — fail if type-info.json missing
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~50, update OwnershipVisitor::new():
pub fn new(config: Config) -> Self {
    // Load type info - NOW REQUIRED
    let type_info_data = match type_info::load_type_info() {
        Some(data) => data,
        None => {
            // Emit compile error with helpful message
            panic!(
                "\n\n\
                ╔════════════════════════════════════════════════════════════╗\n\
                ║  ERROR: BorrowScope analyzer output not found             ║\n\
                ╠════════════════════════════════════════════════════════════╣\n\
                ║                                                            ║\n\
                ║  The #[trace_borrow] macro requires semantic analysis     ║\n\
                ║  data from borrowscope-analyzer.                           ║\n\
                ║                                                            ║\n\
                ║  Please run the analyzer first:                           ║\n\
                ║                                                            ║\n\
                ║    cargo run -p borrowscope-analyzer -- .                 ║\n\
                ║                                                            ║\n\
                ║  This will generate .borrowscope/type-info.json           ║\n\
                ║                                                            ║\n\
                ║  Then rebuild your project:                               ║\n\
                ║                                                            ║\n\
                ║    cargo build                                            ║\n\
                ║                                                            ║\n\
                ╚════════════════════════════════════════════════════════════╝\n\
                "
            );
        }
    };

    Self {
        config,
        type_info_data: Some(type_info_data),  // Always Some now
        // ... rest of fields
    }
}
```

**Rationale:** Clear error message guides users to run analyzer. No silent fallback.

---

#### Step 2.4: Use `copy_semantics` to distinguish copy vs move
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~1154, replace move detection logic:

// OLD CODE (delete lines 1154-1180):
if self.config.track_move && Self::is_potential_move(original_expr) {
    if let Expr::Path(path_expr) = original_expr.as_ref() {
        if let Some(source_ident) = path_expr.path.get_ident() {
            let source_name = source_ident.to_string();
            if let Some(&source_id) = self.var_ids.get(&source_name) {
                let new_expr: Expr = syn::parse_quote! {
                    borrowscope_runtime::track_move_with_id(...)
                };
                // ...
            }
        }
    }
}

// NEW CODE (replace with):
if self.config.track_move && Self::is_potential_move(original_expr) {
    if let Expr::Path(path_expr) = original_expr.as_ref() {
        if let Some(source_ident) = path_expr.path.get_ident() {
            let source_name = source_ident.to_string();
            
            // Check if source has copy semantics
            let is_copy = self.lookup_type_info(&source_name)
                .map(|ti| ti.copy_semantics)
                .unwrap_or(false);
            
            if is_copy {
                // It's a copy, not a move - use track_new
                let new_expr: Expr = syn::parse_quote! {
                    borrowscope_runtime::__track_new_with_id_helper(
                        #var_id, #var_name, #location, #original_expr
                    )
                };
                *init.expr = new_expr;
            } else {
                // True move - use track_move
                if let Some(&source_id) = self.var_ids.get(&source_name) {
                    let new_expr: Expr = syn::parse_quote! {
                        borrowscope_runtime::track_move_with_id(
                            #source_id, #var_id, #var_name, #location, #original_expr
                        )
                    };
                    *init.expr = new_expr;
                }
            }
        }
    }
}
```

**Rationale:** Correctly distinguishes `let y = x` (copy) from actual moves based on semantic data.

---

#### Step 2.5: Skip drop tracking for Copy/primitive types
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~2600, in drop tracking logic, add filter:

// OLD CODE (line ~2620):
for var_name in &scope_vars {
    let drop_stmt = self.create_drop_tracking(var_name);
    drop_stmts.push(drop_stmt);
}

// NEW CODE (replace with):
for var_name in &scope_vars {
    // Skip drop tracking for Copy/primitive types
    let should_track_drop = self.lookup_type_info(var_name)
        .map(|ti| !ti.copy_semantics && !ti.is_primitive)
        .unwrap_or(true);  // If no type info, track conservatively
    
    if should_track_drop {
        let drop_stmt = self.create_drop_tracking(var_name);
        drop_stmts.push(drop_stmt);
    }
}
```

**Rationale:** Reduces noise — Copy types don't have meaningful drop semantics.

---

#### Step 2.6: Remove smart pointer fallback detection
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~869-920, replace entire smart pointer detection block:

// OLD CODE (delete lines 869-920):
if self.config.track_smart_pointers {
    if let Some(type_info) = self.lookup_type_info(&var_name) {
        if let Some(ref init_kind) = type_info.initializer_kind {
            if let Some(new_expr) = self.transform_by_initializer_kind(...) {
                *init.expr = new_expr;
                visit_mut::visit_local_mut(self, local);
                return;
            }
        }
    }
    
    // Fall back to syntactic detection
    if let Some(sp_type) = detect_smart_pointer_new(original_expr) {
        // ... heuristic code
    }
}

// NEW CODE (replace with):
if self.config.track_smart_pointers {
    // Analyzer data is REQUIRED
    let type_info = self.lookup_type_info(&var_name)
        .expect("Type info missing for variable - did you run the analyzer?");
    
    if let Some(ref init_kind) = type_info.initializer_kind {
        if let Some(new_expr) = self.transform_by_initializer_kind(
            init_kind, &var_name, var_id, &location, original_expr, type_info
        ) {
            *init.expr = new_expr;
            visit_mut::visit_local_mut(self, local);
            return;
        }
    }
    
    // No fallback - if initializer_kind is None, it's not a tracked pattern
}
```

**Rationale:** Removes `detect_smart_pointer_new()` fallback. Analyzer must provide `initializer_kind`.

---

#### Step 2.7: Update method call transformation to use analyzer data
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~1700, in transform_method_call(), replace borrow inference:

// OLD CODE (delete lines ~1730-1760):
let borrow_type = if let Some(type_info) = self.lookup_type_info(&receiver_name) {
    type_info.self_borrow.clone().unwrap_or_else(|| {
        self.infer_self_borrow_type_heuristic(&receiver_name, &method_name)
    })
} else {
    self.infer_self_borrow_type_heuristic(&receiver_name, &method_name)
};

// NEW CODE (replace with):
let borrow_type = {
    // Get line number for this method call
    let line = call.span().start().line as u32;
    
    // Look up in analyzer's method_borrows data
    self.lookup_method_borrow_info(&receiver_name, &method_name, line)
        .expect(&format!(
            "Method borrow info missing for {}.{}() at line {} - analyzer may need update",
            receiver_name, method_name, line
        ))
};
```

**Rationale:** No heuristic fallback. Analyzer must provide borrow info for all method calls.

---

#### Step 2.8: Remove heuristic sets (`mutex_vars`, `refcell_vars`, etc.)
**File:** `borrowscope-macro/src/transform_visitor.rs`

**Changes:**
```rust
// Line ~60-70, DELETE these fields from OwnershipVisitor:
mutex_vars: HashSet<String>,           // DELETE
rwlock_vars: HashSet<String>,          // DELETE
refcell_vars: HashSet<String>,         // DELETE
cell_vars: HashSet<String>,            // DELETE
arc_vars: HashSet<String>,             // DELETE
rc_vars: HashSet<String>,              // DELETE
box_vars: HashSet<String>,             // DELETE
pin_vars: HashSet<String>,             // DELETE
unsafe_vars: HashSet<String>,          // DELETE

// Line ~2417-2743, DELETE entire heuristic fallback block:
// This is the code that checks these sets when semantic_op is None
// DELETE lines 2417-2743
```

**Rationale:** These sets were used for heuristic fallback. With analyzer mandatory, all type info comes from `type_info_data`.

---

### Phase 3: Type Info Schema Updates

#### Step 3.1: Update `type_info.rs` to match new analyzer output
**File:** `borrowscope-macro/src/type_info.rs`

**Changes:**
```rust
// Line ~30, add new fields to TypeInfoData:
#[derive(Debug, Clone, Deserialize)]
pub struct TypeInfoData {
    pub variables: HashMap<String, VariableTypeInfo>,
    
    // NEW: Add these fields
    #[serde(default)]
    pub method_borrows: Vec<MethodBorrowInfo>,
    
    #[serde(default)]
    pub field_accesses: Vec<FieldAccessInfo>,
    
    #[serde(default)]
    pub function_calls: Vec<FunctionCallInfo>,
}

// Line ~100, add new structs matching analyzer output:
#[derive(Debug, Clone, Deserialize)]
pub struct MethodBorrowInfo {
    pub method_name: String,
    pub receiver_var: String,
    pub borrow_kind: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldAccessInfo {
    pub base_var: String,
    pub field_path: Vec<String>,
    pub access_kind: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCallInfo {
    pub function_name: String,
    pub return_category: String,
    pub is_copy_return: bool,
    pub line: u32,
    pub column: u32,
}

// Line ~80, add to VariableTypeInfo:
#[serde(default)]
pub copy_semantics: bool,  // NEW: from analyzer
```

**Rationale:** Macro's type info structs must match analyzer's JSON schema.

---

### Phase 4: Testing Infrastructure

#### Step 4.1: Generate type-info.json for all macro test fixtures
**Action:** For each test in `borrowscope-macro/tests/*.rs`:

1. Extract test code into temporary project
2. Run analyzer: `cargo run -p borrowscope-analyzer -- /tmp/test_project`
3. Copy `.borrowscope/type-info.json` to `borrowscope-macro/tests/fixtures/test_name.json`
4. Update test to load fixture

**Example:**
```rust
// borrowscope-macro/tests/smart_pointers.rs

// OLD:
#[test]
fn test_rc_new() {
    let input = quote! {
        let x = Rc::new(42);
    };
    // ... test without type info
}

// NEW:
#[test]
fn test_rc_new() {
    // Load pre-generated type info
    std::env::set_var(
        "BORROWSCOPE_TYPE_INFO",
        "tests/fixtures/rc_new.json"
    );
    
    let input = quote! {
        let x = Rc::new(42);
    };
    // ... test with type info
}
```

**Files to create:**
- `borrowscope-macro/tests/fixtures/` directory
- ~50 JSON files (one per test case)

---

#### Step 4.2: Add integration test for analyzer-macro pipeline
**File:** `borrowscope-macro/tests/integration_analyzer_required.rs` (NEW)

**Content:**
```rust
use quote::quote;
use borrowscope_macro::trace_borrow_impl;

#[test]
#[should_panic(expected = "BorrowScope analyzer output not found")]
fn test_macro_fails_without_analyzer() {
    // Ensure no type-info.json exists
    std::env::remove_var("BORROWSCOPE_TYPE_INFO");
    
    let input = quote! {
        fn example() {
            let x = Rc::new(42);
        }
    };
    
    // Should panic with helpful error message
    let _ = trace_borrow_impl(input);
}

#[test]
fn test_macro_succeeds_with_analyzer() {
    // Point to valid type-info.json
    std::env::set_var(
        "BORROWSCOPE_TYPE_INFO",
        "tests/fixtures/rc_new.json"
    );
    
    let input = quote! {
        fn example() {
            let x = Rc::new(42);
        }
    };
    
    // Should succeed
    let output = trace_borrow_impl(input);
    assert!(output.is_ok());
}
```

---


### Phase 5: Documentation Updates

#### Step 5.1: Update README.md
**File:** `README.md`

**Changes:** Add analyzer as required dependency, update Quick Start to show 3-step workflow (analyzer → macro → build).

#### Step 5.2: Update borrowscope-macro README
**File:** `borrowscope-macro/README.md`

**Changes:** Add warning that analyzer is required, show error message users will see, explain architecture.

#### Step 5.3: Update SEMANTIC_IMPLEMENTATION.md
**File:** `borrowscope-analyzer/SEMANTIC_IMPLEMENTATION.md`

**Changes:** Update success criteria to reflect zero heuristics, analyzer mandatory, complete coverage.

---

### Phase 6: Migration Guide

#### Step 6.1: Create MIGRATION.md
**File:** `MIGRATION.md` (NEW)

Document breaking change, migration steps, troubleshooting, benefits.

---

## Implementation Timeline

- **Phase 1:** 2-3 days (analyzer enhancements)
- **Phase 2:** 2-3 days (macro simplification)
- **Phase 3:** 1 day (schema updates)
- **Phase 4:** 2-3 days (testing)
- **Phase 5:** 1 day (documentation)
- **Phase 6:** 1 day (migration guide)

**Total:** 9-12 days

---

## Code Deletion Summary

### Files to Delete
- `borrowscope-macro/src/smart_pointer.rs` (~400 lines)

### Functions to Delete (11 total, ~480 lines)
- All `detect_*` functions in smart_pointer.rs
- `infer_self_borrow_type_heuristic()` in transform_visitor.rs

### Fields to Delete (9 HashSet fields)
- `mutex_vars`, `rwlock_vars`, `refcell_vars`, `cell_vars`
- `arc_vars`, `rc_vars`, `box_vars`, `pin_vars`, `unsafe_vars`

**Net Change:** -500 lines (900 deleted, 400 added)

---

## Success Metrics

### Code Quality
- ✅ Zero heuristics (verified by grep)
- ✅ All 567 macro tests pass
- ✅ All 9 analyzer tests pass

### User Experience
- ✅ Clear error when analyzer not run
- ✅ Complete documentation
- ✅ Migration guide

### Performance
- ✅ Analyzer <10s for typical projects
- ✅ Macro compilation unchanged

---

## Next Steps

1. Review this design document
2. Create GitHub issue/epic
3. Begin Phase 1 (analyzer enhancements)
4. Incremental commits per phase
5. Continuous test updates

