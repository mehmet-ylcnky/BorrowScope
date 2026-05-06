//! # borrowscope-graph
//!
//! Graph algorithms for ownership analysis. Transforms the flat event stream
//! from `borrowscope-runtime` into a queryable ownership graph with traversal,
//! conflict detection, and multi-format export.

pub mod builder;
pub mod conflict;
pub mod edge;
pub mod error;
pub mod export;
pub mod graph;
pub mod node;
pub mod stats;
pub mod temporal;
pub mod traversal;

pub use edge::{CaptureMode, Edge, EdgeId, EdgeKind};
pub use error::GraphError;
pub use graph::{Direction, OwnershipGraph};
pub use node::{Node, NodeId, ScopeKind, ScopeNode, VariableNode};
