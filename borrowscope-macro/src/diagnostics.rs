//! Compile-time diagnostics for ambiguous patterns
//!
//! This module provides warnings and hints for patterns that cannot be
//! fully analyzed at macro expansion time due to the type information barrier.

use proc_macro2::Span;
use proc_macro_warning::FormattedWarning;

/// Categories of ambiguous patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbiguousPattern {
    /// Function call that might be FFI
    PossibleFfi,
    /// Field access that might be union
    PossibleUnion,
    /// Identifier that might be static
    PossibleStatic,
    /// Clone call that might be Rc/Arc::clone
    PossibleSmartClone,
    /// Raw pointer operation
    RawPointer,
    /// Transmute call
    Transmute,
}

impl AmbiguousPattern {
    /// Get the warning message for this pattern
    pub fn message(&self) -> &'static str {
        match self {
            AmbiguousPattern::PossibleFfi => "cannot determine if this is an FFI call",
            AmbiguousPattern::PossibleUnion => "cannot determine if this is a union field access",
            AmbiguousPattern::PossibleStatic => "cannot determine if this is a static variable",
            AmbiguousPattern::PossibleSmartClone => {
                "cannot determine if this is Rc::clone or Arc::clone"
            }
            AmbiguousPattern::RawPointer => "raw pointer operations require manual verification",
            AmbiguousPattern::Transmute => "transmute type information unavailable at macro time",
        }
    }

    /// Get the hint for resolving this ambiguity
    pub fn hint(&self, name: &str) -> String {
        match self {
            AmbiguousPattern::PossibleFfi => {
                format!("use #[trace_borrow(ffi = [\"{}\"])] to track as FFI", name)
            }
            AmbiguousPattern::PossibleUnion => {
                format!(
                    "use #[trace_borrow(unions = [\"{}\"])] to track union access",
                    name
                )
            }
            AmbiguousPattern::PossibleStatic => {
                format!(
                    "use #[trace_borrow(statics = [\"{}\"])] to track static access",
                    name
                )
            }
            AmbiguousPattern::PossibleSmartClone => {
                "Rc::clone and Arc::clone are detected by method syntax; \
                 for turbofish syntax, tracking may be incomplete"
                    .to_string()
            }
            AmbiguousPattern::RawPointer => {
                "ensure unsafe blocks are properly annotated for complete tracking".to_string()
            }
            AmbiguousPattern::Transmute => {
                "transmute source and target types cannot be determined at macro expansion time"
                    .to_string()
            }
        }
    }

    /// Get the note explaining why this is ambiguous
    pub fn note(&self) -> &'static str {
        match self {
            AmbiguousPattern::PossibleFfi => {
                "proc macros cannot access type information to distinguish FFI from Rust functions"
            }
            AmbiguousPattern::PossibleUnion => {
                "proc macros cannot determine if a type is a union or struct"
            }
            AmbiguousPattern::PossibleStatic => {
                "proc macros cannot distinguish static variables from local bindings"
            }
            AmbiguousPattern::PossibleSmartClone => {
                "Clone::clone() syntax doesn't reveal if the type is Rc or Arc"
            }
            AmbiguousPattern::RawPointer => {
                "raw pointer safety cannot be verified at macro expansion time"
            }
            AmbiguousPattern::Transmute => {
                "generic type parameters are not resolved during macro expansion"
            }
        }
    }

    /// Get a short code for the warning type
    pub fn code(&self) -> &'static str {
        match self {
            AmbiguousPattern::PossibleFfi => "borrowscope::ffi",
            AmbiguousPattern::PossibleUnion => "borrowscope::union",
            AmbiguousPattern::PossibleStatic => "borrowscope::static",
            AmbiguousPattern::PossibleSmartClone => "borrowscope::clone",
            AmbiguousPattern::RawPointer => "borrowscope::rawptr",
            AmbiguousPattern::Transmute => "borrowscope::transmute",
        }
    }
}

/// Diagnostic configuration
#[derive(Debug, Clone, Default)]
pub struct DiagnosticConfig {
    /// Emit warnings for ambiguous patterns
    pub warn_ambiguous: bool,
    /// List of known FFI functions (won't warn)
    pub known_ffi: Vec<String>,
    /// List of known union types (won't warn)
    pub known_unions: Vec<String>,
    /// List of known static variables (won't warn)
    pub known_statics: Vec<String>,
    /// Suppress all warnings
    pub suppress_warnings: bool,
}

impl DiagnosticConfig {
    /// Create config with warnings enabled
    pub fn with_warnings() -> Self {
        Self {
            warn_ambiguous: true,
            ..Default::default()
        }
    }

    /// Check if a function name is known FFI
    pub fn is_known_ffi(&self, name: &str) -> bool {
        self.known_ffi.iter().any(|f| f == name)
    }

    /// Check if a type name is known union
    pub fn is_known_union(&self, name: &str) -> bool {
        self.known_unions.iter().any(|u| u == name)
    }

    /// Check if a variable name is known static
    pub fn is_known_static(&self, name: &str) -> bool {
        self.known_statics.iter().any(|s| s == name)
    }
}

/// Counter for generating unique warning names
static WARNING_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Create a warning for an ambiguous pattern.
/// Returns a TokenStream that should be included in the generated code.
pub fn create_ambiguous_warning(
    pattern: AmbiguousPattern,
    name: &str,
    span: Span,
) -> proc_macro2::TokenStream {
    let message = format!("{} Hint: {}", pattern.message(), pattern.hint(name));

    // Generate unique warning name
    let counter = WARNING_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let warning_name = format!(
        "BorrowScope{}Warning{}",
        pattern.code().replace("::", "_"),
        counter
    );

    let warning = FormattedWarning::new_deprecated(&warning_name, &message, span);

    quote::quote!(#warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambiguous_pattern_messages() {
        let pattern = AmbiguousPattern::PossibleFfi;
        assert!(pattern.message().contains("FFI"));
        assert!(pattern.hint("my_func").contains("ffi"));
        assert!(pattern.note().contains("type information"));
    }

    #[test]
    fn test_diagnostic_config() {
        let mut config = DiagnosticConfig::with_warnings();
        config.known_ffi.push("c_read".to_string());
        config.known_statics.push("GLOBAL".to_string());

        assert!(config.is_known_ffi("c_read"));
        assert!(!config.is_known_ffi("c_write"));
        assert!(config.is_known_static("GLOBAL"));
    }

    #[test]
    fn test_create_ambiguous_warning() {
        let warning = create_ambiguous_warning(
            AmbiguousPattern::PossibleFfi,
            "c_read",
            proc_macro2::Span::call_site(),
        );
        let warning_str = warning.to_string();
        // The warning should generate a deprecated constant
        assert!(warning_str.contains("deprecated"));
    }

    #[test]
    fn test_pattern_codes() {
        assert_eq!(AmbiguousPattern::PossibleFfi.code(), "borrowscope::ffi");
        assert_eq!(
            AmbiguousPattern::PossibleStatic.code(),
            "borrowscope::static"
        );
        assert_eq!(AmbiguousPattern::PossibleUnion.code(), "borrowscope::union");
        assert_eq!(AmbiguousPattern::Transmute.code(), "borrowscope::transmute");
    }
}
