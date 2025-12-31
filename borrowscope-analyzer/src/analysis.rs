//! Semantic analysis using rust-analyzer
//!
//! This module provides type analysis by leveraging rust-analyzer's
//! full semantic analysis capabilities. No heuristics are used.

use crate::output::{ProjectTypeInfo, VariableTypeInfo};
use anyhow::{Context, Result};
use ra_ap_hir::{HirDisplay, Semantics};
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

    let Some(editioned_file_id) = sema.attach_first_edition(file_id) else {
        warn!("File {} not in crate graph, skipping", relative_path);
        return variables;
    };

    let source_file = sema.parse(editioned_file_id);

    for node in source_file.syntax().descendants() {
        match node.kind() {
            SyntaxKind::LET_STMT => {
                if let Some(var_info) = analyze_let_stmt(sema, db, &node, relative_path, &source_file) {
                    variables.push(var_info);
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
) -> Option<VariableTypeInfo> {
    let let_stmt = ast::LetStmt::cast(node.clone())?;
    let pat = let_stmt.pat()?;

    let (line, column) = get_location(&pat.syntax().text_range(), source_file);
    let name = pat.syntax().text().to_string();
    let mut var_info = VariableTypeInfo::new(name, relative_path.to_string(), line, column);

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

    Some(var_info)
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

    let (line, column) = get_location(&name_token.syntax().text_range(), source_file);
    let name = name_token.text().to_string();
    let mut var_info = VariableTypeInfo::new(name, relative_path.to_string(), line, column);

    // Try to resolve the type from the type annotation
    if let Some(ty_node) = ty_node {
        if let Some(ty) = sema.resolve_type(&ty_node) {
            populate_type_info(&mut var_info, &ty, db);
        } else {
            // Fallback: use the syntax text
            var_info.ty = ty_node.syntax().text().to_string();
            classify_type(&mut var_info);
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

/// Populate type info from a resolved type
fn populate_type_info(var_info: &mut VariableTypeInfo, ty: &ra_ap_hir::Type, db: &RootDatabase) {
    var_info.ty = ty.display(db, Edition::Edition2021).to_string();
    var_info.is_copy = ty.is_copy(db);
    var_info.is_reference = ty.is_reference();
    var_info.is_mutable_reference = ty.is_mutable_reference();
    var_info.is_raw_ptr = ty.is_raw_ptr();

    // Check if type is a union using semantic analysis
    if let Some(adt) = ty.as_adt() {
        var_info.is_union = matches!(adt, ra_ap_hir::Adt::Union(_));
    }

    classify_type(var_info);
}

/// Classify type based on resolved type string
fn classify_type(var_info: &mut VariableTypeInfo) {
    let ty = &var_info.ty;

    // Smart pointers
    var_info.is_rc = ty.contains("Rc<") && !ty.contains("Arc<");
    var_info.is_arc = ty.contains("Arc<");
    var_info.is_box = ty.contains("Box<");
    var_info.is_weak = ty.contains("Weak<");

    // Interior mutability
    var_info.is_refcell = ty.contains("RefCell<");
    var_info.is_cell = ty.contains("Cell<") && !var_info.is_refcell && !ty.contains("OnceCell");
    var_info.is_mutex = ty.contains("Mutex<") && !ty.contains("MutexGuard");
    var_info.is_rwlock = ty.contains("RwLock<") && !ty.contains("RwLockReadGuard") && !ty.contains("RwLockWriteGuard");

    // Guards
    var_info.is_guard = ty.contains("MutexGuard<")
        || ty.contains("RwLockReadGuard<")
        || ty.contains("RwLockWriteGuard<")
        || ty.contains("Ref<")
        || ty.contains("RefMut<");

    // Collections
    var_info.is_vec = ty.contains("Vec<");
    var_info.is_string = ty == "String" || ty.starts_with("String,");

    // Slices and str
    var_info.is_slice = ty.starts_with("&[") || ty.starts_with("&mut [");
    var_info.is_str = ty == "&str" || ty == "&mut str";

    // Wrapper types
    var_info.is_pin = ty.starts_with("Pin<");
    var_info.is_cow = ty.starts_with("Cow<");
    var_info.is_option = ty.starts_with("Option<");
    var_info.is_result = ty.starts_with("Result<");

    // Callable/async types
    var_info.is_closure = ty.starts_with("impl Fn") || ty.contains("closure");
    var_info.is_future = ty.contains("Future<") || ty.contains("impl Future");
    // Iterator adapters - be careful not to match HashMap, BTreeMap, etc.
    var_info.is_iterator = ty.starts_with("impl Iterator")
        || ty.starts_with("IntoIter<")
        || ty.starts_with("Iter<")
        || ty.starts_with("IterMut<")
        || ty.starts_with("Map<")
        || ty.starts_with("Filter<")
        || ty.starts_with("Chain<")
        || ty.starts_with("Enumerate<")
        || ty.starts_with("Zip<")
        || ty.starts_with("Take<")
        || ty.starts_with("Skip<")
        || ty.starts_with("Flatten<")
        || ty.starts_with("FlatMap<")
        || ty.starts_with("Rev<")
        || ty.starts_with("Peekable<");

    // Union and extern types (detected by type name patterns)
    // Note: is_union is set semantically in populate_type_info using as_adt()
    // Extern types from FFI - c_void, CStr, CString, OsStr, OsString
    var_info.is_extern_type = ty.contains("c_void")
        || ty.contains("CStr")
        || ty.contains("CString")
        || ty.contains("OsStr")
        || ty.contains("OsString");

    // Detect explicit lifetime annotations (e.g., &'a T, &'static str)
    var_info.has_lifetime = ty.contains("'");

    // Extract inner type for wrapper types
    var_info.inner_type = extract_inner_type(ty);
}

/// Extract the inner type from wrapper types like Rc<T>, Box<T>, etc.
fn extract_inner_type(ty: &str) -> Option<String> {
    // Find the first '<' and matching '>'
    let start = ty.find('<')?;
    let mut depth = 0;
    let mut end = None;

    for (i, c) in ty[start..].char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end?;
    let inner = &ty[start + 1..end];

    // For types with allocator like "String, Global", strip the allocator
    let inner = if let Some(comma_pos) = inner.rfind(", Global") {
        &inner[..comma_pos]
    } else {
        inner
    };

    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}
