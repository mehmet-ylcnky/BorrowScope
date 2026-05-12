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

/// Test method call resolution (step 2.2).
#[test]
#[ignore]
fn test_method_call_resolution() {
    use borrowscope_lsp::analysis::{resolve_method_calls, SelfBorrow};
    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};
    use ra_ap_syntax::ast::HasName;
    use ra_ap_vfs::VfsPath;

    let (db, vfs) = load_workspace();

    let sema = Semantics::new(&db);
    let main_path = VfsPath::new_real_path("/tmp/bs-test-project/src/main.rs".to_string());
    let (file_id, _) = vfs.file_id(&main_path).unwrap();
    let source_file = sema.parse(sema.attach_first_edition(file_id));

    let display_target = hir::Crate::all(&db)
        .first()
        .map(|k| DisplayTarget::from_crate(&db, (*k).into()))
        .unwrap();

    let results = attach_db(&db, || {
        // Find the main function's body
        let main_fn = source_file
            .syntax()
            .descendants()
            .filter_map(ast::Fn::cast)
            .find(|f| f.name().map(|n| n.text() == "main").unwrap_or(false))
            .expect("main fn not found");

        let body = main_fn.body().expect("main has no body");
        resolve_method_calls(&db, &sema, &display_target, &body)
    });

    assert!(!results.is_empty(), "Should have resolved method calls");

    // ── vec.len(): self_borrow=Shared ──
    let len_call = results.iter().find(|r| r.method_name == "len");
    assert!(len_call.is_some(), "Should find len() call. Found: {:?}",
        results.iter().map(|r| &r.method_name).collect::<Vec<_>>());
    let len_call = len_call.unwrap();
    assert_eq!(len_call.self_borrow, SelfBorrow::Shared);

    // ── vec.push(99): self_borrow=Exclusive ──
    let push_call = results.iter().find(|r| r.method_name == "push");
    assert!(push_call.is_some(), "Should find push() call");
    let push_call = push_call.unwrap();
    assert_eq!(push_call.self_borrow, SelfBorrow::Exclusive);

    // ── rc.clone(): is_trait_method=true, trait_name=Clone ──
    let clone_call = results.iter().find(|r| r.method_name == "clone");
    assert!(clone_call.is_some(), "Should find clone() call");
    let clone_call = clone_call.unwrap();
    assert!(clone_call.is_trait_method, "clone should be a trait method");
    assert_eq!(clone_call.trait_name.as_deref(), Some("Clone"));

    // ── s.into_bytes(): self_borrow=Owned (consuming) ──
    let into_bytes = results.iter().find(|r| r.method_name == "into_bytes");
    assert!(into_bytes.is_some(), "Should find into_bytes() call");
    let into_bytes = into_bytes.unwrap();
    assert_eq!(into_bytes.self_borrow, SelfBorrow::Owned);

    // ── mtx.lock(): return type contains MutexGuard or Result ──
    let lock_call = results.iter().find(|r| r.method_name == "lock");
    assert!(lock_call.is_some(), "Should find lock() call");
    let lock_call = lock_call.unwrap();
    assert!(
        lock_call.return_type.contains("Result") || lock_call.return_type.contains("MutexGuard"),
        "lock() return type should contain Result or MutexGuard, got: {}",
        lock_call.return_type
    );

    // ── Unresolvable methods are skipped (no panic) ──
    // This is implicitly tested: if any method panicked, the test would fail

    println!("All method call resolution assertions passed! ({} calls resolved)", results.len());
}

/// Test borrow scope computation (step 2.3).
#[test]
#[ignore]
fn test_borrow_scope_computation() {
    use borrowscope_lsp::analysis::{compute_borrow_scopes, BorrowScopeInfo};
    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode, TextSize};
    use ra_ap_syntax::ast::HasName;
    use ra_ap_vfs::VfsPath;

    let (db, vfs) = load_workspace();

    let sema = Semantics::new(&db);
    let main_path = VfsPath::new_real_path("/tmp/bs-test-project/src/main.rs".to_string());
    let (file_id, _) = vfs.file_id(&main_path).unwrap();
    let source_file = sema.parse(sema.attach_first_edition(file_id));

    // Get the file text for line index computation
    let file_text = std::fs::read_to_string("/tmp/bs-test-project/src/main.rs").unwrap();
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(file_text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    let line_index = |offset: TextSize| -> (u32, u32) {
        let offset = u32::from(offset) as usize;
        let line = line_starts.partition_point(|&start| start <= offset) as u32;
        let col = offset - line_starts.get((line - 1) as usize).copied().unwrap_or(0);
        (line, col as u32)
    };

    let scopes = attach_db(&db, || {
        // Find the borrow_scopes_test function
        let test_fn = source_file
            .syntax()
            .descendants()
            .filter_map(ast::Fn::cast)
            .find(|f| f.name().map(|n| n.text() == "borrow_scopes_test").unwrap_or(false))
            .expect("borrow_scopes_test fn not found");

        compute_borrow_scopes(&db, &sema, &test_fn, &line_index)
    });

    assert!(!scopes.is_empty(), "Should find borrow scopes. Got 0.");

    // ── Simple borrow: r1 borrows data ──
    let r1_scope = scopes.iter().find(|s| s.borrower_name == "r1");
    assert!(r1_scope.is_some(), "Should find r1 scope. Found: {:?}",
        scopes.iter().map(|s| &s.borrower_name).collect::<Vec<_>>());
    let r1 = r1_scope.unwrap();
    assert_eq!(r1.target_name, "data");
    assert!(!r1.is_mutable);
    assert!(r1.end_line > r1.start_line, "r1 scope should span multiple lines (used after creation)");

    // ── Mutable borrow: m1 borrows data2 ──
    let m1_scope = scopes.iter().find(|s| s.borrower_name == "m1");
    assert!(m1_scope.is_some(), "Should find m1 scope");
    let m1 = m1_scope.unwrap();
    assert!(m1.is_mutable, "m1 should be a mutable borrow");
    assert!(m1.end_line > m1.start_line, "m1 scope should span to push() usage");

    // ── Multiple borrows of same variable ──
    let r2_scope = scopes.iter().find(|s| s.borrower_name == "r2");
    let r3_scope = scopes.iter().find(|s| s.borrower_name == "r3");
    assert!(r2_scope.is_some(), "Should find r2 scope");
    assert!(r3_scope.is_some(), "Should find r3 scope");
    let r2 = r2_scope.unwrap();
    let r3 = r3_scope.unwrap();
    assert_eq!(r2.target_name, "data");
    assert_eq!(r3.target_name, "data");
    // Both should have the same end line (used in same println!)
    assert_eq!(r2.end_line, r3.end_line);

    // ── Borrow with no uses: scope is just the declaration ──
    let unused = scopes.iter().find(|s| s.borrower_name == "_unused_ref");
    assert!(unused.is_some(), "Should find _unused_ref scope");
    let unused = unused.unwrap();
    assert_eq!(unused.start_line, unused.end_line,
        "Unused borrow scope should be single line (start={}, end={})",
        unused.start_line, unused.end_line);

    // ── Borrow in a block: scope ends within the block ──
    let block_ref = scopes.iter().find(|s| s.borrower_name == "block_ref");
    assert!(block_ref.is_some(), "Should find block_ref scope. Found: {:?}",
        scopes.iter().map(|s| &s.borrower_name).collect::<Vec<_>>());
    let block_ref = block_ref.unwrap();
    assert!(!block_ref.is_mutable);
    assert_eq!(block_ref.target_name, "data");
    // block_ref is used on the line after its creation (println)
    assert!(block_ref.end_line >= block_ref.start_line);

    // ── Borrow passed to function: scope extends to the function call ──
    let func_ref = scopes.iter().find(|s| s.borrower_name == "func_ref");
    assert!(func_ref.is_some(), "Should find func_ref scope");
    let func_ref = func_ref.unwrap();
    assert!(!func_ref.is_mutable);
    assert_eq!(func_ref.target_name, "data");
    // func_ref is used in consume_ref() call, so end > start
    assert!(func_ref.end_line > func_ref.start_line,
        "func_ref scope should extend to function call (start={}, end={})",
        func_ref.start_line, func_ref.end_line);

    println!("All borrow scope assertions passed! ({} scopes found)", scopes.len());
}
