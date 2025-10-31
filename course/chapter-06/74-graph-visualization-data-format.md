# Section 74: Graph Visualization Data Format

## Learning Objectives

By the end of this section, you will:
- Design visualization-friendly data formats
- Add layout hints for graph rendering
- Include styling metadata
- Optimize for UI consumption
- Prepare for Cytoscape.js integration

## Prerequisites

- Section 73 (Graph Queries and Analysis)
- Understanding of graph visualization libraries
- Basic knowledge of CSS/styling

---

## Visualization Requirements

**Goals:**
1. Provide node positions (or layout hints)
2. Include styling information (colors, sizes)
3. Support interactive features (tooltips, highlights)
4. Optimize for rendering performance
5. Compatible with Cytoscape.js and D3.js

---

## Cytoscape.js Format

Cytoscape.js expects this structure:

```json
{
  "elements": {
    "nodes": [
      { "data": { "id": "1", "label": "x" } }
    ],
    "edges": [
      { "data": { "id": "e1", "source": "2", "target": "1" } }
    ]
  }
}
```

---

## Enhanced Format

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VisualizationExport {
    pub elements: Elements,
    pub style: Vec<StyleRule>,
    pub layout: LayoutConfig,
}

#[derive(Serialize, Deserialize)]
pub struct Elements {
    pub nodes: Vec<NodeElement>,
    pub edges: Vec<EdgeElement>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeElement {
    pub data: NodeData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
    pub is_alive: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeElement {
    pub data: EdgeData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relationship: String,
    pub at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize)]
pub struct StyleRule {
    pub selector: String,
    pub style: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct LayoutConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}
```

---

## Implementation

```rust
impl OwnershipGraph {
    pub fn export_for_visualization(&self) -> VisualizationExport {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        let nodes = self.graph.node_weights()
            .map(|var| {
                let is_alive = var.dropped_at.map_or(true, |d| d > current_time);
                let classes = self.node_classes(var);
                
                NodeElement {
                    data: NodeData {
                        id: var.id.to_string(),
                        label: var.name.clone(),
                        type_name: var.type_name.clone(),
                        created_at: var.created_at,
                        dropped_at: var.dropped_at,
                        scope_depth: var.scope_depth,
                        is_alive,
                    },
                    position: None,  // Let layout algorithm decide
                    classes: Some(classes),
                }
            })
            .collect();
        
        let edges = self.graph.edge_references()
            .enumerate()
            .filter_map(|(i, edge)| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                
                let (relationship, at, extra, classes) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => 
                        ("immutable_borrow".into(), *at, None, "immutable"),
                    Relationship::BorrowsMut { at } => 
                        ("mutable_borrow".into(), *at, None, "mutable"),
                    Relationship::Moves { at } => 
                        ("move".into(), *at, None, "move"),
                    Relationship::RcClone { at, strong_count } => 
                        ("rc_clone".into(), *at, 
                         Some(serde_json::json!({"strong_count": strong_count})), "rc"),
                    Relationship::ArcClone { at, strong_count } => 
                        ("arc_clone".into(), *at, 
                         Some(serde_json::json!({"strong_count": strong_count})), "arc"),
                    Relationship::RefCellBorrow { at, is_mut } => 
                        (if *is_mut { "refcell_mut" } else { "refcell_immut" }.into(), 
                         *at, None, "refcell"),
                };
                
                Some(EdgeElement {
                    data: EdgeData {
                        id: format!("e{}", i),
                        source: from.id.to_string(),
                        target: to.id.to_string(),
                        relationship,
                        at,
                        extra,
                    },
                    classes: Some(classes.into()),
                })
            })
            .collect();
        
        VisualizationExport {
            elements: Elements { nodes, edges },
            style: self.default_styles(),
            layout: LayoutConfig {
                name: "dagre".into(),
                options: Some(serde_json::json!({
                    "rankDir": "LR",
                    "nodeSep": 50,
                    "rankSep": 100
                })),
            },
        }
    }
    
    fn node_classes(&self, var: &Variable) -> String {
        let mut classes = vec![];
        
        if var.dropped_at.is_some() {
            classes.push("dropped");
        } else {
            classes.push("alive");
        }
        
        if var.type_name.starts_with("&mut") {
            classes.push("mutable-ref");
        } else if var.type_name.starts_with('&') {
            classes.push("immutable-ref");
        } else {
            classes.push("owned");
        }
        
        if var.type_name.contains("Rc<") {
            classes.push("rc");
        } else if var.type_name.contains("Arc<") {
            classes.push("arc");
        } else if var.type_name.contains("RefCell<") {
            classes.push("refcell");
        }
        
        classes.join(" ")
    }
    
    fn default_styles(&self) -> Vec<StyleRule> {
        vec![
            StyleRule {
                selector: "node".into(),
                style: serde_json::json!({
                    "label": "data(label)",
                    "text-valign": "center",
                    "text-halign": "center",
                    "background-color": "#3498db",
                    "color": "#fff",
                    "font-size": "12px",
                    "width": "60px",
                    "height": "60px",
                    "border-width": "2px",
                    "border-color": "#2980b9"
                }),
            },
            StyleRule {
                selector: "node.dropped".into(),
                style: serde_json::json!({
                    "background-color": "#95a5a6",
                    "border-color": "#7f8c8d",
                    "opacity": 0.6
                }),
            },
            StyleRule {
                selector: "node.mutable-ref".into(),
                style: serde_json::json!({
                    "background-color": "#e74c3c",
                    "border-color": "#c0392b"
                }),
            },
            StyleRule {
                selector: "node.immutable-ref".into(),
                style: serde_json::json!({
                    "background-color": "#2ecc71",
                    "border-color": "#27ae60"
                }),
            },
            StyleRule {
                selector: "node.rc".into(),
                style: serde_json::json!({
                    "background-color": "#9b59b6",
                    "border-color": "#8e44ad"
                }),
            },
            StyleRule {
                selector: "node.arc".into(),
                style: serde_json::json!({
                    "background-color": "#f39c12",
                    "border-color": "#e67e22"
                }),
            },
            StyleRule {
                selector: "edge".into(),
                style: serde_json::json!({
                    "width": 2,
                    "line-color": "#95a5a6",
                    "target-arrow-color": "#95a5a6",
                    "target-arrow-shape": "triangle",
                    "curve-style": "bezier"
                }),
            },
            StyleRule {
                selector: "edge.immutable".into(),
                style: serde_json::json!({
                    "line-color": "#2ecc71",
                    "target-arrow-color": "#2ecc71"
                }),
            },
            StyleRule {
                selector: "edge.mutable".into(),
                style: serde_json::json!({
                    "line-color": "#e74c3c",
                    "target-arrow-color": "#e74c3c",
                    "width": 3
                }),
            },
            StyleRule {
                selector: "edge.move".into(),
                style: serde_json::json!({
                    "line-color": "#f39c12",
                    "target-arrow-color": "#f39c12",
                    "line-style": "dashed"
                }),
            },
        ]
    }
}
```

---

## Example Output

```json
{
  "elements": {
    "nodes": [
      {
        "data": {
          "id": "1",
          "label": "x",
          "type": "i32",
          "created_at": 1000,
          "dropped_at": null,
          "scope_depth": 0,
          "is_alive": true
        },
        "classes": "alive owned"
      },
      {
        "data": {
          "id": "2",
          "label": "r",
          "type": "&i32",
          "created_at": 1050,
          "dropped_at": 1200,
          "scope_depth": 0,
          "is_alive": false
        },
        "classes": "dropped immutable-ref"
      }
    ],
    "edges": [
      {
        "data": {
          "id": "e0",
          "source": "2",
          "target": "1",
          "relationship": "immutable_borrow",
          "at": 1050
        },
        "classes": "immutable"
      }
    ]
  },
  "style": [
    {
      "selector": "node",
      "style": {
        "label": "data(label)",
        "background-color": "#3498db"
      }
    }
  ],
  "layout": {
    "name": "dagre",
    "options": {
      "rankDir": "LR",
      "nodeSep": 50,
      "rankSep": 100
    }
  }
}
```

---

## Layout Algorithms

### Hierarchical (Dagre)

```rust
impl LayoutConfig {
    pub fn dagre() -> Self {
        Self {
            name: "dagre".into(),
            options: Some(serde_json::json!({
                "rankDir": "LR",  // Left to right
                "nodeSep": 50,
                "rankSep": 100,
                "ranker": "network-simplex"
            })),
        }
    }
}
```

### Force-Directed (Cola)

```rust
impl LayoutConfig {
    pub fn cola() -> Self {
        Self {
            name: "cola".into(),
            options: Some(serde_json::json!({
                "animate": true,
                "maxSimulationTime": 2000,
                "nodeSpacing": 50,
                "edgeLength": 100
            })),
        }
    }
}
```

### Circular

```rust
impl LayoutConfig {
    pub fn circular() -> Self {
        Self {
            name: "circle".into(),
            options: Some(serde_json::json!({
                "radius": 200,
                "startAngle": 0,
                "sweep": 6.28  // 2π
            })),
        }
    }
}
```

---

## Interactive Features

### Tooltips

```rust
#[derive(Serialize)]
pub struct TooltipData {
    pub title: String,
    pub details: Vec<(String, String)>,
}

impl NodeData {
    pub fn tooltip(&self) -> TooltipData {
        let mut details = vec![
            ("Type".into(), self.type_name.clone()),
            ("Created".into(), format!("{}μs", self.created_at)),
        ];
        
        if let Some(dropped) = self.dropped_at {
            details.push(("Dropped".into(), format!("{}μs", dropped)));
            details.push(("Lifetime".into(), format!("{}μs", dropped - self.created_at)));
        } else {
            details.push(("Status".into(), "Alive".into()));
        }
        
        details.push(("Scope Depth".into(), self.scope_depth.to_string()));
        
        TooltipData {
            title: self.label.clone(),
            details,
        }
    }
}
```

### Highlighting

```rust
#[derive(Serialize)]
pub struct HighlightConfig {
    pub node_id: String,
    pub highlight_neighbors: bool,
    pub highlight_path: Option<Vec<String>>,
}

impl OwnershipGraph {
    pub fn highlight_borrowers(&self, id: usize) -> HighlightConfig {
        let borrowers = self.query().direct_borrowers(id);
        let path = borrowers.iter()
            .map(|(v, _)| v.id.to_string())
            .collect();
        
        HighlightConfig {
            node_id: id.to_string(),
            highlight_neighbors: true,
            highlight_path: Some(path),
        }
    }
}
```

---

## Time-Based Visualization

```rust
#[derive(Serialize)]
pub struct TimelineFrame {
    pub timestamp: u64,
    pub elements: Elements,
}

impl OwnershipGraph {
    pub fn export_timeline(&self) -> Vec<TimelineFrame> {
        let mut timestamps = std::collections::BTreeSet::new();
        
        for var in self.graph.node_weights() {
            timestamps.insert(var.created_at);
            if let Some(dropped) = var.dropped_at {
                timestamps.insert(dropped);
            }
        }
        
        timestamps.iter()
            .map(|&time| {
                TimelineFrame {
                    timestamp: time,
                    elements: self.elements_at_time(time),
                }
            })
            .collect()
    }
    
    fn elements_at_time(&self, time: u64) -> Elements {
        let nodes = self.graph.node_weights()
            .filter(|v| v.created_at <= time)
            .map(|var| {
                let is_alive = var.dropped_at.map_or(true, |d| d > time);
                NodeElement {
                    data: NodeData {
                        id: var.id.to_string(),
                        label: var.name.clone(),
                        type_name: var.type_name.clone(),
                        created_at: var.created_at,
                        dropped_at: var.dropped_at,
                        scope_depth: var.scope_depth,
                        is_alive,
                    },
                    position: None,
                    classes: Some(if is_alive { "alive" } else { "dropped" }.into()),
                }
            })
            .collect();
        
        let edges = self.graph.edge_references()
            .enumerate()
            .filter_map(|(i, edge)| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                
                let edge_time = match edge.weight() {
                    Relationship::BorrowsImmut { at } => *at,
                    Relationship::BorrowsMut { at } => *at,
                    Relationship::Moves { at } => *at,
                    Relationship::RcClone { at, .. } => *at,
                    Relationship::ArcClone { at, .. } => *at,
                    Relationship::RefCellBorrow { at, .. } => *at,
                };
                
                if edge_time > time {
                    return None;
                }
                
                Some(EdgeElement {
                    data: EdgeData {
                        id: format!("e{}", i),
                        source: from.id.to_string(),
                        target: to.id.to_string(),
                        relationship: "borrow".into(),
                        at: edge_time,
                        extra: None,
                    },
                    classes: None,
                })
            })
            .collect();
        
        Elements { nodes, edges }
    }
}
```

---

## D3.js Format

```rust
#[derive(Serialize)]
pub struct D3Export {
    pub nodes: Vec<D3Node>,
    pub links: Vec<D3Link>,
}

#[derive(Serialize)]
pub struct D3Node {
    pub id: String,
    pub label: String,
    pub group: usize,
}

#[derive(Serialize)]
pub struct D3Link {
    pub source: String,
    pub target: String,
    pub value: usize,
}

impl OwnershipGraph {
    pub fn export_for_d3(&self) -> D3Export {
        let nodes = self.graph.node_weights()
            .map(|var| D3Node {
                id: var.id.to_string(),
                label: var.name.clone(),
                group: var.scope_depth,
            })
            .collect();
        
        let links = self.graph.edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;
                
                Some(D3Link {
                    source: from.id.to_string(),
                    target: to.id.to_string(),
                    value: 1,
                })
            })
            .collect();
        
        D3Export { nodes, links }
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
    fn test_visualization_export() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: None, scope_depth: 0,
        });
        
        let viz = graph.export_for_visualization();
        assert_eq!(viz.elements.nodes.len(), 1);
        assert_eq!(viz.elements.nodes[0].data.label, "x");
    }

    #[test]
    fn test_node_classes() {
        let mut graph = OwnershipGraph::new();
        
        let var = Variable {
            id: 1, name: "r".into(), type_name: "&mut i32".into(),
            created_at: 1000, dropped_at: Some(2000), scope_depth: 0,
        };
        
        let classes = graph.node_classes(&var);
        assert!(classes.contains("dropped"));
        assert!(classes.contains("mutable-ref"));
    }

    #[test]
    fn test_timeline_export() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: 1, name: "x".into(), type_name: "i32".into(),
            created_at: 1000, dropped_at: Some(2000), scope_depth: 0,
        });
        
        let timeline = graph.export_timeline();
        assert_eq!(timeline.len(), 2);  // Created and dropped
    }
}
```

---

## Key Takeaways

✅ **Cytoscape.js format** - Standard graph visualization structure  
✅ **Styling metadata** - Colors, classes for visual distinction  
✅ **Layout hints** - Algorithm configuration for positioning  
✅ **Interactive data** - Tooltips, highlighting support  
✅ **Timeline export** - Frame-by-frame visualization  

---

## Further Reading

- [Cytoscape.js documentation](https://js.cytoscape.org/)
- [D3.js force layout](https://d3js.org/d3-force)
- [Dagre layout algorithm](https://github.com/dagrejs/dagre)

---

**Previous:** [73-graph-queries-and-analysis.md](./73-graph-queries-and-analysis.md)  
**Next:** [75-optimizing-graph-performance.md](./75-optimizing-graph-performance.md)

**Progress:** 9/10 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜
