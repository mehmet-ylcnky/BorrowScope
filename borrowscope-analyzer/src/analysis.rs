//! Project analysis using rust-analyzer crates
//!
//! This module loads a Rust project and extracts type information.
//! Phase 1: Syntax-based analysis (current)
//! Phase 2: Semantic analysis with full rust-analyzer integration (future)

use crate::output::{ProjectTypeInfo, VariableTypeInfo};
use anyhow::Result;
use ra_ap_syntax::{ast, AstNode, SyntaxKind};
use std::path::Path;
use walkdir::WalkDir;

/// Analyze a Rust project and extract type information
pub fn analyze_project(project_path: &Path) -> Result<ProjectTypeInfo> {
    let cargo_toml = project_path.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found at {}", project_path.display());
    }

    println!("Loading project from: {}", cargo_toml.display());

    let mut info = ProjectTypeInfo::new();

    // Walk through all .rs files
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(project_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Skip target directory
        if relative.starts_with("target") {
            continue;
        }

        println!("  Analyzing: {}", relative);

        // Read and parse the file
        let content = std::fs::read_to_string(path)?;
        let parse = ra_ap_syntax::SourceFile::parse(&content, ra_ap_syntax::Edition::Edition2021);
        let source_file = parse.tree();

        let variables = extract_variables(&source_file, &relative);

        if !variables.is_empty() {
            info.files.insert(relative, variables);
        }
    }

    Ok(info)
}

fn extract_variables(source_file: &ra_ap_syntax::SourceFile, file_path: &str) -> Vec<VariableTypeInfo> {
    let mut variables = Vec::new();

    // Walk the syntax tree looking for let bindings
    for node in source_file.syntax().descendants() {
        if node.kind() == SyntaxKind::LET_STMT {
            if let Some(let_stmt) = ast::LetStmt::cast(node) {
                if let Some(pat) = let_stmt.pat() {
                    // Get the pattern name
                    let name = pat.syntax().text().to_string();

                    // Get explicit type annotation if present
                    let ty_str = if let Some(ty) = let_stmt.ty() {
                        ty.syntax().text().to_string()
                    } else {
                        // Try to infer from initializer syntax
                        infer_type_from_syntax(&let_stmt)
                    };

                    // Detect type characteristics from the type string
                    let is_rc = ty_str.contains("Rc<") || ty_str.contains("Rc::");
                    let is_arc = ty_str.contains("Arc<") || ty_str.contains("Arc::");
                    let is_refcell = ty_str.contains("RefCell<") || ty_str.contains("RefCell::");
                    let is_cell =
                        (ty_str.contains("Cell<") || ty_str.contains("Cell::")) && !is_refcell;
                    let is_mutex = ty_str.contains("Mutex<") || ty_str.contains("Mutex::");
                    let is_rwlock = ty_str.contains("RwLock<") || ty_str.contains("RwLock::");
                    let is_raw_ptr = ty_str.contains("*const") || ty_str.contains("*mut");

                    // Get position
                    let range = pat.syntax().text_range();
                    let text_before = source_file
                        .syntax()
                        .text()
                        .slice(..range.start())
                        .to_string();
                    let line = text_before.lines().count() as u32;
                    let column = text_before.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

                    variables.push(VariableTypeInfo {
                        name,
                        ty: ty_str,
                        is_copy: false, // Can't determine without semantic analysis
                        is_rc,
                        is_arc,
                        is_refcell,
                        is_cell,
                        is_mutex,
                        is_rwlock,
                        is_raw_ptr,
                        is_union: false,
                        is_static: false,
                        is_ffi: false,
                        file: file_path.to_string(),
                        line,
                        column,
                    });
                }
            }
        }
    }

    variables
}

/// Infer type from initializer syntax (heuristic-based)
fn infer_type_from_syntax(let_stmt: &ast::LetStmt) -> String {
    if let Some(init) = let_stmt.initializer() {
        let init_text = init.syntax().text().to_string();

        // Check for common constructor patterns (including wrapped versions)
        if init_text.contains("Rc::new") || init_text.contains("track_rc_new") {
            return "Rc<_>".to_string();
        }
        if init_text.contains("Arc::new") || init_text.contains("track_arc_new") {
            return "Arc<_>".to_string();
        }
        if init_text.contains("RefCell::new") || init_text.contains("track_refcell_new") {
            return "RefCell<_>".to_string();
        }
        if init_text.contains("Cell::new") || init_text.contains("track_cell_new") {
            return "Cell<_>".to_string();
        }
        if init_text.contains("Mutex::new") {
            return "Mutex<_>".to_string();
        }
        if init_text.contains("RwLock::new") {
            return "RwLock<_>".to_string();
        }
        if init_text.contains("Rc::clone") || init_text.contains("track_rc_clone") {
            return "Rc<_>".to_string();
        }
        if init_text.contains("Arc::clone") || init_text.contains("track_arc_clone") {
            return "Arc<_>".to_string();
        }
        if init_text.contains("Weak::new") || init_text.contains("Rc::downgrade") {
            return "Weak<_>".to_string();
        }
        if init_text.contains("Box::new") || init_text.contains("track_new") && init_text.contains("Box::") {
            return "Box<_>".to_string();
        }
        if init_text.starts_with("vec!") || init_text.contains("Vec::") {
            return "Vec<_>".to_string();
        }
        if init_text.contains("String::") || init_text.contains(".to_string()") {
            return "String".to_string();
        }
        if init_text.contains("track_borrow_mut") || (init_text.starts_with('&') && init_text.contains("mut ")) {
            return "&mut _".to_string();
        }
        if init_text.contains("track_borrow") || init_text.starts_with('&') {
            return "&_".to_string();
        }

        // Check for literals
        if init_text.parse::<i64>().is_ok() {
            return "i32".to_string();
        }
        if init_text.parse::<f64>().is_ok() {
            return "f64".to_string();
        }
        if init_text == "true" || init_text == "false" {
            return "bool".to_string();
        }
        if init_text.starts_with('"') {
            return "&str".to_string();
        }
    }

    "unknown".to_string()
}
