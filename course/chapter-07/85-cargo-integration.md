# Section 85: Cargo Integration

## Learning Objectives

By the end of this section, you will:
- Extract metadata from Cargo projects
- Build projects programmatically
- Run binaries with custom environment
- Handle workspaces and multiple packages
- Parse Cargo output and errors

## Prerequisites

- Section 84 (Temporary File Management)
- Understanding of Cargo commands
- Familiarity with JSON parsing

---

## Purpose

Cargo integration enables:
1. Extracting project metadata (dependencies, features, targets)
2. Building projects with custom flags
3. Running binaries with environment variables
4. Handling workspaces with multiple crates
5. Parsing and interpreting Cargo output

---

## Complete Implementation

**src/cargo.rs:**

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoMetadata {
    pub workspace_root: String,
    pub workspace_members: Vec<String>,
    pub packages: Vec<Package>,
    pub target_directory: String,
    pub version: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub id: String,
    pub manifest_path: String,
    pub dependencies: Vec<Dependency>,
    pub targets: Vec<Target>,
    pub features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub req: String,
    pub kind: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    pub src_path: String,
}

pub struct CargoBuilder {
    project_path: PathBuf,
    release: bool,
    features: Vec<String>,
    target: Option<String>,
    verbose: bool,
}

impl CargoBuilder {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            release: false,
            features: Vec::new(),
            target: None,
            verbose: false,
        }
    }
    
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }
    
    pub fn features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
    
    pub fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
    
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn build(&self) -> Result<BuildResult> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        cmd.current_dir(&self.project_path);
        
        if self.release {
            cmd.arg("--release");
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
        
        let output = cmd.output()
            .context("Failed to execute cargo build")?;
        
        parse_build_output(output)
    }
    
    pub fn clean(&self) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("clean");
        cmd.current_dir(&self.project_path);
        
        let status = cmd.status()
            .context("Failed to execute cargo clean")?;
        
        if !status.success() {
            anyhow::bail!("cargo clean failed");
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub artifacts: Vec<Artifact>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct Artifact {
    pub target: String,
    pub filenames: Vec<PathBuf>,
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
                            let text = msg.get("message")
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
        result.errors.push(stderr.to_string());
    }
    
    Ok(result)
}

fn parse_artifact(message: &serde_json::Value) -> Option<Artifact> {
    let target = message.get("target")?
        .get("name")?
        .as_str()?
        .to_string();
    
    let filenames = message.get("filenames")?
        .as_array()?
        .iter()
        .filter_map(|f| f.as_str())
        .map(PathBuf::from)
        .collect();
    
    let fresh = message.get("fresh")
        .and_then(|f| f.as_bool())
        .unwrap_or(false);
    
    Some(Artifact {
        target,
        filenames,
        fresh,
    })
}

pub struct CargoRunner {
    project_path: PathBuf,
    release: bool,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl CargoRunner {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            release: false,
            args: Vec::new(),
            env: HashMap::new(),
        }
    }
    
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }
    
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    
    pub fn env(mut self, key: String, value: String) -> Self {
        self.env.insert(key, value);
        self
    }
    
    pub fn run(&self) -> Result<Output> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run");
        cmd.current_dir(&self.project_path);
        
        if self.release {
            cmd.arg("--release");
        }
        
        if !self.args.is_empty() {
            cmd.arg("--");
            cmd.args(&self.args);
        }
        
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        
        cmd.output()
            .context("Failed to execute cargo run")
    }
    
    pub fn run_with_output(&self) -> Result<String> {
        let output = self.run()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Program failed:\n{}", stderr);
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

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
    
    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)
        .context("Failed to parse cargo metadata")?;
    
    Ok(metadata)
}

pub fn get_package_info(path: &Path) -> Result<Package> {
    let metadata = get_metadata(path)?;
    
    // Find the package for this path
    let manifest_path = path.join("Cargo.toml");
    let manifest_str = manifest_path.to_string_lossy();
    
    metadata.packages
        .into_iter()
        .find(|p| p.manifest_path == manifest_str)
        .ok_or_else(|| anyhow::anyhow!("Package not found in metadata"))
}

pub fn list_features(path: &Path) -> Result<Vec<String>> {
    let package = get_package_info(path)?;
    Ok(package.features.keys().cloned().collect())
}

pub fn list_targets(path: &Path) -> Result<Vec<Target>> {
    let package = get_package_info(path)?;
    Ok(package.targets)
}

pub fn is_workspace(path: &Path) -> Result<bool> {
    let metadata = get_metadata(path)?;
    Ok(metadata.workspace_members.len() > 1)
}

pub fn list_workspace_members(path: &Path) -> Result<Vec<String>> {
    let metadata = get_metadata(path)?;
    Ok(metadata.workspace_members)
}

pub fn check_project(path: &Path) -> Result<CheckResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.arg("--message-format=json");
    cmd.current_dir(path);
    
    let output = cmd.output()
        .context("Failed to run cargo check")?;
    
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
                    let text = msg.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
                    
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

#[derive(Debug)]
pub struct CheckResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn test_project(path: &Path) -> Result<TestResult> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.arg("--");
    cmd.arg("--nocapture");
    cmd.current_dir(path);
    
    let output = cmd.output()
        .context("Failed to run cargo test")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse test output
    let mut passed = 0;
    let mut failed = 0;
    
    for line in stdout.lines() {
        if line.contains("test result:") {
            // Parse: "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
            if let Some(parts) = line.split("test result:").nth(1) {
                if let Some(passed_str) = parts.split("passed").next() {
                    if let Some(num) = passed_str.split_whitespace().last() {
                        passed = num.parse().unwrap_or(0);
                    }
                }
                if let Some(failed_str) = parts.split("failed").next() {
                    if let Some(num) = failed_str.split(';').nth(1).and_then(|s| s.split_whitespace().next()) {
                        failed = num.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    
    Ok(TestResult {
        success: output.status.success(),
        passed,
        failed,
        output: stdout.to_string(),
    })
}

#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub passed: usize,
    pub failed: usize,
    pub output: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_project(dir: &Path) {
        fs::write(
            dir.join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[features]
feature1 = []
feature2 = []
            "#
        ).unwrap();
        
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("src/main.rs"),
            r#"
fn main() {
    println!("Hello, world!");
}
            "#
        ).unwrap();
    }

    #[test]
    fn test_get_metadata() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());
        
        let metadata = get_metadata(temp.path()).unwrap();
        assert_eq!(metadata.packages.len(), 1);
        assert_eq!(metadata.packages[0].name, "test_project");
    }

    #[test]
    fn test_list_features() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());
        
        let features = list_features(temp.path()).unwrap();
        assert!(features.contains(&"feature1".to_string()));
        assert!(features.contains(&"feature2".to_string()));
    }

    #[test]
    fn test_cargo_builder() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());
        
        let builder = CargoBuilder::new(temp.path().to_path_buf())
            .release(false)
            .verbose(false);
        
        let result = builder.build().unwrap();
        assert!(result.success);
    }
}
```

---

## Usage Examples

```rust
use borrowscope_cli::cargo::*;

// Get project metadata
let metadata = get_metadata(Path::new("."))?;
println!("Workspace root: {}", metadata.workspace_root);

// Build project
let result = CargoBuilder::new(PathBuf::from("."))
    .release(true)
    .features(vec!["feature1".into()])
    .build()?;

if result.success {
    println!("Build successful!");
    for artifact in result.artifacts {
        println!("  Built: {}", artifact.target);
    }
}

// Run project
let output = CargoRunner::new(PathBuf::from("."))
    .release(false)
    .args(vec!["--help".into()])
    .env("RUST_LOG".into(), "debug".into())
    .run_with_output()?;

println!("Output: {}", output);
```

---

## Key Takeaways

✅ **Metadata extraction** - Get project information  
✅ **Build integration** - Programmatic builds  
✅ **Run integration** - Execute with custom environment  
✅ **Workspace support** - Handle multi-crate projects  
✅ **Output parsing** - Extract errors and warnings  

---

**Previous:** [84-temp-file-management.md](./84-temp-file-management.md)  
**Next:** [86-configuration-parsing.md](./86-configuration-parsing.md)

**Progress:** 10/13 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜
