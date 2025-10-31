# Section 23: Graph Data Structures

## Learning Objectives

By the end of this section, you will:
- Implement the OwnershipGraph structure
- Use petgraph for graph operations
- Build graphs from events
- Query graph relationships
- Serialize graphs to JSON

## Prerequisites

- Completed Section 22
- Understanding of graph theory basics
- Familiarity with petgraph

---

## Graph Model

### Nodes: Variables

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
}
```

### Edges: Relationships

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    Owns { from: String, to: String },
    BorrowsImmut { from: String, to: String, start: u64, end: u64 },
    BorrowsMut { from: String, to: String, start: u64, end: u64 },
}
```

---

## Step 1: Implement Graph Structures

### File: `borrowscope-runtime/src/graph.rs`

```rust
//! Ownership graph data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::event::Event;

/// A variable in the ownership graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
}

/// A relationship between variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Relationship {
    Owns {
        from: String,
        to: String,
    },
    BorrowsImmut {
        from: String,
        to: String,
        start: u64,
        end: u64,
    },
    BorrowsMut {
        from: String,
        to: String,
        start: u64,
        end: u64,
    },
}

/// The complete ownership graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraph {
    pub nodes: Vec<Variable>,
    pub edges: Vec<Relationship>,
}

impl OwnershipGraph {
    /// Create an empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    /// Add a variable node
    pub fn add_variable(&mut self, var: Variable) {
        self.nodes.push(var);
    }
    
    /// Add a relationship edge
    pub fn add_relationship(&mut self, rel: Relationship) {
        self.edges.push(rel);
    }
    
    /// Find a variable by ID
    pub fn find_variable(&self, id: &str) -> Option<&Variable> {
        self.nodes.iter().find(|v| v.id == id)
    }
    
    /// Find all borrows of a variable
    pub fn find_borrows(&self, var_id: &str) -> Vec<&Relationship> {
        self.edges.iter().filter(|rel| {
            match rel {
                Relationship::BorrowsImmut { to, .. } => to == var_id,
                Relationship::BorrowsMut { to, .. } => to == var_id,
                _ => false,
            }
        }).collect()
    }
    
    /// Get statistics
    pub fn stats(&self) -> GraphStats {
        let mut immut_borrows = 0;
        let mut mut_borrows = 0;
        
        for edge in &self.edges {
            match edge {
                Relationship::BorrowsImmut { .. } => immut_borrows += 1,
                Relationship::BorrowsMut { .. } => mut_borrows += 1,
                _ => {}
            }
        }
        
        GraphStats {
            total_variables: self.nodes.len(),
            total_relationships: self.edges.len(),
            immutable_borrows: immut_borrows,
            mutable_borrows: mut_borrows,
        }
    }
}

impl Default for OwnershipGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_variables: usize,
    pub total_relationships: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
}

/// Build a graph from events
pub fn build_graph(events: &[Event]) -> OwnershipGraph {
    let mut graph = OwnershipGraph::new();
    let mut var_map: HashMap<String, Variable> = HashMap::new();
    let mut borrow_map: HashMap<String, (String, bool, u64)> = HashMap::new();
    
    for event in events {
        match event {
            Event::New { var_name, var_id, type_name, timestamp, .. } => {
                let var = Variable {
                    id: var_id.clone(),
                    name: var_name.clone(),
                    type_name: type_name.clone(),
                    created_at: *timestamp,
                    dropped_at: None,
                };
                var_map.insert(var_id.clone(), var);
            }
            
            Event::Borrow { borrower_id, owner_id, mutable, timestamp, .. } => {
                borrow_map.insert(
                    borrower_id.clone(),
                    (owner_id.clone(), *mutable, *timestamp)
                );
            }
            
            Event::Drop { var_id, timestamp, .. } => {
                // Mark variable as dropped
                if let Some(var) = var_map.get_mut(var_id) {
                    var.dropped_at = Some(*timestamp);
                }
                
                // End borrow if this is a borrower
                if let Some((owner_id, is_mutable, start)) = borrow_map.remove(var_id) {
                    let rel = if is_mutable {
                        Relationship::BorrowsMut {
                            from: var_id.clone(),
                            to: owner_id,
                            start,
                            end: *timestamp,
                        }
                    } else {
                        Relationship::BorrowsImmut {
                            from: var_id.clone(),
                            to: owner_id,
                            start,
                            end: *timestamp,
                        }
                    };
                    graph.add_relationship(rel);
                }
            }
            
            _ => {}
        }
    }
    
    // Add all variables to graph
    for var in var_map.into_values() {
        graph.add_variable(var);
    }
    
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = OwnershipGraph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_add_variable() {
        let mut graph = OwnershipGraph::new();
        
        let var = Variable {
            id: "x_0".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 1,
            dropped_at: None,
        };
        
        graph.add_variable(var);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_find_variable() {
        let mut graph = OwnershipGraph::new();
        
        let var = Variable {
            id: "x_0".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 1,
            dropped_at: None,
        };
        
        graph.add_variable(var);
        
        let found = graph.find_variable("x_0");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
    }

    #[test]
    fn test_build_graph_simple() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Drop {
                timestamp: 2,
                var_id: "x_0".to_string(),
            },
        ];
        
        let graph = build_graph(&events);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].dropped_at, Some(2));
    }

    #[test]
    fn test_build_graph_with_borrow() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "s".to_string(),
                var_id: "s_0".to_string(),
                type_name: "String".to_string(),
            },
            Event::Borrow {
                timestamp: 2,
                borrower_name: "r".to_string(),
                borrower_id: "r_1".to_string(),
                owner_id: "s_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 3,
                var_id: "r_1".to_string(),
            },
            Event::Drop {
                timestamp: 4,
                var_id: "s_0".to_string(),
            },
        ];
        
        let graph = build_graph(&events);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        
        // Check the borrow relationship
        match &graph.edges[0] {
            Relationship::BorrowsImmut { from, to, start, end } => {
                assert_eq!(from, "r_1");
                assert_eq!(to, "s_0");
                assert_eq!(*start, 2);
                assert_eq!(*end, 3);
            }
            _ => panic!("Expected BorrowsImmut"),
        }
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_relationship(Relationship::BorrowsImmut {
            from: "r1".to_string(),
            to: "s".to_string(),
            start: 1,
            end: 2,
        });
        
        graph.add_relationship(Relationship::BorrowsMut {
            from: "r2".to_string(),
            to: "s".to_string(),
            start: 3,
            end: 4,
        });
        
        let stats = graph.stats();
        assert_eq!(stats.immutable_borrows, 1);
        assert_eq!(stats.mutable_borrows, 1);
    }

    #[test]
    fn test_serialization() {
        let graph = OwnershipGraph::new();
        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: OwnershipGraph = serde_json::from_str(&json).unwrap();
        
        assert_eq!(graph.nodes.len(), deserialized.nodes.len());
    }
}
```

---

## Step 2: Add Graph Export

### File: `borrowscope-runtime/src/export.rs`

```rust
//! Export functionality

use crate::event::Event;
use crate::graph::{build_graph, OwnershipGraph};
use crate::tracker::get_events;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

/// Complete export data
#[derive(Debug, Serialize)]
pub struct ExportData {
    pub version: String,
    pub events: Vec<Event>,
    pub graph: OwnershipGraph,
}

/// Export to JSON file
pub fn export_json(path: &str) -> Result<(), std::io::Error> {
    let events = get_events();
    let graph = build_graph(&events);
    
    let data = ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        events,
        graph,
    };
    
    let json = serde_json::to_string_pretty(&data)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    
    Ok(())
}

/// Get the ownership graph
pub fn get_graph() -> OwnershipGraph {
    let events = get_events();
    build_graph(&events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{reset, track_new, track_borrow, track_drop};

    #[test]
    fn test_get_graph() {
        reset();
        
        let s = track_new("s", String::from("hello"));
        let r = track_borrow("r", &s);
        track_drop("r");
        track_drop("s");
        
        let graph = get_graph();
        assert!(graph.nodes.len() >= 2);
    }

    #[test]
    fn test_export_json() {
        reset();
        
        track_new("x", 5);
        track_drop("x");
        
        let result = export_json("/tmp/test_export.json");
        assert!(result.is_ok());
    }
}
```

---

## Step 3: Update lib.rs

### File: `borrowscope-runtime/src/lib.rs`

```rust
//! Runtime tracking for BorrowScope

mod event;
mod tracker;
mod graph;
mod export;

pub use event::Event;
pub use graph::{OwnershipGraph, Variable, Relationship, GraphStats};
pub use tracker::{
    track_new,
    track_borrow,
    track_borrow_mut,
    track_move,
    track_drop,
    reset,
    get_events,
};
pub use export::{export_json, get_graph};
```

---

## Step 4: Integration Tests

### File: `borrowscope-runtime/tests/graph_tests.rs`

```rust
//! Integration tests for graph functionality

use borrowscope_runtime::*;

#[test]
fn test_simple_graph() {
    reset();
    
    let x = track_new("x", 5);
    track_drop("x");
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].name, "x");
    assert!(graph.nodes[0].dropped_at.is_some());
    
    assert_eq!(x, 5);
}

#[test]
fn test_borrow_graph() {
    reset();
    
    let s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    track_drop("r");
    track_drop("s");
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    
    // Verify borrow relationship
    let borrows = graph.find_borrows("s_0");
    assert_eq!(borrows.len(), 1);
    
    assert_eq!(r, "hello");
}

#[test]
fn test_multiple_borrows() {
    reset();
    
    let s = track_new("s", String::from("hello"));
    let r1 = track_borrow("r1", &s);
    let r2 = track_borrow("r2", &s);
    
    track_drop("r2");
    track_drop("r1");
    track_drop("s");
    
    let graph = get_graph();
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
    
    let stats = graph.stats();
    assert_eq!(stats.immutable_borrows, 2);
    
    assert_eq!(r1, "hello");
    assert_eq!(r2, "hello");
}

#[test]
fn test_mutable_borrow() {
    reset();
    
    let mut s = track_new("s", String::from("hello"));
    let r = track_borrow_mut("r", &mut s);
    r.push_str(" world");
    
    track_drop("r");
    track_drop("s");
    
    let graph = get_graph();
    let stats = graph.stats();
    assert_eq!(stats.mutable_borrows, 1);
}

#[test]
fn test_export_and_load() {
    reset();
    
    let x = track_new("x", 5);
    let y = track_new("y", 10);
    track_drop("y");
    track_drop("x");
    
    // Export
    export_json("/tmp/test_graph.json").unwrap();
    
    // Verify file exists and is valid JSON
    let content = std::fs::read_to_string("/tmp/test_graph.json").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert!(parsed["events"].is_array());
    assert!(parsed["graph"].is_object());
}
```

---

## Step 5: Build and Test

### Build

```bash
cargo build -p borrowscope-runtime
```

### Run Tests

```bash
cargo test -p borrowscope-runtime
```

Expected output:
```
running 20 tests
test graph::tests::test_empty_graph ... ok
test graph::tests::test_add_variable ... ok
test graph::tests::test_build_graph_simple ... ok
test graph::tests::test_build_graph_with_borrow ... ok
test graph_tests::test_simple_graph ... ok
test graph_tests::test_borrow_graph ... ok
test graph_tests::test_multiple_borrows ... ok
...

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured
```

---

## Example Output

### JSON Export

```json
{
  "version": "0.1.0",
  "events": [
    {
      "type": "New",
      "timestamp": 1,
      "var_name": "s",
      "var_id": "s_0",
      "type_name": "String"
    },
    {
      "type": "Borrow",
      "timestamp": 2,
      "borrower_name": "r",
      "borrower_id": "r_1",
      "owner_id": "s_0",
      "mutable": false
    },
    {
      "type": "Drop",
      "timestamp": 3,
      "var_id": "r_1"
    },
    {
      "type": "Drop",
      "timestamp": 4,
      "var_id": "s_0"
    }
  ],
  "graph": {
    "nodes": [
      {
        "id": "s_0",
        "name": "s",
        "type_name": "String",
        "created_at": 1,
        "dropped_at": 4
      },
      {
        "id": "r_1",
        "name": "r",
        "type_name": "&String",
        "created_at": 2,
        "dropped_at": 3
      }
    ],
    "edges": [
      {
        "kind": "BorrowsImmut",
        "from": "r_1",
        "to": "s_0",
        "start": 2,
        "end": 3
      }
    ]
  }
}
```

---

## Key Takeaways

### Graph Implementation

✅ **Variable nodes** - Track creation and drop times  
✅ **Relationship edges** - Owns, BorrowsImmut, BorrowsMut  
✅ **Graph building** - Construct from events  
✅ **Queries** - Find variables and relationships  
✅ **Statistics** - Count borrows and variables  

### Export

✅ **JSON format** - Human-readable, standard  
✅ **Complete data** - Events + graph  
✅ **Version tracking** - For compatibility  
✅ **Pretty printing** - Easy to read  

### Testing

✅ **Unit tests** - Each function tested  
✅ **Integration tests** - Complete workflows  
✅ **Real examples** - Actual usage patterns  

---

## What's Next?

In **Section 24: JSON Serialization**, we'll:
- Optimize JSON output
- Add custom serialization
- Handle edge cases
- Improve performance
- Add compression options

---

**Previous:** [22-event-tracking-system.md](./22-event-tracking-system.md)  
**Next:** [24-json-serialization.md](./24-json-serialization.md)

**Progress:** 3/15 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Graphs make relationships visible." 🕸️*
