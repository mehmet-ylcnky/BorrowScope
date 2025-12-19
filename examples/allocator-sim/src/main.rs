//! Allocator Simulator - Demonstrates ALL BorrowScope Runtime Features
//!
//! A mini memory allocator that showcases every tracking function.

use borrowscope_runtime::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Constants and Statics
// ============================================================================
const BLOCK_SIZE: usize = 64;
const POOL_SIZE: usize = 8;

static mut ALLOC_COUNT: usize = 0;
static mut FREE_COUNT: usize = 0;

// ============================================================================
// Memory Block Types
// ============================================================================
#[repr(C)]
union BlockData {
    bytes: [u8; BLOCK_SIZE],
    next_free: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum BlockState {
    Free,
    Allocated,
}

struct Block {
    data: BlockData,
    state: BlockState,
    id: usize,
}

// ============================================================================
// Main
// ============================================================================
fn main() {
    reset();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        Allocator Simulator - Full Runtime Demo               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Track const evaluation
    let block_size = track_const_eval("BLOCK_SIZE", 0, "usize", "main.rs:const", BLOCK_SIZE);
    let pool_size = track_const_eval("POOL_SIZE", 1, "usize", "main.rs:const", POOL_SIZE);
    println!("Config: BLOCK_SIZE={}, POOL_SIZE={}\n", block_size, pool_size);

    demo_basic_allocation();
    demo_borrowed_slices();
    demo_shared_blocks();
    demo_thread_safe_pool();
    demo_interior_mutable_cache();
    demo_raw_memory_ops();
    demo_unsafe_allocator();
    demo_ffi_interop();
    demo_type_punning();
    demo_union_access();

    print_summary();
}

// ============================================================================
// 1. Basic Allocation (track_new, track_move, track_drop)
// ============================================================================
fn demo_basic_allocation() {
    println!("━━━ 1. Basic Allocation ━━━\n");

    // Allocate blocks
    let block1 = track_new("block1", vec![0u8; BLOCK_SIZE]);
    let block2 = track_new("block2", vec![0u8; BLOCK_SIZE]);
    
    print_memory_map(&["block1", "block2"], &[true, true]);

    // Move ownership
    let moved_block = track_move("block1", "moved_block", block1);
    println!("  block1 moved to moved_block");

    // Drop
    track_drop("moved_block");
    track_drop("block2");
    drop(moved_block);
    drop(block2);

    // Batch drop
    let temps: Vec<_> = (0..3).map(|i| {
        track_new(&format!("temp{}", i), vec![0u8; 16])
    }).collect();
    track_drop_batch(&["temp0", "temp1", "temp2"]);
    drop(temps);

    println!("  ✓ Basic allocation complete\n");
}

// ============================================================================
// 2. Borrowed Slices (track_borrow, track_borrow_mut)
// ============================================================================
fn demo_borrowed_slices() {
    println!("━━━ 2. Borrowed Slices ━━━\n");

    let mut buffer = track_new("buffer", vec![0u8; 256]);

    // Multiple immutable borrows
    {
        let slice1 = track_borrow("slice1", &buffer[0..64]);
        let slice2 = track_borrow("slice2", &buffer[64..128]);
        let slice3 = track_borrow("slice3", &buffer[128..192]);

        println!("  Three concurrent read slices:");
        print_slice_map(&[("slice1", 0, 64), ("slice2", 64, 128), ("slice3", 128, 192)]);

        track_drop("slice3");
        track_drop("slice2");
        track_drop("slice1");
    }

    // Mutable borrow
    {
        let write_slice = track_borrow_mut("write_slice", &mut buffer[0..64]);
        write_slice[0] = 0xFF;
        println!("  Mutable slice write: buffer[0] = 0xFF");
        track_drop("write_slice");
    }

    track_drop("buffer");
    println!("  ✓ Borrowed slices complete\n");
}

// ============================================================================
// 3. Shared Blocks with Rc (track_rc_new, track_rc_clone)
// ============================================================================
fn demo_shared_blocks() {
    println!("━━━ 3. Shared Blocks (Rc) ━━━\n");

    let shared_block = track_rc_new("shared", Rc::new(vec![1u8; BLOCK_SIZE]));
    println!("  shared created, count: {}", Rc::strong_count(&shared_block));

    let reader1 = track_rc_clone("reader1", "shared", Rc::clone(&shared_block));
    let reader2 = track_rc_clone("reader2", "shared", Rc::clone(&shared_block));
    let reader3 = track_rc_clone("reader3", "shared", Rc::clone(&shared_block));

    print_rc_sharing(Rc::strong_count(&shared_block));

    // Drop readers
    track_drop("reader3");
    drop(reader3);
    println!("  reader3 dropped, count: {}", Rc::strong_count(&shared_block));

    track_drop("reader2");
    track_drop("reader1");
    track_drop("shared");

    println!("  ✓ Shared blocks complete\n");
}

// ============================================================================
// 4. Thread-Safe Pool with Arc (track_arc_new, track_arc_clone)
// ============================================================================
fn demo_thread_safe_pool() {
    println!("━━━ 4. Thread-Safe Pool (Arc) ━━━\n");

    let pool = track_arc_new("pool", Arc::new(vec![vec![0u8; BLOCK_SIZE]; POOL_SIZE]));
    println!("  Pool created with {} blocks", POOL_SIZE);

    // Simulate thread access
    let thread1 = track_arc_clone("thread1_pool", "pool", Arc::clone(&pool));
    let thread2 = track_arc_clone("thread2_pool", "pool", Arc::clone(&pool));

    print_arc_threads(Arc::strong_count(&pool));

    // Borrow from Arc
    {
        let pool_ref = track_borrow("pool_view", &*thread1);
        println!("  Thread1 reading pool: {} blocks", pool_ref.len());
        track_drop("pool_view");
    }

    track_drop("thread2_pool");
    track_drop("thread1_pool");
    track_drop("pool");

    println!("  ✓ Thread-safe pool complete\n");
}

// ============================================================================
// 5. Interior Mutable Cache (RefCell, Cell)
// ============================================================================
fn demo_interior_mutable_cache() {
    println!("━━━ 5. Interior Mutable Cache ━━━\n");

    // RefCell for complex cache
    let cache = track_refcell_new("cache", RefCell::new(Vec::<(usize, Vec<u8>)>::new()));

    // Add to cache
    {
        let mut cache_mut = refcell_borrow_mut!("cache_write", "cache", cache.borrow_mut());
        cache_mut.push((0, vec![1, 2, 3]));
        cache_mut.push((1, vec![4, 5, 6]));
        println!("  Cache populated: {} entries", cache_mut.len());
        refcell_drop!("cache_write");
    }

    // Read from cache
    {
        let cache_ref = refcell_borrow!("cache_read", "cache", cache.borrow());
        println!("  Cache read: {:?}", cache_ref.get(0));
        refcell_drop!("cache_read");
    }

    track_drop("cache");

    // Cell for simple counter
    let hit_count = track_cell_new("hit_count", Cell::new(0u32));
    
    // Increment counter
    let current = track_cell_get("hit_count", "main.rs:get", hit_count.get());
    track_cell_set("hit_count", "main.rs:set");
    hit_count.set(current + 1);
    
    track_cell_set("hit_count", "main.rs:set");
    hit_count.set(hit_count.get() + 1);
    
    println!("  Hit count: {}", hit_count.get());
    track_drop("hit_count");

    println!("  ✓ Interior mutable cache complete\n");
}

// ============================================================================
// 6. Raw Memory Operations (track_raw_ptr, track_raw_ptr_deref)
// ============================================================================
fn demo_raw_memory_ops() {
    println!("━━━ 6. Raw Memory Operations ━━━\n");

    let mut data = track_new("raw_data", [0u8; 32]);

    // Create raw pointers
    let ptr: *const [u8; 32] = track_raw_ptr("ptr", 0, "*const [u8; 32]", "main.rs:ptr", &data as *const _);
    let ptr_mut: *mut [u8; 32] = track_raw_ptr_mut("ptr_mut", 1, "*mut [u8; 32]", "main.rs:ptr_mut", &mut data as *mut _);

    print_raw_pointers(ptr as usize, ptr_mut as usize);

    // Dereference
    track_unsafe_block_enter(0, "main.rs:unsafe");
    unsafe {
        track_raw_ptr_deref(0, "main.rs:read", false);
        let first_byte = (*ptr)[0];
        println!("  Read via *const: {}", first_byte);

        track_raw_ptr_deref(1, "main.rs:write", true);
        (*ptr_mut)[0] = 0xAB;
        println!("  Write via *mut: 0xAB");
    }
    track_unsafe_block_exit(0, "main.rs:unsafe_end");

    track_drop("raw_data");
    println!("  ✓ Raw memory ops complete\n");
}

// ============================================================================
// 7. Unsafe Allocator Functions
// ============================================================================
fn demo_unsafe_allocator() {
    println!("━━━ 7. Unsafe Allocator ━━━\n");

    // Track static access
    let _ = track_static_init("ALLOC_COUNT", 0, "usize", false, 0usize);
    let _ = track_static_init("FREE_COUNT", 1, "usize", false, 0usize);

    // Simulate allocation
    track_unsafe_block_enter(1, "main.rs:alloc");
    unsafe {
        track_unsafe_fn_call("allocate_block", "main.rs:alloc_fn");
        allocate_block();
        
        track_static_access(0, "ALLOC_COUNT", true, "main.rs:inc_alloc");
        ALLOC_COUNT += 1;
        
        track_static_access(0, "ALLOC_COUNT", false, "main.rs:read_alloc");
        println!("  Allocations: {}", ALLOC_COUNT);
    }
    track_unsafe_block_exit(1, "main.rs:alloc_end");

    // Simulate free
    track_unsafe_block_enter(2, "main.rs:free");
    unsafe {
        track_unsafe_fn_call("free_block", "main.rs:free_fn");
        free_block();
        
        track_static_access(1, "FREE_COUNT", true, "main.rs:inc_free");
        FREE_COUNT += 1;
        
        track_static_access(1, "FREE_COUNT", false, "main.rs:read_free");
        println!("  Frees: {}", FREE_COUNT);
    }
    track_unsafe_block_exit(2, "main.rs:free_end");

    println!("  ✓ Unsafe allocator complete\n");
}

unsafe fn allocate_block() {
    // Simulated allocation
}

unsafe fn free_block() {
    // Simulated free
}

// ============================================================================
// 8. FFI Interop (track_ffi_call)
// ============================================================================
fn demo_ffi_interop() {
    println!("━━━ 8. FFI Interop ━━━\n");

    println!("  Simulating C allocator calls:");

    track_ffi_call("malloc", "main.rs:malloc");
    println!("    → malloc(64) called");

    track_ffi_call("realloc", "main.rs:realloc");
    println!("    → realloc(ptr, 128) called");

    track_ffi_call("free", "main.rs:free");
    println!("    → free(ptr) called");

    track_ffi_call("mmap", "main.rs:mmap");
    println!("    → mmap(NULL, 4096, ...) called");

    track_ffi_call("munmap", "main.rs:munmap");
    println!("    → munmap(ptr, 4096) called");

    println!("  ✓ FFI interop complete\n");
}

// ============================================================================
// 9. Type Punning (track_transmute)
// ============================================================================
fn demo_type_punning() {
    println!("━━━ 9. Type Punning ━━━\n");

    track_unsafe_block_enter(3, "main.rs:transmute_block");
    unsafe {
        // Transmute bytes to u32
        let bytes: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        track_transmute("[u8; 4]", "u32", "main.rs:to_u32");
        let value: u32 = std::mem::transmute(bytes);
        println!("  [0x01, 0x02, 0x03, 0x04] → u32: 0x{:08X}", value);

        // Transmute pointer to usize
        let ptr = &bytes as *const _;
        track_transmute("*const [u8; 4]", "usize", "main.rs:ptr_to_usize");
        let addr: usize = std::mem::transmute(ptr);
        println!("  Pointer → usize: 0x{:X}", addr);
    }
    track_unsafe_block_exit(3, "main.rs:transmute_end");

    println!("  ✓ Type punning complete\n");
}

// ============================================================================
// 10. Union Field Access
// ============================================================================
fn demo_union_access() {
    println!("━━━ 10. Union Field Access ━━━\n");

    #[repr(C)]
    union BlockHeader {
        size: usize,
        next_ptr: *mut u8,
        flags: [u8; 8],
    }

    let header = BlockHeader { size: 64 };

    track_unsafe_block_enter(4, "main.rs:union_block");
    unsafe {
        track_union_field_access("BlockHeader", "size", "main.rs:read_size");
        println!("  header.size = {}", header.size);

        track_union_field_access("BlockHeader", "flags", "main.rs:read_flags");
        println!("  header.flags = {:?}", header.flags);
    }
    track_unsafe_block_exit(4, "main.rs:union_end");

    println!("  ✓ Union access complete\n");
}

// ============================================================================
// Summary
// ============================================================================
fn print_summary() {
    let events = get_events();
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Event Summary                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    
    // Count by type
    let mut counts = std::collections::HashMap::new();
    for e in &events {
        let name = match e {
            Event::New { .. } => "New",
            Event::Borrow { .. } => "Borrow",
            Event::Move { .. } => "Move",
            Event::Drop { .. } => "Drop",
            Event::RcNew { .. } => "RcNew",
            Event::RcClone { .. } => "RcClone",
            Event::ArcNew { .. } => "ArcNew",
            Event::ArcClone { .. } => "ArcClone",
            Event::RefCellNew { .. } => "RefCellNew",
            Event::RefCellBorrow { .. } => "RefCellBorrow",
            Event::RefCellDrop { .. } => "RefCellDrop",
            Event::CellNew { .. } => "CellNew",
            Event::CellGet { .. } => "CellGet",
            Event::CellSet { .. } => "CellSet",
            Event::StaticInit { .. } => "StaticInit",
            Event::StaticAccess { .. } => "StaticAccess",
            Event::ConstEval { .. } => "ConstEval",
            Event::RawPtrCreated { .. } => "RawPtrCreated",
            Event::RawPtrDeref { .. } => "RawPtrDeref",
            Event::UnsafeBlockEnter { .. } => "UnsafeBlockEnter",
            Event::UnsafeBlockExit { .. } => "UnsafeBlockExit",
            Event::UnsafeFnCall { .. } => "UnsafeFnCall",
            Event::FfiCall { .. } => "FfiCall",
            Event::Transmute { .. } => "Transmute",
            Event::UnionFieldAccess { .. } => "UnionFieldAccess",
        };
        *counts.entry(name).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    for (name, count) in sorted {
        println!("║  {:.<40} {:>5} ║", name, count);
    }
    
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  TOTAL EVENTS {:>42} ║", events.len());
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Export
    let path = std::env::temp_dir().join("allocator-sim.json");
    export_json(&path).unwrap();
    println!("\nExported to: {}", path.display());
}

// ============================================================================
// ASCII Visualization Helpers
// ============================================================================
fn print_memory_map(names: &[&str], allocated: &[bool]) {
    println!("  Memory Map:");
    println!("  ┌────────┬────────┬────────┬────────┐");
    print!("  │");
    for (i, name) in names.iter().enumerate() {
        if allocated[i] {
            print!(" {:^6} │", name);
        } else {
            print!("  free  │");
        }
    }
    println!();
    println!("  └────────┴────────┴────────┴────────┘");
}

fn print_slice_map(slices: &[(&str, usize, usize)]) {
    println!("  ┌─────────────────────────────────────────────────────────┐");
    println!("  │ 0        64       128      192      256                 │");
    print!("  │ ");
    for (name, start, end) in slices {
        let width = (end - start) / 16;
        print!("[{:^width$}]", name, width = width);
    }
    println!("                 │");
    println!("  └─────────────────────────────────────────────────────────┘");
}

fn print_rc_sharing(count: usize) {
    println!("  Rc Sharing (count={}):", count);
    println!("       ┌─────────┐");
    println!("       │ shared  │◄─┬─ reader1");
    println!("       │  block  │◄─┼─ reader2");
    println!("       │         │◄─┴─ reader3");
    println!("       └─────────┘");
}

fn print_arc_threads(count: usize) {
    println!("  Arc Pool (count={}):", count);
    println!("       ┌─────────────────────┐");
    println!("       │    Thread-Safe      │");
    println!("       │       Pool          │");
    println!("       └──────────┬──────────┘");
    println!("            ┌─────┴─────┐");
    println!("         thread1     thread2");
}

fn print_raw_pointers(ptr: usize, ptr_mut: usize) {
    println!("  Raw Pointers:");
    println!("    *const → 0x{:016X}", ptr);
    println!("    *mut   → 0x{:016X}", ptr_mut);
}
