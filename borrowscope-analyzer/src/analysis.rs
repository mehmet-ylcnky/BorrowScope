//! Semantic analysis using rust-analyzer
//!
//! This module provides type analysis by leveraging rust-analyzer's
//! full semantic analysis capabilities. No heuristics are used.

use crate::output::{ProjectTypeInfo, VariableTypeInfo, MethodCallInfo, ExpressionInfo};
use anyhow::{Context, Result};
use ra_ap_hir::{db::DefDatabase, HirDisplay, LangItem, Semantics, Function, Adt};
use ra_ap_ide_db::RootDatabase;
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::CargoConfig;
use ra_ap_syntax::{ast, AstNode, Edition, SyntaxKind};
use ra_ap_syntax::ast::{HasName, HasArgList};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

/// Known ADT types looked up once at startup by semantic identity (AdtId).
/// Used for type classification without string matching.
#[derive(Default)]
pub(crate) struct KnownTypes {
    // Smart pointers
    rc: Option<Adt>,
    arc: Option<Adt>,
    box_: Option<Adt>,
    weak_rc: Option<Adt>,
    weak_arc: Option<Adt>,
    
    // Interior mutability
    cell: Option<Adt>,
    refcell: Option<Adt>,
    unsafe_cell: Option<Adt>,
    mutex: Option<Adt>,
    rwlock: Option<Adt>,
    once_cell: Option<Adt>,
    once_lock: Option<Adt>,
    
    // Guards
    ref_guard: Option<Adt>,
    refmut_guard: Option<Adt>,
    mutex_guard: Option<Adt>,
    rwlock_read_guard: Option<Adt>,
    rwlock_write_guard: Option<Adt>,
    
    // Memory
    maybe_uninit: Option<Adt>,
    manually_drop: Option<Adt>,
    
    // Collections
    vec: Option<Adt>,
    string: Option<Adt>,
    hashmap: Option<Adt>,
    hashset: Option<Adt>,
    
    // Wrappers
    pin: Option<Adt>,
    cow: Option<Adt>,
    option: Option<Adt>,
    result: Option<Adt>,
    
    // Channels
    sender: Option<Adt>,
    receiver: Option<Adt>,
    sync_sender: Option<Adt>,
    
    // Paths/FFI
    pathbuf: Option<Adt>,
    osstring: Option<Adt>,
    cstring: Option<Adt>,
    
    // NonNull
    nonnull: Option<Adt>,
}

/// Get full module path as string (e.g., "std::sync::poison::mutex")
fn get_module_path(module: &ra_ap_hir::Module, db: &RootDatabase) -> String {
    let mut parts = Vec::new();
    let mut current = Some(*module);
    while let Some(m) = current {
        if let Some(name) = m.name(db) {
            parts.push(name.display_no_db(Edition::Edition2021).to_string());
        }
        current = m.parent(db);
    }
    parts.reverse();
    parts.join("::")
}

impl KnownTypes {
    /// Build the set of known types by looking them up semantically
    fn new(db: &RootDatabase) -> Self {
        use ra_ap_hir::{import_map, ModuleDef, Crate};
        
        let mut known = Self::default();
        
        // Types to find: (type_name, expected_module, field_setter)
        let types_to_find: &[(&str, &str, fn(&mut KnownTypes, Adt))] = &[
            // Smart pointers
            ("Rc", "rc", |k, a| k.rc = Some(a)),
            ("Arc", "sync", |k, a| k.arc = Some(a)),
            ("Box", "boxed", |k, a| k.box_ = Some(a)),
            ("Weak", "rc", |k, a| k.weak_rc = Some(a)),
            // Note: Weak in sync module handled separately
            
            // Interior mutability
            ("Cell", "cell", |k, a| k.cell = Some(a)),
            ("RefCell", "cell", |k, a| k.refcell = Some(a)),
            ("UnsafeCell", "cell", |k, a| k.unsafe_cell = Some(a)),
            ("Mutex", "sync", |k, a| k.mutex = Some(a)),
            ("RwLock", "sync", |k, a| k.rwlock = Some(a)),
            ("OnceCell", "cell", |k, a| k.once_cell = Some(a)),
            ("OnceLock", "sync", |k, a| k.once_lock = Some(a)),
            
            // Guards
            ("Ref", "cell", |k, a| k.ref_guard = Some(a)),
            ("RefMut", "cell", |k, a| k.refmut_guard = Some(a)),
            ("MutexGuard", "sync", |k, a| k.mutex_guard = Some(a)),
            ("RwLockReadGuard", "sync", |k, a| k.rwlock_read_guard = Some(a)),
            ("RwLockWriteGuard", "sync", |k, a| k.rwlock_write_guard = Some(a)),
            
            // Memory
            ("MaybeUninit", "mem", |k, a| k.maybe_uninit = Some(a)),
            ("ManuallyDrop", "mem", |k, a| k.manually_drop = Some(a)),
            
            // Collections
            ("Vec", "vec", |k, a| k.vec = Some(a)),
            ("String", "string", |k, a| k.string = Some(a)),
            ("HashMap", "hash", |k, a| k.hashmap = Some(a)),
            ("HashSet", "hash", |k, a| k.hashset = Some(a)),
            
            // Wrappers
            ("Pin", "pin", |k, a| k.pin = Some(a)),
            ("Cow", "borrow", |k, a| k.cow = Some(a)),
            ("Option", "option", |k, a| k.option = Some(a)),
            ("Result", "result", |k, a| k.result = Some(a)),
            
            // Channels
            ("Sender", "mpsc", |k, a| k.sender = Some(a)),
            ("Receiver", "mpsc", |k, a| k.receiver = Some(a)),
            ("SyncSender", "mpsc", |k, a| k.sync_sender = Some(a)),
            
            // Paths/FFI
            ("PathBuf", "path", |k, a| k.pathbuf = Some(a)),
            ("OsString", "ffi", |k, a| k.osstring = Some(a)),
            ("CString", "ffi", |k, a| k.cstring = Some(a)),
            
            // NonNull
            ("NonNull", "ptr", |k, a| k.nonnull = Some(a)),
        ];
        
        for krate in Crate::all(db) {
            for (type_name, expected_module, setter) in types_to_find {
                let query = import_map::Query::new(type_name.to_string()).exact();
                for item in krate.query_external_importables(db, query) {
                    if let either::Either::Left(ModuleDef::Adt(adt)) = item {
                        // Build full module path to check against expected_module
                        let module_path = get_module_path(&adt.module(db), db);
                        if module_path.contains(expected_module) {
                            setter(&mut known, adt);
                        }
                    }
                }
            }
        }
        
        // Handle Weak in sync module (Arc's Weak)
        for krate in Crate::all(db) {
            let query = import_map::Query::new("Weak".to_string()).exact();
            for item in krate.query_external_importables(db, query) {
                if let either::Either::Left(ModuleDef::Adt(adt)) = item {
                    let module_path = get_module_path(&adt.module(db), db);
                    if module_path.contains("sync") && !module_path.contains("rc") {
                        known.weak_arc = Some(adt);
                    }
                }
            }
        }
        
        known
    }
    
    /// Classify an ADT by comparing AdtId directly (fully semantic)
    fn classify(&self, adt: &Adt) -> Option<&'static str> {
        // Smart pointers
        if self.rc.as_ref() == Some(adt) { return Some("rc"); }
        if self.arc.as_ref() == Some(adt) { return Some("arc"); }
        if self.box_.as_ref() == Some(adt) { return Some("box"); }
        if self.weak_rc.as_ref() == Some(adt) || self.weak_arc.as_ref() == Some(adt) { return Some("weak"); }
        
        // Interior mutability
        if self.cell.as_ref() == Some(adt) { return Some("cell"); }
        if self.refcell.as_ref() == Some(adt) { return Some("refcell"); }
        if self.unsafe_cell.as_ref() == Some(adt) { return Some("unsafe_cell"); }
        if self.mutex.as_ref() == Some(adt) { return Some("mutex"); }
        if self.rwlock.as_ref() == Some(adt) { return Some("rwlock"); }
        if self.once_cell.as_ref() == Some(adt) { return Some("once_cell"); }
        if self.once_lock.as_ref() == Some(adt) { return Some("once_lock"); }
        
        // Guards
        if self.ref_guard.as_ref() == Some(adt) { return Some("ref_guard"); }
        if self.refmut_guard.as_ref() == Some(adt) { return Some("refmut_guard"); }
        if self.mutex_guard.as_ref() == Some(adt) { return Some("mutex_guard"); }
        if self.rwlock_read_guard.as_ref() == Some(adt) { return Some("rwlock_read_guard"); }
        if self.rwlock_write_guard.as_ref() == Some(adt) { return Some("rwlock_write_guard"); }
        
        // Memory
        if self.maybe_uninit.as_ref() == Some(adt) { return Some("maybe_uninit"); }
        if self.manually_drop.as_ref() == Some(adt) { return Some("manually_drop"); }
        
        // Collections
        if self.vec.as_ref() == Some(adt) { return Some("vec"); }
        if self.string.as_ref() == Some(adt) { return Some("string"); }
        if self.hashmap.as_ref() == Some(adt) { return Some("hashmap"); }
        if self.hashset.as_ref() == Some(adt) { return Some("hashset"); }
        
        // Wrappers
        if self.pin.as_ref() == Some(adt) { return Some("pin"); }
        if self.cow.as_ref() == Some(adt) { return Some("cow"); }
        if self.option.as_ref() == Some(adt) { return Some("option"); }
        if self.result.as_ref() == Some(adt) { return Some("result"); }
        
        // Channels
        if self.sender.as_ref() == Some(adt) { return Some("channel_sender"); }
        if self.receiver.as_ref() == Some(adt) { return Some("channel_receiver"); }
        if self.sync_sender.as_ref() == Some(adt) { return Some("sync_channel_sender"); }
        
        // Paths/FFI
        if self.pathbuf.as_ref() == Some(adt) { return Some("pathbuf"); }
        if self.osstring.as_ref() == Some(adt) { return Some("osstring"); }
        if self.cstring.as_ref() == Some(adt) { return Some("cstring"); }
        
        // NonNull
        if self.nonnull.as_ref() == Some(adt) { return Some("nonnull"); }
        
        None
    }
    
    /// Set boolean flags on VariableTypeInfo by comparing AdtId directly
    fn set_flags(&self, var_info: &mut VariableTypeInfo, adt: &Adt) {
        var_info.is_rc = self.rc.as_ref() == Some(adt);
        var_info.is_arc = self.arc.as_ref() == Some(adt);
        var_info.is_box = self.box_.as_ref() == Some(adt);
        var_info.is_weak = self.weak_rc.as_ref() == Some(adt) || self.weak_arc.as_ref() == Some(adt);
        
        var_info.is_cell = self.cell.as_ref() == Some(adt);
        var_info.is_refcell = self.refcell.as_ref() == Some(adt);
        var_info.is_mutex = self.mutex.as_ref() == Some(adt);
        var_info.is_rwlock = self.rwlock.as_ref() == Some(adt);
        
        var_info.is_guard = self.ref_guard.as_ref() == Some(adt)
            || self.refmut_guard.as_ref() == Some(adt)
            || self.mutex_guard.as_ref() == Some(adt)
            || self.rwlock_read_guard.as_ref() == Some(adt)
            || self.rwlock_write_guard.as_ref() == Some(adt);
        
        var_info.is_vec = self.vec.as_ref() == Some(adt);
        var_info.is_string = self.string.as_ref() == Some(adt);
        
        var_info.is_pin = self.pin.as_ref() == Some(adt);
        var_info.is_cow = self.cow.as_ref() == Some(adt);
        var_info.is_option = self.option.as_ref() == Some(adt);
        var_info.is_result = self.result.as_ref() == Some(adt);
        var_info.is_once_cell = self.once_cell.as_ref() == Some(adt) || self.once_lock.as_ref() == Some(adt);
        var_info.is_maybe_uninit = self.maybe_uninit.as_ref() == Some(adt);
        var_info.is_channel = self.sender.as_ref() == Some(adt) 
            || self.receiver.as_ref() == Some(adt)
            || self.sync_sender.as_ref() == Some(adt);
    }
    
    /// OR flags for tuple/array elements (doesn't clear existing flags)
    fn set_flags_or(&self, var_info: &mut VariableTypeInfo, adt: &Adt) {
        var_info.is_rc |= self.rc.as_ref() == Some(adt);
        var_info.is_arc |= self.arc.as_ref() == Some(adt);
        var_info.is_box |= self.box_.as_ref() == Some(adt);
        var_info.is_weak |= self.weak_rc.as_ref() == Some(adt) || self.weak_arc.as_ref() == Some(adt);
        var_info.is_cell |= self.cell.as_ref() == Some(adt);
        var_info.is_refcell |= self.refcell.as_ref() == Some(adt);
        var_info.is_mutex |= self.mutex.as_ref() == Some(adt);
        var_info.is_rwlock |= self.rwlock.as_ref() == Some(adt);
        var_info.is_guard |= self.ref_guard.as_ref() == Some(adt)
            || self.refmut_guard.as_ref() == Some(adt)
            || self.mutex_guard.as_ref() == Some(adt)
            || self.rwlock_read_guard.as_ref() == Some(adt)
            || self.rwlock_write_guard.as_ref() == Some(adt);
        var_info.is_vec |= self.vec.as_ref() == Some(adt);
        var_info.is_string |= self.string.as_ref() == Some(adt);
        var_info.is_pin |= self.pin.as_ref() == Some(adt);
        var_info.is_cow |= self.cow.as_ref() == Some(adt);
        var_info.is_option |= self.option.as_ref() == Some(adt);
        var_info.is_result |= self.result.as_ref() == Some(adt);
        var_info.is_once_cell |= self.once_cell.as_ref() == Some(adt) || self.once_lock.as_ref() == Some(adt);
        var_info.is_maybe_uninit |= self.maybe_uninit.as_ref() == Some(adt);
        var_info.is_channel |= self.sender.as_ref() == Some(adt) 
            || self.receiver.as_ref() == Some(adt)
            || self.sync_sender.as_ref() == Some(adt);
    }
}

/// Functions we track for ownership-relevant operations.
/// Looked up once at startup by semantic identity (FunctionId), not string matching.
#[derive(Default)]
pub(crate) struct TrackedFunctions {
    /// Maps FunctionId to canonical path for tracked functions
    functions: HashMap<Function, String>,
}

impl TrackedFunctions {
    /// Build the set of tracked functions by looking them up semantically
    fn new(db: &RootDatabase) -> Self {
        use ra_ap_hir::{import_map, ModuleDef, Crate};
        
        let mut tracked = Self::default();
        
        // Functions to track: (function_name, acceptable_modules)
        // Some functions are intrinsics re-exported to mem/ptr
        let functions_to_find: &[(&str, &[&str])] = &[
            ("drop", &["mem"]),
            ("forget", &["mem"]),
            ("transmute", &["mem", "intrinsics"]),
            ("transmute_copy", &["mem"]),
            ("replace", &["mem"]),
            ("swap", &["mem"]),
            ("take", &["mem"]),
            ("spawn", &["thread"]),
            ("read", &["ptr"]),
            ("write", &["ptr"]),
            ("read_volatile", &["ptr", "intrinsics"]),
            ("write_volatile", &["ptr", "intrinsics"]),
            ("copy", &["ptr", "intrinsics"]),
            ("copy_nonoverlapping", &["ptr", "intrinsics"]),
        ];
        
        for krate in Crate::all(db) {
            for (fn_name, acceptable_modules) in functions_to_find {
                let query = import_map::Query::new(fn_name.to_string()).exact();
                for item in krate.query_external_importables(db, query) {
                    if let either::Either::Left(ModuleDef::Function(f)) = item {
                        let module_path = get_module_path(&f.module(db), db);
                        if acceptable_modules.iter().any(|m| module_path.contains(m)) {
                            let path = get_function_path(&f, db);
                            tracked.functions.insert(f, path);
                        }
                    }
                }
            }
        }
        
        for path in tracked.functions.values() {
            println!("    Tracked: {}", path);
        }
        info!("Tracked {} ownership-relevant functions", tracked.functions.len());
        tracked
    }
    
    /// Check if a function is tracked and return its canonical path
    fn get_path(&self, func: &Function) -> Option<&String> {
        self.functions.get(func)
    }
}

/// Get the canonical path of a function
fn get_function_path(f: &Function, db: &RootDatabase) -> String {
    let module = f.module(db);
    let krate = module.krate().display_name(db)
        .map(|n| n.to_string())
        .unwrap_or_default();
    let mod_path: Vec<String> = module.path_to_root(db)
        .into_iter()
        .rev()
        .filter_map(|m| m.name(db))
        .map(|n| n.display_no_db(Edition::Edition2021).to_string())
        .collect();
    let fn_name = f.name(db).display_no_db(Edition::Edition2021).to_string();
    
    if mod_path.is_empty() {
        format!("{}::{}", krate, fn_name)
    } else {
        format!("{}::{}::{}", krate, mod_path.join("::"), fn_name)
    }
}

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
    
    // Look up tracked functions and types once by semantic identity
    let tracked_functions = TrackedFunctions::new(&db);
    let known_types = KnownTypes::new(&db);

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

        let (variables, expressions) = analyze_file(&sema, &db, &tracked_functions, &known_types, file_id, &relative);
        if !variables.is_empty() {
            info.files.insert(relative.clone(), variables);
        }
        if !expressions.is_empty() {
            info.expressions.insert(relative, expressions);
        }
    }

    Ok(info)
}

/// Analyze a single file using semantic analysis only
fn analyze_file(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    tracked_functions: &TrackedFunctions,
    known_types: &KnownTypes,
    file_id: ra_ap_vfs::FileId,
    relative_path: &str,
) -> (Vec<VariableTypeInfo>, Vec<ExpressionInfo>) {
    let mut variables = Vec::new();
    let mut scope_id: u32 = 0;

    let Some(editioned_file_id) = sema.attach_first_edition(file_id) else {
        warn!("File {} not in crate graph, skipping", relative_path);
        return (variables, Vec::new());
    };

    let source_file = sema.parse(editioned_file_id);

    // Track current function context and declaration counts
    let mut current_fn: Option<String> = None;
    let mut decl_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for node in source_file.syntax().descendants() {
        // Track function boundaries
        if let Some(fn_node) = ast::Fn::cast(node.clone()) {
            current_fn = fn_node.name().map(|n| n.text().to_string());
            // Reset decl count for new function
            if let Some(ref fn_name) = current_fn {
                decl_counts.insert(fn_name.clone(), 0);
            }
        }
        
        match node.kind() {
            SyntaxKind::LET_STMT => {
                if let Some(mut var_info) = analyze_let_stmt(sema, db, known_types, &node, relative_path, &source_file, &mut scope_id) {
                    // Set function context
                    var_info.function_name = current_fn.clone();
                    if let Some(ref fn_name) = current_fn {
                        let count = decl_counts.entry(fn_name.clone()).or_insert(0);
                        var_info.decl_index = Some(*count);
                        *count += 1;
                    }
                    variables.push(var_info);
                    scope_id += 1;
                }
            }
            SyntaxKind::STATIC => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, known_types, &node, relative_path, &source_file) {
                    var_info.is_static = true;
                    variables.push(var_info);
                }
            }
            SyntaxKind::CONST => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, known_types, &node, relative_path, &source_file) {
                    var_info.is_const = true;
                    variables.push(var_info);
                }
            }
            _ => {}
        }
    }

    // Analyze method calls on tracked variables
    analyze_method_calls(sema, db, &source_file, &mut variables);
    
    // Analyze standalone expressions (using semantic function lookup)
    let expressions = analyze_expressions(sema, db, tracked_functions, &source_file);

    (variables, expressions)
}

/// Analyze a let statement
fn analyze_let_stmt(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_types: &KnownTypes,
    node: &ra_ap_syntax::SyntaxNode,
    relative_path: &str,
    source_file: &ast::SourceFile,
    scope_id: &mut u32,
) -> Option<VariableTypeInfo> {
    let let_stmt = ast::LetStmt::cast(node.clone())?;
    let pat = let_stmt.pat()?;

    let range = pat.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    
    // Extract the actual variable name (without 'mut' keyword)
    let name = extract_pattern_name(&pat);
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
        populate_type_info(&mut var_info, &type_info.original, db, known_types);
    }

    // Detect impl Trait in type annotation
    if let Some(ty) = let_stmt.ty() {
        var_info.is_impl_trait = matches!(ty, ast::Type::ImplTraitType(_));
    }

    // Detect initializer kind semantically using resolved type
    if let Some(init) = let_stmt.initializer() {
        let resolved_type = sema.type_of_pat(&pat).map(|ti| ti.original);
        var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, known_types, &init, resolved_type.as_ref()));
    }

    // Assign scope ID (simple incrementing for now)
    var_info.scope_id = Some(*scope_id);

    Some(var_info)
}

/// Classify the initializer expression using semantic analysis
/// 
/// This function uses the resolved type from rust-analyzer to determine
/// the initializer kind. Expression structure is used as context for
/// the semantic classification.
fn classify_initializer_semantic(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_types: &KnownTypes,
    expr: &ast::Expr,
    resolved_type: Option<&ra_ap_hir::Type>,
) -> String {
    // Get expression structure as context
    let expr_kind = classify_expr_structure(expr);
    
    // Always try semantic classification first using AdtId comparison
    if let Some(ty) = resolved_type {
        if let Some(semantic_kind) = classify_by_resolved_type_semantic(ty, known_types, &expr_kind) {
            return semantic_kind;
        }
    }
    
    // Fallback to macro-specific classification for macros (semantic)
    if let ast::Expr::MacroExpr(mac) = expr {
        return classify_macro_expr_semantic(sema, db, mac);
    }
    
    // Final fallback: expression structure
    expr_kind
}

/// Classify expression by its syntactic structure (AST node kind)
/// Used as context for semantic classification
fn classify_expr_structure(expr: &ast::Expr) -> String {
    match expr {
        ast::Expr::Literal(_) => "literal".to_string(),
        ast::Expr::CallExpr(_) => "call".to_string(),
        ast::Expr::MethodCallExpr(m) => {
            // Return method name for structural classification
            m.name_ref().map(|n| n.text().to_string()).unwrap_or_else(|| "method".to_string())
        }
        ast::Expr::BlockExpr(_) => "block".to_string(),
        ast::Expr::IfExpr(_) => "if".to_string(),
        ast::Expr::MatchExpr(_) => "match".to_string(),
        ast::Expr::ClosureExpr(_) => "closure".to_string(),
        ast::Expr::RefExpr(ref_expr) => {
            if ref_expr.mut_token().is_some() { "ref_mut".to_string() } else { "ref".to_string() }
        }
        ast::Expr::PathExpr(_) => "path".to_string(),
        ast::Expr::MacroExpr(_) => "macro".to_string(),
        ast::Expr::AwaitExpr(_) => "await".to_string(),
        ast::Expr::TryExpr(_) => "try".to_string(),
        ast::Expr::TupleExpr(_) => "tuple".to_string(),
        ast::Expr::ArrayExpr(_) => "array".to_string(),
        ast::Expr::IndexExpr(_) => "index".to_string(),
        ast::Expr::FieldExpr(_) => "field".to_string(),
        ast::Expr::CastExpr(_) => "cast".to_string(),
        ast::Expr::RecordExpr(_) => "struct_literal".to_string(),
        ast::Expr::RangeExpr(_) => "range".to_string(),
        ast::Expr::BinExpr(_) => "binary".to_string(),
        ast::Expr::ParenExpr(paren) => {
            paren.expr().map(|e| classify_expr_structure(&e)).unwrap_or_else(|| "paren".to_string())
        }
        ast::Expr::PrefixExpr(prefix) => {
            match prefix.op_kind() {
                Some(ast::UnaryOp::Deref) => "deref".to_string(),
                Some(ast::UnaryOp::Not) => "not".to_string(),
                Some(ast::UnaryOp::Neg) => "neg".to_string(),
                _ => "prefix".to_string(),
            }
        }
        ast::Expr::LetExpr(_) => "let_expr".to_string(),
        ast::Expr::UnderscoreExpr(_) => "underscore".to_string(),
        ast::Expr::LoopExpr(_) => "loop".to_string(),
        ast::Expr::WhileExpr(_) => "while".to_string(),
        ast::Expr::ForExpr(_) => "for".to_string(),
        ast::Expr::ContinueExpr(_) => "continue".to_string(),
        ast::Expr::BreakExpr(_) => "break".to_string(),
        ast::Expr::ReturnExpr(_) => "return".to_string(),
        ast::Expr::YieldExpr(_) => "yield".to_string(),
        ast::Expr::YeetExpr(_) => "yeet".to_string(),
        ast::Expr::AsmExpr(_) => "asm".to_string(),
        ast::Expr::BecomeExpr(_) => "become".to_string(),
        ast::Expr::FormatArgsExpr(_) => "format_args".to_string(),
        ast::Expr::OffsetOfExpr(_) => "offset_of".to_string(),
    }
}

/// Classify initializer by the resolved type using AdtId comparison (fully semantic)
/// Returns None if no specific classification applies
fn classify_by_resolved_type_semantic(ty: &ra_ap_hir::Type, known_types: &KnownTypes, expr_kind: &str) -> Option<String> {
    // Get the ADT for type-based classification using AdtId comparison
    if let Some(adt) = ty.as_adt() {
        // Use semantic AdtId comparison instead of string matching
        let type_class = known_types.classify(&adt).unwrap_or_else(|| {
            // Fallback for types not in KnownTypes (user-defined, etc.)
            match &adt {
                ra_ap_hir::Adt::Struct(_) => "user_struct",
                ra_ap_hir::Adt::Enum(_) => "user_enum",
                ra_ap_hir::Adt::Union(_) => "user_union",
            }
        });
        
        // Combine type class with expression kind for full classification
        let kind = match (type_class, expr_kind) {
            // Smart pointer creation vs cloning
            ("rc", "call") => "rc_new",
            ("rc", "clone") => "rc_clone",
            ("arc", "call") => "arc_new",
            ("arc", "clone") => "arc_clone",
            ("box", "call") => "box_new",
            ("weak", "call") => "weak_new",
            ("weak", "clone") => "weak_clone",
            ("weak", "downgrade") => "weak_downgrade",
            ("weak", "upgrade") => "weak_upgrade",
            
            // Interior mutability
            ("unsafe_cell", "call") => "unsafe_cell_new",
            ("cell", "call") => "cell_new",
            ("refcell", "call") => "refcell_new",
            ("ref_guard", "borrow") => "refcell_borrow",
            ("refmut_guard", "borrow_mut") => "refcell_borrow_mut",
            ("mutex", "call") => "mutex_new",
            ("mutex_guard", "lock") => "mutex_lock",
            ("rwlock", "call") => "rwlock_new",
            ("rwlock_read_guard", "read") => "rwlock_read",
            ("rwlock_write_guard", "write") => "rwlock_write",
            ("once_cell", "call") => "once_cell_new",
            ("once_lock", "call") => "once_lock_new",
            
            // Memory
            ("maybe_uninit", "call") => "maybe_uninit_new",
            ("maybe_uninit", _) => "maybe_uninit",
            ("manually_drop", "call") => "manually_drop_new",
            
            // Pin
            ("pin", "call") => "pin_new",
            
            // Collections
            ("vec", "call") => "vec_new",
            ("vec", "macro") => "vec_macro",
            ("vec", "clone") => "vec_clone",
            ("string", "call") => "string_new",
            ("string", "macro") => "string_macro",
            ("string", "clone") => "string_clone",
            ("hashmap", "call") => "hashmap_new",
            ("hashset", "call") => "hashset_new",
            
            // Cow
            ("cow", "call") => "cow_new",
            ("cow", "path") => "cow_variant",
            
            // Option/Result
            ("option", "call") => "option_some",
            ("option", "path") => "option_variant",
            ("result", "call") => "result_variant",
            ("result", "path") => "result_variant",
            
            // Channels
            ("channel_sender", _) | ("channel_receiver", _) => "channel_new",
            ("sync_channel_sender", _) => "sync_channel_new",
            
            // Paths/FFI
            ("pathbuf", "call") => "pathbuf_new",
            ("osstring", "call") => "osstring_new",
            ("cstring", "call") => "cstring_new",
            
            // NonNull
            ("nonnull", "call") => "nonnull_new",
            
            // User-defined types
            ("user_struct", _) => "user_struct",
            ("user_enum", _) => "user_enum",
            ("user_union", _) => "user_union",
            
            // Default: type_class + expression kind
            (tc, ek) => return Some(format!("{}_{}", tc, ek)),
        };
        
        return Some(kind.to_string());
    }
    
    // Check for primitive types
    if let Some(builtin) = ty.as_builtin() {
        if builtin.is_int() || builtin.is_uint() || builtin.is_float() 
            || builtin.is_char() || builtin.is_bool() {
            return Some("primitive".to_string());
        }
        if builtin.is_str() {
            return Some("str".to_string());
        }
    }
    
    // Check for closures
    if ty.is_closure() {
        return Some("closure".to_string());
    }
    
    // Check for tuples
    if ty.is_tuple() {
        return Some("tuple".to_string());
    }
    
    // Check for function pointers
    if ty.is_fn() {
        return Some("fn_ptr".to_string());
    }
    
    // Check for arrays
    if ty.is_array() {
        return Some("array".to_string());
    }
    
    // Check for slices
    if ty.is_slice() {
        return Some("slice".to_string());
    }
    
    // Check for references
    if ty.is_reference() {
        if ty.is_mutable_reference() {
            return Some("ref_mut".to_string());
        }
        return Some("ref".to_string());
    }
    
    // Check for raw pointers
    if ty.is_raw_ptr() {
        return Some("raw_ptr".to_string());
    }
    
    None
}

/// Classify macro expressions
/// Classify macro expression using semantic resolution
fn classify_macro_expr_semantic(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    mac: &ast::MacroExpr,
) -> String {
    let Some(macro_call) = mac.macro_call() else {
        return "macro".to_string();
    };
    
    // Try semantic resolution first
    if let Some(resolved) = sema.resolve_macro_call(&macro_call) {
        let module_path = get_module_path(&resolved.module(db), db);
        let name = resolved.name(db).display_no_db(Edition::Edition2021).to_string();
        
        // Classify by semantic path
        return match (module_path.as_str(), name.as_str()) {
            (m, "vec") if m.contains("vec") => "vec_macro".to_string(),
            (m, "format") if m.contains("fmt") => "format_macro".to_string(),
            (m, "format_args") if m.contains("fmt") => "format_macro".to_string(),
            (m, n) if m.contains("io") && matches!(n, "println" | "print" | "eprintln" | "eprint") => "print_macro".to_string(),
            (m, "panic") if m.contains("panic") => "panic_macro".to_string(),
            (m, n) if m.contains("assert") && n.starts_with("assert") => "assert_macro".to_string(),
            (m, "pin") if m.contains("pin") => "pin_macro".to_string(),
            _ => format!("{}::{}", module_path, name),
        };
    }
    
    // Fallback to syntactic if resolution fails
    let Some(path) = macro_call.path() else {
        return "macro".to_string();
    };
    let macro_name = path.syntax().text().to_string();
    format!("macro:{}", macro_name)
}

/// Analyze a static or const declaration
fn analyze_static_or_const(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_types: &KnownTypes,
    node: &ra_ap_syntax::SyntaxNode,
    relative_path: &str,
    source_file: &ast::SourceFile,
) -> Option<VariableTypeInfo> {
    // Try to cast as Static first, then Const
    let (name_token, ty_node, body_expr) = if let Some(static_item) = ast::Static::cast(node.clone()) {
        (static_item.name()?, static_item.ty(), static_item.body())
    } else if let Some(const_item) = ast::Const::cast(node.clone()) {
        (const_item.name()?, const_item.ty(), const_item.body())
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
            populate_type_info(&mut var_info, &ty, db, known_types);
            
            // Classify initializer if body exists
            if let Some(expr) = body_expr {
                var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, known_types, &expr, Some(&ty)));
            }
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

/// Extract the variable name from a pattern (handles mut, tuple, etc.)
fn extract_pattern_name(pat: &ast::Pat) -> String {
    match pat {
        ast::Pat::IdentPat(ident) => {
            // Get just the identifier name, not "mut x"
            ident.name().map(|n| n.text().to_string()).unwrap_or_else(|| pat.syntax().text().to_string())
        }
        ast::Pat::TuplePat(_) => {
            // For tuples, keep the full pattern text for now
            pat.syntax().text().to_string()
        }
        _ => pat.syntax().text().to_string(),
    }
}

/// Extract individual element names from a tuple pattern string like "(tx, rx)"
fn extract_tuple_elements(tuple_pat: &str) -> Vec<String> {
    tuple_pat
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "_")
        .collect()
}

/// Populate type info from a resolved type using semantic analysis only
fn populate_type_info(var_info: &mut VariableTypeInfo, ty: &ra_ap_hir::Type, db: &RootDatabase, known_types: &KnownTypes) {
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
    
    // === ADT-based classification using AdtId comparison (fully semantic) ===
    if let Some(adt) = ty.as_adt() {
        var_info.is_union = matches!(adt, Adt::Union(_));
        
        // Use KnownTypes for semantic AdtId comparison instead of path strings
        known_types.set_flags(var_info, &adt);
    } else {
        // For tuples/arrays, check if any element is a known type
        for inner in ty.type_arguments() {
            if let Some(adt) = inner.as_adt() {
                known_types.set_flags_or(var_info, &adt);
            }
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

// =============================================================================
// METHOD CALL TRACKING (Phase 1 of Semantic Expansion)
// =============================================================================

/// Analyze method calls on tracked variables and populate their method_calls field
pub fn analyze_method_calls(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    variables: &mut [VariableTypeInfo],
) {
    // Build a map of variable name -> indices for quick lookup
    let mut var_indices: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (idx, var) in variables.iter().enumerate() {
        var_indices.entry(var.name.clone()).or_default().push(idx);
        // Also index individual tuple elements: "(tx, rx)" -> "tx", "rx"
        if var.is_tuple_binding {
            for elem in extract_tuple_elements(&var.name) {
                var_indices.entry(elem).or_default().push(idx);
            }
        }
    }

    for node in source_file.syntax().descendants() {
        let Some(method_call) = ast::MethodCallExpr::cast(node) else {
            continue;
        };

        // Extract receiver name
        let Some(receiver_name) = extract_receiver_name(&method_call) else {
            continue;
        };

        // Find matching variable(s)
        let Some(indices) = var_indices.get(&receiver_name) else {
            continue;
        };

        // Get method info
        let Some(method_name) = method_call.name_ref().map(|n| n.text().to_string()) else {
            continue;
        };

        let (call_line, column) = get_method_call_location(&method_call, source_file);

        // Get receiver type (semantic)
        let receiver_ty = method_call
            .receiver()
            .and_then(|r| sema.type_of_expr(&r))
            .map(|ti| ti.original);

        // Get receiver type display string for output
        let receiver_type = receiver_ty
            .as_ref()
            .map(|ty| ty.display(db, Edition::Edition2021).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get result type
        let result_type = sema
            .type_of_expr(&ast::Expr::MethodCallExpr(method_call.clone()))
            .map(|ti| ti.original.display(db, Edition::Edition2021).to_string());

        // Resolve self borrow type (semantic)
        let self_borrow = resolve_self_borrow(sema, &method_call, db);

        // Get operation as the canonical method path (fully semantic)
        let operation = resolve_method_path(sema, &method_call, db);

        let method_info = MethodCallInfo {
            method: method_name,
            line: call_line,
            column,
            operation,
            self_borrow,
            receiver_type,
            result_type,
        };

        // Find the most recent variable declared before this method call
        // This handles shadowing correctly
        let best_idx = indices.iter()
            .filter(|&&idx| variables[idx].line <= call_line)
            .max_by_key(|&&idx| variables[idx].line);
        
        if let Some(&idx) = best_idx {
            variables[idx].method_calls.push(method_info);
        }
    }
}

// =============================================================================
// STANDALONE EXPRESSION TRACKING (Phase 2 of Semantic Expansion)
// =============================================================================

/// Analyze standalone function calls (thread::spawn, drop, transmute, etc.)
/// Uses semantic function identity comparison (FunctionId), not string matching.
fn analyze_expressions(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    tracked_functions: &TrackedFunctions,
    source_file: &ast::SourceFile,
) -> Vec<ExpressionInfo> {
    let mut expressions = Vec::new();

    for node in source_file.syntax().descendants() {
        // Handle function calls: drop(x), thread::spawn(|| {}), transmute(x)
        if let Some(call) = ast::CallExpr::cast(node) {
            if let Some(expr_info) = analyze_call_expr(sema, db, tracked_functions, &call, source_file) {
                expressions.push(expr_info);
            }
        }
    }

    expressions
}

/// Analyze a function call expression using semantic function identity
fn analyze_call_expr(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    tracked_functions: &TrackedFunctions,
    call: &ast::CallExpr,
    source_file: &ast::SourceFile,
) -> Option<ExpressionInfo> {
    let callee = call.expr()?;
    
    // Get the path expression (e.g., std::mem::drop, thread::spawn)
    let path_expr = match &callee {
        ast::Expr::PathExpr(p) => p.clone(),
        _ => return None,
    };
    
    let path = path_expr.path()?;
    
    // Resolve the function to its semantic identity (FunctionId)
    let resolved = sema.resolve_path(&path)?;
    
    // Extract the Function from the resolution
    let func = match &resolved {
        ra_ap_hir::PathResolution::Def(ra_ap_hir::ModuleDef::Function(f)) => f,
        _ => return None,
    };
    
    // Check if this function is one we track (by FunctionId, not string)
    let canonical_path = tracked_functions.get_path(func)?;
    
    let (line, column) = get_call_location(call, source_file);
    
    // Extract argument - variable name, or captured variables for closures
    let argument = call.arg_list()
        .and_then(|args| args.args().next())
        .and_then(|arg| extract_argument_info(sema, &arg));
    
    // Get result type
    let result_type = sema.type_of_expr(&ast::Expr::CallExpr(call.clone()))
        .map(|ti| ti.original.display(db, Edition::Edition2021).to_string());

    Some(ExpressionInfo {
        line,
        column,
        kind: "function_call".to_string(),
        path: Some(canonical_path.clone()),
        operation: canonical_path.clone(),  // Operation IS the canonical path (fully semantic)
        argument,
        result_type,
    })
}

/// Extract argument info - variable name or captured variables for closures
fn extract_argument_info(sema: &Semantics<'_, RootDatabase>, arg: &ast::Expr) -> Option<String> {
    match arg {
        // Simple variable: drop(x) -> "x"
        ast::Expr::PathExpr(p) => {
            p.path()?.segment()?.name_ref().map(|n| n.text().to_string())
        }
        // Closure: spawn(|| {}) or spawn(move || {}) -> extract captured variables
        ast::Expr::ClosureExpr(closure) => {
            let captured = extract_closure_captures(sema, closure);
            if captured.is_empty() {
                Some("<closure>".to_string())
            } else {
                Some(format!("<closure captures: {}>", captured.join(", ")))
            }
        }
        // Reference: &x or &mut x
        ast::Expr::RefExpr(ref_expr) => {
            ref_expr.expr().and_then(|e| extract_argument_info(sema, &e))
        }
        _ => None,
    }
}

/// Extract variable names captured by a closure (semantic)
fn extract_closure_captures(sema: &Semantics<'_, RootDatabase>, closure: &ast::ClosureExpr) -> Vec<String> {
    let mut captures = Vec::new();
    
    // Get closure parameter names to exclude them
    let mut param_names: Vec<String> = Vec::new();
    if let Some(param_list) = closure.param_list() {
        for param in param_list.params() {
            if let Some(pat) = param.pat() {
                if let ast::Pat::IdentPat(ident) = pat {
                    if let Some(name) = ident.name() {
                        param_names.push(name.text().to_string());
                    }
                }
            }
        }
    }
    
    // Get the closure body
    let Some(body) = closure.body() else {
        return captures;
    };
    
    // Find all path expressions in the closure body that reference outer variables
    for node in body.syntax().descendants() {
        // Skip nested closure parameters
        if let Some(nested_closure) = ast::ClosureExpr::cast(node.clone()) {
            if let Some(nested_params) = nested_closure.param_list() {
                for param in nested_params.params() {
                    if let Some(pat) = param.pat() {
                        if let ast::Pat::IdentPat(ident) = pat {
                            if let Some(name) = ident.name() {
                                param_names.push(name.text().to_string());
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(path_expr) = ast::PathExpr::cast(node) {
            if let Some(path) = path_expr.path() {
                // Only simple identifiers (no :: qualifier)
                if path.qualifier().is_none() {
                    if let Some(segment) = path.segment() {
                        if let Some(name_ref) = segment.name_ref() {
                            let name = name_ref.text().to_string();
                            // Skip closure parameters and already captured
                            if param_names.contains(&name) || captures.contains(&name) {
                                continue;
                            }
                            // Use semantic resolution: if it resolves to a local variable, it's a capture
                            if let Some(resolved) = sema.resolve_path(&path) {
                                use ra_ap_hir::PathResolution;
                                match resolved {
                                    PathResolution::Local(_) => {
                                        captures.push(name);
                                    }
                                    // Not a local - it's a function, type, const, etc.
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    captures
}

/// Get location of a call expression
fn get_call_location(call: &ast::CallExpr, source_file: &ast::SourceFile) -> (u32, u32) {
    let range = call.syntax().text_range();
    get_location(&range, source_file)
}

/// Extract the receiver variable name from a method call expression
/// Only returns a name if the receiver is a direct variable reference.
/// Does NOT recurse into chained method calls - those are intermediate values.
fn extract_receiver_name(method_call: &ast::MethodCallExpr) -> Option<String> {
    let receiver = method_call.receiver()?;
    
    // Handle simple identifier: `cell.set(42)` -> "cell"
    if let ast::Expr::PathExpr(path_expr) = &receiver {
        if let Some(path) = path_expr.path() {
            if path.qualifier().is_none() {
                return path.segment()?.name_ref().map(|n| n.text().to_string());
            }
        }
    }
    
    // Chained method calls: `mutex.lock().unwrap()` 
    // The unwrap() is on Result, not on mutex - return None
    // This is NOT a direct variable reference
    if matches!(&receiver, ast::Expr::MethodCallExpr(_)) {
        return None;
    }
    
    // Handle field access: `self.cell.set(42)` -> "cell"
    if let ast::Expr::FieldExpr(field) = &receiver {
        return field.name_ref().map(|n| n.text().to_string());
    }
    
    None
}

/// Resolve the self borrow type of a method call using rust-analyzer
fn resolve_self_borrow(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> Option<String> {
    let func = sema.resolve_method_call(method_call)?;
    let self_param = func.self_param(db)?;
    
    use ra_ap_hir::Access;
    match self_param.access(db) {
        Access::Shared => Some("immutable".to_string()),
        Access::Exclusive => Some("mutable".to_string()),
        Access::Owned => Some("consuming".to_string()),
    }
}

/// Resolve the canonical path of a method call using rust-analyzer (fully semantic)
/// Returns the full path like "alloc::vec::Vec::push" or "core::cell::Cell::set"
fn resolve_method_path(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> Option<String> {
    let func = sema.resolve_method_call(method_call)?;
    
    // Get the module containing this function
    let module = func.module(db);
    
    // Build the module path
    let mut segments: Vec<String> = module.path_to_root(db)
        .into_iter()
        .filter_map(|m| m.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()))
        .collect();
    segments.reverse();
    
    // Get crate name
    let krate = module.krate();
    let crate_name = krate.display_name(db)
        .map(|n| n.to_string())
        .unwrap_or_default();
    
    if !crate_name.is_empty() {
        segments.insert(0, crate_name);
    }
    
    // Add the function name
    let fn_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
    segments.push(fn_name);
    
    Some(segments.join("::"))
}

/// Get line and column for a method call expression
fn get_method_call_location(method_call: &ast::MethodCallExpr, source_file: &ast::SourceFile) -> (u32, u32) {
    let range = method_call.syntax().text_range();
    get_location(&range, source_file)
}
