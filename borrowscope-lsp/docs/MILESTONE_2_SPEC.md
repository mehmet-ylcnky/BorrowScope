# Milestone 2: Ownership Analysis Engine - Detailed Specification

## 2.1 Exhaustive Type Extraction (all 55 hir::Type methods per variable)

**Objective:** For every variable binding in the active file, call all available methods on `hir::Type` and store the results in a structured `VariableOwnershipInfo`. This ensures we never miss information that rust-analyzer can provide. The extraction runs on-demand when the client requests analysis for a file or function.

**Steps:**
1. Walk the file's syntax tree to find all `let` statements and function parameters
2. For each binding pattern, call `sema.type_of_pat()` to get the resolved type
3. Call all 55 `hir::Type` methods and store results in `VariableOwnershipInfo`
4. Cache results per file (invalidated when file changes via Salsa)

**Code (analysis.rs):**
```rust
use ra_ap_hir::{Adt, HirDatabase, Semantics, Type};

/// Complete ownership information for a single variable.
#[derive(Debug, Clone, Serialize)]
pub struct VariableOwnershipInfo {
    // Identity
    pub name: String,
    pub type_display: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function_name: Option<String>,

    // Boolean type properties (no db needed)
    pub is_unit: bool,
    pub is_bool: bool,
    pub is_str: bool,
    pub is_never: bool,
    pub is_reference: bool,
    pub is_mutable_reference: bool,
    pub is_slice: bool,
    pub is_usize: bool,
    pub is_float: bool,
    pub is_char: bool,
    pub is_int_or_uint: bool,
    pub is_scalar: bool,
    pub is_tuple: bool,
    pub is_array: bool,
    pub is_closure: bool,
    pub is_fn: bool,
    pub is_raw_ptr: bool,
    pub is_unknown: bool,
    pub contains_unknown: bool,

    // Queries requiring db
    pub is_copy: bool,
    pub is_packed: bool,
    pub contains_reference: bool,
    pub impls_fnonce: bool,
    pub impls_iterator: bool,

    // Decomposition
    pub reference_inner: Option<TypeDecomposition>,
    pub adt_info: Option<AdtInfo>,
    pub builtin_type: Option<String>,
    pub dyn_trait: Option<String>,
    pub impl_traits: Vec<String>,
    pub type_arguments: Vec<String>,
    pub future_output: Option<String>,
    pub iterator_item: Option<String>,
    pub tuple_fields: Vec<String>,
    pub struct_fields: Vec<FieldInfo>,
    pub array_info: Option<ArrayInfo>,
    pub autoderef_chain: Vec<String>,
    pub callable_info: Option<CallableInfo>,

    // Layout
    pub layout_size: Option<u64>,
    pub layout_align: Option<u64>,
    pub has_drop_glue: bool,

    // ADT classification (canonical path based)
    pub adt_canonical_path: Option<String>,
    pub ownership_category: OwnershipCategory,

    // Trait implementations (checked exhaustively)
    pub trait_impls: TraitImplInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeDecomposition {
    pub inner_type: String,
    pub mutability: String, // "shared" or "mutable"
}

#[derive(Debug, Clone, Serialize)]
pub struct AdtInfo {
    pub kind: String, // "struct", "enum", "union"
    pub name: String,
    pub canonical_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArrayInfo {
    pub element_type: String,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallableInfo {
    pub params: Vec<String>,
    pub return_type: String,
    pub is_closure: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum OwnershipCategory {
    Owned,          // Vec, String, Box, user structs
    SharedRef,      // &T
    MutableRef,     // &mut T
    SharedOwnership,// Rc, Arc
    InteriorMut,    // RefCell, Cell, Mutex
    RawPointer,     // *const T, *mut T
    Copy,           // i32, bool, &T
    Unknown,
}

/// Extract all ownership info for a single type.
pub fn extract_full_type_info(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    ty: &Type<'_>,
    name: &str,
    location: (String, u32, u32),
    function_name: Option<String>,
) -> VariableOwnershipInfo {
    let display_target = db.display_target();

    VariableOwnershipInfo {
        name: name.to_string(),
        type_display: ty.display(db, display_target).to_string(),
        file: location.0,
        line: location.1,
        column: location.2,
        function_name,

        // All boolean queries
        is_unit: ty.is_unit(),
        is_bool: ty.is_bool(),
        is_str: ty.is_str(),
        is_never: ty.is_never(),
        is_reference: ty.is_reference(),
        is_mutable_reference: ty.is_mutable_reference(),
        is_slice: ty.is_slice(),
        is_usize: ty.is_usize(),
        is_float: ty.is_float(),
        is_char: ty.is_char(),
        is_int_or_uint: ty.is_int_or_uint(),
        is_scalar: ty.is_scalar(),
        is_tuple: ty.is_tuple(),
        is_array: ty.is_array(),
        is_closure: ty.is_closure(),
        is_fn: ty.is_fn(),
        is_raw_ptr: ty.is_raw_ptr(),
        is_unknown: ty.is_unknown(),
        contains_unknown: ty.contains_unknown(),

        // Db-requiring queries
        is_copy: ty.is_copy(db),
        is_packed: ty.is_packed(db),
        contains_reference: ty.contains_reference(db),
        impls_fnonce: ty.impls_fnonce(db),
        impls_iterator: ty.clone().impls_iterator(db),

        // Decomposition
        reference_inner: ty.as_reference().map(|(inner, m)| TypeDecomposition {
            inner_type: inner.display(db, display_target).to_string(),
            mutability: format!("{:?}", m),
        }),
        adt_info: extract_adt_info(db, ty),
        builtin_type: ty.as_builtin().map(|b| format!("{:?}", b)),
        dyn_trait: ty.as_dyn_trait().map(|t| t.name(db).to_string()),
        impl_traits: ty.as_impl_traits(db)
            .map(|traits| traits.map(|t| t.name(db).to_string()).collect())
            .unwrap_or_default(),
        type_arguments: ty.type_arguments()
            .map(|t| t.display(db, display_target).to_string())
            .collect(),
        future_output: ty.clone().future_output(db)
            .map(|t| t.display(db, display_target).to_string()),
        iterator_item: ty.clone().iterator_item(db)
            .map(|t| t.display(db, display_target).to_string()),
        tuple_fields: ty.tuple_fields(db).iter()
            .map(|t| t.display(db, display_target).to_string())
            .collect(),
        struct_fields: ty.fields(db).iter()
            .map(|(f, t)| FieldInfo {
                name: f.name(db).to_string(),
                ty: t.display(db, display_target).to_string(),
            })
            .collect(),
        array_info: ty.as_array(db).map(|(elem, len)| ArrayInfo {
            element_type: elem.display(db, display_target).to_string(),
            length: len,
        }),
        autoderef_chain: ty.autoderef(db)
            .map(|t| t.display(db, display_target).to_string())
            .collect(),
        callable_info: ty.as_callable(db).map(|c| CallableInfo {
            params: c.params().iter()
                .map(|p| p.ty().display(db, display_target).to_string())
                .collect(),
            return_type: c.return_type().display(db, display_target).to_string(),
            is_closure: ty.is_closure(),
        }),

        // Layout
        layout_size: ty.layout(db).ok().map(|l| l.size()),
        layout_align: ty.layout(db).ok().map(|l| l.align()),
        has_drop_glue: matches!(ty.drop_glue(db), ra_ap_hir::DropGlue::HasDropGlue),

        // Classification
        adt_canonical_path: extract_adt_info(db, ty).map(|a| a.canonical_path),
        ownership_category: classify_ownership(db, ty),

        // Traits
        trait_impls: check_all_traits(db, ty),
    }
}
```

**Expectation:** Every variable produces a complete `VariableOwnershipInfo` with all 55+ fields populated. No information that rust-analyzer can provide is left unextracted.

**Tests for 2.1:**
- `i32` variable: `is_copy=true`, `is_scalar=true`, `is_int_or_uint=true`, `ownership_category=Copy`
- `Vec<i32>` variable: `is_copy=false`, `has_drop_glue=true`, `adt_info.kind="struct"`, `type_arguments=["i32"]`
- `&mut Vec<i32>`: `is_mutable_reference=true`, `reference_inner.mutability="mutable"`
- `Rc<String>`: `adt_canonical_path="alloc::rc::Rc"`, `ownership_category=SharedOwnership`
- `impl Future<Output=i32>`: `future_output=Some("i32")`, `impl_traits=["Future"]`
- `[u8; 32]`: `is_array=true`, `array_info={element_type:"u8", length:32}`
- Unknown type: `is_unknown=true`, `ownership_category=Unknown`
- Struct with fields: `struct_fields` populated with name+type for each field

---

## 2.2 Method Call Resolution

**Objective:** For every method call on a tracked variable, resolve it to its definition and extract: the canonical path, self-borrow type, receiver type, return type, whether it's a trait method, and the trait name. This is the same data the current analyzer extracts but computed live.

**Steps:**
1. Walk the function body to find all `MethodCallExpr` nodes
2. For each, call `sema.resolve_method_call()` to get the `Function`
3. Extract self parameter access mode (`Shared`, `Exclusive`, `Owned`)
4. Build canonical path from the function's module hierarchy
5. Detect trait methods via `func.as_assoc_item().containing_trait()`

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct MethodCallResolution {
    pub method_name: String,
    pub line: u32,
    pub column: u32,
    pub canonical_path: String,
    pub self_borrow: SelfBorrow,
    pub receiver_type: String,
    pub return_type: String,
    pub is_trait_method: bool,
    pub trait_name: Option<String>,
    pub is_unsafe: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum SelfBorrow {
    Shared,    // &self
    Exclusive, // &mut self
    Owned,     // self (consuming)
}

pub fn resolve_method_calls(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function_body: &ast::BlockExpr,
) -> Vec<MethodCallResolution> {
    let mut results = Vec::new();

    for method_call in function_body.syntax().descendants()
        .filter_map(ast::MethodCallExpr::cast)
    {
        if let Some(func) = sema.resolve_method_call(&method_call) {
            let self_borrow = match func.self_param(db) {
                Some(param) => match param.access(db) {
                    ra_ap_hir::Access::Shared => SelfBorrow::Shared,
                    ra_ap_hir::Access::Exclusive => SelfBorrow::Exclusive,
                    ra_ap_hir::Access::Owned => SelfBorrow::Owned,
                },
                None => SelfBorrow::Owned,
            };

            let is_trait_method = func.as_assoc_item(db)
                .and_then(|item| item.containing_trait(db))
                .is_some();

            let trait_name = func.as_assoc_item(db)
                .and_then(|item| item.containing_trait(db))
                .map(|t| t.name(db).to_string());

            results.push(MethodCallResolution {
                method_name: method_call.name_ref()
                    .map(|n| n.text().to_string())
                    .unwrap_or_default(),
                line: /* extract from syntax node */,
                column: /* extract from syntax node */,
                canonical_path: get_function_canonical_path(&func, db),
                self_borrow,
                receiver_type: /* from sema.type_of_expr on receiver */,
                return_type: /* from func.ret_type */,
                is_trait_method,
                trait_name,
                is_unsafe: func.is_unsafe(db),
            });
        }
    }
    results
}
```

**Expectation:** Every method call in the function is resolved to its definition with full metadata. Unresolvable calls (macros, dynamic dispatch) are skipped without error.

**Tests for 2.2:**
- `vec.push(1)`: `canonical_path="alloc::vec::Vec::push"`, `self_borrow=Exclusive`
- `vec.len()`: `canonical_path="alloc::vec::Vec::len"`, `self_borrow=Shared`
- `rc.clone()`: `is_trait_method=true`, `trait_name=Some("Clone")`
- `string.into_bytes()`: `self_borrow=Owned` (consuming)
- `mutex.lock()`: `canonical_path` contains "mutex", `return_type` contains "MutexGuard"
- Unresolvable method (macro-generated): skipped, no panic

---

## 2.3 Borrow Scope Computation

**Objective:** For each borrow (`&x` or `&mut x`) in a function, compute the source range where the borrow is active. This is the region from the borrow creation to its last use (NLL semantics), which the editor will highlight with a colored background.

**Steps:**
1. Find all reference expressions (`&expr` and `&mut expr`) in the function
2. Identify the variable being borrowed (the target)
3. Find the last usage of the borrow variable (NLL end point)
4. Compute the source range (start line/col to end line/col)
5. Determine if the borrow is shared or mutable

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct BorrowScopeInfo {
    pub borrower_name: String,
    pub target_name: String,
    pub is_mutable: bool,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub last_use_line: Option<u32>,
}

pub fn compute_borrow_scopes(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> Vec<BorrowScopeInfo> {
    // Walk let statements looking for reference bindings
    // For each, find the scope where the borrow is live
    // Use sema to determine last usage point
    todo!()
}
```

**Borrow scope visualization concept:**
```
fn example() {
    let data = vec![1, 2, 3];
    let r = &data;          // ← borrow starts here
    │                       //
    │  println!("{}", r);   // ← last use of r (NLL end)
    │                       //
    │  // r is technically still in scope but NLL says borrow ends above
    │
    let m = &mut data;      // ← this is OK because r's borrow ended at last use
    m.push(4);
}
```

**Expectation:** Each borrow produces a `BorrowScopeInfo` with the exact source range. The end point reflects NLL semantics (last use, not lexical scope end) when usage data is available.

**Tests for 2.3:**
- Simple borrow: scope from `let r = &x` to last use of `r`
- Mutable borrow: `is_mutable=true`
- Borrow in a block: scope ends at block exit
- Borrow passed to function: scope extends to the function call
- Multiple borrows of same variable: each gets its own scope
- Borrow with no uses after creation: scope is just the declaration line

---

## 2.4 Move Detection

**Objective:** Identify all ownership transfers (moves) within a function. A move occurs when a non-Copy value is assigned to another variable, passed to a function by value, or returned. After a move, the source variable is invalidated.

**Steps:**
1. Find assignments where the RHS is a non-Copy variable
2. Find function calls where a non-Copy argument is passed by value
3. Find return statements returning a non-Copy local variable
4. For each move, record source variable, destination, and location

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct MoveInfo {
    pub source_name: String,
    pub destination: MoveDestination,
    pub line: u32,
    pub column: u32,
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum MoveDestination {
    Variable(String),       // let b = a;
    FunctionArg(String),    // foo(a) - function name
    Return,                 // return a;
    ClosureCapture(String), // captured by move closure
}

pub fn detect_moves(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> Vec<MoveInfo> {
    todo!()
}
```

**Expectation:** Every ownership transfer is detected with source, destination, and location. Copy types are not flagged as moves.

**Tests for 2.4:**
- `let b = a;` where `a: String`: detected as move to Variable("b")
- `let b = a;` where `a: i32`: NOT detected (Copy type)
- `foo(a)` where `a: Vec<i32>`: detected as move to FunctionArg("foo")
- `return data;`: detected as move to Return
- `move || { use(x) }`: detected as move to ClosureCapture
- Reborrow (`let r2 = &*r1`): NOT detected as move

---

## 2.5 Closure Capture Analysis

**Objective:** For each closure in a function, determine which variables it captures and the capture mode (by shared reference, by mutable reference, or by move). This information is critical for understanding why certain borrows exist.

**Steps:**
1. Find all closure expressions in the function
2. For each closure, use `sema` to determine captured variables
3. Classify capture mode: `&`, `&mut`, or `move`
4. Record the closure's `Fn`/`FnMut`/`FnOnce` trait implementation

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ClosureCaptureInfo {
    pub closure_line: u32,
    pub closure_column: u32,
    pub fn_trait: FnTrait,
    pub captures: Vec<CapturedVariable>,
}

#[derive(Debug, Clone, Serialize)]
pub enum FnTrait {
    Fn,
    FnMut,
    FnOnce,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapturedVariable {
    pub name: String,
    pub capture_mode: CaptureMode,
    pub variable_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum CaptureMode {
    BySharedRef,
    ByMutRef,
    ByMove,
}

pub fn analyze_closures(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> Vec<ClosureCaptureInfo> {
    todo!()
}
```

**Expectation:** Every closure's captures are fully enumerated with correct modes. `move` closures show `ByMove` for all captures.

**Tests for 2.5:**
- `|| println!("{}", x)`: captures `x` by shared ref, trait=`Fn`
- `|| x.push(1)`: captures `x` by mut ref, trait=`FnMut`
- `move || drop(x)`: captures `x` by move, trait=`FnOnce`
- `|| {}` (no captures): empty captures list
- Closure capturing multiple variables: all listed with correct modes

---

## 2.6 Rc/Arc Clone Tracking

**Objective:** Identify all `Rc::clone()` and `Arc::clone()` calls and track which variables share ownership of the same allocation. This enables reference count visualization.

**Steps:**
1. Find method calls where the resolved function is `Clone::clone` on an Rc/Arc type
2. Find function calls to `Rc::clone(&x)` (explicit clone syntax)
3. Group clones by their source (all clones of the same Rc form a "family")
4. Record the clone point (line/col) for each

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct RcCloneInfo {
    pub clone_variable: String,
    pub source_variable: String,
    pub clone_type: RcType,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize)]
pub enum RcType {
    Rc,
    Arc,
}

pub fn track_rc_clones(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> Vec<RcCloneInfo> {
    todo!()
}
```

**Expectation:** Every Rc/Arc clone is detected regardless of syntax (`x.clone()`, `Rc::clone(&x)`, `Clone::clone(&x)`).

**Tests for 2.6:**
- `let b = a.clone()` where `a: Rc<T>`: detected as Rc clone
- `let b = Rc::clone(&a)`: detected as Rc clone
- `let b = a.clone()` where `a: String`: NOT detected (not Rc/Arc)
- `let b = a.clone()` where `a: Arc<T>`: detected as Arc clone
- Multiple clones from same source: all share same `source_variable`

---

## 2.7 Conflict Detection

**Objective:** Detect potential borrow conflicts within a function: places where a mutable borrow and a shared borrow (or two mutable borrows) of the same variable could overlap. These are shown as diagnostics in the editor.

**Steps:**
1. Collect all borrow scopes from 2.3
2. Group by target variable
3. For each pair of borrows on the same target, check if their scopes overlap
4. If one is mutable and they overlap, report a conflict

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct BorrowConflict {
    pub variable: String,
    pub borrow_a: BorrowScopeInfo,
    pub borrow_b: BorrowScopeInfo,
    pub conflict_kind: ConflictKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConflictKind {
    MutableAndShared,
    MultipleMutable,
}

pub fn detect_conflicts(scopes: &[BorrowScopeInfo]) -> Vec<BorrowConflict> {
    let mut conflicts = Vec::new();
    // Group by target, check pairwise overlap
    // Only flag if at least one is mutable
    todo!()
}
```

**Expectation:** Conflicts are detected statically from borrow scope overlap. These are educational/informational (the Rust compiler already prevents them), showing the user WHY the borrow checker would reject certain patterns.

**Tests for 2.7:**
- `&x` and `&x` overlapping: no conflict (multiple shared OK)
- `&x` and `&mut x` overlapping: conflict detected (MutableAndShared)
- `&mut x` and `&mut x` overlapping: conflict detected (MultipleMutable)
- `&mut x` then `&x` (non-overlapping): no conflict
- Borrow inside a block that ends before the next borrow: no conflict

---

## 2.8 Per-Function Ownership Summary

**Objective:** Produce a complete ownership summary for a single function, combining all the analyses above into a single response object that the LSP can return to the client.

**Steps:**
1. Run all analyses (2.1-2.7) for the function
2. Combine into a `FunctionOwnershipSummary`
3. Cache the result (invalidated when function body changes)

**Code:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct FunctionOwnershipSummary {
    pub function_name: String,
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,

    // All variables with full type info
    pub variables: Vec<VariableOwnershipInfo>,

    // All method calls resolved
    pub method_calls: Vec<MethodCallResolution>,

    // Borrow scopes for highlighting
    pub borrow_scopes: Vec<BorrowScopeInfo>,

    // Moves for tracking ownership transfers
    pub moves: Vec<MoveInfo>,

    // Closure captures
    pub closures: Vec<ClosureCaptureInfo>,

    // Rc/Arc clone relationships
    pub rc_clones: Vec<RcCloneInfo>,

    // Detected conflicts (educational)
    pub conflicts: Vec<BorrowConflict>,

    // Summary statistics
    pub stats: FunctionStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionStats {
    pub total_variables: usize,
    pub total_borrows: usize,
    pub mutable_borrows: usize,
    pub moves: usize,
    pub rc_clones: usize,
    pub closures: usize,
    pub conflicts: usize,
}

pub fn analyze_function(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> FunctionOwnershipSummary {
    let variables = extract_all_variables(db, sema, function);
    let method_calls = resolve_method_calls(db, sema, function.body().unwrap());
    let borrow_scopes = compute_borrow_scopes(db, sema, function);
    let moves = detect_moves(db, sema, function);
    let closures = analyze_closures(db, sema, function);
    let rc_clones = track_rc_clones(db, sema, function);
    let conflicts = detect_conflicts(&borrow_scopes);

    FunctionOwnershipSummary {
        function_name: function.name().map(|n| n.text().to_string()).unwrap_or_default(),
        // ... fill all fields
        stats: FunctionStats {
            total_variables: variables.len(),
            total_borrows: borrow_scopes.len(),
            mutable_borrows: borrow_scopes.iter().filter(|b| b.is_mutable).count(),
            moves: moves.len(),
            rc_clones: rc_clones.len(),
            closures: closures.len(),
            conflicts: conflicts.len(),
        },
    }
}
```

**Expectation:** A single call to `analyze_function()` returns everything the client needs to render the ownership visualization for that function. The response is JSON-serializable and sent over LSP.

**Tests for 2.8:**
- Empty function: all counts are 0, no errors
- Function with one variable: `variables.len() == 1`, full type info populated
- Function with borrows + moves: all sections populated correctly
- Function with conflicts: `conflicts` non-empty, `stats.conflicts > 0`
- Analysis completes in < 50ms for a typical function (10-20 variables)
- Result is JSON-serializable without errors
