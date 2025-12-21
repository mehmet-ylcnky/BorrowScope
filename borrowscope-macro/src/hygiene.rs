//! Macro hygiene utilities

#![allow(dead_code)]

use proc_macro2::Span;
use syn::Ident;

/// Create a hygienic identifier that won't conflict with user code
pub fn create_hygienic_ident(base_name: &str) -> Ident {
    let name = format!("__borrowscope_{}", base_name);
    Ident::new(&name, Span::call_site())
}

/// Check if an identifier is a BorrowScope internal identifier
pub fn is_internal_ident(ident: &Ident) -> bool {
    ident.to_string().starts_with("__borrowscope_")
}

/// Create a temporary variable name that won't conflict
pub fn create_temp_var(index: usize) -> Ident {
    let name = format!("__borrowscope_temp_{}", index);
    Ident::new(&name, Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_create_hygienic_ident() {
        let ident = create_hygienic_ident("tracker");
        assert_eq!(ident.to_string(), "__borrowscope_tracker");
    }

    #[test]
    fn test_create_hygienic_ident_different_names() {
        let ident1 = create_hygienic_ident("foo");
        let ident2 = create_hygienic_ident("bar");
        assert_eq!(ident1.to_string(), "__borrowscope_foo");
        assert_eq!(ident2.to_string(), "__borrowscope_bar");
    }

    #[test]
    fn test_is_internal_ident_true() {
        let ident: Ident = parse_quote!(__borrowscope_temp);
        assert!(is_internal_ident(&ident));
    }

    #[test]
    fn test_is_internal_ident_false() {
        let ident: Ident = parse_quote!(user_variable);
        assert!(!is_internal_ident(&ident));
    }

    #[test]
    fn test_create_temp_var() {
        let temp1 = create_temp_var(0);
        let temp2 = create_temp_var(1);
        assert_eq!(temp1.to_string(), "__borrowscope_temp_0");
        assert_eq!(temp2.to_string(), "__borrowscope_temp_1");
    }

    #[test]
    fn test_temp_vars_unique() {
        let temp1 = create_temp_var(0);
        let temp2 = create_temp_var(1);
        assert_ne!(temp1.to_string(), temp2.to_string());
    }
}
