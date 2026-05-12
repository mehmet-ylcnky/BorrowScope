//! Integration tests for the ownership analysis engine.
//!
//! Loads the workspace ONCE and runs all type extraction assertions.
//! Run with: cargo test -p borrowscope-lsp --test analysis -- --ignored
//!
//! Requires fixture at /tmp/bs-test-project (created by test setup).

use std::path::Path;

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

    let main_path = VfsPath::new_real_path("/tmp/bs-test-project/src/main.rs".to_string());
    let (file_id, _) = vfs.file_id(&main_path)?;

    let source_file = sema.parse(sema.attach_first_edition(file_id));

    let display_target = hir::Crate::all(db)
        .first()
        .map(|k| DisplayTarget::from_crate(db, (*k).into()))
        .unwrap();

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

fn load_workspace() -> (ra_ap_ide_db::RootDatabase, ra_ap_vfs::Vfs) {
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

    ra_ap_load_cargo::load_workspace_at(fixture_path, &cargo_config, &load_config, &|_| {})
        .map(|(db, vfs, _)| (db, vfs))
        .expect("Failed to load test workspace")
}

/// Single test that loads workspace once and verifies all type extractions.
#[test]
#[ignore]
fn test_exhaustive_type_extraction() {
    use borrowscope_lsp::analysis::OwnershipCategory;

    let (db, vfs) = load_workspace();

    // ── i32: Copy scalar ──
    let info = extract_var_info(&db, &vfs, "x").expect("'x' not found");
    assert_eq!(info.name, "x");
    assert!(info.is_copy, "i32 should be Copy");
    assert!(info.is_scalar, "i32 should be scalar");
    assert!(info.is_int_or_uint, "i32 should be int_or_uint");
    assert!(!info.is_reference);
    assert!(!info.is_unknown);
    assert_eq!(info.ownership_category, OwnershipCategory::Copy);

    // ── Vec<i32>: Owned with drop glue ──
    let info = extract_var_info(&db, &vfs, "v").expect("'v' not found");
    assert!(!info.is_copy, "Vec should not be Copy");
    assert!(info.has_drop_glue, "Vec should have drop glue");
    assert!(info.adt_info.is_some(), "Vec should have ADT info");
    assert_eq!(info.adt_info.as_ref().unwrap().kind, "struct");
    assert!(!info.type_arguments.is_empty(), "Vec<i32> should have type args");
    assert_eq!(info.ownership_category, OwnershipCategory::Owned);

    // ── &Vec<i32>: Shared reference ──
    let info = extract_var_info(&db, &vfs, "r").expect("'r' not found");
    assert!(info.is_reference);
    assert!(!info.is_mutable_reference);
    assert!(info.reference_inner.is_some());
    assert_eq!(info.reference_inner.as_ref().unwrap().mutability, "Shared");
    assert_eq!(info.ownership_category, OwnershipCategory::SharedRef);

    // ── &mut Vec<i32>: Mutable reference ──
    let info = extract_var_info(&db, &vfs, "m").expect("'m' not found");
    assert!(info.is_reference);
    assert!(info.is_mutable_reference);
    assert!(info.reference_inner.is_some());
    assert_eq!(info.reference_inner.as_ref().unwrap().mutability, "Mut");
    assert_eq!(info.ownership_category, OwnershipCategory::MutableRef);

    // ── Rc<String>: Shared ownership ──
    let info = extract_var_info(&db, &vfs, "rc").expect("'rc' not found");
    assert!(!info.is_copy);
    assert!(info.adt_canonical_path.is_some());
    let path = info.adt_canonical_path.as_ref().unwrap();
    assert!(
        path.to_lowercase().contains("rc"),
        "Path should contain Rc, got: {}", path
    );
    assert_eq!(info.ownership_category, OwnershipCategory::SharedOwnership);

    // ── RefCell<i32>: Interior mutability ──
    let info = extract_var_info(&db, &vfs, "cell").expect("'cell' not found");
    assert!(!info.is_copy);
    assert!(info.adt_canonical_path.is_some());
    let path = info.adt_canonical_path.as_ref().unwrap();
    assert!(
        path.to_lowercase().contains("refcell") || path.to_lowercase().contains("cell"),
        "Path should contain RefCell, got: {}", path
    );
    assert_eq!(info.ownership_category, OwnershipCategory::InteriorMut);

    // ── [u8; 32]: Array ──
    let info = extract_var_info(&db, &vfs, "arr").expect("'arr' not found");
    assert!(info.is_array);
    assert!(info.is_copy, "[u8; 32] should be Copy");
    assert!(info.array_info.is_some());
    let arr = info.array_info.as_ref().unwrap();
    assert_eq!(arr.length, 32);

    // ── (i32, String): Tuple ──
    let info = extract_var_info(&db, &vfs, "tup").expect("'tup' not found");
    assert!(info.is_tuple);
    assert!(!info.tuple_fields.is_empty());
    assert_eq!(info.tuple_fields.len(), 2);

    // ── Closure ──
    let info = extract_var_info(&db, &vfs, "closure").expect("'closure' not found");
    assert!(info.is_closure);
    assert!(info.impls_fnonce, "Closures implement FnOnce");
    assert!(info.callable_info.is_some());
    assert!(info.callable_info.as_ref().unwrap().is_closure);

    // ── Struct with fields ──
    let info = extract_var_info(&db, &vfs, "point").expect("'point' not found");
    assert!(!info.struct_fields.is_empty(), "Should have struct fields");
    assert_eq!(info.struct_fields.len(), 2);
    let names: Vec<&str> = info.struct_fields.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"x"));
    assert!(names.contains(&"y"));

    // ── Future (async fn return) ──
    let info = extract_var_info(&db, &vfs, "fut").expect("'fut' not found");
    assert!(
        info.future_output.is_some() || !info.impl_traits.is_empty(),
        "Future should have future_output or impl_traits"
    );

    println!("All 11 type extraction assertions passed!");
}
