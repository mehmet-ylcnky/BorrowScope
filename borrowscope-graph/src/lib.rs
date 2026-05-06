//! # borrowscope-graph
//!
//! Graph algorithms for ownership analysis. Transforms the flat event stream
//! from `borrowscope-runtime` into a queryable ownership graph with traversal,
//! conflict detection, and multi-format export.

pub mod node;
pub mod edge;
pub mod graph;
pub mod builder;
pub mod error;

pub use node::{Node, NodeId, ScopeKind, ScopeNode, VariableNode};
pub use edge::{CaptureMode, Edge, EdgeId, EdgeKind};
pub use graph::OwnershipGraph;
pub use error::GraphError;
