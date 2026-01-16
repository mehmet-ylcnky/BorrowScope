//! Integration tests for method call tracking

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

fn get_variable<'a>(json: &'a serde_json::Value, file: &str, name: &str) -> Option<&'a serde_json::Value> {
    json["files"][file].as_array()?.iter().find(|v| v["name"] == name)
}

fn get_method_calls(var: &serde_json::Value) -> Vec<&str> {
    var["method_calls"].as_array()
        .map(|arr| arr.iter().filter_map(|m| m["method"].as_str()).collect())
        .unwrap_or_default()
}

fn get_operations(var: &serde_json::Value) -> Vec<&str> {
    var["method_calls"].as_array()
        .map(|arr| arr.iter().filter_map(|m| m["operation"].as_str()).collect())
        .unwrap_or_default()
}

#[test]
fn test_cell_method_tracking() {
    let json = run_analyzer("examples/method-call-test");
    let cell = get_variable(&json, "src/main.rs", "cell")
        .expect("Cell variable not found");
    
    // Cell<i32> should have set and get tracked
    if cell["ty"].as_str() == Some("Cell<i32>") {
        let methods = get_method_calls(cell);
        assert!(methods.contains(&"set"), "Cell.set() not tracked");
        assert!(methods.contains(&"get"), "Cell.get() not tracked");
        
        let ops = get_operations(cell);
        assert!(ops.contains(&"cell_set"), "cell_set operation not classified");
        assert!(ops.contains(&"cell_get"), "cell_get operation not classified");
    }
}

#[test]
fn test_cow_method_tracking() {
    let json = run_analyzer("examples/method-call-test");
    let cow = get_variable(&json, "src/main.rs", "cow")
        .expect("Cow variable not found");
    
    let methods = get_method_calls(cow);
    assert!(methods.contains(&"to_mut"), "Cow.to_mut() not tracked");
    
    let ops = get_operations(cow);
    assert!(ops.contains(&"cow_to_mut"), "cow_to_mut operation not classified");
}

#[test]
fn test_once_cell_method_tracking() {
    let json = run_analyzer("examples/method-call-test");
    
    // Find OnceCell variable (there are two "cell" variables)
    let once_cell = json["files"]["src/main.rs"].as_array()
        .and_then(|arr| arr.iter().find(|v| v["ty"].as_str().map(|t| t.starts_with("OnceCell")).unwrap_or(false)))
        .expect("OnceCell variable not found");
    
    let methods = get_method_calls(once_cell);
    assert!(methods.contains(&"set"), "OnceCell.set() not tracked");
    assert!(methods.contains(&"get"), "OnceCell.get() not tracked");
    assert!(methods.contains(&"get_or_init"), "OnceCell.get_or_init() not tracked");
    
    let ops = get_operations(once_cell);
    assert!(ops.contains(&"once_cell_set"));
    assert!(ops.contains(&"once_cell_get"));
    assert!(ops.contains(&"once_cell_get_or_init"));
}

#[test]
fn test_channel_method_tracking() {
    let json = run_analyzer("examples/method-call-test");
    let tuple = get_variable(&json, "src/main.rs", "(tx, rx)")
        .expect("Channel tuple not found");
    
    let methods = get_method_calls(tuple);
    assert!(methods.contains(&"send"), "Sender.send() not tracked");
    assert!(methods.contains(&"recv"), "Receiver.recv() not tracked");
    assert!(methods.contains(&"try_recv"), "Receiver.try_recv() not tracked");
    
    let ops = get_operations(tuple);
    assert!(ops.contains(&"channel_send"));
    assert!(ops.contains(&"channel_recv"));
    assert!(ops.contains(&"channel_try_recv"));
}

#[test]
fn test_join_handle_method_tracking() {
    let json = run_analyzer("examples/method-call-test");
    let handle = get_variable(&json, "src/main.rs", "handle")
        .expect("JoinHandle variable not found");
    
    let methods = get_method_calls(handle);
    assert!(methods.contains(&"join"), "JoinHandle.join() not tracked");
    
    let ops = get_operations(handle);
    assert!(ops.contains(&"thread_join"));
}

#[test]
fn test_self_borrow_detection() {
    let json = run_analyzer("examples/method-call-test");
    
    // JoinHandle.join() takes self (consuming)
    let handle = get_variable(&json, "src/main.rs", "handle").unwrap();
    let join_call = handle["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "join"))
        .expect("join method not found");
    assert_eq!(join_call["self_borrow"], "consuming", "join() should be consuming");
    
    // Cell.set() takes &self (immutable)
    let cell = json["files"]["src/main.rs"].as_array()
        .and_then(|arr| arr.iter().find(|v| v["ty"].as_str() == Some("Cell<i32>")))
        .unwrap();
    let set_call = cell["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "set"))
        .expect("set method not found");
    assert_eq!(set_call["self_borrow"], "immutable", "Cell.set() should be immutable");
    
    // Cow.to_mut() takes &mut self (mutable)
    let cow = get_variable(&json, "src/main.rs", "cow").unwrap();
    let to_mut_call = cow["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "to_mut"))
        .expect("to_mut method not found");
    assert_eq!(to_mut_call["self_borrow"], "mutable", "Cow.to_mut() should be mutable");
}
