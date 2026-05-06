//! Error types for graph operations.

use std::fmt;

/// Errors that can occur during graph operations.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    /// Referenced node does not exist.
    NodeNotFound(super::NodeId),
    /// Referenced edge does not exist.
    EdgeNotFound(super::EdgeId),
    /// Import/parse error.
    ParseError(String),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "node not found: {:?}", id),
            Self::EdgeNotFound(id) => write!(f, "edge not found: {:?}", id),
            Self::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

impl std::error::Error for GraphError {}
