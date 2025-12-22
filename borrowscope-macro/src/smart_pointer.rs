//! Smart pointer detection and analysis
//!
//! This module provides utilities for detecting and analyzing smart pointer
//! operations in Rust code (Box, Rc, Arc, RefCell, Cell, Weak, Pin, Cow, etc.).

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
    /// Weak<T> - Weak reference (Rc)
    WeakRc,
    /// Weak<T> - Weak reference (Arc)
    WeakArc,
    /// Pin<T> - Pinned memory
    Pin,
    /// Cow<T> - Clone-on-write
    Cow,
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
            SmartPointerType::WeakRc => "Weak(Rc)",
            SmartPointerType::WeakArc => "Weak(Arc)",
            SmartPointerType::Pin => "Pin",
            SmartPointerType::Cow => "Cow",
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
        matches!(self, SmartPointerType::Arc | SmartPointerType::WeakArc)
    }

    /// Check if this is a weak reference
    pub fn is_weak(&self) -> bool {
        matches!(self, SmartPointerType::WeakRc | SmartPointerType::WeakArc)
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
    /// Rc::downgrade, Arc::downgrade
    Downgrade(SmartPointerType),
    /// Weak::upgrade
    WeakUpgrade(SmartPointerType),
    /// Weak::clone
    WeakClone(SmartPointerType),
    /// Box::pin
    BoxPin,
    /// Pin::new
    PinNew,
    /// Pin::into_inner
    PinIntoInner,
    /// Cow::Borrowed
    CowBorrowed,
    /// Cow::Owned
    CowOwned,
    /// Cow::to_mut
    CowToMut,
    /// Box::into_raw
    BoxIntoRaw,
    /// Box::from_raw
    BoxFromRaw,
    /// OnceCell::new / OnceLock::new
    OnceCellNew,
    /// OnceCell::set / OnceLock::set
    OnceCellSet,
    /// OnceCell::get / OnceLock::get
    OnceCellGet,
    /// OnceCell::get_or_init / OnceLock::get_or_init
    OnceCellGetOrInit,
    /// MaybeUninit::uninit
    MaybeUninitUninit,
    /// MaybeUninit::new
    MaybeUninitNew,
    /// MaybeUninit::write
    MaybeUninitWrite,
    /// MaybeUninit::assume_init
    MaybeUninitAssumeInit,
    /// MaybeUninit::assume_init_read
    MaybeUninitAssumeInitRead,
    /// MaybeUninit::assume_init_drop
    MaybeUninitAssumeInitDrop,
}

/// Concurrency operations we can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyOp {
    /// thread::spawn
    ThreadSpawn,
    /// JoinHandle::join
    ThreadJoin,
    /// mpsc::channel
    ChannelNew,
    /// Sender::send
    ChannelSend,
    /// Receiver::recv
    ChannelRecv,
    /// Receiver::try_recv
    ChannelTryRecv,
    /// Mutex::lock
    MutexLock,
    /// Mutex::try_lock
    MutexTryLock,
    /// RwLock::read
    RwLockRead,
    /// RwLock::write
    RwLockWrite,
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
            // Check for Cell::new but exclude UnsafeCell::new, OnceCell::new, OnceLock::new
            if normalized.contains("Cell::new") 
                && !normalized.contains("UnsafeCell::new")
                && !normalized.contains("OnceCell::new")
                && !normalized.contains("OnceLock::new") {
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

/// Detect Box::pin call
pub fn detect_box_pin(expr: &Expr) -> bool {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            return path_str.contains("Box::pin");
        }
    }
    false
}

/// Detect Box::into_raw or Box::from_raw
pub fn detect_box_raw_op(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("Box::into_raw") {
                return Some(SmartPointerOp::BoxIntoRaw);
            }
            if path_str.contains("Box::from_raw") {
                return Some(SmartPointerOp::BoxFromRaw);
            }
        }
    }
    None
}

/// Detect Pin::new or Pin::into_inner
pub fn detect_pin_operation(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("Pin::new") {
                return Some(SmartPointerOp::PinNew);
            }
            if path_str.contains("Pin::into_inner") {
                return Some(SmartPointerOp::PinIntoInner);
            }
        }
    }
    None
}

/// Detect Cow::Borrowed or Cow::Owned
pub fn detect_cow_creation(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("Cow::Borrowed") {
                return Some(SmartPointerOp::CowBorrowed);
            }
            if path_str.contains("Cow::Owned") {
                return Some(SmartPointerOp::CowOwned);
            }
        }
    }
    None
}

/// Detect Cow::to_mut method call
pub fn detect_cow_to_mut(expr: &Expr) -> bool {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        return method.to_string() == "to_mut";
    }
    false
}

/// Detect Rc::downgrade or Arc::downgrade
pub fn detect_downgrade(expr: &Expr) -> Option<SmartPointerType> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("Rc::downgrade") {
                return Some(SmartPointerType::WeakRc);
            }
            if path_str.contains("Arc::downgrade") {
                return Some(SmartPointerType::WeakArc);
            }
        }
    }
    None
}

/// Detect Weak::upgrade method call
pub fn detect_weak_upgrade(expr: &Expr) -> bool {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        return method.to_string() == "upgrade";
    }
    false
}

/// Detect OnceCell::new or OnceLock::new
pub fn detect_once_cell_new(expr: &Expr) -> Option<bool> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("OnceCell::new") {
                return Some(false); // Not thread-safe
            }
            if path_str.contains("OnceLock::new") {
                return Some(true); // Thread-safe
            }
        }
    }
    None
}

/// Detect OnceCell/OnceLock method operations
pub fn detect_once_cell_method(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        let method_name = method.to_string();
        match method_name.as_str() {
            "set" => return Some(SmartPointerOp::OnceCellSet),
            "get" => return Some(SmartPointerOp::OnceCellGet),
            "get_or_init" => return Some(SmartPointerOp::OnceCellGetOrInit),
            _ => {}
        }
    }
    None
}

/// Detect MaybeUninit::uninit or MaybeUninit::new
pub fn detect_maybe_uninit_new(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            if path_str.contains("MaybeUninit::uninit") {
                return Some(SmartPointerOp::MaybeUninitUninit);
            }
            if path_str.contains("MaybeUninit::new") {
                return Some(SmartPointerOp::MaybeUninitNew);
            }
        }
    }
    None
}

/// Detect MaybeUninit method operations
pub fn detect_maybe_uninit_method(expr: &Expr) -> Option<SmartPointerOp> {
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        let method_name = method.to_string();
        match method_name.as_str() {
            "write" => return Some(SmartPointerOp::MaybeUninitWrite),
            "assume_init" => return Some(SmartPointerOp::MaybeUninitAssumeInit),
            "assume_init_read" => return Some(SmartPointerOp::MaybeUninitAssumeInitRead),
            "assume_init_drop" => return Some(SmartPointerOp::MaybeUninitAssumeInitDrop),
            _ => {}
        }
    }
    None
}

/// Detect concurrency operations
pub fn detect_concurrency_op(expr: &Expr) -> Option<ConcurrencyOp> {
    // Check for function calls
    if let Expr::Call(ExprCall { func, .. }) = expr {
        if let Expr::Path(path) = &**func {
            let path_str = quote!(#path).to_string().replace(" ", "");
            
            if path_str.contains("thread::spawn") || path_str.contains("std::thread::spawn") {
                return Some(ConcurrencyOp::ThreadSpawn);
            }
            if path_str.contains("mpsc::channel") || path_str.contains("sync::mpsc::channel") {
                return Some(ConcurrencyOp::ChannelNew);
            }
        }
    }
    
    // Check for method calls
    if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
        let method_name = method.to_string();
        
        match method_name.as_str() {
            "join" => return Some(ConcurrencyOp::ThreadJoin),
            "send" => return Some(ConcurrencyOp::ChannelSend),
            "recv" => return Some(ConcurrencyOp::ChannelRecv),
            "try_recv" => return Some(ConcurrencyOp::ChannelTryRecv),
            "lock" => return Some(ConcurrencyOp::MutexLock),
            "try_lock" => return Some(ConcurrencyOp::MutexTryLock),
            "read" => return Some(ConcurrencyOp::RwLockRead),
            "write" => return Some(ConcurrencyOp::RwLockWrite),
            _ => {}
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

    // Check for Box::pin
    if detect_box_pin(expr) {
        return Some(SmartPointerOp::BoxPin);
    }

    // Check for Box::into_raw / Box::from_raw
    if let Some(op) = detect_box_raw_op(expr) {
        return Some(op);
    }

    // Check for Pin operations
    if let Some(op) = detect_pin_operation(expr) {
        return Some(op);
    }

    // Check for Cow creation
    if let Some(op) = detect_cow_creation(expr) {
        return Some(op);
    }

    // Check for Cow::to_mut
    if detect_cow_to_mut(expr) {
        return Some(SmartPointerOp::CowToMut);
    }

    // Check for Rc/Arc downgrade
    if let Some(sp_type) = detect_downgrade(expr) {
        return Some(SmartPointerOp::Downgrade(sp_type));
    }

    // Check for Weak::upgrade
    if detect_weak_upgrade(expr) {
        // Default to Rc weak, caller should determine actual type from context
        return Some(SmartPointerOp::WeakUpgrade(SmartPointerType::WeakRc));
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
