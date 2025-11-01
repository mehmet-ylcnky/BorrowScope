//! Event types for tracking ownership operations

use serde::{Deserialize, Serialize};

/// An ownership or borrowing event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Variable created
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// Variable borrowed
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
    },

    /// Ownership moved
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
    },

    /// Variable dropped
    Drop { timestamp: u64, var_id: String },

    /// Rc::new allocation with reference counting
    RcNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// Rc::clone operation (shared ownership)
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
            | Event::ConstEval { timestamp, .. } => *timestamp,
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
            | Event::CellSet { .. } => None,
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
