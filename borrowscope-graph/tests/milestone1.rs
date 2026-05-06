//! Comprehensive tests for Milestone 1: Core Data Structures and Graph Construction.

use borrowscope_graph::builder::{from_events, GraphStream, GraphUpdate};
use borrowscope_graph::*;
use borrowscope_runtime::Event;

// ═══════════════════════════════════════════════════════════════════════════
// Node tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_id_copy_and_eq() {
    let id = NodeId(42);
    let id2 = id; // Copy
    assert_eq!(id, id2);
    assert_ne!(NodeId(1), NodeId(2));
}

#[test]
fn test_variable_node_is_alive_at() {
    let node = Node::Variable(VariableNode {
        id: NodeId(0),
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 10,
        dropped_at: Some(50),
        scope_depth: 0,
        is_copy: true,
        is_mutable: false,
    });

    assert!(!node.is_alive_at(9)); // before creation
    assert!(node.is_alive_at(10)); // at creation
    assert!(node.is_alive_at(30)); // during lifetime
    assert!(node.is_alive_at(49)); // just before drop
    assert!(!node.is_alive_at(50)); // at drop
    assert!(!node.is_alive_at(100)); // after drop
}

#[test]
fn test_variable_node_never_dropped() {
    let node = Node::Variable(VariableNode {
        id: NodeId(0),
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 10,
        dropped_at: None,
        scope_depth: 0,
        is_copy: false,
        is_mutable: false,
    });

    assert!(node.is_alive_at(10));
    assert!(node.is_alive_at(u64::MAX - 1));
    assert_eq!(node.end_time(), None);
}

#[test]
fn test_scope_node_is_alive_at() {
    let node = Node::Scope(ScopeNode {
        id: NodeId(0),
        name: "main".into(),
        kind: ScopeKind::Function,
        entered_at: 0,
        exited_at: Some(100),
    });

    assert!(node.is_alive_at(0));
    assert!(node.is_alive_at(50));
    assert!(!node.is_alive_at(100));
}

#[test]
fn test_node_accessors() {
    let node = Node::Variable(VariableNode {
        id: NodeId(5),
        name: "data".into(),
        type_name: "Vec<i32>".into(),
        created_at: 20,
        dropped_at: Some(80),
        scope_depth: 1,
        is_copy: false,
        is_mutable: true,
    });

    assert_eq!(node.id(), NodeId(5));
    assert_eq!(node.name(), "data");
    assert_eq!(node.start_time(), 20);
    assert_eq!(node.end_time(), Some(80));
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_id_copy_and_eq() {
    let id = EdgeId(7);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn test_edge_is_active_at() {
    let edge = Edge {
        id: EdgeId(0),
        source: NodeId(1),
        target: NodeId(0),
        kind: EdgeKind::BorrowShared,
        created_at: 10,
        ended_at: Some(50),
    };

    assert!(!edge.is_active_at(9));
    assert!(edge.is_active_at(10));
    assert!(edge.is_active_at(30));
    assert!(!edge.is_active_at(50));
}

#[test]
fn test_edge_open_ended() {
    let edge = Edge {
        id: EdgeId(0),
        source: NodeId(1),
        target: NodeId(0),
        kind: EdgeKind::BorrowMut,
        created_at: 10,
        ended_at: None,
    };

    assert!(edge.is_active_at(10));
    assert!(edge.is_active_at(u64::MAX - 1));
    assert_eq!(edge.duration(), None);
}

#[test]
fn test_edge_is_borrow() {
    assert!(Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::BorrowShared,
        created_at: 0,
        ended_at: None
    }
    .is_borrow());
    assert!(Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::BorrowMut,
        created_at: 0,
        ended_at: None
    }
    .is_borrow());
    assert!(!Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::Move,
        created_at: 0,
        ended_at: None
    }
    .is_borrow());
}

#[test]
fn test_edge_is_mutable() {
    assert!(!Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::BorrowShared,
        created_at: 0,
        ended_at: None
    }
    .is_mutable());
    assert!(Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::BorrowMut,
        created_at: 0,
        ended_at: None
    }
    .is_mutable());
    assert!(Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::RefCellBorrow { mutable: true },
        created_at: 0,
        ended_at: None
    }
    .is_mutable());
    assert!(!Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::RefCellBorrow { mutable: false },
        created_at: 0,
        ended_at: None
    }
    .is_mutable());
}

#[test]
fn test_edge_duration() {
    let edge = Edge {
        id: EdgeId(0),
        source: NodeId(0),
        target: NodeId(1),
        kind: EdgeKind::BorrowShared,
        created_at: 10,
        ended_at: Some(45),
    };
    assert_eq!(edge.duration(), Some(35));
}

// ═══════════════════════════════════════════════════════════════════════════
// OwnershipGraph builder API tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_graph() {
    let graph = OwnershipGraph::new();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_add_variable() {
    let mut graph = OwnershipGraph::new();
    let id = graph.add_variable("x", "i32", 10);

    assert_eq!(graph.node_count(), 1);
    assert_eq!(id, NodeId(0));

    let node = graph.get_node(id).unwrap();
    assert_eq!(node.name(), "x");
    assert_eq!(node.start_time(), 10);
}

#[test]
fn test_add_multiple_variables() {
    let mut graph = OwnershipGraph::new();
    let a = graph.add_variable("a", "i32", 0);
    let b = graph.add_variable("b", "String", 10);
    let c = graph.add_variable("c", "Vec<i32>", 20);

    assert_eq!(graph.node_count(), 3);
    assert_ne!(a, b);
    assert_ne!(b, c);
}

#[test]
fn test_add_scope() {
    let mut graph = OwnershipGraph::new();
    let id = graph.add_scope("main", ScopeKind::Function, 0);

    assert_eq!(graph.node_count(), 1);
    let node = graph.get_node(id).unwrap();
    assert_eq!(node.name(), "main");
}

#[test]
fn test_find_by_name() {
    let mut graph = OwnershipGraph::new();
    graph.add_variable("x", "i32", 0);
    graph.add_variable("y", "i32", 10);
    graph.add_variable("x", "String", 20); // shadowed

    assert_eq!(graph.find_by_name("x").len(), 2);
    assert_eq!(graph.find_by_name("y").len(), 1);
    assert_eq!(graph.find_by_name("z").len(), 0);
}

#[test]
fn test_mark_dropped() {
    let mut graph = OwnershipGraph::new();
    let id = graph.add_variable("x", "i32", 10);

    assert!(graph.get_node(id).unwrap().is_alive_at(50));
    graph.mark_dropped(id, 50);
    assert!(!graph.get_node(id).unwrap().is_alive_at(50));
    assert_eq!(graph.get_node(id).unwrap().end_time(), Some(50));
}

#[test]
fn test_add_borrow_edge() {
    let mut graph = OwnershipGraph::new();
    let owner = graph.add_variable("x", "Vec<i32>", 0);
    let borrower = graph.add_variable("r", "&Vec<i32>", 10);
    let eid = graph.add_borrow(borrower, owner, false, 10);

    assert_eq!(graph.edge_count(), 1);
    let edge = graph.get_edge(eid).unwrap();
    assert_eq!(edge.source, borrower);
    assert_eq!(edge.target, owner);
    assert!(edge.is_borrow());
    assert!(!edge.is_mutable());
}

#[test]
fn test_add_mutable_borrow() {
    let mut graph = OwnershipGraph::new();
    let owner = graph.add_variable("x", "Vec<i32>", 0);
    let borrower = graph.add_variable("m", "&mut Vec<i32>", 10);
    let eid = graph.add_borrow(borrower, owner, true, 10);

    let edge = graph.get_edge(eid).unwrap();
    assert!(edge.is_borrow());
    assert!(edge.is_mutable());
}

#[test]
fn test_add_move_edge() {
    let mut graph = OwnershipGraph::new();
    let a = graph.add_variable("a", "String", 0);
    let b = graph.add_variable("b", "String", 10);
    let eid = graph.add_move(a, b, 10);

    let edge = graph.get_edge(eid).unwrap();
    assert_eq!(edge.kind, EdgeKind::Move);
    assert!(!edge.is_borrow());
}

#[test]
fn test_add_rc_clone_edge() {
    let mut graph = OwnershipGraph::new();
    let rc1 = graph.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = graph.add_variable("rc2", "Rc<i32>", 10);
    let eid = graph.add_rc_clone(rc2, rc1, 2, 10);

    let edge = graph.get_edge(eid).unwrap();
    assert_eq!(edge.kind, EdgeKind::RcClone { strong_count: 2 });
}

#[test]
fn test_end_edge() {
    let mut graph = OwnershipGraph::new();
    let owner = graph.add_variable("x", "i32", 0);
    let borrower = graph.add_variable("r", "&i32", 10);
    let eid = graph.add_borrow(borrower, owner, false, 10);

    assert!(graph.get_edge(eid).unwrap().is_active_at(30));
    graph.end_edge(eid, 50);
    assert!(!graph.get_edge(eid).unwrap().is_active_at(50));
    assert_eq!(graph.get_edge(eid).unwrap().duration(), Some(40));
}

#[test]
fn test_neighbors() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r1 = graph.add_variable("r1", "&i32", 10);
    let r2 = graph.add_variable("r2", "&i32", 20);
    graph.add_borrow(r1, x, false, 10);
    graph.add_borrow(r2, x, false, 20);

    // r1 and r2 point to x
    assert_eq!(graph.neighbors(r1), vec![x]);
    assert_eq!(graph.neighbors(r2), vec![x]);
    assert_eq!(graph.neighbors(x), vec![]); // x has no outgoing edges
}

#[test]
fn test_borrowers_of() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r1 = graph.add_variable("r1", "&i32", 10);
    let r2 = graph.add_variable("r2", "&i32", 20);
    graph.add_borrow(r1, x, false, 10);
    graph.add_borrow(r2, x, false, 20);

    let borrowers = graph.borrowers_of(x);
    assert_eq!(borrowers.len(), 2);
    assert!(borrowers.contains(&r1));
    assert!(borrowers.contains(&r2));
}

#[test]
fn test_owner_of() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r = graph.add_variable("r", "&i32", 10);
    graph.add_borrow(r, x, false, 10);

    assert_eq!(graph.owner_of(r), Some(x));
    assert_eq!(graph.owner_of(x), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// Remove operations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_remove_edge() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r = graph.add_variable("r", "&i32", 10);
    let eid = graph.add_borrow(r, x, false, 10);

    assert_eq!(graph.edge_count(), 1);
    graph.remove_edge(eid);
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.borrowers_of(x), vec![]);
}

#[test]
fn test_remove_node() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r = graph.add_variable("r", "&i32", 10);
    graph.add_borrow(r, x, false, 10);

    graph.remove_node(r);
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0); // edge removed with node
    assert_eq!(graph.borrowers_of(x), vec![]);
}

#[test]
fn test_remove_node_cleans_name_index() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    assert_eq!(graph.find_by_name("x").len(), 1);

    graph.remove_node(x);
    assert_eq!(graph.find_by_name("x").len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// from_events tests
// ═══════════════════════════════════════════════════════════════════════════

fn make_new_event(var_id: &str, var_name: &str, type_name: &str, ts: u64) -> Event {
    Event::New {
        timestamp: ts,
        var_name: var_name.to_string(),
        var_id: var_id.to_string(),
        type_name: type_name.to_string(),
    }
}

fn make_borrow_event(
    borrower_id: &str,
    borrower_name: &str,
    owner_id: &str,
    mutable: bool,
    ts: u64,
) -> Event {
    Event::Borrow {
        timestamp: ts,
        borrower_name: borrower_name.to_string(),
        borrower_id: borrower_id.to_string(),
        owner_id: owner_id.to_string(),
        mutable,
    }
}

fn make_drop_event(var_id: &str, ts: u64) -> Event {
    Event::Drop {
        timestamp: ts,
        var_id: var_id.to_string(),
        location: None,
    }
}

fn make_move_event(from_id: &str, to_name: &str, to_id: &str, ts: u64) -> Event {
    Event::Move {
        timestamp: ts,
        from_id: from_id.to_string(),
        to_name: to_name.to_string(),
        to_id: to_id.to_string(),
    }
}

#[test]
fn test_from_events_empty() {
    let graph = OwnershipGraph::from_events(&[]);
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_from_events_single_variable() {
    let events = vec![
        make_new_event("x_0", "x", "i32", 10),
        make_drop_event("x_0", 50),
    ];
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
    let x = graph.find_by_name("x")[0];
    assert_eq!(graph.get_node(x).unwrap().end_time(), Some(50));
}

#[test]
fn test_from_events_borrow() {
    let events = vec![
        make_new_event("x_0", "x", "Vec<i32>", 0),
        make_new_event("r_0", "r", "&Vec<i32>", 10),
        make_borrow_event("r_0", "r", "x_0", false, 10),
        make_drop_event("r_0", 50),
        make_drop_event("x_0", 60),
    ];
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    let x = graph.find_by_name("x")[0];
    let r = graph.find_by_name("r")[0];
    assert_eq!(graph.borrowers_of(x), vec![r]);
    assert_eq!(graph.owner_of(r), Some(x));

    // Borrow edge ended when r was dropped
    let eid = graph.outgoing_edges(r)[0];
    let edge = graph.get_edge(eid).unwrap();
    assert_eq!(edge.ended_at, Some(50));
}

#[test]
fn test_from_events_move() {
    let events = vec![
        make_new_event("a_0", "a", "String", 0),
        make_move_event("a_0", "b", "b_0", 20),
        make_drop_event("b_0", 50),
    ];
    let graph = OwnershipGraph::from_events(&events);

    // a + b (created by move)
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    let a = graph.find_by_name("a")[0];
    let b = graph.find_by_name("b")[0];
    let edge = graph.get_edge(graph.outgoing_edges(a)[0]).unwrap();
    assert_eq!(edge.kind, EdgeKind::Move);
    assert_eq!(edge.target, b);
}

#[test]
fn test_from_events_multiple_borrows() {
    let events = vec![
        make_new_event("x_0", "x", "Vec<i32>", 0),
        make_new_event("r1_0", "r1", "&Vec<i32>", 10),
        make_borrow_event("r1_0", "r1", "x_0", false, 10),
        make_new_event("r2_0", "r2", "&Vec<i32>", 20),
        make_borrow_event("r2_0", "r2", "x_0", false, 20),
        make_drop_event("r1_0", 40),
        make_drop_event("r2_0", 50),
        make_drop_event("x_0", 60),
    ];
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);

    let x = graph.find_by_name("x")[0];
    assert_eq!(graph.borrowers_of(x).len(), 2);
}

#[test]
fn test_from_events_rc_clone() {
    let events = vec![
        make_new_event("rc1_0", "rc1", "Rc<i32>", 0),
        Event::RcClone {
            timestamp: 10,
            var_name: "rc2".to_string(),
            var_id: "rc2_0".to_string(),
            source_id: "rc1_0".to_string(),
            strong_count: 2,
            weak_count: 0,
        },
        make_drop_event("rc2_0", 50),
        make_drop_event("rc1_0", 60),
    ];
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    let rc2 = graph.find_by_name("rc2")[0];
    let edge = graph.get_edge(graph.outgoing_edges(rc2)[0]).unwrap();
    assert_eq!(edge.kind, EdgeKind::RcClone { strong_count: 2 });
}

#[test]
fn test_from_events_scope_events() {
    let events = vec![
        Event::FnEnter {
            timestamp: 0,
            fn_id: "fn_main".to_string(),
            fn_name: "main".to_string(),
            location: "src/main.rs:1".to_string(),
        },
        make_new_event("x_0", "x", "i32", 5),
        Event::FnExit {
            timestamp: 100,
            fn_id: "fn_main".to_string(),
            fn_name: "main".to_string(),
            location: "src/main.rs:10".to_string(),
        },
    ];
    let graph = OwnershipGraph::from_events(&events);

    // Should have scope node + variable node
    assert_eq!(graph.node_count(), 2);
    let main_nodes = graph.find_by_name("main");
    assert_eq!(main_nodes.len(), 1);
    let main_node = graph.get_node(main_nodes[0]).unwrap();
    assert_eq!(main_node.end_time(), Some(100));
}

#[test]
fn test_from_events_region_events() {
    let events = vec![
        Event::RegionEnter {
            timestamp: 10,
            region_id: "region_0".to_string(),
            name: "loop_body".to_string(),
            location: "src/main.rs:5".to_string(),
        },
        make_new_event("x_0", "x", "i32", 15),
        Event::RegionExit {
            timestamp: 50,
            region_id: "region_0".to_string(),
            location: "src/main.rs:8".to_string(),
        },
    ];
    let graph = OwnershipGraph::from_events(&events);

    assert_eq!(graph.node_count(), 2);
    let scope_nodes = graph.find_by_name("loop_body");
    assert_eq!(scope_nodes.len(), 1);
    let scope = graph.get_node(scope_nodes[0]).unwrap();
    assert_eq!(scope.start_time(), 10);
    assert_eq!(scope.end_time(), Some(50));
}

// ═══════════════════════════════════════════════════════════════════════════
// GraphStream tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_stream_produces_same_as_batch() {
    let events = vec![
        make_new_event("x_0", "x", "i32", 0),
        make_new_event("r_0", "r", "&i32", 10),
        make_borrow_event("r_0", "r", "x_0", false, 10),
        make_drop_event("r_0", 50),
        make_drop_event("x_0", 60),
    ];

    let batch_graph = OwnershipGraph::from_events(&events);

    let mut stream = GraphStream::new();
    for event in &events {
        stream.push(event);
    }
    let stream_graph = stream.into_graph();

    assert_eq!(batch_graph.node_count(), stream_graph.node_count());
    assert_eq!(batch_graph.edge_count(), stream_graph.edge_count());
}

#[test]
fn test_stream_returns_updates() {
    let mut stream = GraphStream::new();

    let update = stream.push(&make_new_event("x_0", "x", "i32", 0));
    assert!(matches!(update, GraphUpdate::NodeAdded(_)));

    let update = stream.push(&make_new_event("r_0", "r", "&i32", 10));
    assert!(matches!(update, GraphUpdate::NodeAdded(_)));

    let update = stream.push(&make_borrow_event("r_0", "r", "x_0", false, 10));
    assert!(matches!(update, GraphUpdate::EdgeAdded(_)));

    let update = stream.push(&make_drop_event("r_0", 50));
    assert!(matches!(update, GraphUpdate::NodeDropped(_)));
}

#[test]
fn test_stream_unknown_event_returns_noop() {
    let mut stream = GraphStream::new();
    // LoopEnter is not handled, should return NoOp
    let event = Event::LoopEnter {
        timestamp: 0,
        loop_id: "l".into(),
        loop_type: "loop".into(),
        location: "src/main.rs:1".into(),
    };
    assert_eq!(stream.push(&event), GraphUpdate::NoOp);
}

// ═══════════════════════════════════════════════════════════════════════════
// Serialization round-trip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_roundtrip() {
    let mut graph = OwnershipGraph::new();
    let x = graph.add_variable("x", "i32", 0);
    let r = graph.add_variable("r", "&i32", 10);
    graph.add_borrow(r, x, false, 10);
    graph.mark_dropped(r, 50);

    let json = serde_json::to_string(&graph).unwrap();
    let mut restored: OwnershipGraph = serde_json::from_str(&json).unwrap();
    restored.rebuild_indices();

    assert_eq!(restored.node_count(), 2);
    assert_eq!(restored.edge_count(), 1);
    assert_eq!(restored.find_by_name("x").len(), 1);
    assert_eq!(restored.find_by_name("r").len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Merge tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_merge_disjoint_graphs() {
    let mut g1 = OwnershipGraph::new();
    g1.add_variable("a", "i32", 0);
    g1.add_variable("b", "i32", 10);

    let mut g2 = OwnershipGraph::new();
    g2.add_variable("c", "String", 0);
    g2.add_variable("d", "String", 10);

    let id_map = g1.merge(&g2);
    assert_eq!(g1.node_count(), 4);
    assert_eq!(id_map.len(), 2);
    assert_eq!(g1.find_by_name("c").len(), 1);
    assert_eq!(g1.find_by_name("d").len(), 1);
}

#[test]
fn test_merge_preserves_edges() {
    let mut g1 = OwnershipGraph::new();
    g1.add_variable("a", "i32", 0);

    let mut g2 = OwnershipGraph::new();
    let x = g2.add_variable("x", "i32", 0);
    let r = g2.add_variable("r", "&i32", 10);
    g2.add_borrow(r, x, false, 10);

    g1.merge(&g2);
    assert_eq!(g1.node_count(), 3);
    assert_eq!(g1.edge_count(), 1);
}

#[test]
fn test_merge_no_id_collisions() {
    let mut g1 = OwnershipGraph::new();
    let a = g1.add_variable("a", "i32", 0);

    let mut g2 = OwnershipGraph::new();
    g2.add_variable("b", "i32", 0);

    let id_map = g1.merge(&g2);
    // The merged node should have a different ID than 'a'
    let new_b_id = id_map[&NodeId(0)];
    assert_ne!(new_b_id, a);
}
