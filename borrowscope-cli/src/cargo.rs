//! Cargo integration for building and running Rust projects
//!
//! This module provides comprehensive integration with Cargo for:
//! - Extracting project metadata
//! - Building projects programmatically
//! - Running binaries with custom environment
//! - Handling workspaces
//! - Parsing build output

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Cargo project metadata
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CargoMetadata {
    /// Workspace root directory
    pub workspace_root: String,
    /// List of workspace member IDs
    pub workspace_members: Vec<String>,
    /// All packages in the workspace
    pub packages: Vec<Package>,
    /// Target directory for build artifacts
    pub target_directory: String,
    /// Metadata format version
    pub version: u32,
}

/// Package information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Unique package ID
    pub id: String,
    /// Path to Cargo.toml
    pub manifest_path: String,
    /// Package dependencies
    pub dependencies: Vec<Dependency>,
    /// Build targets
    pub targets: Vec<Target>,
    /// Available features
    pub features: HashMap<String, Vec<String>>,
}

/// Dependency information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Version requirement
    pub req: String,
    /// Dependency kind (normal, dev, build)
    pub kind: Option<String>,
    /// Whether dependency is optional
    pub optional: bool,
}

/// Build target information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Target {
    /// Target name
    pub name: String,
    /// Target kinds (bin, lib, test, etc.)
    pub kind: Vec<String>,
    /// Crate types
    pub crate_types: Vec<String>,
    /// Source file path
    pub src_path: String,
}

/// Builder for cargo build operations
pub struct CargoBuilder {
    project_path: PathBuf,
    release: bool,
    features: Vec<String>,
    target: Option<String>,
    verbose: bool,
    all_features: bool,
    no_default_features: bool,
}

impl CargoBuilder {
    /// Create a new cargo builder
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            release: false,
            features: Vec::new(),
            target: None,
            verbose: false,
            all_features: false,
            no_default_features: false,
        }
    }

    /// Set release mode
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    /// Set features to enable
    pub fn features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    /// Enable all features
    pub fn all_features(mut self, all: bool) -> Self {
        self.all_features = all;
        self
    }

    /// Disable default features
    pub fn no_default_features(mut self, no_default: bool) -> Self {
        self.no_default_features = no_default;
        self
    }

    /// Set target triple
    pub fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    /// Set verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute the build
    pub fn build(&self) -> Result<BuildResult> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        cmd.current_dir(&self.project_path);

        if self.release {
            cmd.arg("--release");
        }

        if self.all_features {
            cmd.arg("--all-features");
        }

        if self.no_default_features {
            cmd.arg("--no-default-features");
        }

        if !self.features.is_empty() {
            cmd.arg("--features");
            cmd.arg(self.features.join(","));
        }

        if let Some(ref target) = self.target {
            cmd.arg("--target");
            cmd.arg(target);
        }

        if self.verbose {
            cmd.arg("--verbose");
        }

        cmd.arg("--message-format=json");

        let output = cmd.output().context("Failed to execute cargo build")?;

        parse_build_output(output)
    }

    /// Clean build artifacts
    pub fn clean(&self) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("clean");
        cmd.current_dir(&self.project_path);

        let status = cmd.status().context("Failed to execute cargo clean")?;

        if !status.success() {
            anyhow::bail!("cargo clean failed");
        }

        Ok(())
    }
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    /// Whether build succeeded
    pub success: bool,
    /// Built artifacts
    pub artifacts: Vec<Artifact>,
    /// Compilation errors
    pub errors: Vec<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
}

/// Build artifact
#[derive(Debug)]
pub struct Artifact {
    /// Target name
    pub target: String,
    /// Output file paths
    pub filenames: Vec<PathBuf>,
    /// Whether artifact was fresh (not rebuilt)
    pub fresh: bool,
}

fn parse_build_output(output: Output) -> Result<BuildResult> {
    let mut result = BuildResult {
        success: output.status.success(),
        artifacts: Vec::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
            match message.get("reason").and_then(|r| r.as_str()) {
                Some("compiler-artifact") => {
                    if let Some(artifact) = parse_artifact(&message) {
                        result.artifacts.push(artifact);
                    }
                }
                Some("compiler-message") => {
                    if let Some(msg) = message.get("message") {
                        if let Some(level) = msg.get("level").and_then(|l| l.as_str()) {
                            let text = msg
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("")
                                .to_string();

                            match level {
                                "error" => result.errors.push(text),
                                "warning" => result.warnings.push(text),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if !output.status.success() && result.errors.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            result.errors.push(stderr.to_string());
        }
    }

    Ok(result)
}

fn parse_artifact(message: &serde_json::Value) -> Option<Artifact> {
    let target = message.get("target")?.get("name")?.as_str()?.to_string();

    let filenames = message
        .get("filenames")?
        .as_array()?
        .iter()
        .filter_map(|f| f.as_str())
        .map(PathBuf::from)
        .collect();

    let fresh = message
        .get("fresh")
        .and_then(|f| f.as_bool())
        .unwrap_or(false);

    Some(Artifact {
        target,
        filenames,
        fresh,
    })
}

/// Runner for cargo run operations
pub struct CargoRunner {
    project_path: PathBuf,
    release: bool,
    args: Vec<String>,
    env: HashMap<String, String>,
    example: Option<String>,
}

impl CargoRunner {
    /// Create a new cargo runner
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            release: false,
            args: Vec::new(),
            env: HashMap::new(),
            example: None,
        }
    }

    /// Set release mode
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    /// Set program arguments
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: String, value: String) -> Self {
        self.env.insert(key, value);
        self
    }

    /// Run specific example
    pub fn example(mut self, name: String) -> Self {
        self.example = Some(name);
        self
    }

    /// Execute the program
    pub fn run(&self) -> Result<Output> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run");
        cmd.current_dir(&self.project_path);

        if self.release {
            cmd.arg("--release");
        }

        if let Some(ref example) = self.example {
            cmd.arg("--example");
            cmd.arg(example);
        }

        if !self.args.is_empty() {
            cmd.arg("--");
            cmd.args(&self.args);
        }

        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        cmd.output().context("Failed to execute cargo run")
    }

    /// Execute and return stdout as string
    pub fn run_with_output(&self) -> Result<String> {
        let output = self.run()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Program failed:\n{}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Get cargo metadata for a project
pub fn get_metadata(path: &Path) -> Result<CargoMetadata> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .current_dir(path)
        .output()
        .context("Failed to run cargo metadata")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo metadata failed:\n{}", stderr);
    }

    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).context("Failed to parse cargo metadata")?;

    Ok(metadata)
}

/// Get package information for a specific path
pub fn get_package_info(path: &Path) -> Result<Package> {
    let metadata = get_metadata(path)?;

    // Find the package for this path
    let manifest_path = path.join("Cargo.toml");
    let manifest_str = manifest_path.to_string_lossy();

    metadata
        .packages
        .into_iter()
        .find(|p| p.manifest_path.contains(manifest_str.as_ref()))
        .ok_or_else(|| anyhow::anyhow!("Package not found in metadata"))
}

/// List available features
pub fn list_features(path: &Path) -> Result<Vec<String>> {
    let package = get_package_info(path)?;
    Ok(package.features.keys().cloned().collect())
}

/// List build targets
pub fn list_targets(path: &Path) -> Result<Vec<Target>> {
    let package = get_package_info(path)?;
    Ok(package.targets)
}

/// Check if path is a workspace
pub fn is_workspace(path: &Path) -> Result<bool> {
    let metadata = get_metadata(path)?;
    Ok(metadata.workspace_members.len() > 1)
}

/// List workspace members
pub fn list_workspace_members(path: &Path) -> Result<Vec<String>> {
    let metadata = get_metadata(path)?;
    Ok(metadata.workspace_members)
}

/// Check project for errors
pub fn check_project(path: &Path) -> Result<CheckResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.arg("--message-format=json");
    cmd.current_dir(path);

    let output = cmd.output().context("Failed to run cargo check")?;

    let mut result = CheckResult {
        success: output.status.success(),
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some("compiler-message") = message.get("reason").and_then(|r| r.as_str()) {
                if let Some(msg) = message.get("message") {
                    let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("");
                    let text = msg
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();

                    match level {
                        "error" => result.errors.push(text),
                        "warning" => result.warnings.push(text),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Check result
#[derive(Debug)]
pub struct CheckResult {
    /// Whether check succeeded
    pub success: bool,
    /// Compilation errors
    pub errors: Vec<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
}

/// Check if cargo is available
pub fn is_cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get cargo version
pub fn get_cargo_version() -> Result<String> {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .context("Failed to get cargo version")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get cargo version");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(dir: &Path, name: &str) {
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[features]
feature1 = []
feature2 = []

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                name, name
            ),
        )
        .unwrap();

        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("src/main.rs"),
            r#"
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
        )
        .unwrap();
    }

    #[test]
    fn test_is_cargo_available() {
        assert!(is_cargo_available());
    }

    #[test]
    fn test_get_cargo_version() {
        let version = get_cargo_version().unwrap();
        assert!(version.contains("cargo"));
    }

    #[test]
    fn test_get_metadata() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "test_project");

        let metadata = get_metadata(temp.path()).unwrap();
        assert_eq!(metadata.packages.len(), 1);
        assert_eq!(metadata.packages[0].name, "test_project");
    }

    #[test]
    fn test_get_package_info() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "test_pkg");

        let package = get_package_info(temp.path()).unwrap();
        assert_eq!(package.name, "test_pkg");
        assert_eq!(package.version, "0.1.0");
    }

    #[test]
    fn test_list_features() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "test_features");

        let features = list_features(temp.path()).unwrap();
        assert!(features.contains(&"feature1".to_string()));
        assert!(features.contains(&"feature2".to_string()));
    }

    #[test]
    fn test_list_targets() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "test_targets");

        let targets = list_targets(temp.path()).unwrap();
        assert!(!targets.is_empty());
        assert!(targets.iter().any(|t| t.name == "test_targets"));
    }

    #[test]
    fn test_is_workspace() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "single_project");

        let is_ws = is_workspace(temp.path()).unwrap();
        assert!(!is_ws);
    }

    #[test]
    fn test_cargo_builder_new() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf());
        assert!(!builder.release);
        assert!(builder.features.is_empty());
    }

    #[test]
    fn test_cargo_builder_release() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf()).release(true);
        assert!(builder.release);
    }

    #[test]
    fn test_cargo_builder_features() {
        let temp = TempDir::new().unwrap();
        let features = vec!["feat1".to_string(), "feat2".to_string()];
        let builder = CargoBuilder::new(temp.path().to_path_buf()).features(features.clone());
        assert_eq!(builder.features, features);
    }

    #[test]
    fn test_cargo_builder_all_features() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf()).all_features(true);
        assert!(builder.all_features);
    }

    #[test]
    fn test_cargo_builder_build() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "build_test");

        let builder = CargoBuilder::new(temp.path().to_path_buf());
        let result = builder.build().unwrap();

        assert!(result.success);
        assert!(!result.artifacts.is_empty());
    }

    #[test]
    fn test_cargo_builder_clean() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "clean_test");

        let builder = CargoBuilder::new(temp.path().to_path_buf());
        builder.build().unwrap();

        assert!(builder.clean().is_ok());
    }

    #[test]
    fn test_cargo_runner_new() {
        let temp = TempDir::new().unwrap();
        let runner = CargoRunner::new(temp.path().to_path_buf());
        assert!(!runner.release);
        assert!(runner.args.is_empty());
        assert!(runner.env.is_empty());
    }

    #[test]
    fn test_cargo_runner_args() {
        let temp = TempDir::new().unwrap();
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let runner = CargoRunner::new(temp.path().to_path_buf()).args(args.clone());
        assert_eq!(runner.args, args);
    }

    #[test]
    fn test_cargo_runner_env() {
        let temp = TempDir::new().unwrap();
        let runner =
            CargoRunner::new(temp.path().to_path_buf()).env("KEY".to_string(), "VALUE".to_string());
        assert_eq!(runner.env.get("KEY"), Some(&"VALUE".to_string()));
    }

    #[test]
    fn test_cargo_runner_run() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "run_test");

        let runner = CargoRunner::new(temp.path().to_path_buf());
        let output = runner.run().unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Hello, world!"));
    }

    #[test]
    fn test_cargo_runner_run_with_output() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "output_test");

        let runner = CargoRunner::new(temp.path().to_path_buf());
        let output = runner.run_with_output().unwrap();

        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_check_project() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "check_test");

        let result = check_project(temp.path()).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_project_with_error() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "error_test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/main.rs"),
            r#"
fn main() {
    let x = undefined_variable;
}
"#,
        )
        .unwrap();

        let result = check_project(temp.path()).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_list_workspace_members() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "ws_test");

        let members = list_workspace_members(temp.path()).unwrap();
        assert!(!members.is_empty());
    }

    // Edge Case Tests

    #[test]
    fn test_metadata_nonexistent_path() {
        let result = get_metadata(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_invalid_cargo_toml() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("Cargo.toml"), "invalid toml content [[[").unwrap();

        let result = get_metadata(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_package_info_no_cargo_toml() {
        let temp = TempDir::new().unwrap();
        let result = get_package_info(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_multiple_features() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "multi_feat");

        let builder = CargoBuilder::new(temp.path().to_path_buf())
            .features(vec!["feature1".into(), "feature2".into()]);

        assert_eq!(builder.features.len(), 2);
    }

    #[test]
    fn test_builder_conflicting_feature_flags() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf())
            .all_features(true)
            .no_default_features(true)
            .features(vec!["feat1".into()]);

        assert!(builder.all_features);
        assert!(builder.no_default_features);
        assert_eq!(builder.features.len(), 1);
    }

    #[test]
    fn test_builder_with_target_triple() {
        let temp = TempDir::new().unwrap();
        let builder =
            CargoBuilder::new(temp.path().to_path_buf()).target("x86_64-unknown-linux-gnu".into());

        assert_eq!(builder.target, Some("x86_64-unknown-linux-gnu".into()));
    }

    #[test]
    fn test_builder_verbose_mode() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf()).verbose(true);
        assert!(builder.verbose);
    }

    #[test]
    fn test_build_nonexistent_project() {
        let builder = CargoBuilder::new(PathBuf::from("/nonexistent/path/to/project"));
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_syntax_error() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "syntax_error"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() { let x = ; }").unwrap();

        let builder = CargoBuilder::new(temp.path().to_path_buf());
        let result = builder.build().unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_build_release_mode() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "release_test");

        let builder = CargoBuilder::new(temp.path().to_path_buf()).release(true);
        let result = builder.build().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_clean_nonexistent_project() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf());
        let result = builder.clean();
        assert!(result.is_err());
    }

    #[test]
    fn test_runner_with_multiple_args() {
        let temp = TempDir::new().unwrap();
        let args = vec!["arg1".into(), "arg2".into(), "arg3".into()];
        let runner = CargoRunner::new(temp.path().to_path_buf()).args(args.clone());
        assert_eq!(runner.args.len(), 3);
    }

    #[test]
    fn test_runner_with_multiple_env_vars() {
        let temp = TempDir::new().unwrap();
        let runner = CargoRunner::new(temp.path().to_path_buf())
            .env("VAR1".into(), "val1".into())
            .env("VAR2".into(), "val2".into());

        assert_eq!(runner.env.len(), 2);
        assert_eq!(runner.env.get("VAR1"), Some(&"val1".to_string()));
    }

    #[test]
    fn test_runner_example_mode() {
        let temp = TempDir::new().unwrap();
        let runner = CargoRunner::new(temp.path().to_path_buf()).example("my_example".into());

        assert_eq!(runner.example, Some("my_example".into()));
    }

    #[test]
    fn test_runner_nonexistent_project() {
        let runner = CargoRunner::new(PathBuf::from("/nonexistent/path/to/project"));
        let result = runner.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_runner_with_failing_program() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "failing_prog"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/main.rs"),
            r#"
fn main() {
    std::process::exit(1);
}
"#,
        )
        .unwrap();

        let runner = CargoRunner::new(temp.path().to_path_buf());
        let output = runner.run().unwrap();
        assert!(!output.status.success());
    }

    #[test]
    fn test_runner_output_with_failure() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "fail_output"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/main.rs"),
            r#"
fn main() {
    eprintln!("Error message");
    std::process::exit(1);
}
"#,
        )
        .unwrap();

        let runner = CargoRunner::new(temp.path().to_path_buf());
        let result = runner.run_with_output();
        assert!(result.is_err());
    }

    #[test]
    fn test_check_with_warnings() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "warning_test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/main.rs"),
            r#"
fn main() {
    let unused_var = 42;
}
"#,
        )
        .unwrap();

        let result = check_project(temp.path()).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_check_nonexistent_project() {
        let result = check_project(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_features_empty() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "no_features"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let features = list_features(temp.path()).unwrap();
        assert!(features.is_empty());
    }

    #[test]
    fn test_list_targets_multiple() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "multi_target"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "bin1"
path = "src/bin1.rs"

[[bin]]
name = "bin2"
path = "src/bin2.rs"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/bin1.rs"), "fn main() {}").unwrap();
        fs::write(temp.path().join("src/bin2.rs"), "fn main() {}").unwrap();

        let targets = list_targets(temp.path()).unwrap();
        assert!(targets.len() >= 2);
    }

    #[test]
    fn test_workspace_detection_single_package() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "single");

        let is_ws = is_workspace(temp.path()).unwrap();
        assert!(!is_ws);
    }

    #[test]
    fn test_workspace_with_virtual_manifest() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["member1", "member2"]
"#,
        )
        .unwrap();

        for member in &["member1", "member2"] {
            let member_path = temp.path().join(member);
            fs::create_dir_all(&member_path).unwrap();
            create_test_project(&member_path, member);
        }

        let is_ws = is_workspace(temp.path()).unwrap();
        assert!(is_ws);
    }

    #[test]
    fn test_cargo_version_format() {
        let version = get_cargo_version().unwrap();
        assert!(version.starts_with("cargo"));
        assert!(version.contains('.'));
    }

    #[test]
    fn test_metadata_with_dependencies() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "with_deps"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let metadata = get_metadata(temp.path()).unwrap();
        let package = &metadata.packages[0];
        assert!(!package.dependencies.is_empty());
    }

    #[test]
    fn test_package_with_dev_dependencies() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "dev_deps"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
tempfile = "3.0"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let package = get_package_info(temp.path()).unwrap();
        assert!(!package.dependencies.is_empty());
    }

    #[test]
    fn test_builder_chain_all_options() {
        let temp = TempDir::new().unwrap();
        let builder = CargoBuilder::new(temp.path().to_path_buf())
            .release(true)
            .features(vec!["feat1".into()])
            .target("x86_64-unknown-linux-gnu".into())
            .verbose(true)
            .all_features(false)
            .no_default_features(true);

        assert!(builder.release);
        assert_eq!(builder.features.len(), 1);
        assert!(builder.target.is_some());
        assert!(builder.verbose);
        assert!(builder.no_default_features);
    }

    #[test]
    fn test_runner_chain_all_options() {
        let temp = TempDir::new().unwrap();
        let runner = CargoRunner::new(temp.path().to_path_buf())
            .release(true)
            .args(vec!["arg".into()])
            .env("KEY".into(), "VAL".into())
            .example("ex".into());

        assert!(runner.release);
        assert_eq!(runner.args.len(), 1);
        assert_eq!(runner.env.len(), 1);
        assert_eq!(runner.example, Some("ex".into()));
    }

    #[test]
    fn test_build_result_with_warnings() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "warn_build"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/main.rs"),
            r#"
fn main() {
    let x = 42;
}
"#,
        )
        .unwrap();

        let builder = CargoBuilder::new(temp.path().to_path_buf());
        let result = builder.build().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_metadata_target_directory() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "target_test");

        let metadata = get_metadata(temp.path()).unwrap();
        assert!(metadata.target_directory.contains("target"));
    }

    #[test]
    fn test_package_manifest_path() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path(), "manifest_test");

        let package = get_package_info(temp.path()).unwrap();
        assert!(package.manifest_path.contains("Cargo.toml"));
    }

    #[test]
    fn test_empty_project_path() {
        let builder = CargoBuilder::new(PathBuf::new());
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_runner_empty_args() {
        let temp = TempDir::new().unwrap();
        let runner = CargoRunner::new(temp.path().to_path_buf()).args(vec![]);
        assert!(runner.args.is_empty());
    }

    #[test]
    fn test_features_with_dependencies() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "feat_deps"
version = "0.1.0"
edition = "2021"

[features]
default = ["feat1"]
feat1 = []
feat2 = ["feat1"]
"#,
        )
        .unwrap();

        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let features = list_features(temp.path()).unwrap();
        assert!(features.contains(&"default".to_string()));
        assert!(features.contains(&"feat1".to_string()));
        assert!(features.contains(&"feat2".to_string()));
    }
}
