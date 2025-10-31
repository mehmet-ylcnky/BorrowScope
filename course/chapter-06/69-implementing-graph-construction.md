# Section 69: Implementing Graph Construction

## Learning Objectives

By the end of this section, you will:
- Implement the OwnershipGraph struct
- Build graph construction methods
- Integrate with runtime tracker
- Handle concurrent access
- Test graph construction

## Prerequisites

- Section 68 (Designing the Ownership Graph)
- Understanding of Mutex and thread safety
- Familiarity with petgraph API

---

## Implementation Plan

1. Create `borrowscope-graph` crate
2. Implement `OwnershipGraph` struct
3. Add construction methods
4. Integrate with runtime tracker
5. Add tests

---

## Step 1: Create the Crate

```bash
cd borrowscope
cargo new --lib borrowscope-graph
```

**Cargo.toml:**
```toml
[package]
name = "borrowscope-graph"
version = "0.1.0"
edition = "2021"

[dependencies]
petgraph = { version = "0.6", features = ["serde-1"] }
serde = { version = "1.0", features = ["derive"] }
```

---

## Step 2: Define Data Structures

**src/lib.rs:**
```rust
use petgraph::stable_graph::{StableGraph, NodeIndex, EdgeIndex};
use petgraph::Direction;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    BorrowsImmut { at: u64 },
    BorrowsMut { at: u64 },
    Moves { at: u64 },
    RcClone { at: u64, strong_count: usize },
    ArcClone { at: u64, strong_count: usize },
    RefCellBorrow { at: u64, is_mut: bool },
}

pub struct OwnershipGraph {
    graph: StableGraph<Variable, Relationship, petgraph::Directed>,
    id_to_node: HashMap<usize, NodeIndex>,
}
```

---

## Step 3: Implement Construction Methods

```rust
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
    
    pub fn add_borrow(&mut self, borrower_id: usize, owner_id: usize, 
                      is_mut: bool, at: u64) -> Option<EdgeIndex> {
        let borrower = *self.id_to_node.get(&borrower_id)?;
        let owner = *self.id_to_node.get(&owner_id)?;
        
        let rel = if is_mut {
            Relationship::BorrowsMut { at }
        } else {
            Relationship::BorrowsImmut { at }
        };
        
        Some(self.graph.add_edge(borrower, owner, rel))
    }
    
    pub fn add_move(&mut self, from_id: usize, to_id: usize, at: u64) -> Option<EdgeIndex> {
        let from = *self.id_to_node.get(&from_id)?;
        let to = *self.id_to_node.get(&to_id)?;
        
        Some(self.graph.add_edge(to, from, Relationship::Moves { at }))
    }
    
    pub fn mark_dropped(&mut self, id: usize, at: u64) -> bool {
        if let Some(&node) = self.id_to_node.get(&id) {
            if let Some(var) = self.graph.node_weight_mut(node) {
                var.dropped_at = Some(at);
                return true;
            }
        }
        false
    }
    
    pub fn add_rc_clone(&mut self, clone_id: usize, original_id: usize, 
                        at: u64, strong_count: usize) -> Option<EdgeIndex> {
        let clone = *self.id_to_node.get(&clone_id)?;
        let original = *self.id_to_node.get(&original_id)?;
        
        Some(self.graph.add_edge(clone, original, 
            Relationship::RcClone { at, strong_count }))
    }
    
    pub fn add_arc_clone(&mut self, clone_id: usize, original_id: usize, 
                         at: u64, strong_count: usize) -> Option<EdgeIndex> {
        let clone = *self.id_to_node.get(&clone_id)?;
        let original = *self.id_to_node.get(&original_id)?;
        
        Some(self.graph.add_edge(clone, original, 
            Relationship::ArcClone { at, strong_count }))
    }
}

impl Default for OwnershipGraph {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Step 4: Query Methods

```rust
impl OwnershipGraph {
    pub fn get_variable(&self, id: usize) -> Option<&Variable> {
        self.id_to_node.get(&id)
            .and_then(|&node| self.graph.node_weight(node))
    }
    
    pub fn borrowers_of(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node.get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph.neighbors_directed(node, Direction::Incoming)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }
    
    pub fn borrows(&self, id: usize) -> Vec<&Variable> {
        self.id_to_node.get(&id)
            .into_iter()
            .flat_map(|&node| {
                self.graph.neighbors(node)
                    .filter_map(|n| self.graph.node_weight(n))
            })
            .collect()
    }
    
    pub fn is_alive(&self, id: usize, at: u64) -> bool {
        self.get_variable(id)
            .map(|var| var.created_at <= at && var.dropped_at.map_or(true, |d| d > at))
            .unwrap_or(false)
    }
    
    pub fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.graph.node_weights()
    }
    
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
```

---

## Step 5: Serialization

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
    
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.export())
    }
}
```

---

## Step 6: Integration with Runtime

**Update borrowscope-runtime/Cargo.toml:**
```toml
[dependencies]
borrowscope-graph = { path = "../borrowscope-graph" }
parking_lot = "0.12"
```

**Update borrowscope-runtime/src/lib.rs:**
```rust
use borrowscope_graph::{OwnershipGraph, Variable};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static GRAPH: Mutex<OwnershipGraph> = Mutex::new(OwnershipGraph::new());
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

pub fn track_new<T>(name: &str, value: T) -> T {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let mut graph = GRAPH.lock();
    
    graph.add_variable(Variable {
        id,
        name: name.to_string(),
        type_name: std::any::type_name::<T>().to_string(),
        created_at: timestamp(),
        dropped_at: None,
        scope_depth: 0,
    });
    
    value
}

pub fn track_borrow<T>(borrower_name: &str, owner_id: usize, value: &T) -> &T {
    let borrower_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let mut graph = GRAPH.lock();
    
    graph.add_variable(Variable {
        id: borrower_id,
        name: borrower_name.to_string(),
        type_name: std::any::type_name::<&T>().to_string(),
        created_at: timestamp(),
        dropped_at: None,
        scope_depth: 0,
    });
    
    graph.add_borrow(borrower_id, owner_id, false, timestamp());
    
    value
}

pub fn track_borrow_mut<T>(borrower_name: &str, owner_id: usize, value: &mut T) -> &mut T {
    let borrower_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let mut graph = GRAPH.lock();
    
    graph.add_variable(Variable {
        id: borrower_id,
        name: borrower_name.to_string(),
        type_name: std::any::type_name::<&mut T>().to_string(),
        created_at: timestamp(),
        dropped_at: None,
        scope_depth: 0,
    });
    
    graph.add_borrow(borrower_id, owner_id, true, timestamp());
    
    value
}

pub fn track_drop(id: usize) {
    let mut graph = GRAPH.lock();
    graph.mark_dropped(id, timestamp());
}

pub fn export_graph() -> String {
    let graph = GRAPH.lock();
    graph.to_json().unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

pub fn reset_graph() {
    let mut graph = GRAPH.lock();
    *graph = OwnershipGraph::new();
    NEXT_ID.store(1, Ordering::SeqCst);
}
```

---

## Step 7: Testing

**borrowscope-graph/src/lib.rs:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_variable() {
        let mut graph = OwnershipGraph::new();
        
        let var = Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        };
        
        graph.add_variable(var.clone());
        
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.get_variable(1).unwrap().name, "x");
    }

    #[test]
    fn test_add_borrow() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        
        graph.add_variable(Variable {
            id: 2,
            name: "r".into(),
            type_name: "&i32".into(),
            created_at: 1050,
            dropped_at: None,
            scope_depth: 0,
        });
        
        graph.add_borrow(2, 1, false, 1050);
        
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.borrowers_of(1).len(), 1);
        assert_eq!(graph.borrowers_of(1)[0].name, "r");
    }

    #[test]
    fn test_multiple_borrows() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable { id: 1, name: "x".into(), 
            type_name: "i32".into(), created_at: 1000, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 2, name: "r1".into(), 
            type_name: "&i32".into(), created_at: 1050, dropped_at: None, scope_depth: 0 });
        graph.add_variable(Variable { id: 3, name: "r2".into(), 
            type_name: "&i32".into(), created_at: 1100, dropped_at: None, scope_depth: 0 });
        
        graph.add_borrow(2, 1, false, 1050);
        graph.add_borrow(3, 1, false, 1100);
        
        assert_eq!(graph.borrowers_of(1).len(), 2);
    }

    #[test]
    fn test_mark_dropped() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        
        assert!(graph.is_alive(1, 1500));
        
        graph.mark_dropped(1, 2000);
        
        assert!(graph.is_alive(1, 1500));
        assert!(!graph.is_alive(1, 2500));
    }

    #[test]
    fn test_export() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        
        let export = graph.export();
        assert_eq!(export.nodes.len(), 1);
        assert_eq!(export.edges.len(), 0);
        
        let json = graph.to_json().unwrap();
        assert!(json.contains("\"name\": \"x\""));
    }
}
```

---

## Step 8: Integration Test

**borrowscope-runtime/tests/graph_integration.rs:**
```rust
use borrowscope_runtime::{track_new, track_borrow, export_graph, reset_graph};

#[test]
fn test_simple_borrow() {
    reset_graph();
    
    let x = track_new("x", 42);
    let r = track_borrow("r", 1, &x);
    
    let json = export_graph();
    assert!(json.contains("\"name\": \"x\""));
    assert!(json.contains("\"name\": \"r\""));
    assert!(json.contains("BorrowsImmut"));
}
```

---

## Performance Considerations

### Lock Contention

```rust
// Bad: Hold lock during expensive operation
pub fn track_new<T>(name: &str, value: T) -> T {
    let mut graph = GRAPH.lock();
    // ... expensive work ...
    value
}

// Good: Minimize lock duration
pub fn track_new<T>(name: &str, value: T) -> T {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let var = Variable {
        id,
        name: name.to_string(),
        type_name: std::any::type_name::<T>().to_string(),
        created_at: timestamp(),
        dropped_at: None,
        scope_depth: 0,
    };
    
    let mut graph = GRAPH.lock();
    graph.add_variable(var);
    drop(graph);  // Release lock early
    
    value
}
```

### Memory Usage

```rust
impl OwnershipGraph {
    pub fn clear(&mut self) {
        self.graph.clear();
        self.id_to_node.clear();
    }
    
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of_val(&self.graph) + 
        std::mem::size_of_val(&self.id_to_node)
    }
}
```

---

## Key Takeaways

✅ **OwnershipGraph** - Wraps petgraph with helper methods  
✅ **Construction methods** - add_variable, add_borrow, add_move  
✅ **Query methods** - borrowers_of, is_alive  
✅ **Integration** - Connect with runtime tracker  
✅ **Testing** - Unit and integration tests  

---

## Further Reading

- [petgraph documentation](https://docs.rs/petgraph/)
- [Mutex best practices](https://doc.rust-lang.org/std/sync/struct.Mutex.html)

---

**Previous:** [68-designing-the-ownership-graph.md](./68-designing-the-ownership-graph.md)  
**Next:** [70-graph-traversal-algorithms.md](./70-graph-traversal-algorithms.md)

**Progress:** 4/10 ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜
