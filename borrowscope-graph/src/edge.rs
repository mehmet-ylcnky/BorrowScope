//! Edge types for the ownership graph.

use crate::node::NodeId;
use serde::{Deserialize, Serialize};

/// Opaque edge identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub usize);

/// How a closure captures a variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
    ByRef,
    ByMutRef,
    ByMove,
}

/// The kind of ownership relationship an edge represents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Immutable borrow: &T
    BorrowShared,
    /// Mutable borrow: &mut T
    BorrowMut,
    /// Ownership transfer (move)
    Move,
    /// Rc::clone (shared ownership)
    RcClone { strong_count: u32 },
    /// Arc::clone (shared ownership, thread-safe)
    ArcClone { strong_count: u32 },
    /// Weak::downgrade
    WeakDowngrade,
    /// RefCell::borrow / RefCell::borrow_mut
    RefCellBorrow { mutable: bool },
    /// Mutex/RwLock lock acquisition
    LockAcquire { lock_type: String },
    /// Closure captures a variable
    ClosureCapture { capture_mode: CaptureMode },
    /// Variable contained in scope
    ScopeContains,
    /// Channel send (ownership transfer across threads)
    ChannelSend,
}

/// A directed edge between two nodes with temporal bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub created_at: u64,
    pub ended_at: Option<u64>,
}

impl Edge {
    /// Check if this edge is active at the given timestamp.
    pub fn is_active_at(&self, timestamp: u64) -> bool {
        timestamp >= self.created_at && self.ended_at.map_or(true, |e| timestamp < e)
    }

    /// Check if this edge represents a borrow (shared or mutable).
    pub fn is_borrow(&self) -> bool {
        matches!(self.kind, EdgeKind::BorrowShared | EdgeKind::BorrowMut)
    }

    /// Check if this edge represents a mutable relationship.
    pub fn is_mutable(&self) -> bool {
        matches!(
            self.kind,
            EdgeKind::BorrowMut
                | EdgeKind::RefCellBorrow { mutable: true }
                | EdgeKind::ClosureCapture {
                    capture_mode: CaptureMode::ByMutRef
                }
        )
    }

    /// Duration of this edge, if it has ended.
    pub fn duration(&self) -> Option<u64> {
        self.ended_at.map(|e| e - self.created_at)
    }
}
