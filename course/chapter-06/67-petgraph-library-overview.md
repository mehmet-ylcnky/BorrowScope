# Section 67: Petgraph Library Overview

## Learning Objectives

By the end of this section, you will:
- Understand petgraph's graph types
- Choose the right graph structure for BorrowScope
- Learn petgraph's API patterns
- Implement basic graph operations
- Prepare for ownership graph implementation

## Prerequisites

- Section 66 (Graph Theory Basics)
- Understanding of generics and traits
- Familiarity with iterators

---

## What is Petgraph?

**petgraph** is Rust's most popular graph data structure library.

```toml
[dependencies]
petgraph = "0.6"
```

**Features:**
- Multiple graph types (Graph, StableGraph, GraphMap)
- Generic over node/edge data
- Rich algorithm library
- Efficient implementations
- Serialization support

---

## Graph Types

### 1. Graph<N, E, Ty>

Standard graph with compact node indices.

```rust
use petgraph::graph::{Graph, NodeIndex};

let mut graph = Graph::<&str, i32>::new();
let a = graph.add_node("A");
let b = graph.add_node("B");
graph.add_edge(a, b, 10);
```

**Type parameters:**
- `N`: Node weight type
- `E`: Edge weight type
- `Ty`: `Directed` or `Undirected`

**Pros:** Fast, memory efficient  
**Cons:** Node removal invalidates indices

### 2. StableGraph<N, E, Ty>

Graph with stable indices after removal.

```rust
use petgraph::stable_graph::StableGraph;

let mut graph = StableGraph::<&str, i32>::new();
let a = graph.add_node("A");
graph.remove_node(a);  // Index 'a' stays reserved
```

**Pros:** Stable indices, safe removal  
**Cons:** Slightly slower, more memory

### 3. GraphMap<N, E, Ty>

Graph using node values as keys (no separate indices).

```rust
use petgraph::graphmap::DiGraphMap;

let mut graph = DiGraphMap::<&str, i32>::new();
graph.add_edge("A", "B", 10);
```

**Pros:** Simple API, no index management  
**Cons:** Node type must be `Copy + Ord + Hash`

---

## Choosing for BorrowScope

**Requirements:**
- Directed graph (ownership has direction)
- Node removal (when variables drop)
- Stable indices (track variables by ID)
- Custom node/edge data

**Choice:** `StableGraph<Variable, Relationship, Directed>`

```rust
use petgraph::stable_graph::StableGraph;
use petgraph::Directed;

type OwnershipGraph = StableGraph<Variable, Relationship, Directed>;
```

---

## Basic Operations

### Creating a Graph

```rust
use petgraph::graph::DiGraph;

// Directed graph
let mut graph = DiGraph::<String, ()>::new();

// With capacity
let mut graph = DiGraph::with_capacity(100, 200);
```

### Adding Nodes

```rust
let node_a = graph.add_node("A".to_string());
let node_b = graph.add_node("B".to_string());

// NodeIndex is returned
println!("Node A: {:?}", node_a);  // NodeIndex(0)
```

### Adding Edges

```rust
let edge = graph.add_edge(node_a, node_b, ());

// EdgeIndex is returned
println!("Edge: {:?}", edge);  // EdgeIndex(0)
```

### Accessing Data

```rust
// Node weight
if let Some(weight) = graph.node_weight(node_a) {
    println!("Node A: {}", weight);
}

// Edge weight
if let Some(weight) = graph.edge_weight(edge) {
    println!("Edge weight: {:?}", weight);
}

// Mutable access
if let Some(weight) = graph.node_weight_mut(node_a) {
    *weight = "Modified".to_string();
}
```

### Removing Elements

```rust
// Remove edge
graph.remove_edge(edge);

// Remove node (and all connected edges)
graph.remove_node(node_a);
```

---

## Traversal

### Neighbors

```rust
use petgraph::Direction;

// Outgoing edges (successors)
for neighbor in graph.neighbors(node_a) {
    println!("Neighbor: {:?}", neighbor);
}

// Incoming edges (predecessors)
for neighbor in graph.neighbors_directed(node_a, Direction::Incoming) {
    println!("Predecessor: {:?}", neighbor);
}
```

### Edges

```rust
// Outgoing edges
for edge in graph.edges(node_a) {
    println!("Edge to {:?} with weight {:?}", edge.target(), edge.weight());
}

// All edges
for edge in graph.edge_indices() {
    if let Some((source, target)) = graph.edge_endpoints(edge) {
        println!("{:?} -> {:?}", source, target);
    }
}
```

### All Nodes

```rust
for node in graph.node_indices() {
    if let Some(weight) = graph.node_weight(node) {
        println!("Node {:?}: {}", node, weight);
    }
}
```

---

## Algorithms

Petgraph includes many graph algorithms.

### DFS (Depth-First Search)

```rust
use petgraph::visit::Dfs;

let mut dfs = Dfs::new(&graph, start_node);
while let Some(node) = dfs.next(&graph) {
    println!("Visited: {:?}", node);
}
```

### BFS (Breadth-First Search)

```rust
use petgraph::visit::Bfs;

let mut bfs = Bfs::new(&graph, start_node);
while let Some(node) = bfs.next(&graph) {
    println!("Visited: {:?}", node);
}
```

### Topological Sort

```rust
use petgraph::algo::toposort;

match toposort(&graph, None) {
    Ok(order) => println!("Topological order: {:?}", order),
    Err(_) => println!("Graph has cycles!"),
}
```

### Cycle Detection

```rust
use petgraph::algo::is_cyclic_directed;

if is_cyclic_directed(&graph) {
    println!("Graph contains cycles");
}
```

### Shortest Path

```rust
use petgraph::algo::dijkstra;
use std::collections::HashMap;

let distances: HashMap<NodeIndex, i32> = dijkstra(&graph, start_node, None, |_| 1);
```

---

## Example: Building an Ownership Graph

```rust
use petgraph::stable_graph::StableGraph;
use petgraph::Directed;

#[derive(Debug, Clone)]
struct Variable {
    id: usize,
    name: String,
}

#[derive(Debug, Clone)]
enum Relationship {
    BorrowsImmut,
    BorrowsMut,
}

fn main() {
    let mut graph = StableGraph::<Variable, Relationship, Directed>::new();
    
    // let x = 42;
    let x = graph.add_node(Variable { id: 1, name: "x".into() });
    
    // let r1 = &x;
    let r1 = graph.add_node(Variable { id: 2, name: "r1".into() });
    graph.add_edge(r1, x, Relationship::BorrowsImmut);
    
    // let r2 = &x;
    let r2 = graph.add_node(Variable { id: 3, name: "r2".into() });
    graph.add_edge(r2, x, Relationship::BorrowsImmut);
    
    // Print all borrows of x
    println!("Variables borrowing x:");
    for neighbor in graph.neighbors_directed(x, petgraph::Direction::Incoming) {
        if let Some(var) = graph.node_weight(neighbor) {
            println!("  {}", var.name);
        }
    }
}
```

**Output:**
```
Variables borrowing x:
  r1
  r2
```

---

## Serialization

Petgraph supports serialization with serde.

```toml
[dependencies]
petgraph = { version = "0.6", features = ["serde-1"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
struct Variable {
    id: usize,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum Relationship {
    BorrowsImmut,
    BorrowsMut,
}

let json = serde_json::to_string(&graph)?;
let loaded: StableGraph<Variable, Relationship, Directed> = 
    serde_json::from_str(&json)?;
```

---

## Performance Characteristics

| Operation | Graph | StableGraph | GraphMap |
|-----------|-------|-------------|----------|
| Add node | O(1) | O(1) | O(1) |
| Add edge | O(1) | O(1) | O(log V) |
| Remove node | O(E) | O(1) | O(E) |
| Remove edge | O(E) | O(1) | O(log V) |
| Find edge | O(E) | O(E) | O(log V) |
| Neighbors | O(degree) | O(degree) | O(degree) |

**For BorrowScope:** StableGraph provides the best balance.

---

## Common Patterns

### Wrapper Struct

```rust
use petgraph::stable_graph::{StableGraph, NodeIndex};
use std::collections::HashMap;

pub struct OwnershipGraph {
    graph: StableGraph<Variable, Relationship, Directed>,
    id_to_node: HashMap<usize, NodeIndex>,
}

impl OwnershipGraph {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            id_to_node: HashMap::new(),
        }
    }
    
    pub fn add_variable(&mut self, var: Variable) -> NodeIndex {
        let node = self.graph.add_node(var.clone());
        self.id_to_node.insert(var.id, node);
        node
    }
    
    pub fn get_node(&self, id: usize) -> Option<NodeIndex> {
        self.id_to_node.get(&id).copied()
    }
}
```

### Iterator Patterns

```rust
impl OwnershipGraph {
    pub fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.graph.node_weights()
    }
    
    pub fn borrows_of(&self, id: usize) -> impl Iterator<Item = &Variable> + '_ {
        self.get_node(id)
            .into_iter()
            .flat_map(move |node| {
                self.graph.neighbors_directed(node, Direction::Incoming)
                    .filter_map(move |n| self.graph.node_weight(n))
            })
    }
}
```

---

## Key Takeaways

✅ **StableGraph** - Best for BorrowScope (stable indices)  
✅ **Generic types** - Customize node/edge data  
✅ **Rich API** - Traversal, algorithms, serialization  
✅ **Efficient** - O(1) operations for most use cases  
✅ **Wrapper pattern** - Encapsulate graph with helper methods  

---

## Further Reading

- [petgraph documentation](https://docs.rs/petgraph/)
- [petgraph GitHub](https://github.com/petgraph/petgraph)
- [Graph algorithms in Rust](https://depth-first.com/articles/2020/02/03/graphs-in-rust/)

---

**Previous:** [66-graph-theory-basics.md](./66-graph-theory-basics.md)  
**Next:** [68-designing-the-ownership-graph.md](./68-designing-the-ownership-graph.md)

**Progress:** 2/10 ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜
