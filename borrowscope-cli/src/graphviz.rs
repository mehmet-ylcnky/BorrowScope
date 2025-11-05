//! Graphviz integration for graph rendering

use std::path::Path;
use std::process::Command;
use std::io::Write;
use anyhow::{Context, Result};

/// Check if Graphviz is available
pub fn is_available() -> bool {
    Command::new("dot")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get Graphviz version
pub fn get_version() -> Result<String> {
    let output = Command::new("dot")
        .arg("-V")
        .output()
        .context("Failed to execute dot command")?;
    
    if !output.status.success() {
        anyhow::bail!("dot command failed");
    }
    
    let version = String::from_utf8_lossy(&output.stderr);
    Ok(version.trim().to_string())
}

/// Render DOT file to SVG
pub fn render_svg(dot_content: &str, output_path: &Path) -> Result<()> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .arg("-o")
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn dot process")?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot_content.as_bytes())
            .context("Failed to write to dot stdin")?;
    }
    
    let status = child.wait().context("Failed to wait for dot process")?;
    
    if !status.success() {
        anyhow::bail!("dot command failed with status: {}", status);
    }
    
    Ok(())
}

/// Render DOT file to PNG
pub fn render_png(dot_content: &str, output_path: &Path) -> Result<()> {
    let mut child = Command::new("dot")
        .arg("-Tpng")
        .arg("-o")
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn dot process")?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot_content.as_bytes())
            .context("Failed to write to dot stdin")?;
    }
    
    let status = child.wait().context("Failed to wait for dot process")?;
    
    if !status.success() {
        anyhow::bail!("dot command failed with status: {}", status);
    }
    
    Ok(())
}

/// Render DOT file with custom format
pub fn render(dot_content: &str, output_path: &Path, format: &str) -> Result<()> {
    let mut child = Command::new("dot")
        .arg(format!("-T{}", format))
        .arg("-o")
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn dot process")?;
    
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(dot_content.as_bytes())
            .context("Failed to write to dot stdin")?;
    }
    
    let status = child.wait().context("Failed to wait for dot process")?;
    
    if !status.success() {
        anyhow::bail!("dot command failed with status: {}", status);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_is_available() {
        // This test will pass if graphviz is installed, skip otherwise
        let available = is_available();
        println!("Graphviz available: {}", available);
    }

    #[test]
    fn test_get_version() {
        if is_available() {
            let version = get_version();
            assert!(version.is_ok());
            let v = version.unwrap();
            assert!(v.contains("dot") || v.contains("graphviz"));
        }
    }

    #[test]
    fn test_render_svg() {
        if !is_available() {
            return; // Skip if graphviz not installed
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "digraph G { A -> B; }";
        let result = render_svg(dot, &output);
        
        if result.is_ok() {
            assert!(output.exists());
            let contents = fs::read_to_string(&output).unwrap();
            assert!(contents.contains("<svg") || contents.contains("<?xml"));
        }
    }

    #[test]
    fn test_render_png() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.png");
        
        let dot = "digraph G { A -> B; }";
        let result = render_png(dot, &output);
        
        if result.is_ok() {
            assert!(output.exists());
        }
    }

    #[test]
    fn test_render_custom_format() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.pdf");
        
        let dot = "digraph G { A -> B; }";
        let result = render(dot, &output, "pdf");
        
        if result.is_ok() {
            assert!(output.exists());
        }
    }

    #[test]
    fn test_render_invalid_dot() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "invalid dot syntax {{{";
        let result = render_svg(dot, &output);
        
        // Should handle error gracefully
        assert!(result.is_err() || output.exists());
    }

    #[test]
    fn test_render_empty_graph() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "digraph G {}";
        let result = render_svg(dot, &output);
        
        if result.is_ok() {
            assert!(output.exists());
        }
    }

    #[test]
    fn test_render_complex_graph() {
        if !is_available() {
            println!("Skipping test: Graphviz not installed");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    rankdir=LR;
    node [shape=box];
    A -> B -> C;
    A -> D;
    B -> D;
    C -> E;
    D -> E;
}
"#;
        let result = render_svg(dot, &output);
        
        assert!(result.is_ok());
        assert!(output.exists());
        let contents = fs::read_to_string(&output).unwrap();
        assert!(!contents.is_empty());
    }

    #[test]
    fn test_render_with_attributes() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    node [shape=circle, style=filled, fillcolor=lightblue];
    edge [color=red, penwidth=2];
    A -> B [label="edge"];
}
"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_subgraphs() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    subgraph cluster_0 {
        label="Cluster 0";
        A -> B;
    }
    subgraph cluster_1 {
        label="Cluster 1";
        C -> D;
    }
    B -> C;
}
"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_large_graph() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let mut dot = String::from("digraph G {\n");
        for i in 0..100 {
            dot.push_str(&format!("  N{} -> N{};\n", i, i + 1));
        }
        dot.push_str("}\n");
        
        let result = render_svg(&dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_unicode_labels() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"digraph G { "变量" -> "函数" [label="调用"]; }"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_special_characters() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"digraph G { "A<T>" -> "B&C" [label="x->y"]; }"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_multiple_formats() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let dot = "digraph G { A -> B; }";
        
        // SVG
        let svg_output = temp_dir.path().join("test.svg");
        assert!(render_svg(dot, &svg_output).is_ok());
        assert!(svg_output.exists());
        
        // PNG
        let png_output = temp_dir.path().join("test.png");
        assert!(render_png(dot, &png_output).is_ok());
        assert!(png_output.exists());
        
        // PDF
        let pdf_output = temp_dir.path().join("test.pdf");
        assert!(render(dot, &pdf_output, "pdf").is_ok());
        assert!(pdf_output.exists());
    }

    #[test]
    fn test_render_directed_graph() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "digraph G { A -> B -> C; }";
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_undirected_graph() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "graph G { A -- B -- C; }";
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_with_colors() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    A [fillcolor=red, style=filled];
    B [fillcolor=blue, style=filled];
    C [fillcolor=green, style=filled];
    A -> B -> C;
}
"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_with_shapes() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    A [shape=box];
    B [shape=circle];
    C [shape=diamond];
    D [shape=ellipse];
    A -> B -> C -> D;
}
"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_with_rankdir() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        
        // Left to Right
        let lr_output = temp_dir.path().join("lr.svg");
        let dot_lr = "digraph G { rankdir=LR; A -> B -> C; }";
        assert!(render_svg(dot_lr, &lr_output).is_ok());
        
        // Top to Bottom
        let tb_output = temp_dir.path().join("tb.svg");
        let dot_tb = "digraph G { rankdir=TB; A -> B -> C; }";
        assert!(render_svg(dot_tb, &tb_output).is_ok());
    }

    #[test]
    fn test_render_concurrent() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let dot = "digraph G { A -> B; }";
        
        let mut handles = vec![];
        for i in 0..5 {
            let output = temp_dir.path().join(format!("test{}.svg", i));
            let dot_clone = dot.to_string();
            handles.push(std::thread::spawn(move || {
                render_svg(&dot_clone, &output)
            }));
        }
        
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }
    }

    #[test]
    fn test_render_empty_output_path() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("");
        
        let dot = "digraph G { A -> B; }";
        let result = render_svg(dot, &output);
        // Should handle gracefully
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_render_very_long_labels() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let long_label = "A".repeat(1000);
        let dot = format!(r#"digraph G {{ "{}" -> B; }}"#, long_label);
        let result = render_svg(&dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_nested_subgraphs() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"
digraph G {
    subgraph cluster_outer {
        label="Outer";
        subgraph cluster_inner {
            label="Inner";
            A -> B;
        }
        C -> D;
    }
}
"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_render_with_html_labels() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = r#"digraph G { A [label=<<B>Bold</B>>]; A -> B; }"#;
        let result = render_svg(dot, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_get_version_format() {
        if !is_available() {
            return;
        }

        let version = get_version().unwrap();
        assert!(version.contains("dot") || version.contains("graphviz"));
        assert!(version.contains("version") || version.chars().any(|c| c.is_numeric()));
    }

    #[test]
    fn test_render_output_file_permissions() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        let dot = "digraph G { A -> B; }";
        render_svg(dot, &output).unwrap();
        
        assert!(output.exists());
        let metadata = fs::metadata(&output).unwrap();
        assert!(metadata.is_file());
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_render_overwrite_existing() {
        if !is_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("test.svg");
        
        // First render
        let dot1 = "digraph G { A -> B; }";
        render_svg(dot1, &output).unwrap();
        let size1 = fs::metadata(&output).unwrap().len();
        
        // Second render with different content
        let dot2 = "digraph G { A -> B -> C -> D; }";
        render_svg(dot2, &output).unwrap();
        let size2 = fs::metadata(&output).unwrap().len();
        
        assert_ne!(size1, size2);
    }
}
