//! The central ownership graph structure.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::edge::{CaptureMode, Edge, EdgeId, EdgeKind};
use crate::node::{Node, NodeId, ScopeKind, ScopeNode, VariableNode};

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Follow edges from source to target.
    Outgoing,
    /// Follow edges from target to source.
    Incoming,
    /// Follow edges in both directions.
    Both,
}

/// The complete ownership graph with adjacency indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    /// Node -> outgoing edge IDs
    #[serde(skip)]
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    /// Node -> incoming edge IDs
    #[serde(skip)]
    incoming: HashMap<NodeId, Vec<EdgeId>>,
    /// Name -> node IDs (for lookup by variable name)
    #[serde(skip)]
    name_index: HashMap<String, Vec<NodeId>>,
    next_node_id: usize,
    next_edge_id: usize,
}

impl OwnershipGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            name_index: HashMap::new(),
            next_node_id: 0,
            next_edge_id: 0,
        }
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get all edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    // ── Node operations ──

    /// Add a variable node. Returns its NodeId.
    pub fn add_variable(
        &mut self,
        name: &str,
        type_name: &str,
        created_at: u64,
    ) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        let node = VariableNode {
            id,
            name: name.to_string(),
            type_name: type_name.to_string(),
            created_at,
            dropped_at: None,
            scope_depth: 0,
            is_copy: false,
            is_mutable: false,
        };
        self.nodes.push(Node::Variable(node));
        self.name_index
            .entry(name.to_string())
            .or_default()
            .push(id);
        id
    }

    /// Add a scope node. Returns its NodeId.
    pub fn add_scope(
        &mut self,
        name: &str,
        kind: ScopeKind,
        entered_at: u64,
    ) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        let node = ScopeNode {
            id,
            name: name.to_string(),
            kind,
            entered_at,
            exited_at: None,
        };
        self.nodes.push(Node::Scope(node));
        self.name_index
            .entry(name.to_string())
            .or_default()
            .push(id);
        id
    }

    /// Mark a node as dropped/exited at the given timestamp.
    pub fn mark_dropped(&mut self, id: NodeId, timestamp: u64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id() == id) {
            match node {
                Node::Variable(v) => v.dropped_at = Some(timestamp),
                Node::Scope(s) => s.exited_at = Some(timestamp),
            }
        }
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    /// Find nodes by name.
    pub fn find_by_name(&self, name: &str) -> &[NodeId] {
        self.name_index
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // ── Edge operations ──

    fn add_edge(&mut self, source: NodeId, target: NodeId, kind: EdgeKind, created_at: u64) -> EdgeId {
        let id = EdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        let edge = Edge {
            id,
            source,
            target,
            kind,
            created_at,
            ended_at: None,
        };
        self.edges.push(edge);
        self.outgoing.entry(source).or_default().push(id);
        self.incoming.entry(target).or_default().push(id);
        id
    }

    /// Add a borrow edge (shared or mutable).
    pub fn add_borrow(&mut self, borrower: NodeId, owner: NodeId, mutable: bool, at: u64) -> EdgeId {
        let kind = if mutable { EdgeKind::BorrowMut } else { EdgeKind::BorrowShared };
        self.add_edge(borrower, owner, kind, at)
    }

    /// Add a move edge.
    pub fn add_move(&mut self, from: NodeId, to: NodeId, at: u64) -> EdgeId {
        self.add_edge(from, to, EdgeKind::Move, at)
    }

    /// Add an Rc clone edge.
    pub fn add_rc_clone(&mut self, clone: NodeId, source: NodeId, count: u32, at: u64) -> EdgeId {
        self.add_edge(clone, source, EdgeKind::RcClone { strong_count: count }, at)
    }

    /// Add an Arc clone edge.
    pub fn add_arc_clone(&mut self, clone: NodeId, source: NodeId, count: u32, at: u64) -> EdgeId {
        self.add_edge(clone, source, EdgeKind::ArcClone { strong_count: count }, at)
    }

    /// Add a closure capture edge.
    pub fn add_capture(&mut self, closure: NodeId, var: NodeId, mode: CaptureMode, at: u64) -> EdgeId {
        self.add_edge(closure, var, EdgeKind::ClosureCapture { capture_mode: mode }, at)
    }

    /// Add a channel send edge.
    pub fn add_channel_send(&mut self, sender: NodeId, receiver: NodeId, at: u64) -> EdgeId {
        self.add_edge(sender, receiver, EdgeKind::ChannelSend, at)
    }

    /// Add a RefCell borrow edge.
    pub fn add_refcell_borrow(&mut self, guard: NodeId, cell: NodeId, mutable: bool, at: u64) -> EdgeId {
        self.add_edge(guard, cell, EdgeKind::RefCellBorrow { mutable }, at)
    }

    /// Add a lock acquire edge.
    pub fn add_lock_acquire(&mut self, guard: NodeId, lock: NodeId, lock_type: &str, at: u64) -> EdgeId {
        self.add_edge(guard, lock, EdgeKind::LockAcquire { lock_type: lock_type.to_string() }, at)
    }

    /// Add a weak downgrade edge.
    pub fn add_weak_downgrade(&mut self, weak: NodeId, strong: NodeId, at: u64) -> EdgeId {
        self.add_edge(weak, strong, EdgeKind::WeakDowngrade, at)
    }

    /// End an edge at the given timestamp.
    pub fn end_edge(&mut self, id: EdgeId, timestamp: u64) {
        if let Some(edge) = self.edges.iter_mut().find(|e| e.id == id) {
            edge.ended_at = Some(timestamp);
        }
    }

    /// Get an edge by ID.
    pub fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == id)
    }

    // ── Query operations ──

    /// Get outgoing edge IDs for a node.
    pub fn outgoing_edges(&self, id: NodeId) -> &[EdgeId] {
        self.outgoing.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get incoming edge IDs for a node.
    pub fn incoming_edges(&self, id: NodeId) -> &[EdgeId] {
        self.incoming.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all neighbor node IDs (targets of outgoing edges).
    pub fn neighbors(&self, id: NodeId) -> Vec<NodeId> {
        self.outgoing_edges(id)
            .iter()
            .filter_map(|eid| self.get_edge(*eid))
            .map(|e| e.target)
            .collect()
    }

    /// Get neighbors in a specific direction.
    pub fn neighbors_directed(&self, id: NodeId, direction: Direction) -> Vec<NodeId> {
        match direction {
            Direction::Outgoing => {
                self.outgoing_edges(id)
                    .iter()
                    .filter_map(|eid| self.get_edge(*eid))
                    .map(|e| e.target)
                    .collect()
            }
            Direction::Incoming => {
                self.incoming_edges(id)
                    .iter()
                    .filter_map(|eid| self.get_edge(*eid))
                    .map(|e| e.source)
                    .collect()
            }
            Direction::Both => {
                let mut result: Vec<NodeId> = self.outgoing_edges(id)
                    .iter()
                    .filter_map(|eid| self.get_edge(*eid))
                    .map(|e| e.target)
                    .collect();
                for eid in self.incoming_edges(id) {
                    if let Some(e) = self.get_edge(*eid) {
                        if !result.contains(&e.source) {
                            result.push(e.source);
                        }
                    }
                }
                result
            }
        }
    }

    /// Get all nodes that borrow from the given node.
    pub fn borrowers_of(&self, id: NodeId) -> Vec<NodeId> {
        self.incoming_edges(id)
            .iter()
            .filter_map(|eid| self.get_edge(*eid))
            .filter(|e| e.is_borrow())
            .map(|e| e.source)
            .collect()
    }

    /// Find the owner of a borrowed variable (target of its outgoing borrow edge).
    pub fn owner_of(&self, id: NodeId) -> Option<NodeId> {
        self.outgoing_edges(id)
            .iter()
            .filter_map(|eid| self.get_edge(*eid))
            .find(|e| e.is_borrow())
            .map(|e| e.target)
    }

    // ── Incremental operations ──

    /// Remove a node and all its connected edges.
    pub fn remove_node(&mut self, id: NodeId) {
        // Collect edges to remove
        let edge_ids: Vec<EdgeId> = self.edges
            .iter()
            .filter(|e| e.source == id || e.target == id)
            .map(|e| e.id)
            .collect();

        for eid in edge_ids {
            self.remove_edge(eid);
        }

        self.nodes.retain(|n| n.id() != id);

        // Clean name index
        for ids in self.name_index.values_mut() {
            ids.retain(|nid| *nid != id);
        }
    }

    /// Remove a single edge.
    pub fn remove_edge(&mut self, id: EdgeId) {
        if let Some(edge) = self.edges.iter().find(|e| e.id == id) {
            let source = edge.source;
            let target = edge.target;

            if let Some(out) = self.outgoing.get_mut(&source) {
                out.retain(|eid| *eid != id);
            }
            if let Some(inc) = self.incoming.get_mut(&target) {
                inc.retain(|eid| *eid != id);
            }
        }
        self.edges.retain(|e| e.id != id);
    }

    /// Merge another graph into this one. Node and edge IDs are remapped.
    /// Returns a mapping from old NodeIds to new NodeIds.
    pub fn merge(&mut self, other: &OwnershipGraph) -> HashMap<NodeId, NodeId> {
        let mut id_map: HashMap<NodeId, NodeId> = HashMap::new();

        // Remap and add nodes
        for node in other.nodes() {
            let old_id = node.id();
            let new_id = match node {
                Node::Variable(v) => {
                    let nid = self.add_variable(&v.name, &v.type_name, v.created_at);
                    if let Some(dropped) = v.dropped_at {
                        self.mark_dropped(nid, dropped);
                    }
                    nid
                }
                Node::Scope(s) => {
                    let nid = self.add_scope(&s.name, s.kind.clone(), s.entered_at);
                    if let Some(exited) = s.exited_at {
                        self.mark_dropped(nid, exited);
                    }
                    nid
                }
            };
            id_map.insert(old_id, new_id);
        }

        // Remap and add edges
        for edge in other.edges() {
            if let (Some(&new_src), Some(&new_tgt)) =
                (id_map.get(&edge.source), id_map.get(&edge.target))
            {
                let eid = self.add_edge(new_src, new_tgt, edge.kind.clone(), edge.created_at);
                if let Some(ended) = edge.ended_at {
                    self.end_edge(eid, ended);
                }
            }
        }

        id_map
    }

    /// Rebuild internal indices after deserialization.
    pub fn rebuild_indices(&mut self) {
        self.outgoing.clear();
        self.incoming.clear();
        self.name_index.clear();

        for node in &self.nodes {
            self.name_index
                .entry(node.name().to_string())
                .or_default()
                .push(node.id());
        }
        for edge in &self.edges {
            self.outgoing.entry(edge.source).or_default().push(edge.id);
            self.incoming.entry(edge.target).or_default().push(edge.id);
        }
    }
}

impl Default for OwnershipGraph {
    fn default() -> Self {
        Self::new()
    }
}
