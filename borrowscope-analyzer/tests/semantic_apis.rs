//! Integration tests for semantic API coverage
//!
//! Tests all rust-analyzer semantic APIs implemented in the analyzer.

use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

static ANALYSIS: OnceLock<serde_json::Value> = OnceLock::new();

fn get_analysis() -> &'static serde_json::Value {
    ANALYSIS.get_or_init(|| {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let full_project_path = workspace_root.join("examples/type-coverage");

        let status = Command::new("cargo")
            .args([
                "run",
                "-p",
                "borrowscope-analyzer",
                "--",
                full_project_path.to_str().unwrap(),
            ])
            .current_dir(workspace_root)
            .output()
            .expect("Failed to run analyzer");

        assert!(
            status.status.success(),
            "Analyzer failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );

        let json_path = full_project_path.join(".borrowscope/type-info.json");
        let content = std::fs::read_to_string(&json_path).expect("Failed to read output");
        serde_json::from_str(&content).expect("Failed to parse JSON")
    })
}

fn find_var<'a>(json: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    json["files"]["src/main.rs"]
        .as_array()?
        .iter()
        .find(|v| v["name"] == name)
}

#[test]
fn test_all_semantic_apis() {
    let json = get_analysis();
    let file = "src/main.rs";

    // === Type::future_output() ===
    let future_vars: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["future_output_type"].as_str().is_some())
        .collect();
    assert!(
        future_vars.len() >= 10,
        "Expected 10+ futures with output types, got {}",
        future_vars.len()
    );

    // === Type::iterator_item() ===
    let iter_vars: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["iterator_item_type"].as_str().is_some())
        .collect();
    assert!(
        iter_vars.len() >= 5,
        "Expected 5+ iterators with item types, got {}",
        iter_vars.len()
    );

    // === Type::impls_fnonce() / is_callable ===
    let callable_vars: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["is_callable"] == true)
        .collect();
    assert!(
        callable_vars.len() >= 10,
        "Expected 10+ callable vars, got {}",
        callable_vars.len()
    );

    // === Adt::layout() ===
    let layout_vars: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["layout"].is_object())
        .collect();
    assert!(
        layout_vars.len() >= 100,
        "Expected 100+ vars with layout, got {}",
        layout_vars.len()
    );

    // === Type::autoderef() / deref_chain ===
    let deref_vars: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| {
            v["deref_chain"]
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false)
        })
        .collect();
    assert!(
        deref_vars.len() >= 100,
        "Expected 100+ vars with deref chains, got {}",
        deref_vars.len()
    );

    // Verify Rc<String> has deref chain: String -> str
    let rc_var = find_var(json, "my_rc").expect("my_rc not found");
    let deref = rc_var["deref_chain"].as_array().unwrap();
    assert!(
        deref.iter().any(|t| t == "String"),
        "Rc should deref to String"
    );
    assert!(
        deref.iter().any(|t| t == "str"),
        "String should deref to str"
    );

    // === Type::contains_reference() ===
    let ref_containing: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["contains_reference"] == true)
        .collect();
    assert!(
        ref_containing.len() >= 50,
        "Expected 50+ vars containing refs, got {}",
        ref_containing.len()
    );

    // === Local::is_ref() / is_ref_binding ===
    let ref_bindings: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["is_ref_binding"] == true)
        .collect();
    assert!(
        ref_bindings.len() >= 5,
        "Expected 5+ ref bindings, got {}",
        ref_bindings.len()
    );

    // === sema.resolve_variant() ===
    let variants = json["variants"][file].as_array().expect("No variants");
    assert!(
        variants.len() >= 20,
        "Expected 20+ variants, got {}",
        variants.len()
    );

    // Check variant has required fields
    let variant = &variants[0];
    assert!(
        variant["enum_type"].as_str().is_some(),
        "variant missing enum_type"
    );
    assert!(
        variant["variant_name"].as_str().is_some(),
        "variant missing variant_name"
    );
    assert!(
        variant["variant_kind"].as_str().is_some(),
        "variant missing variant_kind"
    );

    // === sema.resolve_lifetime_param() ===
    let lifetimes = json["lifetimes"][file].as_array().expect("No lifetimes");
    assert!(
        lifetimes.len() >= 4,
        "Expected 4+ lifetimes, got {}",
        lifetimes.len()
    );
    assert!(
        lifetimes.iter().any(|l| l["name"] == "'a"),
        "Expected 'a lifetime"
    );

    // === sema.resolve_label() ===
    let labels = json["labels"][file].as_array().expect("No labels");
    assert!(
        labels.len() >= 5,
        "Expected 5+ labels, got {}",
        labels.len()
    );
    assert!(
        labels.iter().any(|l| l["loop_kind"] == "loop"),
        "Expected loop label"
    );
    assert!(
        labels.iter().any(|l| l["loop_kind"] == "while"),
        "Expected while label"
    );
    assert!(
        labels.iter().any(|l| l["loop_kind"] == "for"),
        "Expected for label"
    );

    // === sema.resolve_bind_pat_to_const() ===
    let const_pats = json["const_patterns"][file]
        .as_array()
        .expect("No const_patterns");
    assert!(
        const_pats.len() >= 2,
        "Expected 2+ const patterns, got {}",
        const_pats.len()
    );
    assert!(
        const_pats
            .iter()
            .any(|c| c["const_name"] == "CONST_PATTERN_VALUE"),
        "Expected CONST_PATTERN_VALUE"
    );

    // === Type::as_callable() ===
    let callables = json["callables"][file].as_array().expect("No callables");
    assert!(
        callables.len() >= 100,
        "Expected 100+ callables, got {}",
        callables.len()
    );

    // Check callable has required fields
    let callable = &callables[0];
    assert!(callable["kind"].as_str().is_some(), "callable missing kind");
    assert!(
        callable["param_types"].as_array().is_some(),
        "callable missing param_types"
    );

    // === sema.resolve_record_field() ===
    let record_exprs = json["record_field_exprs"][file]
        .as_array()
        .expect("No record_field_exprs");
    assert!(
        record_exprs.len() >= 30,
        "Expected 30+ record field exprs, got {}",
        record_exprs.len()
    );

    // Check record field has required fields
    let rf = &record_exprs[0];
    assert!(
        rf["parent_type"].as_str().is_some(),
        "record field missing parent_type"
    );
    assert!(
        rf["field_name"].as_str().is_some(),
        "record field missing field_name"
    );
    assert!(
        rf["field_type"].as_str().is_some(),
        "record field missing field_type"
    );

    // === sema.resolve_record_pat_field() ===
    let record_pats = json["record_field_pats"][file]
        .as_array()
        .expect("No record_field_pats");
    assert!(
        record_pats.len() >= 3,
        "Expected 3+ record field pats, got {}",
        record_pats.len()
    );

    // === sema.resolve_await_to_poll() ===
    let await_points = json["await_points"][file]
        .as_array()
        .expect("No await_points");
    let with_poll: Vec<_> = await_points
        .iter()
        .filter(|a| a["poll_function"].as_str().is_some())
        .collect();
    assert!(
        with_poll.len() >= 5,
        "Expected 5+ await points with poll_function, got {}",
        with_poll.len()
    );
    assert!(
        with_poll
            .iter()
            .any(|a| a["poll_function"].as_str().unwrap().contains("poll")),
        "Expected poll function to contain 'poll'"
    );

    // === sema.binding_mode_of_pat() ===
    let with_binding_mode: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["binding_mode"].as_str().is_some())
        .collect();
    assert!(
        with_binding_mode.len() >= 50,
        "Expected 50+ vars with binding_mode, got {}",
        with_binding_mode.len()
    );
    assert!(
        with_binding_mode
            .iter()
            .any(|v| v["binding_mode"] == "move"),
        "Expected move binding"
    );
    assert!(
        with_binding_mode.iter().any(|v| v["binding_mode"] == "ref"),
        "Expected ref binding"
    );

    // === sema.expr_adjustments() ===
    let with_adjustments: Vec<_> = json["files"][file]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| {
            v["adjustments"]
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false)
        })
        .collect();
    assert!(
        with_adjustments.len() >= 20,
        "Expected 20+ vars with adjustments, got {}",
        with_adjustments.len()
    );

    // === Closure::fn_trait() ===
    let closure_traits = json["closure_traits"][file]
        .as_array()
        .expect("No closure_traits");
    assert!(
        closure_traits.len() >= 10,
        "Expected 10+ closure traits, got {}",
        closure_traits.len()
    );
    assert!(
        closure_traits.iter().any(|c| c["fn_trait"] == "Fn"),
        "Expected Fn closure"
    );
    assert!(
        closure_traits.iter().any(|c| c["fn_trait"] == "FnMut"),
        "Expected FnMut closure"
    );
    assert!(
        closure_traits.iter().any(|c| c["fn_trait"] == "FnOnce"),
        "Expected FnOnce closure"
    );

    // === Field access (sema.resolve_field) ===
    let field_accesses = json["field_accesses"][file]
        .as_array()
        .expect("No field_accesses");
    assert!(
        field_accesses.len() >= 20,
        "Expected 20+ field accesses, got {}",
        field_accesses.len()
    );

    // === Unsafe operations ===
    let unsafe_ops = json["unsafe_operations"][file]
        .as_array()
        .expect("No unsafe_operations");
    assert!(
        unsafe_ops.len() >= 10,
        "Expected 10+ unsafe ops, got {}",
        unsafe_ops.len()
    );

    println!("All semantic API tests passed!");
    println!("  future_output_type: {}", future_vars.len());
    println!("  iterator_item_type: {}", iter_vars.len());
    println!("  is_callable: {}", callable_vars.len());
    println!("  layout: {}", layout_vars.len());
    println!("  deref_chain: {}", deref_vars.len());
    println!("  contains_reference: {}", ref_containing.len());
    println!("  is_ref_binding: {}", ref_bindings.len());
    println!("  variants: {}", variants.len());
    println!("  lifetimes: {}", lifetimes.len());
    println!("  labels: {}", labels.len());
    println!("  const_patterns: {}", const_pats.len());
    println!("  callables: {}", callables.len());
    println!("  record_field_exprs: {}", record_exprs.len());
    println!("  record_field_pats: {}", record_pats.len());
    println!("  await_points with poll: {}", with_poll.len());
    println!("  binding_mode: {}", with_binding_mode.len());
    println!("  adjustments: {}", with_adjustments.len());
    println!("  closure_traits: {}", closure_traits.len());
    println!("  field_accesses: {}", field_accesses.len());
    println!("  unsafe_operations: {}", unsafe_ops.len());
}
