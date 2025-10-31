# Section 88: Integration Testing

**tests/cli_integration.rs:**

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Visualize Rust ownership"));
}

#[test]
fn test_version() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_run_simple_project() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("test");
    
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("Cargo.toml"), r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
    "#).unwrap();
    
    fs::write(project.join("src/main.rs"), r#"
fn main() {
    let x = 42;
    println!("{}", x);
}
    "#).unwrap();
    
    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(&project)
        .arg("run")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let temp = TempDir::new().unwrap();
    
    fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    
    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    
    assert!(temp.path().join(".borrowscope.toml").exists());
}

#[test]
fn test_error_file_not_found() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("visualize")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}
```

**Progress:** 13/13 ✅
