//! Battle Test: BorrowScope on real-world LRU cache usage
//! Tests ownership tracking on the well-known `lru` crate (https://github.com/jeromefroe/lru-rs)
//!
//! Patterns tested:
//! - HashMap + linked list ownership (LruCache internals)
//! - Shared references from cache lookups
//! - Mutable borrows for cache mutations
//! - Option handling (get returns Option<&V>)
//! - Clone and move semantics
//! - Iterator patterns
//! - Closure captures

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use lru::LruCache;
use std::num::NonZeroUsize;

#[trace_borrow]
fn basic_cache_operations() {
    let mut cache = LruCache::new(NonZeroUsize::new(3).unwrap());

    // Insert entries
    cache.put("apple", 3);
    cache.put("banana", 5);
    cache.put("cherry", 7);

    // Get (immutable borrow of cache)
    let apple_val = cache.get(&"apple");
    println!("  apple = {:?}", apple_val);

    // Peek (doesn't update LRU order)
    let banana_val = cache.peek(&"banana");
    println!("  banana (peek) = {:?}", banana_val);

    // Contains
    let has_cherry = cache.contains(&"cherry");
    println!("  has cherry = {}", has_cherry);

    // Eviction: inserting 4th entry evicts least recently used
    cache.put("date", 11);
    let evicted = cache.get(&"banana"); // banana was LRU
    println!("  banana after eviction = {:?}", evicted);

    // Length
    let len = cache.len();
    println!("  cache len = {}", len);
}

#[trace_borrow]
fn cache_with_strings() {
    let mut cache: LruCache<String, Vec<u8>> = LruCache::new(NonZeroUsize::new(5).unwrap());

    // Owned key/value insertion
    let key = String::from("session_abc123");
    let data = vec![1, 2, 3, 4, 5];
    cache.put(key, data);

    // Get returns Option<&V>
    let session_key = String::from("session_abc123");
    let result = cache.get(&session_key);
    println!("  session data = {:?}", result);

    // Pop removes and returns owned value
    let popped = cache.pop(&String::from("session_abc123"));
    println!("  popped = {:?}", popped);

    // Verify removed
    let after_pop = cache.get(&String::from("session_abc123"));
    println!("  after pop = {:?}", after_pop);
}

#[trace_borrow]
fn cache_iteration() {
    let mut cache = LruCache::new(NonZeroUsize::new(10).unwrap());
    for i in 0..5 {
        cache.put(format!("key_{}", i), i * 10);
    }

    // Iterate (borrows cache immutably)
    let keys: Vec<&String> = cache.iter().map(|(k, _v)| k).collect();
    println!("  keys = {:?}", keys);

    // Sum values
    let sum: i32 = cache.iter().map(|(_k, v)| v).sum();
    println!("  sum of values = {}", sum);

    // Count with filter
    let big_values = cache.iter().filter(|(_k, v)| **v > 20).count();
    println!("  values > 20: {}", big_values);
}

#[trace_borrow]
fn cache_promote_and_demote() {
    let mut cache = LruCache::new(NonZeroUsize::new(4).unwrap());
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);
    cache.put("d", 4);

    // Access "a" to promote it (moves to most recently used)
    let _ = cache.get(&"a");

    // Now "b" is LRU, inserting "e" should evict "b"
    cache.put("e", 5);

    let b_gone = cache.get(&"b").copied();
    let a_still = cache.get(&"a").copied();
    println!("  b (should be evicted) = {:?}", b_gone);
    println!("  a (should exist) = {:?}", a_still);
}

#[trace_borrow]
fn cache_get_or_insert() {
    let mut cache = LruCache::new(NonZeroUsize::new(5).unwrap());

    // get_or_insert pattern
    let key = "computed";
    let value = cache.get_or_insert(key, || {
        println!("  Computing value for '{}'...", key);
        42
    });
    println!("  get_or_insert result = {}", value);

    // Second call should use cached value
    let value2 = cache.get_or_insert(key, || {
        panic!("Should not be called!");
    });
    println!("  cached result = {}", value2);
}

#[trace_borrow]
fn cache_resize() {
    let mut cache = LruCache::new(NonZeroUsize::new(5).unwrap());
    for i in 0..5 {
        cache.put(i, i * 100);
    }
    println!("  before resize: len = {}", cache.len());

    // Resize smaller — evicts entries
    cache.resize(NonZeroUsize::new(2).unwrap());
    println!("  after resize(2): len = {}", cache.len());

    // Remaining entries should be the most recently used
    let remaining: Vec<(&i32, &i32)> = cache.iter().collect();
    println!("  remaining = {:?}", remaining);
}

fn main() {
    println!("=== BorrowScope Battle Test: lru crate ===\n");

    println!("--- Test 1: Basic Operations ---");
    reset();
    basic_cache_operations();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 2: String Ownership ---");
    reset();
    cache_with_strings();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 3: Iteration ---");
    reset();
    cache_iteration();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 4: Promote/Demote ---");
    reset();
    cache_promote_and_demote();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 5: Get or Insert ---");
    reset();
    cache_get_or_insert();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    println!("--- Test 6: Resize ---");
    reset();
    cache_resize();
    let events = get_events();
    println!("  → {} events tracked\n", events.len());

    // Final summary
    println!("=== Summary ===");
    reset();
    basic_cache_operations();
    cache_with_strings();
    cache_iteration();
    cache_promote_and_demote();
    cache_get_or_insert();
    cache_resize();
    let all_events = get_events();
    println!("Total events across all tests: {}", all_events.len());
    print_summary();
}
