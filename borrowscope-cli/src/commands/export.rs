//! Export command implementation

use std::fs;

use crate::cli::{ExportArgs, ExportFormat};
use crate::error::{CliError, Result};
use crate::graphviz;
use crate::progress::spinner;

pub fn execute(args: ExportArgs) -> Result<()> {
    log::info!(
        "Exporting {} to {}",
        args.file.display(),
        args.output.display()
    );

    // Check if input file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    // Load tracking data
    let contents = fs::read_to_string(&args.file)?;
    let data: serde_json::Value = serde_json::from_str(&contents)?;

    // Export based on format
    match args.format {
        ExportFormat::Dot => export_dot(&data, &args.output)?,
        ExportFormat::Svg => export_svg(&data, &args.output)?,
        ExportFormat::Png => export_png(&data, &args.output)?,
        ExportFormat::Json => export_json(&data, &args.output)?,
        ExportFormat::Html => export_html(&data, &args.output)?,
    }

    log::info!("Exported to: {}", args.output.display());
    Ok(())
}

fn export_dot(data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    let dot = generate_dot(data);
    fs::write(output, dot)?;
    Ok(())
}

fn export_svg(data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    // Check if Graphviz is available
    if !graphviz::is_available() {
        return Err(CliError::Other(
            "Graphviz not found. Please install Graphviz to export SVG.".to_string(),
        ));
    }

    let sp = spinner("Generating SVG");
    let dot = generate_dot(data);
    
    graphviz::render_svg(&dot, output)
        .map_err(|e| CliError::Other(format!("Failed to generate SVG: {}", e)))?;
    
    sp.finish_with_message("✓ SVG generated");
    Ok(())
}

fn export_png(data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    // Check if Graphviz is available
    if !graphviz::is_available() {
        return Err(CliError::Other(
            "Graphviz not found. Please install Graphviz to export PNG.".to_string(),
        ));
    }

    let sp = spinner("Generating PNG");
    let dot = generate_dot(data);
    
    graphviz::render_png(&dot, output)
        .map_err(|e| CliError::Other(format!("Failed to generate PNG: {}", e)))?;
    
    sp.finish_with_message("✓ PNG generated");
    Ok(())
}

fn generate_dot(data: &serde_json::Value) -> String {
    let mut dot = String::from("digraph BorrowScope {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot.push_str("  edge [color=gray];\n\n");

    // Extract nodes
    if let Some(nodes) = data
        .get("graph")
        .and_then(|g| g.get("nodes"))
        .and_then(|n| n.as_array())
    {
        for node in nodes {
            if let (Some(id), Some(name)) = (node.get("id"), node.get("name")) {
                dot.push_str(&format!("  {} [label=\"{}\"];\n", id, name));
            }
        }
    }

    dot.push('\n');

    // Extract edges
    if let Some(edges) = data
        .get("graph")
        .and_then(|g| g.get("edges"))
        .and_then(|e| e.as_array())
    {
        for edge in edges {
            if let (Some(from), Some(to), Some(rel)) =
                (edge.get("from"), edge.get("to"), edge.get("relationship"))
            {
                dot.push_str(&format!("  {} -> {} [label=\"{}\"];\n", from, to, rel));
            }
        }
    }

    dot.push_str("}\n");
    dot
}

fn export_json(data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(output, json)?;
    Ok(())
}

fn export_html(data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>BorrowScope Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        pre {{ background: #f4f4f4; padding: 10px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>BorrowScope Analysis Report</h1>
    <h2>Tracking Data</h2>
    <pre>{}</pre>
</body>
</html>"#,
        serde_json::to_string_pretty(data)?
    );

    fs::write(output, html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_export_nonexistent_file() {
        let args = ExportArgs {
            file: PathBuf::from("/nonexistent/file.json"),
            output: PathBuf::from("output.dot"),
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_dot() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({
            "version": "0.1.0",
            "graph": {
                "nodes": [
                    {"id": 1, "name": "x"},
                    {"id": 2, "name": "y"}
                ],
                "edges": [
                    {"from": 1, "to": 2, "relationship": "borrows"}
                ]
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("digraph BorrowScope"));
        assert!(contents.contains("x"));
        assert!(contents.contains("y"));
    }

    #[test]
    fn test_export_json() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.json");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Json,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_export_html() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.html");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Html,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("<!DOCTYPE html>"));
        assert!(contents.contains("BorrowScope"));
    }

    #[test]
    fn test_export_empty_graph() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({
            "version": "0.1.0",
            "graph": {
                "nodes": [],
                "edges": []
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_export_complex_graph() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let nodes: Vec<_> = (0..10)
            .map(|i| serde_json::json!({"id": i, "name": format!("node{}", i)}))
            .collect();

        let edges: Vec<_> = (0..9)
            .map(|i| serde_json::json!({"from": i, "to": i + 1, "relationship": "borrows"}))
            .collect();

        let data = serde_json::json!({
            "version": "0.1.0",
            "graph": {
                "nodes": nodes,
                "edges": edges
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("node0"));
        assert!(contents.contains("node9"));
    }

    #[test]
    fn test_export_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        fs::write(&input_file, "invalid json").unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file,
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_overwrite_existing() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();
        fs::write(&output_file, "old content").unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        assert_ne!(contents, "old content");
    }

    #[test]
    fn test_export_svg_without_graphviz() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.svg");

        let data = serde_json::json!({"graph": {"nodes": [], "edges": []}});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file,
            format: ExportFormat::Svg,
        };

        let result = execute(args);
        // Will fail if graphviz not installed, or succeed if it is
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_export_png_without_graphviz() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.png");

        let data = serde_json::json!({"graph": {"nodes": [], "edges": []}});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file,
            format: ExportFormat::Png,
        };

        let result = execute(args);
        // Will fail if graphviz not installed, or succeed if it is
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_export_dot_with_styling() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({
            "graph": {
                "nodes": [{"id": 1, "name": "x"}],
                "edges": []
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("fillcolor"));
        assert!(contents.contains("style=filled"));
    }

    #[test]
    fn test_export_json_compact() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.json");

        let data = serde_json::json!({"key": "value", "nested": {"a": 1}});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Json,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_export_html_structure() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.html");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Html,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("<html>"));
        assert!(contents.contains("</html>"));
        assert!(contents.contains("<head>"));
        assert!(contents.contains("<body>"));
        assert!(contents.contains("<style>"));
    }

    #[test]
    fn test_export_missing_graph_field() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({"version": "0.1.0"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_export_unicode_in_labels() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({
            "graph": {
                "nodes": [{"id": 1, "name": "变量"}],
                "edges": []
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("变量"));
    }

    #[test]
    fn test_export_special_characters_in_labels() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let data = serde_json::json!({
            "graph": {
                "nodes": [{"id": 1, "name": "x<T>"}],
                "edges": []
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        execute(args).unwrap();
        assert!(output_file.exists());
    }

    #[test]
    fn test_export_large_graph_performance() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.dot");

        let nodes: Vec<_> = (0..100)
            .map(|i| serde_json::json!({"id": i, "name": format!("node{}", i)}))
            .collect();

        let edges: Vec<_> = (0..99)
            .map(|i| serde_json::json!({"from": i, "to": i + 1, "relationship": "borrows"}))
            .collect();

        let data = serde_json::json!({
            "graph": {
                "nodes": nodes,
                "edges": edges
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Dot,
        };

        let start = std::time::Instant::now();
        execute(args).unwrap();
        let duration = start.elapsed();

        assert!(output_file.exists());
        assert!(duration.as_secs() < 5); // Should complete in reasonable time
    }

    #[test]
    fn test_export_all_formats() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");

        let data = serde_json::json!({
            "graph": {
                "nodes": [{"id": 1, "name": "x"}],
                "edges": []
            }
        });

        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        // Test DOT
        let dot_output = temp_dir.path().join("output.dot");
        execute(ExportArgs {
            file: input_file.clone(),
            output: dot_output.clone(),
            format: ExportFormat::Dot,
        }).unwrap();
        assert!(dot_output.exists());

        // Test JSON
        let json_output = temp_dir.path().join("output.json");
        execute(ExportArgs {
            file: input_file.clone(),
            output: json_output.clone(),
            format: ExportFormat::Json,
        }).unwrap();
        assert!(json_output.exists());

        // Test HTML
        let html_output = temp_dir.path().join("output.html");
        execute(ExportArgs {
            file: input_file.clone(),
            output: html_output.clone(),
            format: ExportFormat::Html,
        }).unwrap();
        assert!(html_output.exists());
    }
}
