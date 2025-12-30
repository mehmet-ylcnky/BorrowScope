//! Semantic analysis using rust-analyzer
//!
//! This module provides production-ready type analysis by leveraging
//! rust-analyzer's full semantic analysis capabilities.

use crate::output::{ProjectTypeInfo, VariableTypeInfo};
use anyhow::{Context, Result};
use ra_ap_hir::{HirDisplay, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::CargoConfig;
use ra_ap_syntax::{ast, AstNode, SyntaxKind, SourceFile, Edition};
use ra_ap_vfs::Vfs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Analyze a Rust project and extract type information for all variables
pub fn analyze_project(project_path: &Path) -> Result<ProjectTypeInfo> {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found at {}", project_path.display());
    }

    info!("Loading workspace...");

    let mut cargo_config = CargoConfig::default();
    // Enable sysroot discovery to resolve std library types
    cargo_config.sysroot = Some(ra_ap_project_model::RustLibSource::Discover);
    
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,  // Load build script outputs
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: true,  // Prefill caches for better analysis
    };

    let (db, vfs, _proc_macro) = load_workspace_at(
        project_path,
        &cargo_config,
        &load_config,
        &|msg| {
            if msg.contains("workspace") || msg.contains("Discovering") {
                println!("  {}", msg);
            }
            debug!("{}", msg);
        },
    )
    .context("Failed to load workspace")?;

    info!("Workspace loaded, analyzing files...");

    let project_abs = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    let mut info = ProjectTypeInfo::new();
    let sema = Semantics::new(&db);

    // Process each file in the VFS
    for (file_id, vfs_path) in vfs.iter() {
        let path_str = match vfs_path.as_path() {
            Some(p) => p.to_string(),
            None => continue,
        };

        // Skip non-Rust files
        if !path_str.ends_with(".rs") {
            continue;
        }

        // Skip external dependencies
        if path_str.contains("/.cargo/")
            || path_str.contains("/rustup/")
            || path_str.contains("\\registry\\")
        {
            continue;
        }

        // Check if file is in our project
        let project_prefix = project_abs.to_string_lossy();
        if !path_str.starts_with(project_prefix.as_ref()) {
            continue;
        }

        // Get relative path
        let relative = path_str
            .strip_prefix(project_prefix.as_ref())
            .unwrap_or(&path_str)
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .to_string();

        // Skip target directory
        if relative.starts_with("target") {
            continue;
        }

        println!("  Analyzing: {}", relative);

        // Analyze this file
        let variables = analyze_file(&sema, &db, &vfs, file_id, &relative);
        if !variables.is_empty() {
            info.files.insert(relative, variables);
        }
    }

    Ok(info)
}

/// Analyze a single file and extract variable type information
fn analyze_file(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    vfs: &Vfs,
    file_id: ra_ap_vfs::FileId,
    relative_path: &str,
) -> Vec<VariableTypeInfo> {
    let mut variables = Vec::new();

    // Try to get the file's edition from the crate
    let edition = sema.attach_first_edition(file_id);

    // Get file contents from VFS - we need to read it ourselves
    let file_path = vfs.file_path(file_id);
    let content = match std::fs::read_to_string(file_path.as_path().unwrap().as_str()) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read {}: {}", relative_path, e);
            return variables;
        }
    };

    // Parse the file
    let source_file = SourceFile::parse(&content, Edition::Edition2021).tree();

    // If we have semantic info, use it; otherwise fall back to syntax
    if let Some(editioned_file_id) = edition {
        // We have semantic analysis available
        let parsed = sema.parse(editioned_file_id);
        extract_with_semantics(sema, db, &parsed, relative_path, &mut variables);
    } else {
        // Fall back to syntax-only analysis
        debug!("No semantic info for {}, using syntax analysis", relative_path);
        extract_syntax_only(&source_file, relative_path, &mut variables);
    }

    variables
}

/// Extract variables using full semantic analysis
fn extract_with_semantics(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &SourceFile,
    file_path: &str,
    variables: &mut Vec<VariableTypeInfo>,
) {
    for node in source_file.syntax().descendants() {
        if node.kind() != SyntaxKind::LET_STMT {
            continue;
        }

        let Some(let_stmt) = ast::LetStmt::cast(node) else {
            continue;
        };
        let Some(pat) = let_stmt.pat() else {
            continue;
        };

        // Get position info
        let range = pat.syntax().text_range();
        let text_before = source_file
            .syntax()
            .text()
            .slice(..range.start())
            .to_string();
        let line = text_before.lines().count() as u32;
        let column = text_before.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

        let name = pat.syntax().text().to_string();
        let mut var_info = VariableTypeInfo::new(name, file_path.to_string(), line, column);

        // Try to get type from semantic analysis
        if let Some(init) = let_stmt.initializer() {
            if let Some(type_info) = sema.type_of_expr(&init) {
                let ty = type_info.original;

                // Get display string - use Edition2021 as default
                var_info.ty = ty.display(db, Edition::Edition2021).to_string();

                // Query type properties
                var_info.is_copy = ty.is_copy(db);
                var_info.is_reference = ty.is_reference();
                var_info.is_mutable_reference = ty.is_mutable_reference();
                var_info.is_raw_ptr = ty.is_raw_ptr();

                // Classify by type name
                classify_type(&mut var_info);
            } else {
                // Semantic analysis didn't return type, try syntax heuristics
                infer_from_syntax(&let_stmt, &mut var_info);
            }
        } else if let Some(ty_annotation) = let_stmt.ty() {
            // Use explicit type annotation
            var_info.ty = ty_annotation.syntax().text().to_string();
            classify_type(&mut var_info);
        }

        variables.push(var_info);
    }
}

/// Extract variables using syntax-only analysis (fallback)
fn extract_syntax_only(
    source_file: &SourceFile,
    file_path: &str,
    variables: &mut Vec<VariableTypeInfo>,
) {
    for node in source_file.syntax().descendants() {
        if node.kind() != SyntaxKind::LET_STMT {
            continue;
        }

        let Some(let_stmt) = ast::LetStmt::cast(node) else {
            continue;
        };
        let Some(pat) = let_stmt.pat() else {
            continue;
        };

        let range = pat.syntax().text_range();
        let text_before = source_file
            .syntax()
            .text()
            .slice(..range.start())
            .to_string();
        let line = text_before.lines().count() as u32;
        let column = text_before.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

        let name = pat.syntax().text().to_string();
        let mut var_info = VariableTypeInfo::new(name, file_path.to_string(), line, column);

        // Try explicit type annotation first
        if let Some(ty) = let_stmt.ty() {
            var_info.ty = ty.syntax().text().to_string();
            classify_type(&mut var_info);
        } else {
            // Infer from initializer syntax
            infer_from_syntax(&let_stmt, &mut var_info);
        }

        variables.push(var_info);
    }
}

/// Classify type based on type string
fn classify_type(var_info: &mut VariableTypeInfo) {
    let ty = &var_info.ty;

    var_info.is_rc = ty.contains("Rc<") && !ty.contains("Arc<");
    var_info.is_arc = ty.contains("Arc<");
    var_info.is_refcell = ty.contains("RefCell<");
    var_info.is_cell = ty.contains("Cell<") && !var_info.is_refcell;
    var_info.is_mutex = ty.contains("Mutex<");
    var_info.is_rwlock = ty.contains("RwLock<");
    var_info.is_box = ty.contains("Box<");
    var_info.is_vec = ty.contains("Vec<");
    var_info.is_string = ty == "String" || ty.contains("::String");
    var_info.is_raw_ptr = ty.starts_with("*const") || ty.starts_with("*mut");
    var_info.is_reference = ty.starts_with('&') && !var_info.is_raw_ptr;
    var_info.is_mutable_reference = ty.starts_with("&mut");
}

/// Infer type from initializer syntax (heuristic fallback)
fn infer_from_syntax(let_stmt: &ast::LetStmt, var_info: &mut VariableTypeInfo) {
    let Some(init) = let_stmt.initializer() else {
        return;
    };

    let text = init.syntax().text().to_string();

    // Smart pointer patterns (including BorrowScope wrappers)
    let patterns = [
        ("Rc::new", "Rc<_>"),
        ("track_rc_new", "Rc<_>"),
        ("Arc::new", "Arc<_>"),
        ("track_arc_new", "Arc<_>"),
        ("RefCell::new", "RefCell<_>"),
        ("track_refcell_new", "RefCell<_>"),
        ("Cell::new", "Cell<_>"),
        ("track_cell_new", "Cell<_>"),
        ("Mutex::new", "Mutex<_>"),
        ("RwLock::new", "RwLock<_>"),
        ("Rc::clone", "Rc<_>"),
        ("track_rc_clone", "Rc<_>"),
        ("Arc::clone", "Arc<_>"),
        ("track_arc_clone", "Arc<_>"),
        ("Rc::downgrade", "Weak<_>"),
        ("Arc::downgrade", "Weak<_>"),
        ("Box::new", "Box<_>"),
    ];

    for (pattern, ty) in patterns {
        if text.contains(pattern) {
            var_info.ty = ty.to_string();
            classify_type(var_info);
            return;
        }
    }

    // Collection patterns
    if text.starts_with("vec!") || text.contains("Vec::") {
        var_info.ty = "Vec<_>".to_string();
        var_info.is_vec = true;
        return;
    }

    if text.contains("String::") || text.contains(".to_string()") || text.contains(".to_owned()") {
        var_info.ty = "String".to_string();
        var_info.is_string = true;
        return;
    }

    // Reference patterns
    if text.contains("track_borrow_mut") || (text.starts_with("&mut")) {
        var_info.ty = "&mut _".to_string();
        var_info.is_reference = true;
        var_info.is_mutable_reference = true;
        return;
    }

    if text.contains("track_borrow") || text.starts_with('&') {
        var_info.ty = "&_".to_string();
        var_info.is_reference = true;
        return;
    }

    // Literal patterns
    if text.parse::<i64>().is_ok() {
        var_info.ty = "i32".to_string();
        var_info.is_copy = true;
        return;
    }

    if text.parse::<f64>().is_ok() {
        var_info.ty = "f64".to_string();
        var_info.is_copy = true;
        return;
    }

    if text == "true" || text == "false" {
        var_info.ty = "bool".to_string();
        var_info.is_copy = true;
        return;
    }

    if text.starts_with('"') || text.starts_with("r#\"") || text.starts_with("r\"") {
        var_info.ty = "&str".to_string();
        var_info.is_reference = true;
        var_info.is_copy = true;
        return;
    }

    if text.starts_with('\'') && text.ends_with('\'') && text.len() <= 4 {
        var_info.ty = "char".to_string();
        var_info.is_copy = true;
    }
}
