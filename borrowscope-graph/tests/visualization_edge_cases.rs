use borrowscope_graph::{OwnershipGraph, Variable};

// ============================================================================
// Extreme Scale Tests
// ============================================================================

#[test]
fn test_visualization_10k_nodes() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=10000 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
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

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 10000);
    assert!(!viz.style.is_empty());
}

#[test]
fn test_visualization_dense_edges() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 10,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    for i in 2..=100 {
        for j in 1..i {
            if (i * j) % 7 == 0 {
                graph.add_borrow(i, j, false, i as u64 * 10);
            }
        }
    }

    let viz = graph.export_for_visualization();
    assert!(viz.elements.edges.len() > 100);
}

#[test]
fn test_timeline_1000_timestamps() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=500 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 2,
            dropped_at: Some(i as u64 * 2 + 1),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();
    assert_eq!(timeline.len(), 1000);
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_visualization_zero_timestamps() {
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

    let viz = graph.export_for_visualization_at(Some(0));
    assert_eq!(viz.elements.nodes.len(), 10);
    assert!(viz.elements.nodes.iter().all(|n| !n.data.is_alive));
}

#[test]
fn test_visualization_u64_max_all_fields() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: usize::MAX,
        name: "max".into(),
        type_name: "i32".into(),
        created_at: u64::MAX - 1000,
        dropped_at: Some(u64::MAX),
        scope_depth: usize::MAX,
    });

    let viz = graph.export_for_visualization_at(Some(u64::MAX - 500));
    assert_eq!(viz.elements.nodes.len(), 1);
    assert!(viz.elements.nodes[0].data.is_alive);
    assert_eq!(viz.elements.nodes[0].data.scope_depth, usize::MAX);
}

#[test]
fn test_timeline_at_exact_boundaries() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "x".into(),
        type_name: "i32".into(),
        created_at: 1000,
        dropped_at: Some(2000),
        scope_depth: 0,
    });

    let timeline = graph.export_timeline();

    let at_1000 = timeline.iter().find(|f| f.timestamp == 1000).unwrap();
    assert_eq!(at_1000.elements.nodes.len(), 1);
    assert!(at_1000.elements.nodes[0].data.is_alive);

    let at_2000 = timeline.iter().find(|f| f.timestamp == 2000).unwrap();
    assert_eq!(at_2000.elements.nodes.len(), 1);
    assert!(!at_2000.elements.nodes[0].data.is_alive);
}

// ============================================================================
// Unicode and Special Character Tests
// ============================================================================

#[test]
fn test_visualization_all_unicode_scripts() {
    let mut graph = OwnershipGraph::new();

    let names = [
        "变量",       // Chinese
        "переменная", // Russian
        "μεταβλητή",  // Greek
        "متغير",      // Arabic
        "変数",       // Japanese
        "변수",       // Korean
        "משתנה",      // Hebrew
        "ตัวแปร",      // Thai
    ];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    let json = serde_json::to_string(&viz).unwrap();

    for name in &names {
        assert!(json.contains(name));
    }
}

#[test]
fn test_visualization_emoji_sequences() {
    let mut graph = OwnershipGraph::new();

    let emojis = ["🦀", "👨‍👩‍👧‍👦", "🏳️‍🌈", "👍🏿"];

    for (i, emoji) in emojis.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: emoji.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 4);
}

#[test]
fn test_visualization_control_characters() {
    let mut graph = OwnershipGraph::new();

    let names = [
        "var\nwith\nnewlines",
        "var\twith\ttabs",
        "var\rwith\rreturns",
    ];

    for (i, name) in names.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: name.to_string(),
            type_name: "i32".into(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    let json = serde_json::to_string(&viz).unwrap();
    assert!(json.contains("\\n"));
    assert!(json.contains("\\t"));
}

#[test]
fn test_visualization_empty_strings() {
    let mut graph = OwnershipGraph::new();

    graph.add_variable(Variable {
        id: 1,
        name: "".into(),
        type_name: "".into(),
        created_at: 100,
        dropped_at: None,
        scope_depth: 0,
    });

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 1);
    assert_eq!(viz.elements.nodes[0].data.label, "");
    assert_eq!(viz.elements.nodes[0].data.type_name, "");
}

// ============================================================================
// Complex Type Tests
// ============================================================================

#[test]
fn test_visualization_deeply_nested_types() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Arc<Mutex<Vec<Box<dyn Trait + Send + Sync>>>>",
        "HashMap<String, Vec<Rc<RefCell<Option<i32>>>>>",
        "&'static [&'a [&'b [u8]]]",
        "fn(Box<dyn Fn(i32) -> Result<String, Error>>) -> Option<Vec<i32>>",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();
    assert_eq!(viz.elements.nodes.len(), 4);

    let classes: Vec<_> = viz
        .elements
        .nodes
        .iter()
        .filter_map(|n| n.classes.as_ref())
        .collect();

    assert!(classes.iter().any(|c| c.contains("arc")));
    assert!(classes.iter().any(|c| c.contains("rc")));
}

#[test]
fn test_visualization_all_smart_pointer_combinations() {
    let mut graph = OwnershipGraph::new();

    let types = [
        "Box<i32>",
        "Rc<i32>",
        "Arc<i32>",
        "RefCell<i32>",
        "Box<Rc<i32>>",
        "Rc<Box<i32>>",
        "Arc<RefCell<i32>>",
        "Box<Arc<Rc<RefCell<i32>>>>",
    ];

    for (i, ty) in types.iter().enumerate() {
        graph.add_variable(Variable {
            id: i + 1,
            name: format!("v{}", i),
            type_name: ty.to_string(),
            created_at: (i + 1) as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let viz = graph.export_for_visualization();

    // The else-if chain checks in order: Rc, Arc, RefCell, Box
    // So only the first match is added:
    // Box<i32> -> box (no Rc/Arc/RefCell)
    // Rc<i32> -> rc
    // Arc<i32> -> arc
    // RefCell<i32> -> refcell
    // Box<Rc<i32>> -> rc (Rc comes first)
    // Rc<Box<i32>> -> rc (Rc comes first)
    // Arc<RefCell<i32>> -> arc (Arc comes first)
    // Box<Arc<Rc<RefCell<i32>>>> -> rc (Rc comes first)

    let with_box = viz
        .elements
        .nodes
        .iter()
        .filter(|n| {
            let classes = n.classes.as_ref().unwrap();
            classes.split_whitespace().any(|c| c == "box")
        })
        .count();
    assert_eq!(with_box, 1); // Only Box<i32>

    let with_rc = viz
        .elements
        .nodes
        .iter()
        .filter(|n| {
            let classes = n.classes.as_ref().unwrap();
            classes.split_whitespace().any(|c| c == "rc")
        })
        .count();
    assert_eq!(with_rc, 4); // Rc<i32>, Box<Rc<i32>>, Rc<Box<i32>>, Box<Arc<Rc<RefCell<i32>>>>

    let with_arc = viz
        .elements
        .nodes
        .iter()
        .filter(|n| {
            let classes = n.classes.as_ref().unwrap();
            classes.split_whitespace().any(|c| c == "arc")
        })
        .count();
    assert_eq!(with_arc, 2); // Arc<i32>, Arc<RefCell<i32>>

    let with_refcell = viz
        .elements
        .nodes
        .iter()
        .filter(|n| {
            let classes = n.classes.as_ref().unwrap();
            classes.split_whitespace().any(|c| c == "refcell")
        })
        .count();
    assert_eq!(with_refcell, 1); // RefCell<i32>
}

// ============================================================================
// Timeline Edge Cases
// ============================================================================

#[test]
fn test_timeline_simultaneous_events() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: 1000,
            dropped_at: Some(2000),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].elements.nodes.len(), 100);
    assert_eq!(timeline[1].elements.nodes.len(), 100);
}

#[test]
fn test_timeline_interleaved_lifetimes() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=50 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 10,
            dropped_at: Some(i as u64 * 10 + 100),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();

    let mid_frame = timeline.iter().find(|f| f.timestamp == 250).unwrap();

    let alive = mid_frame
        .elements
        .nodes
        .iter()
        .filter(|n| n.data.is_alive)
        .count();

    assert!(alive >= 10);
}

#[test]
fn test_timeline_no_drops() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: i as u64 * 100,
            dropped_at: None,
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();
    assert_eq!(timeline.len(), 100);

    for frame in &timeline {
        assert!(frame.elements.nodes.iter().all(|n| n.data.is_alive));
    }
}

#[test]
fn test_timeline_all_dropped_immediately() {
    let mut graph = OwnershipGraph::new();

    for i in 1..=100 {
        let time = i as u64 * 100;
        graph.add_variable(Variable {
            id: i,
            name: format!("v{}", i),
            type_name: "i32".into(),
            created_at: time,
            dropped_at: Some(time),
            scope_depth: 0,
        });
    }

    let timeline = graph.export_timeline();
    assert_eq!(timeline.len(), 100);
}
