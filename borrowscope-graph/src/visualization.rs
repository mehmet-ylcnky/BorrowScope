use crate::{OwnershipGraph, Relationship, Variable};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualizationExport {
    pub elements: Elements,
    pub style: Vec<StyleRule>,
    pub layout: LayoutConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Elements {
    pub nodes: Vec<NodeElement>,
    pub edges: Vec<EdgeElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeElement {
    pub data: NodeData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeElement {
    pub data: EdgeData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relationship: String,
    pub at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StyleRule {
    pub selector: String,
    pub style: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct TooltipData {
    pub title: String,
    pub details: Vec<(String, String)>,
}

#[derive(Serialize, Debug)]
pub struct HighlightConfig {
    pub node_id: String,
    pub highlight_neighbors: bool,
    pub highlight_path: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct TimelineFrame {
    pub timestamp: u64,
    pub elements: Elements,
}

#[derive(Serialize, Debug)]
pub struct D3Export {
    pub nodes: Vec<D3Node>,
    pub links: Vec<D3Link>,
}

#[derive(Serialize, Debug)]
pub struct D3Node {
    pub id: String,
    pub label: String,
    pub group: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct D3Link {
    pub source: String,
    pub target: String,
    pub value: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl OwnershipGraph {
    pub fn export_for_visualization(&self) -> VisualizationExport {
        self.export_for_visualization_at(None)
    }

    pub fn export_for_visualization_at(&self, current_time: Option<u64>) -> VisualizationExport {
        let time = current_time.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        });

        let nodes = self
            .graph
            .node_weights()
            .map(|var| {
                let is_alive = var.dropped_at.map_or(true, |d| d > time);
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
                    position: None,
                    classes: Some(classes),
                }
            })
            .collect();

        let edges = self
            .graph
            .edge_references()
            .enumerate()
            .filter_map(|(i, edge)| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;

                let (relationship, at, extra, classes) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => {
                        ("immutable_borrow".into(), *at, None, "immutable")
                    }
                    Relationship::BorrowsMut { at } => {
                        ("mutable_borrow".into(), *at, None, "mutable")
                    }
                    Relationship::Moves { at } => ("move".into(), *at, None, "move"),
                    Relationship::RcClone { at, strong_count } => (
                        "rc_clone".into(),
                        *at,
                        Some(serde_json::json!({"strong_count": strong_count})),
                        "rc",
                    ),
                    Relationship::ArcClone { at, strong_count } => (
                        "arc_clone".into(),
                        *at,
                        Some(serde_json::json!({"strong_count": strong_count})),
                        "arc",
                    ),
                    Relationship::RefCellBorrow { at, is_mut } => (
                        if *is_mut {
                            "refcell_mut"
                        } else {
                            "refcell_immut"
                        }
                        .into(),
                        *at,
                        None,
                        "refcell",
                    ),
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
            style: Self::default_styles(),
            layout: LayoutConfig::dagre(),
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
        } else if var.type_name.contains("Box<") {
            classes.push("box");
        }

        classes.join(" ")
    }

    fn default_styles() -> Vec<StyleRule> {
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
                selector: "node.box".into(),
                style: serde_json::json!({
                    "background-color": "#1abc9c",
                    "border-color": "#16a085"
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

    pub fn export_timeline(&self) -> Vec<TimelineFrame> {
        let mut timestamps = std::collections::BTreeSet::new();

        for var in self.graph.node_weights() {
            timestamps.insert(var.created_at);
            if let Some(dropped) = var.dropped_at {
                timestamps.insert(dropped);
            }
        }

        for edge in self.graph.edge_references() {
            let at = match edge.weight() {
                Relationship::BorrowsImmut { at } => *at,
                Relationship::BorrowsMut { at } => *at,
                Relationship::Moves { at } => *at,
                Relationship::RcClone { at, .. } => *at,
                Relationship::ArcClone { at, .. } => *at,
                Relationship::RefCellBorrow { at, .. } => *at,
            };
            timestamps.insert(at);
        }

        timestamps
            .iter()
            .map(|&time| TimelineFrame {
                timestamp: time,
                elements: self.elements_at_time(time),
            })
            .collect()
    }

    fn elements_at_time(&self, time: u64) -> Elements {
        let nodes = self
            .graph
            .node_weights()
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

        let edges = self
            .graph
            .edge_references()
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

                if edge_time > time || from.created_at > time || to.created_at > time {
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

    pub fn export_for_d3(&self) -> D3Export {
        let nodes = self
            .graph
            .node_weights()
            .map(|var| D3Node {
                id: var.id.to_string(),
                label: var.name.clone(),
                group: var.scope_depth,
                metadata: Some(serde_json::json!({
                    "type": var.type_name,
                    "created_at": var.created_at,
                    "dropped_at": var.dropped_at,
                })),
            })
            .collect();

        let links = self
            .graph
            .edge_references()
            .filter_map(|edge| {
                let from = self.graph.node_weight(edge.source())?;
                let to = self.graph.node_weight(edge.target())?;

                let (rel_type, at) = match edge.weight() {
                    Relationship::BorrowsImmut { at } => ("immutable", at),
                    Relationship::BorrowsMut { at } => ("mutable", at),
                    Relationship::Moves { at } => ("move", at),
                    Relationship::RcClone { at, .. } => ("rc", at),
                    Relationship::ArcClone { at, .. } => ("arc", at),
                    Relationship::RefCellBorrow { at, .. } => ("refcell", at),
                };

                Some(D3Link {
                    source: from.id.to_string(),
                    target: to.id.to_string(),
                    value: 1,
                    metadata: Some(serde_json::json!({
                        "type": rel_type,
                        "at": at,
                    })),
                })
            })
            .collect();

        D3Export { nodes, links }
    }

    pub fn highlight_borrowers(&self, id: usize) -> Option<HighlightConfig> {
        if !self.id_to_node.contains_key(&id) {
            return None;
        }

        let borrowers = self.borrowers_of(id);
        let path = borrowers.iter().map(|v| v.id.to_string()).collect();

        Some(HighlightConfig {
            node_id: id.to_string(),
            highlight_neighbors: true,
            highlight_path: Some(path),
        })
    }
}

impl NodeData {
    pub fn tooltip(&self) -> TooltipData {
        let mut details = vec![
            ("Type".into(), self.type_name.clone()),
            ("Created".into(), format!("{}μs", self.created_at)),
        ];

        if let Some(dropped) = self.dropped_at {
            details.push(("Dropped".into(), format!("{}μs", dropped)));
            details.push((
                "Lifetime".into(),
                format!("{}μs", dropped - self.created_at),
            ));
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

impl LayoutConfig {
    pub fn dagre() -> Self {
        Self {
            name: "dagre".into(),
            options: Some(serde_json::json!({
                "rankDir": "LR",
                "nodeSep": 50,
                "rankSep": 100,
                "ranker": "network-simplex"
            })),
        }
    }

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

    pub fn circular() -> Self {
        Self {
            name: "circle".into(),
            options: Some(serde_json::json!({
                "radius": 200,
                "startAngle": 0,
                "sweep": std::f64::consts::TAU
            })),
        }
    }

    pub fn grid() -> Self {
        Self {
            name: "grid".into(),
            options: Some(serde_json::json!({
                "rows": 5,
                "cols": 5,
                "position": null
            })),
        }
    }

    pub fn breadthfirst() -> Self {
        Self {
            name: "breadthfirst".into(),
            options: Some(serde_json::json!({
                "directed": true,
                "spacingFactor": 1.5
            })),
        }
    }
}
