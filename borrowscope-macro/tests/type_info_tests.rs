//! Tests for type_info.rs deserialization of method_calls and expressions

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MethodCallInfo {
    pub method: String,
    pub line: u32,
    pub column: u32,
    pub operation: Option<String>,
    pub self_borrow: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExpressionInfo {
    pub line: u32,
    pub column: u32,
    pub path: Option<String>,
    pub operation: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct VariableTypeInfo {
    pub name: String,
    pub ty: String,
    #[serde(default)]
    pub method_calls: Vec<MethodCallInfo>,
}

#[test]
fn test_deserialize_method_calls() {
    let json = r#"{
        "name": "test_var",
        "ty": "Vec<i32>",
        "method_calls": [
            {
                "method": "push",
                "line": 10,
                "column": 5,
                "operation": "alloc::vec::push",
                "self_borrow": "mutable"
            },
            {
                "method": "len",
                "line": 11,
                "column": 5,
                "operation": "alloc::vec::len",
                "self_borrow": "immutable"
            }
        ]
    }"#;

    let var: VariableTypeInfo = serde_json::from_str(json).unwrap();
    assert_eq!(var.name, "test_var");
    assert_eq!(var.method_calls.len(), 2);

    let push_call = &var.method_calls[0];
    assert_eq!(push_call.method, "push");
    assert_eq!(push_call.line, 10);
    assert_eq!(push_call.column, 5);
    assert_eq!(push_call.operation, Some("alloc::vec::push".to_string()));
    assert_eq!(push_call.self_borrow, Some("mutable".to_string()));

    let len_call = &var.method_calls[1];
    assert_eq!(len_call.method, "len");
    assert_eq!(len_call.self_borrow, Some("immutable".to_string()));
}

#[test]
fn test_deserialize_empty_method_calls() {
    let json = r#"{
        "name": "test_var",
        "ty": "i32"
    }"#;

    let var: VariableTypeInfo = serde_json::from_str(json).unwrap();
    assert_eq!(var.name, "test_var");
    assert_eq!(var.method_calls.len(), 0);
}

#[test]
fn test_deserialize_expression_info() {
    let json = r#"{
        "line": 42,
        "column": 10,
        "path": "std::thread::functions::spawn",
        "operation": "std::thread::functions::spawn"
    }"#;

    let expr: ExpressionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(expr.line, 42);
    assert_eq!(expr.column, 10);
    assert_eq!(expr.path, Some("std::thread::functions::spawn".to_string()));
    assert_eq!(expr.operation, "std::thread::functions::spawn");
}

#[test]
fn test_deserialize_method_call_info() {
    let json = r#"{
        "method": "lock",
        "line": 100,
        "column": 20,
        "operation": "std::sync::Mutex::lock",
        "self_borrow": "immutable"
    }"#;

    let mc: MethodCallInfo = serde_json::from_str(json).unwrap();
    assert_eq!(mc.method, "lock");
    assert_eq!(mc.line, 100);
    assert_eq!(mc.column, 20);
    assert_eq!(mc.operation, Some("std::sync::Mutex::lock".to_string()));
    assert_eq!(mc.self_borrow, Some("immutable".to_string()));
}

#[test]
fn test_method_call_with_none_fields() {
    let json = r#"{
        "method": "unknown",
        "line": 50,
        "column": 15,
        "operation": null,
        "self_borrow": null
    }"#;

    let mc: MethodCallInfo = serde_json::from_str(json).unwrap();
    assert_eq!(mc.method, "unknown");
    assert_eq!(mc.operation, None);
    assert_eq!(mc.self_borrow, None);
}
