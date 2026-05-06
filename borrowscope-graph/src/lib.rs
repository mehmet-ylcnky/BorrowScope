//! # borrowscope-graph
//!
//! Graph algorithms for ownership analysis. Transforms the flat event stream
//! from `borrowscope-runtime` into a queryable ownership graph with traversal,
//! conflict detection, temporal analysis, and multi-format export.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use borrowscope_graph::*;
//!
//! // Build from runtime events
//! let graph = OwnershipGraph::from_runtime();
//!
//! // Or build manually
//! let mut graph = OwnershipGraph::new();
//! let x = graph.add_variable("x", "Vec<i32>", 0);
//! let r = graph.add_variable("r", "&Vec<i32>", 10);
//! graph.add_borrow(r, x, false, 10);
//!
//! // Traverse
//! let order = traversal::dfs(&graph, r, Direction::Outgoing);
//!
//! // Detect conflicts
//! let conflicts = conflict::find_conflicts(&graph);
//!
//! // Export to Graphviz DOT
//! let dot = export::to_dot(&graph, &export::DotOptions::default());
//! ```
//!
//! ## Modules
//!
//! - [`builder`] - Graph construction from event streams (batch and streaming)
//! - [`traversal`] - DFS, BFS, shortest path, topological order, reachability
//! - [`conflict`] - Borrow conflict detection, cycle detection, validation
//! - [`temporal`] - Lifetime spans, ownership timelines, reference count history
//! - [`stats`] - Graph statistics, hotspot detection, borrow frequency
//! - [`export`] - JSON, DOT, MessagePack, D3.js export and import
//! - [`analyzer`] - Integration with borrowscope-analyzer's type-info.json

pub mod analyzer;
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
