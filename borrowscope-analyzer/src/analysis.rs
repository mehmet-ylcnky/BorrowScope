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
                if let Some(mut var_info) = analyze_let_stmt(sema, db, &node, relative_path, &source_file, &mut scope_id) {
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

    // Detect initializer kind semantically using resolved type
    if let Some(init) = let_stmt.initializer() {
        let resolved_type = sema.type_of_pat(&pat).map(|ti| ti.original);
        var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, &init, resolved_type.as_ref()));
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
    _sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    expr: &ast::Expr,
    resolved_type: Option<&ra_ap_hir::Type>,
) -> String {
    // Get expression structure as context
    let expr_kind = classify_expr_structure(expr);
    
    // Always try semantic classification first
    if let Some(ty) = resolved_type {
        if let Some(semantic_kind) = classify_by_resolved_type(ty, db, &expr_kind) {
            return semantic_kind;
        }
    }
    
    // Fallback to macro-specific classification for macros
    if let ast::Expr::MacroExpr(mac) = expr {
        return classify_macro_expr(mac);
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

/// Classify initializer by the resolved type (fully semantic)
/// Returns None if no specific classification applies
fn classify_by_resolved_type(ty: &ra_ap_hir::Type, db: &RootDatabase, expr_kind: &str) -> Option<String> {
    // Get the ADT path for type-based classification
    if let Some(adt) = ty.as_adt() {
        let path = get_adt_path(&adt, db)?;
        
        // Classify based on canonical type path
        let type_class = match path.as_str() {
            // Smart pointers
            "alloc::rc::Rc" | "std::rc::Rc" => "rc",
            "alloc::sync::Arc" | "std::sync::Arc" => "arc",
            "alloc::boxed::Box" | "std::boxed::Box" => "box",
            "alloc::rc::Weak" | "std::rc::Weak" | "alloc::sync::Weak" | "std::sync::Weak" => "weak",
            
            // Interior mutability
            "core::cell::UnsafeCell" | "std::cell::UnsafeCell" => "unsafe_cell",
            "core::cell::Cell" | "std::cell::Cell" => "cell",
            "core::cell::RefCell" | "std::cell::RefCell" => "refcell",
            "std::sync::Mutex" | "std::sync::poison::mutex::Mutex" | "std::sync::mutex::Mutex" => "mutex",
            "std::sync::RwLock" | "std::sync::poison::rwlock::RwLock" | "std::sync::rwlock::RwLock" => "rwlock",
            "core::cell::OnceCell" | "std::cell::OnceCell" | "core::cell::once::OnceCell" => "once_cell",
            "std::sync::OnceLock" | "std::sync::once_lock::OnceLock" => "once_lock",
            
            // Guards
            "core::cell::Ref" | "std::cell::Ref" => "ref_guard",
            "core::cell::RefMut" | "std::cell::RefMut" => "refmut_guard",
            "std::sync::MutexGuard" | "std::sync::poison::mutex::MutexGuard" | "std::sync::mutex::MutexGuard" => "mutex_guard",
            "std::sync::RwLockReadGuard" | "std::sync::poison::rwlock::RwLockReadGuard" | "std::sync::rwlock::RwLockReadGuard" => "rwlock_read_guard",
            "std::sync::RwLockWriteGuard" | "std::sync::poison::rwlock::RwLockWriteGuard" | "std::sync::rwlock::RwLockWriteGuard" => "rwlock_write_guard",
            
            // Memory
            "core::mem::MaybeUninit" | "std::mem::MaybeUninit" | "core::mem::maybe_uninit::MaybeUninit" => "maybe_uninit",
            "core::mem::ManuallyDrop" | "std::mem::ManuallyDrop" | "core::mem::manually_drop::ManuallyDrop" => "manually_drop",
            
            // Pin
            "core::pin::Pin" | "std::pin::Pin" => "pin",
            
            // Collections
            "alloc::vec::Vec" | "std::vec::Vec" => "vec",
            "alloc::string::String" | "std::string::String" => "string",
            "std::collections::HashMap" | "std::collections::hash::map::HashMap" => "hashmap",
            "std::collections::HashSet" | "std::collections::hash::set::HashSet" => "hashset",
            "std::collections::BTreeMap" | "alloc::collections::btree::map::BTreeMap" => "btreemap",
            "std::collections::BTreeSet" | "alloc::collections::btree::set::BTreeSet" => "btreeset",
            "std::collections::VecDeque" | "alloc::collections::vec_deque::VecDeque" => "vecdeque",
            "std::collections::LinkedList" | "alloc::collections::linked_list::LinkedList" => "linkedlist",
            "std::collections::BinaryHeap" | "alloc::collections::binary_heap::BinaryHeap" => "binaryheap",
            
            // Cow
            "alloc::borrow::Cow" | "std::borrow::Cow" => "cow",
            
            // Option/Result
            "core::option::Option" | "std::option::Option" => "option",
            "core::result::Result" | "std::result::Result" => "result",
            
            // Channels
            "std::sync::mpsc::Sender" => "channel_sender",
            "std::sync::mpsc::Receiver" => "channel_receiver",
            "std::sync::mpsc::SyncSender" => "sync_channel_sender",
            
            // Paths
            "std::path::PathBuf" | "std::path::pathbuf::PathBuf" => "pathbuf",
            "std::ffi::OsString" | "std::ffi::os_str::OsString" => "osstring",
            "std::ffi::CString" | "alloc::ffi::c_str::CString" | "std::ffi::c_str::CString" => "cstring",
            
            // Pointers
            "core::ptr::NonNull" | "std::ptr::NonNull" | "core::ptr::non_null::NonNull" => "nonnull",
            
            // Atomics
            "core::sync::atomic::AtomicBool" | "std::sync::atomic::AtomicBool" => "atomic_bool",
            "core::sync::atomic::AtomicI8" | "std::sync::atomic::AtomicI8" => "atomic_i8",
            "core::sync::atomic::AtomicI16" | "std::sync::atomic::AtomicI16" => "atomic_i16",
            "core::sync::atomic::AtomicI32" | "std::sync::atomic::AtomicI32" => "atomic_i32",
            "core::sync::atomic::AtomicI64" | "std::sync::atomic::AtomicI64" => "atomic_i64",
            "core::sync::atomic::AtomicIsize" | "std::sync::atomic::AtomicIsize" => "atomic_isize",
            "core::sync::atomic::AtomicU8" | "std::sync::atomic::AtomicU8" => "atomic_u8",
            "core::sync::atomic::AtomicU16" | "std::sync::atomic::AtomicU16" => "atomic_u16",
            "core::sync::atomic::AtomicU32" | "std::sync::atomic::AtomicU32" => "atomic_u32",
            "core::sync::atomic::AtomicU64" | "std::sync::atomic::AtomicU64" => "atomic_u64",
            "core::sync::atomic::AtomicUsize" | "std::sync::atomic::AtomicUsize" => "atomic_usize",
            "core::sync::atomic::AtomicPtr" | "std::sync::atomic::AtomicPtr" => "atomic_ptr",
            
            // Time
            "core::time::Duration" | "std::time::Duration" => "duration",
            "std::time::Instant" => "instant",
            "std::time::SystemTime" => "system_time",
            
            // IO
            "std::io::Cursor" | "std::io::cursor::Cursor" => "cursor",
            "std::io::BufReader" | "std::io::buffered::bufreader::BufReader" => "bufreader",
            "std::io::BufWriter" | "std::io::buffered::bufwriter::BufWriter" => "bufwriter",
            "std::fs::File" => "file",
            "std::io::Empty" | "std::io::util::Empty" => "io_empty",
            "std::io::Repeat" | "std::io::util::Repeat" => "io_repeat",
            "std::io::Sink" | "std::io::util::Sink" => "io_sink",
            
            // Ordering
            "core::cmp::Ordering" | "std::cmp::Ordering" => "ordering",
            
            // Poll
            "core::task::Poll" | "std::task::Poll" | "core::task::poll::Poll" => "poll",
            
            // Location
            "core::panic::Location" | "std::panic::Location" | "core::panic::location::Location" => "location",
            
            // Ranges (public API and internal module paths)
            "core::ops::Range" | "std::ops::Range" | "core::ops::range::Range" => "range_type",
            "core::ops::RangeFrom" | "std::ops::RangeFrom" | "core::ops::range::RangeFrom" => "range_from",
            "core::ops::RangeTo" | "std::ops::RangeTo" | "core::ops::range::RangeTo" => "range_to",
            "core::ops::RangeInclusive" | "std::ops::RangeInclusive" | "core::ops::range::RangeInclusive" => "range_inclusive",
            "core::ops::RangeToInclusive" | "std::ops::RangeToInclusive" | "core::ops::range::RangeToInclusive" => "range_to_inclusive",
            "core::ops::RangeFull" | "std::ops::RangeFull" | "core::ops::range::RangeFull" => "range_full",
            
            // User-defined types - classify by ADT kind
            _ => match &adt {
                ra_ap_hir::Adt::Struct(_) => "user_struct",
                ra_ap_hir::Adt::Enum(_) => "user_enum",
                ra_ap_hir::Adt::Union(_) => "user_union",
            },
        };
        
        // Combine type class with expression kind for full classification
        // e.g., "rc" + "call" -> "rc_new", "rc" + "clone" -> "rc_clone"
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
            ("cell", "get") => "cell_get",
            ("cell", "set") => "cell_set",
            ("cell", "replace") => "cell_replace",
            ("cell", "take") => "cell_take",
            ("refcell", "call") => "refcell_new",
            ("ref_guard", "borrow") => "refcell_borrow",
            ("refmut_guard", "borrow_mut") => "refcell_borrow_mut",
            ("mutex", "call") => "mutex_new",
            ("mutex_guard", "lock") => "mutex_lock",
            ("mutex_guard", "try_lock") => "mutex_try_lock",
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
            ("string", "call") => "string_new",
            ("hashmap", "call") => "hashmap_new",
            ("hashset", "call") => "hashset_new",
            ("btreemap", "call") => "btreemap_new",
            ("btreeset", "call") => "btreeset_new",
            ("vecdeque", "call") => "vecdeque_new",
            ("linkedlist", "call") => "linkedlist_new",
            ("binaryheap", "call") => "binaryheap_new",
            
            // Cow
            ("cow", "call") => "cow_new",
            ("cow", "path") => "cow_variant",
            
            // Option/Result
            ("option", "call") => "option_some",
            ("option", "path") => "option_variant",
            ("option", "unwrap") => "unwrap",
            ("option", "expect") => "expect",
            ("option", "map") => "map",
            ("result", "call") => "result_variant",
            ("result", "path") => "result_variant",
            ("result", "unwrap") => "unwrap",
            ("result", "expect") => "expect",
            ("result", "ok") => "result_ok_method",
            ("result", "err") => "result_err_method",
            
            // Channels
            ("channel_sender", _) | ("channel_receiver", _) => "channel_new",
            ("sync_channel_sender", _) => "sync_channel_new",
            
            // Paths
            ("pathbuf", "call") => "pathbuf_new",
            ("osstring", "call") => "osstring_new",
            ("cstring", "call") => "cstring_new",
            
            // Pointers
            ("nonnull", "call") => "nonnull_new",
            
            // Atomics
            (atomic, "call") if atomic.starts_with("atomic_") => {
                return Some(format!("{}_new", atomic));
            }
            (atomic, method) if atomic.starts_with("atomic_") => {
                return Some(format!("atomic_{}", method));
            }
            
            // Time
            ("duration", "call") => "duration_new",
            ("duration", method) if method.starts_with("as_") => {
                return Some(format!("duration_{}", method));
            }
            ("instant", "call") => "instant_now",
            ("instant", "elapsed") => "instant_elapsed",
            ("instant", "duration_since") => "instant_duration_since",
            ("system_time", "call") => "system_time_now",
            
            // IO
            ("cursor", "call") => "cursor_new",
            ("bufreader", "call") => "bufreader_new",
            ("bufwriter", "call") => "bufwriter_new",
            ("file", "open") => "file_open",
            ("file", "create") => "file_create",
            
            // Ordering/Poll/Location
            ("ordering", _) => "ordering",
            ("poll", _) => "poll",
            ("location", _) => "location",
            
            // Ranges
            (range_type, _) if range_type.starts_with("range") => range_type,
            
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
    
    // Check for impl Trait types
    if let Some(mut traits) = ty.as_impl_traits(db) {
        if let Some(first_trait) = traits.next() {
            let trait_name = first_trait.name(db).as_str().to_lowercase();
            return Some(format!("impl_{}", trait_name));
        }
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
fn classify_macro_expr(mac: &ast::MacroExpr) -> String {
    let Some(macro_call) = mac.macro_call() else {
        return "macro".to_string();
    };
    
    let Some(path) = macro_call.path() else {
        return "macro".to_string();
    };
    
    let macro_name = path.syntax().text().to_string();
    
    match macro_name.as_str() {
        "vec" => "vec_macro".to_string(),
        "format" => "format_macro".to_string(),
        "println" | "print" | "eprintln" | "eprint" => "print_macro".to_string(),
        "panic" => "panic_macro".to_string(),
        "assert" | "assert_eq" | "assert_ne" => "assert_macro".to_string(),
        "pin" => "pin_macro".to_string(),
        _ => "macro".to_string(),
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
            populate_type_info(&mut var_info, &ty, db);
            
            // Classify initializer if body exists
            if let Some(expr) = body_expr {
                var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, &expr, Some(&ty)));
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
    
    // OnceCell/OnceLock
    var_info.is_once_cell = path == "core::cell::once::OnceCell" || path == "std::cell::OnceCell"
        || path == "std::sync::once_lock::OnceLock" || path == "std::sync::OnceLock"
        || path == "core::cell::lazy::LazyCell" || path == "std::sync::lazy_lock::LazyLock";
    
    // MaybeUninit
    var_info.is_maybe_uninit = path == "core::mem::maybe_uninit::MaybeUninit" || path == "std::mem::MaybeUninit";
    
    // Channels (Sender/Receiver)
    var_info.is_channel = path == "std::sync::mpsc::Sender" || path == "std::sync::mpsc::Receiver"
        || path == "std::sync::mpsc::SyncSender";
    
    // FFI types
    var_info.is_extern_type = path == "core::ffi::c_void" || path == "std::ffi::c_void"
        || path == "core::ffi::CStr" || path == "std::ffi::CStr"
        || path == "alloc::ffi::CString" || path == "std::ffi::CString"
        || path == "std::ffi::OsStr" || path == "std::ffi::OsString";
}
