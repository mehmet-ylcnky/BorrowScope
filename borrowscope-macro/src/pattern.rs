//! Pattern analysis utilities for extracting variable names from patterns

#![allow(dead_code)]

use syn::{Ident, Pat, PatIdent, PatStruct, PatTuple};

/// Information extracted from a pattern
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub variables: Vec<Ident>,
    pub is_simple: bool,
}

impl PatternInfo {
    /// Analyze a pattern and extract variable names
    pub fn analyze(pat: &Pat) -> Self {
        let mut variables = Vec::new();
        extract_variables(pat, &mut variables);

        let is_simple = variables.len() == 1;

        PatternInfo {
            variables,
            is_simple,
        }
    }
}

/// Extract all variable names from a pattern recursively
fn extract_variables(pat: &Pat, variables: &mut Vec<Ident>) {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => {
            if !ident.to_string().starts_with('_') {
                variables.push(ident.clone());
            }
        }
        Pat::Tuple(PatTuple { elems, .. }) => {
            for elem in elems {
                extract_variables(elem, variables);
            }
        }
        Pat::Struct(PatStruct { fields, .. }) => {
            for field in fields {
                extract_variables(&field.pat, variables);
            }
        }
        Pat::TupleStruct(tuple_struct) => {
            for elem in &tuple_struct.elems {
                extract_variables(elem, variables);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_pattern() {
        let pat: Pat = parse_quote! { x };
        let info = PatternInfo::analyze(&pat);
        assert_eq!(info.variables.len(), 1);
        assert_eq!(info.variables[0].to_string(), "x");
        assert!(info.is_simple);
    }

    #[test]
    fn test_tuple_pattern() {
        let pat: Pat = parse_quote! { (x, y) };
        let info = PatternInfo::analyze(&pat);
        assert_eq!(info.variables.len(), 2);
        assert_eq!(info.variables[0].to_string(), "x");
        assert_eq!(info.variables[1].to_string(), "y");
        assert!(!info.is_simple);
    }

    #[test]
    fn test_struct_pattern() {
        let pat: Pat = parse_quote! { Point { x, y } };
        let info = PatternInfo::analyze(&pat);
        assert_eq!(info.variables.len(), 2);
        assert!(info.variables.iter().any(|v| v.to_string() == "x"));
        assert!(info.variables.iter().any(|v| v.to_string() == "y"));
    }

    #[test]
    fn test_underscore_ignored() {
        let pat: Pat = parse_quote! { _x };
        let info = PatternInfo::analyze(&pat);
        assert_eq!(info.variables.len(), 0);
    }

    #[test]
    fn test_nested_tuple() {
        let pat: Pat = parse_quote! { (a, (b, c)) };
        let info = PatternInfo::analyze(&pat);
        assert_eq!(info.variables.len(), 3);
    }
}
