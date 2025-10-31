# Section 66: Graph Theory Basics

## Learning Objectives

By the end of this section, you will:
- Understand fundamental graph concepts
- Distinguish directed vs undirected graphs
- Learn graph representations (adjacency list, matrix)
- Apply graph theory to ownership tracking
- Prepare for petgraph implementation

## Prerequisites

- Completed Chapter 5 (Advanced Patterns)
- Basic understanding of data structures
- Familiarity with ownership concepts

---

## What is a Graph?

A graph G = (V, E) consists of:
- **V**: Set of vertices (nodes)
- **E**: Set of edges connecting vertices

```
Graph Example:
    A → B
    ↓   ↓
    C → D
```

---

## Directed vs Undirected Graphs

### Undirected Graph

Edges have no direction:

```
A — B
|   |
C — D
```

**Example:** Social network (friendship is mutual)

### Directed Graph (Digraph)

Edges have direction:

```
A → B
↓   ↓
C → D
```

**Example:** Ownership graph (x borrows y, but not vice versa)

**For BorrowScope:** We use **directed graphs** because ownership relationships have direction.

---

## Graph Representations

### Adjacency List

```rust
struct Graph {
    nodes: Vec<Node>,
    edges: HashMap<usize, Vec<usize>>,  // node_id -> [neighbor_ids]
}
```

**Example:**
```
A → B, C
B → D
C → D
D → (none)

edges = {
    0: [1, 2],  // A → B, C
    1: [3],     // B → D
    2: [3],     // C → D
    3: [],      // D → (none)
}
```

**Pros:** Space efficient, fast neighbor lookup  
**Cons:** Slow edge existence check

### Adjacency Matrix

```rust
struct Graph {
    nodes: Vec<Node>,
    matrix: Vec<Vec<bool>>,  // matrix[i][j] = edge from i to j
}
```

**Example:**
```
    A  B  C  D
A [ 0  1  1  0 ]
B [ 0  0  0  1 ]
C [ 0  0  0  1 ]
D [ 0  0  0  0 ]
```

**Pros:** Fast edge existence check  
**Cons:** O(V²) space, slow for sparse graphs

### Edge List

```rust
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<(usize, usize)>,  // (from, to)
}
```

**Pros:** Simple, easy to serialize  
**Cons:** Slow lookups

---

## Ownership Graph Model

For BorrowScope, we model ownership as a directed graph:

### Nodes (Variables)

```rust
struct Variable {
    id: usize,
    name: String,
    type_name: String,
    created_at: u64,
    dropped_at: Option<u64>,
}
```

**Example nodes:**
- `x: i32` (id=1)
- `r: &i32` (id=2)
- `s: String` (id=3)

### Edges (Relationships)

```rust
enum Relationship {
    Owns,           // x owns its value
    BorrowsImmut,   // r borrows x immutably
    BorrowsMut,     // r borrows x mutably
    SharesOwnership, // Rc/Arc clones
}
```

**Example edges:**
- `r → x` (BorrowsImmut): r borrows x
- `y → x` (Owns): y owns x (after move)

---

## Graph Properties

### Acyclic

Ownership graphs should be acyclic (no cycles):

```
Valid:
x → y → z

Invalid:
x → y
↑   ↓
└── z
```

**Why?** Rust's borrow checker prevents cycles.

### Multiple Edges

Multiple borrows of the same variable:

```
r1 → x
r2 → x
r3 → x
```

**Meaning:** Three references to x.

### Disconnected Components

Multiple independent ownership chains:

```
x → r1

y → r2
```

**Meaning:** x and y are unrelated.

---

## Graph Algorithms for Ownership

### 1. Reachability

**Question:** Can variable A reach variable B?

```rust
fn can_reach(graph: &Graph, from: usize, to: usize) -> bool {
    // Use DFS or BFS
}
```

**Use case:** Determine if a borrow transitively depends on a variable.

### 2. Cycle Detection

**Question:** Are there any cycles?

```rust
fn has_cycle(graph: &Graph) -> bool {
    // Use DFS with color marking
}
```

**Use case:** Detect invalid ownership patterns (shouldn't happen in valid Rust).

### 3. Topological Sort

**Question:** What's the dependency order?

```rust
fn topological_sort(graph: &Graph) -> Vec<usize> {
    // Kahn's algorithm or DFS-based
}
```

**Use case:** Determine drop order.

### 4. Connected Components

**Question:** Which variables are related?

```rust
fn connected_components(graph: &Graph) -> Vec<Vec<usize>> {
    // Union-find or DFS
}
```

**Use case:** Group related variables for visualization.

---

## Example: Ownership Graph

**Code:**
```rust
let x = 42;
let r1 = &x;
let r2 = &x;
```

**Graph:**
```
Nodes:
- x (id=1, type=i32)
- r1 (id=2, type=&i32)
- r2 (id=3, type=&i32)

Edges:
- r1 → x (BorrowsImmut)
- r2 → x (BorrowsImmut)
```

**Visualization:**
```
r1 ──→ x
r2 ──→ x
```

---

## Graph Metrics

### Degree

- **In-degree:** Number of incoming edges
- **Out-degree:** Number of outgoing edges

```rust
fn in_degree(graph: &Graph, node: usize) -> usize {
    graph.edges.iter()
        .filter(|(_, to)| *to == node)
        .count()
}

fn out_degree(graph: &Graph, node: usize) -> usize {
    graph.edges.get(&node).map(|v| v.len()).unwrap_or(0)
}
```

**For ownership:**
- High in-degree = many borrows of this variable
- High out-degree = this variable borrows many others

### Path Length

```rust
fn shortest_path(graph: &Graph, from: usize, to: usize) -> Option<usize> {
    // BFS to find shortest path
}
```

**Use case:** How many indirections between variables?

---

## Implementation Preview

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct OwnershipGraph {
    graph: DiGraph<Variable, Relationship>,
    id_to_node: HashMap<usize, NodeIndex>,
}

impl OwnershipGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_to_node: HashMap::new(),
        }
    }
    
    pub fn add_variable(&mut self, var: Variable) -> NodeIndex {
        let node_idx = self.graph.add_node(var.clone());
        self.id_to_node.insert(var.id, node_idx);
        node_idx
    }
    
    pub fn add_relationship(&mut self, from_id: usize, to_id: usize, rel: Relationship) {
        if let (Some(&from_node), Some(&to_node)) = 
            (self.id_to_node.get(&from_id), self.id_to_node.get(&to_id)) {
            self.graph.add_edge(from_node, to_node, rel);
        }
    }
}
```

---

## Key Takeaways

✅ **Graphs model relationships** - Nodes and edges  
✅ **Directed graphs** - Ownership has direction  
✅ **Adjacency list** - Best for sparse graphs  
✅ **Graph algorithms** - Reachability, cycles, topological sort  
✅ **Ownership is acyclic** - No cycles in valid Rust  

---

## Further Reading

- [Graph theory](https://en.wikipedia.org/wiki/Graph_theory)
- [Directed graphs](https://en.wikipedia.org/wiki/Directed_graph)
- [Graph algorithms](https://en.wikipedia.org/wiki/Graph_traversal)
- [Introduction to Algorithms (CLRS)](https://mitpress.mit.edu/9780262046305/introduction-to-algorithms/)

---

**Previous:** Chapter 5 Summary  
**Next:** [67-petgraph-library-overview.md](./67-petgraph-library-overview.md)

**Progress:** 1/10 ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜
