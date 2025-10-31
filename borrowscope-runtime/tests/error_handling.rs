#![allow(unused_variables)]

mod common;
use borrowscope_runtime::*;
use common::TestFixture;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_export_json_success() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());

    let result = export.to_json();
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("nodes"));
    assert!(json.contains("events"));
}

#[test]
fn test_export_to_file_success() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_export_success.json");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());

    let result = export.to_file(&file_path);
    assert!(result.is_ok());
    assert!(file_path.exists());

    // Verify file content
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("nodes"));

    // Cleanup
    fs::remove_file(&file_path).ok();
}

#[test]
fn test_export_to_invalid_path() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    // Try to write to invalid path
    let invalid_path = PathBuf::from("/invalid/path/that/does/not/exist/test.json");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());

    let result = export.to_file(&invalid_path);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, Error::IoError(_)));
    }
}

#[test]
fn test_export_empty_data() {
    let fixture = TestFixture::new();

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());

    let result = export.to_json();
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["nodes"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["events"].as_array().unwrap().len(), 0);
}

#[test]
fn test_export_large_dataset() {
    let fixture = TestFixture::new();

    // Create many variables
    for i in 0..100 {
        let x = track_new(&format!("var_{}", i), i);
        assert_eq!(x, i);
        track_drop(&format!("var_{}", i));
    }

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());

    let result = export.to_json();
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.len() > 1000); // Should be substantial
}

#[test]
fn test_error_display() {
    let err = Error::LockError;
    assert_eq!(err.to_string(), "Failed to acquire lock");

    let err = Error::ExportError("test error".to_string());
    assert_eq!(err.to_string(), "Export error: test error");

    let err = Error::InvalidEventSequence("bad sequence".to_string());
    assert_eq!(err.to_string(), "Invalid event sequence: bad sequence");
}

#[test]
fn test_error_from_serde() {
    let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let err: Error = json_err.into();
    assert!(matches!(err, Error::SerializationError(_)));
    assert!(err.to_string().contains("Serialization error"));
}

#[test]
fn test_result_propagation() {
    fn export_and_read(path: &std::path::Path) -> Result<String> {
        let events = get_events();
        let graph = build_graph(&events);
        let export = ExportData::new(graph, events);
        export.to_file(path)?;
        Ok(std::fs::read_to_string(path)?)
    }

    let fixture = TestFixture::new();
    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    let temp_path = std::env::temp_dir().join("test_propagation.json");
    let result = export_and_read(&temp_path);
    assert!(result.is_ok());

    fs::remove_file(&temp_path).ok();
}

#[test]
fn test_graceful_degradation() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);

    // Try to export to invalid path
    let invalid_path = PathBuf::from("/invalid/path/test.json");
    let result = export.to_file(&invalid_path);

    // Should fail gracefully
    assert!(result.is_err());

    // But we can still export to JSON string
    let json_result = export.to_json();
    assert!(json_result.is_ok());
}
