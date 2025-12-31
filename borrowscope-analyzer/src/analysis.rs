//! Semantic analysis using rust-analyzer
//!
//! This module provides type analysis by leveraging rust-analyzer's
//! full semantic analysis capabilities. No heuristics are used.

use crate::output::{ProjectTypeInfo, VariableTypeInfo};
use anyhow::{Context, Result};
use ra_ap_hir::{db::DefDatabase, HirDisplay, LangItem, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::CargoConfig;
use ra_ap_syntax::{ast, AstNode, Edition, SyntaxKind};
use ra_ap_syntax::ast::HasName;
use std::path::Path;
use tracing::{info, warn};

/// Analyze a Rust project and extract type information for all variables
pub fn analyze_project(project_path: &Path) -> Result<ProjectTypeInfo> {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found at {}", project_path.display());
    }

    info!("Loading workspace...");

    let mut cargo_config = CargoConfig::default();
    cargo_config.sysroot = Some(ra_ap_project_model::RustLibSource::Discover);

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: true,
    };

    let (db, vfs, _proc_macro) = load_workspace_at(
        project_path,
        &cargo_config,
        &load_config,
        &|_msg| {},
    )
    .context("Failed to load workspace")?;

    info!("Workspace loaded, analyzing files...");

    let project_abs = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    let mut info = ProjectTypeInfo::new();
    let sema = Semantics::new(&db);

    for (file_id, vfs_path) in vfs.iter() {
        let path_str = match vfs_path.as_path() {
            Some(p) => p.to_string(),
            None => continue,
        };

        if !path_str.ends_with(".rs") {
            continue;
        }

        if path_str.contains("/.cargo/")
            || path_str.contains("/rustup/")
            || path_str.contains("\\registry\\")
        {
            continue;
        }

        let project_prefix = project_abs.to_string_lossy();
        if !path_str.starts_with(project_prefix.as_ref()) {
            continue;
        }

        let relative = path_str
            .strip_prefix(project_prefix.as_ref())
            .unwrap_or(&path_str)
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .to_string();

        if relative.starts_with("target") {
            continue;
        }

        println!("  Analyzing: {}", relative);

        let variables = analyze_file(&sema, &db, file_id, &relative);
        if !variables.is_empty() {
            info.files.insert(relative, variables);
        }
    }

    Ok(info)
}

/// Analyze a single file using semantic analysis only
fn analyze_file(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    file_id: ra_ap_vfs::FileId,
    relative_path: &str,
) -> Vec<VariableTypeInfo> {
    let mut variables = Vec::new();
    let mut scope_id: u32 = 0;

    let Some(editioned_file_id) = sema.attach_first_edition(file_id) else {
        warn!("File {} not in crate graph, skipping", relative_path);
        return variables;
    };

    let source_file = sema.parse(editioned_file_id);

    for node in source_file.syntax().descendants() {
        match node.kind() {
            SyntaxKind::LET_STMT => {
                if let Some(var_info) = analyze_let_stmt(sema, db, &node, relative_path, &source_file, &mut scope_id) {
                    variables.push(var_info);
                    scope_id += 1;
                }
            }
            SyntaxKind::STATIC => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, &node, relative_path, &source_file) {
                    var_info.is_static = true;
                    variables.push(var_info);
                }
            }
            SyntaxKind::CONST => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, &node, relative_path, &source_file) {
                    var_info.is_const = true;
                    variables.push(var_info);
                }
            }
            _ => {}
        }
    }

    variables
}

/// Analyze a let statement
fn analyze_let_stmt(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    relative_path: &str,
    source_file: &ast::SourceFile,
    scope_id: &mut u32,
) -> Option<VariableTypeInfo> {
    let let_stmt = ast::LetStmt::cast(node.clone())?;
    let pat = let_stmt.pat()?;

    let range = pat.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    let name = pat.syntax().text().to_string();
    let mut var_info = VariableTypeInfo::new(name, relative_path.to_string(), line, column);

    // Set span offsets
    var_info.span_start = u32::from(range.start());
    var_info.span_end = u32::from(range.end());

    // Detect tuple binding pattern
    var_info.is_tuple_binding = matches!(&pat, ast::Pat::TuplePat(_));

    // Detect mut binding
    if let ast::Pat::IdentPat(ident_pat) = &pat {
        var_info.is_mut_binding = ident_pat.mut_token().is_some();
    }

    if let Some(type_info) = sema.type_of_pat(&pat) {
        populate_type_info(&mut var_info, &type_info.original, db);
    }

    // Detect impl Trait in type annotation
    if let Some(ty) = let_stmt.ty() {
        var_info.is_impl_trait = matches!(ty, ast::Type::ImplTraitType(_));
    }

    // Detect initializer kind
    if let Some(init) = let_stmt.initializer() {
        var_info.initializer_kind = Some(classify_initializer(&init));
    }

    // Assign scope ID (simple incrementing for now)
    var_info.scope_id = Some(*scope_id);

    Some(var_info)
}

/// Classify the initializer expression kind
fn classify_initializer(expr: &ast::Expr) -> String {
    match expr {
        ast::Expr::Literal(_) => "literal".to_string(),
        ast::Expr::CallExpr(_) => "call".to_string(),
        ast::Expr::MethodCallExpr(_) => "method".to_string(),
        ast::Expr::BlockExpr(_) => "block".to_string(),
        ast::Expr::IfExpr(_) => "if".to_string(),
        ast::Expr::MatchExpr(_) => "match".to_string(),
        ast::Expr::ClosureExpr(_) => "closure".to_string(),
        ast::Expr::RefExpr(_) => "ref".to_string(),
        ast::Expr::PathExpr(_) => "path".to_string(),
        ast::Expr::MacroExpr(_) => "macro".to_string(),
        ast::Expr::AwaitExpr(_) => "await".to_string(),
        ast::Expr::TryExpr(_) => "try".to_string(),
        _ => "other".to_string(),
    }
}

/// Analyze a static or const declaration
fn analyze_static_or_const(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    relative_path: &str,
    source_file: &ast::SourceFile,
) -> Option<VariableTypeInfo> {
    // Try to cast as Static first, then Const
    let (name_token, ty_node) = if let Some(static_item) = ast::Static::cast(node.clone()) {
        (static_item.name()?, static_item.ty())
    } else if let Some(const_item) = ast::Const::cast(node.clone()) {
        (const_item.name()?, const_item.ty())
    } else {
        return None;
    };

    let range = name_token.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    let name = name_token.text().to_string();
    let mut var_info = VariableTypeInfo::new(name, relative_path.to_string(), line, column);

    // Set span offsets
    var_info.span_start = u32::from(range.start());
    var_info.span_end = u32::from(range.end());

    // Try to resolve the type from the type annotation
    if let Some(ty_node) = ty_node {
        if let Some(ty) = sema.resolve_type(&ty_node) {
            populate_type_info(&mut var_info, &ty, db);
        } else {
            // Fallback: use the syntax text, no classification without semantic info
            var_info.ty = ty_node.syntax().text().to_string();
        }
    }

    Some(var_info)
}

/// Get line and column from a text range
fn get_location(range: &ra_ap_syntax::TextRange, source_file: &ast::SourceFile) -> (u32, u32) {
    let text_before = source_file
        .syntax()
        .text()
        .slice(..range.start())
        .to_string();
    let line = text_before.lines().count() as u32;
    let column = text_before.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
    (line, column)
}

/// Populate type info from a resolved type using semantic analysis only
fn populate_type_info(var_info: &mut VariableTypeInfo, ty: &ra_ap_hir::Type, db: &RootDatabase) {
    use ra_ap_hir::Adt;
    
    var_info.ty = ty.display(db, Edition::Edition2021).to_string();
    
    // Get krate for lang item lookups - prefer from ADT, fallback to first crate
    // Lang items are the same across all crates, so any krate works for lookup
    let krate = ty.as_adt()
        .map(|adt| adt.module(db).krate())
        .or_else(|| ra_ap_hir::Crate::all(db).first().copied());
    
    // === Core trait implementations (semantic) ===
    var_info.is_copy = ty.is_copy(db);
    
    if let Some(krate) = krate {
        let krate_id = krate.into();
        
        if let Some(clone_trait) = db.lang_item(krate_id, LangItem::Clone).and_then(|li| li.as_trait()) {
            var_info.is_clone = ty.impls_trait(db, clone_trait.into(), &[]);
        }
        if let Some(drop_trait) = db.lang_item(krate_id, LangItem::Drop).and_then(|li| li.as_trait()) {
            var_info.is_drop = ty.impls_trait(db, drop_trait.into(), &[]);
        }
        if let Some(sync_trait) = db.lang_item(krate_id, LangItem::Sync).and_then(|li| li.as_trait()) {
            var_info.is_sync = ty.impls_trait(db, sync_trait.into(), &[]);
        }
        if let Some(sized_trait) = db.lang_item(krate_id, LangItem::Sized).and_then(|li| li.as_trait()) {
            var_info.is_sized = ty.impls_trait(db, sized_trait.into(), &[]);
        }
        // Future trait
        if let Some(future_trait) = db.lang_item(krate_id, LangItem::Future).and_then(|li| li.as_trait()) {
            var_info.is_future = ty.impls_trait(db, future_trait.into(), &[]);
        }
        // Iterator trait  
        if let Some(iterator_trait) = db.lang_item(krate_id, LangItem::Iterator).and_then(|li| li.as_trait()) {
            var_info.is_iterator = ty.impls_trait(db, iterator_trait.into(), &[]);
        }
        
        // Send trait - not a lang item, must be found via import_map search
        if let Some(send_trait) = find_send_trait(db, krate) {
            var_info.is_send = ty.impls_trait(db, send_trait, &[]);
        }
    }
    
    // === Type structure (semantic via Type methods) ===
    var_info.is_reference = ty.is_reference();
    var_info.is_mutable_reference = ty.is_mutable_reference();
    var_info.is_raw_ptr = ty.is_raw_ptr();
    var_info.is_closure = ty.is_closure();
    var_info.is_fn_ptr = ty.is_fn();
    
    // Check for slice - either bare [T] or contained in reference/smart pointer
    var_info.is_slice = ty.is_slice() || ty.strip_reference().is_slice()
        || ty.type_arguments().any(|inner| inner.is_slice());
    
    // Primitive detection via builtin type
    if let Some(builtin) = ty.as_builtin() {
        var_info.is_primitive = builtin.is_int() || builtin.is_uint() || builtin.is_float() 
            || builtin.is_char() || builtin.is_bool() || builtin.is_str();
    }
    var_info.is_primitive = var_info.is_primitive || ty.is_unit();
    
    // str type (the unsized string slice type)
    if let Some(builtin) = ty.as_builtin() {
        var_info.is_str = builtin.is_str();
    }
    
    // === ADT-based classification (semantic via canonical path) ===
    if let Some(adt) = ty.as_adt() {
        var_info.is_union = matches!(adt, Adt::Union(_));
        
        if let Some(path) = get_adt_path(&adt, db) {
            classify_by_path(var_info, &path);
        }
    }
    
    // Check for dyn trait - either bare dyn Trait or contained in reference/smart pointer
    var_info.is_dyn_trait = ty.as_dyn_trait().is_some() 
        || ty.strip_reference().as_dyn_trait().is_some()
        || ty.type_arguments().any(|inner| inner.as_dyn_trait().is_some());
}

/// Find the Send trait by searching dependencies for core::marker::Send
fn find_send_trait(db: &RootDatabase, krate: ra_ap_hir::Crate) -> Option<ra_ap_hir::Trait> {
    use ra_ap_hir::{import_map, ModuleDef};
    
    // Helper to search a krate for Send trait
    let search_krate = |k: ra_ap_hir::Crate| -> Option<ra_ap_hir::Trait> {
        let query = import_map::Query::new("Send".to_string()).exact();
        for item in k.query_external_importables(db, query) {
            if let either::Either::Left(ModuleDef::Trait(t)) = item {
                let module = t.module(db);
                let module_name = module.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string());
                if module_name.as_deref() == Some("marker") {
                    return Some(t);
                }
            }
        }
        None
    };
    
    // Try given krate first
    if let Some(t) = search_krate(krate) {
        return Some(t);
    }
    
    // If not found, try all crates
    for other_krate in ra_ap_hir::Crate::all(db) {
        if let Some(t) = search_krate(other_krate) {
            return Some(t);
        }
    }
    None
}

/// Get the canonical path of an ADT (e.g., "std::rc::Rc", "std::vec::Vec")
fn get_adt_path(adt: &ra_ap_hir::Adt, db: &RootDatabase) -> Option<String> {
    let module = adt.module(db);
    let name = match adt {
        ra_ap_hir::Adt::Struct(s) => s.name(db),
        ra_ap_hir::Adt::Union(u) => u.name(db),
        ra_ap_hir::Adt::Enum(e) => e.name(db),
    };
    
    let mut segments: Vec<String> = module.path_to_root(db)
        .into_iter()
        .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
        .collect();
    segments.reverse();
    segments.push(name.display_no_db(Edition::Edition2021).to_string());
    
    // Get crate name
    let krate = module.krate();
    let crate_name = krate.display_name(db)
        .map(|n| n.to_string())
        .unwrap_or_default();
    
    if !crate_name.is_empty() {
        segments.insert(0, crate_name);
    }
    
    Some(segments.join("::"))
}

/// Classify type based on its canonical path (semantic)
fn classify_by_path(var_info: &mut VariableTypeInfo, path: &str) {
    // Smart pointers
    var_info.is_rc = path == "alloc::rc::Rc" || path == "std::rc::Rc";
    var_info.is_arc = path == "alloc::sync::Arc" || path == "std::sync::Arc";
    var_info.is_box = path == "alloc::boxed::Box" || path == "std::boxed::Box";
    var_info.is_weak = path == "alloc::rc::Weak" || path == "std::rc::Weak" 
        || path == "alloc::sync::Weak" || path == "std::sync::Weak";
    
    // Interior mutability
    var_info.is_refcell = path == "core::cell::RefCell" || path == "std::cell::RefCell";
    var_info.is_cell = path == "core::cell::Cell" || path == "std::cell::Cell";
    var_info.is_mutex = path == "std::sync::Mutex" || path == "std::sync::poison::mutex::Mutex";
    var_info.is_rwlock = path == "std::sync::RwLock" || path == "std::sync::poison::rwlock::RwLock";
    
    // Guards
    var_info.is_guard = path == "std::sync::MutexGuard" 
        || path == "std::sync::poison::mutex::MutexGuard"
        || path == "std::sync::RwLockReadGuard"
        || path == "std::sync::poison::rwlock::RwLockReadGuard"
        || path == "std::sync::RwLockWriteGuard"
        || path == "std::sync::poison::rwlock::RwLockWriteGuard"
        || path == "core::cell::Ref" || path == "std::cell::Ref"
        || path == "core::cell::RefMut" || path == "std::cell::RefMut";
    
    // Collections
    var_info.is_vec = path == "alloc::vec::Vec" || path == "std::vec::Vec";
    var_info.is_string = path == "alloc::string::String" || path == "std::string::String";
    
    // Wrapper types
    var_info.is_pin = path == "core::pin::Pin" || path == "std::pin::Pin";
    var_info.is_cow = path == "alloc::borrow::Cow" || path == "std::borrow::Cow";
    var_info.is_option = path == "core::option::Option" || path == "std::option::Option";
    var_info.is_result = path == "core::result::Result" || path == "std::result::Result";
    
    // FFI types
    var_info.is_extern_type = path == "core::ffi::c_void" || path == "std::ffi::c_void"
        || path == "core::ffi::CStr" || path == "std::ffi::CStr"
        || path == "alloc::ffi::CString" || path == "std::ffi::CString"
        || path == "std::ffi::OsStr" || path == "std::ffi::OsString";
}
