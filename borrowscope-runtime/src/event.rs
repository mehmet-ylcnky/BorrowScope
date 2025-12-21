//! Event types for tracking ownership operations.
//!
//! This module defines the [`Event`] enum which represents all possible
//! ownership and borrowing events that can be tracked at runtime.
//!
//! # Event Categories
//!
//! - **Basic ownership**: `New`, `Borrow`, `Move`, `Drop`
//! - **Smart pointers**: `RcNew`, `RcClone`, `ArcNew`, `ArcClone`
//! - **Interior mutability**: `RefCellNew`, `RefCellBorrow`, `RefCellDrop`, `CellNew`, `CellGet`, `CellSet`
//! - **Unsafe operations**: `RawPtrCreated`, `RawPtrDeref`, `UnsafeBlockEnter`, `UnsafeBlockExit`
//!
//! # Serialization
//!
//! All events serialize to JSON with a `type` tag for easy filtering.

use serde::{Deserialize, Serialize};

/// An ownership or borrowing event recorded at runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Variable created via [`track_new`](crate::track_new).
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// Variable borrowed via [`track_borrow`](crate::track_borrow).
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
    },

    /// Ownership moved via [`track_move`](crate::track_move).
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
    },

    /// Variable dropped via [`track_drop`](crate::track_drop).
    Drop { timestamp: u64, var_id: String },

    /// `Rc::new` allocation with reference counting.
    RcNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// `Rc::clone` operation (shared ownership).
    RcClone {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// Arc::new allocation with atomic reference counting
    ArcNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// Arc::clone operation (thread-safe shared ownership)
    ArcClone {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// RefCell::new allocation
    RefCellNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// RefCell::borrow or borrow_mut operation
    RefCellBorrow {
        timestamp: u64,
        borrow_id: String,
        refcell_id: String,
        is_mutable: bool,
        location: String,
    },

    /// RefCell borrow dropped (Ref/RefMut dropped)
    RefCellDrop {
        timestamp: u64,
        borrow_id: String,
        location: String,
    },

    /// Cell::new allocation
    CellNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// Cell::get operation
    CellGet {
        timestamp: u64,
        cell_id: String,
        location: String,
    },

    /// Cell::set operation
    CellSet {
        timestamp: u64,
        cell_id: String,
        location: String,
    },

    /// Static variable initialization
    StaticInit {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        is_mutable: bool,
    },

    /// Static variable access (read or write)
    StaticAccess {
        timestamp: u64,
        var_id: String,
        var_name: String,
        is_write: bool,
        location: String,
    },

    /// Const evaluation (compile-time constant)
    ConstEval {
        timestamp: u64,
        const_name: String,
        const_id: String,
        type_name: String,
        location: String,
    },

    /// Raw pointer created
    RawPtrCreated {
        timestamp: u64,
        var_name: String,
        var_id: String,
        ptr_type: String,
        address: usize,
        location: String,
    },

    /// Raw pointer dereferenced
    RawPtrDeref {
        timestamp: u64,
        ptr_id: String,
        location: String,
        is_write: bool,
    },

    /// Unsafe block entered
    UnsafeBlockEnter {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Unsafe block exited
    UnsafeBlockExit {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Unsafe function called
    UnsafeFnCall {
        timestamp: u64,
        fn_name: String,
        location: String,
    },

    /// FFI (Foreign Function Interface) call
    FfiCall {
        timestamp: u64,
        fn_name: String,
        location: String,
    },

    /// Transmute operation
    Transmute {
        timestamp: u64,
        from_type: String,
        to_type: String,
        location: String,
    },

    /// Union field access
    UnionFieldAccess {
        timestamp: u64,
        union_name: String,
        field_name: String,
        location: String,
    },

    /// Async block entered
    AsyncBlockEnter {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Async block exited
    AsyncBlockExit {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Await expression started
    AwaitStart {
        timestamp: u64,
        await_id: String,
        future_name: String,
        location: String,
    },

    /// Await expression completed
    AwaitEnd {
        timestamp: u64,
        await_id: String,
        location: String,
    },

    // ========== Phase 5: Extended Tracking ==========

    /// Loop entered (for, while, loop)
    LoopEnter {
        timestamp: u64,
        loop_id: String,
        loop_type: String,
        location: String,
    },

    /// Loop iteration
    LoopIteration {
        timestamp: u64,
        loop_id: String,
        iteration: usize,
        location: String,
    },

    /// Loop exited
    LoopExit {
        timestamp: u64,
        loop_id: String,
        location: String,
    },

    /// Match expression entered
    MatchEnter {
        timestamp: u64,
        match_id: String,
        location: String,
    },

    /// Match arm taken
    MatchArm {
        timestamp: u64,
        match_id: String,
        arm_index: usize,
        pattern: String,
        location: String,
    },

    /// Match expression exited
    MatchExit {
        timestamp: u64,
        match_id: String,
        location: String,
    },

    /// Branch taken (if/else)
    Branch {
        timestamp: u64,
        branch_id: String,
        branch_type: String,
        location: String,
    },

    /// Return statement
    Return {
        timestamp: u64,
        return_id: String,
        has_value: bool,
        location: String,
    },

    /// Try/? operator
    Try {
        timestamp: u64,
        try_id: String,
        location: String,
    },

    /// Index access (arr[i])
    IndexAccess {
        timestamp: u64,
        access_id: String,
        container: String,
        location: String,
    },

    /// Field access (obj.field)
    FieldAccess {
        timestamp: u64,
        access_id: String,
        base: String,
        field: String,
        location: String,
    },

    /// Function call
    Call {
        timestamp: u64,
        call_id: String,
        fn_name: String,
        location: String,
    },

    /// Lock acquired (Mutex/RwLock)
    Lock {
        timestamp: u64,
        lock_id: String,
        lock_type: String,
        var_name: String,
        location: String,
    },

    /// Unwrap called (Option/Result)
    Unwrap {
        timestamp: u64,
        unwrap_id: String,
        method: String,
        var_name: String,
        location: String,
    },

    /// Clone called
    Clone {
        timestamp: u64,
        clone_id: String,
        var_name: String,
        location: String,
    },

    /// Dereference operation
    Deref {
        timestamp: u64,
        deref_id: String,
        var_name: String,
        location: String,
    },

    // ========== Phase 6: Additional Tracking ==========

    /// Break statement
    Break {
        timestamp: u64,
        break_id: String,
        loop_label: Option<String>,
        location: String,
    },

    /// Continue statement
    Continue {
        timestamp: u64,
        continue_id: String,
        loop_label: Option<String>,
        location: String,
    },

    /// Closure creation
    ClosureCreate {
        timestamp: u64,
        closure_id: String,
        capture_mode: String,
        location: String,
    },

    /// Struct construction
    StructCreate {
        timestamp: u64,
        struct_id: String,
        type_name: String,
        location: String,
    },

    /// Tuple construction
    TupleCreate {
        timestamp: u64,
        tuple_id: String,
        len: usize,
        location: String,
    },

    /// Let-else pattern
    LetElse {
        timestamp: u64,
        let_id: String,
        pattern: String,
        location: String,
    },

    /// Range expression
    Range {
        timestamp: u64,
        range_id: String,
        range_type: String,
        location: String,
    },

    /// Binary operation
    BinaryOp {
        timestamp: u64,
        op_id: String,
        operator: String,
        location: String,
    },

    /// Array creation
    ArrayCreate {
        timestamp: u64,
        array_id: String,
        len: usize,
        location: String,
    },

    /// Type cast (non-pointer)
    TypeCast {
        timestamp: u64,
        cast_id: String,
        to_type: String,
        location: String,
    },

    /// Region enter
    RegionEnter {
        timestamp: u64,
        region_id: String,
        name: String,
        location: String,
    },

    /// Region exit
    RegionExit {
        timestamp: u64,
        region_id: String,
        location: String,
    },
}

impl Event {
    /// Get the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::New { timestamp, .. }
            | Event::Borrow { timestamp, .. }
            | Event::Move { timestamp, .. }
            | Event::Drop { timestamp, .. }
            | Event::RcNew { timestamp, .. }
            | Event::RcClone { timestamp, .. }
            | Event::ArcNew { timestamp, .. }
            | Event::ArcClone { timestamp, .. }
            | Event::RefCellNew { timestamp, .. }
            | Event::RefCellBorrow { timestamp, .. }
            | Event::RefCellDrop { timestamp, .. }
            | Event::CellNew { timestamp, .. }
            | Event::CellGet { timestamp, .. }
            | Event::CellSet { timestamp, .. }
            | Event::StaticInit { timestamp, .. }
            | Event::StaticAccess { timestamp, .. }
            | Event::ConstEval { timestamp, .. }
            | Event::RawPtrCreated { timestamp, .. }
            | Event::RawPtrDeref { timestamp, .. }
            | Event::UnsafeBlockEnter { timestamp, .. }
            | Event::UnsafeBlockExit { timestamp, .. }
            | Event::UnsafeFnCall { timestamp, .. }
            | Event::FfiCall { timestamp, .. }
            | Event::Transmute { timestamp, .. }
            | Event::UnionFieldAccess { timestamp, .. }
            | Event::AsyncBlockEnter { timestamp, .. }
            | Event::AsyncBlockExit { timestamp, .. }
            | Event::AwaitStart { timestamp, .. }
            | Event::AwaitEnd { timestamp, .. }
            | Event::LoopEnter { timestamp, .. }
            | Event::LoopIteration { timestamp, .. }
            | Event::LoopExit { timestamp, .. }
            | Event::MatchEnter { timestamp, .. }
            | Event::MatchArm { timestamp, .. }
            | Event::MatchExit { timestamp, .. }
            | Event::Branch { timestamp, .. }
            | Event::Return { timestamp, .. }
            | Event::Try { timestamp, .. }
            | Event::IndexAccess { timestamp, .. }
            | Event::FieldAccess { timestamp, .. }
            | Event::Call { timestamp, .. }
            | Event::Lock { timestamp, .. }
            | Event::Unwrap { timestamp, .. }
            | Event::Clone { timestamp, .. }
            | Event::Deref { timestamp, .. }
            | Event::Break { timestamp, .. }
            | Event::Continue { timestamp, .. }
            | Event::ClosureCreate { timestamp, .. }
            | Event::StructCreate { timestamp, .. }
            | Event::TupleCreate { timestamp, .. }
            | Event::LetElse { timestamp, .. }
            | Event::Range { timestamp, .. }
            | Event::BinaryOp { timestamp, .. }
            | Event::ArrayCreate { timestamp, .. }
            | Event::TypeCast { timestamp, .. }
            | Event::RegionEnter { timestamp, .. }
            | Event::RegionExit { timestamp, .. } => *timestamp,
        }
    }

    /// Get the variable name (if applicable)
    pub fn var_name(&self) -> Option<&str> {
        match self {
            Event::New { var_name, .. }
            | Event::RcNew { var_name, .. }
            | Event::RcClone { var_name, .. }
            | Event::ArcNew { var_name, .. }
            | Event::ArcClone { var_name, .. }
            | Event::RefCellNew { var_name, .. }
            | Event::CellNew { var_name, .. }
            | Event::StaticInit { var_name, .. }
            | Event::StaticAccess { var_name, .. }
            | Event::RawPtrCreated { var_name, .. }
            | Event::ConstEval {
                const_name: var_name,
                ..
            } => Some(var_name),
            Event::Borrow { borrower_name, .. } => Some(borrower_name),
            Event::Move { to_name, .. } => Some(to_name),
            Event::Drop { var_id, .. } => Some(var_id),
            Event::RefCellBorrow { .. }
            | Event::RefCellDrop { .. }
            | Event::CellGet { .. }
            | Event::CellSet { .. }
            | Event::RawPtrDeref { .. }
            | Event::UnsafeBlockEnter { .. }
            | Event::UnsafeBlockExit { .. }
            | Event::UnsafeFnCall { .. }
            | Event::FfiCall { .. }
            | Event::Transmute { .. }
            | Event::UnionFieldAccess { .. }
            | Event::AsyncBlockEnter { .. }
            | Event::AsyncBlockExit { .. }
            | Event::AwaitStart { .. }
            | Event::AwaitEnd { .. }
            | Event::LoopEnter { .. }
            | Event::LoopIteration { .. }
            | Event::LoopExit { .. }
            | Event::MatchEnter { .. }
            | Event::MatchArm { .. }
            | Event::MatchExit { .. }
            | Event::Branch { .. }
            | Event::Return { .. }
            | Event::Try { .. }
            | Event::IndexAccess { .. }
            | Event::FieldAccess { .. }
            | Event::Call { .. }
            | Event::Lock { .. }
            | Event::Unwrap { .. }
            | Event::Clone { .. }
            | Event::Deref { .. }
            | Event::Break { .. }
            | Event::Continue { .. }
            | Event::ClosureCreate { .. }
            | Event::StructCreate { .. }
            | Event::TupleCreate { .. }
            | Event::LetElse { .. }
            | Event::Range { .. }
            | Event::BinaryOp { .. }
            | Event::ArrayCreate { .. }
            | Event::TypeCast { .. }
            | Event::RegionEnter { .. }
            | Event::RegionExit { .. } => None,
        }
    }

    /// Check if this is a New event
    pub fn is_new(&self) -> bool {
        matches!(self, Event::New { .. })
    }

    /// Check if this is a Borrow event
    pub fn is_borrow(&self) -> bool {
        matches!(self, Event::Borrow { .. })
    }

    /// Check if this is a Move event
    pub fn is_move(&self) -> bool {
        matches!(self, Event::Move { .. })
    }

    /// Check if this is a Drop event
    pub fn is_drop(&self) -> bool {
        matches!(self, Event::Drop { .. })
    }

    /// Check if this is an Rc event (new or clone)
    pub fn is_rc(&self) -> bool {
        matches!(self, Event::RcNew { .. } | Event::RcClone { .. })
    }

    /// Check if this is an Arc event (new or clone)
    pub fn is_arc(&self) -> bool {
        matches!(self, Event::ArcNew { .. } | Event::ArcClone { .. })
    }

    /// Check if this is a reference-counted event
    pub fn is_refcounted(&self) -> bool {
        self.is_rc() || self.is_arc()
    }

    /// Check if this is a RefCell event
    pub fn is_refcell(&self) -> bool {
        matches!(
            self,
            Event::RefCellNew { .. } | Event::RefCellBorrow { .. } | Event::RefCellDrop { .. }
        )
    }

    /// Check if this is a Cell event
    pub fn is_cell(&self) -> bool {
        matches!(
            self,
            Event::CellNew { .. } | Event::CellGet { .. } | Event::CellSet { .. }
        )
    }

    /// Check if this is an interior mutability event
    pub fn is_interior_mutability(&self) -> bool {
        self.is_refcell() || self.is_cell()
    }

    /// Check if this is a static event
    pub fn is_static(&self) -> bool {
        matches!(self, Event::StaticInit { .. } | Event::StaticAccess { .. })
    }

    /// Check if this is a const event
    pub fn is_const(&self) -> bool {
        matches!(self, Event::ConstEval { .. })
    }

    /// Check if this is a global variable event (static or const)
    pub fn is_global(&self) -> bool {
        self.is_static() || self.is_const()
    }

    /// Check if this is an unsafe event
    pub fn is_unsafe(&self) -> bool {
        matches!(
            self,
            Event::RawPtrCreated { .. }
                | Event::RawPtrDeref { .. }
                | Event::UnsafeBlockEnter { .. }
                | Event::UnsafeBlockExit { .. }
                | Event::UnsafeFnCall { .. }
                | Event::FfiCall { .. }
                | Event::Transmute { .. }
                | Event::UnionFieldAccess { .. }
        )
    }

    /// Check if this is a raw pointer event
    pub fn is_raw_ptr(&self) -> bool {
        matches!(
            self,
            Event::RawPtrCreated { .. } | Event::RawPtrDeref { .. }
        )
    }

    /// Check if this is an FFI event
    pub fn is_ffi(&self) -> bool {
        matches!(self, Event::FfiCall { .. })
    }

    /// Get strong count if this is a reference-counted event
    pub fn strong_count(&self) -> Option<usize> {
        match self {
            Event::RcNew { strong_count, .. }
            | Event::RcClone { strong_count, .. }
            | Event::ArcNew { strong_count, .. }
            | Event::ArcClone { strong_count, .. } => Some(*strong_count),
            _ => None,
        }
    }

    /// Get weak count if this is a reference-counted event
    pub fn weak_count(&self) -> Option<usize> {
        match self {
            Event::RcNew { weak_count, .. }
            | Event::RcClone { weak_count, .. }
            | Event::ArcNew { weak_count, .. }
            | Event::ArcClone { weak_count, .. } => Some(*weak_count),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let event = Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        };

        assert_eq!(event.timestamp(), 1);
        assert_eq!(event.var_name(), Some("x"));
        assert!(event.is_new());
        assert!(!event.is_borrow());
        assert!(!event.is_move());
        assert!(!event.is_drop());
    }

    #[test]
    fn test_event_borrow() {
        let event = Event::Borrow {
            timestamp: 2,
            borrower_name: "r".to_string(),
            borrower_id: "r_1".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        };

        assert_eq!(event.timestamp(), 2);
        assert_eq!(event.var_name(), Some("r"));
        assert!(event.is_borrow());
        assert!(!event.is_new());
    }

    #[test]
    fn test_event_move() {
        let event = Event::Move {
            timestamp: 3,
            from_id: "x_0".to_string(),
            to_name: "y".to_string(),
            to_id: "y_1".to_string(),
        };

        assert_eq!(event.timestamp(), 3);
        assert_eq!(event.var_name(), Some("y"));
        assert!(event.is_move());
    }

    #[test]
    fn test_event_drop() {
        let event = Event::Drop {
            timestamp: 4,
            var_id: "x_0".to_string(),
        };

        assert_eq!(event.timestamp(), 4);
        assert!(event.is_drop());
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_borrow_mutable_flag() {
        let immut = Event::Borrow {
            timestamp: 1,
            borrower_name: "r".to_string(),
            borrower_id: "r_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        };

        let mut_borrow = Event::Borrow {
            timestamp: 2,
            borrower_name: "r".to_string(),
            borrower_id: "r_1".to_string(),
            owner_id: "x_0".to_string(),
            mutable: true,
        };

        assert_ne!(immut, mut_borrow);
    }
}
