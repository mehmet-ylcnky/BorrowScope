//! Configuration for trace_borrow attribute
//!
//! Supports various tracking modes and feature toggles.

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitStr, Token,
};

/// Configuration for what to track
#[derive(Debug, Clone, Default)]
pub struct TraceConfig {
    /// Track variable creation (new)
    pub track_new: bool,
    /// Track moves
    pub track_move: bool,
    /// Track drops
    pub track_drop: bool,
    /// Track borrows
    pub track_borrow: bool,
    /// Track smart pointers (Rc, Arc, RefCell, Cell)
    pub track_smart_pointers: bool,
    /// Track loops (for, while, loop)
    pub track_loops: bool,
    /// Track branches (if/else, match)
    pub track_branches: bool,
    /// Track control flow (break, continue, return)
    pub track_control_flow: bool,
    /// Track try/? operator
    pub track_try: bool,
    /// Track method calls (clone, lock, unwrap)
    pub track_methods: bool,
    /// Track async (async blocks, await)
    pub track_async: bool,
    /// Track unsafe blocks
    pub track_unsafe: bool,
    /// Track expressions (struct, tuple, array, range, cast)
    pub track_expressions: bool,
}

impl TraceConfig {
    /// Default configuration - standard tracking
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
        }
    }

    /// Quiet mode - ownership only
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
        }
    }

    /// Verbose mode - everything enabled
    pub fn verbose() -> Self {
        Self::standard()
    }

    /// Apply skip list
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
                _ => {} // ignore unknown
            }
        }
    }

    /// Apply only list - disable everything except specified
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
                _ => {} // ignore unknown
            }
        }
    }
}

/// Parsed attribute arguments
pub struct TraceArgs {
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
