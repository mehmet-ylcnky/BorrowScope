//! Semantic analysis using rust-analyzer
//!
//! This module provides type analysis by leveraging rust-analyzer's
//! full semantic analysis capabilities. No heuristics are used.

use crate::output::{ProjectTypeInfo, VariableTypeInfo, MethodCallInfo, ExpressionInfo, UnsafeOperationInfo, VariableUsageInfo, BorrowSpanInfo, DestructuringInfo, MatchBindingInfo, PatternBindingInfo, FieldAccessInfo, ClosureTraitInfo, VariantInfo, LifetimeInfo, LabelInfo, ConstPatternInfo, CallableInfo, RecordFieldExprInfo, RecordFieldPatInfo, LayoutInfo};
use anyhow::{Context, Result};
use ra_ap_hir::{db::DefDatabase, HirDisplay, Semantics, Function, Adt, HasContainer, BindingMode, Mutability, ItemContainer, Macro, StructKind, HasSource};
use ra_ap_hir_ty::attach_db;
use ra_ap_ide_db::RootDatabase;
use ra_ap_ide_db::defs::Definition;
use ra_ap_ide_db::search::ReferenceCategory;
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::CargoConfig;
use ra_ap_syntax::{ast, AstNode, Edition, SyntaxKind};
use ra_ap_syntax::ast::{HasName, HasArgList, HasGenericArgs, HasLoopBody};
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
    mapped_mutex_guard: Option<Adt>,
    mapped_rwlock_read_guard: Option<Adt>,
    mapped_rwlock_write_guard: Option<Adt>,
    
    // Memory
    maybe_uninit: Option<Adt>,
    manually_drop: Option<Adt>,
    
    // Collections
    vec: Option<Adt>,
    string: Option<Adt>,
    hashmap: Option<Adt>,
    hashset: Option<Adt>,
    btreemap: Option<Adt>,
    btreeset: Option<Adt>,
    vecdeque: Option<Adt>,
    linkedlist: Option<Adt>,
    binaryheap: Option<Adt>,
    
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
    cstr: Option<Adt>,
    
    // NonNull
    nonnull: Option<Adt>,
    
    // Threading
    join_handle: Option<Adt>,
    
    // Time
    duration: Option<Adt>,
    instant: Option<Adt>,
    
    // Async
    poll: Option<Adt>,
    context: Option<Adt>,
    
    // Ranges
    range: Option<Adt>,
    range_from: Option<Adt>,
    range_to: Option<Adt>,
    range_full: Option<Adt>,
    range_inclusive: Option<Adt>,
    range_to_inclusive: Option<Adt>,
    
    // Other lang items
    phantom_data: Option<Adt>,
    alloc_layout: Option<Adt>,
    
    // Atomics
    atomic_bool: Option<Adt>,
    atomic_i8: Option<Adt>,
    atomic_i16: Option<Adt>,
    atomic_i32: Option<Adt>,
    atomic_i64: Option<Adt>,
    atomic_isize: Option<Adt>,
    atomic_u8: Option<Adt>,
    atomic_u16: Option<Adt>,
    atomic_u32: Option<Adt>,
    atomic_u64: Option<Adt>,
    atomic_usize: Option<Adt>,
    atomic_ptr: Option<Adt>,
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
    /// Build the set of known types by looking them up semantically.
    /// Uses LangItem for types that are lang items (fully semantic, no string matching).
    /// Falls back to import_map search for types without lang items.
    fn new(db: &RootDatabase) -> Self {
        use ra_ap_hir::{import_map, ModuleDef, Crate};
        use ra_ap_hir_def::lang_item::lang_items;
        
        let mut known = Self::default();
        let all_crates = Crate::all(db);
        
        // === Phase 1: Look up types via lang_items (fully semantic, zero string matching) ===
        // lang_items() traverses dependencies, so use first crate that yields results
        for krate in &all_crates {
            let li = lang_items(db, (*krate).into());
            if li.OwnedBox.is_some() {
                if let Some(id) = li.OwnedBox { known.box_ = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.UnsafeCell { known.unsafe_cell = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.Pin { known.pin = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.Option { known.option = Some(Adt::Enum(id.into())); }
                if let Some(id) = li.String { known.string = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.ManuallyDrop { known.manually_drop = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.MaybeUninit { known.maybe_uninit = Some(Adt::Union(id.into())); }
                if let Some(id) = li.PhantomData { known.phantom_data = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.Poll { known.poll = Some(Adt::Enum(id.into())); }
                if let Some(id) = li.Context { known.context = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.Range { known.range = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.RangeFrom { known.range_from = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.RangeTo { known.range_to = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.RangeFull { known.range_full = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.RangeInclusiveStruct { known.range_inclusive = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.RangeToInclusive { known.range_to_inclusive = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.CStr { known.cstr = Some(Adt::Struct(id.into())); }
                if let Some(id) = li.AllocLayout { known.alloc_layout = Some(Adt::Struct(id.into())); }
                break;
            }
        }
        
        // === Phase 2: Look up remaining types via import_map (types without lang items) ===
        // These require module path filtering since they're not lang items
        
        let types_to_find: &[(&str, &str, fn(&mut KnownTypes, Adt))] = &[
            // Smart pointers (no lang items for Rc, Arc, Weak)
            ("Rc", "rc", |k, a| k.rc = Some(a)),
            ("Arc", "sync", |k, a| k.arc = Some(a)),
            ("Weak", "rc", |k, a| k.weak_rc = Some(a)),
            
            // Interior mutability (UnsafeCell is lang item, others are not)
            ("Cell", "cell", |k, a| k.cell = Some(a)),
            ("RefCell", "cell", |k, a| k.refcell = Some(a)),
            ("Mutex", "sync", |k, a| k.mutex = Some(a)),
            ("RwLock", "sync", |k, a| k.rwlock = Some(a)),
            ("OnceCell", "cell", |k, a| k.once_cell = Some(a)),
            ("OnceLock", "sync", |k, a| k.once_lock = Some(a)),
            
            // Guards (no lang items)
            ("Ref", "cell", |k, a| k.ref_guard = Some(a)),
            ("RefMut", "cell", |k, a| k.refmut_guard = Some(a)),
            ("MutexGuard", "sync", |k, a| k.mutex_guard = Some(a)),
            ("RwLockReadGuard", "sync", |k, a| k.rwlock_read_guard = Some(a)),
            ("RwLockWriteGuard", "sync", |k, a| k.rwlock_write_guard = Some(a)),
            ("MappedMutexGuard", "sync", |k, a| k.mapped_mutex_guard = Some(a)),
            ("MappedRwLockReadGuard", "sync", |k, a| k.mapped_rwlock_read_guard = Some(a)),
            ("MappedRwLockWriteGuard", "sync", |k, a| k.mapped_rwlock_write_guard = Some(a)),
            
            // Collections (no lang items except String)
            ("Vec", "vec", |k, a| k.vec = Some(a)),
            ("HashMap", "hash", |k, a| k.hashmap = Some(a)),
            ("HashSet", "hash", |k, a| k.hashset = Some(a)),
            ("BTreeMap", "btree", |k, a| k.btreemap = Some(a)),
            ("BTreeSet", "btree", |k, a| k.btreeset = Some(a)),
            ("VecDeque", "vec_deque", |k, a| k.vecdeque = Some(a)),
            ("LinkedList", "linked_list", |k, a| k.linkedlist = Some(a)),
            ("BinaryHeap", "binary_heap", |k, a| k.binaryheap = Some(a)),
            
            // Wrappers (Pin and Option are lang items)
            ("Cow", "borrow", |k, a| k.cow = Some(a)),
            ("Result", "result", |k, a| k.result = Some(a)),
            
            // Channels (no lang items)
            ("Sender", "mpsc", |k, a| k.sender = Some(a)),
            ("Receiver", "mpsc", |k, a| k.receiver = Some(a)),
            ("SyncSender", "mpsc", |k, a| k.sync_sender = Some(a)),
            
            // Paths/FFI (no lang items)
            ("PathBuf", "path", |k, a| k.pathbuf = Some(a)),
            ("OsString", "ffi", |k, a| k.osstring = Some(a)),
            ("CString", "ffi", |k, a| k.cstring = Some(a)),
            
            // NonNull (no lang item)
            ("NonNull", "ptr", |k, a| k.nonnull = Some(a)),
            
            // Threading (no lang item)
            ("JoinHandle", "thread", |k, a| k.join_handle = Some(a)),
            
            // Time (no lang items)
            ("Duration", "time", |k, a| k.duration = Some(a)),
            ("Instant", "time", |k, a| k.instant = Some(a)),
            
            // Atomics (no lang items)
            ("AtomicBool", "atomic", |k, a| k.atomic_bool = Some(a)),
            ("AtomicI8", "atomic", |k, a| k.atomic_i8 = Some(a)),
            ("AtomicI16", "atomic", |k, a| k.atomic_i16 = Some(a)),
            ("AtomicI32", "atomic", |k, a| k.atomic_i32 = Some(a)),
            ("AtomicI64", "atomic", |k, a| k.atomic_i64 = Some(a)),
            ("AtomicIsize", "atomic", |k, a| k.atomic_isize = Some(a)),
            ("AtomicU8", "atomic", |k, a| k.atomic_u8 = Some(a)),
            ("AtomicU16", "atomic", |k, a| k.atomic_u16 = Some(a)),
            ("AtomicU32", "atomic", |k, a| k.atomic_u32 = Some(a)),
            ("AtomicU64", "atomic", |k, a| k.atomic_u64 = Some(a)),
            ("AtomicUsize", "atomic", |k, a| k.atomic_usize = Some(a)),
            ("AtomicPtr", "atomic", |k, a| k.atomic_ptr = Some(a)),
        ];
        
        // Only search std/core/alloc for standard library types
        let std_crates: Vec<_> = all_crates.iter()
            .filter(|k| {
                let name = k.display_name(db).map(|n| n.to_string()).unwrap_or_default();
                name == "core" || name == "std" || name == "alloc"
            })
            .collect();
        
        for krate in &std_crates {
            for (type_name, expected_module, setter) in types_to_find {
                let query = import_map::Query::new(type_name.to_string()).exact();
                for (item, _) in krate.query_external_importables(db, query) {
                    if let either::Either::Left(ModuleDef::Adt(adt)) = item {
                        let module_path = get_module_path(&adt.module(db), db);
                        if module_path.contains(expected_module) {
                            setter(&mut known, adt);
                        }
                    }
                }
            }
        }
        
        // Handle Weak in sync module (Arc's Weak)
        for krate in &std_crates {
            let query = import_map::Query::new("Weak".to_string()).exact();
            for (item, _) in krate.query_external_importables(db, query) {
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
        if self.mapped_mutex_guard.as_ref() == Some(adt) { return Some("mapped_mutex_guard"); }
        if self.mapped_rwlock_read_guard.as_ref() == Some(adt) { return Some("mapped_rwlock_read_guard"); }
        if self.mapped_rwlock_write_guard.as_ref() == Some(adt) { return Some("mapped_rwlock_write_guard"); }
        
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
        if self.cstr.as_ref() == Some(adt) { return Some("cstr"); }
        
        // NonNull
        if self.nonnull.as_ref() == Some(adt) { return Some("nonnull"); }
        
        // Additional collections
        if self.btreemap.as_ref() == Some(adt) { return Some("btreemap"); }
        if self.btreeset.as_ref() == Some(adt) { return Some("btreeset"); }
        if self.vecdeque.as_ref() == Some(adt) { return Some("vecdeque"); }
        if self.linkedlist.as_ref() == Some(adt) { return Some("linkedlist"); }
        if self.binaryheap.as_ref() == Some(adt) { return Some("binaryheap"); }
        
        // Threading
        if self.join_handle.as_ref() == Some(adt) { return Some("join_handle"); }
        
        // Time
        if self.duration.as_ref() == Some(adt) { return Some("duration"); }
        if self.instant.as_ref() == Some(adt) { return Some("instant"); }
        
        // Async
        if self.poll.as_ref() == Some(adt) { return Some("poll"); }
        if self.context.as_ref() == Some(adt) { return Some("context"); }
        
        // Ranges
        if self.range.as_ref() == Some(adt) { return Some("range"); }
        if self.range_from.as_ref() == Some(adt) { return Some("range_from"); }
        if self.range_to.as_ref() == Some(adt) { return Some("range_to"); }
        if self.range_full.as_ref() == Some(adt) { return Some("range_full"); }
        if self.range_inclusive.as_ref() == Some(adt) { return Some("range_inclusive"); }
        if self.range_to_inclusive.as_ref() == Some(adt) { return Some("range_to_inclusive"); }
        
        // Other lang items
        if self.phantom_data.as_ref() == Some(adt) { return Some("phantom_data"); }
        if self.alloc_layout.as_ref() == Some(adt) { return Some("alloc_layout"); }
        
        // Atomics
        if self.atomic_bool.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_i8.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_i16.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_i32.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_i64.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_isize.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_u8.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_u16.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_u32.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_u64.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_usize.as_ref() == Some(adt) { return Some("atomic"); }
        if self.atomic_ptr.as_ref() == Some(adt) { return Some("atomic"); }
        
        None
    }
    
    /// Check if an ADT is an atomic type
    fn is_atomic(&self, adt: &Adt) -> bool {
        self.atomic_bool.as_ref() == Some(adt)
            || self.atomic_i8.as_ref() == Some(adt)
            || self.atomic_i16.as_ref() == Some(adt)
            || self.atomic_i32.as_ref() == Some(adt)
            || self.atomic_i64.as_ref() == Some(adt)
            || self.atomic_isize.as_ref() == Some(adt)
            || self.atomic_u8.as_ref() == Some(adt)
            || self.atomic_u16.as_ref() == Some(adt)
            || self.atomic_u32.as_ref() == Some(adt)
            || self.atomic_u64.as_ref() == Some(adt)
            || self.atomic_usize.as_ref() == Some(adt)
            || self.atomic_ptr.as_ref() == Some(adt)
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
        var_info.is_atomic = self.is_atomic(adt);
        var_info.is_join_handle = self.join_handle.as_ref() == Some(adt);
        var_info.is_duration = self.duration.as_ref() == Some(adt);
        var_info.is_instant = self.instant.as_ref() == Some(adt);
        
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
        var_info.is_atomic |= self.is_atomic(adt);
        var_info.is_join_handle |= self.join_handle.as_ref() == Some(adt);
        var_info.is_duration |= self.duration.as_ref() == Some(adt);
        var_info.is_instant |= self.instant.as_ref() == Some(adt);
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

/// Known macros looked up once at startup by semantic identity (MacroId).
/// Used for macro classification without string matching.
#[derive(Default)]
pub(crate) struct KnownMacros {
    // Standard library macros
    vec: Option<Macro>,
    format: Option<Macro>,
    format_args: Option<Macro>,
    println: Option<Macro>,
    print: Option<Macro>,
    eprintln: Option<Macro>,
    eprint: Option<Macro>,
    panic: Option<Macro>,
    assert: Option<Macro>,
    assert_eq: Option<Macro>,
    assert_ne: Option<Macro>,
    debug_assert: Option<Macro>,
    debug_assert_eq: Option<Macro>,
    debug_assert_ne: Option<Macro>,
    write: Option<Macro>,
    writeln: Option<Macro>,
    todo: Option<Macro>,
    unimplemented: Option<Macro>,
    unreachable: Option<Macro>,
    dbg: Option<Macro>,
    env: Option<Macro>,
    option_env: Option<Macro>,
    concat: Option<Macro>,
    stringify: Option<Macro>,
    include: Option<Macro>,
    include_str: Option<Macro>,
    include_bytes: Option<Macro>,
    cfg: Option<Macro>,
    line: Option<Macro>,
    column: Option<Macro>,
    file: Option<Macro>,
    module_path: Option<Macro>,
}

impl KnownMacros {
    /// Build the set of known macros by looking them up semantically
    fn new(db: &RootDatabase) -> Self {
        use ra_ap_hir::{import_map, ModuleDef, Crate};
        
        let mut known = Self::default();
        
        // Macros to find: (macro_name, field_setter)
        let macros_to_find: &[(&str, fn(&mut KnownMacros, Macro))] = &[
            ("vec", |k, m| k.vec = Some(m)),
            ("format", |k, m| k.format = Some(m)),
            ("format_args", |k, m| k.format_args = Some(m)),
            ("println", |k, m| k.println = Some(m)),
            ("print", |k, m| k.print = Some(m)),
            ("eprintln", |k, m| k.eprintln = Some(m)),
            ("eprint", |k, m| k.eprint = Some(m)),
            ("panic", |k, m| k.panic = Some(m)),
            ("assert", |k, m| k.assert = Some(m)),
            ("assert_eq", |k, m| k.assert_eq = Some(m)),
            ("assert_ne", |k, m| k.assert_ne = Some(m)),
            ("debug_assert", |k, m| k.debug_assert = Some(m)),
            ("debug_assert_eq", |k, m| k.debug_assert_eq = Some(m)),
            ("debug_assert_ne", |k, m| k.debug_assert_ne = Some(m)),
            ("write", |k, m| k.write = Some(m)),
            ("writeln", |k, m| k.writeln = Some(m)),
            ("todo", |k, m| k.todo = Some(m)),
            ("unimplemented", |k, m| k.unimplemented = Some(m)),
            ("unreachable", |k, m| k.unreachable = Some(m)),
            ("dbg", |k, m| k.dbg = Some(m)),
            ("env", |k, m| k.env = Some(m)),
            ("option_env", |k, m| k.option_env = Some(m)),
            ("concat", |k, m| k.concat = Some(m)),
            ("stringify", |k, m| k.stringify = Some(m)),
            ("include", |k, m| k.include = Some(m)),
            ("include_str", |k, m| k.include_str = Some(m)),
            ("include_bytes", |k, m| k.include_bytes = Some(m)),
            ("cfg", |k, m| k.cfg = Some(m)),
            ("line", |k, m| k.line = Some(m)),
            ("column", |k, m| k.column = Some(m)),
            ("file", |k, m| k.file = Some(m)),
            ("module_path", |k, m| k.module_path = Some(m)),
        ];
        
        // Only search std/core for standard macros
        let std_crates: Vec<_> = Crate::all(db).into_iter()
            .filter(|k| {
                let name = k.display_name(db).map(|n| n.to_string()).unwrap_or_default();
                name == "core" || name == "std" || name == "alloc"
            })
            .collect();
        
        for krate in &std_crates {
            for (macro_name, setter) in macros_to_find {
                let query = import_map::Query::new(macro_name.to_string()).exact();
                for (item, _) in krate.query_external_importables(db, query) {
                    if let either::Either::Left(ModuleDef::Macro(mac)) = item {
                        // Only take function-like macros (not derives or attributes)
                        if mac.is_fn_like(db) {
                            setter(&mut known, mac);
                        }
                    }
                }
            }
        }
        
        known
    }
    
    /// Classify a macro by comparing MacroId directly (fully semantic)
    fn classify(&self, mac: &Macro) -> Option<&'static str> {
        // Collection macros
        if self.vec.as_ref() == Some(mac) { return Some("vec_macro"); }
        
        // Formatting macros
        if self.format.as_ref() == Some(mac) { return Some("format_macro"); }
        if self.format_args.as_ref() == Some(mac) { return Some("format_args_macro"); }
        
        // Print macros
        if self.println.as_ref() == Some(mac) { return Some("println_macro"); }
        if self.print.as_ref() == Some(mac) { return Some("print_macro"); }
        if self.eprintln.as_ref() == Some(mac) { return Some("eprintln_macro"); }
        if self.eprint.as_ref() == Some(mac) { return Some("eprint_macro"); }
        
        // Panic/assert macros
        if self.panic.as_ref() == Some(mac) { return Some("panic_macro"); }
        if self.assert.as_ref() == Some(mac) { return Some("assert_macro"); }
        if self.assert_eq.as_ref() == Some(mac) { return Some("assert_eq_macro"); }
        if self.assert_ne.as_ref() == Some(mac) { return Some("assert_ne_macro"); }
        if self.debug_assert.as_ref() == Some(mac) { return Some("debug_assert_macro"); }
        if self.debug_assert_eq.as_ref() == Some(mac) { return Some("debug_assert_eq_macro"); }
        if self.debug_assert_ne.as_ref() == Some(mac) { return Some("debug_assert_ne_macro"); }
        
        // Write macros
        if self.write.as_ref() == Some(mac) { return Some("write_macro"); }
        if self.writeln.as_ref() == Some(mac) { return Some("writeln_macro"); }
        
        // Placeholder macros
        if self.todo.as_ref() == Some(mac) { return Some("todo_macro"); }
        if self.unimplemented.as_ref() == Some(mac) { return Some("unimplemented_macro"); }
        if self.unreachable.as_ref() == Some(mac) { return Some("unreachable_macro"); }
        
        // Debug macro
        if self.dbg.as_ref() == Some(mac) { return Some("dbg_macro"); }
        
        // Environment macros
        if self.env.as_ref() == Some(mac) { return Some("env_macro"); }
        if self.option_env.as_ref() == Some(mac) { return Some("option_env_macro"); }
        
        // String macros
        if self.concat.as_ref() == Some(mac) { return Some("concat_macro"); }
        if self.stringify.as_ref() == Some(mac) { return Some("stringify_macro"); }
        
        // Include macros
        if self.include.as_ref() == Some(mac) { return Some("include_macro"); }
        if self.include_str.as_ref() == Some(mac) { return Some("include_str_macro"); }
        if self.include_bytes.as_ref() == Some(mac) { return Some("include_bytes_macro"); }
        
        // Compile-time info macros
        if self.cfg.as_ref() == Some(mac) { return Some("cfg_macro"); }
        if self.line.as_ref() == Some(mac) { return Some("line_macro"); }
        if self.column.as_ref() == Some(mac) { return Some("column_macro"); }
        if self.file.as_ref() == Some(mac) { return Some("file_macro"); }
        if self.module_path.as_ref() == Some(mac) { return Some("module_path_macro"); }
        
        None
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
        
        // Search std crate (re-exports core functions)
        let all_crates = Crate::all(db);
        let std_crate = all_crates.iter()
            .find(|k| k.display_name(db).map(|n| n.to_string()).unwrap_or_default() == "std");
        
        if let Some(krate) = std_crate {
            for (fn_name, acceptable_modules) in functions_to_find {
                let query = import_map::Query::new(fn_name.to_string()).exact();
                for (item, _) in krate.query_external_importables(db, query) {
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
        
        // Fallback: search all crates for functions not found in std
        for (fn_name, acceptable_modules) in functions_to_find {
            let already_found = tracked.functions.values().any(|p| p.ends_with(fn_name));
            if !already_found {
                for krate in &all_crates {
                    let query = import_map::Query::new(fn_name.to_string()).exact();
                    for (item, _) in krate.query_external_importables(db, query) {
                        if let either::Either::Left(ModuleDef::Function(f)) = item {
                            let module_path = get_module_path(&f.module(db), db);
                            if acceptable_modules.iter().any(|m| module_path.contains(m)) {
                                let path = get_function_path(&f, db);
                                tracked.functions.insert(f, path);
                                break;
                            }
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
    let krate = module.krate(db).display_name(db)
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
        proc_macro_processes: 0,
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
    
    // Look up tracked functions, types, and macros once by semantic identity
    let tracked_functions = TrackedFunctions::new(&db);
    let known_types = KnownTypes::new(&db);
    let known_macros = KnownMacros::new(&db);
    
    // Create DisplayTarget for type display (use first crate's edition)
    use ra_ap_hir::DisplayTarget;
    let display_target = ra_ap_hir::Crate::all(&db)
        .first()
        .map(|k| DisplayTarget::from_crate(&db, (*k).into()))
        .unwrap_or_else(|| DisplayTarget::from_crate(&db, ra_ap_hir::Crate::all(&db)[0].into()));

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

        let (variables, expressions, await_points, unsafe_ops, borrow_spans, destructuring, match_bindings, field_accesses, closure_traits, variants, lifetimes, labels, const_patterns, callables, record_field_exprs, record_field_pats) = attach_db(&db, || {
            analyze_file(&sema, &db, &tracked_functions, &known_types, &known_macros, &display_target, file_id, &relative)
        });
        if !variables.is_empty() {
            info.files.insert(relative.clone(), variables);
        }
        if !expressions.is_empty() {
            info.expressions.insert(relative.clone(), expressions);
        }
        if !await_points.is_empty() {
            info.await_points.insert(relative.clone(), await_points);
        }
        if !unsafe_ops.is_empty() {
            info.unsafe_operations.insert(relative.clone(), unsafe_ops);
        }
        if !borrow_spans.is_empty() {
            info.borrow_spans.insert(relative.clone(), borrow_spans);
        }
        if !destructuring.is_empty() {
            info.destructuring.insert(relative.clone(), destructuring);
        }
        if !match_bindings.is_empty() {
            info.match_bindings.insert(relative.clone(), match_bindings);
        }
        if !field_accesses.is_empty() {
            info.field_accesses.insert(relative.clone(), field_accesses);
        }
        if !closure_traits.is_empty() {
            info.closure_traits.insert(relative.clone(), closure_traits);
        }
        if !variants.is_empty() {
            info.variants.insert(relative.clone(), variants);
        }
        if !lifetimes.is_empty() {
            info.lifetimes.insert(relative.clone(), lifetimes);
        }
        if !labels.is_empty() {
            info.labels.insert(relative.clone(), labels);
        }
        if !const_patterns.is_empty() {
            info.const_patterns.insert(relative.clone(), const_patterns);
        }
        if !callables.is_empty() {
            info.callables.insert(relative.clone(), callables);
        }
        if !record_field_exprs.is_empty() {
            info.record_field_exprs.insert(relative.clone(), record_field_exprs);
        }
        if !record_field_pats.is_empty() {
            info.record_field_pats.insert(relative, record_field_pats);
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
    known_macros: &KnownMacros,
    display_target: &ra_ap_hir::DisplayTarget,
    file_id: ra_ap_vfs::FileId,
    relative_path: &str,
) -> (Vec<VariableTypeInfo>, Vec<ExpressionInfo>, Vec<crate::output::AwaitPointInfo>, Vec<UnsafeOperationInfo>, Vec<BorrowSpanInfo>, Vec<DestructuringInfo>, Vec<MatchBindingInfo>, Vec<FieldAccessInfo>, Vec<ClosureTraitInfo>, Vec<VariantInfo>, Vec<LifetimeInfo>, Vec<LabelInfo>, Vec<ConstPatternInfo>, Vec<CallableInfo>, Vec<RecordFieldExprInfo>, Vec<RecordFieldPatInfo>) {
    let mut variables = Vec::new();
    let mut await_points = Vec::new();
    let mut unsafe_ops = Vec::new();
    let mut borrow_spans = Vec::new();
    let mut destructuring = Vec::new();
    let mut match_bindings = Vec::new();
    let mut scope_id: u32 = 0;

    let editioned_file_id = sema.attach_first_edition(file_id);

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
                if let Some(mut var_info) = analyze_let_stmt(sema, db, known_types, known_macros, display_target, &node, relative_path, &source_file, &mut scope_id) {
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
                // Also check for destructuring patterns
                if let Some(destr_info) = analyze_destructuring_pattern(sema, db, &node, &source_file) {
                    destructuring.push(destr_info);
                }
            }
            SyntaxKind::STATIC => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, known_types, known_macros, display_target, &node, relative_path, &source_file) {
                    var_info.is_static = true;
                    variables.push(var_info);
                }
            }
            SyntaxKind::CONST => {
                if let Some(mut var_info) = analyze_static_or_const(sema, db, known_types, known_macros, display_target, &node, relative_path, &source_file) {
                    var_info.is_const = true;
                    variables.push(var_info);
                }
            }
            SyntaxKind::AWAIT_EXPR => {
                if let Some(await_info) = analyze_await_expr(sema, db, &node, &source_file, display_target) {
                    await_points.push(await_info);
                }
            }
            SyntaxKind::MATCH_EXPR => {
                analyze_match_bindings(sema, db, &node, &source_file, &mut match_bindings, display_target);
            }
            SyntaxKind::IF_EXPR => {
                analyze_if_let_bindings(sema, db, &node, &source_file, &mut match_bindings, display_target);
            }
            SyntaxKind::WHILE_EXPR => {
                analyze_while_let_bindings(sema, db, &node, &source_file, &mut match_bindings, display_target);
            }
            _ => {}
        }
    }

    // Analyze method calls on tracked variables
    analyze_method_calls(sema, db, &source_file, &mut variables, display_target);
    
    // Analyze variable usages (reads and writes)
    analyze_variable_usages(sema, &source_file, &mut variables);
    
    // Analyze standalone expressions (using semantic function lookup)
    let expressions = analyze_expressions(sema, db, tracked_functions, &source_file, display_target);
    
    // Analyze unsafe operations
    analyze_unsafe_operations(sema, db, &source_file, display_target, &mut unsafe_ops);
    
    // Analyze borrow spans
    analyze_borrow_spans(sema, editioned_file_id, &source_file, &mut borrow_spans);
    
    // Analyze field accesses (for partial borrow tracking)
    let field_accesses = analyze_field_accesses(sema, db, &source_file, display_target);
    
    // Analyze closure traits (Fn/FnMut/FnOnce)
    let closure_traits = analyze_closure_traits(sema, db, &source_file, display_target);
    
    // Analyze enum variant constructions (semantic via sema.resolve_variant)
    let variants = analyze_variants(sema, db, &source_file, display_target);
    
    // Analyze lifetime parameters (semantic via sema.resolve_lifetime_param)
    let lifetimes = analyze_lifetimes(sema, db, &source_file);
    
    // Analyze loop labels (semantic via sema.resolve_label)
    let labels = analyze_labels(sema, db, &source_file);
    
    // Analyze const pattern bindings (semantic via sema.resolve_bind_pat_to_const)
    let const_patterns = analyze_const_patterns(sema, db, &source_file, display_target);
    
    // Analyze callable expressions (semantic via Type::as_callable, impls_fnonce)
    let callables = analyze_callables(sema, db, &source_file, display_target);
    
    // Analyze record field expressions (semantic via sema.resolve_record_field)
    let record_field_exprs = analyze_record_field_exprs(sema, db, &source_file, display_target);
    
    // Analyze record field patterns (semantic via sema.resolve_record_pat_field)
    let record_field_pats = analyze_record_field_pats(sema, db, &source_file, display_target);
    
    // Update await points with poll function resolution
    update_await_points_with_poll(sema, db, &source_file, display_target, &mut await_points);

    (variables, expressions, await_points, unsafe_ops, borrow_spans, destructuring, match_bindings, field_accesses, closure_traits, variants, lifetimes, labels, const_patterns, callables, record_field_exprs, record_field_pats)
}

/// Analyze a let statement
fn analyze_let_stmt(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_types: &KnownTypes,
    known_macros: &KnownMacros,
    display_target: &ra_ap_hir::DisplayTarget,
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

    // Detect mut binding and binding mode
    if let ast::Pat::IdentPat(ident_pat) = &pat {
        var_info.is_mut_binding = ident_pat.mut_token().is_some();
        
        // Get semantic binding mode (move, ref, ref_mut)
        if let Some(mode) = sema.binding_mode_of_pat(&ident_pat) {
            var_info.binding_mode = Some(match mode {
                BindingMode::Move => "move".to_string(),
                BindingMode::Ref(Mutability::Shared) => "ref".to_string(),
                BindingMode::Ref(Mutability::Mut) => "ref_mut".to_string(),
            });
        }
        
        // Check if this is a ref binding (semantic via Local::is_ref)
        if let Some(local) = sema.to_def(ident_pat) {
            var_info.is_ref_binding = local.is_ref(db);
        }
    }
    
    // Get pattern adjustments on the outer pattern (semantic via sema.pattern_adjustments)
    // Pattern adjustments happen when match ergonomics peels off references
    var_info.pattern_adjustments = sema.pattern_adjustments(&pat)
        .into_iter()
        .map(|ty| ty.display(db, *display_target).to_string())
        .collect();

    if let Some(type_info) = sema.type_of_pat(&pat) {
        populate_type_info(&mut var_info, &type_info.original, db, known_types, display_target);
        
        // Detect impl Trait semantically via Type::as_impl_traits
        var_info.is_impl_trait = type_info.original.as_impl_traits(db).is_some();
    }

    // Extract lifetime from explicit type annotation
    if let Some(ty) = let_stmt.ty() {
        // Extract explicit lifetime from reference type annotation
        var_info.lifetime = extract_lifetime_from_type(&ty);
    }

    // Detect initializer kind semantically using resolved type
    if let Some(init) = let_stmt.initializer() {
        let resolved_type = sema.type_of_pat(&pat).map(|ti| ti.original);
        var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, known_types, known_macros, &init, resolved_type.as_ref()));
        
        // Extract expression adjustments (semantic via sema.expr_adjustments)
        if let Some(adjustments) = sema.expr_adjustments(&init) {
            var_info.adjustments = adjustments.into_iter()
                .map(|adj| {
                    let kind = match adj.kind {
                        ra_ap_hir::Adjust::NeverToAny => "never_to_any",
                        ra_ap_hir::Adjust::Deref(_) => "deref",
                        ra_ap_hir::Adjust::Borrow(ra_ap_hir::AutoBorrow::Ref(m)) => {
                            if m.is_mut() { "borrow_mut" } else { "borrow_shared" }
                        }
                        ra_ap_hir::Adjust::Borrow(ra_ap_hir::AutoBorrow::RawPtr(m)) => {
                            if m.is_mut() { "raw_ptr_mut" } else { "raw_ptr_shared" }
                        }
                        ra_ap_hir::Adjust::Pointer(_) => "pointer_cast",
                    };
                    crate::output::AdjustmentInfo {
                        kind: kind.to_string(),
                        target: adj.target.display(db, *display_target).to_string(),
                    }
                })
                .collect();
        }
        
        // Extract closure captures if initializer is a closure
        if let ast::Expr::ClosureExpr(closure) = &init {
            var_info.closure_captures = extract_closure_captures_semantic(sema, db, closure, display_target);
        }
    }

    // Detect drop point - find the enclosing block and get its end position
    if let Some((drop_line, drop_column)) = find_drop_point(let_stmt.syntax(), source_file) {
        var_info.drop_line = Some(drop_line);
        var_info.drop_column = Some(drop_column);
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
    known_macros: &KnownMacros,
    expr: &ast::Expr,
    resolved_type: Option<&ra_ap_hir::Type>,
) -> String {
    // Get expression structure as context
    let expr_kind = classify_expr_structure(expr);
    
    // Always try semantic classification first using AdtId comparison
    if let Some(ty) = resolved_type {
        if let Some(semantic_kind) = classify_by_resolved_type_semantic(ty, known_types, &expr_kind, db) {
            return semantic_kind;
        }
    }
    
    // Fallback to macro-specific classification for macros (semantic via MacroId)
    if let ast::Expr::MacroExpr(mac) = expr {
        return classify_macro_expr_semantic(sema, db, known_macros, mac);
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
fn classify_by_resolved_type_semantic(ty: &ra_ap_hir::Type, known_types: &KnownTypes, expr_kind: &str, db: &RootDatabase) -> Option<String> {
    // Get the ADT for type-based classification using AdtId comparison
    if let Some(adt) = ty.as_adt() {
        // Use semantic AdtId comparison instead of string matching
        let type_class = known_types.classify(&adt).unwrap_or_else(|| {
            // Fallback: check crate + name for std types not in KnownTypes (e.g., unstable/nightly)
            let adt_name = adt.name(db).display_no_db(Edition::Edition2021).to_string();
            let crate_name = adt.module(db).krate(db).display_name(db).map(|n| n.to_string()).unwrap_or_default();
            let is_std = crate_name == "std" || crate_name == "core" || crate_name == "alloc";
            match (adt_name.as_str(), is_std) {
                ("MappedMutexGuard", true) => "mapped_mutex_guard",
                ("MappedRwLockReadGuard", true) => "mapped_rwlock_read_guard",
                ("MappedRwLockWriteGuard", true) => "mapped_rwlock_write_guard",
                _ => match &adt {
                    ra_ap_hir::Adt::Struct(_) => "user_struct",
                    ra_ap_hir::Adt::Enum(_) => "user_enum",
                    ra_ap_hir::Adt::Union(_) => "user_union",
                },
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
            ("mapped_mutex_guard", _) => "mutex_guard_mapped",
            ("rwlock", "call") => "rwlock_new",
            ("rwlock_read_guard", "read") => "rwlock_read",
            ("rwlock_write_guard", "write") => "rwlock_write",
            ("mapped_rwlock_read_guard", _) => "rwlock_read_guard_mapped",
            ("mapped_rwlock_write_guard", _) => "rwlock_write_guard_mapped",
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
            ("option", "macro") => "option_macro",
            ("result", "call") => "result_variant",
            ("result", "path") => "result_variant",
            
            // Channels
            ("channel_sender", _) | ("channel_receiver", _) => "channel_new",
            ("sync_channel_sender", _) => "sync_channel_new",
            
            // Paths/FFI
            ("pathbuf", "call") => "pathbuf_new",
            ("osstring", "call") => "osstring_new",
            ("cstring", "call") => "cstring_new",
            ("cstr", _) => "cstr",
            
            // NonNull
            ("nonnull", "call") => "nonnull_new",
            
            // Additional collections
            ("btreemap", "call") => "btreemap_new",
            ("btreeset", "call") => "btreeset_new",
            ("vecdeque", "call") => "vecdeque_new",
            ("linkedlist", "call") => "linkedlist_new",
            ("binaryheap", "call") => "binaryheap_new",
            
            // Threading
            ("join_handle", _) => "join_handle",
            
            // Time
            ("duration", "call") => "duration_new",
            ("duration", _) => "duration",
            ("instant", "call") => "instant_new",
            ("instant", _) => "instant",
            
            // Async
            ("poll", _) => "poll",
            ("context", _) => "async_context",
            
            // Ranges
            ("range", _) => "range",
            ("range_from", _) => "range_from",
            ("range_to", _) => "range_to",
            ("range_full", _) => "range_full",
            ("range_inclusive", _) => "range_inclusive",
            ("range_to_inclusive", _) => "range_to_inclusive",
            
            // Other lang items
            ("phantom_data", _) => "phantom_data",
            ("alloc_layout", "call") => "alloc_layout_new",
            ("alloc_layout", _) => "alloc_layout",
            
            // Atomics
            ("atomic", "call") => "atomic_new",
            ("atomic", _) => "atomic",
            
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

/// Classify macro expressions using semantic MacroId comparison
fn classify_macro_expr_semantic(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_macros: &KnownMacros,
    mac: &ast::MacroExpr,
) -> String {
    let Some(macro_call) = mac.macro_call() else {
        return "macro".to_string();
    };
    
    // Try semantic resolution via MacroId comparison (fully semantic)
    if let Some(resolved) = sema.resolve_macro_call(&macro_call) {
        // First try direct MacroId comparison via KnownMacros
        if let Some(classification) = known_macros.classify(&resolved) {
            return classification.to_string();
        }
        
        // For unknown macros, return the canonical path
        let module_path = get_module_path(&resolved.module(db), db);
        let name = resolved.name(db).display_no_db(Edition::Edition2021).to_string();
        return format!("{}::{}", module_path, name);
    }
    
    // Fallback to syntactic only if semantic resolution completely fails
    let Some(path) = macro_call.path() else {
        return "macro".to_string();
    };
    let macro_name = path.syntax().text().to_string();
    format!("macro:{}", macro_name)
}

/// Analyze a static or const declaration
/// Analyze an await expression
fn analyze_await_expr(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Option<crate::output::AwaitPointInfo> {
    let await_expr = ast::AwaitExpr::cast(node.clone())?;
    let inner_expr = await_expr.expr()?;
    
    let range = await_expr.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    
    // Get the type of the awaited expression (the Future type)
    let awaited_type = sema.type_of_expr(&inner_expr)
        .map(|ti| ti.original.display(db, *display_target).to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    // Get the result type (what the await resolves to)
    let result_type = sema.type_of_expr(&ast::Expr::from(await_expr.clone()))
        .map(|ti| ti.original.display(db, *display_target).to_string());
    
    Some(crate::output::AwaitPointInfo {
        line,
        column,
        awaited_type,
        result_type,
        live_variables: Vec::new(), // TODO: implement live variable analysis
        poll_function: None, // Will be populated by update_await_points_with_poll
    })
}

fn analyze_static_or_const(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    known_types: &KnownTypes,
    known_macros: &KnownMacros,
    display_target: &ra_ap_hir::DisplayTarget,
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
            populate_type_info(&mut var_info, &ty, db, known_types, display_target);
            
            // Classify initializer if body exists
            if let Some(expr) = body_expr {
                var_info.initializer_kind = Some(classify_initializer_semantic(sema, db, known_types, known_macros, &expr, Some(&ty)));
            }
        } else {
            // Fallback: use the syntax text, no classification without semantic info
            var_info.ty = ty_node.syntax().text().to_string();
        }
    }

    Some(var_info)
}

/// Find the drop point for a variable by locating the end of its enclosing scope.
/// The drop point is where the variable goes out of scope (end of enclosing block).
fn find_drop_point(node: &ra_ap_syntax::SyntaxNode, source_file: &ast::SourceFile) -> Option<(u32, u32)> {
    // Walk up the syntax tree to find the enclosing block expression
    for ancestor in node.ancestors() {
        // Check for block expressions: { ... }
        if let Some(block) = ast::BlockExpr::cast(ancestor.clone()) {
            // Get the closing brace position
            if let Some(r_curly) = block.stmt_list().and_then(|sl| sl.r_curly_token()) {
                let range = r_curly.text_range();
                return Some(get_location(&range, source_file));
            }
        }
        
        // Check for function body
        if let Some(fn_def) = ast::Fn::cast(ancestor.clone()) {
            if let Some(body) = fn_def.body() {
                if let Some(r_curly) = body.stmt_list().and_then(|sl| sl.r_curly_token()) {
                    let range = r_curly.text_range();
                    return Some(get_location(&range, source_file));
                }
            }
        }
        
        // Check for closure body
        if let Some(closure) = ast::ClosureExpr::cast(ancestor.clone()) {
            if let Some(body) = closure.body() {
                // For block body closures
                if let ast::Expr::BlockExpr(block) = body {
                    if let Some(r_curly) = block.stmt_list().and_then(|sl| sl.r_curly_token()) {
                        let range = r_curly.text_range();
                        return Some(get_location(&range, source_file));
                    }
                }
            }
        }
        
        // Check for if/else, match, loop bodies
        if let Some(if_expr) = ast::IfExpr::cast(ancestor.clone()) {
            if let Some(then_branch) = if_expr.then_branch() {
                if let Some(r_curly) = then_branch.stmt_list().and_then(|sl| sl.r_curly_token()) {
                    let range = r_curly.text_range();
                    return Some(get_location(&range, source_file));
                }
            }
        }
        
        if let Some(loop_expr) = ast::LoopExpr::cast(ancestor.clone()) {
            if let Some(body) = loop_expr.loop_body() {
                if let Some(r_curly) = body.stmt_list().and_then(|sl| sl.r_curly_token()) {
                    let range = r_curly.text_range();
                    return Some(get_location(&range, source_file));
                }
            }
        }
        
        if let Some(while_expr) = ast::WhileExpr::cast(ancestor.clone()) {
            if let Some(body) = while_expr.loop_body() {
                if let Some(r_curly) = body.stmt_list().and_then(|sl| sl.r_curly_token()) {
                    let range = r_curly.text_range();
                    return Some(get_location(&range, source_file));
                }
            }
        }
        
        if let Some(for_expr) = ast::ForExpr::cast(ancestor.clone()) {
            if let Some(body) = for_expr.loop_body() {
                if let Some(r_curly) = body.stmt_list().and_then(|sl| sl.r_curly_token()) {
                    let range = r_curly.text_range();
                    return Some(get_location(&range, source_file));
                }
            }
        }
    }
    
    None
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

/// Extract explicit lifetime from a type annotation (recursively searches nested types)
fn extract_lifetime_from_type(ty: &ast::Type) -> Option<String> {
    match ty {
        ast::Type::RefType(ref_ty) => {
            // Direct reference type - check for lifetime
            if let Some(lifetime) = ref_ty.lifetime() {
                return Some(lifetime.syntax().text().to_string());
            }
            // Check inner type for nested lifetimes
            ref_ty.ty().and_then(|inner| extract_lifetime_from_type(&inner))
        }
        ast::Type::PathType(path_ty) => {
            // Check generic arguments for lifetimes (e.g., Cow<'a, str>)
            path_ty.path()
                .and_then(|p| p.segments().last())
                .and_then(|seg| seg.generic_arg_list())
                .and_then(|args| {
                    args.lifetime_args().next()
                        .and_then(|la| la.lifetime())
                        .map(|lt| lt.syntax().text().to_string())
                })
        }
        ast::Type::SliceType(slice_ty) => {
            slice_ty.ty().and_then(|inner| extract_lifetime_from_type(&inner))
        }
        ast::Type::ArrayType(arr_ty) => {
            arr_ty.ty().and_then(|inner| extract_lifetime_from_type(&inner))
        }
        ast::Type::PtrType(ptr_ty) => {
            ptr_ty.ty().and_then(|inner| extract_lifetime_from_type(&inner))
        }
        ast::Type::TupleType(tuple_ty) => {
            tuple_ty.fields().find_map(|f| extract_lifetime_from_type(&f))
        }
        ast::Type::ParenType(paren_ty) => {
            paren_ty.ty().and_then(|inner| extract_lifetime_from_type(&inner))
        }
        _ => None,
    }
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
fn populate_type_info(var_info: &mut VariableTypeInfo, ty: &ra_ap_hir::Type, db: &RootDatabase, known_types: &KnownTypes, display_target: &ra_ap_hir::DisplayTarget) {
    use ra_ap_hir::Adt;
    
    var_info.ty = ty.display(db, *display_target).to_string();
    
    // Extract generic type arguments (e.g., ["String", "i32"] for HashMap<String, i32>)
    var_info.type_arguments = ty.type_arguments()
        .map(|arg| arg.display(db, *display_target).to_string())
        .collect();
    
    // Get krate for lang item lookups - prefer from ADT, fallback to first crate
    let krate = ty.as_adt()
        .map(|adt| adt.module(db).krate(db))
        .or_else(|| ra_ap_hir::Crate::all(db).first().copied());
    
    // === Core trait implementations (semantic) ===
    var_info.is_copy = ty.is_copy(db);
    
    if let Some(krate) = krate {
        use ra_ap_hir_def::lang_item::lang_items;
        let li = lang_items(db, krate.into());
        
        if let Some(trait_id) = li.Clone {
            var_info.is_clone = ty.impls_trait(db, trait_id.into(), &[]);
        }
        if let Some(trait_id) = li.Drop {
            var_info.is_drop = ty.impls_trait(db, trait_id.into(), &[]);
        }
        if let Some(trait_id) = li.Sync {
            var_info.is_sync = ty.impls_trait(db, trait_id.into(), &[]);
        }
        if let Some(trait_id) = li.Sized {
            var_info.is_sized = ty.impls_trait(db, trait_id.into(), &[]);
        }
        // Future trait
        if let Some(trait_id) = li.Future {
            var_info.is_future = ty.impls_trait(db, trait_id.into(), &[]);
            if var_info.is_future {
                if let Some(output_ty) = ty.clone().future_output(db) {
                    var_info.future_output_type = Some(output_ty.display(db, *display_target).to_string());
                }
            }
        }
        // Iterator trait  
        if let Some(trait_id) = li.Iterator {
            var_info.is_iterator = ty.impls_trait(db, trait_id.into(), &[]);
            if var_info.is_iterator {
                if let Some(item_ty) = ty.clone().iterator_item(db) {
                    var_info.iterator_item_type = Some(item_ty.display(db, *display_target).to_string());
                }
            }
        }
        
        // Send trait - not a lang item, must be found via import_map search
        if let Some(send_trait) = find_send_trait(db, krate) {
            var_info.is_send = ty.impls_trait(db, send_trait, &[]);
        }
    }
    
    // Callable check (semantic via Type::impls_fnonce)
    var_info.is_callable = ty.impls_fnonce(db);
    
    // Memory layout (semantic via Adt::layout)
    if let Some(adt) = ty.as_adt() {
        if let Ok(layout) = adt.layout(db) {
            var_info.layout = Some(LayoutInfo {
                size: layout.size(),
                align: layout.align(),
            });
        }
    }
    
    // === Type structure (semantic via Type methods) ===
    var_info.is_reference = ty.is_reference();
    var_info.is_mutable_reference = ty.is_mutable_reference();
    var_info.is_raw_ptr = ty.is_raw_ptr();
    var_info.is_closure = ty.is_closure();
    var_info.is_fn_ptr = ty.is_fn();
    
    // Reference analysis (semantic via Type methods)
    var_info.contains_reference = ty.contains_reference(db);
    if let Some((_, mutability)) = ty.as_reference() {
        var_info.reference_mutability = Some(if mutability.is_mut() { "mutable" } else { "shared" }.to_string());
    }
    
    // Deref chain (semantic via Type::autoderef)
    var_info.deref_chain = ty.autoderef(db)
        .skip(1) // Skip the type itself
        .map(|t| t.display(db, *display_target).to_string())
        .collect();
    
    // Struct fields (semantic via Type::fields)
    var_info.fields = ty.fields(db)
        .into_iter()
        .map(|(field, field_ty)| {
            crate::output::FieldInfo {
                name: field.name(db).display_no_db(Edition::Edition2021).to_string(),
                ty: field_ty.display(db, *display_target).to_string(),
            }
        })
        .collect();
    
    // Check for slice - either bare [T] or contained in reference/smart pointer
    var_info.is_slice = ty.is_slice() || ty.strip_reference().is_slice()
        || ty.type_arguments().any(|inner| inner.is_slice());
    
    // Primitive detection via builtin type
    if let Some(builtin) = ty.as_builtin() {
        var_info.is_primitive = builtin.is_int() || builtin.is_uint() || builtin.is_float() 
            || builtin.is_char() || builtin.is_bool() || builtin.is_str();
    }
    var_info.is_primitive = var_info.is_primitive || ty.is_unit();
    
    // str type (the unsized string slice type) - check both direct and referenced
    if let Some(builtin) = ty.as_builtin() {
        var_info.is_str = builtin.is_str();
    }
    // Also check if it's a reference to str (e.g., &str)
    if let Some(inner) = ty.as_reference() {
        if let Some(builtin) = inner.0.as_builtin() {
            var_info.is_str = builtin.is_str();
        }
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
        for (item, _) in k.query_external_importables(db, query) {
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
    let krate = module.krate(db);
    let crate_name = krate.display_name(db)
        .map(|n| n.to_string())
        .unwrap_or_default();
    
    if !crate_name.is_empty() {
        segments.insert(0, crate_name);
    }
    
    Some(segments.join("::"))
}

// =============================================================================
// VARIABLE USAGE TRACKING
// =============================================================================

/// Analyze variable usages (reads and writes) using semantic Definition::usages() API
fn analyze_variable_usages(
    sema: &Semantics<'_, RootDatabase>,
    source_file: &ast::SourceFile,
    variables: &mut [VariableTypeInfo],
) {
    // For each variable, find its Local definition and get usages
    for var_info in variables.iter_mut() {
        // Skip statics and consts - they don't have Local definitions
        if var_info.is_static || var_info.is_const {
            continue;
        }
        
        // Find the IdentPat for this variable by span
        let Some(ident_pat) = find_ident_pat_at_span(source_file, var_info.span_start, var_info.span_end) else {
            continue;
        };
        
        // Resolve to Local
        let Some(local) = sema.to_def(&ident_pat) else {
            continue;
        };
        
        // Create Definition from Local and find usages
        let def = Definition::Local(local);
        let usages = def.usages(sema).all();
        
        // Convert usages to VariableUsageInfo
        for (_file_id, refs) in usages.references {
            for reference in refs {
                let range = reference.range;
                let (line, column) = get_location(&range, source_file);
                
                // Determine usage kind from ReferenceCategory
                let kind = if reference.category.contains(ReferenceCategory::WRITE) && reference.category.contains(ReferenceCategory::READ) {
                    "read_write"
                } else if reference.category.contains(ReferenceCategory::WRITE) {
                    "write"
                } else {
                    "read"
                }.to_string();
                
                var_info.usages.push(VariableUsageInfo { line, column, kind });
            }
        }
    }
}

/// Find an IdentPat at the given span
fn find_ident_pat_at_span(source_file: &ast::SourceFile, span_start: u32, span_end: u32) -> Option<ast::IdentPat> {
    use ra_ap_syntax::TextRange;
    use ra_ap_syntax::TextSize;
    
    let start = TextSize::from(span_start);
    let end = TextSize::from(span_end);
    let range = TextRange::new(start, end);
    
    // Find the token at this range and walk up to find IdentPat
    let token = source_file.syntax().token_at_offset(start).right_biased()?;
    
    for ancestor in token.parent_ancestors() {
        if let Some(ident_pat) = ast::IdentPat::cast(ancestor.clone()) {
            if ident_pat.syntax().text_range() == range || ident_pat.syntax().text_range().contains_range(range) {
                return Some(ident_pat);
            }
        }
    }
    
    None
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
    display_target: &ra_ap_hir::DisplayTarget,
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
            .map(|ty| ty.display(db, *display_target).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get result type
        let result_type = sema
            .type_of_expr(&ast::Expr::MethodCallExpr(method_call.clone()))
            .map(|ti| ti.original.display(db, *display_target).to_string());

        // Resolve self borrow type (semantic)
        let self_borrow = resolve_self_borrow(sema, &method_call, db);

        // Get operation as the canonical method path (fully semantic)
        let operation = resolve_method_path(sema, &method_call, db);

        // Check if this is a trait method and get trait info (semantic)
        let (is_trait_method, trait_name) = resolve_trait_info(sema, &method_call, db);
        
        // Check if this method is unsafe to call (semantic via Function::is_unsafe_to_call)
        let is_unsafe = sema.resolve_method_call(&method_call)
            .filter(|func| func.is_unsafe_to_call(db, None, Edition::Edition2021))
            .map(|_| true);

        let method_info = MethodCallInfo {
            method: method_name,
            line: call_line,
            column,
            operation,
            self_borrow,
            receiver_type,
            result_type,
            is_trait_method,
            trait_name,
            is_unsafe,
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

/// Resolve trait information for a method call (semantic)
fn resolve_trait_info(
    sema: &Semantics<'_, RootDatabase>,
    method_call: &ast::MethodCallExpr,
    db: &RootDatabase,
) -> (Option<bool>, Option<String>) {
    let Some(func) = sema.resolve_method_call(method_call) else {
        return (None, None);
    };
    
    use ra_ap_hir::ItemContainer;
    match func.container(db) {
        ItemContainer::Trait(t) => {
            let trait_name = t.name(db).display_no_db(Edition::Edition2021).to_string();
            (Some(true), Some(trait_name))
        }
        ItemContainer::Impl(i) => {
            if let Some(trait_ref) = i.trait_(db) {
                let trait_name = trait_ref.name(db).display_no_db(Edition::Edition2021).to_string();
                (Some(true), Some(trait_name))
            } else {
                (Some(false), None)
            }
        }
        _ => (None, None),
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
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<ExpressionInfo> {
    let mut expressions = Vec::new();

    for node in source_file.syntax().descendants() {
        // Handle function calls: drop(x), thread::spawn(|| {}), transmute(x)
        if let Some(call) = ast::CallExpr::cast(node) {
            if let Some(expr_info) = analyze_call_expr(sema, db, tracked_functions, &call, source_file, display_target) {
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
    display_target: &ra_ap_hir::DisplayTarget,
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
    
    // Extract argument info and closure captures
    let first_arg = call.arg_list().and_then(|args| args.args().next());
    let argument = first_arg.as_ref().and_then(|arg| extract_argument_info(sema, arg));
    
    // Extract semantic closure captures if argument is a closure
    let closure_captures = first_arg
        .and_then(|arg| {
            if let ast::Expr::ClosureExpr(closure) = arg {
                Some(extract_closure_captures_semantic(sema, db, &closure, display_target))
            } else {
                None
            }
        })
        .unwrap_or_default();
    
    // Get result type
    let result_type = sema.type_of_expr(&ast::Expr::CallExpr(call.clone()))
        .map(|ti| ti.original.display(db, *display_target).to_string());

    // Check if this function is unsafe (semantic)
    let is_unsafe = if func.is_unsafe_to_call(db, None, Edition::Edition2021) {
        Some(true)
    } else {
        None
    };

    Some(ExpressionInfo {
        line,
        column,
        kind: "function_call".to_string(),
        path: Some(canonical_path.clone()),
        operation: canonical_path.clone(),
        argument,
        result_type,
        is_unsafe,
        closure_captures,
    })
}

/// Extract semantic closure captures with capture kinds using rust-analyzer
fn extract_closure_captures_semantic(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    closure: &ast::ClosureExpr,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<crate::output::ClosureCaptureInfo> {
    use ra_ap_hir::CaptureKind;
    
    // Try to get the closure's HIR representation
    let Some(closure_ty) = sema.type_of_expr(&ast::Expr::ClosureExpr(closure.clone())) else {
        return Vec::new();
    };
    
    let Some(closure_hir) = closure_ty.original.as_closure() else {
        return Vec::new();
    };
    
    closure_hir.captured_items(db)
        .into_iter()
        .map(|capture| {
            let local = capture.local();
            let name = local.name(db).display_no_db(Edition::Edition2021).to_string();
            let capture_kind = match capture.kind() {
                CaptureKind::SharedRef => "shared_ref",
                CaptureKind::UniqueSharedRef => "unique_shared_ref", 
                CaptureKind::MutableRef => "mutable_ref",
                CaptureKind::Move => "move",
            }.to_string();
            // Get the type of the local variable
            let ty = Some(local.ty(db).display(db, *display_target).to_string());
            
            crate::output::ClosureCaptureInfo { name, capture_kind, ty }
        })
        .collect()
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

/// Analyze unsafe operations in a file using semantic analysis
fn analyze_unsafe_operations(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
    unsafe_ops: &mut Vec<UnsafeOperationInfo>,
) {
    for node in source_file.syntax().descendants() {
        // 1. Raw pointer dereference: *ptr
        if let Some(prefix_expr) = ast::PrefixExpr::cast(node.clone()) {
            if prefix_expr.op_kind() == Some(ast::UnaryOp::Deref) {
                if let Some(inner) = prefix_expr.expr() {
                    // Check if the inner expression is a raw pointer type
                    if let Some(ty) = sema.type_of_expr(&inner) {
                        if ty.original.is_raw_ptr() {
                            let range = prefix_expr.syntax().text_range();
                            let (line, column) = get_location(&range, source_file);
                            let inside_unsafe = sema.is_inside_unsafe(&ast::Expr::PrefixExpr(prefix_expr.clone()));
                            unsafe_ops.push(UnsafeOperationInfo {
                                line,
                                column,
                                kind: "deref_raw_ptr".to_string(),
                                inside_unsafe_block: inside_unsafe,
                                context: Some(ty.original.display(db, *display_target).to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        // 2. Unsafe function calls
        if let Some(call) = ast::CallExpr::cast(node.clone()) {
            if let Some(callee) = call.expr() {
                if let ast::Expr::PathExpr(path_expr) = callee {
                    if let Some(path) = path_expr.path() {
                        if let Some(resolved) = sema.resolve_path(&path) {
                            if let ra_ap_hir::PathResolution::Def(ra_ap_hir::ModuleDef::Function(func)) = resolved {
                                let is_unsafe = func.is_unsafe_to_call(db, None, Edition::Edition2021);
                                let is_ffi = matches!(func.container(db), ItemContainer::ExternBlock(_));
                                
                                if is_unsafe || is_ffi {
                                    let range = call.syntax().text_range();
                                    let (line, column) = get_location(&range, source_file);
                                    let inside_unsafe = sema.is_inside_unsafe(&ast::Expr::CallExpr(call.clone()));
                                    let func_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
                                    
                                    let kind = if is_ffi { "ffi_call" } else { "call_unsafe_fn" };
                                    
                                    unsafe_ops.push(UnsafeOperationInfo {
                                        line,
                                        column,
                                        kind: kind.to_string(),
                                        inside_unsafe_block: inside_unsafe,
                                        context: Some(func_name),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 3. Unsafe method calls
        if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
            if let Some(func) = sema.resolve_method_call(&method_call) {
                if func.is_unsafe_to_call(db, None, Edition::Edition2021) {
                    let range = method_call.syntax().text_range();
                    let (line, column) = get_location(&range, source_file);
                    let inside_unsafe = sema.is_inside_unsafe(&ast::Expr::MethodCallExpr(method_call.clone()));
                    let method_name = func.name(db).display_no_db(Edition::Edition2021).to_string();
                    
                    unsafe_ops.push(UnsafeOperationInfo {
                        line,
                        column,
                        kind: "call_unsafe_method".to_string(),
                        inside_unsafe_block: inside_unsafe,
                        context: Some(method_name),
                    });
                }
            }
        }
        
        // 4. Mutable static access
        if let Some(path_expr) = ast::PathExpr::cast(node.clone()) {
            if let Some(path) = path_expr.path() {
                if let Some(resolved) = sema.resolve_path(&path) {
                    if let ra_ap_hir::PathResolution::Def(ra_ap_hir::ModuleDef::Static(static_def)) = resolved {
                        if static_def.is_mut(db) {
                            let range = path_expr.syntax().text_range();
                            let (line, column) = get_location(&range, source_file);
                            // Check if we're inside an unsafe block
                            // For path expressions, we need to find the parent expression
                            let inside_unsafe = path_expr.syntax().ancestors()
                                .find_map(|n| ast::Expr::cast(n))
                                .map(|e| sema.is_inside_unsafe(&e))
                                .unwrap_or(false);
                            let static_name = static_def.name(db).display_no_db(Edition::Edition2021).to_string();
                            
                            unsafe_ops.push(UnsafeOperationInfo {
                                line,
                                column,
                                kind: "access_mutable_static".to_string(),
                                inside_unsafe_block: inside_unsafe,
                                context: Some(static_name),
                            });
                        }
                    }
                }
            }
        }
        
        // 5. Unsafe ref expressions - removed in ra_ap_hir 0.0.318
        // 6. Unsafe ident patterns - removed in ra_ap_hir 0.0.318
    }
}

/// Analyze field accesses for partial borrow tracking (semantic via sema.resolve_field)
fn analyze_field_accesses(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<FieldAccessInfo> {
    let mut field_accesses = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(field_expr) = ast::FieldExpr::cast(node.clone()) {
            if let Some(resolved) = sema.resolve_field(&field_expr) {
                let range = field_expr.syntax().text_range();
                let (line, column) = get_location(&range, source_file);
                
                // Get the variable name from the receiver
                let variable = field_expr.expr()
                    .and_then(|e| {
                        if let ast::Expr::PathExpr(path) = e {
                            path.path()?.segment()?.name_ref().map(|n| n.text().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "<expr>".to_string());
                
                let (field_name, field_type) = match resolved {
                    either::Either::Left(field) => (
                        field.name(db).display_no_db(Edition::Edition2021).to_string(),
                        field.ty(db).display(db, *display_target).to_string(),
                    ),
                    either::Either::Right(tuple_field) => (
                        format!("{}", tuple_field.index),
                        tuple_field.ty(db).display(db, *display_target).to_string(),
                    ),
                };
                
                // Determine access kind based on context
                let access_kind = determine_field_access_kind(&field_expr);
                
                field_accesses.push(FieldAccessInfo {
                    line,
                    column,
                    variable,
                    field: field_name,
                    field_type,
                    access_kind,
                });
            }
        }
    }
    
    field_accesses
}

/// Determine the kind of field access (read, write, borrow_shared, borrow_mut)
fn determine_field_access_kind(field_expr: &ast::FieldExpr) -> String {
    // Check if this field access is the target of an assignment
    if let Some(parent) = field_expr.syntax().parent() {
        if let Some(bin_expr) = ast::BinExpr::cast(parent.clone()) {
            if bin_expr.op_kind() == Some(ast::BinaryOp::Assignment { op: None }) {
                if let Some(lhs) = bin_expr.lhs() {
                    if lhs.syntax().text_range() == field_expr.syntax().text_range() {
                        return "write".to_string();
                    }
                }
            }
        }
        // Check if it's being borrowed
        if let Some(ref_expr) = ast::RefExpr::cast(parent) {
            if ref_expr.mut_token().is_some() {
                return "borrow_mut".to_string();
            } else {
                return "borrow_shared".to_string();
            }
        }
    }
    "read".to_string()
}

/// Analyze closure traits (Fn/FnMut/FnOnce) using semantic Closure::fn_trait
fn analyze_closure_traits(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<ClosureTraitInfo> {
    let mut closure_traits = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(closure_expr) = ast::ClosureExpr::cast(node.clone()) {
            // Get the type of the closure expression
            if let Some(type_info) = sema.type_of_expr(&ast::Expr::ClosureExpr(closure_expr.clone())) {
                // Check if it's a closure type and get the fn_trait
                if let Some(closure) = type_info.original.as_closure() {
                    // Use semantic Closure::fn_trait() API - returns FnTrait enum
                    // FnTrait implements Display: Fn -> "Fn", FnMut -> "FnMut", FnOnce -> "FnOnce"
                    let fn_trait_str = closure.fn_trait(db).to_string();
                    
                    let range = closure_expr.syntax().text_range();
                    let (line, column) = get_location(&range, source_file);
                    
                    // Get captures
                    let captures = extract_closure_captures_semantic(sema, db, &closure_expr, display_target);
                    
                    closure_traits.push(ClosureTraitInfo {
                        line,
                        column,
                        fn_trait: fn_trait_str,
                        captures,
                    });
                }
            }
        }
    }
    
    closure_traits
}

/// Analyze enum variant constructions (semantic via sema.resolve_variant)
fn analyze_variants(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<VariantInfo> {
    let mut variants = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(record_expr) = ast::RecordExpr::cast(node.clone()) {
            if let Some(variant_def) = sema.resolve_variant(record_expr.clone()) {
                let range = record_expr.syntax().text_range();
                let (line, column) = get_location(&range, source_file);
                
                let (enum_type, variant_name, variant_kind, field_types) = match variant_def {
                    ra_ap_hir::VariantDef::Variant(v) => {
                        let enum_def = v.parent_enum(db);
                        let kind = match v.kind(db) {
                            StructKind::Record => "struct",
                            StructKind::Tuple => "tuple",
                            StructKind::Unit => "unit",
                        };
                        let fields: Vec<String> = v.fields(db)
                            .into_iter()
                            .map(|f| f.ty(db).display(db, *display_target).to_string())
                            .collect();
                        (
                            enum_def.name(db).display_no_db(Edition::Edition2021).to_string(),
                            v.name(db).display_no_db(Edition::Edition2021).to_string(),
                            kind.to_string(),
                            fields,
                        )
                    }
                    ra_ap_hir::VariantDef::Struct(s) => {
                        let kind = match s.kind(db) {
                            StructKind::Record => "struct",
                            StructKind::Tuple => "tuple",
                            StructKind::Unit => "unit",
                        };
                        let fields: Vec<String> = s.fields(db)
                            .into_iter()
                            .map(|f| f.ty(db).display(db, *display_target).to_string())
                            .collect();
                        (
                            s.name(db).display_no_db(Edition::Edition2021).to_string(),
                            s.name(db).display_no_db(Edition::Edition2021).to_string(),
                            kind.to_string(),
                            fields,
                        )
                    }
                    ra_ap_hir::VariantDef::Union(u) => {
                        let fields: Vec<String> = u.fields(db)
                            .into_iter()
                            .map(|f| f.ty(db).display(db, *display_target).to_string())
                            .collect();
                        (
                            u.name(db).display_no_db(Edition::Edition2021).to_string(),
                            u.name(db).display_no_db(Edition::Edition2021).to_string(),
                            "union".to_string(),
                            fields,
                        )
                    }
                };
                
                variants.push(VariantInfo {
                    line,
                    column,
                    enum_type,
                    variant_name,
                    variant_kind,
                    field_types,
                });
            }
        }
    }
    
    variants
}

/// Analyze lifetime parameters (semantic via sema.resolve_lifetime_param)
fn analyze_lifetimes(
    sema: &Semantics<'_, RootDatabase>,
    _db: &RootDatabase,
    source_file: &ast::SourceFile,
) -> Vec<LifetimeInfo> {
    let mut lifetimes = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(lifetime) = ast::Lifetime::cast(node.clone()) {
            if let Some(_lifetime_param) = sema.resolve_lifetime_param(&lifetime) {
                let range = lifetime.syntax().text_range();
                let (line, column) = get_location(&range, source_file);
                
                let name = lifetime.text().to_string();
                
                // Determine context from parent
                let context = lifetime.syntax().ancestors()
                    .find_map(|ancestor| {
                        if ast::Fn::can_cast(ancestor.kind()) {
                            Some("function")
                        } else if ast::Struct::can_cast(ancestor.kind()) {
                            Some("struct")
                        } else if ast::Enum::can_cast(ancestor.kind()) {
                            Some("enum")
                        } else if ast::Impl::can_cast(ancestor.kind()) {
                            Some("impl")
                        } else if ast::Trait::can_cast(ancestor.kind()) {
                            Some("trait")
                        } else if ast::TypeAlias::can_cast(ancestor.kind()) {
                            Some("type_alias")
                        } else {
                            None
                        }
                    })
                    .unwrap_or("unknown");
                
                lifetimes.push(LifetimeInfo {
                    line,
                    column,
                    name,
                    context: context.to_string(),
                });
            }
        }
    }
    
    lifetimes
}

/// Analyze loop labels (semantic via sema.resolve_label)
fn analyze_labels(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
) -> Vec<LabelInfo> {
    let mut labels = Vec::new();
    
    for node in source_file.syntax().descendants() {
        // Check for label references in break/continue
        if let Some(lifetime) = ast::Lifetime::cast(node.clone()) {
            if let Some(label) = sema.resolve_label(&lifetime) {
                let range = lifetime.syntax().text_range();
                let (line, column) = get_location(&range, source_file);
                
                let name = lifetime.text().to_string();
                
                // Get the loop kind from the label's source
                let loop_kind = label.source(db)
                    .and_then(|src| src.value.syntax().parent())
                    .and_then(|parent| {
                        if ast::LoopExpr::can_cast(parent.kind()) {
                            Some("loop")
                        } else if ast::WhileExpr::can_cast(parent.kind()) {
                            Some("while")
                        } else if ast::ForExpr::can_cast(parent.kind()) {
                            Some("for")
                        } else {
                            None
                        }
                    })
                    .unwrap_or("unknown");
                
                labels.push(LabelInfo {
                    line,
                    column,
                    name,
                    loop_kind: loop_kind.to_string(),
                });
            }
        }
    }
    
    labels
}

/// Analyze const pattern bindings (semantic via sema.resolve_bind_pat_to_const)
fn analyze_const_patterns(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<ConstPatternInfo> {
    let mut const_patterns = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(ident_pat) = ast::IdentPat::cast(node.clone()) {
            if let Some(module_def) = sema.resolve_bind_pat_to_const(&ident_pat) {
                let range = ident_pat.syntax().text_range();
                let (line, column) = get_location(&range, source_file);
                
                let (const_name, const_type) = match module_def {
                    ra_ap_hir::ModuleDef::Const(c) => (
                        c.name(db).map(|n| n.display_no_db(Edition::Edition2021).to_string()).unwrap_or_default(),
                        c.ty(db).display(db, *display_target).to_string(),
                    ),
                    _ => continue,
                };
                
                const_patterns.push(ConstPatternInfo {
                    line,
                    column,
                    const_name,
                    const_type,
                    const_value: None, // Value extraction would require const evaluation
                });
            }
        }
    }
    
    const_patterns
}

/// Analyze callable expressions (semantic via Type::as_callable, impls_fnonce)
fn analyze_callables(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<CallableInfo> {
    let mut callables = Vec::new();
    
    for node in source_file.syntax().descendants() {
        // Check call expressions where the callee might be a closure or fn pointer
        if let Some(call_expr) = ast::CallExpr::cast(node.clone()) {
            if let Some(callee) = call_expr.expr() {
                if let Some(type_info) = sema.type_of_expr(&callee) {
                    let ty = type_info.original;
                    
                    // Check if it's callable
                    if ty.impls_fnonce(db) {
                        if let Some(callable) = ty.as_callable(db) {
                            let range = call_expr.syntax().text_range();
                            let (line, column) = get_location(&range, source_file);
                            
                            let kind = if ty.is_closure() {
                                "closure"
                            } else if ty.is_fn() {
                                "fn_ptr"
                            } else {
                                "fn_trait"
                            };
                            
                            let param_types: Vec<String> = callable.params()
                                .into_iter()
                                .map(|p| p.ty().display(db, *display_target).to_string())
                                .collect();
                            
                            let return_type = Some(callable.return_type().display(db, *display_target).to_string());
                            
                            callables.push(CallableInfo {
                                line,
                                column,
                                kind: kind.to_string(),
                                param_types,
                                return_type,
                                is_callable: true,
                            });
                        }
                    }
                }
            }
        }
    }
    
    callables
}

/// Analyze record field expressions (semantic via sema.resolve_record_field)
fn analyze_record_field_exprs(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<RecordFieldExprInfo> {
    let mut record_fields = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(record_expr) = ast::RecordExpr::cast(node.clone()) {
            if let Some(field_list) = record_expr.record_expr_field_list() {
                for field in field_list.fields() {
                    if let Some((resolved_field, _, ty)) = sema.resolve_record_field(&field) {
                        let range = field.syntax().text_range();
                        let (line, column) = get_location(&range, source_file);
                        
                        let parent_type = resolved_field.parent_def(db);
                        let parent_type_name = match parent_type {
                            ra_ap_hir::VariantDef::Struct(s) => s.name(db).display_no_db(Edition::Edition2021).to_string(),
                            ra_ap_hir::VariantDef::Variant(v) => {
                                format!("{}::{}", 
                                    v.parent_enum(db).name(db).display_no_db(Edition::Edition2021),
                                    v.name(db).display_no_db(Edition::Edition2021))
                            }
                            ra_ap_hir::VariantDef::Union(u) => u.name(db).display_no_db(Edition::Edition2021).to_string(),
                        };
                        
                        let value_type = field.expr()
                            .and_then(|e| sema.type_of_expr(&e))
                            .map(|ti| ti.original.display(db, *display_target).to_string());
                        
                        record_fields.push(RecordFieldExprInfo {
                            line,
                            column,
                            parent_type: parent_type_name,
                            field_name: resolved_field.name(db).display_no_db(Edition::Edition2021).to_string(),
                            field_type: ty.display(db, *display_target).to_string(),
                            value_type,
                        });
                    }
                }
            }
        }
    }
    
    record_fields
}

/// Analyze record field patterns (semantic via sema.resolve_record_pat_field)
fn analyze_record_field_pats(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Vec<RecordFieldPatInfo> {
    let mut record_fields = Vec::new();
    
    for node in source_file.syntax().descendants() {
        if let Some(record_pat) = ast::RecordPat::cast(node.clone()) {
            if let Some(field_list) = record_pat.record_pat_field_list() {
                for field in field_list.fields() {
                    if let Some((resolved_field, ty)) = sema.resolve_record_pat_field(&field) {
                        let range = field.syntax().text_range();
                        let (line, column) = get_location(&range, source_file);
                        
                        let parent_type = resolved_field.parent_def(db);
                        let parent_type_name = match parent_type {
                            ra_ap_hir::VariantDef::Struct(s) => s.name(db).display_no_db(Edition::Edition2021).to_string(),
                            ra_ap_hir::VariantDef::Variant(v) => {
                                format!("{}::{}", 
                                    v.parent_enum(db).name(db).display_no_db(Edition::Edition2021),
                                    v.name(db).display_no_db(Edition::Edition2021))
                            }
                            ra_ap_hir::VariantDef::Union(u) => u.name(db).display_no_db(Edition::Edition2021).to_string(),
                        };
                        
                        record_fields.push(RecordFieldPatInfo {
                            line,
                            column,
                            parent_type: parent_type_name,
                            field_name: resolved_field.name(db).display_no_db(Edition::Edition2021).to_string(),
                            field_type: ty.display(db, *display_target).to_string(),
                        });
                    }
                }
            }
        }
    }
    
    record_fields
}

/// Update await points with poll function resolution (semantic via sema.resolve_await_to_poll)
fn update_await_points_with_poll(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    source_file: &ast::SourceFile,
    display_target: &ra_ap_hir::DisplayTarget,
    await_points: &mut Vec<crate::output::AwaitPointInfo>,
) {
    // Build a map of (line, column) -> await_point index for quick lookup
    let mut await_map: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
    for (idx, ap) in await_points.iter().enumerate() {
        await_map.insert((ap.line, ap.column), idx);
    }
    
    for node in source_file.syntax().descendants() {
        if let Some(await_expr) = ast::AwaitExpr::cast(node.clone()) {
            let range = await_expr.syntax().text_range();
            let (line, column) = get_location(&range, source_file);
            
            // Find the corresponding await point
            if let Some(&idx) = await_map.get(&(line, column)) {
                // Resolve the poll function (semantic via sema.resolve_await_to_poll)
                if let Some(poll_fn) = sema.resolve_await_to_poll(&await_expr) {
                    let poll_fn_name = poll_fn.name(db).display_no_db(Edition::Edition2021).to_string();
                    let container = poll_fn.container(db);
                    let full_path = match container {
                        ItemContainer::Trait(t) => {
                            format!("{}::{}", t.name(db).display_no_db(Edition::Edition2021), poll_fn_name)
                        }
                        ItemContainer::Impl(i) => {
                            if let Some(trait_ref) = i.trait_(db) {
                                format!("<{} as {}>::{}", 
                                    i.self_ty(db).display(db, *display_target),
                                    trait_ref.name(db).display_no_db(Edition::Edition2021),
                                    poll_fn_name)
                            } else {
                                format!("<{}>::{}", 
                                    i.self_ty(db).display(db, *display_target),
                                    poll_fn_name)
                            }
                        }
                        _ => poll_fn_name,
                    };
                    
                    await_points[idx].poll_function = Some(full_path);
                }
            }
        }
    }
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
    let krate = module.krate(db);
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


/// Analyze borrow spans
fn analyze_borrow_spans(
    sema: &Semantics<'_, RootDatabase>,
    file_id: ra_ap_ide_db::EditionedFileId,
    source_file: &ast::SourceFile,
    borrow_spans: &mut Vec<BorrowSpanInfo>,
) {
    for node in source_file.syntax().descendants() {
        // Look for let statements with reference initializers: let r = &x or let r = &mut x
        if let Some(let_stmt) = ast::LetStmt::cast(node.clone()) {
            let Some(pat) = let_stmt.pat() else { continue };
            let Some(init) = let_stmt.initializer() else { continue };
            
            // Check if initializer is a reference expression
            let Some(ref_expr) = ast::RefExpr::cast(init.syntax().clone()) else { continue };
            let Some(borrowed_expr) = ref_expr.expr() else { continue };
            
            // Get the binding from the pattern
            let Some(ident_pat) = ast::IdentPat::cast(pat.syntax().clone()) else { continue };
            let Some(local) = sema.to_def(&ident_pat) else { continue };
            
            let range = ref_expr.syntax().text_range();
            let (start_line, start_column) = get_location(&range, source_file);
            let kind = if ref_expr.mut_token().is_some() { "mutable" } else { "shared" };
            let variable = borrowed_expr.syntax().text().to_string();
            
            // Find all usages of the reference variable to determine borrow span
            let def = Definition::Local(local);
            let mut use_sites = Vec::new();
            let mut end_line: Option<u32> = None;
            let mut end_column: Option<u32> = None;
            
            let search_scope = ra_ap_ide_db::search::SearchScope::single_file(file_id);
            for (_, refs) in def.usages(sema).in_scope(&search_scope).all() {
                for r in refs {
                    let (use_line, use_col) = get_location(&r.range, source_file);
                    use_sites.push((use_line, use_col));
                    
                    // Track the last use (highest line, or highest column on same line)
                    let is_later = match (end_line, end_column) {
                        (None, _) => true,
                        (Some(el), Some(ec)) => use_line > el || (use_line == el && use_col > ec),
                        (Some(el), None) => use_line >= el,
                    };
                    if is_later {
                        end_line = Some(use_line);
                        end_column = Some(use_col);
                    }
                }
            }
            
            // Sort use sites by line then column
            use_sites.sort();
            
            borrow_spans.push(BorrowSpanInfo {
                variable,
                kind: kind.to_string(),
                start_line,
                start_column,
                end_line,
                end_column,
                use_sites,
            });
        }
    }
}

/// Analyze destructuring patterns
fn analyze_destructuring_pattern(
    _sema: &Semantics<'_, RootDatabase>,
    _db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    source_file: &ast::SourceFile,
) -> Option<DestructuringInfo> {
    let let_stmt = ast::LetStmt::cast(node.clone())?;
    let pat = let_stmt.pat()?;
    let range = pat.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    
    let (kind, bindings) = match &pat {
        ast::Pat::TuplePat(_) => ("tuple", extract_bindings(&pat)),
        ast::Pat::RecordPat(_) => ("struct", extract_bindings(&pat)),
        ast::Pat::SlicePat(_) => ("slice", extract_bindings(&pat)),
        ast::Pat::TupleStructPat(_) => ("tuple_struct", extract_bindings(&pat)),
        _ => return None,
    };
    
    if bindings.is_empty() { return None; }
    
    Some(DestructuringInfo {
        line, column,
        kind: kind.to_string(),
        source_expr: let_stmt.initializer().map(|i| i.syntax().text().to_string()),
        bindings,
    })
}

fn extract_bindings(pat: &ast::Pat) -> Vec<String> {
    let mut bindings = Vec::new();
    for node in pat.syntax().descendants() {
        if let Some(ident) = ast::IdentPat::cast(node) {
            if let Some(name) = ident.name() {
                bindings.push(name.text().to_string());
            }
        }
    }
    bindings
}

/// Analyze match bindings
fn analyze_match_bindings(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    source_file: &ast::SourceFile,
    match_bindings: &mut Vec<MatchBindingInfo>,
    display_target: &ra_ap_hir::DisplayTarget,
) {
    let Some(match_expr) = ast::MatchExpr::cast(node.clone()) else { return };
    let Some(arm_list) = match_expr.match_arm_list() else { return };
    
    for arm in arm_list.arms() {
        if let Some(pat) = arm.pat() {
            if let Some(info) = make_match_binding_info(sema, db, &pat, source_file, "match", display_target) {
                match_bindings.push(info);
            }
        }
    }
}

/// Analyze if-let bindings
fn analyze_if_let_bindings(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    source_file: &ast::SourceFile,
    match_bindings: &mut Vec<MatchBindingInfo>,
    display_target: &ra_ap_hir::DisplayTarget,
) {
    let Some(if_expr) = ast::IfExpr::cast(node.clone()) else { return };
    let Some(cond) = if_expr.condition() else { return };
    if let Some(let_expr) = ast::LetExpr::cast(cond.syntax().clone()) {
        if let Some(pat) = let_expr.pat() {
            if let Some(info) = make_match_binding_info(sema, db, &pat, source_file, "if_let", display_target) {
                match_bindings.push(info);
            }
        }
    }
}

/// Analyze while-let bindings
fn analyze_while_let_bindings(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    node: &ra_ap_syntax::SyntaxNode,
    source_file: &ast::SourceFile,
    match_bindings: &mut Vec<MatchBindingInfo>,
    display_target: &ra_ap_hir::DisplayTarget,
) {
    let Some(while_expr) = ast::WhileExpr::cast(node.clone()) else { return };
    let Some(cond) = while_expr.condition() else { return };
    if let Some(let_expr) = ast::LetExpr::cast(cond.syntax().clone()) {
        if let Some(pat) = let_expr.pat() {
            if let Some(info) = make_match_binding_info(sema, db, &pat, source_file, "while_let", display_target) {
                match_bindings.push(info);
            }
        }
    }
}

fn make_match_binding_info(
    sema: &Semantics<'_, RootDatabase>,
    db: &RootDatabase,
    pat: &ast::Pat,
    source_file: &ast::SourceFile,
    context: &str,
    display_target: &ra_ap_hir::DisplayTarget,
) -> Option<MatchBindingInfo> {
    let range = pat.syntax().text_range();
    let (line, column) = get_location(&range, source_file);
    let mut bindings = Vec::new();
    
    for node in pat.syntax().descendants() {
        if let Some(ident) = ast::IdentPat::cast(node) {
            if let Some(name) = ident.name() {
                let mode = sema.binding_mode_of_pat(&ident)
                    .map(|m| match m {
                        BindingMode::Move => "move",
                        BindingMode::Ref(Mutability::Shared) => "ref",
                        BindingMode::Ref(Mutability::Mut) => "ref_mut",
                    })
                    .unwrap_or("move").to_string();
                let ty = sema.to_def(&ident).map(|l| l.ty(db).display(db, *display_target).to_string());
                bindings.push(PatternBindingInfo { name: name.text().to_string(), mode, ty });
            }
        }
    }
    
    if bindings.is_empty() { return None; }
    Some(MatchBindingInfo { line, column, pattern: pat.syntax().text().to_string(), bindings, context: context.to_string() })
}
