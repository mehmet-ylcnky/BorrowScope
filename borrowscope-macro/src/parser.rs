//! Function parsing and analysis
//!
//! This module provides utilities for parsing and extracting information
//! from function items.

#![allow(dead_code)]

use syn::{FnArg, ItemFn, ReturnType};

/// Information extracted from a function
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInfo {
    pub name: String,
    pub param_count: usize,
    pub has_return_type: bool,
    pub is_async: bool,
}

/// Parse a function and extract metadata
pub fn parse_function(func: &ItemFn) -> FunctionInfo {
    FunctionInfo {
        name: func.sig.ident.to_string(),
        param_count: func.sig.inputs.len(),
        has_return_type: !matches!(func.sig.output, ReturnType::Default),
        is_async: func.sig.asyncness.is_some(),
    }
}

/// Get parameter names from a function
pub fn get_parameter_names(func: &ItemFn) -> Vec<String> {
    func.sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.to_string());
                }
            }
            None
        })
        .collect()
}

/// Check if function has generic parameters
pub fn has_generics(func: &ItemFn) -> bool {
    !func.sig.generics.params.is_empty()
}

/// Check if function has lifetime parameters
pub fn has_lifetimes(func: &ItemFn) -> bool {
    func.sig.generics.lifetimes().next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_simple_function() {
        let func: ItemFn = parse_quote! {
            fn example() {
                let x = 5;
            }
        };
        let info = parse_function(&func);
        assert_eq!(info.name, "example");
        assert_eq!(info.param_count, 0);
        assert!(!info.has_return_type);
        assert!(!info.is_async);
    }

    #[test]
    fn test_parse_function_with_params() {
        let func: ItemFn = parse_quote! {
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        };
        let info = parse_function(&func);
        assert_eq!(info.name, "add");
        assert_eq!(info.param_count, 2);
        assert!(info.has_return_type);
        assert!(!info.is_async);
    }

    #[test]
    fn test_parse_async_function() {
        let func: ItemFn = parse_quote! {
            async fn fetch_data() -> String {
                String::from("data")
            }
        };
        let info = parse_function(&func);
        assert_eq!(info.name, "fetch_data");
        assert!(info.has_return_type);
        assert!(info.is_async);
    }

    #[test]
    fn test_get_parameter_names() {
        let func: ItemFn = parse_quote! {
            fn process(input: String, count: usize) {
                println!("{} {}", input, count);
            }
        };
        let names = get_parameter_names(&func);
        assert_eq!(names, vec!["input", "count"]);
    }

    #[test]
    fn test_get_parameter_names_empty() {
        let func: ItemFn = parse_quote! {
            fn no_params() {
                println!("hello");
            }
        };
        let names = get_parameter_names(&func);
        assert!(names.is_empty());
    }

    #[test]
    fn test_has_generics() {
        let func: ItemFn = parse_quote! {
            fn generic<T>(value: T) -> T {
                value
            }
        };
        assert!(has_generics(&func));

        let func: ItemFn = parse_quote! {
            fn not_generic(value: i32) -> i32 {
                value
            }
        };
        assert!(!has_generics(&func));
    }

    #[test]
    fn test_has_lifetimes() {
        let func: ItemFn = parse_quote! {
            fn with_lifetime<'a>(s: &'a str) -> &'a str {
                s
            }
        };
        assert!(has_lifetimes(&func));

        let func: ItemFn = parse_quote! {
            fn no_lifetime(s: String) -> String {
                s
            }
        };
        assert!(!has_lifetimes(&func));
    }
}
