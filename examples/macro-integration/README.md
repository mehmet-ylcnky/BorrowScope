# Macro Integration Example

This example demonstrates how `borrowscope-macro` integrates with `borrowscope-analyzer` to use semantic type information.

## Prerequisites

Build the analyzer:
```bash
cd /path/to/borrowscope
cargo build -p borrowscope-analyzer
```

## Usage

**Step 1: Run the analyzer to generate type info**
```bash
../../target/debug/borrowscope-analyzer .
```

This creates `.borrowscope/type-info.json` with semantic type information.

**Step 2: Build and run**
```bash
cargo run
```

## What This Demonstrates

1. **Semantic Type Detection** - The macro uses type info from the analyzer instead of heuristics
2. **Name-Based Lookup** - Variables are matched by name since stable Rust doesn't expose span location
3. **Fallback to Heuristics** - If type info isn't available, heuristics are used

## Verifying It Works

Check the expanded macro output:
```bash
cargo expand
```

You should see:
- `track_rc_new_with_id` with the actual type string from analyzer
- `track_arc_new_with_id` with the actual type string from analyzer
- Correct tracking calls based on semantic analysis

## Type Info Schema

The `.borrowscope/type-info.json` uses schema v2.1 with a `by_name` index:
```json
{
  "version": "2.1",
  "by_name": {
    "rc_data": [{ "is_rc": true, "ty": "Rc<Vec<i32>>", ... }],
    "arc_data": [{ "is_arc": true, "ty": "Arc<String>", ... }]
  }
}
```
