//! # BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.
//!
//! ## Quick Start
//!
//! ```ignore
//! use borrowscope_macro::trace_borrow;
//! use borrowscope_runtime::*;
//!
//! #[trace_borrow]
//! fn example() {
//!     let x = String::from("hello");  // New event
//!     let y = &x;                      // Borrow event
//!     let z = x;                       // Move event
//! }                                    // Drop events
//!
//! fn main() {
//!     reset();
//!     example();
//!     println!("{:?}", get_events());
//! }
//! ```
//!
//! ## Attribute Options
//!
//! ### Presets
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow]` | Standard tracking (recommended) |
//! | `#[trace_borrow(quiet)]` | Ownership only (new, move, drop, borrow) |
//! | `#[trace_borrow(verbose)]` | All tracking including noisy features |
//!
//! ### Feature Selection
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow(skip = "loops,branches")]` | Skip specific feature groups |
//! | `#[trace_borrow(only = "ownership")]` | Enable only specified feature groups |
//!
//! ### Filtering & Sampling (Performance)
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow(filter = "data*")]` | Only track variables matching glob pattern |
//! | `#[trace_borrow(sample = 0.1)]` | Track ~10% of operations (probabilistic) |
//!
//! ### Conditional Compilation
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow(debug_only)]` | Only track in debug builds |
//! | `#[trace_borrow(release_only)]` | Only track in release builds |
//! | `#[trace_borrow(feature = "tracing")]` | Only track when cargo feature enabled |
//!
//! ## Feature Groups
//!
//! Use these group names with `skip` or `only` options:
//!
//! | Group | Aliases | Description |
//! |-------|---------|-------------|
//! | `ownership` | - | Variable creation, moves, drops, borrows |
//! | `smart_pointers` | `pointers` | Rc, Arc, RefCell, Cell operations |
//! | `loops` | - | for, while, loop tracking |
//! | `branches` | - | if/else, match tracking |
//! | `control_flow` | `control` | break, continue, return |
//! | `try` | - | ? operator |
//! | `methods` | - | clone, lock, unwrap |
//! | `async` | - | async blocks, await |
//! | `unsafe` | - | unsafe blocks, raw pointers, transmute |
//! | `expressions` | `exprs` | struct, tuple, array, range, cast |
//! | `functions` | `fn` | Function entry/exit (disabled by default) |
//!
//! ## Filtering
//!
//! Filter which variables are tracked using glob patterns:
//!
//! ```ignore
//! #[trace_borrow(filter = "data*")]      // Track vars starting with "data"
//! #[trace_borrow(filter = "*_count")]    // Track vars ending with "_count"
//! #[trace_borrow(filter = "user_?")]     // Track user_1, user_2, etc.
//! ```
//!
//! **Pattern syntax:**
//! - `*` matches zero or more characters
//! - `?` matches exactly one character
//!
//! **Note:** Filtering is applied at compile-time. No tracking code is generated
//! for variables that don't match the pattern, resulting in zero overhead.
//!
//! ## Sampling
//!
//! Reduce tracking overhead by only recording a percentage of operations:
//!
//! ```ignore
//! #[trace_borrow(sample = 0.1)]   // Track ~10% of operations
//! #[trace_borrow(sample = 0.5)]   // Track ~50% of operations
//! #[trace_borrow(sample = 1.0)]   // Track 100% (same as no sampling)
//! ```
//!
//! **Use cases:**
//! - High-frequency loops where full tracking is too expensive
//! - Production monitoring with minimal overhead
//! - Statistical analysis where sampling is acceptable
//!
//! **Note:** Sampling uses a fast PRNG (xorshift64) for minimal overhead.
//! The decision is made at runtime for each tracking call.
//!
//! ## Conditional Compilation
//!
//! Control when tracking code is included:
//!
//! ```ignore
//! // Only in debug builds (recommended for development)
//! #[trace_borrow(debug_only)]
//! fn dev_function() { }
//!
//! // Only in release builds (for production monitoring)
//! #[trace_borrow(release_only)]
//! fn prod_function() { }
//!
//! // Only when cargo feature is enabled
//! #[trace_borrow(feature = "tracing")]
//! fn optional_tracing() { }
//! ```
//!
//! **Generated code:**
//! - `debug_only` → `#[cfg(debug_assertions)]`
//! - `release_only` → `#[cfg(not(debug_assertions))]`
//! - `feature = "x"` → `#[cfg(feature = "x")]`
//!
//! ## Combining Options
//!
//! Multiple options can be combined:
//!
//! ```ignore
//! // Debug-only, quiet mode
//! #[trace_borrow(debug_only, quiet)]
//!
//! // Filter + sampling for high-performance tracking
//! #[trace_borrow(filter = "user*", sample = 0.1)]
//!
//! // Feature-gated with specific groups
//! #[trace_borrow(feature = "trace", only = "ownership,smart_pointers")]
//!
//! // Skip noisy features, debug only
//! #[trace_borrow(debug_only, skip = "loops,branches,expressions")]
//! ```
//!
//! ## Tracked Operations
//!
//! ### Basic Ownership (`ownership` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `let x = value;` | `New` |
//! | `let y = &x;` | `Borrow` |
//! | `let y = &mut x;` | `Borrow` (mutable) |
//! | `let y = x;` (move) | `Move` |
//! | Scope exit | `Drop` |
//!
//! ### Smart Pointers (`smart_pointers` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `Rc::new(v)` | `RcNew` |
//! | `Rc::clone(&rc)` | `RcClone` |
//! | `Arc::new(v)` | `ArcNew` |
//! | `Arc::clone(&arc)` | `ArcClone` |
//! | `Box::new(v)` | `BoxNew` |
//! | `Box::pin(v)` | `PinNew` |
//! | `RefCell::new(v)` | `RefCellNew` |
//! | `refcell.borrow()` | `RefCellBorrow` |
//! | `refcell.borrow_mut()` | `RefCellBorrowMut` |
//! | `Cell::new(v)` | `CellNew` |
//! | `cell.get()` | `CellGet` |
//! | `cell.set(v)` | `CellSet` |
//!
//! ### Loops (`loops` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `for`/`while`/`loop` entry | `LoopEnter` |
//! | Each iteration | `LoopIteration` |
//! | Loop end | `LoopExit` |
//!
//! ### Branches (`branches` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `if`/`else` | `Branch` |
//! | `match` entry | `MatchEnter` |
//! | Match arm taken | `MatchArm` |
//! | Match end | `MatchExit` |
//!
//! ### Control Flow (`control_flow` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `break` | `Break` |
//! | `continue` | `Continue` |
//! | `return` | `Return` |
//!
//! ### Other Groups
//!
//! | Group | Code Patterns | Events |
//! |-------|---------------|--------|
//! | `try` | `expr?` | `Try` |
//! | `methods` | `.clone()`, `.lock()`, `.unwrap()` | `Clone`, `Lock`, `Unwrap` |
//! | `async` | `async { }`, `.await` | `AsyncBlockEnter/Exit`, `AwaitStart/End` |
//! | `unsafe` | `unsafe { }`, `*ptr`, `transmute` | `UnsafeBlockEnter/Exit`, `RawPtrDeref`, `Transmute` |
//! | `expressions` | structs, tuples, arrays, ranges, casts | `StructCreate`, `TupleCreate`, etc. |
//! | `functions` | fn entry/exit | `FnEnter`, `FnExit` |
//!
//! ## Advanced Smart Pointer Tracking
//!
//! Beyond basic `Rc`, `Arc`, `RefCell`, and `Cell`, the macro tracks:
//!
//! ### Weak References
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `Rc::downgrade(&rc)` | `WeakNew` |
//! | `Arc::downgrade(&arc)` | `WeakNewSync` |
//! | `weak.upgrade()` | `WeakUpgrade` / `WeakUpgradeSync` |
//! | `weak.clone()` | `WeakClone` / `WeakCloneSync` |
//!
//! ### Pin, Cow, OnceCell, MaybeUninit
//!
//! | Type | Operations |
//! |------|------------|
//! | `Box` | `Box::pin`, `Box::into_raw`, `Box::from_raw` |
//! | `Pin<T>` | `Pin::new`, `Pin::into_inner` |
//! | `Cow<T>` | `Cow::Borrowed`, `Cow::Owned`, `to_mut()` |
//! | `OnceCell<T>` | `new()`, `set()`, `get()`, `get_or_init()` |
//! | `OnceLock<T>` | `new()`, `set()`, `get()`, `get_or_init()` |
//! | `MaybeUninit<T>` | `uninit()`, `new()`, `write()`, `assume_init()` |
//!
//! ## Concurrency Tracking
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `thread::spawn(...)` | `ThreadSpawn` |
//! | `handle.join()` | `ThreadJoin` |
//! | `mpsc::channel()` | `ChannelNew` |
//! | `tx.send(v)` | `ChannelSend` |
//! | `rx.recv()` | `ChannelRecv` |
//! | `rx.try_recv()` | `ChannelTryRecv` |
//!
//! ## Expression Tracking (`expressions` group)
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `Point { x, y }` | `StructCreate` (with type name) |
//! | `(a, b, c)` | `TupleCreate` (with arity) |
//! | `[1, 2, 3]` | `ArrayCreate` (with length) |
//! | `0..10` | `Range` (half_open) |
//! | `0..=10` | `Range` (closed) |
//! | `x as i64` | `TypeCast` (with target type) |
//!
//! ## Closure Tracking
//!
//! | Code Pattern | Event |
//! |--------------|-------|
//! | `\|x\| x + 1` | `ClosureCreate` (capture mode: ref) |
//! | `move \|x\| x + 1` | `ClosureCreate` (capture mode: move) |
//! | Captured variable | `ClosureCapture` (per variable) |
//!
//! ## Diagnostic Options
//!
//! For patterns that cannot be auto-detected, use diagnostic attributes:
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow(warn)]` | Emit warnings for ambiguous patterns |
//! | `#[trace_borrow(ffi = ["malloc"])]` | Declare known FFI functions |
//! | `#[trace_borrow(unions = ["MyUnion"])]` | Declare known union types |
//! | `#[trace_borrow(statics = ["GLOBAL"])]` | Declare known static variables |
//!
//! ## How It Works
//!
//! The macro transforms functions by:
//!
//! 1. **Parsing** the function into an AST using `syn`
//! 2. **Walking** the AST with `OwnershipVisitor` that maintains:
//!    - Unique IDs for each variable (for event correlation)
//!    - Scope stack for LIFO drop ordering
//!    - Type context (tracks which vars are Weak, Cow, OnceCell, etc.)
//! 3. **Injecting** `borrowscope_runtime::track_*` calls
//! 4. **Generating** drop calls at scope exits in reverse order
//!
//! ### ID-Based Correlation
//!
//! Each variable gets a unique ID, enabling correlation:
//! - Borrows link to their owner's ID
//! - Clones link to their source's ID
//! - Moves link source and destination IDs
//!
//! ## Performance Tips
//!
//! 1. **Use `quiet` mode** for minimal overhead when you only need ownership tracking
//! 2. **Use `filter`** to track only relevant variables (zero overhead for non-matching)
//! 3. **Use `sample`** for high-frequency code paths
//! 4. **Use `debug_only`** to eliminate all overhead in release builds
//! 5. **Use `skip`** to disable noisy features like loops and branches
//!
//! ```ignore
//! // Minimal overhead configuration
//! #[trace_borrow(debug_only, quiet, filter = "important_*")]
//! fn performance_critical() { }
//! ```
//!
//! ## Common Patterns
//!
//! ```ignore
//! // Development: full tracking, debug only
//! #[trace_borrow(debug_only)]
//! fn dev_function() { }
//!
//! // Learning: ownership only, cleaner output
//! #[trace_borrow(quiet)]
//! fn learning_example() { }
//!
//! // Production monitoring: sampled, feature-gated
//! #[trace_borrow(feature = "monitoring", sample = 0.01)]
//! fn production_function() { }
//!
//! // Debugging specific variables
//! #[trace_borrow(filter = "suspect_*", verbose)]
//! fn debug_specific() { }
//! ```
//!
//! ## Limitations
//!
//! - **const fn**: Cannot be used (tracking requires runtime)
//! - **extern fn**: Cannot be used (only Rust ABI supported)
//! - **async fn**: Works but may not capture all ownership across await points
//! - **unsafe fn**: Works but tracking cannot verify safety invariants
//! - **Macros**: Variables created inside macro expansions may not be tracked
//!
//! ## Troubleshooting
//!
//! **No events recorded:**
//! - Ensure `borrowscope_runtime` has `features = ["track"]` enabled
//! - Call `reset()` before the traced function
//! - Check if `debug_only` is set but running in release mode
//!
//! **Too many events:**
//! - Use `quiet` mode or `only = "ownership"`
//! - Use `skip = "loops,branches"` to reduce noise
//! - Use `filter` to track specific variables
//!
//! **Performance issues:**
//! - Use `sample = 0.1` or lower for high-frequency code
//! - Use `debug_only` to disable in release builds
//! - Use `filter` to reduce tracked variables

mod best_practices;
mod borrow_detection;
mod codegen;
mod config;
mod examples;
mod formatting;
mod generic_handler;
mod diagnostics;
mod hygiene;
mod optimized_transform;
mod parser;
mod pattern;
mod span_utils;
mod transform_visitor;
mod type_info;
mod validation;
mod visitor;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, ItemFn};
use transform_visitor::OwnershipVisitor;

/// Validate function before transformation
fn validate_function(func: &ItemFn) {
    // Check for const functions
    if func.sig.constness.is_some() {
        abort!(
            func.sig.constness,
            "const functions cannot be tracked";
            help = "remove the `const` keyword to enable tracking";
            note = "tracking requires runtime operations, but const functions are evaluated at compile time"
        );
    }

    // Check for extern functions
    if let Some(abi) = &func.sig.abi {
        abort!(
            abi,
            "extern functions cannot be tracked";
            help = "only Rust ABI functions can be tracked"
        );
    }

    // Warn about async functions (they work but with limitations)
    if func.sig.asyncness.is_some() {
        // Note: We allow async but could add a warning in the future
    }

    // Warn about unsafe functions (they work but can't verify safety)
    if func.sig.unsafety.is_some() {
        // Note: We allow unsafe but tracking may be incomplete
    }
}

/// Attribute macro to trace ownership and borrowing in a function.
///
/// This macro transforms a function to inject runtime tracking calls that record
/// ownership transfers, borrows, drops, and other operations. The events can be
/// retrieved using `borrowscope_runtime::get_events()`.
///
/// # Basic Usage
///
/// ```ignore
/// use borrowscope_macro::trace_borrow;
/// use borrowscope_runtime::*;
///
/// #[trace_borrow]
/// fn example() {
///     let x = String::from("hello");  // New event
///     let y = &x;                      // Borrow event
///     let z = x;                       // Move event
/// }                                    // Drop events
/// ```
///
/// # Attribute Options
///
/// ## `quiet` - Minimal tracking
///
/// Only tracks basic ownership: new, move, drop, borrow.
///
/// ```ignore
/// #[trace_borrow(quiet)]
/// fn minimal() {
///     let x = vec![1, 2, 3];
///     for i in &x { }  // Loop NOT tracked
/// }
/// ```
///
/// ## `verbose` - All tracking
///
/// Enables all tracking features (same as default currently).
///
/// ```ignore
/// #[trace_borrow(verbose)]
/// fn everything() { }
/// ```
///
/// ## `skip` - Disable specific features
///
/// Comma-separated list of feature groups to disable.
///
/// ```ignore
/// #[trace_borrow(skip = "loops,branches")]
/// fn skip_noisy() {
///     for i in 0..10 { }  // NOT tracked
///     if true { }         // NOT tracked
/// }
/// ```
///
/// ## `only` - Enable only specific features
///
/// Comma-separated list of feature groups to enable (all others disabled).
///
/// ```ignore
/// #[trace_borrow(only = "ownership,functions")]
/// fn focused() {
///     let x = 1;  // Tracked (ownership)
///     // FnEnter/FnExit tracked (functions)
/// }
/// ```
///
/// # Feature Groups
///
/// | Group | Aliases | What it tracks |
/// |-------|---------|----------------|
/// | `ownership` | - | `let`, moves, drops, borrows |
/// | `smart_pointers` | `pointers` | Rc, Arc, RefCell, Cell |
/// | `loops` | - | for, while, loop |
/// | `branches` | - | if/else, match |
/// | `control_flow` | `control` | break, continue, return |
/// | `try` | - | `?` operator |
/// | `methods` | - | clone, lock, unwrap |
/// | `async` | - | async blocks, await |
/// | `unsafe` | - | unsafe blocks, raw pointers |
/// | `expressions` | `exprs` | struct, tuple, array, range, cast |
/// | `functions` | `fn` | Function entry/exit (off by default) |
///
/// # Conditional Compilation
///
/// | Option | Description |
/// |--------|-------------|
/// | `debug_only` | Only instrument in debug builds |
/// | `release_only` | Only instrument in release builds |
/// | `feature = "name"` | Only instrument when cargo feature is enabled |
///
/// # Limitations
///
/// - Cannot be used on `const fn` (tracking requires runtime)
/// - Cannot be used on `extern` functions
/// - Async functions work but may miss some ownership across await points
#[proc_macro_attribute]
#[proc_macro_error]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute arguments
    let args = parse_macro_input!(attr as config::TraceArgs);

    // Parse the input as a function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function
    validate_function(&input_fn);

    // Clone for transformation
    let mut transformed_fn = input_fn.clone();

    // Transform the function body using OwnershipVisitor with config
    let mut visitor = OwnershipVisitor::with_config(args.config.clone());
    visitor.visit_item_fn_mut(&mut transformed_fn);

    // Collect warnings
    let warnings = visitor.take_warnings();

    // Generate output based on conditional mode
    let output = if args.config.conditional_mode.is_conditional() {
        // Generate both versions with cfg attributes
        let cfg_tokens = args.config.conditional_mode.cfg_tokens().unwrap();
        let neg_cfg_tokens = match &args.config.conditional_mode {
            config::ConditionalMode::DebugOnly => quote! { #[cfg(not(debug_assertions))] },
            config::ConditionalMode::ReleaseOnly => quote! { #[cfg(debug_assertions)] },
            config::ConditionalMode::Feature(name) => {
                let feature_name = syn::LitStr::new(name, proc_macro2::Span::call_site());
                quote! { #[cfg(not(feature = #feature_name))] }
            }
            config::ConditionalMode::Always => unreachable!(),
        };

        quote! {
            #(#warnings)*

            #cfg_tokens
            #transformed_fn

            #neg_cfg_tokens
            #input_fn
        }
    } else {
        // Always instrument
        quote! {
            #(#warnings)*

            #transformed_fn
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    use std::sync::Once;
    static INIT: Once = Once::new();
    fn init_type_info() {
        INIT.call_once(|| {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let test_project = manifest_dir.join("tests/semantic_test_project");
            crate::type_info::load_from_path(&test_project);
        });
    }

    fn transform_function(func: &mut ItemFn) {
        init_type_info();
        let mut visitor = OwnershipVisitor::new();
        visitor.visit_item_fn_mut(func);
    }

    #[test]
    fn test_transform_simple_variable() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_new"));
    }

    #[test]
    fn test_transform_borrow() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
                let y = &x;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_borrow"));
    }

    #[test]
    fn test_transform_mut_borrow() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let mut x = 5;
                let y = &mut x;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("track_borrow_mut"));
    }

    #[test]
    fn test_preserves_function_signature() {
        let mut func: ItemFn = parse_quote! {
            fn example(a: i32) -> i32 {
                let x = a;
                x
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("fn example"));
        assert!(output.contains("a : i32"));
        assert!(output.contains("-> i32"));
    }

    #[test]
    fn test_preserves_generics() {
        let mut func: ItemFn = parse_quote! {
            fn example<T>(value: T) -> T {
                value
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("fn example"));
        assert!(output.contains("< T >"));
    }

    #[test]
    fn test_no_transform_without_init() {
        let mut func: ItemFn = parse_quote! {
            fn example() {
                let x;
                x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        // Should not add tracking for uninitialized variables
        assert!(!output.contains("track_new"));
    }

    #[test]
    fn test_preserves_visibility() {
        let mut func: ItemFn = parse_quote! {
            pub fn example() {
                let x = 5;
            }
        };

        transform_function(&mut func);
        let output = quote! { #func }.to_string();

        assert!(output.contains("pub fn example"));
    }

    #[test]
    fn test_conditional_mode_cfg_tokens() {
        use config::ConditionalMode;

        // debug_only
        let mode = ConditionalMode::DebugOnly;
        let tokens = mode.cfg_tokens().unwrap().to_string();
        assert!(tokens.contains("debug_assertions"));

        // release_only
        let mode = ConditionalMode::ReleaseOnly;
        let tokens = mode.cfg_tokens().unwrap().to_string();
        assert!(tokens.contains("not"));
        assert!(tokens.contains("debug_assertions"));

        // feature
        let mode = ConditionalMode::Feature("tracing".to_string());
        let tokens = mode.cfg_tokens().unwrap().to_string();
        assert!(tokens.contains("feature"));
        assert!(tokens.contains("tracing"));
    }
}
