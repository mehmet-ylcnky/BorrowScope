//! Battle Test: BorrowScope on uuid crate
//! Tests ownership tracking with the popular uuid crate.
//!
//! Patterns: String conversions, byte arrays, Option/Result, HashMap, Vec,
//! sorting, filtering, cloning, formatting

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use std::collections::HashMap;
use uuid::Uuid;

#[trace_borrow]
fn uuid_creation_and_conversion() {
    // Create random UUID (v4)
    let id = Uuid::new_v4();
    println!("  UUID v4: {}", id);

    // Convert to string (owned)
    let id_string = id.to_string();
    println!("  As string: {}", id_string);

    // Convert to bytes (copy)
    let bytes = *id.as_bytes();
    println!("  Bytes len: {}", bytes.len());

    // Parse from string
    let parsed = Uuid::parse_str(&id_string).unwrap();
    println!("  Parsed == original: {}", parsed == id);

    // Nil UUID
    let nil = Uuid::nil();
    let is_nil = nil.is_nil();
    println!("  Nil UUID is_nil: {}", is_nil);
}

#[trace_borrow]
fn uuid_collections() {
    // Build a registry of UUIDs
    let mut registry: HashMap<Uuid, String> = HashMap::new();

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let user3 = Uuid::new_v4();

    registry.insert(user1, String::from("Alice"));
    registry.insert(user2, String::from("Bob"));
    registry.insert(user3, String::from("Charlie"));

    // Lookup
    let name = registry.get(&user1);
    println!("  User1 name: {:?}", name);

    // Collect keys
    let ids: Vec<&Uuid> = registry.keys().collect();
    println!("  Registry has {} entries", ids.len());

    // Filter and collect
    let bobs: Vec<(&Uuid, &String)> = registry.iter()
        .filter(|(_id, name)| name.starts_with("B"))
        .collect();
    println!("  Names starting with B: {}", bobs.len());
}

#[trace_borrow]
fn uuid_namespace_operations() {
    // v5 UUID (namespace-based, deterministic)
    let namespace = Uuid::NAMESPACE_DNS;
    let name = "example.com";
    let id = Uuid::new_v5(&namespace, name.as_bytes());
    println!("  v5 UUID for '{}': {}", name, id);

    // Same input = same output (deterministic)
    let id2 = Uuid::new_v5(&namespace, name.as_bytes());
    println!("  Deterministic: {}", id == id2);

    // Different name = different UUID
    let id3 = Uuid::new_v5(&namespace, b"other.com");
    println!("  Different: {}", id != id3);
}

#[trace_borrow]
fn uuid_sorting_and_comparison() {
    let mut ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    println!("  Generated {} UUIDs", ids.len());

    // Sort
    ids.sort();
    println!("  Sorted first: {}", ids[0]);
    println!("  Sorted last: {}", ids[4]);

    // Dedup (after sort)
    ids.push(ids[0]); // duplicate
    ids.sort();
    ids.dedup();
    println!("  After dedup: {} unique", ids.len());

    // Find min/max
    let min = ids.iter().min().unwrap();
    let max = ids.iter().max().unwrap();
    println!("  Min: {}", min);
    println!("  Max: {}", max);
}

#[trace_borrow]
fn uuid_option_result_patterns() {
    // Parse valid
    let valid: Result<Uuid, _> = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000");
    let id = valid.unwrap();
    println!("  Parsed valid: {}", id);

    // Parse invalid
    let invalid: Result<Uuid, _> = Uuid::parse_str("not-a-uuid");
    let err = invalid.unwrap_err();
    println!("  Parse error: {}", err);

    // Option patterns
    let maybe_id: Option<Uuid> = Some(Uuid::new_v4());
    let extracted = maybe_id.unwrap();
    println!("  Extracted: {}", extracted);

    // Map over Option
    let mapped: Option<String> = Some(Uuid::new_v4()).map(|id| id.to_string());
    println!("  Mapped: {:?}", mapped);
}

#[trace_borrow]
fn uuid_clone_and_move() {
    let original = Uuid::new_v4();

    // Clone
    let cloned = original.clone();
    println!("  Original: {}", original);
    println!("  Cloned: {}", cloned);
    println!("  Equal: {}", original == cloned);

    // Move into Vec
    let mut collection = Vec::new();
    collection.push(original);
    collection.push(cloned);
    println!("  Collection size: {}", collection.len());

    // Drain
    let drained: Vec<Uuid> = collection.drain(..).collect();
    println!("  Drained: {} items", drained.len());
    println!("  Original collection empty: {}", collection.is_empty());
}

fn main() {
    println!("=== BorrowScope Battle Test: uuid crate ===\n");

    println!("--- Test 1: Creation & Conversion ---");
    reset();
    uuid_creation_and_conversion();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 2: Collections ---");
    reset();
    uuid_collections();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 3: Namespace Operations ---");
    reset();
    uuid_namespace_operations();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 4: Sorting & Comparison ---");
    reset();
    uuid_sorting_and_comparison();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 5: Option/Result Patterns ---");
    reset();
    uuid_option_result_patterns();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 6: Clone & Move ---");
    reset();
    uuid_clone_and_move();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    // Summary
    println!("=== Final Summary ===");
    reset();
    uuid_creation_and_conversion();
    uuid_collections();
    uuid_namespace_operations();
    uuid_sorting_and_comparison();
    uuid_option_result_patterns();
    uuid_clone_and_move();
    let all = get_events();
    println!("Total events: {}", all.len());
    print_summary();
}
