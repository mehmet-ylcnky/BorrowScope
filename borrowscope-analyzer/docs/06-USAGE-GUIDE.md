## 6. Usage Guide

### 6.1 Running the Analyzer

The borrowscope-analyzer is a standalone binary that analyzes a Rust project and produces type information. It requires the project path as its only argument:

```bash
# From the BorrowScope workspace
cargo run -p borrowscope-analyzer -- /path/to/your/project

# Or if installed
borrowscope-analyzer /path/to/your/project
```

The analyzer expects a valid Cargo project with a `Cargo.toml` at the specified path. It will:

1. Load the workspace using rust-analyzer's infrastructure
2. Discover the Rust sysroot for standard library type resolution
3. Analyze all `.rs` files in the project (excluding dependencies and target/)
4. Write results to `.borrowscope/type-info.json`

Example output:

```
BorrowScope Analyzer v0.1.0
═══════════════════════════════════════════
Project: /home/user/my-project

  Loading workspace...
  Analyzing: src/main.rs
  Analyzing: src/lib.rs
  Analyzing: src/utils.rs

═══════════════════════════════════════════
Summary:
  Files analyzed: 3
  Variables found: 47
  Types resolved: 47 (100.0%)

Output: /home/user/my-project/.borrowscope/type-info.json
```

The analyzer logs progress to stderr and can be configured with the `RUST_LOG` environment variable for detailed debugging:

```bash
RUST_LOG=debug cargo run -p borrowscope-analyzer -- /path/to/project
```

### 6.2 Workflow for Users

The complete workflow for using BorrowScope with full type information involves three steps:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      COMPLETE USER WORKFLOW                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  STEP 1: Analyze                                                            │
│  ───────────────                                                            │
│                                                                             │
│  $ borrowscope-analyzer .                                                   │
│                                                                             │
│  Creates: .borrowscope/type-info.json                                       │
│                                                                             │
│                         │                                                   │
│                         ▼                                                   │
│                                                                             │
│  STEP 2: Build                                                              │
│  ────────────                                                               │
│                                                                             │
│  $ cargo build                                                              │
│                                                                             │
│  The #[trace_borrow] macro reads type-info.json during expansion            │
│  and generates accurate tracking calls.                                     │
│                                                                             │
│                         │                                                   │
│                         ▼                                                   │
│                                                                             │
│  STEP 3: Run                                                                │
│  ─────────                                                                  │
│                                                                             │
│  $ cargo run                                                                │
│                                                                             │
│  Runtime tracking captures ownership events with full type information.     │
│  Export to JSON for visualization.                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Step 1: Analyze**

Run the analyzer whenever your code changes significantly. The analyzer needs to re-run when:

- New variables are added
- Variable types change
- Files are added or renamed
- Line numbers shift significantly (due to added/removed code)

For development workflows, consider running the analyzer as part of your build process or IDE save hook.

**Step 2: Build**

Build your project normally with `cargo build` or `cargo run`. The `#[trace_borrow]` macro will automatically detect and use the type information file if present. No code changes are required—the macro transparently upgrades its tracking precision when type information is available.

**Step 3: Run and Analyze**

Execute your instrumented program. The runtime tracking will now include accurate type information in its events. Export the events for analysis:

```rust
use borrowscope_runtime::*;

fn main() {
    reset();
    
    // Your instrumented code runs here
    my_function();
    
    // Export events with full type information
    let events = get_events();
    std::fs::write(
        "trace.json",
        serde_json::to_string_pretty(&events).unwrap()
    ).unwrap();
}
```

### Automation with Build Scripts

For projects that want automatic analysis, a `build.rs` script can invoke the analyzer:

```rust
// build.rs
use std::process::Command;

fn main() {
    // Re-run if any Rust source changes
    println!("cargo:rerun-if-changed=src/");
    
    // Run the analyzer
    let status = Command::new("borrowscope-analyzer")
        .arg(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .status();
    
    if let Err(e) = status {
        println!("cargo:warning=borrowscope-analyzer not found: {}", e);
        println!("cargo:warning=Type information will not be available");
    }
}
```

This ensures type information is always up-to-date, though it adds to build time. For large projects, consider running the analyzer only in CI or as a separate development step.

### Verifying Type Information

To verify the analyzer is working correctly, inspect the generated JSON:

```bash
# Check that the file was created
cat .borrowscope/type-info.json | head -50

# Count variables by type
cat .borrowscope/type-info.json | jq '.files[].[] | .ty' | sort | uniq -c | sort -rn

# Find all Rc variables
cat .borrowscope/type-info.json | jq '.files[][] | select(.is_rc == true) | .name'

# Check resolution rate
cat .borrowscope/type-info.json | jq '[.files[][]] | length as $total | [.files[][] | select(.ty != "unknown")] | length as $resolved | "\($resolved)/\($total) types resolved"'
```

A 100% resolution rate indicates the analyzer successfully determined types for all variables. Lower rates may indicate files outside the crate graph or analysis errors—check the analyzer's stderr output for warnings.

---

