# Section 72: Graph Serialization

## Learning Objectives

By the end of this section, you will:
- Serialize ownership graphs to JSON
- Design efficient serialization formats
- Handle large graphs
- Support multiple export formats
- Prepare data for UI consumption

## Prerequisites

- Section 71 (Detecting Borrow Conflicts)
- Understanding of serde
- JSON format knowledge

---

## Serialization Requirements

**Goals:**
1. Export complete graph structure
2. Preserve all metadata (timestamps, types)
3. Efficient format for large graphs
4. Easy to consume in JavaScript/UI
5. Human-readable for debugging

---

## Basic JSON Format

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GraphExport {
    pub metadata: GraphMetadata,
    pub nodes: Vec<NodeExport>,
    pub edges: Vec<EdgeExport>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphMetadata {
    pub version: String,
    pub timestamp: u64,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct NodeExport {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeExport {
    pub from: usize,
    pub to: usize,
    pub relationship: RelationshipExport,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelationshipExport {
    BorrowsImmut { at: u64 },
    BorrowsMut { at: u64 },
    Moves { at: u64 },
    RcClone { at: u64, strong_count: usize },
    ArcClone { at: u64, strong_count: usize },
    RefCellBorrow { at: u64, is_mut: bool },
}
```

---

## Implementation

```rust
impl OwnershipGraph {
    pub fn export(&self) -> GraphExport {
        let nodes = self.graph.node_weights()
            .map(|var| NodeExport {
                id: var.id,
                name: var.name.clone(),
                type_name: var.type_name.clone(),
                created_at: var.created_at,
                dropped_at: var.dropped_at,
                scope_depth: var.scope_depth,
            })
            .collect();
        
        let edges = self.graph.edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                
                let relationship = match edge.weight() {
                    Relationship::BorrowsImmut { at } => 
                        RelationshipExport::BorrowsImmut { at: *at },
                    Relationship::BorrowsMut { at } => 
                        RelationshipExport::BorrowsMut { at: *at },
                    Relationship::Moves { at } => 
                        RelationshipExport::Moves { at: *at },
                    Relationship::RcClone { at, strong_count } => 
                        RelationshipExport::RcClone { at: *at, strong_count: *strong_count },
                    Relationship::ArcClone { at, strong_count } => 
                        RelationshipExport::ArcClone { at: *at, strong_count: *strong_count },
                    Relationship::RefCellBorrow { at, is_mut } => 
                        RelationshipExport::RefCellBorrow { at: *at, is_mut: *is_mut },
                };
                
                Some(EdgeExport {
                    from: from.id,
                    to: to.id,
                    relationship,
                })
            })
            .collect();
        
        GraphExport {
            metadata: GraphMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                node_count: self.graph.node_count(),
                edge_count: self.graph.edge_count(),
            },
            nodes,
            edges,
        }
    }
    
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.export())
    }
    
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.export())
    }
}
```

---

## Example Output

```json
{
  "metadata": {
    "version": "0.1.0",
    "timestamp": 1698765432,
    "node_count": 3,
    "edge_count": 2
  },
  "nodes": [
    {
      "id": 1,
      "name": "x",
      "type_name": "i32",
      "created_at": 1000,
      "dropped_at": null,
      "scope_depth": 0
    },
    {
      "id": 2,
      "name": "r1",
      "type_name": "&i32",
      "created_at": 1050,
      "dropped_at": 1200,
      "scope_depth": 0
    },
    {
      "id": 3,
      "name": "r2",
      "type_name": "&i32",
      "created_at": 1100,
      "dropped_at": 1250,
      "scope_depth": 0
    }
  ],
  "edges": [
    {
      "from": 2,
      "to": 1,
      "relationship": {
        "type": "BorrowsImmut",
        "at": 1050
      }
    },
    {
      "from": 3,
      "to": 1,
      "relationship": {
        "type": "BorrowsImmut",
        "at": 1100
      }
    }
  ]
}
```

---

## Optimized Format for Large Graphs

### Compact Node IDs

```rust
#[derive(Serialize)]
pub struct CompactGraphExport {
    pub meta: GraphMetadata,
    pub nodes: Vec<CompactNode>,
    pub edges: Vec<[usize; 3]>,  // [from, to, rel_type]
    pub relationships: Vec<CompactRelationship>,
}

#[derive(Serialize)]
pub struct CompactNode {
    pub i: usize,           // id
    pub n: String,          // name
    pub t: String,          // type
    pub c: u64,             // created_at
    pub d: Option<u64>,     // dropped_at
    pub s: usize,           // scope_depth
}

#[derive(Serialize)]
pub struct CompactRelationship {
    pub t: u8,              // type: 0=immut, 1=mut, 2=move, etc.
    pub a: u64,             // at
    pub x: Option<usize>,   // extra data (strong_count, etc.)
}

impl OwnershipGraph {
    pub fn export_compact(&self) -> CompactGraphExport {
        let nodes = self.graph.node_weights()
            .map(|v| CompactNode {
                i: v.id,
                n: v.name.clone(),
                t: v.type_name.clone(),
                c: v.created_at,
                d: v.dropped_at,
                s: v.scope_depth,
            })
            .collect();
        
        let mut relationships = Vec::new();
        let edges = self.graph.edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?.id;
                let to = self.graph.node_weight(edge.target())?.id;
                
                let (rel_type, at, extra) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => (0, *at, None),
                    Relationship::BorrowsMut { at } => (1, *at, None),
                    Relationship::Moves { at } => (2, *at, None),
                    Relationship::RcClone { at, strong_count } => (3, *at, Some(*strong_count)),
                    Relationship::ArcClone { at, strong_count } => (4, *at, Some(*strong_count)),
                    Relationship::RefCellBorrow { at, is_mut } => (5, *at, Some(*is_mut as usize)),
                };
                
                let rel_idx = relationships.len();
                relationships.push(CompactRelationship { t: rel_type, a: at, x: extra });
                
                Some([from, to, rel_idx])
            })
            .collect();
        
        CompactGraphExport {
            meta: GraphMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                node_count: self.graph.node_count(),
                edge_count: self.graph.edge_count(),
            },
            nodes,
            edges,
            relationships,
        }
    }
}
```

---

## Streaming for Large Graphs

```rust
use std::io::Write;

impl OwnershipGraph {
    pub fn stream_json<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"{\n  \"metadata\": ")?;
        serde_json::to_writer(&mut *writer, &GraphMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
        })?;
        
        writer.write_all(b",\n  \"nodes\": [\n")?;
        
        let mut first = true;
        for var in self.graph.node_weights() {
            if !first {
                writer.write_all(b",\n")?;
            }
            first = false;
            
            writer.write_all(b"    ")?;
            serde_json::to_writer(&mut *writer, &NodeExport {
                id: var.id,
                name: var.name.clone(),
                type_name: var.type_name.clone(),
                created_at: var.created_at,
                dropped_at: var.dropped_at,
                scope_depth: var.scope_depth,
            })?;
        }
        
        writer.write_all(b"\n  ],\n  \"edges\": [\n")?;
        
        first = true;
        for edge in self.graph.edge_references() {
            if let (Some(from), Some(to)) = (
                self.graph.node_weight(edge.source()),
                self.graph.node_weight(edge.target())
            ) {
                if !first {
                    writer.write_all(b",\n")?;
                }
                first = false;
                
                writer.write_all(b"    ")?;
                serde_json::to_writer(&mut *writer, &EdgeExport {
                    from: from.id,
                    to: to.id,
                    relationship: match edge.weight() {
                        Relationship::BorrowsImmut { at } => 
                            RelationshipExport::BorrowsImmut { at: *at },
                        Relationship::BorrowsMut { at } => 
                            RelationshipExport::BorrowsMut { at: *at },
                        Relationship::Moves { at } => 
                            RelationshipExport::Moves { at: *at },
                        Relationship::RcClone { at, strong_count } => 
                            RelationshipExport::RcClone { at: *at, strong_count: *strong_count },
                        Relationship::ArcClone { at, strong_count } => 
                            RelationshipExport::ArcClone { at: *at, strong_count: *strong_count },
                        Relationship::RefCellBorrow { at, is_mut } => 
                            RelationshipExport::RefCellBorrow { at: *at, is_mut: *is_mut },
                    },
                })?;
            }
        }
        
        writer.write_all(b"\n  ]\n}\n")?;
        Ok(())
    }
}
```

---

## Alternative Formats

### DOT Format (Graphviz)

```rust
impl OwnershipGraph {
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph OwnershipGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");
        
        // Nodes
        for var in self.graph.node_weights() {
            let color = if var.dropped_at.is_some() { "gray" } else { "black" };
            dot.push_str(&format!(
                "  {} [label=\"{}\\n{}\" color={}];\n",
                var.id, var.name, var.type_name, color
            ));
        }
        
        dot.push('\n');
        
        // Edges
        for edge in self.graph.edge_references() {
            if let (Some(from), Some(to)) = (
                self.graph.node_weight(edge.source()),
                self.graph.node_weight(edge.target())
            ) {
                let (label, color) = match edge.weight() {
                    Relationship::BorrowsImmut { .. } => ("&", "blue"),
                    Relationship::BorrowsMut { .. } => ("&mut", "red"),
                    Relationship::Moves { .. } => ("move", "green"),
                    Relationship::RcClone { .. } => ("Rc", "purple"),
                    Relationship::ArcClone { .. } => ("Arc", "orange"),
                    Relationship::RefCellBorrow { is_mut, .. } => {
                        if *is_mut { ("&mut (RefCell)", "red") } else { ("& (RefCell)", "blue") }
                    }
                };
                
                dot.push_str(&format!(
                    "  {} -> {} [label=\"{}\" color={}];\n",
                    from.id, to.id, label, color
                ));
            }
        }
        
        dot.push_str("}\n");
        dot
    }
}
```

### Binary Format (MessagePack)

```toml
[dependencies]
rmp-serde = "1.1"
```

```rust
impl OwnershipGraph {
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self.export())
    }
    
    pub fn from_msgpack(data: &[u8]) -> Result<GraphExport, rmp_serde::decode::Error> {
        rmp_serde::from_slice(data)
    }
}
```

---

## File Export

```rust
use std::fs::File;
use std::path::Path;

impl OwnershipGraph {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        self.stream_json(&mut writer)
    }
    
    pub fn save_compact<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = serde_json::to_string(&self.export_compact())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }
    
    pub fn save_dot<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::write(path, self.to_dot())
    }
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_json() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        
        let json = graph.to_json().unwrap();
        assert!(json.contains("\"name\": \"x\""));
        assert!(json.contains("\"type_name\": \"i32\""));
    }

    #[test]
    fn test_export_roundtrip() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        graph.add_variable(Variable {
            id: 2, name: "r".into(), type_name: "&i32".into(),
            created_at: 1050, dropped_at: Some(1200), scope_depth: 0,
        });
        graph.add_borrow(2, 1, false, 1050);
        
        let export = graph.export();
        let json = serde_json::to_string(&export).unwrap();
        let loaded: GraphExport = serde_json::from_str(&json).unwrap();
        
        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.edges.len(), 1);
    }

    #[test]
    fn test_dot_format() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        
        let dot = graph.to_dot();
        assert!(dot.contains("digraph OwnershipGraph"));
        assert!(dot.contains("x\\ni32"));
    }
}
```

---

## Performance Benchmarks

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_export_large_graph() {
        let mut graph = OwnershipGraph::new();
        
        // Create 1000 nodes
        for i in 0..1000 {
            graph.add_variable(Variable {
                id: i,
                name: format!("var_{}", i),
                type_name: "i32".into(),
                created_at: i as u64 * 100,
                dropped_at: None,
                scope_depth: 0,
            });
        }
        
        // Add edges
        for i in 1..1000 {
            graph.add_borrow(i, i - 1, false, i as u64 * 100);
        }
        
        let start = Instant::now();
        let _json = graph.to_json().unwrap();
        let duration = start.elapsed();
        
        println!("Export 1000 nodes: {:?}", duration);
        assert!(duration.as_millis() < 100);  // Should be fast
    }
}
```

---

## Key Takeaways

✅ **JSON format** - Standard, human-readable export  
✅ **Compact format** - Optimized for large graphs  
✅ **Streaming** - Handle graphs that don't fit in memory  
✅ **Multiple formats** - JSON, DOT, MessagePack  
✅ **File export** - Save to disk for UI consumption  

---

## Further Reading

- [serde documentation](https://serde.rs/)
- [JSON specification](https://www.json.org/)
- [DOT language](https://graphviz.org/doc/info/lang.html)

---

**Previous:** [71-detecting-borrow-conflicts.md](./71-detecting-borrow-conflicts.md)  
**Next:** [73-graph-queries-and-analysis.md](./73-graph-queries-and-analysis.md)

**Progress:** 7/10 ⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜
