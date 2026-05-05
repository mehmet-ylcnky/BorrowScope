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
//! # Conditional Compilation
//!
//! | Option | Description |
//! |--------|-------------|
//! | `debug_only` | Only instrument in debug builds (`#[cfg(debug_assertions)]`) |
//! | `release_only` | Only instrument in release builds (`#[cfg(not(debug_assertions))]`) |
//! | `feature = "name"` | Only instrument when cargo feature is enabled |
//!
//! # Usage
//!
//! ```ignore
//! #[trace_borrow]                           // standard (default)
//! #[trace_borrow(quiet)]                    // ownership only
//! #[trace_borrow(verbose)]                  // all features
//! #[trace_borrow(skip = "loops,branches")]  // skip specific groups
//! #[trace_borrow(only = "ownership")]       // only specific groups
//! #[trace_borrow(debug_only)]               // only in debug builds
//! #[trace_borrow(feature = "tracing")]      // only when feature enabled
//! ```

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitStr, Token,
};

/// Conditional compilation mode for tracking code.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConditionalMode {
    /// Always generate tracking code (default)
    #[default]
    Always,
    /// Only generate tracking code in debug builds
    DebugOnly,
    /// Only generate tracking code in release builds
    ReleaseOnly,
    /// Only generate tracking code when a specific feature is enabled
    Feature(String),
}

impl ConditionalMode {
    /// Generate the cfg attribute tokens for this mode.
    /// Returns None for Always mode (no cfg needed).
    pub fn cfg_tokens(&self) -> Option<proc_macro2::TokenStream> {
        use quote::quote;
        match self {
            ConditionalMode::Always => None,
            ConditionalMode::DebugOnly => Some(quote! { #[cfg(debug_assertions)] }),
            ConditionalMode::ReleaseOnly => Some(quote! { #[cfg(not(debug_assertions))] }),
            ConditionalMode::Feature(name) => {
                let feature_name = syn::LitStr::new(name, proc_macro2::Span::call_site());
                Some(quote! { #[cfg(feature = #feature_name)] })
            }
        }
    }

    /// Check if this mode requires conditional compilation.
    pub fn is_conditional(&self) -> bool {
        !matches!(self, ConditionalMode::Always)
    }
}

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

    /// Conditional compilation mode.
    ///
    /// Controls when tracking code is compiled:
    /// - `Always` (default): Always compile tracking code
    /// - `DebugOnly`: Only compile in debug builds
    /// - `ReleaseOnly`: Only compile in release builds
    /// - `Feature(name)`: Only compile when cargo feature is enabled
    pub conditional_mode: ConditionalMode,

    /// Emit warnings for ambiguous patterns.
    ///
    /// When enabled, the macro will emit compile-time warnings for patterns
    /// that cannot be fully analyzed due to the type information barrier.
    pub warn_ambiguous: bool,

    /// Known FFI function names (won't warn about these).
    pub known_ffi: Vec<String>,

    /// Known union type names (won't warn about these).
    pub known_unions: Vec<String>,

    /// Known static variable names (won't warn about these).
    pub known_statics: Vec<String>,

    /// Filter pattern for variable names.
    ///
    /// Only track variables matching this pattern. Supports:
    /// - `name:prefix*` - Variables starting with prefix
    /// - `name:*suffix` - Variables ending with suffix
    /// - `name:*contains*` - Variables containing substring
    /// - `name:exact` - Exact match
    pub filter_pattern: Option<String>,

    /// Sample rate for tracking (0.0 to 1.0).
    ///
    /// When set, only a random percentage of tracking calls are executed.
    /// - `1.0` (default): Track all calls
    /// - `0.5`: Track ~50% of calls
    /// - `0.01`: Track ~1% of calls
    pub sample_rate: Option<f64>,
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
            conditional_mode: ConditionalMode::Always,
            warn_ambiguous: false,
            known_ffi: Vec::new(),
            known_unions: Vec::new(),
            known_statics: Vec::new(),
            filter_pattern: None,
            sample_rate: None,
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
            conditional_mode: ConditionalMode::Always,
            warn_ambiguous: false,
            known_ffi: Vec::new(),
            known_unions: Vec::new(),
            known_statics: Vec::new(),
            filter_pattern: None,
            sample_rate: None,
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
        // Preserve settings that shouldn't be reset
        let mode = self.conditional_mode.clone();
        let warn = self.warn_ambiguous;
        let ffi = std::mem::take(&mut self.known_ffi);
        let unions = std::mem::take(&mut self.known_unions);
        let statics = std::mem::take(&mut self.known_statics);
        let filter = self.filter_pattern.take();
        let sample = self.sample_rate.take();

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
            conditional_mode: mode,
            warn_ambiguous: warn,
            known_ffi: ffi,
            known_unions: unions,
            known_statics: statics,
            filter_pattern: filter,
            sample_rate: sample,
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
                TraceArg::DebugOnly => config.conditional_mode = ConditionalMode::DebugOnly,
                TraceArg::ReleaseOnly => config.conditional_mode = ConditionalMode::ReleaseOnly,
                TraceArg::Feature(name) => config.conditional_mode = ConditionalMode::Feature(name),
                TraceArg::WarnAmbiguous => config.warn_ambiguous = true,
                TraceArg::Ffi(names) => config.known_ffi.extend(names),
                TraceArg::Unions(names) => config.known_unions.extend(names),
                TraceArg::Statics(names) => config.known_statics.extend(names),
                TraceArg::Filter(pattern) => config.filter_pattern = Some(pattern),
                TraceArg::Sample(rate) => config.sample_rate = Some(rate),
            }
        }

        Ok(Self { config })
    }
}

#[derive(Debug)]
enum TraceArg {
    Verbose,
    Quiet,
    Skip(String),
    Only(String),
    DebugOnly,
    ReleaseOnly,
    Feature(String),
    WarnAmbiguous,
    Ffi(Vec<String>),
    Unions(Vec<String>),
    Statics(Vec<String>),
    Filter(String),
    Sample(f64),
}

impl Parse for TraceArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let name = ident.to_string();

        match name.as_str() {
            "verbose" => Ok(TraceArg::Verbose),
            "quiet" => Ok(TraceArg::Quiet),
            "debug_only" => Ok(TraceArg::DebugOnly),
            "release_only" => Ok(TraceArg::ReleaseOnly),
            "skip" => {
                // Support both skip = "..." and skip(...)
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    Ok(TraceArg::Skip(value.value()))
                } else {
                    let content;
                    syn::parenthesized!(content in input);
                    let items: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
                    let value = items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                    Ok(TraceArg::Skip(value))
                }
            }
            "only" => {
                // Support both only = "..." and only(...)
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    Ok(TraceArg::Only(value.value()))
                } else {
                    let content;
                    syn::parenthesized!(content in input);
                    let items: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
                    let value = items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                    Ok(TraceArg::Only(value))
                }
            }
            "feature" => {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                Ok(TraceArg::Feature(value.value()))
            }
            "warn" | "warn_ambiguous" => Ok(TraceArg::WarnAmbiguous),
            "ffi" => {
                input.parse::<Token![=]>()?;
                let names = parse_string_list(input)?;
                Ok(TraceArg::Ffi(names))
            }
            "unions" => {
                input.parse::<Token![=]>()?;
                let names = parse_string_list(input)?;
                Ok(TraceArg::Unions(names))
            }
            "statics" => {
                input.parse::<Token![=]>()?;
                let names = parse_string_list(input)?;
                Ok(TraceArg::Statics(names))
            }
            "filter" => {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                let pattern = value.value();

                // Validate filter pattern
                if pattern.is_empty() {
                    return Err(syn::Error::new(
                        value.span(),
                        "filter pattern cannot be empty"
                    ));
                }

                // Check for invalid characters (only alphanumeric, _, *, ? allowed)
                for ch in pattern.chars() {
                    if !ch.is_alphanumeric() && ch != '_' && ch != '*' && ch != '?' {
                        return Err(syn::Error::new(
                            value.span(),
                            format!(
                                "invalid character '{}' in filter pattern. \
                                 Only alphanumeric, '_', '*', and '?' are allowed",
                                ch
                            )
                        ));
                    }
                }

                Ok(TraceArg::Filter(pattern))
            }
            "sample" => {
                input.parse::<Token![=]>()?;
                let value: syn::LitFloat = input.parse()?;
                let rate: f64 = value.base10_parse()?;

                // Validate sample rate range
                if rate < 0.0 || rate > 1.0 {
                    return Err(syn::Error::new(
                        value.span(),
                        format!(
                            "sample rate must be between 0.0 and 1.0, got {}",
                            rate
                        )
                    ));
                }

                Ok(TraceArg::Sample(rate))
            }
            _ => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unknown attribute argument: {}. Expected one of: verbose, quiet, \
                     debug_only, release_only, skip, only, feature, warn, ffi, unions, statics, filter, sample",
                    name
                ),
            )),
        }
    }
}

/// Parse a list of strings in bracket syntax: ["a", "b", "c"]
fn parse_string_list(input: ParseStream) -> syn::Result<Vec<String>> {
    let content;
    syn::bracketed!(content in input);
    let items: Punctuated<LitStr, Token![,]> = Punctuated::parse_terminated(&content)?;
    Ok(items.iter().map(|s| s.value()).collect())
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
        assert_eq!(config.conditional_mode, ConditionalMode::Always);
    }

    #[test]
    fn test_quiet_config() {
        let config = TraceConfig::quiet();
        assert!(config.track_new);
        assert!(!config.track_loops);
        assert!(!config.track_branches);
        assert_eq!(config.conditional_mode, ConditionalMode::Always);
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

    #[test]
    fn test_conditional_mode_debug_only() {
        let mut config = TraceConfig::standard();
        config.conditional_mode = ConditionalMode::DebugOnly;
        assert!(config.conditional_mode.is_conditional());
        assert!(config.conditional_mode.cfg_tokens().is_some());
    }

    #[test]
    fn test_conditional_mode_release_only() {
        let mut config = TraceConfig::standard();
        config.conditional_mode = ConditionalMode::ReleaseOnly;
        assert!(config.conditional_mode.is_conditional());
        assert!(config.conditional_mode.cfg_tokens().is_some());
    }

    #[test]
    fn test_conditional_mode_feature() {
        let mut config = TraceConfig::standard();
        config.conditional_mode = ConditionalMode::Feature("tracing".to_string());
        assert!(config.conditional_mode.is_conditional());
        assert!(config.conditional_mode.cfg_tokens().is_some());
    }

    #[test]
    fn test_conditional_mode_always() {
        let config = TraceConfig::standard();
        assert!(!config.conditional_mode.is_conditional());
        assert!(config.conditional_mode.cfg_tokens().is_none());
    }

    #[test]
    fn test_only_preserves_conditional_mode() {
        let mut config = TraceConfig::standard();
        config.conditional_mode = ConditionalMode::DebugOnly;
        config.only("ownership");
        assert_eq!(config.conditional_mode, ConditionalMode::DebugOnly);
    }

    #[test]
    fn test_filter_pattern() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("data*".to_string());
        assert_eq!(config.filter_pattern, Some("data*".to_string()));
    }

    #[test]
    fn test_sample_rate() {
        let mut config = TraceConfig::standard();
        config.sample_rate = Some(0.1);
        assert_eq!(config.sample_rate, Some(0.1));
    }

    #[test]
    fn test_only_preserves_filter_and_sample() {
        let mut config = TraceConfig::standard();
        config.filter_pattern = Some("user*".to_string());
        config.sample_rate = Some(0.5);
        config.only("ownership");
        assert_eq!(config.filter_pattern, Some("user*".to_string()));
        assert_eq!(config.sample_rate, Some(0.5));
    }

    #[test]
    fn test_filter_pattern_validation_valid() {
        // Valid patterns should parse
        let valid = ["data*", "*_count", "user_?", "abc123", "a*b?c"];
        for pattern in valid {
            let input = format!("filter = \"{}\"", pattern);
            let result: syn::Result<TraceArg> = syn::parse_str(&input);
            assert!(result.is_ok(), "Pattern '{}' should be valid", pattern);
        }
    }

    #[test]
    fn test_filter_pattern_validation_empty() {
        let result: syn::Result<TraceArg> = syn::parse_str("filter = \"\"");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_filter_pattern_validation_invalid_chars() {
        let invalid = ["data-name", "user.name", "path/to", "a@b", "x#y"];
        for pattern in invalid {
            let input = format!("filter = \"{}\"", pattern);
            let result: syn::Result<TraceArg> = syn::parse_str(&input);
            assert!(result.is_err(), "Pattern '{}' should be invalid", pattern);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("invalid character"));
        }
    }

    #[test]
    fn test_sample_rate_validation_valid() {
        // Note: syn requires explicit float syntax (with decimal point)
        let valid = ["0.0", "0.1", "0.5", "1.0"];
        for rate in valid {
            let input = format!("sample = {}", rate);
            let result: syn::Result<TraceArg> = syn::parse_str(&input);
            assert!(result.is_ok(), "Rate {} should be valid", rate);
        }
    }

    #[test]
    fn test_sample_rate_validation_out_of_range() {
        let invalid = ["-0.1", "1.1", "2.0", "-1.0"];
        for rate in invalid {
            let input = format!("sample = {}", rate);
            let result: syn::Result<TraceArg> = syn::parse_str(&input);
            assert!(result.is_err(), "Rate {} should be invalid", rate);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("between 0.0 and 1.0"));
        }
    }
}
