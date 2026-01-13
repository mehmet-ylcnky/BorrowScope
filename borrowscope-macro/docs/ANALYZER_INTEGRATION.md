# Macro Integration with Analyzer Type Info

This document describes how `borrowscope-macro` integrates with `borrowscope-analyzer` to use semantic type information instead of heuristic-based detection.

## The Problem

The `#[trace_borrow]` macro needs to know the type of each variable to generate appropriate tracking calls (e.g., `track_rc_new` for `Rc<T>`, `track_arc_new` for `Arc<T>`).

### Original Approach: Heuristics
The macro originally used pattern matching on expressions:
- `Rc::new(...)` → detected as Rc
- `Arc::clone(...)` → detected as Arc clone
- etc.

**Limitations:**
- Can't detect type aliases: `type MyRc<T> = Rc<T>;`
- Can't detect re-exports: `use my_crate::Rc;`
- Can't detect trait implementations (Copy, Clone, Send, Sync)
- Relies on string matching, which is fragile

### New Approach: Semantic Analysis
The `borrowscope-analyzer` uses rust-analyzer to get actual type information:
- Resolves all type aliases
- Detects trait implementations via `impls_trait`
- Uses canonical paths for ADT classification
- Outputs structured JSON with full type info

## The Challenge: Bridging Analyzer and Macro

```
┌─────────────────────────────────────────────────────────────────┐
│  borrowscope-analyzer (runs before build)                       │
│  ─────────────────────────────────────────                      │
│  - Has full semantic analysis via rust-analyzer                 │
│  - Outputs: .borrowscope/type-info.json                         │
│  - Knows: file, line, column, type, traits for each variable    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  type-info.json (v2.1 schema)                                   │
│  ─────────────────────────────                                  │
│  {                                                              │
│    "version": "2.1",                                            │
│    "files": { "src/main.rs": [...] },                           │
│    "by_name": {                                                 │
│      "rc_data": [{ is_rc: true, is_clone: true, ... }],         │
│      "counter": [{ is_primitive: true, is_copy: true, ... }]    │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  borrowscope-macro (runs during cargo build)                    │
│  ───────────────────────────────────────────                    │
│  - Sees AST, can extract variable names                         │
│  - On stable Rust: CANNOT get file/line/column from span        │
│  - Needs to lookup type info somehow                            │
└─────────────────────────────────────────────────────────────────┘
```

### The Stable Rust Limitation

On stable Rust, `proc_macro::Span` does not expose:
- File path
- Line number  
- Column number

These are only available with `#![feature(proc_macro_span)]` on nightly.

## Our Solution: Name-Based Lookup

Since we can extract variable names from the AST, we use name-based lookup:

```rust
// In type_info.rs
pub fn lookup_by_name(var_name: &str) -> Option<&'static VariableTypeInfo> {
    let entries = TYPE_INFO.by_name.get(var_name)?;
    if entries.len() == 1 {
        Some(&entries[0])  // Unique match
    } else if all_same_classification(entries) {
        Some(&entries[0])  // All have same type info
    } else {
        None  // Ambiguous - fall back to heuristics
    }
}
```

### Handling Ambiguity

When multiple variables share the same name:

1. **Same type info** → Use it (e.g., two `i` loop counters, both `i32`)
2. **Different type info** → Fall back to heuristics

## Workflow

```bash
# Step 1: Run analyzer to generate type-info.json
borrowscope-analyzer .

# Step 2: Build with macro (reads type-info.json)
cargo build
```

The macro automatically loads `.borrowscope/type-info.json` at compile time.

## Schema v2.1

The `by_name` index was added in schema v2.1:

```json
{
  "version": "2.1",
  "analyzer_version": "0.1.0",
  "files": {
    "src/main.rs": [
      { "name": "rc_data", "line": 10, "is_rc": true, ... }
    ]
  },
  "by_name": {
    "rc_data": [
      { "name": "rc_data", "line": 10, "is_rc": true, ... }
    ]
  }
}
```

## Limitations

1. **Requires running analyzer first** - If `type-info.json` doesn't exist, falls back to heuristics
2. **Ambiguous names** - Variables with same name but different types fall back to heuristics
3. **Stale data** - If code changes after analyzer runs, type info may be outdated

## Future Improvements

- **Build script integration** - Run analyzer automatically in `build.rs`
- **Incremental updates** - Only re-analyze changed files
- **Nightly support** - Use `proc_macro_span` for exact location matching when available
