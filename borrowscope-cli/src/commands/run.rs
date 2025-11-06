//! Run command implementation

use std::fs;
use std::path::PathBuf;

use crate::cargo::{CargoBuilder, CargoRunner};
use crate::cli::RunArgs;
use crate::config::Config;
use crate::error::{CliError, Result};
use crate::instrumentation::Instrumenter;
use crate::progress::{build_progress, spinner};
use crate::utils::TempWorkspace;

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

    // Step 1: Create temporary workspace
    let sp = spinner("Creating temporary workspace");
    let mut workspace = TempWorkspace::new().map_err(|e| CliError::Other(e.to_string()))?;
    let project_copy = workspace
        .copy_project(&args.path)
        .map_err(|e| CliError::Other(e.to_string()))?;
    sp.finish_with_message("✓ Workspace created");

    // Step 2: Instrument the project
    let pb = build_progress("Instrumenting project");
    let instrumented_dir = project_copy.join("instrumented");
    fs::create_dir_all(&instrumented_dir)?;

    let config_inst = crate::instrumentation::InstrumentationConfig::default();
    let instrumenter =
        Instrumenter::new(project_copy.clone(), instrumented_dir.clone(), config_inst);
    instrumenter
        .instrument_project()
        .map_err(|e| CliError::Other(e.to_string()))?;
    pb.finish_with_message("✓ Instrumentation complete");

    // Step 3: Build the instrumented project
    let pb = build_progress("Building project");
    let mut builder = CargoBuilder::new(instrumented_dir.clone()).release(args.release);

    if !args.features.is_empty() {
        builder = builder.features(args.features.clone());
    }

    let build_result = builder
        .build()
        .map_err(|e| CliError::Other(e.to_string()))?;
    if !build_result.success {
        return Err(CliError::ExecutionFailed(build_result.errors.join("\n")));
    }
    pb.finish_with_message("✓ Build complete");

    // Step 4: Run the instrumented binary
    let pb = build_progress("Running instrumented binary");
    let mut runner = CargoRunner::new(instrumented_dir.clone())
        .release(args.release)
        .env(
            "BORROWSCOPE_OUTPUT".to_string(),
            output_file.display().to_string(),
        );

    if !args.args.is_empty() {
        runner = runner.args(args.args.clone());
    }

    // Handle different targets
    if let Some(ref target) = args.target {
        match target {
            crate::cli::RunTarget::Example => {
                if let Some(ref example) = args.example {
                    runner = runner.example(example.clone());
                } else {
                    return Err(CliError::Other(
                        "Example name required when using --target example".to_string(),
                    ));
                }
            }
            _ => {}
        }
    }

    let run_output = runner.run().map_err(|e| CliError::Other(e.to_string()))?;

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        return Err(CliError::ExecutionFailed(stderr.to_string()));
    }

    // Capture output if requested
    if !args.no_capture {
        let stdout = String::from_utf8_lossy(&run_output.stdout);
        log::info!("Program output:\n{}", stdout);
    }
    pb.finish_with_message("✓ Execution complete");

    // Step 5: Collect tracking data
    let sp = spinner("Collecting tracking data");
    // Check if runtime generated tracking data
    if output_file.exists() {
        sp.finish_with_message("✓ Tracking data collected");
    } else {
        // Fallback to placeholder if runtime didn't generate data
        let placeholder_data = serde_json::json!({
            "version": "0.1.0",
            "source": args.path.display().to_string(),
            "events": [],
            "graph": {
                "nodes": [],
                "edges": []
            },
            "note": "Runtime tracking data not available"
        });

        fs::write(
            output_file,
            serde_json::to_string_pretty(&placeholder_data)?,
        )?;
        sp.finish_with_message("⚠ Using placeholder data");
    }

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
    }

    #[test]
    fn test_run_with_no_capture() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"test\"); }").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: true,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_multiple_features() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: false,
            features: vec![
                "feat1".to_string(),
                "feat2".to_string(),
                "feat3".to_string(),
            ],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_multiple_args() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()],
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
    fn test_run_release_with_features() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let args = RunArgs {
            path: test_file,
            output: None,
            visualize: false,
            args: vec![],
            release: true,
            features: vec!["feature1".to_string()],
            no_capture: false,
            target: None,
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_all_options_combined() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let output_file = temp_dir.path().join("output.json");
        let args = RunArgs {
            path: test_file,
            output: Some(output_file.clone()),
            visualize: false,
            args: vec!["--arg".to_string()],
            release: true,
            features: vec!["feat1".to_string()],
            no_capture: true,
            target: Some(crate::cli::RunTarget::Bin),
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_run_output_json_structure() {
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

        execute(args, config).unwrap();

        let contents = fs::read_to_string(&output_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(json.get("version").is_some());
        assert!(json.get("source").is_some());
        assert!(json.get("events").is_some());
        assert!(json.get("graph").is_some());
    }

    #[test]
    fn test_run_target_bin_explicit() {
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
            target: Some(crate::cli::RunTarget::Bin),
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_target_bench() {
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
            target: Some(crate::cli::RunTarget::Bench),
            example: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_output_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let output_file = temp_dir.path().join("output.json");

        // First run
        let args1 = RunArgs {
            path: test_file.clone(),
            output: Some(output_file.clone()),
            visualize: false,
            args: vec![],
            release: false,
            features: vec![],
            no_capture: false,
            target: None,
            example: None,
        };
        execute(args1, Config::default()).unwrap();

        // Second run should overwrite
        let args2 = RunArgs {
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
        let result = execute(args2, Config::default());
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_run_special_characters_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test file.rs");
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

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_unicode_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("测试.rs");
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

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_very_long_path() {
        let temp_dir = TempDir::new().unwrap();
        let long_name = "a".repeat(100) + ".rs";
        let test_file = temp_dir.path().join(long_name);
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

        let result = execute(args, config);
        assert!(result.is_ok());
    }
}
