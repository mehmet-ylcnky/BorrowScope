//! Check command implementation

use std::fs;

use crate::cli::CheckArgs;
use crate::error::{CliError, Result};

pub fn execute(args: CheckArgs) -> Result<()> {
    log::info!("Checking: {}", args.file.display());

    // Check if file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    // Load tracking data
    let contents = fs::read_to_string(&args.file)?;
    let data: serde_json::Value = serde_json::from_str(&contents)?;

    // Validate structure
    validate_structure(&data)?;

    // Show statistics if requested
    if args.stats {
        show_statistics(&data)?;
    }

    // Check for conflicts if requested
    if args.conflicts {
        check_conflicts(&data)?;
    }

    // Validate graph integrity if requested
    if args.validate {
        validate_graph(&data)?;
    }

    // If no specific checks requested, do basic validation
    if !args.stats && !args.conflicts && !args.validate {
        println!("✓ Tracking data is valid");
    }

    Ok(())
}

fn validate_structure(data: &serde_json::Value) -> Result<()> {
    // Check required fields
    if data.get("version").is_none() {
        return Err(CliError::InvalidFormat("Missing 'version' field".to_string()));
    }

    if data.get("events").is_none() {
        return Err(CliError::InvalidFormat("Missing 'events' field".to_string()));
    }

    if data.get("graph").is_none() {
        return Err(CliError::InvalidFormat("Missing 'graph' field".to_string()));
    }

    Ok(())
}

fn show_statistics(data: &serde_json::Value) -> Result<()> {
    println!("\n📊 Statistics:");

    // Count events
    let empty_vec = vec![];
    let events = data.get("events").and_then(|e| e.as_array()).unwrap_or(&empty_vec);
    println!("  Total events: {}", events.len());

    // Count event types
    let mut new_count = 0;
    let mut borrow_count = 0;
    let mut move_count = 0;
    let mut drop_count = 0;

    for event in events {
        if let Some(event_type) = event.get("type").and_then(|t| t.as_str()) {
            match event_type {
                "New" => new_count += 1,
                "Borrow" => borrow_count += 1,
                "Move" => move_count += 1,
                "Drop" => drop_count += 1,
                _ => {}
            }
        }
    }

    println!("  - New: {}", new_count);
    println!("  - Borrow: {}", borrow_count);
    println!("  - Move: {}", move_count);
    println!("  - Drop: {}", drop_count);

    // Count graph elements
    if let Some(graph) = data.get("graph") {
        let empty_nodes = vec![];
        let empty_edges = vec![];
        let nodes = graph.get("nodes").and_then(|n| n.as_array()).unwrap_or(&empty_nodes);
        let edges = graph.get("edges").and_then(|e| e.as_array()).unwrap_or(&empty_edges);

        println!("\n  Graph:");
        println!("  - Nodes: {}", nodes.len());
        println!("  - Edges: {}", edges.len());
    }

    Ok(())
}

fn check_conflicts(data: &serde_json::Value) -> Result<()> {
    println!("\n🔍 Checking for borrow conflicts...");

    let empty_vec = vec![];
    let events = data.get("events").and_then(|e| e.as_array()).unwrap_or(&empty_vec);

    // Simple conflict detection: check for multiple mutable borrows
    let mut active_borrows: std::collections::HashMap<String, Vec<(usize, bool)>> = std::collections::HashMap::new();
    let mut conflicts = Vec::new();

    for (idx, event) in events.iter().enumerate() {
        if let Some(event_type) = event.get("type").and_then(|t| t.as_str()) {
            match event_type {
                "Borrow" => {
                    if let (Some(owner_id), Some(is_mutable)) = (
                        event.get("owner_id").and_then(|id| id.as_str()),
                        event.get("mutable").and_then(|m| m.as_bool()),
                    ) {
                        let borrows = active_borrows.entry(owner_id.to_string()).or_default();

                        // Check for conflicts
                        if is_mutable && !borrows.is_empty() {
                            conflicts.push(format!(
                                "Mutable borrow conflict at event {}: variable {} already borrowed",
                                idx, owner_id
                            ));
                        } else if !is_mutable && borrows.iter().any(|(_, m)| *m) {
                            conflicts.push(format!(
                                "Borrow conflict at event {}: variable {} has active mutable borrow",
                                idx, owner_id
                            ));
                        }

                        borrows.push((idx, is_mutable));
                    }
                }
                "Drop" => {
                    if let Some(var_id) = event.get("var_id").and_then(|id| id.as_str()) {
                        active_borrows.remove(var_id);
                    }
                }
                _ => {}
            }
        }
    }

    if conflicts.is_empty() {
        println!("  ✓ No conflicts detected");
    } else {
        println!("  ✗ Found {} conflict(s):", conflicts.len());
        for conflict in conflicts {
            println!("    - {}", conflict);
        }
    }

    Ok(())
}

fn validate_graph(data: &serde_json::Value) -> Result<()> {
    println!("\n✓ Validating graph integrity...");

    if let Some(graph) = data.get("graph") {
        let empty_nodes = vec![];
        let empty_edges = vec![];
        let nodes = graph.get("nodes").and_then(|n| n.as_array()).unwrap_or(&empty_nodes);
        let edges = graph.get("edges").and_then(|e| e.as_array()).unwrap_or(&empty_edges);

        // Collect node IDs
        let mut node_ids = std::collections::HashSet::new();
        for node in nodes {
            if let Some(id) = node.get("id") {
                node_ids.insert(id.to_string());
            }
        }

        // Validate edges reference existing nodes
        let mut invalid_edges = 0;
        for edge in edges {
            if let (Some(from), Some(to)) = (edge.get("from"), edge.get("to")) {
                if !node_ids.contains(&from.to_string()) || !node_ids.contains(&to.to_string()) {
                    invalid_edges += 1;
                }
            }
        }

        if invalid_edges == 0 {
            println!("  ✓ Graph is valid");
        } else {
            println!("  ✗ Found {} invalid edge(s)", invalid_edges);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_check_nonexistent_file() {
        let args = CheckArgs {
            file: PathBuf::from("/nonexistent/file.json"),
            conflicts: false,
            stats: false,
            validate: false,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {
                "nodes": [],
                "edges": []
            }
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = CheckArgs {
            file: test_file,
            conflicts: false,
            stats: false,
            validate: false,
        };

        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_invalid_structure() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "invalid": "structure"
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = CheckArgs {
            file: test_file,
            conflicts: false,
            stats: false,
            validate: false,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_show_statistics() {
        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [
                {"type": "New"},
                {"type": "Borrow"},
                {"type": "Drop"}
            ],
            "graph": {
                "nodes": [{"id": 1}],
                "edges": []
            }
        });

        let result = show_statistics(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_graph() {
        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {
                "nodes": [
                    {"id": 1},
                    {"id": 2}
                ],
                "edges": [
                    {"from": 1, "to": 2}
                ]
            }
        });

        let result = validate_graph(&data);
        assert!(result.is_ok());
    }
}
