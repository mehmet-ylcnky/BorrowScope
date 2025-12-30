//! Project analysis using rust-analyzer crates
//!
//! Current: Syntax-based analysis (~70% accuracy)
//! Future: Semantic analysis with ra_ap_hir for full type resolution
//!
//! # TODO: Semantic Analysis Integration
//!
//! The current implementation uses syntax-only analysis which cannot:
//! - Resolve inferred types (e.g., `let x = foo()` where foo returns Rc<T>)
//! - Detect Copy trait implementation
//! - Identify type aliases (e.g., `type MyRc = Rc<String>`)
//! - Track types through function calls and method chains
//! - Detect FFI function calls (extern "C")
//! - Identify union field access
//! - Resolve static variable references
//!
//! To achieve ~99% accuracy, integrate ra_ap_hir semantic analysis:
//!
//! ```ignore
//! // 1. Load workspace with full analysis
//! let (db, vfs, _) = load_workspace_at(...);
//! let sema = Semantics::new(&db);
//!
//! // 2. For each file, get EditionedFileId (not just FileId)
//! let file_id = sema.attach_first_edition(file_id);
//!
//! // 3. Query types for expressions
//! if let Some(ty_info) = sema.type_of_expr(&expr) {
//!     let ty = ty_info.original;
//!     let is_copy = ty.is_copy(db);
//!     let display = ty.display(db, Edition::Edition2021).to_string();
//! }
//!
//! // 4. Check for specific traits
//! ty.impls_trait(db, copy_trait, &[])
//! ```
//!
//! Key challenges encountered:
//! - FileId vs EditionedFileId conversion
//! - SourceDatabase trait not in scope by default
//! - display() requires Edition parameter
//! - VFS doesn't store file contents directly
//!
//! Reference: rust-analyzer source code at
//! https://github.com/rust-lang/rust-analyzer/blob/master/crates/ide/src/

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

    println!("Analyzing project (syntax mode)...");

    let mut info = ProjectTypeInfo::new();

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

        if relative.starts_with("target") {
            continue;
        }

        println!("  {}", relative);

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

    for node in source_file.syntax().descendants() {
        if node.kind() != SyntaxKind::LET_STMT {
            continue;
        }

        let Some(let_stmt) = ast::LetStmt::cast(node) else { continue };
        let Some(pat) = let_stmt.pat() else { continue };

        let name = pat.syntax().text().to_string();
        let ty_str = let_stmt.ty()
            .map(|ty| ty.syntax().text().to_string())
            .unwrap_or_else(|| infer_type(&let_stmt));

        let is_rc = ty_str.contains("Rc<") || ty_str.contains("Rc::");
        let is_arc = ty_str.contains("Arc<") || ty_str.contains("Arc::");
        let is_refcell = ty_str.contains("RefCell<") || ty_str.contains("RefCell::");
        let is_cell = (ty_str.contains("Cell<") || ty_str.contains("Cell::")) && !is_refcell;
        let is_mutex = ty_str.contains("Mutex<") || ty_str.contains("Mutex::");
        let is_rwlock = ty_str.contains("RwLock<") || ty_str.contains("RwLock::");
        let is_raw_ptr = ty_str.contains("*const") || ty_str.contains("*mut");

        let range = pat.syntax().text_range();
        let text_before = source_file.syntax().text().slice(..range.start()).to_string();
        let line = text_before.lines().count() as u32;
        let column = text_before.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

        variables.push(VariableTypeInfo {
            name,
            ty: ty_str,
            is_copy: false, // TODO: requires semantic analysis
            is_rc,
            is_arc,
            is_refcell,
            is_cell,
            is_mutex,
            is_rwlock,
            is_raw_ptr,
            is_union: false,  // TODO: requires semantic analysis
            is_static: false, // TODO: requires semantic analysis
            is_ffi: false,    // TODO: requires semantic analysis
            file: file_path.to_string(),
            line,
            column,
        });
    }

    variables
}

/// Infer type from initializer syntax (heuristic-based)
///
/// TODO: This is a best-effort approach that catches common patterns.
/// For accurate type inference, use ra_ap_hir::Semantics::type_of_expr()
fn infer_type(let_stmt: &ast::LetStmt) -> String {
    let Some(init) = let_stmt.initializer() else {
        return "unknown".to_string();
    };

    let text = init.syntax().text().to_string();

    // Smart pointers (including BorrowScope wrappers)
    if text.contains("Rc::new") || text.contains("track_rc_new") { return "Rc<_>".to_string(); }
    if text.contains("Arc::new") || text.contains("track_arc_new") { return "Arc<_>".to_string(); }
    if text.contains("RefCell::new") || text.contains("track_refcell_new") { return "RefCell<_>".to_string(); }
    if text.contains("Cell::new") || text.contains("track_cell_new") { return "Cell<_>".to_string(); }
    if text.contains("Mutex::new") { return "Mutex<_>".to_string(); }
    if text.contains("RwLock::new") { return "RwLock<_>".to_string(); }
    if text.contains("Rc::clone") || text.contains("track_rc_clone") { return "Rc<_>".to_string(); }
    if text.contains("Arc::clone") || text.contains("track_arc_clone") { return "Arc<_>".to_string(); }
    if text.contains("Rc::downgrade") { return "Weak<_>".to_string(); }
    if text.contains("Box::new") { return "Box<_>".to_string(); }

    // Collections
    if text.starts_with("vec!") || text.contains("Vec::") { return "Vec<_>".to_string(); }
    if text.contains("String::") || text.contains(".to_string()") { return "String".to_string(); }

    // References
    if text.contains("track_borrow_mut") || (text.starts_with('&') && text.contains("mut ")) {
        return "&mut _".to_string();
    }
    if text.contains("track_borrow") || text.starts_with('&') { return "&_".to_string(); }

    // Literals
    if text.parse::<i64>().is_ok() { return "i32".to_string(); }
    if text.parse::<f64>().is_ok() { return "f64".to_string(); }
    if text == "true" || text == "false" { return "bool".to_string(); }
    if text.starts_with('"') { return "&str".to_string(); }

    "unknown".to_string()
}
