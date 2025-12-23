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
    let message = format!(
        "{} Hint: {}",
        pattern.message(),
        pattern.hint(name)
    );

    // Generate unique warning name
    let counter = WARNING_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let warning_name = format!("BorrowScope{}Warning{}", pattern.code().replace("::", "_"), counter);

    let warning = FormattedWarning::new_deprecated(&warning_name, &message, span);

    quote::quote!(#warning)
}

/// Check if a function name looks like it could be FFI
/// Returns true for names that follow C naming conventions
pub fn looks_like_ffi(name: &str) -> bool {
    // Common FFI prefixes
    let ffi_prefixes = [
        "c_", "C_", "ffi_", "FFI_", "sys_", "libc_", "extern_", "native_",
    ];

    // Common FFI patterns
    let ffi_patterns = [
        "malloc", "free", "realloc", "calloc", "memcpy", "memset", "memmove", "strlen", "strcpy",
        "strcmp", "printf", "sprintf", "fprintf", "scanf", "fopen", "fclose", "fread", "fwrite",
        "pthread_", "socket", "bind", "listen", "accept", "connect", "send", "recv", "read",
        "write", "open", "close", "ioctl", "fcntl", "fork", "exec", "wait", "kill", "signal",
    ];

    // Check prefixes
    if ffi_prefixes.iter().any(|p| name.starts_with(p)) {
        return true;
    }

    // Check known patterns
    if ffi_patterns.iter().any(|p| name.starts_with(p)) {
        return true;
    }

    // Check for all-caps with underscores (common C macro style)
    if name.len() > 2 && name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric()) {
        return true;
    }

    false
}

/// Check if a name looks like a static variable
/// Returns true for SCREAMING_SNAKE_CASE names
pub fn looks_like_static(name: &str) -> bool {
    if name.len() < 2 {
        return false;
    }

    // SCREAMING_SNAKE_CASE pattern
    let has_uppercase = name.chars().any(|c| c.is_uppercase());
    let no_lowercase = !name.chars().any(|c| c.is_lowercase());
    let valid_chars = name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric());

    has_uppercase && no_lowercase && valid_chars
}

/// Check if a type name could be a union
/// This is heuristic-based since we can't know for sure
pub fn looks_like_union(name: &str) -> bool {
    // Common union naming patterns
    let union_hints = [
        "Union",
        "union",
        "Raw",
        "Packed",
        "Repr",
        "Bits",
        "Data",
        "Value",
    ];

    union_hints.iter().any(|h| name.contains(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_ffi() {
        assert!(looks_like_ffi("c_read"));
        assert!(looks_like_ffi("ffi_call"));
        assert!(looks_like_ffi("malloc"));
        assert!(looks_like_ffi("pthread_create"));
        assert!(looks_like_ffi("SOME_C_MACRO"));

        assert!(!looks_like_ffi("my_function"));
        assert!(!looks_like_ffi("process_data"));
        assert!(!looks_like_ffi("Vec"));
    }

    #[test]
    fn test_looks_like_static() {
        assert!(looks_like_static("GLOBAL_CONFIG"));
        assert!(looks_like_static("MAX_SIZE"));
        assert!(looks_like_static("HTTP_200"));

        assert!(!looks_like_static("globalConfig"));
        assert!(!looks_like_static("max_size"));
        assert!(!looks_like_static("X")); // Too short
    }

    #[test]
    fn test_looks_like_union() {
        assert!(looks_like_union("DataUnion"));
        assert!(looks_like_union("RawValue"));
        assert!(looks_like_union("PackedData"));

        assert!(!looks_like_union("MyStruct"));
        assert!(!looks_like_union("Config"));
    }

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
        assert_eq!(AmbiguousPattern::PossibleStatic.code(), "borrowscope::static");
        assert_eq!(AmbiguousPattern::PossibleUnion.code(), "borrowscope::union");
        assert_eq!(AmbiguousPattern::Transmute.code(), "borrowscope::transmute");
    }
}
