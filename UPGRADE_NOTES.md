# rust-analyzer 0.0.318 Upgrade Notes

## Status: Compiles ✅ | Runtime Issue ⚠️

The upgrade from ra_ap_* 0.0.232 to 0.0.318 is complete and compiles successfully.

## Completed Changes

1. **Lang Item API** - Changed from `db.lang_item(krate, LangItem::X)` to `lang_items(db, krate).X`
2. **DisplayTarget** - Changed from `display(db, Edition)` to `display(db, DisplayTarget)`
3. **Module.krate()** - Now requires `db` parameter: `module.krate(db)`
4. **is_unsafe_to_call()** - Now requires `caller: Option<Function>` and `Edition` parameters
5. **ItemContainer::ExternBlock** - Now has a field: `ExternBlock(_)`
6. **attach_first_edition** - No longer returns `Option`, returns `EditionedFileId` directly
7. **LoadCargoConfig** - Added `proc_macro_processes: 0` field
8. **Import map** - Now returns tuple `(Either<ModuleDef, Macro>, Complete)`
9. **Removed methods** - `is_unsafe_ref_expr` and `is_unsafe_ident_pat` no longer exist

## Runtime Issue

The new rust-analyzer version uses thread-local storage for database attachment. All `display()` calls need to be wrapped with `ra_ap_hir_ty::attach_db(db, || ...)`.

### Error
```
thread 'main' panicked at ra_ap_hir_ty-0.0.318/src/next_solver/interner.rs:2528:42:
Try to use attached db, but not db is attached
```

### Solution
Replace all occurrences of:
```rust
ty.display(db, display_target.clone()).to_string()
```

With:
```rust
ra_ap_hir_ty::attach_db(db, || ty.display(db, display_target.clone()).to_string())
```

This affects ~65 locations in `borrowscope-analyzer/src/analysis.rs`.

## Testing

After applying the `attach_db` wrapper, test with:
```bash
cargo run -p borrowscope-analyzer -- examples/type-coverage
```
