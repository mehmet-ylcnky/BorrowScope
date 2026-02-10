# rust-analyzer 0.0.318 Upgrade Notes

## Status: Complete ✅

The upgrade from ra_ap_* 0.0.232 to 0.0.318 is complete and fully functional.

## Changes Made

1. **Lang Item API** - `db.lang_item(krate, LangItem::X)` → `lang_items(db, krate).X`
2. **DisplayTarget** - `display(db, Edition)` → `display(db, DisplayTarget)`
3. **Module.krate()** - Now requires `db` parameter: `module.krate(db)`
4. **is_unsafe_to_call()** - Now requires `caller: Option<Function>` and `Edition` parameters
5. **ItemContainer::ExternBlock** - Now has a field: `ExternBlock(_)`
6. **attach_first_edition** - No longer returns `Option`, returns `EditionedFileId` directly
7. **LoadCargoConfig** - Added `proc_macro_processes: 0` field
8. **Import map** - Now returns tuple `(Either<ModuleDef, Macro>, Complete)`
9. **Removed methods** - `is_unsafe_ref_expr` and `is_unsafe_ident_pat` no longer exist

## Runtime Fix Applied

The new rust-analyzer version uses thread-local storage for database attachment. The fix wraps `analyze_file` with `attach_db`:

```rust
use ra_ap_hir_ty::attach_db;

let results = attach_db(&db, || {
    analyze_file(&sema, &db, &tracked_functions, &known_types, &known_macros, &display_target, file_id, &relative)
});
```

## New Capability: AsyncFn Detection

Version 0.0.318 enables detection of async closure traits:
- `AsyncFn`
- `AsyncFnMut`  
- `AsyncFnOnce`

## Performance

Total analysis time: ~3.9s (down from ~45-50s with optimizations)
