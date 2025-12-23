//! Conditional Compilation Example
//!
//! Demonstrates how to use conditional compilation to control when
//! tracking code is generated, enabling zero-overhead in production.
//!
//! Run with:
//!   cargo run --example conditional_compilation
//!   cargo run --example conditional_compilation --release
//!   cargo run --example conditional_compilation --features tracing

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{get_events, reset};

/// Only instrumented in debug builds.
/// In release builds, this compiles to the original function with no overhead.
#[trace_borrow(debug_only)]
fn debug_only_tracking() {
    let data = vec![1, 2, 3];
    let sum: i32 = data.iter().sum();
    println!("  Sum: {}", sum);
}

/// Only instrumented in release builds.
/// Useful for production-only monitoring.
#[trace_borrow(release_only)]
fn release_only_tracking() {
    let message = String::from("production");
    println!("  Message: {}", message);
}

/// Only instrumented when the "tracing" feature is enabled.
/// Add to Cargo.toml: [features] tracing = []
#[trace_borrow(feature = "tracing")]
fn feature_gated_tracking() {
    let config = std::collections::HashMap::<String, i32>::new();
    println!("  Config entries: {}", config.len());
}

/// Combines conditional compilation with other options.
#[trace_borrow(debug_only, skip(loops, branches))]
fn combined_options() {
    let mut total = 0;
    for i in 0..5 {
        total += i;
    }
    println!("  Total: {}", total);
}

fn main() {
    println!("=== Conditional Compilation Demo ===\n");

    // Detect build mode
    #[cfg(debug_assertions)]
    println!("Build mode: DEBUG");
    #[cfg(not(debug_assertions))]
    println!("Build mode: RELEASE");

    #[cfg(feature = "tracing")]
    println!("Feature 'tracing': ENABLED");
    #[cfg(not(feature = "tracing"))]
    println!("Feature 'tracing': DISABLED");

    println!();

    // Test debug_only
    reset();
    println!("1. debug_only function:");
    debug_only_tracking();
    let events = get_events();
    #[cfg(debug_assertions)]
    println!("   → {} events captured (debug build)\n", events.len());
    #[cfg(not(debug_assertions))]
    println!("   → {} events captured (release build - no tracking)\n", events.len());

    // Test release_only
    reset();
    println!("2. release_only function:");
    release_only_tracking();
    let events = get_events();
    #[cfg(debug_assertions)]
    println!("   → {} events captured (debug build - no tracking)\n", events.len());
    #[cfg(not(debug_assertions))]
    println!("   → {} events captured (release build)\n", events.len());

    // Test feature-gated
    reset();
    println!("3. feature-gated function:");
    feature_gated_tracking();
    let events = get_events();
    #[cfg(feature = "tracing")]
    println!("   → {} events captured (feature enabled)\n", events.len());
    #[cfg(not(feature = "tracing"))]
    println!("   → {} events captured (feature disabled - no tracking)\n", events.len());

    // Test combined options
    reset();
    println!("4. combined options (debug_only + skip):");
    combined_options();
    let events = get_events();
    println!("   → {} events captured\n", events.len());

    println!("=== Demo Complete ===");
}
