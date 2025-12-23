//! # BorrowScope Procedural Macros
//!
//! This crate provides the `#[trace_borrow]` attribute macro that instruments
//! Rust code to track ownership and borrowing operations at runtime.
//!
//! # Quick Start
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
//! # Attribute Options
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `#[trace_borrow]` | Default: all standard tracking enabled |
//! | `#[trace_borrow(quiet)]` | Ownership only (new, move, drop, borrow) |
//! | `#[trace_borrow(verbose)]` | All tracking including noisy features |
//! | `#[trace_borrow(skip = "loops,branches")]` | Skip specific feature groups |
//! | `#[trace_borrow(only = "ownership")]` | Enable only specified feature groups |
//!
//! # Feature Groups
//!
//! Use these group names with `skip` or `only` options:
//!
//! | Group | Aliases | Description | Events Generated |
//! |-------|---------|-------------|------------------|
//! | `ownership` | - | Variable creation, moves, drops, borrows | `New`, `Move`, `Drop`, `Borrow` |
//! | `smart_pointers` | `pointers` | Rc, Arc, RefCell, Cell operations | `RcNew`, `RcClone`, `ArcNew`, `ArcClone`, `RefCellNew`, `RefCellBorrow`, `CellNew`, `CellGet`, `CellSet` |
//! | `loops` | - | for, while, loop tracking | `LoopEnter`, `LoopIteration`, `LoopExit` |
//! | `branches` | - | if/else, match tracking | `Branch`, `MatchEnter`, `MatchArm`, `MatchExit` |
//! | `control_flow` | `control` | break, continue, return | `Break`, `Continue`, `Return` |
//! | `try` | - | ? operator | `Try` |
//! | `methods` | - | clone, lock, unwrap | `Clone`, `Lock`, `Unwrap` |
//! | `async` | - | async blocks, await | `AsyncBlockEnter`, `AsyncBlockExit`, `AwaitStart`, `AwaitEnd` |
//! | `unsafe` | - | unsafe blocks, raw pointers, transmute | `UnsafeBlockEnter`, `UnsafeBlockExit`, `RawPtrCreated`, `RawPtrDeref`, `Transmute` |
//! | `expressions` | `exprs` | struct, tuple, array, range, cast | `StructCreate`, `TupleCreate`, `ArrayCreate`, `Range`, `TypeCast`, `ClosureCreate` |
//! | `functions` | `fn` | Function entry/exit (disabled by default) | `FnEnter`, `FnExit` |
//!
//! # Tracked Operations
//!
//! ## Basic Ownership (`ownership` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `let x = value;` | `track_new()` | `New` |
//! | `let y = &x;` | `track_borrow()` | `Borrow` |
//! | `let y = &mut x;` | `track_borrow_mut()` | `Borrow` (mutable) |
//! | `let y = x;` (move) | `track_move()` | `Move` |
//! | Scope exit | `track_drop()` | `Drop` |
//!
//! ## Smart Pointers (`smart_pointers` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `Rc::new(v)` | `track_rc_new()` | `RcNew` |
//! | `Rc::clone(&rc)` | `track_rc_clone()` | `RcClone` |
//! | `Arc::new(v)` | `track_arc_new()` | `ArcNew` |
//! | `Arc::clone(&arc)` | `track_arc_clone()` | `ArcClone` |
//! | `RefCell::new(v)` | `track_refcell_new()` | `RefCellNew` |
//! | `refcell.borrow()` | `track_refcell_borrow()` | `RefCellBorrow` |
//! | `refcell.borrow_mut()` | `track_refcell_borrow_mut()` | `RefCellBorrow` |
//! | `Cell::new(v)` | `track_cell_new()` | `CellNew` |
//! | `cell.get()` | `track_cell_get()` | `CellGet` |
//! | `cell.set(v)` | `track_cell_set()` | `CellSet` |
//!
//! ## Loops (`loops` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `for x in iter { }` | `track_loop_enter()` | `LoopEnter` |
//! | Each iteration | `track_loop_iteration()` | `LoopIteration` |
//! | Loop end | `track_loop_exit()` | `LoopExit` |
//! | `while cond { }` | Same as for | Same |
//! | `loop { }` | Same as for | Same |
//!
//! ## Branches (`branches` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `if cond { }` | `track_branch()` | `Branch` |
//! | `if cond { } else { }` | `track_branch()` | `Branch` |
//! | `match expr { }` | `track_match_enter()` | `MatchEnter` |
//! | Match arm taken | `track_match_arm()` | `MatchArm` |
//! | Match end | `track_match_exit()` | `MatchExit` |
//!
//! ## Control Flow (`control_flow` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `break;` | `track_break()` | `Break` |
//! | `break 'label;` | `track_break()` | `Break` (with label) |
//! | `continue;` | `track_continue()` | `Continue` |
//! | `return expr;` | `track_return()` | `Return` |
//!
//! ## Try Operator (`try` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `expr?` | `track_try()` | `Try` |
//!
//! ## Methods (`methods` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `.clone()` | `track_clone()` | `Clone` |
//! | `.lock()` / `.read()` / `.write()` | `track_lock()` | `Lock` |
//! | `.unwrap()` / `.expect()` | `track_unwrap()` | `Unwrap` |
//!
//! ## Async (`async` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `async { }` | `track_async_block_enter()` | `AsyncBlockEnter` |
//! | Async block end | `track_async_block_exit()` | `AsyncBlockExit` |
//! | `.await` start | `track_await_start()` | `AwaitStart` |
//! | `.await` end | `track_await_end()` | `AwaitEnd` |
//!
//! ## Unsafe (`unsafe` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `unsafe { }` | `track_unsafe_block_enter()` | `UnsafeBlockEnter` |
//! | Unsafe block end | `track_unsafe_block_exit()` | `UnsafeBlockExit` |
//! | `&raw const x` | `track_raw_ptr()` | `RawPtrCreated` |
//! | `*ptr` | `track_raw_ptr_deref()` | `RawPtrDeref` |
//! | `transmute()` | `track_transmute()` | `Transmute` |
//!
//! ## Expressions (`expressions` group)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | `Point { x, y }` | `track_struct_create()` | `StructCreate` |
//! | `(a, b, c)` | `track_tuple_create()` | `TupleCreate` |
//! | `[1, 2, 3]` | `track_array_create()` | `ArrayCreate` |
//! | `0..10` / `0..=10` | `track_range()` | `Range` |
//! | `x as i64` | `track_type_cast()` | `TypeCast` |
//! | `\|\| expr` / `move \|\| expr` | `track_closure_create()` | `ClosureCreate` |
//!
//! ## Functions (`functions` group, disabled by default)
//!
//! | Code Pattern | Runtime Function | Event |
//! |--------------|------------------|-------|
//! | Function entry | `track_fn_enter()` | `FnEnter` |
//! | Function exit | `track_fn_exit()` | `FnExit` |
//!
//! # Configuration Examples
//!
//! ```ignore
//! // Track only ownership operations
//! #[trace_borrow(only = "ownership")]
//! fn minimal_tracking() { }
//!
//! // Track ownership + function boundaries
//! #[trace_borrow(only = "ownership,functions")]
//! fn with_fn_tracking() { }
//!
//! // Skip noisy loop and branch tracking
//! #[trace_borrow(skip = "loops,branches")]
//! fn cleaner_output() { }
//!
//! // Quiet mode - same as only = "ownership"
//! #[trace_borrow(quiet)]
//! fn quiet_mode() { }
//! ```
//!
//! # Limitations
//!
//! - Cannot be used on `const fn` (tracking requires runtime)
//! - Cannot be used on `extern` functions
//! - Async functions work but may not capture all ownership across await points
//! - Unsafe functions work but tracking cannot verify safety invariants

mod best_practices;
mod borrow_detection;
mod codegen;
mod config;
mod examples;
mod formatting;
mod generic_handler;
mod hygiene;
mod optimized_transform;
mod parser;
mod pattern;
mod smart_pointer;
mod span_utils;
mod transform_visitor;
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
            #cfg_tokens
            #transformed_fn

            #neg_cfg_tokens
            #input_fn
        }
    } else {
        // Always instrument
        quote! {
            #transformed_fn
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn transform_function(func: &mut ItemFn) {
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
