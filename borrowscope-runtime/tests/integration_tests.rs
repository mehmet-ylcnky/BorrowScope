mod common;
use borrowscope_runtime::*;
use common::TestFixture;
use serde_json::Value;

// ===== Lifecycle Tests =====

#[test]
fn test_variable_creation_and_drop() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    fixture.assert_event_types(&["New", "Drop"]);
}

#[test]
fn test_multiple_variables() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let y = track_new("y", 100);
    assert_eq!(x + y, 142);

    track_drop("y");
    track_drop("x");

    assert_eq!(fixture.event_count(), 4);
    fixture.assert_event_types(&["New", "New", "Drop", "Drop"]);
}

// ===== Borrowing Tests =====

#[test]
fn test_immutable_borrow() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let r = track_borrow("r", &x);
    assert_eq!(*r, 42);

    track_drop("r");
    track_drop("x");

    fixture.assert_event_types(&["New", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_mutable_borrow() {
    let fixture = TestFixture::new();

    let mut x = track_new("x", 42);
    let r = track_borrow_mut("r", &mut x);
    *r = 100;
    assert_eq!(*r, 100);

    track_drop("r");
    track_drop("x");

    fixture.assert_event_types(&["New", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_multiple_immutable_borrows() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let r1 = track_borrow("r1", &x);
    let r2 = track_borrow("r2", &x);
    assert_eq!(*r1 + *r2, 84);

    track_drop("r2");
    track_drop("r1");
    track_drop("x");

    fixture.assert_event_types(&["New", "Borrow", "Borrow", "Drop", "Drop", "Drop"]);
}

// ===== Move Tests =====

#[test]
fn test_simple_move() {
    let fixture = TestFixture::new();

    let x = track_new("x", String::from("hello"));
    let y = track_move("x", "y", x);
    assert_eq!(y, "hello");

    track_drop("y");

    fixture.assert_event_types(&["New", "Drop"]);
}

// ===== Graph Building Tests =====

#[test]
fn test_graph_from_events() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert_eq!(x, 42);
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);

    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].name, "x");
}

#[test]
fn test_graph_multiple_variables() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let y = track_new("y", 100);
    let z = track_new("z", String::from("test"));
    assert!(x < y);
    assert_eq!(z, "test");

    track_drop("z");
    track_drop("y");
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);

    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.nodes.iter().any(|v| v.name == "x"));
    assert!(graph.nodes.iter().any(|v| v.name == "y"));
    assert!(graph.nodes.iter().any(|v| v.name == "z"));
}

#[test]
fn test_graph_statistics() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let y = track_new("y", 100);
    assert_ne!(x, y);

    track_drop("y");
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);
    let stats = graph.stats();

    assert_eq!(stats.total_variables, 2);
}

// ===== JSON Export Tests =====

#[test]
fn test_json_structure() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    assert!(x > 0);
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());
    let json = export.to_json().unwrap();

    let data: Value = serde_json::from_str(&json).unwrap();

    assert!(data["nodes"].is_array());
    assert!(data["edges"].is_array());
    assert!(data["events"].is_array());
    assert!(data["metadata"].is_object());
}

#[test]
fn test_json_content() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let y = track_new("y", 100);
    assert!(x < y);

    track_drop("y");
    track_drop("x");

    let events = fixture.events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events.clone());
    let json = export.to_json().unwrap();

    let data: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(data["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(data["events"].as_array().unwrap().len(), 4);
}

// ===== Real World Scenarios =====

#[test]
fn test_vector_operations() {
    let fixture = TestFixture::new();

    let mut v = track_new("v", vec![1, 2, 3]);
    let r = track_borrow("r", &v);
    assert_eq!(r.len(), 3);
    track_drop("r");

    let r_mut = track_borrow_mut("r_mut", &mut v);
    r_mut.push(4);
    assert_eq!(r_mut.len(), 4);
    track_drop("r_mut");

    track_drop("v");

    fixture.assert_event_types(&["New", "Borrow", "Drop", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_string_operations() {
    let fixture = TestFixture::new();

    let mut s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    assert_eq!(r.len(), 5);
    track_drop("r");

    let r_mut = track_borrow_mut("r_mut", &mut s);
    r_mut.push_str(" world");
    assert_eq!(r_mut, "hello world");
    track_drop("r_mut");

    track_drop("s");

    fixture.assert_event_types(&["New", "Borrow", "Drop", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_nested_scopes() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);

    {
        let r1 = track_borrow("r1", &x);
        assert_eq!(*r1, 42);
        track_drop("r1");
    }

    {
        let r2 = track_borrow("r2", &x);
        assert_eq!(*r2, 42);
        track_drop("r2");
    }

    track_drop("x");

    fixture.assert_event_types(&["New", "Borrow", "Drop", "Borrow", "Drop", "Drop"]);
}

#[test]
fn test_complex_workflow() {
    let fixture = TestFixture::new();

    let x = track_new("x", 42);
    let y = track_new("y", 100);
    let mut v = track_new("v", vec![1, 2, 3]);

    let r_x = track_borrow("r_x", &x);
    assert_eq!(*r_x, 42);
    track_drop("r_x");

    let r_v = track_borrow_mut("r_v", &mut v);
    r_v.push(4);
    assert_eq!(r_v.len(), 4);
    track_drop("r_v");

    let z = track_move("y", "z", y);
    assert_eq!(z, 100);

    track_drop("z");
    track_drop("v");
    track_drop("x");

    assert!(fixture.event_count() >= 10);
    fixture.assert_has_event_type("New");
    fixture.assert_has_event_type("Borrow");
    fixture.assert_has_event_type("Drop");
}
