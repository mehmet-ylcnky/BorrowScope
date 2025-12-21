//! Best practices for macro development

#![allow(dead_code)]

use proc_macro2::Span;
use syn::{Error, Ident, ItemFn};

/// Validate function before transformation
pub fn validate_before_transform(func: &ItemFn) -> syn::Result<()> {
    if func.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            func.sig.asyncness,
            "trace_borrow does not support async functions yet",
        ));
    }

    if func.sig.unsafety.is_some() {
        return Err(Error::new_spanned(
            func.sig.unsafety,
            "trace_borrow cannot be used on unsafe functions",
        ));
    }

    if func.sig.constness.is_some() {
        return Err(Error::new_spanned(
            func.sig.constness,
            "trace_borrow cannot be used on const functions",
        ));
    }

    Ok(())
}

/// Check if transformation is safe
pub fn is_safe_to_transform(func: &ItemFn) -> bool {
    func.sig.asyncness.is_none() && func.sig.unsafety.is_none() && func.sig.constness.is_none()
}

/// Generate unique temporary name
pub fn generate_unique_name(prefix: &str, counter: usize) -> Ident {
    let name = format!("__borrowscope_{}_{}", prefix, counter);
    Ident::new(&name, Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_validate_simple_function() {
        let func: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(validate_before_transform(&func).is_ok());
    }

    #[test]
    fn test_validate_async_function() {
        let func: ItemFn = parse_quote! {
            async fn example() {}
        };
        assert!(validate_before_transform(&func).is_err());
    }

    #[test]
    fn test_validate_unsafe_function() {
        let func: ItemFn = parse_quote! {
            unsafe fn example() {}
        };
        assert!(validate_before_transform(&func).is_err());
    }

    #[test]
    fn test_validate_const_function() {
        let func: ItemFn = parse_quote! {
            const fn example() {}
        };
        assert!(validate_before_transform(&func).is_err());
    }

    #[test]
    fn test_is_safe_to_transform_simple() {
        let func: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(is_safe_to_transform(&func));
    }

    #[test]
    fn test_is_safe_to_transform_async() {
        let func: ItemFn = parse_quote! {
            async fn example() {}
        };
        assert!(!is_safe_to_transform(&func));
    }

    #[test]
    fn test_is_safe_to_transform_unsafe() {
        let func: ItemFn = parse_quote! {
            unsafe fn example() {}
        };
        assert!(!is_safe_to_transform(&func));
    }

    #[test]
    fn test_is_safe_to_transform_const() {
        let func: ItemFn = parse_quote! {
            const fn example() {}
        };
        assert!(!is_safe_to_transform(&func));
    }

    #[test]
    fn test_generate_unique_name() {
        let name1 = generate_unique_name("temp", 0);
        let name2 = generate_unique_name("temp", 1);

        assert_eq!(name1.to_string(), "__borrowscope_temp_0");
        assert_eq!(name2.to_string(), "__borrowscope_temp_1");
    }

    #[test]
    fn test_generate_unique_name_different_prefix() {
        let name1 = generate_unique_name("var", 0);
        let name2 = generate_unique_name("tmp", 0);

        assert_eq!(name1.to_string(), "__borrowscope_var_0");
        assert_eq!(name2.to_string(), "__borrowscope_tmp_0");
    }

    #[test]
    fn test_validate_function_with_generics() {
        let func: ItemFn = parse_quote! {
            fn example<T>() {}
        };
        assert!(validate_before_transform(&func).is_ok());
    }

    #[test]
    fn test_validate_function_with_lifetimes() {
        let func: ItemFn = parse_quote! {
            fn example<'a>() {}
        };
        assert!(validate_before_transform(&func).is_ok());
    }
}
