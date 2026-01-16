//! Integration tests for method call tracking
//! 
//! These tests verify the semantic method call tracking implementation.
//! All operations use fully semantic paths from rust-analyzer (no heuristics).

use std::process::Command;
use std::path::Path;

fn run_analyzer(project_path: &str) -> serde_json::Value {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let full_project_path = workspace_root.join(project_path);
    
    let status = Command::new("cargo")
        .args(["run", "-p", "borrowscope-analyzer", "--", full_project_path.to_str().unwrap()])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run analyzer");
    
    assert!(status.status.success(), "Analyzer failed: {}", String::from_utf8_lossy(&status.stderr));
    
    let json_path = full_project_path.join(".borrowscope/type-info.json");
    let content = std::fs::read_to_string(&json_path).expect("Failed to read output");
    serde_json::from_str(&content).expect("Failed to parse JSON")
}

fn find_var<'a>(json: &'a serde_json::Value, name: &str, func: Option<&str>) -> Option<&'a serde_json::Value> {
    json["files"]["src/main.rs"].as_array()?.iter().find(|v| {
        v["name"] == name && func.map(|f| v["function_name"].as_str() == Some(f)).unwrap_or(true)
    })
}

fn get_methods(var: &serde_json::Value) -> Vec<&str> {
    var["method_calls"].as_array()
        .map(|arr| arr.iter().filter_map(|m| m["method"].as_str()).collect())
        .unwrap_or_default()
}

fn get_ops(var: &serde_json::Value) -> Vec<&str> {
    var["method_calls"].as_array()
        .map(|arr| arr.iter().filter_map(|m| m["operation"].as_str()).collect())
        .unwrap_or_default()
}

#[test]
fn test_cell_methods() {
    let json = run_analyzer("examples/type-coverage");
    let cell = find_var(&json, "cell", Some("test_method_calls_cell"))
        .expect("Cell variable not found");
    
    assert!(get_methods(cell).contains(&"set"));
    assert!(get_methods(cell).contains(&"get"));
    assert!(get_ops(cell).contains(&"core::cell::set"));
    assert!(get_ops(cell).contains(&"core::cell::get"));
}

#[test]
fn test_cow_methods() {
    let json = run_analyzer("examples/type-coverage");
    let cow = find_var(&json, "cow", Some("test_method_calls_cow"))
        .expect("Cow variable not found");
    
    assert!(get_methods(cow).contains(&"to_mut"));
    assert!(get_ops(cow).contains(&"alloc::borrow::to_mut"));
}

#[test]
fn test_once_cell_methods() {
    let json = run_analyzer("examples/type-coverage");
    let cell = find_var(&json, "cell", Some("test_method_calls_once_cell"))
        .expect("OnceCell variable not found");
    
    assert!(get_methods(cell).contains(&"set"));
    assert!(get_methods(cell).contains(&"get_or_init"));
    assert!(get_ops(cell).contains(&"core::cell::once::set"));
    assert!(get_ops(cell).contains(&"core::cell::once::get_or_init"));
}

#[test]
fn test_channel_methods() {
    let json = run_analyzer("examples/type-coverage");
    let tuple = find_var(&json, "(tx, rx)", Some("test_method_calls_channels"))
        .expect("Channel tuple not found");
    
    assert!(get_methods(tuple).contains(&"send"));
    assert!(get_methods(tuple).contains(&"recv"));
    assert!(get_ops(tuple).contains(&"std::sync::mpsc::send"));
    assert!(get_ops(tuple).contains(&"std::sync::mpsc::recv"));
}

#[test]
fn test_join_handle_methods() {
    let json = run_analyzer("examples/type-coverage");
    let handle = find_var(&json, "handle", Some("test_method_calls_thread_join"))
        .expect("JoinHandle variable not found");
    
    assert!(get_methods(handle).contains(&"join"));
    assert!(get_ops(handle).contains(&"std::thread::join"));
}

#[test]
fn test_self_borrow_types() {
    let json = run_analyzer("examples/type-coverage");
    
    // Cell.set() takes &self (immutable)
    let cell = find_var(&json, "cell", Some("test_method_calls_cell")).unwrap();
    let set_call = cell["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "set")).unwrap();
    assert_eq!(set_call["self_borrow"], "immutable");
    
    // Cow.to_mut() takes &mut self (mutable)
    let cow = find_var(&json, "cow", Some("test_method_calls_cow")).unwrap();
    let to_mut = cow["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "to_mut")).unwrap();
    assert_eq!(to_mut["self_borrow"], "mutable");
    
    // JoinHandle.join() takes self (consuming)
    let handle = find_var(&json, "handle", Some("test_method_calls_thread_join")).unwrap();
    let join = handle["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "join")).unwrap();
    assert_eq!(join["self_borrow"], "consuming");
}

#[test]
fn test_standalone_expressions() {
    let json = run_analyzer("examples/type-coverage");
    let exprs = json["expressions"]["src/main.rs"].as_array().expect("No expressions");
    
    assert!(exprs.iter().any(|e| e["operation"] == "core::mem::drop"));
    assert!(exprs.iter().any(|e| e["operation"] == "core::mem::forget"));
    assert!(exprs.iter().any(|e| e["operation"] == "std::thread::spawn"));
    assert!(exprs.iter().any(|e| e["operation"] == "core::intrinsics::transmute"));
    assert!(exprs.iter().any(|e| e["operation"] == "core::ptr::read"));
    assert!(exprs.iter().any(|e| e["operation"] == "core::ptr::write"));
}

#[test]
fn test_no_null_values() {
    let json = run_analyzer("examples/type-coverage");
    
    for var in json["files"]["src/main.rs"].as_array().unwrap() {
        if let Some(calls) = var["method_calls"].as_array() {
            for call in calls {
                assert!(call["operation"].as_str().is_some(), 
                    "Null operation: {} {}", var["name"], call["method"]);
                assert!(call["self_borrow"].as_str().is_some(),
                    "Null self_borrow: {} {}", var["name"], call["method"]);
            }
        }
    }
}

#[test]
fn test_chained_calls_not_attributed() {
    let json = run_analyzer("examples/type-coverage");
    
    // mutex.lock().unwrap() - only lock should be on mutex
    let mutex = find_var(&json, "mutex", Some("test_method_calls_mutex_rwlock"))
        .expect("Mutex not found");
    
    let methods = get_methods(mutex);
    assert!(methods.contains(&"lock"));
    assert!(!methods.contains(&"unwrap"), "unwrap should not be on mutex");
}

#[test]
fn test_maybe_uninit_methods() {
    let json = run_analyzer("examples/type-coverage");
    
    let mu = find_var(&json, "mu", Some("test_advanced_types")).expect("mu not found");
    assert!(get_ops(mu).contains(&"core::mem::maybe_uninit::write"));
    assert!(get_ops(mu).contains(&"core::mem::maybe_uninit::assume_init"));
    
    let mu2 = find_var(&json, "mu2", Some("test_advanced_types")).expect("mu2 not found");
    assert!(get_ops(mu2).contains(&"core::mem::maybe_uninit::assume_init_read"));
    
    let mu3 = find_var(&json, "mu3", Some("test_advanced_types")).expect("mu3 not found");
    assert!(get_ops(mu3).contains(&"core::mem::maybe_uninit::assume_init_drop"));
}
