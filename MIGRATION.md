# Migration Guide: v0.1 → v0.2

## Breaking Change: Analyzer Now Required

Starting with v0.2, `borrowscope-analyzer` must be run before building any project that uses `#[trace_borrow]`. The macro will **panic at compile time** with a clear error message if the analyzer output is missing.

### Old Workflow (v0.1.x)

```bash
cargo build
```

The macro used heuristic fallbacks when analyzer data was unavailable.

### New Workflow (v0.2.0)

```bash
# Step 1: Run the analyzer (generates .borrowscope/type-info.json)
cargo run -p borrowscope-analyzer -- /path/to/your/project

# Step 2: Build your project
cargo build
```

### Why This Change?

- **Zero heuristics** — all 109 tracking patterns are now fully semantic
- **Correct Copy vs Move tracking** — no more misclassification
- **Accurate method borrow detection** — `&self` vs `&mut self` vs `self` from rust-analyzer
- **Simpler codebase** — ~220 lines of dead heuristic code removed

## New Tracking Capabilities in v0.2

### Enriched Events

| Event | New Fields | Source |
|-------|-----------|--------|
| `AwaitStart` | `live_variables: Vec<String>` | Variables live across `.await` |
| `UnsafeBlockEnter` | `operation_kind`, `operation_context` | Unsafe operation classification |
| `ClosureCreate` | `fn_trait: Option<String>` | `Fn`/`FnMut`/`FnOnce` |
| `MatchArm` | `bindings: Vec<String>` | Pattern binding names |
| `Call` | `receiver_type`, `result_type` | Method call type metadata |
| `Drop` | `location: Option<String>` | Precise drop location from analyzer |

### New Runtime Functions

| Function | Purpose |
|----------|---------|
| `track_await_start_with_live_vars` | Await with live variable tracking |
| `track_unsafe_block_enter_enriched` | Unsafe block with operation metadata |
| `track_closure_create_with_trait` | Closure with Fn trait info |
| `track_match_arm_with_bindings` | Match arm with binding names |
| `track_method_call` | Method call with receiver/result types |
| `track_drop_at` | Drop with precise location |
| `track_atomic_new` | Atomic type creation |
| `track_duration_new` | Duration creation |
| `track_instant_new` | Instant creation |
| `track_autoref` | Implicit autoref tracking |
| `track_autoderef` | Implicit autoderef tracking |
| `track_var_read` | Variable read tracking |
| `track_var_write` | Variable write tracking |
| `track_borrow_span` | Borrow span metadata |
| `track_destructure` | Pattern destructuring |
| `track_variant_construct` | Enum variant construction |

### Re-enabled Features

- **Field access tracking** — `track_field_access` now works for read-only accesses (gated by analyzer `field_accesses` data to avoid lvalue issues)
- **Deref tracking** — `track_deref` re-enabled for rvalue derefs when analyzer confirms deref adjustments

## Backward Compatibility

All new `Event` fields use `#[serde(default, skip_serializing_if)]` — existing JSON consumers will not break. New fields are simply absent in older event streams.

## Troubleshooting

### "ERROR: BorrowScope analyzer output not found"

Run the analyzer first:
```bash
cargo run -p borrowscope-analyzer -- .
```

### Analyzer output is stale

Re-run the analyzer after code changes:
```bash
cargo run -p borrowscope-analyzer -- .
cargo build
```

### `.borrowscope/` directory

Add to `.gitignore`:
```
.borrowscope/
```

The analyzer output is machine-generated and should not be committed.
