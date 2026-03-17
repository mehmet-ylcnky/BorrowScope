# How to determine raw pointer mutability (*mut vs *const) from ra_ap_hir::Type?

## Tags: rust, rust-analyzer, static-analysis, raw-pointers

---

I'm building a static analysis tool using the `ra_ap_hir` crate (rust-analyzer's high-level IR) version `0.0.318`. I need to determine whether a resolved type is `*mut T` or `*const T`.

### The problem

`ra_ap_hir::Type` provides `is_raw_ptr()` to check if a type is a raw pointer, but there's no `is_mutable_raw_ptr()` method to distinguish `*mut` from `*const`. This is asymmetric with references, where both `is_reference()` and `is_mutable_reference()` exist:

```rust
// These exist:
pub fn is_reference(&self) -> bool;
pub fn is_mutable_reference(&self) -> bool;  // checks TyKind::Ref(Mutability::Mut, ..)
pub fn as_reference(&self) -> Option<(Type, Mutability)>;

// This exists:
pub fn is_raw_ptr(&self) -> bool;  // checks TyKind::RawPtr(..)

// These do NOT exist:
// pub fn is_mutable_raw_ptr(&self) -> bool;
// pub fn as_raw_ptr(&self) -> Option<(Type, Mutability)>;
```

### What I've investigated

Looking at the `ra_ap_hir` source, the internal representation clearly has the mutability information:

```rust
// Inside ra_ap_hir (pub(crate) — not accessible from outside)
pub fn is_raw_ptr(&self) -> bool {
    matches!(self.ty.kind(), TyKind::RawPtr(..))
}

pub fn remove_raw_ptr(&self) -> Option<Type<'db>> {
    if let TyKind::RawPtr(ty, _) = self.ty.kind() {  // second field is Mutability
        Some(self.derived(ty))
    } else { None }
}

// For comparison, is_mutable_reference accesses the same kind of data:
pub fn is_mutable_reference(&self) -> bool {
    matches!(self.ty.kind(), TyKind::Ref(.., hir_ty::next_solver::Mutability::Mut))
}
```

The `TyKind::RawPtr(Ty, Mutability)` variant holds the mutability, but `Type.ty` is `pub(crate)` so I can't access `ty.kind()` from outside the crate.

### What I've tried

1. **`remove_raw_ptr()`** — strips the pointer and returns the inner type, but discards the mutability.

2. **`as_reference()`** — only works for `&T`/`&mut T`, not raw pointers.

3. **`ra_ap_hir_ty::attach_db`** — sets a thread-local DB context but doesn't provide access to `Type`'s private `ty` field.

4. **`Type::walk()`** — visits inner types but doesn't expose `TyKind` variants.

### Current workaround

I'm currently reading the display output of the semantically-resolved type:

```rust
var_info.is_raw_ptr = ty.is_raw_ptr();
// HEURISTIC: no public API for raw pointer mutability
var_info.is_mutable_raw_ptr = ty.is_raw_ptr() && var_info.ty.starts_with("*mut ");
```

where `var_info.ty` is set via `ty.display(db, display_target).to_string()`. This works but feels wrong — I'm parsing a display string to recover information that the type system already has.

### Question

Is there a proper way to determine raw pointer mutability through the `ra_ap_hir` public API that I'm missing? Or is this a gap in the API that would need an upstream addition (like adding `as_raw_ptr() -> Option<(Type, Mutability)>` to mirror `as_reference()`)?

### Environment

- `ra_ap_hir` version: `0.0.318`
- `ra_ap_hir_ty` version: `0.0.318`
- `ra_ap_hir_def` version: `0.0.318`
