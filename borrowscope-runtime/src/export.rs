//! Export functionality for graphs and events

use crate::event::Event;
use crate::graph::{OwnershipGraph, Relationship, Variable};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Complete export format optimized for visualization
#[derive(Debug, Serialize)]
pub struct ExportData {
    pub nodes: Vec<Variable>,
    pub edges: Vec<ExportEdge>,
    pub events: Vec<Event>,
    pub metadata: ExportMetadata,
}

/// Serializable edge for export
#[derive(Debug, Serialize)]
pub struct ExportEdge {
    pub from: String,
    pub to: String,
    pub relationship: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

/// Export metadata
#[derive(Debug, Serialize)]
pub struct ExportMetadata {
    pub total_variables: usize,
    pub total_relationships: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub total_events: usize,
}

impl ExportData {
    /// Create export data from graph and events
    pub fn new(graph: OwnershipGraph, events: Vec<Event>) -> Self {
        let stats = graph.stats();
        let edges = graph
            .edges
            .iter()
            .map(|rel| match rel {
                Relationship::Owns { from, to } => ExportEdge {
                    from: from.clone(),
                    to: to.clone(),
                    relationship: "owns".to_string(),
                    start: None,
                    end: None,
                },
                Relationship::BorrowsImmut {
                    from,
                    to,
                    start,
                    end,
                } => ExportEdge {
                    from: from.clone(),
                    to: to.clone(),
                    relationship: "borrows_immut".to_string(),
                    start: Some(*start),
                    end: Some(*end),
                },
                Relationship::BorrowsMut {
                    from,
                    to,
                    start,
                    end,
                } => ExportEdge {
                    from: from.clone(),
                    to: to.clone(),
                    relationship: "borrows_mut".to_string(),
                    start: Some(*start),
                    end: Some(*end),
                },
            })
            .collect();

        ExportData {
            nodes: graph.nodes,
            edges,
            events: events.clone(),
            metadata: ExportMetadata {
                total_variables: stats.total_variables,
                total_relationships: stats.total_relationships,
                immutable_borrows: stats.immutable_borrows,
                mutable_borrows: stats.mutable_borrows,
                total_events: events.len(),
            },
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to JSON file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = self.to_json().map_err(std::io::Error::other)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::build_graph;

    #[test]
    fn test_export_empty() {
        let graph = OwnershipGraph::new();
        let events = vec![];
        let export = ExportData::new(graph, events);

        assert_eq!(export.nodes.len(), 0);
        assert_eq!(export.edges.len(), 0);
        assert_eq!(export.metadata.total_events, 0);
    }

    #[test]
    fn test_export_with_variable() {
        let events = vec![Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        }];

        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        assert_eq!(export.nodes.len(), 1);
        assert_eq!(export.nodes[0].name, "x");
        assert_eq!(export.metadata.total_events, 1);
    }

    #[test]
    fn test_export_with_borrow() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Borrow {
                timestamp: 2,
                borrower_name: "r".to_string(),
                borrower_id: "r_1".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Drop {
                timestamp: 3,
                var_id: "r_1".to_string(),
            },
        ];

        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        assert_eq!(export.edges.len(), 1);
        assert_eq!(export.edges[0].relationship, "borrows_immut");
        assert_eq!(export.metadata.immutable_borrows, 1);
    }

    #[test]
    fn test_json_serialization() {
        let events = vec![Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        }];

        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);
        let json = export.to_json().unwrap();

        assert!(json.contains("\"name\": \"x\""));
        assert!(json.contains("\"type_name\": \"i32\""));
    }

    #[test]
    fn test_json_deserialization() {
        let events = vec![Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        }];

        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);
        let json = export.to_json().unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["nodes"].is_array());
        assert!(parsed["edges"].is_array());
        assert!(parsed["events"].is_array());
        assert!(parsed["metadata"].is_object());
    }

    #[test]
    fn test_metadata() {
        let events = vec![
            Event::New {
                timestamp: 1,
                var_name: "x".to_string(),
                var_id: "x_0".to_string(),
                type_name: "i32".to_string(),
            },
            Event::Borrow {
                timestamp: 2,
                borrower_name: "r".to_string(),
                borrower_id: "r_1".to_string(),
                owner_id: "x_0".to_string(),
                mutable: false,
            },
            Event::Borrow {
                timestamp: 3,
                borrower_name: "s".to_string(),
                borrower_id: "s_2".to_string(),
                owner_id: "x_0".to_string(),
                mutable: true,
            },
            Event::Drop {
                timestamp: 4,
                var_id: "r_1".to_string(),
            },
            Event::Drop {
                timestamp: 5,
                var_id: "s_2".to_string(),
            },
        ];

        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);

        assert_eq!(export.metadata.total_variables, 1);
        assert_eq!(export.metadata.total_relationships, 2);
        assert_eq!(export.metadata.immutable_borrows, 1);
        assert_eq!(export.metadata.mutable_borrows, 1);
        assert_eq!(export.metadata.total_events, 5);
    }
}
