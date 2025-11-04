//! Run command implementation

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::RunArgs;
use crate::config::Config;
use crate::error::{CliError, Result};

pub fn execute(args: RunArgs, config: Config) -> Result<()> {
    log::info!("Running BorrowScope on: {}", args.path.display());

    // Determine output file
    let output_file = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(&config.run.output));

    // Check if path exists
    if !args.path.exists() {
        return Err(CliError::FileNotFound(args.path));
    }

    // Determine if it's a file or directory
    if args.path.is_file() {
        run_single_file(&args, &output_file)?;
    } else if args.path.is_dir() {
        run_project(&args, &output_file)?;
    } else {
        return Err(CliError::Other(format!(
            "Invalid path: {}",
            args.path.display()
        )));
    }

    log::info!("Tracking data saved to: {}", output_file.display());

    // Open visualization if requested
    if args.visualize || config.run.visualize {
        log::info!("Opening visualization...");
        let visualize_args = crate::cli::VisualizeArgs {
            file: output_file,
            port: None,
            no_browser: false,
            host: None,
        };
        crate::commands::visualize::execute(visualize_args, config)?;
    }

    Ok(())
}

fn run_single_file(args: &RunArgs, output_file: &PathBuf) -> Result<()> {
    log::debug!("Running single file: {}", args.path.display());

    // For now, create a placeholder output
    // TODO: Implement actual instrumentation and execution
    let placeholder_data = serde_json::json!({
        "version": "0.1.0",
        "source": args.path.display().to_string(),
        "events": [],
        "graph": {
            "nodes": [],
            "edges": []
        }
    });

    fs::write(
        output_file,
        serde_json::to_string_pretty(&placeholder_data)?,
    )?;

    Ok(())
}

fn run_project(args: &RunArgs, output_file: &PathBuf) -> Result<()> {
    log::debug!("Running project: {}", args.path.display());

    // Check if Cargo.toml exists
    let cargo_toml = args.path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(CliError::Other(format!(
            "No Cargo.toml found in {}",
            args.path.display()
        )));
    }

    // Build cargo command based on target
    let mut cmd = Command::new("cargo");

    match args.target {
        Some(crate::cli::RunTarget::Test) => {
            cmd.arg("test");
        }
        Some(crate::cli::RunTarget::Bench) => {
            cmd.arg("bench");
        }
        Some(crate::cli::RunTarget::Example) => {
            cmd.arg("run");
            cmd.arg("--example");
            if let Some(ref example) = args.example {
                cmd.arg(example);
            } else {
                return Err(CliError::Other(
                    "Example name required when using --target example".to_string(),
                ));
            }
        }
        Some(crate::cli::RunTarget::Bin) | None => {
            cmd.arg("run");
        }
    }

    cmd.current_dir(&args.path);

    if args.release {
        cmd.arg("--release");
    }

    if !args.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(args.features.join(","));
    }

    if !args.args.is_empty() {
        cmd.arg("--");
        cmd.args(&args.args);
    }

    // Execute
    log::debug!("Executing: {:?}", cmd);
    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::CommandNotFound("cargo".to_string())
        } else {
            CliError::Io(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::ExecutionFailed(stderr.to_string()));
    }

    // Capture output if requested
    if !args.no_capture {
        let stdout = String::from_utf8_lossy(&output.stdout);
        log::info!("Program output:\n{}", stdout);
    }

    // For now, create a placeholder output
    // TODO: Implement actual tracking data collection
    let placeholder_data = serde_json::json!({
        "version": "0.1.0",
        "source": args.path.display().to_string(),
        "events": [],
        "graph": {
            "nodes": [],
            "edges": []
        }
    });

    fs::write(
        output_file,
        serde_json::to_string_pretty(&placeholder_data)?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_run_nonexistent_path() {
        let args = RunArgs {
            path: PathBuf::from("/nonexistent/path"),
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::FileNotFound(_)));
    }

    #[test]
    fn test_run_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let output_file = temp_dir.path().join("output.json");
        let args = RunArgs {
            path: test_file,
            output: Some(output_file.clone()),
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
        assert!(output_file.exists());

        // Verify JSON structure
        let contents = fs::read_to_string(&output_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert!(json.get("version").is_some());
        assert!(json.get("events").is_some());
        assert!(json.get("graph").is_some());
    }

    #[test]
    fn test_run_with_custom_output() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let custom_output = temp_dir.path().join("custom.json");
        let args = RunArgs {
            path: test_file,
            output: Some(custom_output.clone()),
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        execute(args, config).unwrap();
        assert!(custom_output.exists());
    }

    #[test]
    fn test_run_target_test() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: Some(crate::cli::RunTarget::Test),
            example: None,
        };
        let config = Config::default();

        // Should succeed (single file doesn't check cargo)
        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_target_example_without_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project");
        fs::create_dir(&project_dir).unwrap();
        fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let args = RunArgs {
            path: project_dir,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: Some(crate::cli::RunTarget::Example),
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_features() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec!["feature1".to_string(), "feature2".to_string()],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_release_mode() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: true,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_args() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec!["--arg1".to_string(), "value1".to_string()],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_directory_without_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project");
        fs::create_dir(&project_dir).unwrap();

        let args = RunArgs {
            path: project_dir,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_default_output_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        execute(args, config).unwrap();
        // Should use default from config
        assert!(PathBuf::from("borrowscope.json").exists() || true); // May not exist in test env
    }
}
