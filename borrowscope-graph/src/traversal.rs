//! Graph traversal algorithms for ownership analysis.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::edge::EdgeKind;
use crate::graph::{Direction, OwnershipGraph};
use crate::node::NodeId;

// ═══════════════════════════════════════════════════════════════════════════
// 2.1 DFS
// ═══════════════════════════════════════════════════════════════════════════

/// Depth-first search from a starting node. Returns nodes in visit order (pre-order).
pub fn dfs(graph: &OwnershipGraph, start: NodeId, direction: Direction) -> Vec<NodeId> {
    let mut visited = HashSet::new();
    let mut stack = vec![start];
    let mut order = Vec::new();

    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        order.push(node);
        for neighbor in graph.neighbors_directed(node, direction) {
            if !visited.contains(&neighbor) {
                stack.push(neighbor);
            }
        }
    }
    order
}

/// DFS with early termination. Returns the path to the target if found.
pub fn dfs_until(
    graph: &OwnershipGraph,
    start: NodeId,
    direction: Direction,
    target: NodeId,
) -> Option<Vec<NodeId>> {
    dfs_find(graph, start, direction, |n| n == target)
}

/// DFS with a predicate. Returns the path to the first node satisfying the predicate.
pub fn dfs_find(
    graph: &OwnershipGraph,
    start: NodeId,
    direction: Direction,
    predicate: impl Fn(NodeId) -> bool,
) -> Option<Vec<NodeId>> {
    let mut visited = HashSet::new();
    let mut stack = vec![(start, vec![start])];

    while let Some((node, path)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if predicate(node) && path.len() > 1 {
            return Some(path);
        }
        for neighbor in graph.neighbors_directed(node, direction) {
            if !visited.contains(&neighbor) {
                let mut new_path = path.clone();
                new_path.push(neighbor);
                if predicate(neighbor) {
                    return Some(new_path);
                }
                stack.push((neighbor, new_path));
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.2 BFS
// ═══════════════════════════════════════════════════════════════════════════

/// Breadth-first search returning nodes with their distance from start.
pub fn bfs(graph: &OwnershipGraph, start: NodeId, direction: Direction) -> Vec<(NodeId, u32)> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((node, dist)) = queue.pop_front() {
        result.push((node, dist));
        for neighbor in graph.neighbors_directed(node, direction) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, dist + 1));
            }
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.3 Shortest path
// ═══════════════════════════════════════════════════════════════════════════

/// Find the shortest path between two nodes. Returns None if unreachable.
pub fn shortest_path(graph: &OwnershipGraph, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
    if from == to {
        return Some(vec![from]);
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<NodeId, NodeId> = HashMap::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(node) = queue.pop_front() {
        for neighbor in graph.neighbors_directed(node, Direction::Both) {
            if visited.insert(neighbor) {
                parent.insert(neighbor, node);
                if neighbor == to {
                    // Reconstruct path
                    let mut path = vec![to];
                    let mut current = to;
                    while let Some(&p) = parent.get(&current) {
                        path.push(p);
                        current = p;
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(neighbor);
            }
        }
    }
    None
}

/// Shortest path with edge IDs along the path.
/// Returns pairs of (NodeId, Option<EdgeId>) where the EdgeId connects to the next node.
pub fn shortest_path_with_edges(
    graph: &OwnershipGraph,
    from: NodeId,
    to: NodeId,
) -> Option<Vec<(NodeId, Option<crate::edge::EdgeId>)>> {
    let path = shortest_path(graph, from, to)?;
    let mut result = Vec::new();

    for i in 0..path.len() {
        let edge_id = if i + 1 < path.len() {
            // Find edge between path[i] and path[i+1]
            let a = path[i];
            let b = path[i + 1];
            graph.outgoing_edges(a)
                .iter()
                .find(|eid| graph.get_edge(**eid).map_or(false, |e| e.target == b))
                .or_else(|| {
                    graph.incoming_edges(a)
                        .iter()
                        .find(|eid| graph.get_edge(**eid).map_or(false, |e| e.source == b))
                })
                .copied()
        } else {
            None
        };
        result.push((path[i], edge_id));
    }
    Some(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.4 Topological ordering (drop order)
// ═══════════════════════════════════════════════════════════════════════════

/// Error returned when a cycle is detected during topological sort.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleError {
    /// Nodes involved in the cycle.
    pub cycle: Vec<NodeId>,
}

/// Compute topological order using Kahn's algorithm.
/// Borrow edges define ordering: borrower must appear before owner.
/// Returns Err if the graph contains cycles.
pub fn topological_order(graph: &OwnershipGraph) -> Result<Vec<NodeId>, CycleError> {
    // Build in-degree map (only for borrow/move edges that define ordering)
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for node in graph.nodes() {
        in_degree.entry(node.id()).or_insert(0);
    }

    for edge in graph.edges() {
        // source -> target means source depends on target (source borrows from target)
        // In topo order: source (borrower) comes before target (owner) in drop order
        match &edge.kind {
            EdgeKind::BorrowShared | EdgeKind::BorrowMut | EdgeKind::RefCellBorrow { .. } => {
                // borrower (source) must be dropped before owner (target)
                adj.entry(edge.target).or_default().push(edge.source);
                *in_degree.entry(edge.source).or_insert(0) += 1;
            }
            _ => {}
        }
    }

    let mut queue: VecDeque<NodeId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut order = Vec::new();

    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(neighbors) = adj.get(&node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(&neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if order.len() != in_degree.len() {
        // Cycle detected: remaining nodes with in_degree > 0 form the cycle
        let cycle: Vec<NodeId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg > 0)
            .map(|(&id, _)| id)
            .collect();
        Err(CycleError { cycle })
    } else {
        Ok(order)
    }
}

/// Drop order: borrowers are dropped before owners.
/// This is the reverse of the dependency order.
pub fn drop_order(graph: &OwnershipGraph) -> Result<Vec<NodeId>, CycleError> {
    let mut order = topological_order(graph)?;
    order.reverse();
    Ok(order)
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.5 Reachability
// ═══════════════════════════════════════════════════════════════════════════

/// Check if `to` is reachable from `from` following edges in the given direction.
pub fn can_reach(graph: &OwnershipGraph, from: NodeId, to: NodeId) -> bool {
    if from == to {
        return true;
    }
    let visited = dfs(graph, from, Direction::Outgoing);
    visited.contains(&to)
}

/// Get all nodes reachable from `start`.
pub fn all_reachable(graph: &OwnershipGraph, start: NodeId, direction: Direction) -> HashSet<NodeId> {
    dfs(graph, start, direction).into_iter().collect()
}

/// Check if two nodes are in the same connected component (undirected).
pub fn are_connected(graph: &OwnershipGraph, a: NodeId, b: NodeId) -> bool {
    let visited = dfs(graph, a, Direction::Both);
    visited.contains(&b)
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.6 Connected components
// ═══════════════════════════════════════════════════════════════════════════

/// Find all connected components (treating edges as undirected).
pub fn connected_components(graph: &OwnershipGraph) -> Vec<Vec<NodeId>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    for node in graph.nodes() {
        let id = node.id();
        if !visited.contains(&id) {
            let component = dfs(graph, id, Direction::Both);
            for &n in &component {
                visited.insert(n);
            }
            components.push(component);
        }
    }
    components
}

/// Number of connected components.
pub fn component_count(graph: &OwnershipGraph) -> usize {
    connected_components(graph).len()
}

/// Get the component containing a specific node.
pub fn component_of(graph: &OwnershipGraph, node: NodeId) -> Vec<NodeId> {
    dfs(graph, node, Direction::Both)
}

// ═══════════════════════════════════════════════════════════════════════════
// 2.7 Borrow chain and depth
// ═══════════════════════════════════════════════════════════════════════════

/// Get the chain of borrows from a variable back to its root owner.
/// Returns [variable, ..., root_owner].
pub fn borrow_chain(graph: &OwnershipGraph, node: NodeId) -> Vec<NodeId> {
    let mut chain = vec![node];
    let mut current = node;

    loop {
        // Find outgoing borrow edge (borrower -> owner)
        let owner = graph.outgoing_edges(current)
            .iter()
            .filter_map(|eid| graph.get_edge(*eid))
            .find(|e| e.is_borrow())
            .map(|e| e.target);

        match owner {
            Some(o) if !chain.contains(&o) => {
                chain.push(o);
                current = o;
            }
            _ => break,
        }
    }
    chain
}

/// Depth of borrow nesting (0 = owner, 1 = direct borrow, 2 = borrow of borrow).
pub fn borrow_depth(graph: &OwnershipGraph, node: NodeId) -> u32 {
    let chain = borrow_chain(graph, node);
    (chain.len() - 1) as u32
}

/// Find the root owner of a borrowed variable.
pub fn root_owner(graph: &OwnershipGraph, node: NodeId) -> NodeId {
    let chain = borrow_chain(graph, node);
    *chain.last().unwrap()
}
