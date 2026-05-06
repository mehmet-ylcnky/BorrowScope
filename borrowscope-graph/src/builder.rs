//! Graph construction from runtime event streams.

use std::collections::HashMap;

use borrowscope_runtime::Event;

use crate::edge::{CaptureMode, EdgeId};
use crate::graph::OwnershipGraph;
use crate::node::NodeId;

/// Result of processing a single event.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphUpdate {
    NodeAdded(NodeId),
    EdgeAdded(EdgeId),
    NodeDropped(NodeId),
    EdgeEnded(EdgeId),
    NoOp,
}

/// Streaming graph builder that processes events one at a time.
pub struct GraphStream {
    graph: OwnershipGraph,
    /// Maps runtime var_id strings to graph NodeIds
    var_ids: HashMap<String, NodeId>,
    /// Maps runtime var_id to active borrow EdgeIds
    active_borrows: HashMap<String, Vec<EdgeId>>,
    /// Callbacks fired on each graph update
    callbacks: Vec<Box<dyn Fn(&GraphUpdate)>>,
}

impl GraphStream {
    /// Create a new streaming graph builder.
    pub fn new() -> Self {
        Self {
            graph: OwnershipGraph::new(),
            var_ids: HashMap::new(),
            active_borrows: HashMap::new(),
            callbacks: Vec::new(),
        }
    }

    /// Register a callback that fires on each graph update.
    pub fn on_update(&mut self, callback: impl Fn(&GraphUpdate) + 'static) {
        self.callbacks.push(Box::new(callback));
    }

    /// Process all events currently in the runtime buffer.
    pub fn drain_runtime(&mut self) -> Vec<GraphUpdate> {
        let events = borrowscope_runtime::get_events();
        let updates: Vec<GraphUpdate> = events.iter().map(|e| self.push(e)).collect();
        updates
    }

    /// Process a single event, updating the graph incrementally.
    pub fn push(&mut self, event: &Event) -> GraphUpdate {
        match event {
            Event::New {
                var_id,
                var_name,
                type_name,
                timestamp,
                ..
            } => {
                let id = self.graph.add_variable(var_name, type_name, *timestamp);
                self.var_ids.insert(var_id.clone(), id);
                GraphUpdate::NodeAdded(id)
            }
            Event::Borrow {
                borrower_id,
                owner_id,
                mutable,
                timestamp,
                ..
            } => {
                if let (Some(&borrower), Some(&owner)) =
                    (self.var_ids.get(borrower_id), self.var_ids.get(owner_id))
                {
                    let eid = self.graph.add_borrow(borrower, owner, *mutable, *timestamp);
                    self.active_borrows
                        .entry(borrower_id.clone())
                        .or_default()
                        .push(eid);
                    GraphUpdate::EdgeAdded(eid)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::Move {
                from_id,
                to_name,
                to_id,
                timestamp,
                ..
            } => {
                if let Some(&from) = self.var_ids.get(from_id) {
                    let to = self.graph.add_variable(to_name, "", *timestamp);
                    self.var_ids.insert(to_id.clone(), to);
                    let eid = self.graph.add_move(from, to, *timestamp);
                    GraphUpdate::EdgeAdded(eid)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::Drop {
                var_id, timestamp, ..
            } => {
                if let Some(&id) = self.var_ids.get(var_id) {
                    self.graph.mark_dropped(id, *timestamp);
                    // End all borrows held by this variable
                    if let Some(edges) = self.active_borrows.remove(var_id) {
                        for eid in &edges {
                            self.graph.end_edge(*eid, *timestamp);
                        }
                    }
                    GraphUpdate::NodeDropped(id)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::RcClone {
                var_id,
                var_name,
                source_id,
                strong_count,
                timestamp,
                ..
            } => {
                let clone_id = if let Some(&id) = self.var_ids.get(var_id) {
                    id
                } else {
                    let id = self.graph.add_variable(var_name, "Rc", *timestamp);
                    self.var_ids.insert(var_id.clone(), id);
                    id
                };
                if let Some(&src) = self.var_ids.get(source_id) {
                    let eid =
                        self.graph
                            .add_rc_clone(clone_id, src, *strong_count as u32, *timestamp);
                    GraphUpdate::EdgeAdded(eid)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::ArcClone {
                var_id,
                var_name,
                source_id,
                strong_count,
                timestamp,
                ..
            } => {
                let clone_id = if let Some(&id) = self.var_ids.get(var_id) {
                    id
                } else {
                    let id = self.graph.add_variable(var_name, "Arc", *timestamp);
                    self.var_ids.insert(var_id.clone(), id);
                    id
                };
                if let Some(&src) = self.var_ids.get(source_id) {
                    let eid =
                        self.graph
                            .add_arc_clone(clone_id, src, *strong_count as u32, *timestamp);
                    GraphUpdate::EdgeAdded(eid)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::ClosureCapture {
                closure_id,
                var_name,
                capture_mode,
                timestamp,
                ..
            } => {
                if let (Some(&closure), Some(&var)) =
                    (self.var_ids.get(closure_id), self.var_ids.get(var_name))
                {
                    let mode = match capture_mode.as_str() {
                        "by_mut_ref" => CaptureMode::ByMutRef,
                        "by_move" => CaptureMode::ByMove,
                        _ => CaptureMode::ByRef,
                    };
                    let eid = self.graph.add_capture(closure, var, mode, *timestamp);
                    GraphUpdate::EdgeAdded(eid)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::FnEnter {
                fn_id,
                fn_name,
                timestamp,
                ..
            } => {
                let id =
                    self.graph
                        .add_scope(fn_name, crate::node::ScopeKind::Function, *timestamp);
                self.var_ids.insert(fn_id.clone(), id);
                GraphUpdate::NodeAdded(id)
            }
            Event::FnExit {
                fn_id, timestamp, ..
            } => {
                if let Some(&id) = self.var_ids.get(fn_id) {
                    self.graph.mark_dropped(id, *timestamp);
                    GraphUpdate::NodeDropped(id)
                } else {
                    GraphUpdate::NoOp
                }
            }
            Event::RegionEnter {
                region_id,
                name,
                timestamp,
                ..
            } => {
                let id = self
                    .graph
                    .add_scope(name, crate::node::ScopeKind::Block, *timestamp);
                self.var_ids.insert(region_id.clone(), id);
                GraphUpdate::NodeAdded(id)
            }
            Event::RegionExit {
                region_id,
                timestamp,
                ..
            } => {
                if let Some(&id) = self.var_ids.get(region_id) {
                    self.graph.mark_dropped(id, *timestamp);
                    GraphUpdate::NodeDropped(id)
                } else {
                    GraphUpdate::NoOp
                }
            }
            _ => GraphUpdate::NoOp,
        }
    }

    /// Process all events from a slice.
    pub fn push_all(&mut self, events: &[Event]) -> Vec<GraphUpdate> {
        events
            .iter()
            .map(|e| {
                let update = self.push(e);
                for cb in &self.callbacks {
                    cb(&update);
                }
                update
            })
            .collect()
    }

    /// Get a reference to the current graph state.
    pub fn graph(&self) -> &OwnershipGraph {
        &self.graph
    }

    /// Consume the stream, returning the final graph.
    pub fn into_graph(self) -> OwnershipGraph {
        self.graph
    }
}

impl Default for GraphStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a graph from a complete event stream (batch mode).
pub fn from_events(events: &[Event]) -> OwnershipGraph {
    let mut stream = GraphStream::new();
    stream.push_all(events);
    stream.into_graph()
}

impl OwnershipGraph {
    /// Build a graph from a runtime event stream.
    pub fn from_events(events: &[Event]) -> Self {
        from_events(events)
    }

    /// Build graph from the global runtime event buffer.
    pub fn from_runtime() -> Self {
        let events = borrowscope_runtime::get_events();
        Self::from_events(&events)
    }

    /// Build graph from runtime events matching a predicate.
    pub fn from_runtime_filtered(predicate: impl Fn(&Event) -> bool) -> Self {
        let events: Vec<_> = borrowscope_runtime::get_events()
            .into_iter()
            .filter(predicate)
            .collect();
        Self::from_events(&events)
    }

    /// Build graph from events for a specific variable.
    pub fn from_runtime_for_var(name: &str) -> Self {
        let name = name.to_string();
        Self::from_runtime_filtered(|e| {
            e.var_name().map_or(false, |n| n == name)
                || match e {
                    Event::Borrow { owner_id, .. } => owner_id == &name,
                    Event::Move { from_id, .. } => from_id == &name,
                    _ => false,
                }
        })
    }
}
