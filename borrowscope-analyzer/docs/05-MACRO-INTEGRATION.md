## 5. Integration with borrowscope-macro

The type information produced by the analyzer enables `borrowscope-macro` to make informed decisions during code transformation. This section describes the integration architecture and the enhanced tracking capabilities it enables.

### Type Information Lookup

When the `#[trace_borrow]` macro processes a function, it needs to determine the appropriate tracking function for each variable binding. With the analyzer's output available, the macro can perform precise lookups using function context and declaration order:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MACRO TYPE LOOKUP FLOW (v2.2)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Source Code                    Macro Processing                            │
│  ───────────                    ────────────────                            │
│                                                                             │
│  #[trace_borrow]                                                            │
│  fn example() {                 ┌─────────────────────────────────┐         │
│      let data = Rc::new(x); ──▶ │ 1. Set function context: "example"│        │
│      ...                        │ 2. Lookup in by_function index   │         │
│  }                              │    key: ("example", "data", 0)   │         │
│                                 │ 3. Find: initializer_kind="rc_new"│        │
│                                 │ 4. Emit: track_rc_new_with_id(   │         │
│                                 │          id, "data", type, loc,  │         │
│                                 │          Rc::new(x))             │         │
│                                 └─────────────────────────────────┘         │
│                                                                             │
│  type-info.json                                                             │
│  ──────────────                                                             │
│  {                                                                          │
│    "by_function": {             ◄─── Primary lookup index (v2.2)            │
│      "example": {                                                           │
│        "data": [{                                                           │
│          "name": "data",                                                    │
│          "function_name": "example",                                        │
│          "decl_index": 0,       ◄─── Disambiguates shadowed vars            │
│          "initializer_kind": "rc_new",  ◄─── Determines tracking fn         │
│          "is_rc": true                                                      │
│        }]                                                                   │
│      }                                                                      │
│    },                                                                       │
│    "by_name": {                 ◄─── Fallback index                         │
│      "data": [...]                                                          │
│    }                                                                        │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The macro uses a two-tier lookup strategy:

1. **Primary**: `lookup_in_function(fn_name, var_name, decl_index)` - Uses the `by_function` index for precise matching
2. **Fallback**: `lookup_by_name(var_name)` - Uses the `by_name` index when function context is unavailable

This approach handles variable shadowing correctly:

```rust
#[trace_borrow]
fn shadowing_example() {
    let x = 1;           // decl_index: 0, type: i32
    let x = "hello";     // decl_index: 1, type: &str  
    let x = vec![1, 2];  // decl_index: 2, type: Vec<i32>
    let x = Rc::new(x);  // decl_index: 3, type: Rc<Vec<i32>>
}
```

Each `x` is correctly identified by its `decl_index`, allowing the macro to select the appropriate tracking function for each.

### Enhanced Tracking Function Selection

With complete type information including `initializer_kind`, the macro can select the most appropriate tracking function for each variable. The decision is now based on semantic analysis rather than heuristics:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 TRACKING FUNCTION SELECTION LOGIC (v2.2)                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Given: VariableTypeInfo for binding                                        │
│                                                                             │
│  // === Primary: Use initializer_kind for precise tracking ===              │
│                                                                             │
│  match initializer_kind:                                                    │
│      "rc_new"           → track_rc_new_with_id(...)                         │
│      "rc_clone"         → track_rc_clone_with_id(...)                       │
│      "arc_new"          → track_arc_new_with_id(...)                        │
│      "arc_clone"        → track_arc_clone_with_id(...)                      │
│      "box_new"          → track_box_new(...)                                │
│      "refcell_new"      → track_refcell_new(...)                            │
│      "refcell_borrow"   → track_refcell_borrow(...)                         │
│      "refcell_borrow_mut" → track_refcell_borrow_mut(...)                   │
│      "cell_new"         → track_cell_new(...)                               │
│      "mutex_lock"       → track_lock_guard_acquire(...)                     │
│      "channel_new"      → track_channel(...)                                │
│      "weak_new"         → track_weak_new(...)                               │
│      "pin_new"          → track_pin_new(...)                                │
│      "cow_borrowed"     → track_cow_borrowed(...)                           │
│      "cow_owned"        → track_cow_owned(...)                              │
│      "ref"              → track_borrow_with_id(...)                         │
│      "ref_mut"          → track_borrow_mut_with_id(...)                     │
│                                                                             │
│  // === Fallback: Use type flags for generic initializers ===               │
│                                                                             │
│  if is_rc:              → track_rc_new_with_id(...)                         │
│  else if is_arc:        → track_arc_new_with_id(...)                        │
│  else if is_box:        → track_box_new(...)                                │
│  else if is_refcell:    → track_refcell_new(...)                            │
│  else if is_cell:       → track_cell_new(...)                               │
│  else if is_raw_ptr:    → track_raw_ptr_create(...)                         │
│                                                                             │
│  // === Default: Generic tracking ===                                       │
│                                                                             │
│  else:                  → track_new_with_id(...)                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The `initializer_kind` field enables precise tracking even for type aliases and factory functions:

```rust
type MyRc<T> = Rc<T>;

fn example() {
    // Without initializer_kind: macro sees "MyRc::new" - doesn't match "Rc::new"
    // With initializer_kind: analyzer resolves type to Rc, sets is_rc=true
    let x = MyRc::new(42);  // Correctly tracked as Rc
    
    // Factory function returning Rc
    let y = create_shared(42);  // initializer_kind="call", but is_rc=true
                                // Falls back to type-based tracking
}
```

### Fallback Behavior

The macro must handle cases where type information is unavailable. This occurs when:

1. The analyzer has not been run on the project
2. The source file was modified after analysis
3. The binding location doesn't match any entry (line number drift)

The fallback strategy preserves backward compatibility:

```rust
fn get_tracking_call(binding: &LetBinding, type_info: Option<&VariableTypeInfo>) -> TokenStream {
    match type_info {
        Some(info) => {
            // Use precise type information
            select_tracking_function(info)
        }
        None => {
            // Fall back to syntactic heuristics (current behavior)
            infer_from_syntax(binding)
        }
    }
}
```

When falling back, the macro logs a warning indicating that type information was not found, encouraging users to run the analyzer for complete tracking accuracy.

### Benefits Over Syntactic Analysis

The integration provides several concrete improvements:

**Accurate Smart Pointer Detection**: Any expression that evaluates to `Rc<T>` is correctly identified, regardless of how it was constructed. Factory functions, conditional expressions, and match arms all resolve to their actual types.

**Copy Semantics**: The runtime can now distinguish between ownership transfers and copies. This is essential for accurate visualization—a copy creates a new independent value, while a move transfers the original.

**Nested Type Awareness**: A type like `Arc<Mutex<Vec<String>>>` has multiple classification flags set, allowing the runtime to track all relevant aspects: atomic reference counting, mutex locking, vector operations, and string allocations.

**User-Defined Types**: While the analyzer cannot automatically classify user-defined smart pointers, the full type string is available. Future versions could support user-provided classification rules or trait-based detection.

**Closure Types**: Closures have opaque types like `impl Fn(i32) -> i32`. The analyzer captures these types, enabling tracking of closure creation and potential capture analysis.

### Runtime Event Enhancement

With type information available, the runtime events become more informative:

```json
// Without type info
{"event": "new", "name": "data", "type": "unknown"}

// With type info
{"event": "rc_new", "name": "data", "type": "Rc<RefCell<Vec<i32, Global>>, Global>", 
 "is_copy": false, "ref_count": 1}
```

The enhanced events enable richer visualization and analysis. A visualization tool can render reference-counted pointers differently from owned values, show interior mutability boundaries, and accurately depict copy vs move semantics.

---

