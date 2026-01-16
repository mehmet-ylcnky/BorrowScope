//! Integration tests for method call tracking
//! 
//! Single test that runs analyzer once and validates all method call tracking.

use std::process::Command;
use std::path::Path;
use std::sync::OnceLock;

static ANALYSIS: OnceLock<serde_json::Value> = OnceLock::new();

fn get_analysis() -> &'static serde_json::Value {
    ANALYSIS.get_or_init(|| {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let full_project_path = workspace_root.join("examples/type-coverage");
        
        let status = Command::new("cargo")
            .args(["run", "-p", "borrowscope-analyzer", "--", full_project_path.to_str().unwrap()])
            .current_dir(workspace_root)
            .output()
            .expect("Failed to run analyzer");
        
        assert!(status.status.success(), "Analyzer failed: {}", String::from_utf8_lossy(&status.stderr));
        
        let json_path = full_project_path.join(".borrowscope/type-info.json");
        let content = std::fs::read_to_string(&json_path).expect("Failed to read output");
        serde_json::from_str(&content).expect("Failed to parse JSON")
    })
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
fn test_all_method_call_tracking() {
    let json = get_analysis();
    
    // === Cell methods ===
    let cell = find_var(json, "cell", Some("test_method_calls_cell"))
        .expect("Cell variable not found");
    assert!(get_methods(cell).contains(&"set"), "Cell.set not found");
    assert!(get_methods(cell).contains(&"get"), "Cell.get not found");
    assert!(get_ops(cell).contains(&"core::cell::set"), "Cell set op not found");
    assert!(get_ops(cell).contains(&"core::cell::get"), "Cell get op not found");
    
    // === Cow methods ===
    let cow = find_var(json, "cow", Some("test_method_calls_cow"))
        .expect("Cow variable not found");
    assert!(get_methods(cow).contains(&"to_mut"), "Cow.to_mut not found");
    assert!(get_ops(cow).contains(&"alloc::borrow::to_mut"), "Cow to_mut op not found");
    
    // === OnceCell methods ===
    let once_cell = find_var(json, "cell", Some("test_method_calls_once_cell"))
        .expect("OnceCell variable not found");
    assert!(get_methods(once_cell).contains(&"set"), "OnceCell.set not found");
    assert!(get_methods(once_cell).contains(&"get_or_init"), "OnceCell.get_or_init not found");
    assert!(get_ops(once_cell).contains(&"core::cell::once::set"), "OnceCell set op not found");
    assert!(get_ops(once_cell).contains(&"core::cell::once::get_or_init"), "OnceCell get_or_init op not found");
    
    // === Channel methods ===
    let tuple = find_var(json, "(tx, rx)", Some("test_method_calls_channels"))
        .expect("Channel tuple not found");
    assert!(get_methods(tuple).contains(&"send"), "Channel.send not found");
    assert!(get_methods(tuple).contains(&"recv"), "Channel.recv not found");
    assert!(get_ops(tuple).contains(&"std::sync::mpsc::send"), "Channel send op not found");
    assert!(get_ops(tuple).contains(&"std::sync::mpsc::recv"), "Channel recv op not found");
    
    // === JoinHandle methods ===
    let handle = find_var(json, "handle", Some("test_method_calls_thread_join"))
        .expect("JoinHandle variable not found");
    assert!(get_methods(handle).contains(&"join"), "JoinHandle.join not found");
    assert!(get_ops(handle).contains(&"std::thread::join"), "JoinHandle join op not found");
    
    // === Self borrow types ===
    // Cell.set() takes &self (immutable)
    let set_call = cell["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "set")).unwrap();
    assert_eq!(set_call["self_borrow"], "immutable", "Cell.set should be immutable");
    
    // Cow.to_mut() takes &mut self (mutable)
    let to_mut = cow["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "to_mut")).unwrap();
    assert_eq!(to_mut["self_borrow"], "mutable", "Cow.to_mut should be mutable");
    
    // JoinHandle.join() takes self (consuming)
    let join = handle["method_calls"].as_array()
        .and_then(|arr| arr.iter().find(|m| m["method"] == "join")).unwrap();
    assert_eq!(join["self_borrow"], "consuming", "JoinHandle.join should be consuming");
    
    // === Standalone expressions ===
    let exprs = json["expressions"]["src/main.rs"].as_array().expect("No expressions");
    assert!(exprs.iter().any(|e| e["operation"] == "core::mem::drop"), "drop not found");
    assert!(exprs.iter().any(|e| e["operation"] == "core::mem::forget"), "forget not found");
    assert!(exprs.iter().any(|e| e["operation"] == "std::thread::spawn"), "spawn not found");
    assert!(exprs.iter().any(|e| e["operation"] == "core::intrinsics::transmute"), "transmute not found");
    assert!(exprs.iter().any(|e| e["operation"] == "core::ptr::read"), "ptr::read not found");
    assert!(exprs.iter().any(|e| e["operation"] == "core::ptr::write"), "ptr::write not found");
    
    // === No null values ===
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
    
    // === Chained calls not attributed ===
    let mutex = find_var(json, "mutex", Some("test_method_calls_mutex_rwlock"))
        .expect("Mutex not found");
    let methods = get_methods(mutex);
    assert!(methods.contains(&"lock"), "Mutex.lock not found");
    assert!(!methods.contains(&"unwrap"), "unwrap should not be on mutex");
    
    // === MaybeUninit methods ===
    let mu = find_var(json, "mu", Some("test_advanced_types")).expect("mu not found");
    assert!(get_ops(mu).contains(&"core::mem::maybe_uninit::write"), "MaybeUninit.write not found");
    assert!(get_ops(mu).contains(&"core::mem::maybe_uninit::assume_init"), "MaybeUninit.assume_init not found");
    
    let mu2 = find_var(json, "mu2", Some("test_advanced_types")).expect("mu2 not found");
    assert!(get_ops(mu2).contains(&"core::mem::maybe_uninit::assume_init_read"), "MaybeUninit.assume_init_read not found");
    
    let mu3 = find_var(json, "mu3", Some("test_advanced_types")).expect("mu3 not found");
    assert!(get_ops(mu3).contains(&"core::mem::maybe_uninit::assume_init_drop"), "MaybeUninit.assume_init_drop not found");
}
