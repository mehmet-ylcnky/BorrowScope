# Section 82: Init and Check Commands Implementation

## Learning Objectives

By the end of this section, you will:
- Implement the `init` command for project setup
- Create configuration templates
- Implement the `check` command for validation
- Detect borrow conflicts
- Generate statistical reports
- Validate graph integrity

## Prerequisites

- Section 81 (Export Command)
- Understanding of TOML configuration
- Familiarity with graph validation algorithms

---

## Init Command Purpose

The `init` command initializes BorrowScope in a project:
1. Creates `.borrowscope.toml` configuration file
2. Offers multiple templates (default, minimal, advanced)
3. Validates project structure
4. Sets up ignore patterns
5. Provides interactive configuration

---

## Check Command Purpose

The `check` command validates tracking data:
1. Validates graph integrity (no cycles, valid references)
2. Detects borrow conflicts
3. Computes statistics
4. Generates detailed reports
5. Suggests fixes for issues

---

## Init Command Implementation

**src/commands/init.rs:**

```rust
use crate::{cli::{InitArgs, ConfigTemplate}, config::Config, output::Output, error::CliError};
use anyhow::{Context, Result};
use std::path::Path;
use std::io::{self, Write};

pub async fn execute(args: InitArgs) -> Result<()> {
    let output = Output::new(false, false);
    
    // Validate path exists
    if !args.path.exists() {
        return Err(CliError::FileNotFound(args.path.display().to_string()).into());
    }
    
    let config_path = args.path.join(".borrowscope.toml");
    
    // Check if config already exists
    if config_path.exists() && !args.force {
        output.warning("Configuration file already exists");
        output.info("Use --force to overwrite");
        
        // Ask user if they want to continue
        if !args.force && !confirm_overwrite()? {
            output.info("Initialization cancelled");
            return Ok(());
        }
    }
    
    // Detect project type
    let project_info = detect_project_type(&args.path)?;
    output.info(&format!("Detected project: {}", project_info.name));
    
    // Create configuration based on template
    output.info(&format!("Creating configuration from {:?} template", args.template));
    
    let config = match args.template {
        ConfigTemplate::Default => create_default_config(&project_info),
        ConfigTemplate::Minimal => create_minimal_config(&project_info),
        ConfigTemplate::Advanced => create_advanced_config(&project_info),
    };
    
    // Save configuration
    config.save(&config_path)
        .context("Failed to save configuration")?;
    
    output.success(&format!("Created configuration: {:?}", config_path));
    
    // Create .gitignore entry if needed
    add_to_gitignore(&args.path)?;
    
    // Print next steps
    print_next_steps(&output, &args.template);
    
    Ok(())
}

struct ProjectInfo {
    name: String,
    is_workspace: bool,
    has_tests: bool,
    has_examples: bool,
}

fn detect_project_type(path: &Path) -> Result<ProjectInfo> {
    let cargo_toml = path.join("Cargo.toml");
    
    if !cargo_toml.exists() {
        anyhow::bail!("Not a Cargo project (no Cargo.toml found)");
    }
    
    let content = std::fs::read_to_string(&cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;
    
    let name = toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let is_workspace = toml.get("workspace").is_some();
    let has_tests = path.join("tests").exists();
    let has_examples = path.join("examples").exists();
    
    Ok(ProjectInfo {
        name,
        is_workspace,
        has_tests,
        has_examples,
    })
}

fn create_default_config(info: &ProjectInfo) -> Config {
    Config {
        run: crate::config::RunConfig {
            output: "borrowscope.json".into(),
            visualize: false,
            capture: true,
        },
        visualize: crate::config::VisualizeConfig {
            port: 3000,
            browser: true,
            host: "127.0.0.1".into(),
        },
        export: crate::config::ExportConfig {
            format: "dot".into(),
        },
        tracking: crate::config::TrackingConfig {
            smart_pointers: true,
            async_code: true,
            unsafe_code: false,
        },
        ignore: crate::config::IgnoreConfig {
            patterns: vec![
                "*.test.rs".into(),
                "*_test.rs".into(),
                "target/**".into(),
            ],
            directories: vec![
                "target".into(),
                if info.has_tests { "tests".into() } else { String::new() },
            ].into_iter().filter(|s| !s.is_empty()).collect(),
        },
    }
}

fn create_minimal_config(info: &ProjectInfo) -> Config {
    Config {
        run: crate::config::RunConfig {
            output: "borrowscope.json".into(),
            visualize: false,
            capture: true,
        },
        visualize: crate::config::VisualizeConfig {
            port: 3000,
            browser: true,
            host: "127.0.0.1".into(),
        },
        export: crate::config::ExportConfig {
            format: "dot".into(),
        },
        tracking: crate::config::TrackingConfig {
            smart_pointers: false,
            async_code: false,
            unsafe_code: false,
        },
        ignore: crate::config::IgnoreConfig {
            patterns: vec!["target/**".into()],
            directories: vec!["target".into()],
        },
    }
}

fn create_advanced_config(info: &ProjectInfo) -> Config {
    Config {
        run: crate::config::RunConfig {
            output: "borrowscope.json".into(),
            visualize: true,  // Auto-visualize
            capture: true,
        },
        visualize: crate::config::VisualizeConfig {
            port: 3000,
            browser: true,
            host: "127.0.0.1".into(),
        },
        export: crate::config::ExportConfig {
            format: "html".into(),  // HTML reports
        },
        tracking: crate::config::TrackingConfig {
            smart_pointers: true,
            async_code: true,
            unsafe_code: true,  // Track unsafe code
        },
        ignore: crate::config::IgnoreConfig {
            patterns: vec![
                "*.test.rs".into(),
                "*_test.rs".into(),
                "target/**".into(),
                "benches/**".into(),
            ],
            directories: vec![
                "target".into(),
                "tests".into(),
                "benches".into(),
            ],
        },
    }
}

fn confirm_overwrite() -> Result<bool> {
    print!("Overwrite existing configuration? [y/N]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y")
}

fn add_to_gitignore(path: &Path) -> Result<()> {
    let gitignore = path.join(".gitignore");
    
    if !gitignore.exists() {
        return Ok(());
    }
    
    let content = std::fs::read_to_string(&gitignore)?;
    
    if content.contains("borrowscope.json") {
        return Ok(());
    }
    
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&gitignore)?;
    
    writeln!(file, "\n# BorrowScope")?;
    writeln!(file, "borrowscope.json")?;
    
    Ok(())
}

fn print_next_steps(output: &Output, template: &ConfigTemplate) {
    output.info("\n📝 Next steps:");
    output.info("  1. Edit .borrowscope.toml to customize settings");
    output.info("  2. Run: borrowscope run");
    output.info("  3. Visualize: borrowscope visualize borrowscope.json");
    
    match template {
        ConfigTemplate::Minimal => {
            output.info("\n💡 Tip: You're using minimal tracking.");
            output.info("   Enable more features in .borrowscope.toml");
        }
        ConfigTemplate::Advanced => {
            output.info("\n💡 Tip: Advanced tracking is enabled.");
            output.info("   This includes unsafe code tracking.");
        }
        _ => {}
    }
}
```

---

## Check Command Implementation

**src/commands/check.rs:**

```rust
use crate::{cli::CheckArgs, output::Output, error::CliError};
use anyhow::{Context, Result};
use borrowscope_graph::{OwnershipGraph, BorrowConflict};
use colored::Colorize;

pub async fn execute(args: CheckArgs) -> Result<()> {
    let output = Output::new(false, false);
    
    // Validate file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.display().to_string()).into());
    }
    
    // Load graph
    output.info("Loading tracking data...");
    let graph = load_graph(&args.file)?;
    
    output.success(&format!(
        "Loaded graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    ));
    
    let mut has_issues = false;
    
    // Run validation checks
    if args.validate {
        output.info("\n🔍 Validating graph integrity...");
        if let Err(e) = validate_graph(&graph, &output) {
            output.error(&format!("Validation failed: {}", e));
            has_issues = true;
        }
    }
    
    // Check for conflicts
    if args.conflicts {
        output.info("\n⚠️  Checking for borrow conflicts...");
        let conflicts = check_conflicts(&graph, &output)?;
        if !conflicts.is_empty() {
            has_issues = true;
        }
    }
    
    // Show statistics
    if args.stats {
        output.info("\n📊 Computing statistics...");
        show_statistics(&graph, &output)?;
    }
    
    // Summary
    println!("\n{}", "═".repeat(60));
    if has_issues {
        output.warning("Check complete with issues found");
        std::process::exit(1);
    } else {
        output.success("Check complete - no issues found! ✓");
    }
    
    Ok(())
}

fn load_graph(path: &std::path::Path) -> Result<OwnershipGraph> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read tracking data")?;
    
    let export: borrowscope_graph::GraphExport = serde_json::from_str(&content)
        .context("Failed to parse tracking data")?;
    
    // Reconstruct graph
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

fn validate_graph(graph: &OwnershipGraph, output: &Output) -> Result<()> {
    let mut issues = Vec::new();
    
    // Check for cycles
    if graph.has_cycle() {
        issues.push("Graph contains cycles (invalid ownership pattern)");
    } else {
        output.success("  ✓ Graph is acyclic");
    }
    
    // Check for orphaned nodes
    let orphaned = graph.query().orphaned();
    if !orphaned.is_empty() {
        issues.push(&format!("Found {} orphaned nodes", orphaned.len()));
        for node in orphaned.iter().take(5) {
            output.warning(&format!("    • {}", node.name));
        }
        if orphaned.len() > 5 {
            output.warning(&format!("    ... and {} more", orphaned.len() - 5));
        }
    } else {
        output.success("  ✓ No orphaned nodes");
    }
    
    // Check node count
    if graph.node_count() == 0 {
        issues.push("Graph is empty");
    } else {
        output.success(&format!("  ✓ {} nodes validated", graph.node_count()));
    }
    
    // Check edge count
    if graph.edge_count() == 0 {
        output.warning("  ⚠ No edges (no borrows tracked)");
    } else {
        output.success(&format!("  ✓ {} edges validated", graph.edge_count()));
    }
    
    // Check for invalid timestamps
    let mut invalid_timestamps = 0;
    for var in graph.all_variables() {
        if let Some(dropped) = var.dropped_at {
            if dropped < var.created_at {
                invalid_timestamps += 1;
            }
        }
    }
    
    if invalid_timestamps > 0 {
        issues.push(&format!("Found {} variables with invalid timestamps", invalid_timestamps));
    } else {
        output.success("  ✓ All timestamps valid");
    }
    
    if !issues.is_empty() {
        anyhow::bail!("Validation failed:\n  {}", issues.join("\n  "));
    }
    
    Ok(())
}

fn check_conflicts(graph: &OwnershipGraph, output: &Output) -> Result<Vec<BorrowConflict>> {
    let conflicts = graph.find_conflicts_optimized();
    
    if conflicts.is_empty() {
        output.success("  ✓ No borrow conflicts detected");
        return Ok(vec![]);
    }
    
    output.warning(&format!("  ⚠ Found {} conflict(s):", conflicts.len()));
    
    for (i, conflict) in conflicts.iter().enumerate() {
        println!("\n  {}. {}", i + 1, format_conflict(conflict, graph));
        
        // Show affected variables
        output.info(&format!("     Owner: {}", 
            graph.get_variable(conflict.owner_id)
                .map(|v| v.name.as_str())
                .unwrap_or("<unknown>")));
        
        output.info("     Borrowers:");
        for &borrower_id in &conflict.borrowers {
            if let Some(var) = graph.get_variable(borrower_id) {
                output.info(&format!("       • {} ({})", var.name, var.type_name));
            }
        }
        
        output.info(&format!("     Time range: {} - {} μs", 
            conflict.time_range.0, conflict.time_range.1));
        
        // Suggest fix
        if let Some(suggestion) = suggest_fix(conflict, graph) {
            output.info(&format!("     💡 Suggestion: {}", suggestion.green()));
        }
    }
    
    Ok(conflicts)
}

fn format_conflict(conflict: &BorrowConflict, graph: &OwnershipGraph) -> String {
    use borrowscope_graph::ConflictType;
    
    let owner = graph.get_variable(conflict.owner_id)
        .map(|v| v.name.as_str())
        .unwrap_or("<unknown>");
    
    match conflict.conflict_type {
        ConflictType::MultipleMutableBorrows => {
            format!("{} Multiple mutable borrows of '{}'", "✗".red(), owner)
        }
        ConflictType::MutableWithImmutable => {
            format!("{} Mutable and immutable borrows of '{}'", "✗".red(), owner)
        }
    }
}

fn suggest_fix(conflict: &BorrowConflict, graph: &OwnershipGraph) -> Option<String> {
    use borrowscope_graph::ConflictType;
    
    match conflict.conflict_type {
        ConflictType::MultipleMutableBorrows => {
            Some("Ensure only one mutable borrow exists at a time. \
                  Consider using interior mutability (RefCell) if needed.".into())
        }
        ConflictType::MutableWithImmutable => {
            Some("Mutable and immutable borrows cannot coexist. \
                  Drop the immutable borrow before creating a mutable one.".into())
        }
    }
}

fn show_statistics(graph: &OwnershipGraph, output: &Output) -> Result<()> {
    let stats = graph.statistics();
    
    println!("\n  {}", "General Statistics".bold());
    println!("  ├─ Total variables:     {}", stats.total_variables);
    println!("  ├─ Alive variables:     {} ({:.1}%)", 
        stats.alive_variables,
        (stats.alive_variables as f64 / stats.total_variables as f64) * 100.0
    );
    println!("  └─ Dropped variables:   {} ({:.1}%)", 
        stats.total_variables - stats.alive_variables,
        ((stats.total_variables - stats.alive_variables) as f64 / stats.total_variables as f64) * 100.0
    );
    
    println!("\n  {}", "Borrow Statistics".bold());
    println!("  ├─ Total borrows:       {}", stats.total_borrows);
    println!("  ├─ Immutable borrows:   {} ({:.1}%)", 
        stats.immutable_borrows,
        (stats.immutable_borrows as f64 / stats.total_borrows.max(1) as f64) * 100.0
    );
    println!("  ├─ Mutable borrows:     {} ({:.1}%)", 
        stats.mutable_borrows,
        (stats.mutable_borrows as f64 / stats.total_borrows.max(1) as f64) * 100.0
    );
    println!("  └─ Moves:               {}", stats.moves);
    
    println!("\n  {}", "Lifetime Statistics".bold());
    println!("  ├─ Average lifetime:    {:.2} μs", stats.avg_lifetime);
    println!("  └─ Max borrow depth:    {}", stats.max_borrow_depth);
    
    // Find interesting patterns
    println!("\n  {}", "Interesting Patterns".bold());
    
    let most_borrowed = graph.query().most_borrowed();
    if let Some((var, count)) = most_borrowed {
        println!("  ├─ Most borrowed:       {} ({} borrows)", var.name, count);
    }
    
    let longest_lived = graph.query().longest_lived();
    if let Some(var) = longest_lived {
        let lifetime = var.dropped_at.unwrap() - var.created_at;
        println!("  └─ Longest lived:       {} ({} μs)", var.name, lifetime);
    }
    
    Ok(())
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

    #[tokio::test]
    async fn test_init_default() {
        let temp = TempDir::new().unwrap();
        
        // Create minimal Cargo.toml
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
            "#
        ).unwrap();
        
        let args = InitArgs {
            path: temp.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };
        
        let result = execute(args).await;
        assert!(result.is_ok());
        
        let config_path = temp.path().join(".borrowscope.toml");
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_check_valid_graph() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("data.json");
        
        let data = r#"{
            "nodes": [
                {"id": 1, "name": "x", "type_name": "i32", "created_at": 1000, "dropped_at": null, "scope_depth": 0}
            ],
            "edges": []
        }"#;
        
        fs::write(&file, data).unwrap();
        
        let args = CheckArgs {
            file,
            conflicts: false,
            stats: true,
            validate: true,
        };
        
        let result = execute(args).await;
        assert!(result.is_ok());
    }
}
```

---

## Usage Examples

```bash
# Initialize with default template
borrowscope init

# Initialize with minimal template
borrowscope init --template minimal

# Initialize with advanced template
borrowscope init --template advanced

# Force overwrite existing config
borrowscope init --force

# Check for conflicts
borrowscope check data.json --conflicts

# Show statistics
borrowscope check data.json --stats

# Full validation
borrowscope check data.json --validate --conflicts --stats

# Check specific file
borrowscope check custom.json --validate
```

---

## Key Takeaways

✅ **Init command** - Project setup with templates  
✅ **Check command** - Comprehensive validation  
✅ **Conflict detection** - Find borrow violations  
✅ **Statistics** - Detailed graph metrics  
✅ **Suggestions** - Helpful fix recommendations  

---

**Previous:** [81-export-command.md](./81-export-command.md)  
**Next:** [83-file-instrumentation.md](./83-file-instrumentation.md)

**Progress:** 7/13 ⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜
