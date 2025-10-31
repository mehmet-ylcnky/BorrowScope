# Section 68: Designing the Ownership Graph

## Learning Objectives

By the end of this section, you will:
- Design node and edge structures for ownership tracking
- Define the ownership graph schema
- Plan graph construction strategy
- Handle edge cases and special scenarios
- Prepare for implementation

## Prerequisites

- Section 67 (Petgraph Library Overview)
- Understanding of Rust ownership rules
- Familiarity with Chapter 3 (Runtime Tracker)

---

## Design Goals

**Requirements:**
1. Track all variables and their relationships
2. Support immutable and mutable borrows
3. Handle moves and drops
4. Track smart pointers (Rc, Arc, RefCell)
5. Efficient queries (who borrows what?)
6. Serializable for UI consumption

---

## Node Structure: Variable

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}
```

**Fields:**
- `id`: Unique identifier (from runtime tracker)
- `name`: Variable name from source code
- `type_name`: Type information (e.g., "i32", "&str")
- `created_at`: Timestamp when created
- `dropped_at`: Timestamp when dropped (None if still alive)
- `scope_depth`: Nesting level (for visualization)

**Example:**
```rust
Variable {
    id: 1,
    name: "x".into(),
    type_name: "i32".into(),
    created_at: 1000,
    dropped_at: None,
    scope_depth: 0,
}
```

---

## Edge Structure: Relationship

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    BorrowsImmut { at: u64 },
    BorrowsMut { at: u64 },
    Moves { at: u64 },
    RcClone { at: u64, strong_count: usize },
    ArcClone { at: u64, strong_count: usize },
    RefCellBorrow { at: u64, is_mut: bool },
}
```

**Variants:**
- `BorrowsImmut`: Immutable reference (`&T`)
- `BorrowsMut`: Mutable reference (`&mut T`)
- `Moves`: Ownership transfer
- `RcClone`: Rc clone (with reference count)
- `ArcClone`: Arc clone (with reference count)
- `RefCellBorrow`: RefCell borrow (runtime check)

**Edge direction:** `borrower → owner`

**Example:**
```rust
// let r = &x;
Relationship::BorrowsImmut { at: 1050 }
```

---

## Graph Structure

```rust
use petgraph::stable_graph::{StableGraph, NodeIndex, EdgeIndex};
use petgraph::Directed;
use std::collections::HashMap;

pub struct OwnershipGraph {
    graph: StableGraph<Variable, Relationship, Directed>,
    id_to_node: HashMap<usize, NodeIndex>,
}
```

**Components:**
- `graph`: The actual petgraph structure
- `id_to_node`: Fast lookup from variable ID to graph node

---

## Construction Strategy

### 1. On Variable Creation (track_new)

```rust
impl OwnershipGraph {
    pub fn add_variable(&mut self, var: Variable) -> NodeIndex {
        let node = self.graph.add_node(var.clone());
        self.id_to_node.insert(var.id, node);
        node
    }
}
```

**Example:**
```rust
// let x = 42;
graph.add_variable(Variable {
    id: 1,
    name: "x".into(),
    type_name: "i32".into(),
    created_at: 1000,
    dropped_at: None,
    scope_depth: 0,
});
```

### 2. On Borrow (track_borrow)

```rust
impl OwnershipGraph {
    pub fn add_borrow(&mut self, borrower_id: usize, owner_id: usize, 
                      is_mut: bool, at: u64) {
        if let (Some(&borrower), Some(&owner)) = 
            (self.id_to_node.get(&borrower_id), self.id_to_node.get(&owner_id)) {
            let rel = if is_mut {
                Relationship::BorrowsMut { at }
            } else {
                Relationship::BorrowsImmut { at }
            };
            self.graph.add_edge(borrower, owner, rel);
        }
    }
}
```

**Example:**
```rust
// let r = &x;
graph.add_borrow(2, 1, false, 1050);
```

**Graph:**
```
r (id=2) --BorrowsImmut--> x (id=1)
```

### 3. On Move (track_move)

```rust
impl OwnershipGraph {
    pub fn add_move(&mut self, from_id: usize, to_id: usize, at: u64) {
        if let (Some(&from), Some(&to)) = 
            (self.id_to_node.get(&from_id), self.id_to_node.get(&to_id)) {
            self.graph.add_edge(to, from, Relationship::Moves { at });
        }
    }
}
```

**Example:**
```rust
// let y = x;  // x moved to y
graph.add_move(1, 2, 1100);
```

**Graph:**
```
y (id=2) --Moves--> x (id=1)
```

### 4. On Drop

```rust
impl OwnershipGraph {
    pub fn mark_dropped(&mut self, id: usize, at: u64) {
        if let Some(&node) = self.id_to_node.get(&id) {
            if let Some(var) = self.graph.node_weight_mut(node) {
                var.dropped_at = Some(at);
            }
        }
    }
}
```

---

## Example Scenarios

### Scenario 1: Simple Borrow

```rust
let x = 42;
let r = &x;
```

**Graph:**
```
Nodes:
  x (id=1, type=i32, created_at=1000)
  r (id=2, type=&i32, created_at=1050)

Edges:
  r --BorrowsImmut(at=1050)--> x
```

### Scenario 2: Multiple Borrows

```rust
let x = 42;
let r1 = &x;
let r2 = &x;
```

**Graph:**
```
r1 --BorrowsImmut--> x
r2 --BorrowsImmut--> x
```

### Scenario 3: Mutable Borrow

```rust
let mut x = 42;
let r = &mut x;
```

**Graph:**
```
r --BorrowsMut--> x
```

### Scenario 4: Move

```rust
let x = String::from("hello");
let y = x;  // x moved
```

**Graph:**
```
y --Moves--> x
```

**Note:** x is marked as moved (dropped_at set).

### Scenario 5: Rc Clone

```rust
let x = Rc::new(42);
let y = Rc::clone(&x);
```

**Graph:**
```
Nodes:
  x (id=1, type=Rc<i32>)
  y (id=2, type=Rc<i32>)

Edges:
  y --RcClone(strong_count=2)--> x
```

---

## Query Operations

### Who borrows this variable?

```rust
impl OwnershipGraph {
    pub fn borrowers_of(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node.get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph.neighbors_directed(node, Direction::Incoming)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }
}
```

### What does this variable borrow?

```rust
impl OwnershipGraph {
    pub fn borrows(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node.get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph.neighbors(node)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }
}
```

### Is this variable alive?

```rust
impl OwnershipGraph {
    pub fn is_alive(&self, id: usize, at: u64) -> bool {
        self.id_to_node.get(&id)
            .and_then(|&node| self.graph.node_weight(node))
            .map(|var| var.created_at <= at && var.dropped_at.map_or(true, |d| d > at))
            .unwrap_or(false)
    }
}
```

### Active borrows at time T

```rust
impl OwnershipGraph {
    pub fn active_borrows_at(&self, id: usize, at: u64) -> Vec<(&Variable, &Relationship)> {
        self.id_to_node.get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph.edges_directed(node, Direction::Incoming)
                    .filter_map(|edge| {
                        let rel = edge.weight();
                        let borrower = self.graph.node_weight(edge.source())?;
                        
                        // Check if borrow is active at time 'at'
                        let borrow_time = match rel {
                            Relationship::BorrowsImmut { at: t } => *t,
                            Relationship::BorrowsMut { at: t } => *t,
                            _ => return None,
                        };
                        
                        if borrow_time <= at && borrower.dropped_at.map_or(true, |d| d > at) {
                            Some((borrower, rel))
                        } else {
                            None
                        }
                    })
            })
            .collect()
    }
}
```

---

## Edge Cases

### 1. Reborrow

```rust
let x = 42;
let r1 = &x;
let r2 = &*r1;  // reborrow
```

**Graph:**
```
r1 --BorrowsImmut--> x
r2 --BorrowsImmut--> r1
```

**Note:** r2 borrows r1, not x directly.

### 2. Temporary Values

```rust
let r = &String::from("temp");
```

**Graph:**
```
<temp> (id=1, name="<temp>")
r (id=2) --BorrowsImmut--> <temp>
```

**Note:** Use synthetic names for temporaries.

### 3. Field Borrows

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 1, y: 2 };
let r = &p.x;
```

**Graph:**
```
r --BorrowsImmut--> p
```

**Note:** Simplified - borrow entire struct (field-level tracking is complex).

### 4. Closure Captures

```rust
let x = 42;
let f = || println!("{}", x);
```

**Graph:**
```
f --BorrowsImmut--> x
```

**Note:** Closure borrows x.

---

## Serialization Format

```rust
#[derive(Serialize, Deserialize)]
pub struct GraphExport {
    pub nodes: Vec<Variable>,
    pub edges: Vec<EdgeExport>,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeExport {
    pub from_id: usize,
    pub to_id: usize,
    pub relationship: Relationship,
}

impl OwnershipGraph {
    pub fn export(&self) -> GraphExport {
        let nodes = self.graph.node_weights().cloned().collect();
        let edges = self.graph.edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                Some(EdgeExport {
                    from_id: from.id,
                    to_id: to.id,
                    relationship: edge.weight().clone(),
                })
            })
            .collect();
        
        GraphExport { nodes, edges }
    }
}
```

**JSON output:**
```json
{
  "nodes": [
    {"id": 1, "name": "x", "type_name": "i32", "created_at": 1000, "dropped_at": null, "scope_depth": 0},
    {"id": 2, "name": "r", "type_name": "&i32", "created_at": 1050, "dropped_at": 1100, "scope_depth": 0}
  ],
  "edges": [
    {"from_id": 2, "to_id": 1, "relationship": {"BorrowsImmut": {"at": 1050}}}
  ]
}
```

---

## Integration with Runtime Tracker

```rust
// In borrowscope-runtime
use borrowscope_graph::OwnershipGraph;

static GRAPH: Mutex<OwnershipGraph> = Mutex::new(OwnershipGraph::new());

pub fn track_new<T>(id: usize, name: &str, value: T) -> T {
    let mut graph = GRAPH.lock();
    graph.add_variable(Variable {
        id,
        name: name.into(),
        type_name: std::any::type_name::<T>().into(),
        created_at: timestamp(),
        dropped_at: None,
        scope_depth: 0,  // TODO: track from macro
    });
    value
}

pub fn track_borrow<T>(borrower_id: usize, owner_id: usize, value: &T) -> &T {
    let mut graph = GRAPH.lock();
    graph.add_borrow(borrower_id, owner_id, false, timestamp());
    value
}
```

---

## Key Takeaways

✅ **Variable nodes** - Track ID, name, type, timestamps  
✅ **Relationship edges** - Different borrow types with metadata  
✅ **Edge direction** - borrower → owner  
✅ **Query API** - Who borrows what, when  
✅ **Serialization** - Export to JSON for UI  

---

## Further Reading

- [Rust ownership rules](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Graph data modeling](https://en.wikipedia.org/wiki/Graph_database)

---

**Previous:** [67-petgraph-library-overview.md](./67-petgraph-library-overview.md)  
**Next:** [69-implementing-graph-construction.md](./69-implementing-graph-construction.md)

**Progress:** 3/10 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜
