# BorrowScope Battle Test Analysis

## Summary Across All Projects

| Project | Functions | Pass Rate | Primary Issues |
|---------|-----------|-----------|----------------|
| ripgrep | 927 | 99.2% | ERR-002, ERR-003, ERR-012, ERR-013 |
| tokio | 3,748 | 71.2% | ERR-012, ERR-014, ERR-003 |
| bat | ~200 | ~85% | ERR-007, ERR-008, ERR-009 |
| fd | ~150 | ~80% | ERR-002, ERR-003, ERR-008, ERR-009 |
| zoxide | ~50 | ~70% | ERR-002, ERR-003, ERR-004 |

---

## Error Pattern Taxonomy

### Critical Priority (Must Fix)

#### 1. ERR-003: Mutable Borrow Conflicts
**Frequency:** Very High (appears in ALL 5 projects)
**Rust Errors:** E0596, E0507

**Pattern:**
```rust
#[trace_borrow]
fn example() {
    let mut cmd = Command::new("ls");
    cmd.args(["--help"]);  // Error: cannot borrow as mutable
}
```

**Root Cause:**
Macro wraps method receivers with `track_borrow("name", &receiver)` which returns `&T`, but method requires `&mut self`.

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs - detect mutable method calls
fn transform_method_call(&mut self, expr: &mut ExprMethodCall) {
    // Option 1: Use track_borrow_mut for mutable receivers
    // Option 2: Skip tracking method call receivers entirely
    // Option 3: Only track after method chain completes
}
```

---

#### 2. ERR-012: Trait Impl Method Signature Mismatch
**Frequency:** Very High (tokio: 517 errors, ripgrep: 4 errors)
**Rust Errors:** E0407, E0599

**Pattern:**
```rust
impl AsyncRead for MyType {
    #[trace_borrow]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<...> {
        // Error: method `poll_read` is not a member of trait
    }
}
```

**Root Cause:**
Macro transformation changes method signature (return type or parameters), breaking trait conformance.

**Fix in borrowscope-macro:**
```rust
// In lib.rs or transform_visitor.rs
fn is_trait_impl_method(item: &ItemFn) -> bool {
    // Detect if function is inside impl Trait for Type block
}

// For trait impl methods: preserve exact signature, only instrument body
if is_trait_impl_method(&item_fn) {
    return transform_body_only(&item_fn);
}
```

---

#### 3. ERR-002: Tuple/Variable Scope Corruption
**Frequency:** High (appears in 4 projects)
**Rust Errors:** E0425

**Pattern:**
```rust
#[trace_borrow]
fn parse(input: &str) -> (String, bool) {
    let (s, flag) = match input { ... };
    (s.to_string(), flag)  // Error: cannot find value `s`
}
```

**Root Cause:**
Macro transforms tuple patterns into single temporary but fails to extract individual elements.

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs
fn transform_let(&mut self, local: &mut Local) {
    match &local.pat {
        Pat::Tuple(tuple) => {
            // Skip tracking OR track the whole tuple then destructure
            // let __tracked = track_new("tuple", (a, b));
            // let (a, b) = __tracked;
        }
        _ => { /* normal transformation */ }
    }
}
```

---

#### 4. ERR-009: Self-Consuming / Move Semantics
**Frequency:** High (appears in 3 projects)
**Rust Errors:** E0507, E0515

**Pattern:**
```rust
#[trace_borrow]
pub fn with_name(self, name: &str) -> Self {
    self  // Error: cannot move out of shared reference
}
```

**Root Cause:**
Macro transforms `self` to `&self`, preventing ownership transfer.

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs
fn is_self_consuming(sig: &Signature) -> bool {
    sig.inputs.iter().any(|arg| {
        matches!(arg, FnArg::Receiver(r) if r.reference.is_none())
    })
}

// Skip or use track_move for self-consuming functions
if is_self_consuming(&item_fn.sig) {
    return transform_with_move_tracking(&item_fn);
}
```

---

### High Priority

#### 5. ERR-014: Trait Method Declaration Without Body
**Frequency:** Medium (tokio-specific but common pattern)
**Rust Errors:** "expected curly braces"

**Pattern:**
```rust
trait MyTrait {
    #[trace_borrow]
    fn my_method(&self) -> i32;  // Error: expected curly braces
}
```

**Fix in borrowscope-macro:**
```rust
// In lib.rs - detect trait method declarations
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn: ItemFn = match syn::parse(item.clone()) {
        Ok(f) => f,
        Err(_) => {
            // Could be trait method declaration - return unchanged
            return item;
        }
    };
    
    // Check if function has no body (trait declaration)
    if item_fn.block.stmts.is_empty() && /* ends with semicolon */ {
        return item; // Return unchanged
    }
}
```

---

#### 6. ERR-008: impl Into/Trait Bounds Fail
**Frequency:** Medium (appears in 3 projects)
**Rust Errors:** E0277, E0282

**Pattern:**
```rust
#[trace_borrow]
pub fn new(s: impl Into<String>) -> Self {
    let s = s.into();  // Error: trait bound not satisfied
}
```

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs - skip tracking for impl Trait parameters
fn should_track_param(param: &FnArg) -> bool {
    match param {
        FnArg::Typed(pat_type) => {
            // Check if type is impl Trait
            !matches!(&*pat_type.ty, Type::ImplTrait(_))
        }
        _ => true
    }
}
```

---

#### 7. ERR-004: Const Context Issues
**Frequency:** Low (zoxide-specific)
**Rust Errors:** E0015

**Pattern:**
```rust
#[trace_borrow]
fn example() {
    const VALUE: usize = if cfg!(windows) { 5 } else { 1 };
    // Error: cannot call non-const function in constants
}
```

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs - track const context
struct OwnershipVisitor {
    in_const_context: bool,
}

impl VisitMut for OwnershipVisitor {
    fn visit_item_const_mut(&mut self, item: &mut ItemConst) {
        self.in_const_context = true;
        visit_item_const_mut(self, item);
        self.in_const_context = false;
    }
    
    fn visit_expr_if_mut(&mut self, expr: &mut ExprIf) {
        if self.in_const_context {
            return; // Skip tracking in const context
        }
        // Normal transformation
    }
}
```

---

#### 8. ERR-013: Lifetime Mismatch
**Frequency:** Medium (ripgrep, zoxide)
**Rust Errors:** E0597, E0716

**Pattern:**
```rust
impl<'a> Candidate<'a> {
    #[trace_borrow]
    pub fn new(path: &'a Path) -> Candidate<'a> {
        // Error: `path` does not live long enough
    }
}
```

**Fix in borrowscope-macro:**
```rust
// In transform_visitor.rs - preserve lifetime annotations
fn transform_with_lifetimes(item_fn: &ItemFn) -> TokenStream {
    // Don't create temporaries that outlive the function
    // Track references without extending their lifetime
}
```

---

## Recommended Implementation Order

### Phase 1: Critical Fixes (Highest Impact)
1. **ERR-003**: Mutability-aware method call transformation
2. **ERR-012**: Trait impl method detection and signature preservation
3. **ERR-002**: Tuple pattern handling

### Phase 2: High Priority
4. **ERR-009**: Self-consuming function detection
5. **ERR-014**: Trait method declaration detection
6. **ERR-008**: impl Trait parameter handling

### Phase 3: Medium Priority
7. **ERR-004**: Const context awareness
8. **ERR-013**: Lifetime-preserving transformations

---

## Changes Required

### borrowscope-macro/src/lib.rs

```rust
// Add early detection for unsupported patterns
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Try to parse as ItemFn
    let item_fn: ItemFn = match syn::parse(item.clone()) {
        Ok(f) => f,
        Err(_) => return item, // Return unchanged if not a function
    };
    
    // 2. Check for trait method declaration (no body)
    if is_trait_method_declaration(&item_fn) {
        return item;
    }
    
    // 3. Check for self-consuming functions
    if is_self_consuming(&item_fn.sig) {
        return transform_with_move_tracking(item_fn);
    }
    
    // 4. Normal transformation
    transform_function(item_fn, attr)
}
```

### borrowscope-macro/src/transform_visitor.rs

```rust
struct OwnershipVisitor {
    // Add context tracking
    in_const_context: bool,
    in_trait_impl: bool,
    current_mutability: HashMap<String, bool>, // Track variable mutability
}

impl VisitMut for OwnershipVisitor {
    // 1. Track const context
    fn visit_item_const_mut(&mut self, item: &mut ItemConst) { ... }
    
    // 2. Handle tuple patterns
    fn visit_local_mut(&mut self, local: &mut Local) {
        if let Pat::Tuple(_) = &local.pat {
            // Skip or handle specially
        }
    }
    
    // 3. Handle method calls with mutability awareness
    fn visit_expr_method_call_mut(&mut self, expr: &mut ExprMethodCall) {
        let receiver_name = extract_receiver_name(&expr.receiver);
        if self.current_mutability.get(&receiver_name) == Some(&true) {
            // Use track_borrow_mut or skip
        }
    }
    
    // 4. Skip tracking for impl Trait parameters
    fn should_track_param(&self, param: &FnArg) -> bool { ... }
}
```

### borrowscope-runtime (No Changes Required)

The runtime library already has all necessary tracking functions:
- `track_new`, `track_borrow`, `track_borrow_mut`, `track_move`, `track_drop`

The issues are entirely in the **macro transformation logic**, not the runtime.

---

## Test Cases to Add

```rust
// tests/trait_impl.rs
#[test]
fn test_trait_impl_method() {
    // Should compile without errors
}

// tests/tuple_patterns.rs
#[test]
fn test_tuple_destructuring() {
    // Should preserve variable bindings
}

// tests/mutable_chains.rs
#[test]
fn test_mutable_method_chain() {
    // Should allow mutable method calls
}

// tests/self_consuming.rs
#[test]
fn test_self_consuming_function() {
    // Should handle ownership transfer
}

// tests/const_context.rs
#[test]
fn test_const_in_function() {
    // Should skip tracking in const context
}
```

---

## Expected Impact After Fixes

| Project | Current Pass Rate | Expected After Fixes |
|---------|-------------------|---------------------|
| ripgrep | 99.2% | ~100% |
| tokio | 71.2% | ~95% |
| bat | ~85% | ~98% |
| fd | ~80% | ~98% |
| zoxide | ~70% | ~95% |

The primary blocker for tokio is ERR-012 (trait impl methods), which accounts for 54% of all errors. Fixing this single issue would raise tokio's pass rate from 71% to ~90%.
