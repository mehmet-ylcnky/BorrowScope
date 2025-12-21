//! Smart pointer detection and analysis
//!
//! This module provides utilities for detecting and analyzing smart pointer
//! operations in Rust code (Box, Rc, Arc, RefCell, Cell).

#![allow(dead_code)] // Functions will be used in future sections

use quote::quote;
use syn::{Expr, ExprCall, ExprMethodCall};

/// Types of smart pointers we can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartPointerType {
    /// Box<T> - Heap allocation
    Box,
    /// Rc<T> - Reference counting
    Rc,
    /// Arc<T> - Atomic reference counting
    Arc,
    /// RefCell<T> - Interior mutability with runtime borrow checking
    RefCell,
    /// Cell<T> - Interior mutability for Copy types
    Cell,
}

impl SmartPointerType {
    /// Get the name of the smart pointer type
    pub fn name(&self) -> &'static str {
        match self {
            SmartPointerType::Box => "Box",
            SmartPointerType::Rc => "Rc",
            SmartPointerType::Arc => "Arc",
            SmartPointerType::RefCell => "RefCell",
            SmartPointerType::Cell => "Cell",
        }
    }

    /// Check if this is a reference-counted type
    pub fn is_reference_counted(&self) -> bool {
        matches!(self, SmartPointerType::Rc | SmartPointerType::Arc)
    }

    /// Check if this provides interior mutability
    pub fn has_interior_mutability(&self) -> bool {
        matches!(self, SmartPointerType::RefCell | SmartPointerType::Cell)
    }

    /// Check if this is thread-safe
    pub fn is_thread_safe(&self) -> bool {
        matches!(self, SmartPointerType::Arc)
    }
}

/// Smart pointer operations we can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartPointerOp {
    /// Box::new, Rc::new, Arc::new, etc.
    New(SmartPointerType),
    /// Rc::clone, Arc::clone
    Clone(SmartPointerType),
    /// RefCell::borrow
    Borrow,
    /// RefCell::borrow_mut
    BorrowMut,
    /// Cell::get
    CellGet,
    /// Cell::set
    CellSet,
}

/// Detect smart pointer creation (::new calls)
pub fn detect_smart_pointer_new(expr: &Expr) -> Option<SmartPointerType> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string();

            // Normalize whitespace for matching
            let normalized = path_str.replace(" ", "");

            if normalized.contains("Box::new") {
                return Some(SmartPointerType::Box);
            }
            if normalized.contains("Rc::new") {
                return Some(SmartPointerType::Rc);
            }
            if normalized.contains("Arc::new") {
                return Some(SmartPointerType::Arc);
            }
            if normalized.contains("RefCell::new") {
                return Some(SmartPointerType::RefCell);
            }
            if normalized.contains("Cell::new") {
                return Some(SmartPointerType::Cell);
            }
        }
    }
    None
}

/// Detect Rc::clone or Arc::clone
pub fn detect_rc_clone(expr: &Expr) -> Option<SmartPointerType> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string();
            let normalized = path_str.replace(" ", "");

            if normalized.contains("Rc::clone") {
                return Some(SmartPointerType::Rc);
            }
            if normalized.contains("Arc::clone") {
                return Some(SmartPointerType::Arc);
            }
        }
    }
    None
}

/// Detect RefCell borrow operations
pub fn detect_refcell_borrow(expr: &Expr) -> Option<bool> {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        let method_name = method.to_string();

        if method_name == "borrow" {
            return Some(false); // Immutable borrow
        }
        if method_name == "borrow_mut" {
            return Some(true); // Mutable borrow
        }
    }
    None
}

/// Detect Cell get/set operations
pub fn detect_cell_operation(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        let method_name = method.to_string();

        if method_name == "get" {
            return Some(SmartPointerOp::CellGet);
        }
        if method_name == "set" {
            return Some(SmartPointerOp::CellSet);
        }
    }
    None
}

/// Check if an expression is a smart pointer operation
pub fn is_smart_pointer_operation(expr: &Expr) -> Option<SmartPointerOp> {
    // Check for ::new
    if let Some(sp_type) = detect_smart_pointer_new(expr) {
        return Some(SmartPointerOp::New(sp_type));
    }

    // Check for Rc/Arc clone
    if let Some(sp_type) = detect_rc_clone(expr) {
        return Some(SmartPointerOp::Clone(sp_type));
    }

    // Check for RefCell borrow
    if let Some(is_mut) = detect_refcell_borrow(expr) {
        return Some(if is_mut {
            SmartPointerOp::BorrowMut
        } else {
            SmartPointerOp::Borrow
        });
    }

    // Check for Cell operations
    if let Some(op) = detect_cell_operation(expr) {
        return Some(op);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_box_new() {
        let expr: Expr = parse_quote! { Box::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Box));
    }

    #[test]
    fn test_detect_rc_new() {
        let expr: Expr = parse_quote! { Rc::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Rc));
    }

    #[test]
    fn test_detect_arc_new() {
        let expr: Expr = parse_quote! { Arc::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Arc));
    }

    #[test]
    fn test_detect_refcell_new() {
        let expr: Expr = parse_quote! { RefCell::new(42) };
        assert_eq!(
            detect_smart_pointer_new(&expr),
            Some(SmartPointerType::RefCell)
        );
    }

    #[test]
    fn test_detect_cell_new() {
        let expr: Expr = parse_quote! { Cell::new(42) };
        assert_eq!(
            detect_smart_pointer_new(&expr),
            Some(SmartPointerType::Cell)
        );
    }

    #[test]
    fn test_detect_rc_clone() {
        let expr: Expr = parse_quote! { Rc::clone(&x) };
        assert_eq!(detect_rc_clone(&expr), Some(SmartPointerType::Rc));
    }

    #[test]
    fn test_detect_arc_clone() {
        let expr: Expr = parse_quote! { Arc::clone(&x) };
        assert_eq!(detect_rc_clone(&expr), Some(SmartPointerType::Arc));
    }

    #[test]
    fn test_detect_refcell_borrow() {
        let expr: Expr = parse_quote! { x.borrow() };
        assert_eq!(detect_refcell_borrow(&expr), Some(false));
    }

    #[test]
    fn test_detect_refcell_borrow_mut() {
        let expr: Expr = parse_quote! { x.borrow_mut() };
        assert_eq!(detect_refcell_borrow(&expr), Some(true));
    }

    #[test]
    fn test_detect_cell_get() {
        let expr: Expr = parse_quote! { x.get() };
        assert_eq!(detect_cell_operation(&expr), Some(SmartPointerOp::CellGet));
    }

    #[test]
    fn test_detect_cell_set() {
        let expr: Expr = parse_quote! { x.set(42) };
        assert_eq!(detect_cell_operation(&expr), Some(SmartPointerOp::CellSet));
    }

    #[test]
    fn test_smart_pointer_type_properties() {
        assert_eq!(SmartPointerType::Box.name(), "Box");
        assert_eq!(SmartPointerType::Rc.name(), "Rc");
        assert_eq!(SmartPointerType::Arc.name(), "Arc");

        assert!(SmartPointerType::Rc.is_reference_counted());
        assert!(SmartPointerType::Arc.is_reference_counted());
        assert!(!SmartPointerType::Box.is_reference_counted());

        assert!(SmartPointerType::RefCell.has_interior_mutability());
        assert!(SmartPointerType::Cell.has_interior_mutability());
        assert!(!SmartPointerType::Box.has_interior_mutability());

        assert!(SmartPointerType::Arc.is_thread_safe());
        assert!(!SmartPointerType::Rc.is_thread_safe());
    }

    #[test]
    fn test_is_smart_pointer_operation_box() {
        let expr: Expr = parse_quote! { Box::new(42) };
        assert_eq!(
            is_smart_pointer_operation(&expr),
            Some(SmartPointerOp::New(SmartPointerType::Box))
        );
    }

    #[test]
    fn test_is_smart_pointer_operation_rc_clone() {
        let expr: Expr = parse_quote! { Rc::clone(&x) };
        assert_eq!(
            is_smart_pointer_operation(&expr),
            Some(SmartPointerOp::Clone(SmartPointerType::Rc))
        );
    }

    #[test]
    fn test_is_smart_pointer_operation_refcell_borrow() {
        let expr: Expr = parse_quote! { x.borrow() };
        assert_eq!(
            is_smart_pointer_operation(&expr),
            Some(SmartPointerOp::Borrow)
        );
    }

    #[test]
    fn test_non_smart_pointer_expression() {
        let expr: Expr = parse_quote! { 42 };
        assert_eq!(is_smart_pointer_operation(&expr), None);

        let expr: Expr = parse_quote! { String::from("hello") };
        assert_eq!(is_smart_pointer_operation(&expr), None);
    }

    #[test]
    fn test_detect_with_full_path() {
        let expr: Expr = parse_quote! { std::boxed::Box::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Box));

        let expr: Expr = parse_quote! { std::rc::Rc::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Rc));

        let expr: Expr = parse_quote! { std::sync::Arc::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Arc));
    }

    #[test]
    fn test_detect_with_use_statement() {
        // After: use std::rc::Rc;
        let expr: Expr = parse_quote! { Rc::new(42) };
        assert_eq!(detect_smart_pointer_new(&expr), Some(SmartPointerType::Rc));
    }
}
