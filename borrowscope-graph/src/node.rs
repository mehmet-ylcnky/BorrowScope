//! Node types for the ownership graph.

use serde::{Deserialize, Serialize};

/// Opaque node identifier. Cheap to copy and compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Kind of scope boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKind {
    Function,
    Block,
    Loop,
    Match,
}

/// A variable in the ownership graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNode {
    pub id: NodeId,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: u32,
    pub is_copy: bool,
    pub is_mutable: bool,
}

/// A scope boundary (function body, block, loop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeNode {
    pub id: NodeId,
    pub name: String,
    pub kind: ScopeKind,
    pub entered_at: u64,
    pub exited_at: Option<u64>,
}

/// Unified node type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "node_type")]
pub enum Node {
    Variable(VariableNode),
    Scope(ScopeNode),
}

impl Node {
    /// Get the node's unique identifier.
    pub fn id(&self) -> NodeId {
        match self {
            Node::Variable(v) => v.id,
            Node::Scope(s) => s.id,
        }
    }

    /// Get the node's name.
    pub fn name(&self) -> &str {
        match self {
            Node::Variable(v) => &v.name,
            Node::Scope(s) => &s.name,
        }
    }

    /// Check if this node is alive at the given timestamp.
    pub fn is_alive_at(&self, timestamp: u64) -> bool {
        match self {
            Node::Variable(v) => {
                timestamp >= v.created_at
                    && v.dropped_at.map_or(true, |d| timestamp < d)
            }
            Node::Scope(s) => {
                timestamp >= s.entered_at
                    && s.exited_at.map_or(true, |e| timestamp < e)
            }
        }
    }

    /// Get the creation/entry timestamp.
    pub fn start_time(&self) -> u64 {
        match self {
            Node::Variable(v) => v.created_at,
            Node::Scope(s) => s.entered_at,
        }
    }

    /// Get the drop/exit timestamp, if any.
    pub fn end_time(&self) -> Option<u64> {
        match self {
            Node::Variable(v) => v.dropped_at,
            Node::Scope(s) => s.exited_at,
        }
    }
}
