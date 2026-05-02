//! Better Error Messages Example
//!
//! Demonstrates compile-time warnings for ambiguous patterns that cannot
//! be fully analyzed due to the type information barrier.
//!
//! Run with:
//!   cargo run --example better_error_messages 2>&1
//!
//! The warnings appear during compilation, not at runtime.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};

// Simulated FFI functions (in real code, these would be extern "C")
fn c_read(_ptr: *const u8) -> i32 {
    42
}
fn malloc(_size: usize) -> *mut u8 {
    std::ptr::null_mut()
}
fn free(_ptr: *mut u8) {}

// Simulated static (in real code, this would be `static`)
const GLOBAL_CONFIG: i32 = 100;
const MAX_SIZE: usize = 1024;

// Simulated union-like type
struct RawData {
    value: i32,
}

struct PackedValue {
    data: u64,
}

// ============================================================================
// Example 1: With warnings enabled - shows potential issues
// ============================================================================

/// This function has `warn` enabled, so it will emit warnings for
/// ambiguous patterns during compilation.
#[trace_borrow(warn)]
fn with_warnings_enabled() {
    // This will warn: "cannot determine if this is an FFI call"
    // Hint: use #[trace_borrow(ffi = ["c_read"])] to track as FFI
    let ptr = std::ptr::null();
    let _result = c_read(ptr);

    // This will warn: "cannot determine if this is an FFI call"
    let _mem = malloc(100);

    // This will warn: "cannot determine if this is a static variable"
    // Hint: use #[trace_borrow(statics = ["GLOBAL_CONFIG"])] to track static access
    let _config = GLOBAL_CONFIG;
    let _size = MAX_SIZE;

    // This will warn: "cannot determine if this is a union field access"
    // Hint: use #[trace_borrow(unions = ["RawData"])] to track union access
    let raw = RawData { value: 42 };
    let _v = raw.value;

    println!("  Function completed with warnings shown during compilation");
}

// ============================================================================
// Example 2: With known declarations - suppresses specific warnings
// ============================================================================

/// This function declares known FFI functions, statics, and unions,
/// so those specific warnings are suppressed.
#[trace_borrow(warn, ffi = ["c_read", "malloc", "free"], statics = ["GLOBAL_CONFIG", "MAX_SIZE"], unions = ["RawData", "PackedValue"])]
fn with_known_declarations() {
    // No warning - c_read is declared as known FFI
    let ptr = std::ptr::null();
    let _result = c_read(ptr);

    // No warning - malloc is declared as known FFI
    let mem = malloc(100);
    free(mem);

    // No warning - GLOBAL_CONFIG is declared as known static
    let _config = GLOBAL_CONFIG;
    let _size = MAX_SIZE;

    // No warning - RawData is declared as known union
    let raw = RawData { value: 42 };
    let _v = raw.value;

    let packed = PackedValue { data: 0x1234 };
    let _d = packed.data;

    println!("  Function completed - no warnings for declared items");
}

// ============================================================================
// Example 3: Without warnings - default behavior
// ============================================================================

/// This function doesn't enable warnings, so no diagnostics are emitted.
/// This is the default behavior for backward compatibility.
#[trace_borrow]
fn without_warnings() {
    let ptr = std::ptr::null();
    let _result = c_read(ptr);
    let _config = GLOBAL_CONFIG;

    println!("  Function completed - no warnings (default behavior)");
}

// ============================================================================
// Example 4: Transmute warning
// ============================================================================

/// Transmute always emits a warning when `warn` is enabled because
/// type information is unavailable at macro expansion time.
#[trace_borrow(warn)]
fn transmute_example() {
    let x: u32 = 0x12345678;
    // This will warn about transmute type info being unavailable
    let _y: f32 = unsafe { std::mem::transmute(x) };

    println!("  Transmute completed - warning shown during compilation");
}

fn main() {
    println!("=== Better Error Messages Demo ===\n");
    println!("Note: Warnings appear during COMPILATION, not at runtime.\n");
    println!("Look at the compiler output above to see the warnings.\n");

    // Example 1: With warnings
    reset();
    println!("1. Function with warnings enabled:");
    with_warnings_enabled();
    println!("   Events: {}\n", get_events().len());

    // Example 2: With known declarations
    reset();
    println!("2. Function with known declarations (warnings suppressed):");
    with_known_declarations();
    println!("   Events: {}\n", get_events().len());

    // Example 3: Without warnings
    reset();
    println!("3. Function without warnings (default):");
    without_warnings();
    println!("   Events: {}\n", get_events().len());

    // Example 4: Transmute
    reset();
    println!("4. Transmute example:");
    transmute_example();
    println!("   Events: {}\n", get_events().len());

    println!("=== Demo Complete ===");
    println!("\nTo see the warnings, look at the compiler output during build.");
    println!("The warnings include:");
    println!("  - Possible FFI calls (c_read, malloc, free)");
    println!("  - Possible static variables (GLOBAL_CONFIG, MAX_SIZE)");
    println!("  - Possible union field access (RawData, PackedValue)");
    println!("  - Transmute type information unavailable");
}
