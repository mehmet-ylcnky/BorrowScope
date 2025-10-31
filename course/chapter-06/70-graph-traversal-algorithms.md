# Section 70: Graph Traversal Algorithms

## Learning Objectives

By the end of this section, you will:
- Implement DFS and BFS for ownership graphs
- Detect cycles in the graph
- Perform topological sorting
- Find connected components
- Apply algorithms to ownership analysis

## Prerequisites

- Section 69 (Implementing Graph Construction)
- Understanding of graph algorithms
- Familiarity with recursion and queues

---

## Why Traversal Matters

**Use cases:**
- Find all variables transitively borrowed
- Detect invalid ownership patterns (cycles)
- Determine drop order (topological sort)
- Group related variables (connected components)

---

## Depth-First Search (DFS)

### Algorithm

Visit nodes deeply before backtracking.

```rust
use petgraph::visit::Dfs;
use petgraph::stable_graph::NodeIndex;

impl OwnershipGraph {
    pub fn dfs_from(&self, start_id: usize) -> Vec<usize> {
        let start_node = match self.id_to_node.get(&start_id) {
            Some(&node) => node,
            None => return vec![],
        };
        
        let mut dfs = Dfs::new(&self.graph, start_node);
        let mut visited = Vec::new();
        
        while let Some(node) = dfs.next(&self.graph) {
            if let Some(var) = self.graph.node_weight(node) {
                visited.push(var.id);
            }
        }
        
        visited
    }
}
```

### Example

```rust
// let x = 42;
// let r1 = &x;
// let r2 = &r1;

let visited = graph.dfs_from(1);  // Start from x
// visited = [1, 2, 3] or [1, 3, 2] (order depends on edge insertion)
```

### Use Case: Find All Dependent Variables

```rust
impl OwnershipGraph {
    pub fn find_all_borrowers(&self, id: usize) -> Vec<usize> {
        let node = match self.id_to_node.get(&id) {
            Some(&n) => n,
            None => return vec![],
        };
        
        let mut dfs = Dfs::new(&self.graph, node);
        let mut borrowers = Vec::new();
        
        while let Some(n) = dfs.next(&self.graph) {
            if n != node {  // Exclude the starting node
                if let Some(var) = self.graph.node_weight(n) {
                    borrowers.push(var.id);
                }
            }
        }
        
        borrowers
    }
}
```

---

## Breadth-First Search (BFS)

### Algorithm

Visit nodes level by level.

```rust
use petgraph::visit::Bfs;

impl OwnershipGraph {
    pub fn bfs_from(&self, start_id: usize) -> Vec<usize> {
        let start_node = match self.id_to_node.get(&start_id) {
            Some(&node) => node,
            None => return vec![],
        };
        
        let mut bfs = Bfs::new(&self.graph, start_node);
        let mut visited = Vec::new();
        
        while let Some(node) = bfs.next(&self.graph) {
            if let Some(var) = self.graph.node_weight(node) {
                visited.push(var.id);
            }
        }
        
        visited
    }
}
```

### Use Case: Shortest Borrow Chain

```rust
impl OwnershipGraph {
    pub fn shortest_path(&self, from_id: usize, to_id: usize) -> Option<Vec<usize>> {
        use std::collections::{VecDeque, HashMap};
        
        let from_node = *self.id_to_node.get(&from_id)?;
        let to_node = *self.id_to_node.get(&to_id)?;
        
        let mut queue = VecDeque::new();
        let mut parent: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        
        queue.push_back(from_node);
        
        while let Some(current) = queue.pop_front() {
            if current == to_node {
                // Reconstruct path
                let mut path = vec![to_id];
                let mut node = to_node;
                
                while let Some(&p) = parent.get(&node) {
                    if let Some(var) = self.graph.node_weight(p) {
                        path.push(var.id);
                    }
                    node = p;
                }
                
                path.reverse();
                return Some(path);
            }
            
            for neighbor in self.graph.neighbors(current) {
                if !parent.contains_key(&neighbor) && neighbor != from_node {
                    parent.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }
        
        None
    }
}
```

---

## Cycle Detection

### Why It Matters

Rust's borrow checker prevents cycles, but detecting them helps validate our tracking.

```rust
use petgraph::algo::is_cyclic_directed;

impl OwnershipGraph {
    pub fn has_cycle(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }
    
    pub fn find_cycle(&self) -> Option<Vec<usize>> {
        use petgraph::visit::DfsPostOrder;
        use std::collections::HashSet;
        
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for node in self.graph.node_indices() {
            if !visited.contains(&node) {
                if let Some(cycle) = self.dfs_cycle(node, &mut visited, &mut rec_stack) {
                    return Some(cycle);
                }
            }
        }
        
        None
    }
    
    fn dfs_cycle(&self, node: NodeIndex, visited: &mut HashSet<NodeIndex>, 
                 rec_stack: &mut HashSet<NodeIndex>) -> Option<Vec<usize>> {
        visited.insert(node);
        rec_stack.insert(node);
        
        for neighbor in self.graph.neighbors(node) {
            if !visited.contains(&neighbor) {
                if let Some(cycle) = self.dfs_cycle(neighbor, visited, rec_stack) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(&neighbor) {
                // Cycle detected
                let mut cycle = vec![];
                if let Some(var) = self.graph.node_weight(neighbor) {
                    cycle.push(var.id);
                }
                if let Some(var) = self.graph.node_weight(node) {
                    cycle.push(var.id);
                }
                return Some(cycle);
            }
        }
        
        rec_stack.remove(&node);
        None
    }
}
```

---

## Topological Sort

### Algorithm

Order nodes such that for every edge u → v, u comes before v.

```rust
use petgraph::algo::toposort;

impl OwnershipGraph {
    pub fn topological_order(&self) -> Result<Vec<usize>, String> {
        match toposort(&self.graph, None) {
            Ok(order) => {
                Ok(order.into_iter()
                    .filter_map(|node| self.graph.node_weight(node))
                    .map(|var| var.id)
                    .collect())
            }
            Err(_) => Err("Graph contains cycles".to_string()),
        }
    }
}
```

### Use Case: Drop Order

```rust
impl OwnershipGraph {
    pub fn drop_order(&self) -> Vec<usize> {
        self.topological_order()
            .unwrap_or_else(|_| {
                // Fallback: reverse creation order
                let mut vars: Vec<_> = self.graph.node_weights().collect();
                vars.sort_by_key(|v| v.created_at);
                vars.into_iter().map(|v| v.id).rev().collect()
            })
    }
}
```

**Example:**
```rust
// let x = 42;
// let r = &x;
// Drop order: [r, x] (r must drop before x)

let order = graph.drop_order();
assert_eq!(order, vec![2, 1]);  // r (id=2), then x (id=1)
```

---

## Connected Components

### Algorithm

Find groups of connected variables.

```rust
use petgraph::algo::kosaraju_scc;

impl OwnershipGraph {
    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        kosaraju_scc(&self.graph)
            .into_iter()
            .map(|component| {
                component.into_iter()
                    .filter_map(|node| self.graph.node_weight(node))
                    .map(|var| var.id)
                    .collect()
            })
            .collect()
    }
}
```

**Example:**
```rust
// let x = 42;
// let r = &x;
// let y = 10;

let components = graph.connected_components();
// [[1, 2], [3]]  // x and r are connected, y is separate
```

---

## Reachability

### Can A Reach B?

```rust
impl OwnershipGraph {
    pub fn can_reach(&self, from_id: usize, to_id: usize) -> bool {
        let from_node = match self.id_to_node.get(&from_id) {
            Some(&n) => n,
            None => return false,
        };
        
        let to_node = match self.id_to_node.get(&to_id) {
            Some(&n) => n,
            None => return false,
        };
        
        let mut dfs = Dfs::new(&self.graph, from_node);
        
        while let Some(node) = dfs.next(&self.graph) {
            if node == to_node {
                return true;
            }
        }
        
        false
    }
}
```

---

## Practical Examples

### Example 1: Find All Variables Affected by Drop

```rust
impl OwnershipGraph {
    pub fn affected_by_drop(&self, id: usize) -> Vec<usize> {
        // All variables that borrow (directly or transitively) from this one
        self.find_all_borrowers(id)
    }
}
```

**Usage:**
```rust
// let x = 42;
// let r1 = &x;
// let r2 = &r1;

let affected = graph.affected_by_drop(1);  // Drop x
// affected = [2, 3]  // r1 and r2 are affected
```

### Example 2: Validate Graph Integrity

```rust
impl OwnershipGraph {
    pub fn validate(&self) -> Result<(), String> {
        // Check for cycles
        if self.has_cycle() {
            return Err("Graph contains cycles (invalid ownership)".to_string());
        }
        
        // Check for orphaned nodes
        for node in self.graph.node_indices() {
            let in_degree = self.graph.neighbors_directed(node, Direction::Incoming).count();
            let out_degree = self.graph.neighbors(node).count();
            
            if in_degree == 0 && out_degree == 0 {
                if let Some(var) = self.graph.node_weight(node) {
                    if var.dropped_at.is_none() {
                        // Orphaned node that's still alive - might be root variable
                        continue;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### Example 3: Borrow Depth

```rust
impl OwnershipGraph {
    pub fn borrow_depth(&self, id: usize) -> usize {
        let node = match self.id_to_node.get(&id) {
            Some(&n) => n,
            None => return 0,
        };
        
        let mut max_depth = 0;
        let mut stack = vec![(node, 0)];
        let mut visited = std::collections::HashSet::new();
        
        while let Some((current, depth)) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            
            max_depth = max_depth.max(depth);
            
            for neighbor in self.graph.neighbors(current) {
                stack.push((neighbor, depth + 1));
            }
        }
        
        max_depth
    }
}
```

**Example:**
```rust
// let x = 42;
// let r1 = &x;
// let r2 = &r1;

assert_eq!(graph.borrow_depth(1), 2);  // x -> r1 -> r2
assert_eq!(graph.borrow_depth(2), 1);  // r1 -> r2
assert_eq!(graph.borrow_depth(3), 0);  // r2 (leaf)
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable { id: 1, name: "x".into(), 
            type_name: "i32".into(), created_at: 1000, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 2, name: "r1".into(), 
            type_name: "&i32".into(), created_at: 1050, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 3, name: "r2".into(), 
            type_name: "&&i32".into(), created_at: 1100, dropped_at: None, scope_depth: 0 });
        
        graph.add_borrow(2, 1, false, 1050);
        graph.add_borrow(3, 2, false, 1100);
        
        let visited = graph.dfs_from(1);
        assert_eq!(visited.len(), 3);
        assert!(visited.contains(&1));
        assert!(visited.contains(&2));
        assert!(visited.contains(&3));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable { id: 1, name: "x".into(), 
            type_name: "i32".into(), created_at: 1000, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 2, name: "r".into(), 
            type_name: "&i32".into(), created_at: 1050, dropped_at: None, scope_depth: 0 });
        
        graph.add_borrow(2, 1, false, 1050);
        
        let order = graph.topological_order().unwrap();
        let r_pos = order.iter().position(|&id| id == 2).unwrap();
        let x_pos = order.iter().position(|&id| id == 1).unwrap();
        
        assert!(r_pos < x_pos);  // r must come before x in drop order
    }

    #[test]
    fn test_no_cycles() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable { id: 1, name: "x".into(), 
            type_name: "i32".into(), created_at: 1000, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 2, name: "r".into(), 
            type_name: "&i32".into(), created_at: 1050, dropped_at: None, scope_depth: 0 });
        
        graph.add_borrow(2, 1, false, 1050);
        
        assert!(!graph.has_cycle());
    }
}
```

---

## Key Takeaways

✅ **DFS** - Deep exploration, find all dependencies  
✅ **BFS** - Level-by-level, shortest paths  
✅ **Cycle detection** - Validate ownership integrity  
✅ **Topological sort** - Determine drop order  
✅ **Connected components** - Group related variables  

---

## Further Reading

- [Graph traversal algorithms](https://en.wikipedia.org/wiki/Graph_traversal)
- [Topological sorting](https://en.wikipedia.org/wiki/Topological_sorting)
- [Strongly connected components](https://en.wikipedia.org/wiki/Strongly_connected_component)

---

**Previous:** [69-implementing-graph-construction.md](./69-implementing-graph-construction.md)  
**Next:** [71-detecting-borrow-conflicts.md](./71-detecting-borrow-conflicts.md)

**Progress:** 5/10 ⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜
