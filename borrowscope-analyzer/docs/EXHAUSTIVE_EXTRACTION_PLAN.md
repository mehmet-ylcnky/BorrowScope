# Exhaustive Type Extraction Plan

## Objective

Call every available method on `hir::Type` (ra_ap_hir 0.0.318) for every variable binding, storing all results in `type-info.json`. This ensures we never miss information that rust-analyzer can provide.

## Currently Extracted (32 methods called)

- `ty.display()` - type string
- `ty.is_copy(db)` - Copy trait
- `ty.is_reference()` / `ty.is_mutable_reference()`
- `ty.is_raw_ptr()`
- `ty.is_slice()`
- `ty.is_closure()` / `ty.is_fn()`
- `ty.is_never()`
- `ty.is_unit()`
- `ty.is_tuple()` / `ty.is_array()`
- `ty.as_adt()` - ADT classification (Rc, Arc, Vec, etc.)
- `ty.as_builtin()` - primitive detection
- `ty.as_dyn_trait()` - dyn trait detection
- `ty.as_reference()` - inner type + mutability
- `ty.as_callable(db)` - callable detection
- `ty.contains_reference(db)`
- `ty.impls_fnonce(db)`
- `ty.impls_trait(db, ...)` - 8 specific traits checked
- `ty.type_arguments()` - generic parameters
- `ty.future_output(db)` / `ty.iterator_item(db)`

## Missing (23 methods NOT yet called)

### Simple booleans (add to VariableTypeInfo):
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.is_bool()` | `is_bool` | Distinguish bool from other scalars |
| `ty.is_usize()` | `is_usize` | Index type detection |
| `ty.is_float()` | `is_float` | Float vs integer distinction |
| `ty.is_char()` | `is_char` | Character type |
| `ty.is_int_or_uint()` | `is_integer` | Any integer type |
| `ty.is_scalar()` | `is_scalar` | All primitive scalars |
| `ty.is_packed(db)` | `is_packed` | repr(packed) detection |
| `ty.is_unknown()` | `is_unknown` | Type resolution failed |
| `ty.contains_unknown()` | `contains_unknown` | Partial resolution failure |

### Trait queries:
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.impls_iterator(db)` | `is_iterator` (exists but may not use this method) | Iterator trait |

### Structural decomposition:
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.tuple_fields(db)` | `tuple_field_types: Vec<String>` | Element types of tuples |
| `ty.fields(db)` | `struct_fields: Vec<{name, type}>` | All struct fields with types |
| `ty.as_array(db)` | `array_element_type: Option<String>`, `array_length: Option<usize>` | Array details |
| `ty.as_impl_traits(db)` | `impl_traits: Vec<String>` | Traits in `impl Trait` return |
| `ty.as_type_param(db)` | `is_type_param: bool`, `type_param_name: Option<String>` | Generic parameter info |
| `ty.as_slice()` | `slice_element_type: Option<String>` | Slice inner type |

### Chain/sequence data:
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.autoderef(db)` | `autoderef_chain: Vec<String>` | Full deref chain |
| `ty.strip_references()` | `stripped_type: String` | Type after removing all & |

### Layout information:
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.layout(db)` | `layout_size: Option<u64>`, `layout_align: Option<u64>` | Memory layout |
| `ty.drop_glue(db)` | `has_drop_glue: bool` | Whether drop code runs |

### Reference manipulation:
| Method | New Field | Purpose |
|--------|-----------|---------|
| `ty.remove_ref()` | (use for `inner_ref_type`) | Type inside & |
| `ty.remove_raw_ptr()` | (use for `inner_ptr_type`) | Type inside *const/*mut |

## Implementation Steps

1. Add new fields to `VariableTypeInfo` in `output.rs` (with `#[serde(default)]`)
2. Add extraction calls in `analysis.rs` where `ty` is available
3. Update schema version to 3.1
4. Update `04-OUTPUT-FORMAT.md` documentation
5. Run tests to verify no regressions
6. Run analyzer on macro-examples to verify JSON output

## Schema Version Bump

Current: 3.0
New: 3.1 (backward compatible - all new fields have defaults)

## JSON Size Impact

Estimated increase per variable: ~200-400 bytes (mostly from Vec fields like tuple_fields, struct_fields, autoderef_chain)
For a project with 500 variables: ~100-200 KB additional (acceptable)
