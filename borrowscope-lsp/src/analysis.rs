//! Ownership analysis engine.
//!
//! Extracts exhaustive type information from the Salsa database
//! using all available hir::Type methods.

use ra_ap_hir::{self as hir, HirDisplay, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::Edition;
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════════════════
// Data structures
// ═══════════════════════════════════════════════════════════════════════════

/// Complete ownership information for a single variable.
#[derive(Debug, Clone, Serialize, Default)]
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

    // ADT classification
    pub adt_canonical_path: Option<String>,
    pub ownership_category: OwnershipCategory,

    // Trait implementations
    pub trait_impls: TraitImplInfo,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TypeDecomposition {
    pub inner_type: String,
    pub mutability: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AdtInfo {
    pub kind: String,
    pub name: String,
    pub canonical_path: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ArrayInfo {
    pub element_type: String,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CallableInfo {
    pub params: Vec<String>,
    pub return_type: String,
    pub is_closure: bool,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum OwnershipCategory {
    Owned,
    SharedRef,
    MutableRef,
    SharedOwnership,
    InteriorMut,
    RawPointer,
    Copy,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TraitImplInfo {
    pub is_send: bool,
    pub is_sync: bool,
    pub is_clone: bool,
    pub is_drop: bool,
    pub is_sized: bool,
    pub is_debug: bool,
    pub is_display: bool,
    pub is_default: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Extraction
// ═══════════════════════════════════════════════════════════════════════════

/// Extract all ownership info for a single type.
pub fn extract_full_type_info(
    db: &RootDatabase,
    display_target: &hir::DisplayTarget,
    ty: &hir::Type<'_>,
    name: &str,
    file: &str,
    line: u32,
    column: u32,
    function_name: Option<&str>,
) -> VariableOwnershipInfo {
    let mut info = VariableOwnershipInfo {
        name: name.to_string(),
        type_display: ty.display(db, *display_target).to_string(),
        file: file.to_string(),
        line,
        column,
        function_name: function_name.map(|s| s.to_string()),
        ..Default::default()
    };

    // ── Boolean queries (no db) ──
    info.is_unit = ty.is_unit();
    info.is_bool = ty.is_bool();
    info.is_str = ty.is_str();
    info.is_never = ty.is_never();
    info.is_reference = ty.is_reference();
    info.is_mutable_reference = ty.is_mutable_reference();
    info.is_slice = ty.is_slice();
    info.is_usize = ty.is_usize();
    info.is_float = ty.is_float();
    info.is_char = ty.is_char();
    info.is_int_or_uint = ty.is_int_or_uint();
    info.is_scalar = ty.is_scalar();
    info.is_tuple = ty.is_tuple();
    info.is_array = ty.is_array();
    info.is_closure = ty.is_closure();
    info.is_fn = ty.is_fn();
    info.is_raw_ptr = ty.is_raw_ptr();
    info.is_unknown = ty.is_unknown();
    info.contains_unknown = ty.contains_unknown();

    // ── Queries requiring db ──
    info.is_copy = ty.is_copy(db);
    info.is_packed = ty.is_packed(db);
    info.contains_reference = ty.contains_reference(db);
    info.impls_fnonce = ty.impls_fnonce(db);
    info.impls_iterator = ty.clone().impls_iterator(db);

    // ── Reference decomposition ──
    if let Some((inner, mutability)) = ty.as_reference() {
        info.reference_inner = Some(TypeDecomposition {
            inner_type: inner.display(db, *display_target).to_string(),
            mutability: format!("{:?}", mutability),
        });
    }

    // ── ADT info ──
    if let Some(adt) = ty.as_adt() {
        let kind = match adt {
            hir::Adt::Struct(_) => "struct",
            hir::Adt::Enum(_) => "enum",
            hir::Adt::Union(_) => "union",
        };
        let module = adt.module(db);
        let canonical_path = module.path_to_root(db).iter().rev().filter_map(|m| m.name(db)).map(|n| n.display_no_db(Edition::Edition2021).to_string()).collect::<Vec<_>>().join("::");
        let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();

        info.adt_info = Some(AdtInfo {
            kind: kind.to_string(),
            name: adt_name.clone(),
            canonical_path: format!("{}::{}", canonical_path, adt_name),
        });
        info.adt_canonical_path = Some(format!("{}::{}", canonical_path, adt_name));
    }

    // ── Builtin type ──
    if let Some(builtin) = ty.as_builtin() {
        info.builtin_type = Some(builtin.name().display_no_db(Edition::Edition2021).to_string());
    }

    // ── Dyn trait ──
    if let Some(trait_) = ty.as_dyn_trait() {
        info.dyn_trait = Some(trait_.name(db).display_no_db(Edition::Edition2021).to_string());
    }

    // ── Impl traits ──
    if let Some(traits) = ty.as_impl_traits(db) {
        info.impl_traits = traits.map(|t| t.name(db).display_no_db(Edition::Edition2021).to_string()).collect();
    }

    // ── Type arguments ──
    info.type_arguments = ty.type_arguments().map(|t| t.display(db, *display_target).to_string()).collect();

    // ── Future output ──
    info.future_output = ty.clone().future_output(db).map(|t| t.display(db, *display_target).to_string());

    // ── Iterator item ──
    info.iterator_item = ty.clone().iterator_item(db).map(|t| t.display(db, *display_target).to_string());

    // ── Tuple fields ──
    info.tuple_fields = ty.tuple_fields(db).iter().map(|t| t.display(db, *display_target).to_string()).collect();

    // ── Struct fields ──
    info.struct_fields = ty
        .fields(db)
        .iter()
        .map(|(field, field_ty)| FieldInfo {
            name: field.name(db).display_no_db(Edition::Edition2021).to_string(),
            ty: field_ty.display(db, *display_target).to_string(),
        })
        .collect();

    // ── Array info ──
    if let Some((elem_ty, length)) = ty.as_array(db) {
        info.array_info = Some(ArrayInfo {
            element_type: elem_ty.display(db, *display_target).to_string(),
            length,
        });
    }

    // ── Autoderef chain ──
    info.autoderef_chain = ty
        .autoderef(db)
        .map(|t| t.display(db, *display_target).to_string())
        .collect();

    // ── Callable info ──
    if let Some(callable) = ty.as_callable(db) {
        info.callable_info = Some(CallableInfo {
            params: callable
                .params()
                .iter()
                .map(|p| p.ty().display(db, *display_target).to_string())
                .collect(),
            return_type: callable.return_type().display(db, *display_target).to_string(),
            is_closure: ty.is_closure(),
        });
    }

    // ── Layout ──
    if let Ok(layout) = ty.layout(db) {
        info.layout_size = Some(layout.size());
        info.layout_align = Some(layout.align());
    }

    // ── Drop glue ──
    info.has_drop_glue = !matches!(ty.drop_glue(db), hir::DropGlue::None);

    // ── Ownership category ──
    info.ownership_category = classify_ownership(db, ty, &info);

    // ── Trait impls ──
    info.trait_impls = check_traits(db, ty);

    info
}

// ═══════════════════════════════════════════════════════════════════════════
// Classification
// ═══════════════════════════════════════════════════════════════════════════

fn classify_ownership(
    db: &RootDatabase,
    ty: &hir::Type<'_>,
    info: &VariableOwnershipInfo,
) -> OwnershipCategory {
    if info.is_unknown {
        return OwnershipCategory::Unknown;
    }
    if info.is_raw_ptr {
        return OwnershipCategory::RawPointer;
    }
    if info.is_mutable_reference {
        return OwnershipCategory::MutableRef;
    }
    if info.is_reference {
        return OwnershipCategory::SharedRef;
    }
    if info.is_copy {
        return OwnershipCategory::Copy;
    }

    // Check ADT path for smart pointers
    if let Some(ref path) = info.adt_canonical_path {
        let p = path.to_lowercase();
        if p.contains("rc::rc") || p.contains("sync::arc") || p == "rc" || p.ends_with("::rc") {
            return OwnershipCategory::SharedOwnership;
        }
        if p.contains("cell::refcell")
            || p.contains("cell::cell")
            || p.contains("sync::mutex")
            || p.contains("sync::rwlock")
        {
            return OwnershipCategory::InteriorMut;
        }
    }

    OwnershipCategory::Owned
}

fn check_traits(db: &RootDatabase, ty: &hir::Type<'_>) -> TraitImplInfo {
    // Note: checking specific traits requires looking them up by name.
    // For now, we use the available methods. Full trait checking
    // will be expanded when we have trait lookup infrastructure.
    TraitImplInfo {
        is_send: false, // Requires trait lookup (Send is not a lang item)
        is_sync: false, // Requires trait lookup
        is_clone: false, // Requires trait lookup
        is_drop: false, // Requires trait lookup
        is_sized: true, // Most types are sized
        is_debug: false,
        is_display: false,
        is_default: false,
    }
}
