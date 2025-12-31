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
        ast::Expr::CallExpr(call) => classify_call_expr(call),
        ast::Expr::MethodCallExpr(method) => classify_method_call(method),
        ast::Expr::BlockExpr(_) => "block".to_string(),
        ast::Expr::IfExpr(_) => "if".to_string(),
        ast::Expr::MatchExpr(_) => "match".to_string(),
        ast::Expr::ClosureExpr(_) => "closure".to_string(),
        ast::Expr::RefExpr(ref_expr) => {
            if ref_expr.mut_token().is_some() {
                "ref_mut".to_string()
            } else {
                "ref".to_string()
            }
        }
        ast::Expr::PathExpr(path) => classify_path_expr(path),
        ast::Expr::MacroExpr(mac) => classify_macro_expr(mac),
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
            // Unwrap parentheses
            paren.expr().map(|e| classify_initializer(&e)).unwrap_or_else(|| "paren".to_string())
        }
        ast::Expr::PrefixExpr(prefix) => {
            if prefix.op_kind() == Some(ast::UnaryOp::Deref) {
                "deref".to_string()
            } else if prefix.op_kind() == Some(ast::UnaryOp::Not) {
                "not".to_string()
            } else if prefix.op_kind() == Some(ast::UnaryOp::Neg) {
                "neg".to_string()
            } else {
                "prefix".to_string()
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

/// Classify a function call expression to detect specific patterns
fn classify_call_expr(call: &ast::CallExpr) -> String {
    let Some(callee) = call.expr() else {
        return "call".to_string();
    };
    
    // Get the path of the callee
    let path_str = match &callee {
        ast::Expr::PathExpr(path) => path.path().map(|p| p.syntax().text().to_string()),
        _ => None,
    };
    
    let Some(path) = path_str else {
        return "call".to_string();
    };
    
    // Match specific patterns
    match path.as_str() {
        // Smart pointer constructors
        "Rc::new" | "std::rc::Rc::new" | "alloc::rc::Rc::new" => "rc_new".to_string(),
        "Arc::new" | "std::sync::Arc::new" | "alloc::sync::Arc::new" => "arc_new".to_string(),
        "Box::new" | "std::boxed::Box::new" | "alloc::boxed::Box::new" => "box_new".to_string(),
        "Box::pin" | "std::boxed::Box::pin" => "box_pin".to_string(),
        
        // Interior mutability
        "UnsafeCell::new" | "std::cell::UnsafeCell::new" | "core::cell::UnsafeCell::new" => "unsafe_cell_new".to_string(),
        "RefCell::new" | "std::cell::RefCell::new" | "core::cell::RefCell::new" => "refcell_new".to_string(),
        "Cell::new" | "std::cell::Cell::new" | "core::cell::Cell::new" => "cell_new".to_string(),
        "Mutex::new" | "std::sync::Mutex::new" => "mutex_new".to_string(),
        "RwLock::new" | "std::sync::RwLock::new" => "rwlock_new".to_string(),
        
        // OnceCell/OnceLock
        "OnceCell::new" | "std::cell::OnceCell::new" | "core::cell::OnceCell::new" => "once_cell_new".to_string(),
        "OnceLock::new" | "std::sync::OnceLock::new" => "once_lock_new".to_string(),
        
        // MaybeUninit
        "MaybeUninit::uninit" | "std::mem::MaybeUninit::uninit" | "core::mem::MaybeUninit::uninit" => "maybe_uninit_uninit".to_string(),
        "MaybeUninit::new" | "std::mem::MaybeUninit::new" | "core::mem::MaybeUninit::new" => "maybe_uninit_new".to_string(),
        "MaybeUninit::zeroed" | "std::mem::MaybeUninit::zeroed" | "core::mem::MaybeUninit::zeroed" => "maybe_uninit_zeroed".to_string(),
        
        // Channels
        "channel" | "std::sync::mpsc::channel" => "channel_new".to_string(),
        "sync_channel" | "std::sync::mpsc::sync_channel" => "sync_channel_new".to_string(),
        
        // Pin
        "Pin::new" | "std::pin::Pin::new" | "core::pin::Pin::new" => "pin_new".to_string(),
        "Pin::new_unchecked" | "std::pin::Pin::new_unchecked" => "pin_new_unchecked".to_string(),
        
        // Cow
        "Cow::Borrowed" | "std::borrow::Cow::Borrowed" => "cow_borrowed".to_string(),
        "Cow::Owned" | "std::borrow::Cow::Owned" => "cow_owned".to_string(),
        
        // Weak
        "Weak::new" | "std::rc::Weak::new" | "std::sync::Weak::new" => "weak_new".to_string(),
        
        // Option/Result constructors
        "Some" | "core::option::Option::Some" | "std::option::Option::Some" => "option_some".to_string(),
        "None" | "core::option::Option::None" | "std::option::Option::None" => "option_none".to_string(),
        "Ok" | "core::result::Result::Ok" | "std::result::Result::Ok" => "result_ok".to_string(),
        "Err" | "core::result::Result::Err" | "std::result::Result::Err" => "result_err".to_string(),
        
        // String constructors
        "String::new" | "std::string::String::new" | "alloc::string::String::new" => "string_new".to_string(),
        "String::from" | "std::string::String::from" | "alloc::string::String::from" => "string_from".to_string(),
        "String::with_capacity" => "string_with_capacity".to_string(),
        
        // Vec constructors
        "Vec::new" | "std::vec::Vec::new" | "alloc::vec::Vec::new" => "vec_new".to_string(),
        "Vec::with_capacity" | "std::vec::Vec::with_capacity" => "vec_with_capacity".to_string(),
        
        // Collection constructors
        "HashMap::new" | "std::collections::HashMap::new" => "hashmap_new".to_string(),
        "HashSet::new" | "std::collections::HashSet::new" => "hashset_new".to_string(),
        "BTreeMap::new" | "std::collections::BTreeMap::new" => "btreemap_new".to_string(),
        "BTreeSet::new" | "std::collections::BTreeSet::new" => "btreeset_new".to_string(),
        "VecDeque::new" | "std::collections::VecDeque::new" => "vecdeque_new".to_string(),
        "LinkedList::new" | "std::collections::LinkedList::new" => "linkedlist_new".to_string(),
        "BinaryHeap::new" | "std::collections::BinaryHeap::new" => "binaryheap_new".to_string(),
        
        // Path constructors
        "PathBuf::new" | "std::path::PathBuf::new" => "pathbuf_new".to_string(),
        "PathBuf::from" | "std::path::PathBuf::from" => "pathbuf_from".to_string(),
        "OsString::new" | "std::ffi::OsString::new" => "osstring_new".to_string(),
        "OsString::from" | "std::ffi::OsString::from" => "osstring_from".to_string(),
        "CString::new" | "std::ffi::CString::new" => "cstring_new".to_string(),
        
        // Raw pointer constructors
        "ptr::null" | "std::ptr::null" | "core::ptr::null" => "ptr_null".to_string(),
        "ptr::null_mut" | "std::ptr::null_mut" | "core::ptr::null_mut" => "ptr_null_mut".to_string(),
        "NonNull::new" | "std::ptr::NonNull::new" | "core::ptr::NonNull::new" => "nonnull_new".to_string(),
        "NonNull::dangling" | "std::ptr::NonNull::dangling" => "nonnull_dangling".to_string(),
        
        // Default trait
        "Default::default" | "std::default::Default::default" | "core::default::Default::default" => "default".to_string(),
        
        // Clone trait
        p if p.ends_with("::clone") => {
            if p.starts_with("Rc::") || p.contains("rc::Rc::") {
                "rc_clone".to_string()
            } else if p.starts_with("Arc::") || p.contains("sync::Arc::") {
                "arc_clone".to_string()
            } else {
                "clone".to_string()
            }
        }
        
        // Raw pointer operations
        "Box::into_raw" | "std::boxed::Box::into_raw" => "box_into_raw".to_string(),
        "Box::from_raw" | "std::boxed::Box::from_raw" => "box_from_raw".to_string(),
        
        // ManuallyDrop
        "ManuallyDrop::new" | "std::mem::ManuallyDrop::new" | "core::mem::ManuallyDrop::new" => "manually_drop_new".to_string(),
        "ManuallyDrop::into_inner" | "std::mem::ManuallyDrop::into_inner" => "manually_drop_into_inner".to_string(),
        
        // Atomics
        "AtomicBool::new" | "std::sync::atomic::AtomicBool::new" | "core::sync::atomic::AtomicBool::new" => "atomic_bool_new".to_string(),
        "AtomicI8::new" | "std::sync::atomic::AtomicI8::new" => "atomic_i8_new".to_string(),
        "AtomicI16::new" | "std::sync::atomic::AtomicI16::new" => "atomic_i16_new".to_string(),
        "AtomicI32::new" | "std::sync::atomic::AtomicI32::new" => "atomic_i32_new".to_string(),
        "AtomicI64::new" | "std::sync::atomic::AtomicI64::new" => "atomic_i64_new".to_string(),
        "AtomicIsize::new" | "std::sync::atomic::AtomicIsize::new" => "atomic_isize_new".to_string(),
        "AtomicU8::new" | "std::sync::atomic::AtomicU8::new" => "atomic_u8_new".to_string(),
        "AtomicU16::new" | "std::sync::atomic::AtomicU16::new" => "atomic_u16_new".to_string(),
        "AtomicU32::new" | "std::sync::atomic::AtomicU32::new" => "atomic_u32_new".to_string(),
        "AtomicU64::new" | "std::sync::atomic::AtomicU64::new" => "atomic_u64_new".to_string(),
        "AtomicUsize::new" | "std::sync::atomic::AtomicUsize::new" => "atomic_usize_new".to_string(),
        "AtomicPtr::new" | "std::sync::atomic::AtomicPtr::new" => "atomic_ptr_new".to_string(),
        
        // Time
        "Duration::new" | "std::time::Duration::new" | "core::time::Duration::new" => "duration_new".to_string(),
        "Duration::from_secs" | "std::time::Duration::from_secs" => "duration_from_secs".to_string(),
        "Duration::from_millis" | "std::time::Duration::from_millis" => "duration_from_millis".to_string(),
        "Duration::from_micros" | "std::time::Duration::from_micros" => "duration_from_micros".to_string(),
        "Duration::from_nanos" | "std::time::Duration::from_nanos" => "duration_from_nanos".to_string(),
        "Duration::from_secs_f32" | "std::time::Duration::from_secs_f32" => "duration_from_secs_f".to_string(),
        "Duration::from_secs_f64" | "std::time::Duration::from_secs_f64" => "duration_from_secs_f".to_string(),
        "Instant::now" | "std::time::Instant::now" => "instant_now".to_string(),
        "SystemTime::now" | "std::time::SystemTime::now" => "system_time_now".to_string(),
        
        // IO
        "Cursor::new" | "std::io::Cursor::new" => "cursor_new".to_string(),
        "BufReader::new" | "std::io::BufReader::new" => "bufreader_new".to_string(),
        "BufReader::with_capacity" | "std::io::BufReader::with_capacity" => "bufreader_with_capacity".to_string(),
        "BufWriter::new" | "std::io::BufWriter::new" => "bufwriter_new".to_string(),
        "BufWriter::with_capacity" | "std::io::BufWriter::with_capacity" => "bufwriter_with_capacity".to_string(),
        "File::open" | "std::fs::File::open" => "file_open".to_string(),
        "File::create" | "std::fs::File::create" => "file_create".to_string(),
        
        // Ordering (comparison result)
        "Ordering::Less" | "std::cmp::Ordering::Less" => "ordering_less".to_string(),
        "Ordering::Equal" | "std::cmp::Ordering::Equal" => "ordering_equal".to_string(),
        "Ordering::Greater" | "std::cmp::Ordering::Greater" => "ordering_greater".to_string(),
        
        // Poll (async support)
        "Poll::Ready" | "std::task::Poll::Ready" => "poll_ready".to_string(),
        "Poll::Pending" | "std::task::Poll::Pending" => "poll_pending".to_string(),
        
        // Location (panic support)
        "Location::caller" | "std::panic::Location::caller" => "location_caller".to_string(),
        
        // Async
        "async" => "async_block".to_string(),
        
        // Default
        _ => "call".to_string(),
    }
}

/// Classify a method call expression
fn classify_method_call(method: &ast::MethodCallExpr) -> String {
    let Some(name) = method.name_ref() else {
        return "method".to_string();
    };
    
    let method_name = name.text().to_string();
    
    match method_name.as_str() {
        // RefCell methods
        "borrow" => "refcell_borrow".to_string(),
        "borrow_mut" => "refcell_borrow_mut".to_string(),
        "try_borrow" => "refcell_try_borrow".to_string(),
        "try_borrow_mut" => "refcell_try_borrow_mut".to_string(),
        
        // Cell methods
        "get" => "cell_get".to_string(),
        "set" => "cell_set".to_string(),
        "replace" => "cell_replace".to_string(),
        "take" => "cell_take".to_string(),
        
        // Mutex/RwLock methods
        "lock" => "mutex_lock".to_string(),
        "try_lock" => "mutex_try_lock".to_string(),
        "read" => "rwlock_read".to_string(),
        "write" => "rwlock_write".to_string(),
        "try_read" => "rwlock_try_read".to_string(),
        "try_write" => "rwlock_try_write".to_string(),
        
        // OnceCell methods
        "get_or_init" => "once_cell_get_or_init".to_string(),
        "get_or_try_init" => "once_cell_get_or_try_init".to_string(),
        
        // MaybeUninit methods
        "assume_init" => "maybe_uninit_assume_init".to_string(),
        "assume_init_read" => "maybe_uninit_assume_init_read".to_string(),
        "assume_init_ref" => "maybe_uninit_assume_init_ref".to_string(),
        "assume_init_mut" => "maybe_uninit_assume_init_mut".to_string(),
        
        // Weak methods
        "downgrade" => "weak_downgrade".to_string(),
        "upgrade" => "weak_upgrade".to_string(),
        
        // Cow methods
        "to_mut" => "cow_to_mut".to_string(),
        "into_owned" => "cow_into_owned".to_string(),
        
        // Clone
        "clone" => "clone".to_string(),
        
        // Pin methods
        "as_ref" => "pin_as_ref".to_string(),
        "as_mut" => "pin_as_mut".to_string(),
        "into_inner" => "into_inner".to_string(),
        
        // Atomic methods
        "load" => "atomic_load".to_string(),
        "store" => "atomic_store".to_string(),
        "swap" => "atomic_swap".to_string(),
        "compare_exchange" => "atomic_compare_exchange".to_string(),
        "compare_exchange_weak" => "atomic_compare_exchange_weak".to_string(),
        "fetch_add" => "atomic_fetch_add".to_string(),
        "fetch_sub" => "atomic_fetch_sub".to_string(),
        "fetch_and" => "atomic_fetch_and".to_string(),
        "fetch_or" => "atomic_fetch_or".to_string(),
        "fetch_xor" => "atomic_fetch_xor".to_string(),
        "fetch_max" => "atomic_fetch_max".to_string(),
        "fetch_min" => "atomic_fetch_min".to_string(),
        "fetch_update" => "atomic_fetch_update".to_string(),
        
        // Duration methods
        "as_secs" => "duration_as_secs".to_string(),
        "as_millis" => "duration_as_millis".to_string(),
        "as_micros" => "duration_as_micros".to_string(),
        "as_nanos" => "duration_as_nanos".to_string(),
        "as_secs_f32" => "duration_as_secs_f".to_string(),
        "as_secs_f64" => "duration_as_secs_f".to_string(),
        "elapsed" => "instant_elapsed".to_string(),
        "duration_since" => "instant_duration_since".to_string(),
        
        // Iterator methods
        "iter" => "iter".to_string(),
        "iter_mut" => "iter_mut".to_string(),
        "into_iter" => "into_iter".to_string(),
        
        // Common methods
        "unwrap" => "unwrap".to_string(),
        "expect" => "expect".to_string(),
        "map" => "map".to_string(),
        "and_then" => "and_then".to_string(),
        "ok" => "ok".to_string(),
        "err" => "err".to_string(),
        
        _ => "method".to_string(),
    }
}

/// Classify path expressions (unit variants, constants, etc.)
fn classify_path_expr(path: &ast::PathExpr) -> String {
    let Some(p) = path.path() else {
        return "path".to_string();
    };
    
    let path_str = p.syntax().text().to_string();
    
    match path_str.as_str() {
        // Ordering variants
        "Ordering::Less" | "std::cmp::Ordering::Less" => "ordering_less".to_string(),
        "Ordering::Equal" | "std::cmp::Ordering::Equal" => "ordering_equal".to_string(),
        "Ordering::Greater" | "std::cmp::Ordering::Greater" => "ordering_greater".to_string(),
        
        // Poll variants (unit variant)
        "Poll::Pending" | "std::task::Poll::Pending" => "poll_pending".to_string(),
        
        // Option/Result variants
        "None" | "Option::None" | "std::option::Option::None" => "none".to_string(),
        
        _ => "path".to_string(),
    }
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
