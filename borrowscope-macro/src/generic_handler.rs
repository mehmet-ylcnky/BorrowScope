//! Generic function handling utilities

use syn::{GenericParam, Generics, ItemFn};

/// Information about generic parameters in a function
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct GenericInfo {
    /// Type parameter names (e.g., T, U, V)
    pub type_params: Vec<String>,
    /// Lifetime parameter names (e.g., 'a, 'b)
    pub lifetime_params: Vec<String>,
    /// Const parameter names (e.g., N)
    pub const_params: Vec<String>,
}

#[allow(dead_code)]
impl GenericInfo {
    /// Extract generic information from function generics
    pub fn from_generics(generics: &Generics) -> Self {
        let mut type_params = Vec::new();
        let mut lifetime_params = Vec::new();
        let mut const_params = Vec::new();

        for param in &generics.params {
            match param {
                GenericParam::Type(type_param) => {
                    type_params.push(type_param.ident.to_string());
                }
                GenericParam::Lifetime(lifetime_param) => {
                    lifetime_params.push(lifetime_param.lifetime.ident.to_string());
                }
                GenericParam::Const(const_param) => {
                    const_params.push(const_param.ident.to_string());
                }
            }
        }

        Self {
            type_params,
            lifetime_params,
            const_params,
        }
    }

    /// Check if function has any generic parameters
    pub fn is_generic(&self) -> bool {
        !self.type_params.is_empty()
            || !self.lifetime_params.is_empty()
            || !self.const_params.is_empty()
    }

    /// Check if a type name is a generic type parameter
    pub fn is_type_param(&self, name: &str) -> bool {
        self.type_params.iter().any(|p| p == name)
    }
}

/// Check if function is generic
#[allow(dead_code)]
pub fn is_generic_function(func: &ItemFn) -> bool {
    !func.sig.generics.params.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_type_params() {
        let func: ItemFn = parse_quote! {
            fn example<T, U>(t: T, u: U) -> (T, U) {
                (t, u)
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.type_params, vec!["T", "U"]);
        assert!(info.is_generic());
    }

    #[test]
    fn test_extract_lifetime_params() {
        let func: ItemFn = parse_quote! {
            fn example<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
                x
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.lifetime_params, vec!["a", "b"]);
        assert!(info.is_generic());
    }

    #[test]
    fn test_extract_const_params() {
        let func: ItemFn = parse_quote! {
            fn example<const N: usize>(arr: [i32; N]) -> [i32; N] {
                arr
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.const_params, vec!["N"]);
        assert!(info.is_generic());
    }

    #[test]
    fn test_mixed_generics() {
        let func: ItemFn = parse_quote! {
            fn example<'a, T, const N: usize>(x: &'a [T; N]) -> &'a T {
                &x[0]
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.lifetime_params, vec!["a"]);
        assert_eq!(info.type_params, vec!["T"]);
        assert_eq!(info.const_params, vec!["N"]);
    }

    #[test]
    fn test_is_type_param() {
        let func: ItemFn = parse_quote! {
            fn example<T, U>(t: T, u: U) {}
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert!(info.is_type_param("T"));
        assert!(info.is_type_param("U"));
        assert!(!info.is_type_param("V"));
    }

    #[test]
    fn test_is_generic_function() {
        let func: ItemFn = parse_quote! {
            fn example<T>(value: T) -> T { value }
        };
        assert!(is_generic_function(&func));

        let func: ItemFn = parse_quote! {
            fn example(value: i32) -> i32 { value }
        };
        assert!(!is_generic_function(&func));
    }

    #[test]
    fn test_generic_with_where_clause() {
        let func: ItemFn = parse_quote! {
            fn example<T>(value: T) -> T
            where
                T: Clone + std::fmt::Debug,
            {
                value
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.type_params, vec!["T"]);
        assert!(is_generic_function(&func));
    }

    #[test]
    fn test_multiple_type_params() {
        let func: ItemFn = parse_quote! {
            fn example<T, U, V>(t: T, u: U, v: V) -> (T, U, V) {
                (t, u, v)
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.type_params.len(), 3);
        assert!(info.is_type_param("T"));
        assert!(info.is_type_param("U"));
        assert!(info.is_type_param("V"));
    }

    #[test]
    fn test_generic_with_default() {
        let func: ItemFn = parse_quote! {
            fn example<T = i32>(value: T) -> T {
                value
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.type_params, vec!["T"]);
    }

    #[test]
    fn test_non_generic_function() {
        let func: ItemFn = parse_quote! {
            fn example(x: i32, y: i32) -> i32 {
                x + y
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert!(!info.is_generic());
        assert!(info.type_params.is_empty());
        assert!(info.lifetime_params.is_empty());
        assert!(info.const_params.is_empty());
    }

    #[test]
    fn test_generic_with_trait_bounds() {
        let func: ItemFn = parse_quote! {
            fn example<T: Clone + Send>(value: T) -> T {
                value.clone()
            }
        };

        let info = GenericInfo::from_generics(&func.sig.generics);
        assert_eq!(info.type_params, vec!["T"]);
        assert!(info.is_generic());
    }
}
