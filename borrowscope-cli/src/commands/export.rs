//! Export command implementation

use std::fs;

use crate::cli::{ExportArgs, ExportFormat};
use crate::error::{CliError, Result};

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
    let mut dot = String::from("digraph BorrowScope {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");

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

    fs::write(output, dot)?;
    Ok(())
}

fn export_svg(_data: &serde_json::Value, output: &std::path::Path) -> Result<()> {
    // TODO: Implement SVG export using graphviz or similar
    let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">
  <text x="400" y="300" text-anchor="middle">SVG export not yet implemented</text>
</svg>"#;

    fs::write(output, svg)?;
    Ok(())
}

fn export_png(_data: &serde_json::Value, _output: &std::path::Path) -> Result<()> {
    // TODO: Implement PNG export
    Err(CliError::Other(
        "PNG export not yet implemented".to_string(),
    ))
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
    fn test_export_svg_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.svg");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Svg,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(output_file.exists());

        let contents = fs::read_to_string(&output_file).unwrap();
        assert!(contents.contains("<?xml"));
        assert!(contents.contains("<svg"));
    }

    #[test]
    fn test_export_png_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.png");

        let data = serde_json::json!({"test": "data"});
        fs::write(&input_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = ExportArgs {
            file: input_file,
            output: output_file.clone(),
            format: ExportFormat::Png,
        };

        // PNG export is not implemented yet, should return error
        let result = execute(args);
        assert!(result.is_err());
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
}
