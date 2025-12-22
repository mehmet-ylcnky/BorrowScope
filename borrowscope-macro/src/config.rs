//! Configuration for the `#[trace_borrow]` attribute.
//!
//! This module provides [`TraceConfig`] for controlling which operations are tracked
//! and [`TraceArgs`] for parsing attribute arguments.
//!
//! # Feature Groups
//!
//! | Group | Config Field | Description |
//! |-------|--------------|-------------|
//! | `ownership` | `track_new`, `track_move`, `track_drop`, `track_borrow` | Basic ownership |
//! | `smart_pointers` | `track_smart_pointers` | Rc, Arc, RefCell, Cell |
//! | `loops` | `track_loops` | for, while, loop |
//! | `branches` | `track_branches` | if/else, match |
//! | `control_flow` | `track_control_flow` | break, continue, return |
//! | `try` | `track_try` | ? operator |
//! | `methods` | `track_methods` | clone, lock, unwrap |
//! | `async` | `track_async` | async blocks, await |
//! | `unsafe` | `track_unsafe` | unsafe blocks, pointers |
//! | `expressions` | `track_expressions` | struct, tuple, array, range, cast |
//! | `functions` | `track_functions` | Function entry/exit |
//!
//! # Usage
//!
//! ```ignore
//! #[trace_borrow]                           // standard (default)
//! #[trace_borrow(quiet)]                    // ownership only
//! #[trace_borrow(verbose)]                  // all features
//! #[trace_borrow(skip = "loops,branches")]  // skip specific groups
//! #[trace_borrow(only = "ownership")]       // only specific groups
//! ```

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitStr, Token,
};

/// Configuration for what operations to track.
///
/// Each field controls a category of tracking. All fields default to `true`
/// in standard mode except `track_functions` which defaults to `false`.
///
/// # Example
///
/// ```ignore
/// let config = TraceConfig::standard();
/// assert!(config.track_new);
/// assert!(config.track_borrow);
/// assert!(!config.track_functions); // disabled by default
/// ```
#[derive(Debug, Clone, Default)]
pub struct TraceConfig {
    /// Track variable creation via `let x = value;`
    ///
    /// Generates `New` events with variable name and type.
    pub track_new: bool,

    /// Track ownership moves via `let y = x;`
    ///
    /// Generates `Move` events showing source and destination.
    pub track_move: bool,

    /// Track variable drops at scope exit.
    ///
    /// Generates `Drop` events in LIFO order.
    pub track_drop: bool,

    /// Track borrows via `&x` and `&mut x`.
    ///
    /// Generates `Borrow` events with mutability info.
    pub track_borrow: bool,

    /// Track smart pointer operations (Rc, Arc, RefCell, Cell).
    ///
    /// Generates events like `RcNew`, `RcClone`, `RefCellBorrow`, etc.
    pub track_smart_pointers: bool,

    /// Track loop constructs (for, while, loop).
    ///
    /// Generates `LoopEnter`, `LoopIteration`, `LoopExit` events.
    pub track_loops: bool,

    /// Track branching (if/else, match).
    ///
    /// Generates `Branch`, `MatchEnter`, `MatchArm`, `MatchExit` events.
    pub track_branches: bool,

    /// Track control flow (break, continue, return).
    ///
    /// Generates `Break`, `Continue`, `Return` events.
    pub track_control_flow: bool,

    /// Track the `?` operator.
    ///
    /// Generates `Try` events at each `?` usage.
    pub track_try: bool,

    /// Track method calls (clone, lock, unwrap).
    ///
    /// Generates `Clone`, `Lock`, `Unwrap` events.
    pub track_methods: bool,

    /// Track async operations (async blocks, await).
    ///
    /// Generates `AsyncBlockEnter/Exit`, `AwaitStart/End` events.
    pub track_async: bool,

    /// Track unsafe blocks and operations.
    ///
    /// Generates `UnsafeBlockEnter/Exit`, `RawPtrCreated`, `Transmute` events.
    pub track_unsafe: bool,

    /// Track expression constructs (struct, tuple, array, range, cast).
    ///
    /// Generates `StructCreate`, `TupleCreate`, `ArrayCreate`, `Range`, `TypeCast` events.
    pub track_expressions: bool,

    /// Track function entry and exit points.
    ///
    /// Generates `FnEnter` and `FnExit` events. Disabled by default.
    pub track_functions: bool,
}

impl TraceConfig {
    /// Create standard configuration with all tracking enabled except functions.
    ///
    /// This is the default when using `#[trace_borrow]` without arguments.
    pub fn standard() -> Self {
        Self {
            track_new: true,
            track_move: true,
            track_drop: true,
            track_borrow: true,
            track_smart_pointers: true,
            track_loops: true,
            track_branches: true,
            track_control_flow: true,
            track_try: true,
            track_methods: true,
            track_async: true,
            track_unsafe: true,
            track_expressions: true,
            track_functions: false,
        }
    }

    /// Create quiet configuration with only ownership tracking.
    ///
    /// Used with `#[trace_borrow(quiet)]`. Tracks only:
    /// - Variable creation (new)
    /// - Moves
    /// - Drops
    /// - Borrows
    pub fn quiet() -> Self {
        Self {
            track_new: true,
            track_move: true,
            track_drop: true,
            track_borrow: true,
            track_smart_pointers: false,
            track_loops: false,
            track_branches: false,
            track_control_flow: false,
            track_try: false,
            track_methods: false,
            track_async: false,
            track_unsafe: false,
            track_expressions: false,
            track_functions: false,
        }
    }

    /// Create verbose configuration with all tracking enabled.
    ///
    /// Used with `#[trace_borrow(verbose)]`. Same as standard for now.
    pub fn verbose() -> Self {
        Self::standard()
    }

    /// Disable specific feature groups.
    ///
    /// Used with `#[trace_borrow(skip = "loops,branches")]`.
    ///
    /// # Supported Groups
    ///
    /// - `loops` - Disable loop tracking
    /// - `branches` - Disable if/else and match tracking
    /// - `control_flow` or `control` - Disable break/continue/return
    /// - `try` - Disable ? operator tracking
    /// - `methods` - Disable clone/lock/unwrap tracking
    /// - `async` - Disable async block and await tracking
    /// - `unsafe` - Disable unsafe block tracking
    /// - `expressions` or `exprs` - Disable struct/tuple/array/range/cast
    /// - `smart_pointers` or `pointers` - Disable Rc/Arc/RefCell/Cell
    /// - `functions` or `fn` - Disable function entry/exit
    pub fn skip(&mut self, features: &str) {
        for feature in features.split(',').map(|s| s.trim()) {
            match feature {
                "loops" => self.track_loops = false,
                "branches" => self.track_branches = false,
                "control_flow" | "control" => self.track_control_flow = false,
                "try" => self.track_try = false,
                "methods" => self.track_methods = false,
                "async" => self.track_async = false,
                "unsafe" => self.track_unsafe = false,
                "expressions" | "exprs" => self.track_expressions = false,
                "smart_pointers" | "pointers" => self.track_smart_pointers = false,
                "functions" | "fn" => self.track_functions = false,
                "drop" | "drops" => self.track_drop = false,
                _ => {} // ignore unknown
            }
        }
    }

    /// Enable only specific feature groups, disabling all others.
    ///
    /// Used with `#[trace_borrow(only = "ownership")]`.
    ///
    /// # Supported Groups
    ///
    /// - `ownership` - Enable new, move, drop, borrow
    /// - `new` - Enable only variable creation
    /// - `move` or `moves` - Enable only move tracking
    /// - `drop` or `drops` - Enable only drop tracking
    /// - `borrow` or `borrows` - Enable only borrow tracking
    /// - `loops` - Enable loop tracking
    /// - `branches` - Enable if/else and match tracking
    /// - `control_flow` or `control` - Enable break/continue/return
    /// - `try` - Enable ? operator tracking
    /// - `methods` - Enable clone/lock/unwrap tracking
    /// - `async` - Enable async block and await tracking
    /// - `unsafe` - Enable unsafe block tracking
    /// - `expressions` or `exprs` - Enable struct/tuple/array/range/cast
    /// - `smart_pointers` or `pointers` - Enable Rc/Arc/RefCell/Cell
    /// - `functions` or `fn` - Enable function entry/exit
    pub fn only(&mut self, features: &str) {
        // Start with nothing
        *self = Self {
            track_new: false,
            track_move: false,
            track_drop: false,
            track_borrow: false,
            track_smart_pointers: false,
            track_loops: false,
            track_branches: false,
            track_control_flow: false,
            track_try: false,
            track_methods: false,
            track_async: false,
            track_unsafe: false,
            track_expressions: false,
            track_functions: false,
        };

        for feature in features.split(',').map(|s| s.trim()) {
            match feature {
                "ownership" => {
                    self.track_new = true;
                    self.track_move = true;
                    self.track_drop = true;
                    self.track_borrow = true;
                }
                "new" => self.track_new = true,
                "move" | "moves" => self.track_move = true,
                "drop" | "drops" => self.track_drop = true,
                "borrow" | "borrows" => self.track_borrow = true,
                "loops" => self.track_loops = true,
                "branches" => self.track_branches = true,
                "control_flow" | "control" => self.track_control_flow = true,
                "try" => self.track_try = true,
                "methods" => self.track_methods = true,
                "async" => self.track_async = true,
                "unsafe" => self.track_unsafe = true,
                "expressions" | "exprs" => self.track_expressions = true,
                "smart_pointers" | "pointers" => self.track_smart_pointers = true,
                "functions" | "fn" => self.track_functions = true,
                _ => {} // ignore unknown
            }
        }
    }
}

/// Parsed attribute arguments for `#[trace_borrow(...)]`.
///
/// This struct is used internally to parse the attribute arguments.
pub struct TraceArgs {
    /// The resulting configuration after parsing arguments.
    pub config: TraceConfig,
}

impl Default for TraceArgs {
    fn default() -> Self {
        Self {
            config: TraceConfig::standard(),
        }
    }
}

impl Parse for TraceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let mut config = TraceConfig::standard();

        // Parse comma-separated arguments
        let args: Punctuated<TraceArg, Token![,]> = Punctuated::parse_terminated(input)?;

        for arg in args {
            match arg {
                TraceArg::Verbose => config = TraceConfig::verbose(),
                TraceArg::Quiet => config = TraceConfig::quiet(),
                TraceArg::Skip(features) => config.skip(&features),
                TraceArg::Only(features) => config.only(&features),
            }
        }

        Ok(Self { config })
    }
}

enum TraceArg {
    Verbose,
    Quiet,
    Skip(String),
    Only(String),
}

impl Parse for TraceArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let name = ident.to_string();

        match name.as_str() {
            "verbose" => Ok(TraceArg::Verbose),
            "quiet" => Ok(TraceArg::Quiet),
            "skip" => {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                Ok(TraceArg::Skip(value.value()))
            }
            "only" => {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                Ok(TraceArg::Only(value.value()))
            }
            _ => Err(syn::Error::new(
                ident.span(),
                format!("unknown attribute argument: {}", name),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_config() {
        let config = TraceConfig::standard();
        assert!(config.track_new);
        assert!(config.track_loops);
        assert!(config.track_branches);
    }

    #[test]
    fn test_quiet_config() {
        let config = TraceConfig::quiet();
        assert!(config.track_new);
        assert!(!config.track_loops);
        assert!(!config.track_branches);
    }

    #[test]
    fn test_skip() {
        let mut config = TraceConfig::standard();
        config.skip("loops, branches");
        assert!(!config.track_loops);
        assert!(!config.track_branches);
        assert!(config.track_new);
    }

    #[test]
    fn test_only_ownership() {
        let mut config = TraceConfig::standard();
        config.only("ownership");
        assert!(config.track_new);
        assert!(config.track_move);
        assert!(config.track_drop);
        assert!(config.track_borrow);
        assert!(!config.track_loops);
        assert!(!config.track_branches);
    }
}
