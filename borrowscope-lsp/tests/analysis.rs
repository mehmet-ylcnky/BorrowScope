//! Integration tests for the ownership analysis engine.
//!
//! These tests load a real workspace and verify type extraction.
//! They take 30-40s due to workspace loading, so they are marked #[ignore].
//! Run with: cargo test -p borrowscope-lsp --test analysis -- --ignored

use std::path::Path;

// Re-use the workspace loading from the LSP crate
fn load_test_workspace() -> (ra_ap_ide_db::RootDatabase, ra_ap_vfs::Vfs) {
    let fixture_path = Path::new("/tmp/bs-test-project");
    assert!(
        fixture_path.join("Cargo.toml").exists(),
        "Test fixture not found at /tmp/bs-test-project. Create it first."
    );

    let mut cargo_config = ra_ap_project_model::CargoConfig::default();
    cargo_config.sysroot = Some(ra_ap_project_model::RustLibSource::Discover);

    let load_config = ra_ap_load_cargo::LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ra_ap_load_cargo::ProcMacroServerChoice::None,
        prefill_caches: true,
        proc_macro_processes: 0,
    };

    let (db, vfs, _) =
        ra_ap_load_cargo::load_workspace_at(fixture_path, &cargo_config, &load_config, &|_| {})
            .expect("Failed to load test workspace");

    (db, vfs)
}

/// Find a variable by name in the fixture's main.rs and extract its type info.
fn extract_var_info(
    db: &ra_ap_ide_db::RootDatabase,
    vfs: &ra_ap_vfs::Vfs,
    var_name: &str,
) -> Option<borrowscope_lsp::analysis::VariableOwnershipInfo> {
    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode, Edition};
    use ra_ap_vfs::VfsPath;

    let sema = Semantics::new(db);

    // Find main.rs in VFS
    let main_path = VfsPath::new_real_path("/tmp/bs-test-project/src/main.rs".to_string());
    let (file_id, _) = vfs.file_id(&main_path)?;

    let source_file = sema.parse(sema.attach_first_edition(file_id));

    // Get display target
    let display_target = hir::Crate::all(db)
        .first()
        .map(|k| DisplayTarget::from_crate(db, (*k).into()))
        .unwrap();

    // Walk let statements to find the variable - wrapped in attach_db for TLS
    attach_db(db, || {
        for node in source_file.syntax().descendants() {
            if let Some(let_stmt) = ast::LetStmt::cast(node.clone()) {
                if let Some(pat) = let_stmt.pat() {
                    let pat_text = pat.syntax().text().to_string();
                    if pat_text.trim() == var_name {
                        if let Some(ty_info) = sema.type_of_pat(&pat) {
                            let ty = ty_info.original;
                            let info = borrowscope_lsp::analysis::extract_full_type_info(
                                db,
                                &display_target,
                                &ty,
                                var_name,
                                "src/main.rs",
                                0,
                                0,
                                Some("main"),
                            );
                            return Some(info);
                        }
                    }
                }
            }
        }
        None
    })
}

#[test]
#[ignore] // Takes 30-40s (workspace loading)
fn test_i32_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "x").expect("Variable 'x' not found");

    assert_eq!(info.name, "x");
    assert!(info.is_copy, "i32 should be Copy");
    assert!(info.is_scalar, "i32 should be scalar");
    assert!(info.is_int_or_uint, "i32 should be int_or_uint");
    assert!(!info.is_reference);
    assert!(!info.is_unknown);
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::Copy);
}

#[test]
#[ignore]
fn test_vec_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "v").expect("Variable 'v' not found");

    assert_eq!(info.name, "v");
    assert!(!info.is_copy, "Vec should not be Copy");
    assert!(info.has_drop_glue, "Vec should have drop glue");
    assert!(info.adt_info.is_some(), "Vec should have ADT info");
    assert_eq!(info.adt_info.as_ref().unwrap().kind, "struct");
    assert!(!info.type_arguments.is_empty(), "Vec<i32> should have type arguments");
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::Owned);
}

#[test]
#[ignore]
fn test_shared_reference() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "r").expect("Variable 'r' not found");

    assert!(info.is_reference, "Should be a reference");
    assert!(!info.is_mutable_reference, "Should not be mutable");
    assert!(info.reference_inner.is_some(), "Should have inner type");
    assert_eq!(info.reference_inner.as_ref().unwrap().mutability, "Shared");
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::SharedRef);
}

#[test]
#[ignore]
fn test_mutable_reference() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "m").expect("Variable 'm' not found");

    assert!(info.is_reference);
    assert!(info.is_mutable_reference);
    assert!(info.reference_inner.is_some());
    assert_eq!(info.reference_inner.as_ref().unwrap().mutability, "Mut");
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::MutableRef);
}

#[test]
#[ignore]
fn test_rc_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "rc").expect("Variable 'rc' not found");

    assert!(!info.is_copy);
    assert!(info.adt_canonical_path.is_some(), "Should have ADT path, got: {:?}", info.adt_info);
    let path = info.adt_canonical_path.as_ref().unwrap();
    assert!(path.contains("rc") || path.contains("Rc"), "Path should contain Rc, got: {}", path);
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::SharedOwnership,
        "Expected SharedOwnership, path was: {}", path);
}

#[test]
#[ignore]
fn test_refcell_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "cell").expect("Variable 'cell' not found");

    assert!(!info.is_copy);
    assert!(info.adt_canonical_path.is_some(), "Should have ADT path, got: {:?}", info.adt_info);
    let path = info.adt_canonical_path.as_ref().unwrap();
    assert!(path.to_lowercase().contains("refcell") || path.to_lowercase().contains("cell"),
        "Path should contain RefCell, got: {}", path);
    assert_eq!(info.ownership_category, borrowscope_lsp::analysis::OwnershipCategory::InteriorMut,
        "Expected InteriorMut, path was: {}", path);
}

#[test]
#[ignore]
fn test_array_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "arr").expect("Variable 'arr' not found");

    assert!(info.is_array, "Should be an array");
    assert!(info.is_copy, "[u8; 32] should be Copy");
    assert!(info.array_info.is_some());
    let arr = info.array_info.as_ref().unwrap();
    assert_eq!(arr.length, 32);
    assert!(arr.element_type.contains("u8"));
}

#[test]
#[ignore]
fn test_tuple_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "tup").expect("Variable 'tup' not found");

    assert!(info.is_tuple, "Should be a tuple");
    assert!(!info.tuple_fields.is_empty(), "Should have tuple fields");
    assert_eq!(info.tuple_fields.len(), 2);
}

#[test]
#[ignore]
fn test_closure_variable() {
    let (db, vfs) = load_test_workspace();
    let info = extract_var_info(&db, &vfs, "closure").expect("Variable 'closure' not found");

    assert!(info.is_closure, "Should be a closure");
    assert!(info.impls_fnonce, "Closures implement FnOnce");
    assert!(info.callable_info.is_some(), "Should be callable");
    let callable = info.callable_info.as_ref().unwrap();
    assert!(callable.is_closure);
    assert!(!callable.params.is_empty());
}
