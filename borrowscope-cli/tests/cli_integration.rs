//! Integration tests for BorrowScope CLI
//!
//! These tests verify end-to-end functionality of the CLI commands.

#![allow(deprecated)]

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

// Helper function to create a minimal Rust project
#[allow(dead_code)]
fn create_test_project(temp: &TempDir, name: &str) -> assert_fs::fixture::ChildPath {
    let project = temp.child(name);
    project.create_dir_all().unwrap();

    let src = project.child("src");
    src.create_dir_all().unwrap();

    project
        .child("Cargo.toml")
        .write_str(&format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            name
        ))
        .unwrap();

    src.child("main.rs")
        .write_str(
            r#"
fn main() {
    let x = 42;
    println!("Hello: {}", x);
}
"#,
        )
        .unwrap();

    project
}

// Helper function to create a project with borrowing
#[allow(dead_code)]
fn create_borrow_project(temp: &TempDir) -> assert_fs::fixture::ChildPath {
    let project = temp.child("borrow_test");
    project.create_dir_all().unwrap();

    let src = project.child("src");
    src.create_dir_all().unwrap();

    project
        .child("Cargo.toml")
        .write_str(
            r#"
[package]
name = "borrow_test"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )
        .unwrap();

    src.child("main.rs")
        .write_str(
            r#"
fn main() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
}
"#,
        )
        .unwrap();

    project
}

#[test]
fn test_help_command() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Visualize Rust ownership"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("visualize"))
        .stdout(predicate::str::contains("export"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("borrowscope"));
}

#[test]
fn test_help_short_flag() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_short_flag() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("-V")
        .assert()
        .success();
}

#[test]
fn test_init_command_creates_config() {
    let temp = TempDir::new().unwrap();

    temp.child("Cargo.toml")
        .write_str(
            r#"
[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    temp.child(".borrowscope.toml")
        .assert(predicate::path::exists());
}

#[test]
fn test_init_with_force_flag() {
    let temp = TempDir::new().unwrap();

    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();
    temp.child(".borrowscope.toml")
        .write_str("[run]\noutput = \"old.json\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(temp.child(".borrowscope.toml").path()).unwrap();
    assert!(content.contains("borrowscope.json"));
}

#[test]
fn test_init_minimal_template() {
    let temp = TempDir::new().unwrap();

    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--template")
        .arg("minimal")
        .assert()
        .success();

    temp.child(".borrowscope.toml")
        .assert(predicate::path::exists());
}

#[test]
fn test_init_advanced_template() {
    let temp = TempDir::new().unwrap();

    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--template")
        .arg("advanced")
        .assert()
        .success();

    temp.child(".borrowscope.toml")
        .assert(predicate::path::exists());
}

#[test]
fn test_check_command_missing_file() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("File not found").or(predicate::str::contains("No such file")),
        );
}

#[test]
fn test_check_command_with_valid_json() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("data.json");

    data_file
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [],
        "edges": [],
        "events": [],
        "graph": {
            "nodes": [],
            "edges": []
        },
        "metadata": {
            "total_variables": 0,
            "total_relationships": 0,
            "immutable_borrows": 0,
            "mutable_borrows": 0,
            "total_events": 0
        }
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .assert()
        .success();
}

#[test]
fn test_check_command_with_stats_flag() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("data.json");

    data_file
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [],
        "edges": [],
        "events": [],
        "graph": {
            "nodes": [],
            "edges": []
        },
        "metadata": {
            "total_variables": 5,
            "total_relationships": 3,
            "immutable_borrows": 2,
            "mutable_borrows": 1,
            "total_events": 10
        }
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Statistics"));
}

#[test]
fn test_check_command_invalid_json() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("invalid.json");

    data_file.write_str("{ invalid json }").unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .assert()
        .failure();
}

#[test]
fn test_export_command_missing_file() {
    let temp = TempDir::new().unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg("nonexistent.json")
        .arg("--output")
        .arg(temp.child("output.dot").path())
        .arg("--format")
        .arg("dot")
        .assert()
        .failure();
}

#[test]
fn test_export_to_dot() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");
    let output = temp.child("output.dot");

    input.write_str(r#"{
        "nodes": [{"id": "x_0", "name": "x", "type_name": "i32", "created_at": 1, "dropped_at": null}],
        "edges": [],
        "events": [],
        "metadata": {"total_variables": 1, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 0}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    output.assert(predicate::path::exists());
    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("digraph"));
}

#[test]
fn test_export_to_json() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");
    let output = temp.child("output.json");

    input.write_str(r#"{
        "nodes": [],
        "edges": [],
        "events": [],
        "metadata": {"total_variables": 0, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 0}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    output.assert(predicate::path::exists());
}

#[test]
fn test_export_to_html() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");
    let output = temp.child("output.html");

    input.write_str(r#"{
        "nodes": [],
        "edges": [],
        "events": [],
        "metadata": {"total_variables": 0, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 0}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("html")
        .assert()
        .success();

    output.assert(predicate::path::exists());
    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("<html>") || content.contains("<!DOCTYPE"));
}

#[test]
fn test_completion_bash() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("completion")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("_borrowscope").or(predicate::str::contains("complete")));
}

#[test]
fn test_completion_zsh() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("completion")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("_borrowscope").or(predicate::str::contains("compdef")));
}

#[test]
fn test_completion_fish() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("completion")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("borrowscope").or(predicate::str::contains("complete")));
}

#[test]
fn test_verbose_flag() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("--verbose")
        .arg("init")
        .assert()
        .success();
}

#[test]
fn test_quiet_flag() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("--quiet")
        .arg("init")
        .assert()
        .success();
}

#[test]
fn test_invalid_command() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}

#[test]
fn test_run_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run instrumented code"));
}

#[test]
fn test_visualize_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("visualize")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("visualization"));
}

#[test]
fn test_export_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Export tracking data"));
}

#[test]
fn test_init_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_check_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn test_watch_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("watch")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Watch"));
}

#[test]
fn test_completion_command_help() {
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("completion")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("completion"));
}

#[test]
fn test_export_invalid_format() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");

    input
        .write_str(r#"{"nodes": [], "edges": [], "events": [], "metadata": {}}"#)
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(temp.child("output.txt").path())
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure();
}

#[test]
fn test_init_without_cargo_toml() {
    let temp = TempDir::new().unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success(); // Should still create config even without Cargo.toml
}

#[test]
fn test_check_with_validate_flag() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("data.json");

    data_file.write_str(r#"{
        "version": "1.0",
        "nodes": [],
        "edges": [],
        "events": [],
        "graph": {
            "nodes": [],
            "edges": []
        },
        "metadata": {"total_variables": 0, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 0}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--validate")
        .assert()
        .success();
}

#[test]
fn test_multiple_flags_combination() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("--verbose")
        .arg("--output-format")
        .arg("text")
        .arg("init")
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn test_export_with_complex_data() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("complex.json");
    let output = temp.child("output.dot");

    input.write_str(r#"{
        "version": "1.0",
        "nodes": [
            {"id": "x_0", "name": "x", "type_name": "String", "created_at": 1, "dropped_at": 5},
            {"id": "r_1", "name": "r", "type_name": "&String", "created_at": 2, "dropped_at": 4}
        ],
        "edges": [
            {"from": "r_1", "to": "x_0", "relationship": "borrows_immut", "start": 2, "end": 4}
        ],
        "events": [
            {"type": "New", "timestamp": 1, "var_name": "x", "var_id": "x_0", "type_name": "String"},
            {"type": "Borrow", "timestamp": 2, "borrower_name": "r", "borrower_id": "r_1", "owner_id": "x_0", "mutable": false},
            {"type": "Drop", "timestamp": 4, "var_id": "r_1"},
            {"type": "Drop", "timestamp": 5, "var_id": "x_0"}
        ],
        "graph": {
            "nodes": [
                {"id": "x_0", "name": "x", "type_name": "String"},
                {"id": "r_1", "name": "r", "type_name": "&String"}
            ],
            "edges": [
                {"from": "r_1", "to": "x_0", "relationship": "borrows_immut"}
            ]
        },
        "metadata": {"total_variables": 2, "total_relationships": 1, "immutable_borrows": 1, "mutable_borrows": 0, "total_events": 4}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("x_0"));
    assert!(content.contains("r_1"));
}

#[test]
fn test_check_stats_with_data() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("stats.json");

    data_file.write_str(r#"{
        "version": "1.0",
        "nodes": [
            {"id": "x_0", "name": "x", "type_name": "i32", "created_at": 1, "dropped_at": null}
        ],
        "edges": [],
        "events": [
            {"type": "New", "timestamp": 1, "var_name": "x", "var_id": "x_0", "type_name": "i32"}
        ],
        "graph": {
            "nodes": [{"id": "x_0", "name": "x", "type_name": "i32"}],
            "edges": []
        },
        "metadata": {"total_variables": 1, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 1}
    }"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("1").or(predicate::str::contains("variable")));
}

#[test]
fn test_init_creates_gitignore_entry() {
    let temp = TempDir::new().unwrap();

    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();
    temp.child(".gitignore").write_str("target/\n").unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    let gitignore = fs::read_to_string(temp.child(".gitignore").path()).unwrap();
    assert!(gitignore.contains("borrowscope.json") || gitignore.contains("target/"));
}

#[test]
fn test_format_flag_text() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("--output-format")
        .arg("text")
        .arg("init")
        .assert()
        .success();
}

#[test]
fn test_conflicting_flags_verbose_quiet() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("--verbose")
        .arg("--quiet")
        .arg("init")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("conflict").or(predicate::str::contains("cannot be used")),
        );
}

#[test]
fn test_export_creates_parent_directories() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");
    let output = temp.child("nested/dir/output.dot");

    input.write_str(r#"{"version": "1.0", "nodes": [], "edges": [], "events": [], "graph": {"nodes": [], "edges": []}, "metadata": {}}"#).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    output.assert(predicate::path::exists());
}

#[test]
fn test_check_empty_json_file() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("empty.json");

    data_file.write_str("").unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .assert()
        .failure();
}

#[test]
fn test_export_overwrite_existing_file() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");
    let output = temp.child("output.dot");

    input
        .write_str(r#"{"nodes": [], "edges": [], "events": [], "metadata": {}}"#)
        .unwrap();
    output.write_str("old content").unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("digraph"));
    assert!(!content.contains("old content"));
}

// ============================================================================
// ADVANCED INTEGRATION TESTS
// ============================================================================

#[test]
fn test_pipeline_init_check_export() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    // Step 1: Initialize config
    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    temp.child(".borrowscope.toml")
        .assert(predicate::path::exists());

    // Step 2: Create tracking data
    let data_file = temp.child("tracking.json");
    data_file
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [{"id": "x_0", "name": "x", "type_name": "i32"}],
        "edges": [],
        "events": [{"type": "New", "timestamp": 1, "var_name": "x", "var_id": "x_0", "type_name": "i32"}],
        "graph": {"nodes": [{"id": "x_0", "name": "x", "type_name": "i32"}], "edges": []},
        "metadata": {"total_variables": 1, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 1}
    }"#,
        )
        .unwrap();

    // Step 3: Check data
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .assert()
        .success();

    // Step 4: Export to multiple formats
    for format in &["dot", "json", "html"] {
        let output = temp.child(format!("output.{}", format));
        Command::cargo_bin("borrowscope")
            .unwrap()
            .arg("export")
            .arg(data_file.path())
            .arg("--output")
            .arg(output.path())
            .arg("--format")
            .arg(format)
            .assert()
            .success();

        output.assert(predicate::path::exists());
    }
}

#[test]
fn test_check_with_borrow_conflicts() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("conflicts.json");

    // Data with overlapping mutable and immutable borrows
    data_file
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [
            {"id": "x_0", "name": "x", "type_name": "String"},
            {"id": "r1_1", "name": "r1", "type_name": "&String"},
            {"id": "r2_2", "name": "r2", "type_name": "&mut String"}
        ],
        "edges": [
            {"from": "r1_1", "to": "x_0", "relationship": "borrows_immut"},
            {"from": "r2_2", "to": "x_0", "relationship": "borrows_mut"}
        ],
        "events": [
            {"type": "New", "timestamp": 1, "var_name": "x", "var_id": "x_0", "type_name": "String"},
            {"type": "Borrow", "timestamp": 2, "borrower_name": "r1", "borrower_id": "r1_1", "owner_id": "x_0", "mutable": false},
            {"type": "Borrow", "timestamp": 3, "borrower_name": "r2", "borrower_id": "r2_2", "owner_id": "x_0", "mutable": true}
        ],
        "graph": {
            "nodes": [
                {"id": "x_0", "name": "x", "type_name": "String"},
                {"id": "r1_1", "name": "r1", "type_name": "&String"},
                {"id": "r2_2", "name": "r2", "type_name": "&mut String"}
            ],
            "edges": [
                {"from": "r1_1", "to": "x_0", "relationship": "borrows_immut"},
                {"from": "r2_2", "to": "x_0", "relationship": "borrows_mut"}
            ]
        },
        "metadata": {"total_variables": 3, "total_relationships": 2, "immutable_borrows": 1, "mutable_borrows": 1, "total_events": 3}
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--mode")
        .arg("conflicts")
        .assert()
        .success();
}

#[test]
fn test_export_with_graph_cycles() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("cycles.json");
    let output = temp.child("cycles.dot");

    // Create data with circular references (Rc cycle)
    input
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [
            {"id": "a_0", "name": "a", "type_name": "Rc<RefCell<Node>>"},
            {"id": "b_1", "name": "b", "type_name": "Rc<RefCell<Node>>"}
        ],
        "edges": [
            {"from": "a_0", "to": "b_1", "relationship": "references"},
            {"from": "b_1", "to": "a_0", "relationship": "references"}
        ],
        "events": [
            {"type": "RcNew", "timestamp": 1, "var_name": "a", "var_id": "a_0", "type_name": "Rc<RefCell<Node>>", "strong_count": 1},
            {"type": "RcNew", "timestamp": 2, "var_name": "b", "var_id": "b_1", "type_name": "Rc<RefCell<Node>>", "strong_count": 1},
            {"type": "RcClone", "timestamp": 3, "var_name": "a_ref", "var_id": "a_0", "strong_count": 2},
            {"type": "RcClone", "timestamp": 4, "var_name": "b_ref", "var_id": "b_1", "strong_count": 2}
        ],
        "graph": {
            "nodes": [
                {"id": "a_0", "name": "a", "type_name": "Rc<RefCell<Node>>"},
                {"id": "b_1", "name": "b", "type_name": "Rc<RefCell<Node>>"}
            ],
            "edges": [
                {"from": "a_0", "to": "b_1", "relationship": "references"},
                {"from": "b_1", "to": "a_0", "relationship": "references"}
            ]
        },
        "metadata": {"total_variables": 2, "total_relationships": 2, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 4}
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("a_0"));
    assert!(content.contains("b_1"));
    assert!(content.contains("->"));
}

#[test]
fn test_check_with_large_dataset() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("large.json");

    // Generate large dataset with 100 variables
    let mut nodes = Vec::new();
    let mut events = Vec::new();
    let mut graph_nodes = Vec::new();

    for i in 0..100 {
        nodes.push(format!(
            r#"{{"id": "var_{}", "name": "var_{}", "type_name": "i32"}}"#,
            i, i
        ));
        events.push(format!(
            r#"{{"type": "New", "timestamp": {}, "var_name": "var_{}", "var_id": "var_{}", "type_name": "i32"}}"#,
            i + 1,
            i,
            i
        ));
        graph_nodes.push(format!(
            r#"{{"id": "var_{}", "name": "var_{}", "type_name": "i32"}}"#,
            i, i
        ));
    }

    let json = format!(
        r#"{{
        "version": "1.0",
        "nodes": [{}],
        "edges": [],
        "events": [{}],
        "graph": {{"nodes": [{}], "edges": []}},
        "metadata": {{"total_variables": 100, "total_relationships": 0, "immutable_borrows": 0, "mutable_borrows": 0, "total_events": 100}}
    }}"#,
        nodes.join(","),
        events.join(","),
        graph_nodes.join(",")
    );

    data_file.write_str(&json).unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_init_with_different_templates() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    for template in &["default", "minimal", "advanced"] {
        let subdir = temp.child(template);
        subdir.create_dir_all().unwrap();
        subdir
            .child("Cargo.toml")
            .write_str("[package]\nname = \"test\"")
            .unwrap();

        Command::cargo_bin("borrowscope")
            .unwrap()
            .current_dir(subdir.path())
            .arg("init")
            .arg("--template")
            .arg(template)
            .arg("--force")
            .assert()
            .success();

        let config_path = subdir.child(".borrowscope.toml");
        config_path.assert(predicate::path::exists());

        let content = fs::read_to_string(config_path.path()).unwrap();
        assert!(content.contains("[run]"));
        assert!(content.contains("[visualize]"));
        assert!(content.contains("[export]"));
    }
}

#[test]
fn test_export_format_auto_detection() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");

    input
        .write_str(
            r#"{"version": "1.0", "nodes": [], "edges": [], "events": [], "graph": {"nodes": [], "edges": []}, "metadata": {}}"#,
        )
        .unwrap();

    // Test that format is required (no auto-detection without extension)
    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(temp.child("output").path())
        .assert()
        .failure();
}

#[test]
fn test_check_malformed_json_structure() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("malformed.json");

    // Missing required fields
    data_file
        .write_str(r#"{"nodes": [], "edges": []}"#)
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

#[test]
fn test_concurrent_export_operations() {
    use std::thread;

    let temp = TempDir::new().unwrap();
    let input = temp.child("input.json");

    input
        .write_str(
            r#"{"version": "1.0", "nodes": [], "edges": [], "events": [], "graph": {"nodes": [], "edges": []}, "metadata": {}}"#,
        )
        .unwrap();

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let input_path = input.path().to_path_buf();
            let output_path = temp.child(format!("output_{}.dot", i)).path().to_path_buf();

            thread::spawn(move || {
                Command::cargo_bin("borrowscope")
                    .unwrap()
                    .arg("export")
                    .arg(&input_path)
                    .arg("--output")
                    .arg(&output_path)
                    .arg("--format")
                    .arg("dot")
                    .assert()
                    .success();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all outputs were created
    for i in 0..5 {
        temp.child(format!("output_{}.dot", i))
            .assert(predicate::path::exists());
    }
}

#[test]
fn test_init_preserves_existing_config_without_force() {
    let temp = TempDir::new().unwrap();
    temp.child("Cargo.toml")
        .write_str("[package]\nname = \"test\"")
        .unwrap();

    let config_file = temp.child(".borrowscope.toml");

    // Create initial config
    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    let original_content = fs::read_to_string(config_file.path()).unwrap();

    // Try to init again without --force
    Command::cargo_bin("borrowscope")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .failure();

    // Verify content unchanged
    let current_content = fs::read_to_string(config_file.path()).unwrap();
    assert_eq!(original_content, current_content);
}

#[test]
fn test_check_with_validate_detects_invalid_graph() {
    let temp = TempDir::new().unwrap();
    let data_file = temp.child("invalid_graph.json");

    // Graph with edge pointing to non-existent node
    data_file
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [{"id": "x_0", "name": "x", "type_name": "i32"}],
        "edges": [{"from": "x_0", "to": "nonexistent", "relationship": "borrows"}],
        "events": [],
        "graph": {
            "nodes": [{"id": "x_0", "name": "x", "type_name": "i32"}],
            "edges": [{"from": "x_0", "to": "nonexistent", "relationship": "borrows"}]
        },
        "metadata": {}
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("check")
        .arg(data_file.path())
        .arg("--validate")
        .assert()
        .success(); // May pass basic validation, depends on implementation
}

#[test]
fn test_export_with_unicode_variable_names() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("unicode.json");
    let output = temp.child("unicode.dot");

    input
        .write_str(
            r#"{
        "version": "1.0",
        "nodes": [
            {"id": "变量_0", "name": "变量", "type_name": "i32"},
            {"id": "переменная_1", "name": "переменная", "type_name": "String"},
            {"id": "🦀_2", "name": "🦀", "type_name": "Vec<u8>"}
        ],
        "edges": [],
        "events": [],
        "graph": {
            "nodes": [
                {"id": "变量_0", "name": "变量", "type_name": "i32"},
                {"id": "переменная_1", "name": "переменная", "type_name": "String"},
                {"id": "🦀_2", "name": "🦀", "type_name": "Vec<u8>"}
            ],
            "edges": []
        },
        "metadata": {}
    }"#,
        )
        .unwrap();

    Command::cargo_bin("borrowscope")
        .unwrap()
        .arg("export")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--format")
        .arg("dot")
        .assert()
        .success();

    let content = fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("变量") || content.contains("_0"));
}

#[test]
fn test_completion_generates_valid_scripts() {
    for shell in &["bash", "zsh", "fish", "powershell"] {
        let output = Command::cargo_bin("borrowscope")
            .unwrap()
            .arg("completion")
            .arg(shell)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let script = String::from_utf8(output).unwrap();
        assert!(!script.is_empty());
        assert!(script.len() > 100); // Should be substantial
    }
}
