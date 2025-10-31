# Section 81: Export Command Implementation

## Learning Objectives

By the end of this section, you will:
- Implement export to multiple formats (DOT, SVG, PNG, JSON, HTML)
- Integrate with Graphviz for image generation
- Create standalone HTML reports
- Handle large graphs efficiently
- Implement custom styling for exports

## Prerequisites

- Section 80 (Visualize Command)
- Understanding of Graphviz DOT format
- Familiarity with image generation
- Knowledge of HTML/CSS

---

## Command Purpose

The `export` command converts tracking data to various formats:
1. **DOT** - Graphviz format for further processing
2. **SVG** - Scalable vector graphics
3. **PNG** - Raster image for presentations
4. **JSON** - Compact format for sharing
5. **HTML** - Standalone interactive report

---

## Complete Implementation

**src/commands/export.rs:**

```rust
use crate::{cli::{ExportArgs, ExportFormat}, error::CliError, output::Output};
use anyhow::{Context, Result};
use borrowscope_graph::OwnershipGraph;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

pub async fn execute(args: ExportArgs) -> Result<()> {
    let output = Output::new(false, false);
    
    // Validate input file
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.display().to_string()).into());
    }
    
    // Load and reconstruct graph
    output.info("Loading tracking data...");
    let graph = load_graph(&args.file)?;
    
    output.success(&format!(
        "Loaded graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    ));
    
    // Export to requested format
    output.info(&format!("Exporting to {:?} format...", args.format));
    
    let pb = output.progress("Generating export...");
    
    let result = match args.format {
        ExportFormat::Dot => export_dot(&graph, &args.output),
        ExportFormat::Svg => export_svg(&graph, &args.output),
        ExportFormat::Png => export_png(&graph, &args.output),
        ExportFormat::Json => export_json(&graph, &args.output),
        ExportFormat::Html => export_html(&graph, &args.output),
    };
    
    if let Some(pb) = pb {
        if result.is_ok() {
            pb.finish_with_message("Export complete ✓");
        } else {
            pb.fail_with_message("Export failed ✗");
        }
    }
    
    result?;
    
    output.success(&format!("Exported to: {:?}", args.output));
    
    // Show file size
    if let Ok(metadata) = std::fs::metadata(&args.output) {
        let size = metadata.len();
        let size_str = format_file_size(size);
        output.info(&format!("File size: {}", size_str));
    }
    
    Ok(())
}

fn load_graph(path: &Path) -> Result<OwnershipGraph> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read tracking data")?;
    
    let export: borrowscope_graph::GraphExport = serde_json::from_str(&content)
        .context("Failed to parse tracking data")?;
    
    // Reconstruct graph from export
    let mut graph = OwnershipGraph::new();
    
    for node in export.nodes {
        graph.add_variable(borrowscope_graph::Variable {
            id: node.id,
            name: node.name,
            type_name: node.type_name,
            created_at: node.created_at,
            dropped_at: node.dropped_at,
            scope_depth: node.scope_depth,
        });
    }
    
    for edge in export.edges {
        match edge.relationship {
            borrowscope_graph::Relationship::BorrowsImmut { at } => {
                graph.add_borrow(edge.from_id, edge.to_id, false, at);
            }
            borrowscope_graph::Relationship::BorrowsMut { at } => {
                graph.add_borrow(edge.from_id, edge.to_id, true, at);
            }
            borrowscope_graph::Relationship::Moves { at } => {
                graph.add_move(edge.from_id, edge.to_id, at);
            }
            _ => {}
        }
    }
    
    Ok(graph)
}

fn export_dot(graph: &OwnershipGraph, output: &Path) -> Result<()> {
    let dot = graph.to_dot();
    std::fs::write(output, dot)
        .context("Failed to write DOT file")?;
    Ok(())
}

fn export_svg(graph: &OwnershipGraph, output: &Path) -> Result<()> {
    // Check if graphviz is installed
    check_graphviz_installed()?;
    
    // Generate DOT
    let dot = graph.to_dot();
    
    // Convert to SVG using dot command
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn dot command")?;
    
    // Write DOT to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot.as_bytes())
            .context("Failed to write to dot stdin")?;
    }
    
    // Read SVG from stdout
    let output_data = child.wait_with_output()
        .context("Failed to wait for dot command")?;
    
    if !output_data.status.success() {
        let stderr = String::from_utf8_lossy(&output_data.stderr);
        anyhow::bail!("dot command failed: {}", stderr);
    }
    
    // Write SVG to file
    std::fs::write(output, &output_data.stdout)
        .context("Failed to write SVG file")?;
    
    Ok(())
}

fn export_png(graph: &OwnershipGraph, output: &Path) -> Result<()> {
    // Check if graphviz is installed
    check_graphviz_installed()?;
    
    // Generate DOT
    let dot = graph.to_dot();
    
    // Convert to PNG using dot command
    let mut child = Command::new("dot")
        .arg("-Tpng")
        .arg("-Gdpi=300")  // High DPI for better quality
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn dot command")?;
    
    // Write DOT to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot.as_bytes())
            .context("Failed to write to dot stdin")?;
    }
    
    // Read PNG from stdout
    let output_data = child.wait_with_output()
        .context("Failed to wait for dot command")?;
    
    if !output_data.status.success() {
        let stderr = String::from_utf8_lossy(&output_data.stderr);
        anyhow::bail!("dot command failed: {}", stderr);
    }
    
    // Write PNG to file
    std::fs::write(output, &output_data.stdout)
        .context("Failed to write PNG file")?;
    
    Ok(())
}

fn export_json(graph: &OwnershipGraph, output: &Path) -> Result<()> {
    let json = graph.to_json_compact()
        .context("Failed to serialize graph")?;
    
    std::fs::write(output, json)
        .context("Failed to write JSON file")?;
    
    Ok(())
}

fn export_html(graph: &OwnershipGraph, output: &Path) -> Result<()> {
    let viz_data = graph.export_for_visualization();
    let json = serde_json::to_string(&viz_data)
        .context("Failed to serialize visualization data")?;
    
    let html = generate_standalone_html(&json);
    
    std::fs::write(output, html)
        .context("Failed to write HTML file")?;
    
    Ok(())
}

fn check_graphviz_installed() -> Result<()> {
    let output = Command::new("dot")
        .arg("-V")
        .output();
    
    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => {
            anyhow::bail!(
                "Graphviz is not installed. Please install it:\n\
                 - Ubuntu/Debian: sudo apt-get install graphviz\n\
                 - macOS: brew install graphviz\n\
                 - Windows: Download from https://graphviz.org/download/"
            )
        }
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn generate_standalone_html(json_data: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BorrowScope Report</title>
    <script src="https://unpkg.com/cytoscape@3.26.0/dist/cytoscape.min.js"></script>
    <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f5f5;
        }}
        
        .header {{
            background: #2c3e50;
            color: white;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        .header h1 {{
            font-size: 28px;
            margin-bottom: 5px;
        }}
        
        .header p {{
            opacity: 0.8;
            font-size: 14px;
        }}
        
        .container {{
            display: flex;
            height: calc(100vh - 80px);
        }}
        
        .sidebar {{
            width: 350px;
            background: white;
            border-right: 1px solid #ddd;
            overflow-y: auto;
            padding: 20px;
        }}
        
        #cy {{
            flex: 1;
            background: white;
        }}
        
        .section {{
            margin-bottom: 30px;
        }}
        
        .section h2 {{
            font-size: 18px;
            color: #2c3e50;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 2px solid #3498db;
        }}
        
        .stat-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 15px;
        }}
        
        .stat-card {{
            background: #ecf0f1;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }}
        
        .stat-value {{
            font-size: 32px;
            font-weight: bold;
            color: #3498db;
            margin-bottom: 5px;
        }}
        
        .stat-label {{
            font-size: 12px;
            color: #7f8c8d;
            text-transform: uppercase;
        }}
        
        .legend-item {{
            display: flex;
            align-items: center;
            margin-bottom: 12px;
            padding: 8px;
            background: #f8f9fa;
            border-radius: 4px;
        }}
        
        .legend-color {{
            width: 24px;
            height: 24px;
            border-radius: 4px;
            margin-right: 12px;
            border: 2px solid #ddd;
        }}
        
        .legend-text {{
            flex: 1;
        }}
        
        .legend-title {{
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 2px;
        }}
        
        .legend-desc {{
            font-size: 12px;
            color: #7f8c8d;
        }}
        
        .controls {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 10px;
        }}
        
        button {{
            padding: 10px 15px;
            background: #3498db;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: background 0.2s;
        }}
        
        button:hover {{
            background: #2980b9;
        }}
        
        button:active {{
            transform: scale(0.98);
        }}
        
        .info-box {{
            background: #e8f4f8;
            border-left: 4px solid #3498db;
            padding: 15px;
            margin-top: 20px;
            border-radius: 4px;
        }}
        
        .info-box h3 {{
            font-size: 14px;
            color: #2c3e50;
            margin-bottom: 8px;
        }}
        
        .info-box p {{
            font-size: 13px;
            color: #7f8c8d;
            line-height: 1.6;
        }}
        
        @media print {{
            .sidebar {{
                display: none;
            }}
            
            #cy {{
                width: 100%;
            }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🦀 BorrowScope Report</h1>
        <p>Rust Ownership and Borrowing Visualization</p>
    </div>
    
    <div class="container">
        <div class="sidebar">
            <div class="section">
                <h2>📊 Statistics</h2>
                <div class="stat-grid">
                    <div class="stat-card">
                        <div class="stat-value" id="stat-nodes">0</div>
                        <div class="stat-label">Variables</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="stat-edges">0</div>
                        <div class="stat-label">Relationships</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="stat-alive">0</div>
                        <div class="stat-label">Alive</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value" id="stat-dropped">0</div>
                        <div class="stat-label">Dropped</div>
                    </div>
                </div>
            </div>
            
            <div class="section">
                <h2>🎨 Legend</h2>
                <div class="legend-item">
                    <div class="legend-color" style="background: #3498db;"></div>
                    <div class="legend-text">
                        <div class="legend-title">Owned Variable</div>
                        <div class="legend-desc">Variable with ownership</div>
                    </div>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #2ecc71;"></div>
                    <div class="legend-text">
                        <div class="legend-title">Immutable Borrow</div>
                        <div class="legend-desc">Shared reference (&T)</div>
                    </div>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #e74c3c;"></div>
                    <div class="legend-text">
                        <div class="legend-title">Mutable Borrow</div>
                        <div class="legend-desc">Exclusive reference (&mut T)</div>
                    </div>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #95a5a6;"></div>
                    <div class="legend-text">
                        <div class="legend-title">Dropped</div>
                        <div class="legend-desc">Variable out of scope</div>
                    </div>
                </div>
            </div>
            
            <div class="section">
                <h2>🎮 Controls</h2>
                <div class="controls">
                    <button onclick="resetZoom()">Reset Zoom</button>
                    <button onclick="fitGraph()">Fit Screen</button>
                    <button onclick="exportPNG()">Export PNG</button>
                    <button onclick="window.print()">Print</button>
                </div>
            </div>
            
            <div class="info-box">
                <h3>💡 Tips</h3>
                <p>
                    • Click and drag to pan<br>
                    • Scroll to zoom<br>
                    • Click nodes for details<br>
                    • Use controls to navigate
                </p>
            </div>
        </div>
        
        <div id="cy"></div>
    </div>
    
    <script>
        const data = {};
        let cy;
        
        function initializeGraph() {{
            const elements = {{
                nodes: data.elements.nodes,
                edges: data.elements.edges
            }};
            
            cy = cytoscape({{
                container: document.getElementById('cy'),
                elements: elements,
                style: data.style,
                layout: data.layout
            }});
            
            // Update statistics
            updateStats();
            
            // Add interactivity
            cy.on('tap', 'node', function(evt) {{
                const node = evt.target;
                const data = node.data();
                alert(`Variable: ${{data.label}}\nType: ${{data.type}}`);
            }});
        }}
        
        function updateStats() {{
            const nodes = data.elements.nodes;
            const edges = data.elements.edges;
            
            document.getElementById('stat-nodes').textContent = nodes.length;
            document.getElementById('stat-edges').textContent = edges.length;
            
            const alive = nodes.filter(n => n.data.is_alive).length;
            document.getElementById('stat-alive').textContent = alive;
            document.getElementById('stat-dropped').textContent = nodes.length - alive;
        }}
        
        function resetZoom() {{
            if (cy) cy.zoom(1);
        }}
        
        function fitGraph() {{
            if (cy) cy.fit();
        }}
        
        function exportPNG() {{
            if (cy) {{
                const png = cy.png({{ full: true, scale: 2 }});
                const link = document.createElement('a');
                link.download = 'borrowscope-graph.png';
                link.href = png;
                link.click();
            }}
        }}
        
        // Initialize on load
        window.addEventListener('load', initializeGraph);
    </script>
</body>
</html>"#, json_data)
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_graph() -> OwnershipGraph {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(borrowscope_graph::Variable {
            id: 1,
            name: "x".into(),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: None,
            scope_depth: 0,
        });
        
        graph.add_variable(borrowscope_graph::Variable {
            id: 2,
            name: "r".into(),
            type_name: "&i32".into(),
            created_at: 1050,
            dropped_at: Some(1200),
            scope_depth: 0,
        });
        
        graph.add_borrow(2, 1, false, 1050);
        
        graph
    }

    #[tokio::test]
    async fn test_export_dot() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("graph.dot");
        
        let graph = create_test_graph();
        let result = export_dot(&graph, &output);
        
        assert!(result.is_ok());
        assert!(output.exists());
        
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("digraph"));
    }

    #[tokio::test]
    async fn test_export_json() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("graph.json");
        
        let graph = create_test_graph();
        let result = export_json(&graph, &output);
        
        assert!(result.is_ok());
        assert!(output.exists());
        
        let content = fs::read_to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn test_export_html() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("report.html");
        
        let graph = create_test_graph();
        let result = export_html(&graph, &output);
        
        assert!(result.is_ok());
        assert!(output.exists());
        
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("BorrowScope"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 bytes");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
```

---

## Usage Examples

```bash
# Export to DOT
borrowscope export data.json -o graph.dot -f dot

# Export to SVG
borrowscope export data.json -o graph.svg -f svg

# Export to PNG (high quality)
borrowscope export data.json -o graph.png -f png

# Export to compact JSON
borrowscope export data.json -o compact.json -f json

# Export to standalone HTML
borrowscope export data.json -o report.html -f html

# Then open report.html in any browser
```

---

## Key Takeaways

✅ **Multiple formats** - DOT, SVG, PNG, JSON, HTML  
✅ **Graphviz integration** - Image generation  
✅ **Standalone HTML** - Self-contained reports  
✅ **Error handling** - Missing dependencies, invalid data  
✅ **File size reporting** - User feedback  

---

**Previous:** [80-visualize-command.md](./80-visualize-command.md)  
**Next:** [82-init-check-commands.md](./82-init-check-commands.md)

**Progress:** 6/13 ⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜
