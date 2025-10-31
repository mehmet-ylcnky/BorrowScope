# Section 79: Run Command Implementation

## Learning Objectives

By the end of this section, you will:
- Implement the `run` subcommand
- Integrate with Cargo build system
- Execute instrumented code
- Capture tracking data
- Handle program arguments

## Prerequisites

- Section 78 (Creating the CLI Crate)
- Understanding of std::process::Command
- Familiarity with Cargo commands

---

## Command Purpose

The `run` command is the primary workflow:
1. Build the Rust project
2. Execute with instrumentation
3. Capture ownership tracking data
4. Optionally open visualization

---

## Implementation

**src/commands/run.rs:**

```rust
use crate::{cli::RunArgs, config::Config, error::CliError, output::Output};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub async fn execute(args: RunArgs) -> Result<()> {
    let output = Output::new(false, false);
    let config = Config::load()?;
    
    // Validate path
    validate_project_path(&args.path)?;
    
    // Build project
    output.info(&format!("Building project at {:?}", args.path));
    let pb = output.progress("Compiling...");
    build_project(&args)?;
    if let Some(pb) = pb {
        pb.finish_with_message("Build complete ✓");
    }
    
    // Run instrumented code
    output.info("Executing instrumented code");
    let pb = output.progress("Running...");
    let execution_result = run_instrumented(&args)?;
    if let Some(pb) = pb {
        pb.finish_with_message("Execution complete ✓");
    }
    
    // Display output if not captured
    if !args.no_capture {
        if !execution_result.stdout.is_empty() {
            println!("\n{}", "Program Output:".bold());
            println!("{}", String::from_utf8_lossy(&execution_result.stdout));
        }
    }
    
    // Save tracking data
    output.success(&format!("Tracking data saved to: {:?}", args.output));
    
    // Open visualization if requested
    if args.visualize {
        output.info("Opening visualization...");
        let viz_args = crate::cli::VisualizeArgs {
            file: args.output.clone(),
            port: config.visualize.port,
            no_browser: !config.visualize.browser,
            host: config.visualize.host.clone(),
        };
        crate::commands::visualize::execute(viz_args).await?;
    }
    
    Ok(())
}

fn validate_project_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(CliError::FileNotFound(path.display().to_string()).into());
    }
    
    // Check if it's a Cargo project
    let cargo_toml = if path.is_file() {
        // Single file - look for Cargo.toml in parent directories
        find_cargo_toml(path.parent().unwrap())
    } else {
        // Directory - check for Cargo.toml
        path.join("Cargo.toml")
    };
    
    if !cargo_toml.exists() {
        anyhow::bail!("Not a Cargo project (no Cargo.toml found)");
    }
    
    Ok(())
}

fn find_cargo_toml(start: &Path) -> std::path::PathBuf {
    let mut current = start;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return cargo_toml;
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    start.join("Cargo.toml")
}

fn build_project(args: &RunArgs) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    
    // Set working directory
    if args.path.is_dir() {
        cmd.current_dir(&args.path);
    } else if let Some(parent) = args.path.parent() {
        cmd.current_dir(parent);
    }
    
    // Release mode
    if args.release {
        cmd.arg("--release");
    }
    
    // Features
    if !args.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(args.features.join(","));
    }
    
    // Execute build
    let output = cmd.output().context("Failed to execute cargo build")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::CompilationError(stderr.to_string()).into());
    }
    
    Ok(())
}

fn run_instrumented(args: &RunArgs) -> Result<std::process::Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    
    // Set working directory
    if args.path.is_dir() {
        cmd.current_dir(&args.path);
    } else if let Some(parent) = args.path.parent() {
        cmd.current_dir(parent);
    }
    
    // Release mode
    if args.release {
        cmd.arg("--release");
    }
    
    // Features
    if !args.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(args.features.join(","));
    }
    
    // Program arguments
    if !args.args.is_empty() {
        cmd.arg("--");
        cmd.args(&args.args);
    }
    
    // Set environment variable for output file
    cmd.env("BORROWSCOPE_OUTPUT", &args.output);
    
    // Capture or inherit stdout/stderr
    if args.no_capture {
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
    }
    
    // Execute
    let output = cmd.output().context("Failed to execute program")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::CompilationError(
            format!("Program exited with error:\n{}", stderr)
        ).into());
    }
    
    Ok(output)
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
    async fn test_run_simple_project() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("test_project");
        
        // Create minimal Cargo project
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(
            project.join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
            "#
        ).unwrap();
        
        fs::write(
            project.join("src/main.rs"),
            r#"
fn main() {
    let x = 42;
    println!("{}", x);
}
            "#
        ).unwrap();
        
        let args = RunArgs {
            path: project.clone(),
            output: project.join("output.json"),
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: true,
        };
        
        let result = execute(args).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_project_path() {
        let temp = TempDir::new().unwrap();
        
        // Should fail - no Cargo.toml
        let result = validate_project_path(temp.path());
        assert!(result.is_err());
        
        // Create Cargo.toml
        fs::write(temp.path().join("Cargo.toml"), "").unwrap();
        
        // Should succeed
        let result = validate_project_path(temp.path());
        assert!(result.is_ok());
    }
}
```

---

## Usage Examples

```bash
# Run current project
borrowscope run

# Run specific file
borrowscope run examples/basic.rs

# Run with arguments
borrowscope run -- --input data.txt --verbose

# Run in release mode
borrowscope run --release

# Run with features
borrowscope run --features "feature1,feature2"

# Run and visualize
borrowscope run --visualize

# Custom output file
borrowscope run --output custom.json

# Don't capture output
borrowscope run --no-capture
```

---

## Key Takeaways

✅ **Cargo integration** - Build and run Rust projects  
✅ **Argument passing** - Forward args to program  
✅ **Environment variables** - Set output file path  
✅ **Error handling** - Compilation and runtime errors  
✅ **Output capture** - Optional stdout/stderr capture  

---

**Previous:** [78-creating-the-cli-crate.md](./78-creating-the-cli-crate.md)  
**Next:** [80-visualize-command-implementation.md](./80-visualize-command-implementation.md)

**Progress:** 4/13 ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜
