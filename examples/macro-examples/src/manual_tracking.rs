//! Manual Tracking Example
//!
//! This example demonstrates how to manually instrument patterns that the
//! `#[trace_borrow]` macro cannot auto-detect due to the type information barrier.
//!
//! These patterns require semantic (type) information that procedural macros
//! cannot access:
//! - FFI function calls
//! - Union field access
//! - Static variable access
//!
//! Run with: cargo run --example manual_tracking -p borrowscope-macro --features "borrowscope-runtime/track"

use borrowscope_runtime::{
    get_events, reset, track_ffi_call, track_static_access, track_static_init,
    track_union_field_access,
};
use std::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// FFI Example
// =============================================================================

// Simulated FFI declarations (in real code, these would be extern "C")
mod ffi_simulation {
    /// Simulates an FFI function - in real code this would be:
    /// ```
    /// extern "C" {
    ///     fn process_buffer(ptr: *const u8, len: usize) -> i32;
    /// }
    /// ```
    pub fn process_buffer(ptr: *const u8, len: usize) -> i32 {
        // Simulate some processing
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        slice.iter().map(|&b| b as i32).sum()
    }

    pub fn get_system_time() -> u64 {
        // Simulate getting system time
        1234567890
    }
}

/// Wrapper that tracks FFI calls
fn tracked_process_buffer(data: &[u8]) -> i32 {
    // Track BEFORE the FFI call
    track_ffi_call("process_buffer", "manual_tracking.rs:42");

    // Make the actual FFI call
    ffi_simulation::process_buffer(data.as_ptr(), data.len())
}

fn tracked_get_system_time() -> u64 {
    track_ffi_call("get_system_time", "manual_tracking.rs:48");
    ffi_simulation::get_system_time()
}

// =============================================================================
// Union Example
// =============================================================================

/// A union for reinterpreting data
#[repr(C)]
union NumberUnion {
    int_val: i32,
    float_val: f32,
    bytes: [u8; 4],
}

/// Read union as integer with tracking
fn read_as_int(data: &NumberUnion) -> i32 {
    track_union_field_access("NumberUnion", "int_val", "manual_tracking.rs:64");
    unsafe { data.int_val }
}

/// Read union as float with tracking
fn read_as_float(data: &NumberUnion) -> f32 {
    track_union_field_access("NumberUnion", "float_val", "manual_tracking.rs:70");
    unsafe { data.float_val }
}

/// Read union as bytes with tracking
fn read_as_bytes(data: &NumberUnion) -> [u8; 4] {
    track_union_field_access("NumberUnion", "bytes", "manual_tracking.rs:76");
    unsafe { data.bytes }
}

// =============================================================================
// Static Variable Example
// =============================================================================

static REQUEST_COUNTER: AtomicU32 = AtomicU32::new(0);
static ERROR_COUNTER: AtomicU32 = AtomicU32::new(0);

// Unique IDs for static variables (use consistent IDs throughout your code)
const REQUEST_COUNTER_ID: usize = 1;
const ERROR_COUNTER_ID: usize = 2;

/// Initialize static tracking (call once at program start)
fn init_statics() {
    track_static_init(
        "REQUEST_COUNTER",
        REQUEST_COUNTER_ID,
        "AtomicU32",
        false, // not mutable (interior mutability via Atomic)
        (),
    );
    track_static_init(
        "ERROR_COUNTER",
        ERROR_COUNTER_ID,
        "AtomicU32",
        false,
        (),
    );
}

/// Increment request counter with tracking
fn increment_requests() {
    track_static_access(
        REQUEST_COUNTER_ID,
        "REQUEST_COUNTER",
        true, // is_write = true
        "manual_tracking.rs:108",
    );
    REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);
}

/// Read request counter with tracking
fn get_request_count() -> u32 {
    track_static_access(
        REQUEST_COUNTER_ID,
        "REQUEST_COUNTER",
        false, // is_write = false
        "manual_tracking.rs:119",
    );
    REQUEST_COUNTER.load(Ordering::SeqCst)
}

/// Increment error counter with tracking
fn increment_errors() {
    track_static_access(
        ERROR_COUNTER_ID,
        "ERROR_COUNTER",
        true,
        "manual_tracking.rs:130",
    );
    ERROR_COUNTER.fetch_add(1, Ordering::SeqCst);
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    reset();

    println!("=== Manual Tracking Example ===\n");
    println!("Demonstrating manual instrumentation for patterns that");
    println!("the #[trace_borrow] macro cannot auto-detect.\n");

    // Initialize static tracking
    println!("--- Static Variable Initialization ---");
    init_statics();

    // FFI calls
    println!("\n--- FFI Calls ---");
    let data = vec![1u8, 2, 3, 4, 5];
    let sum = tracked_process_buffer(&data);
    println!("FFI process_buffer returned: {}", sum);

    let time = tracked_get_system_time();
    println!("FFI get_system_time returned: {}", time);

    // Union access
    println!("\n--- Union Field Access ---");
    let num = NumberUnion { int_val: 42 };

    let as_int = read_as_int(&num);
    println!("Union as int: {}", as_int);

    let as_float = read_as_float(&num);
    println!("Union as float: {}", as_float);

    let as_bytes = read_as_bytes(&num);
    println!("Union as bytes: {:?}", as_bytes);

    // Static access
    println!("\n--- Static Variable Access ---");
    increment_requests();
    increment_requests();
    increment_requests();
    increment_errors();

    let requests = get_request_count();
    println!("Request count: {}", requests);

    // Print tracked events
    println!("\n--- Tracked Events ---");
    let events = get_events();
    for event in &events {
        println!("{:?}", event);
    }

    println!("\n--- Summary ---");
    println!("Total events tracked: {}", events.len());
    println!(
        "  - StaticInit events: {}",
        events
            .iter()
            .filter(|e| format!("{:?}", e).contains("StaticInit"))
            .count()
    );
    println!(
        "  - StaticAccess events: {}",
        events
            .iter()
            .filter(|e| format!("{:?}", e).contains("StaticAccess"))
            .count()
    );
    println!(
        "  - FfiCall events: {}",
        events
            .iter()
            .filter(|e| format!("{:?}", e).contains("FfiCall"))
            .count()
    );
    println!(
        "  - UnionFieldAccess events: {}",
        events
            .iter()
            .filter(|e| format!("{:?}", e).contains("UnionFieldAccess"))
            .count()
    );
}
