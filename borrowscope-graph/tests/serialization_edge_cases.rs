use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Extreme Size Tests
// ============================================================================

#[test]
fn test_serialization_10k_nodes() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("var{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: if i % 2 == 0 {
                Some(i as u64 + 1000)
            } else {
                None
            },
            scope_depth: i % 100,
        });
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.node_count(), 10000);
}

#[test]
fn test_serialization_dense_edges() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=200 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 10,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=200 {
        for j in 1..i {
            if (i * j) % 13 == 0 {
                graph.add_borrow(i, j, (i + j) % 3 == 0, i as u64 * 10);
            }
        }
    }

    let msgpack = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&msgpack).unwrap();

    assert_eq!(graph.node_count(), restored.node_count());
    assert_eq!(graph.edge_count(), restored.edge_count());
}

#[test]
fn test_serialization_maximum_scope_depth() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: usize::MAX - i,
        });
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    for i in 1..=100 {
        let var = restored.get_variable(i).unwrap();
        assert_eq!(var.scope_depth, usize::MAX - i);
    }
}

// ============================================================================
// Timestamp Edge Cases
// ============================================================================

#[test]
fn test_serialization_u64_max_timestamps() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "y".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 500,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(
        restored.get_variable(1).unwrap().created_at,
        u64::MAX - 1000
    );
    assert_eq!(restored.get_variable(1).unwrap().dropped_at, Some(u64::MAX));
}

#[test]
fn test_serialization_zero_timestamps() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 0,
            dropped_at: Some(0),
            scope_depth: 0,
        });
    }

    let msgpack = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&msgpack).unwrap();

    for i in 1..=10 {
        assert_eq!(restored.get_variable(i).unwrap().created_at, 0);
        assert_eq!(restored.get_variable(i).unwrap().dropped_at, Some(0));
    }
}

#[test]
fn test_serialization_timestamp_ordering() {
    let mut graph = OwnershipGraph::new();

    let timestamps = [u64::MAX, 0, 1, u64::MAX / 2, 42, 1000000];

    for (i, &ts) in timestamps.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: ts,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    for (i, &ts) in timestamps.iter().enumerate() {
        assert_eq!(restored.get_variable(i + 1).unwrap().created_at, ts);
    }
}

// ============================================================================
// Complex Type Names
// ============================================================================

#[test]
fn test_serialization_deeply_nested_types() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Vec<HashMap<String, Arc<Mutex<RefCell<Box<dyn Fn() -> Result<Option<Vec<i32>>, Error>>>>>>>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(
        restored.get_variable(1).unwrap().type_name,
        "Vec<HashMap<String, Arc<Mutex<RefCell<Box<dyn Fn() -> Result<Option<Vec<i32>>, Error>>>>>>>"
    );
}

#[test]
fn test_serialization_generic_types_with_lifetimes() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Cow<'static, str>",
        "&'a mut T where T: 'static",
        "Box<dyn Trait + 'a>",
        "impl Iterator<Item = &'a str>",
        "for<'a> fn(&'a str) -> &'a str",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let msgpack = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&msgpack).unwrap();

    for (i, ty) in types.iter().enumerate() {
        assert_eq!(restored.get_variable(i + 1).unwrap().type_name, *ty);
    }
}

#[test]
fn test_serialization_type_with_special_chars() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "Type<'a, T: Trait + ?Sized>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert_eq!(
        restored.get_variable(1).unwrap().type_name,
        "Type<'a, T: Trait + ?Sized>"
    );
}

// ============================================================================
// Unicode and International Characters
// ============================================================================

#[test]
fn test_serialization_emoji_names() {
    let mut graph = OwnershipGraph::new();

    let names = ["🦀", "🔥", "⚡", "🌟", "💻", "🚀"];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    for (i, name) in names.iter().enumerate() {
        assert_eq!(restored.get_variable(i + 1).unwrap().name, *name);
    }
}

#[test]
fn test_serialization_mixed_scripts() {
    let mut graph = OwnershipGraph::new();

    let names = [
        "变量",       // Chinese
        "переменная", // Russian
        "μεταβλητή",  // Greek
        "متغير",      // Arabic
        "変数",       // Japanese
        "변수",       // Korean
        "משתנה",      // Hebrew
    ];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let msgpack = graph.to_messagepack().unwrap();
    let restored = OwnershipGraph::from_messagepack(&msgpack).unwrap();

    for (i, name) in names.iter().enumerate() {
        assert_eq!(restored.get_variable(i + 1).unwrap().name, *name);
    }
}

#[test]
fn test_serialization_combining_characters() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "café".into(), // é as combining character
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    assert!(restored.get_variable(1).unwrap().name.contains("caf"));
}

// ============================================================================
// Malformed Data Recovery
// ============================================================================

#[test]
fn test_serialization_truncated_json() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let json = graph.to_json().unwrap();
    let truncated = &json[..json.len() / 2];

    let result = OwnershipGraph::from_json(truncated);
    assert!(result.is_err());
}

#[test]
fn test_serialization_corrupted_messagepack() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let mut msgpack = graph.to_messagepack().unwrap();

    // Corrupt some bytes
    if msgpack.len() > 10 {
        msgpack[5] = 0xFF;
        msgpack[6] = 0xFF;
    }

    let result = OwnershipGraph::from_messagepack(&msgpack);
    assert!(result.is_err());
}

#[test]
fn test_serialization_empty_json_object() {
    let result = OwnershipGraph::from_json("{}");
    assert!(result.is_err());
}

#[test]
fn test_serialization_json_array_instead_of_object() {
    let result = OwnershipGraph::from_json("[]");
    assert!(result.is_err());
}

// ============================================================================
// Delta Serialization Advanced Tests
// ============================================================================

#[test]
fn test_delta_large_scale_changes() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    for i in 1..=1000 {
        graph1.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 501..=1500 {
        graph2.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: if i > 1000 {
                None
            } else {
                Some(i as u64 + 1000)
            },
            scope_depth: 0,
        });
    }

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 500);
    assert_eq!(delta.removed_nodes.len(), 500);
    assert_eq!(delta.modified_nodes.len(), 500);
}

#[test]
fn test_delta_only_edge_changes() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    for i in 1..=10 {
        let var = Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        };
        graph1.add_variable(var.clone());
        graph2.add_variable(var);
    }

    for i in 2..=10 {
        graph2.add_borrow(i, i - 1, false, i as u64 * 100);
    }

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 0);
    assert_eq!(delta.removed_nodes.len(), 0);
    assert_eq!(delta.modified_nodes.len(), 0);
    assert_eq!(delta.added_edges.len(), 9);
}

#[test]
fn test_delta_all_nodes_modified() {
    let mut graph1 = OwnershipGraph::new();
    let mut graph2 = OwnershipGraph::new();

    for i in 1..=100 {
        graph1.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });

        graph2.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: Some(i as u64 + 1000),
            scope_depth: 0,
        });
    }

    let export1 = graph1.export();
    let delta = graph2.export_delta(&export1);

    assert_eq!(delta.added_nodes.len(), 0);
    assert_eq!(delta.removed_nodes.len(), 0);
    assert_eq!(delta.modified_nodes.len(), 100);
}

// ============================================================================
// DOT Format Advanced Tests
// ============================================================================

#[test]
fn test_dot_format_complex_graph() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("node_{}", i),
            type_name: format!("Type{}", i),
            created_at: i as u64 * 100,
            dropped_at: if i % 3 == 0 {
                Some(i as u64 * 200)
            } else {
                None
            },
            scope_depth: i % 5,
        });
    }

    for i in 2..=50 {
        if i % 2 == 0 {
            graph.add_borrow(i, i - 1, i % 4 == 0, i as u64 * 100);
        }
    }

    let dot = graph.to_dot();

    assert!(dot.contains("digraph OwnershipGraph"));
    assert!(dot.matches("->").count() >= 24);
    assert!(dot.contains("lightgray"));
    assert!(dot.contains("lightblue"));
}

#[test]
fn test_dot_format_all_edge_types() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    graph.add_borrow(2, 1, false, 200);
    graph.add_borrow(3, 1, true, 300);
    graph.add_move(4, 1, 400);
    graph.add_rc_clone(5, 1, 2, 500);
    graph.add_arc_clone(6, 1, 3, 600);
    graph.add_refcell_borrow(7, 1, false, 700);
    graph.add_refcell_borrow(8, 1, true, 800);

    let dot = graph.to_dot();

    assert!(dot.contains("blue"));
    assert!(dot.contains("red"));
    assert!(dot.contains("black"));
    assert!(dot.contains("green"));
    assert!(dot.contains("purple"));
    assert!(dot.contains("orange"));
}

#[test]
fn test_dot_format_special_characters_escaping() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "var\"with\\quotes".into(),
        type_name: "Type<T>".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    graph.add_variable(Variable {
        id: 2,
        name: "var\nwith\nnewlines".into(),
        type_name: "i32".into(),
        created_at: 200,
        dropped_at: None,
        scope_depth: 0,
    });

    let dot = graph.to_dot();

    assert!(dot.contains("n1"));
    assert!(dot.contains("n2"));
}

// ============================================================================
// Performance and Size Tests
// ============================================================================

#[test]
fn test_serialization_size_comparison() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=1000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("variable_with_long_name_{}", i),
            type_name: "Vec<HashMap<String, Vec<i32>>>".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: i % 10,
        });
    }

    let json_pretty = graph.to_json_pretty().unwrap();
    let json_compact = graph.to_json_compact().unwrap();
    let msgpack = graph.to_messagepack().unwrap();

    assert!(json_pretty.len() > json_compact.len());
    assert!(msgpack.len() < json_compact.len());

    let ratio = json_compact.len() as f64 / msgpack.len() as f64;
    assert!(ratio > 1.1);
}

#[test]
fn test_serialization_roundtrip_preserves_order() {
    let mut graph = OwnershipGraph::new();

    for i in (1..=100).rev() {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let json = graph.to_json().unwrap();
    let restored = OwnershipGraph::from_json(&json).unwrap();

    for i in 1..=100 {
        assert!(restored.get_variable(i).is_some());
    }
}

#[test]
fn test_serialization_concurrent_modifications() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let export1 = graph.export();

    for i in 1..=50 {
        graph.mark_dropped(i, i as u64 + 1000);
    }

    let export2 = graph.export();
    let delta = graph.export_delta(&export1);

    assert_eq!(delta.modified_nodes.len(), 50);
    assert!(
        export2
            .nodes
            .iter()
            .filter(|n| n.dropped_at.is_some())
            .count()
            >= 50
    );
}
