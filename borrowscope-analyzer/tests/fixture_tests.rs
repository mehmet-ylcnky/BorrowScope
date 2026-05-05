//! Integration tests for analyzer fixture projects.
//!
//! Tests semantic method_call tracking and mapped guard type resolution.

use std::path::Path;
use std::process::Command;

fn run_analyzer(fixture: &str) -> serde_json::Value {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let project_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);

    // Clean previous output
    let _ = std::fs::remove_dir_all(project_path.join(".borrowscope"));

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "borrowscope-analyzer",
            "--",
            project_path.to_str().unwrap(),
        ])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run analyzer");

    assert!(
        output.status.success(),
        "Analyzer failed on {}: {}",
        fixture,
        String::from_utf8_lossy(&output.stderr)
    );

    let json_path = project_path.join(".borrowscope/type-info.json");
    let content = std::fs::read_to_string(&json_path).expect("Failed to read type-info.json");
    serde_json::from_str(&content).expect("Failed to parse JSON")
}

fn find_var<'a>(json: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    json["files"]["src/main.rs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == name)
        .unwrap_or_else(|| panic!("Variable '{}' not found in analyzer output", name))
}

// === Semantic project: method_calls with self_borrow ===

#[test]
fn test_split_off_detected_as_mutable() {
    let json = run_analyzer("semantic_project");
    let v2 = find_var(&json, "v2");
    let methods = v2["method_calls"]
        .as_array()
        .expect("v2 should have method_calls");
    let split_off = methods
        .iter()
        .find(|mc| mc["method"] == "split_off")
        .expect("split_off should be in method_calls");
    assert_eq!(split_off["self_borrow"].as_str(), Some("mutable"));
}

#[test]
fn test_retain_detected_as_mutable() {
    let json = run_analyzer("semantic_project");
    let v = find_var(&json, "v");
    let methods = v["method_calls"]
        .as_array()
        .expect("v should have method_calls");
    let retain = methods
        .iter()
        .find(|mc| mc["method"] == "retain")
        .expect("retain should be in method_calls");
    assert_eq!(retain["self_borrow"].as_str(), Some("mutable"));
}

#[test]
fn test_windows_detected_as_immutable() {
    let json = run_analyzer("semantic_project");
    let v = find_var(&json, "v");
    let methods = v["method_calls"]
        .as_array()
        .expect("v should have method_calls");
    let windows = methods
        .iter()
        .find(|mc| mc["method"] == "windows")
        .expect("windows should be in method_calls");
    assert_eq!(windows["self_borrow"].as_str(), Some("immutable"));
}

#[test]
fn test_by_name_index_populated() {
    let json = run_analyzer("semantic_project");
    let by_name = json["by_name"]
        .as_object()
        .expect("by_name should be object");
    assert!(!by_name.is_empty(), "by_name index should not be empty");
    assert!(by_name.contains_key("v"), "by_name should contain 'v'");
    assert!(by_name.contains_key("v2"), "by_name should contain 'v2'");
}

// === Mapped guards project: type resolution ===

#[test]
fn test_mapped_mutex_guard_type_resolved() {
    let json = run_analyzer("mapped_guards_project");
    let mapped = find_var(&json, "mapped");
    let ty = mapped["ty"].as_str().expect("mapped should have ty");
    assert!(
        ty.contains("MappedMutexGuard"),
        "Expected MappedMutexGuard, got: {}",
        ty
    );
}

#[test]
fn test_mapped_rwlock_read_guard_type_resolved() {
    let json = run_analyzer("mapped_guards_project");
    let mapped_read = find_var(&json, "mapped_read");
    let ty = mapped_read["ty"]
        .as_str()
        .expect("mapped_read should have ty");
    assert!(
        ty.contains("MappedRwLockReadGuard"),
        "Expected MappedRwLockReadGuard, got: {}",
        ty
    );
}

#[test]
fn test_mapped_rwlock_write_guard_type_resolved() {
    let json = run_analyzer("mapped_guards_project");
    let mapped_write = find_var(&json, "mapped_write");
    let ty = mapped_write["ty"]
        .as_str()
        .expect("mapped_write should have ty");
    assert!(
        ty.contains("MappedRwLockWriteGuard"),
        "Expected MappedRwLockWriteGuard, got: {}",
        ty
    );
}

#[test]
fn test_mutex_lock_method_tracked() {
    let json = run_analyzer("mapped_guards_project");
    let mutex = find_var(&json, "mutex");
    let methods = mutex["method_calls"]
        .as_array()
        .expect("mutex should have method_calls");
    let lock = methods
        .iter()
        .find(|mc| mc["method"] == "lock")
        .expect("lock should be in method_calls");
    assert!(
        lock["operation"].as_str().unwrap().contains("lock"),
        "lock operation should contain 'lock'"
    );
}

#[test]
fn test_ordering_type_detected() {
    let json = run_analyzer("semantic_project");
    let ord = find_var(&json, "ord");
    assert_eq!(
        ord["is_ordering"].as_bool(),
        Some(true),
        "ord should have is_ordering: true"
    );
    assert_eq!(
        ord["ty"].as_str(),
        Some("Ordering"),
        "ord should have type Ordering"
    );
}
