# Section 6: Rust Toolchain Configuration

## Learning Objectives

By the end of this section, you will:
- Understand Rust toolchains and their components
- Configure rust-toolchain.toml for consistent builds
- Set and enforce MSRV (Minimum Supported Rust Version)
- Configure rustfmt and clippy in detail
- Understand Rust editions and their impact
- Set up toolchain overrides for different scenarios

## Prerequisites

- Completed Section 5 (CI/CD setup)
- Rust installed via rustup
- Understanding of Cargo.toml basics

---

## Understanding Rust Toolchains

### What is a Toolchain?

A **toolchain** is a complete Rust installation including:
- `rustc` - The Rust compiler
- `cargo` - Package manager and build tool
- `rustdoc` - Documentation generator
- Standard library
- Optional components (rustfmt, clippy, etc.)

### Toolchain Channels

**Stable:**
- Released every 6 weeks
- Guaranteed stability
- Production-ready
- Example: `1.75.0`

**Beta:**
- Next stable release (6 weeks ahead)
- Testing ground
- Example: `1.76.0-beta`

**Nightly:**
- Bleeding edge features
- Updated daily
- Unstable features
- Example: `nightly-2024-01-15`

### Why Toolchain Configuration Matters

**Without configuration:**
```
Developer A: Rust 1.75.0 ✅ Compiles
Developer B: Rust 1.70.0 ❌ Doesn't compile
CI: Rust 1.76.0 ⚠️ New warnings
```

**With configuration:**
```
Everyone uses: Rust 1.75.0 ✅ Consistent
```

---

## Step 1: Create rust-toolchain.toml

Create `rust-toolchain.toml` in the project root:

```toml
[toolchain]
# Specify the Rust channel
channel = "stable"

# Specify exact version (optional, but recommended)
# channel = "1.75.0"

# Required components
components = [
    "rustfmt",
    "clippy",
    "rust-src",
]

# Optional components (don't fail if unavailable)
# targets = ["wasm32-unknown-unknown"]

# Profile (minimal, default, complete)
profile = "default"
```

### Understanding Each Field

#### `channel`

```toml
channel = "stable"
```

**Options:**
- `"stable"` - Latest stable release
- `"1.75.0"` - Specific version (recommended)
- `"beta"` - Beta channel
- `"nightly"` - Nightly channel
- `"nightly-2024-01-15"` - Specific nightly

**Recommendation for BorrowScope:**
```toml
channel = "1.75.0"
```

Pin to specific version for reproducibility.

#### `components`

```toml
components = [
    "rustfmt",    # Code formatter
    "clippy",     # Linter
    "rust-src",   # Source code (for rust-analyzer)
]
```

**Common components:**
- `rustfmt` - Format code
- `clippy` - Lint code
- `rust-src` - IDE support
- `rust-analyzer` - LSP server
- `llvm-tools-preview` - Profiling tools
- `miri` - Interpreter for detecting UB

#### `profile`

```toml
profile = "default"
```

**Options:**
- `minimal` - Just rustc and cargo
- `default` - Includes rustdoc and rust-std
- `complete` - Everything

**Recommendation:** `default` (good balance)

---

## Step 2: Set MSRV (Minimum Supported Rust Version)

### In Cargo.toml

Update workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV
```

### Why MSRV Matters

**Scenario:**
```rust
// This feature requires Rust 1.75+
let Some(x) = option else { return };
```

**Without MSRV:**
- Users with Rust 1.70 get cryptic errors
- No clear minimum version

**With MSRV:**
- Cargo checks version before building
- Clear error message if too old

### Choosing an MSRV

**Factors to consider:**
1. **Features needed** - What Rust features do you use?
2. **Dependencies** - What do your deps require?
3. **User base** - How old are their Rust installations?
4. **Maintenance** - How often will you update?

**For BorrowScope:**
```toml
rust-version = "1.75"
```

**Reasoning:**
- Stable features we need
- Not too old (released ~3 months ago)
- Reasonable for users to update

### Testing MSRV in CI

Add to `.github/workflows/ci.yml`:

```yaml
msrv:
  name: MSRV Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust MSRV
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.75.0"
    
    - name: Check MSRV
      run: cargo check --workspace
```

---

## Step 3: Detailed Rustfmt Configuration

Update `.rustfmt.toml` with comprehensive settings:

```toml
# Rustfmt Configuration for BorrowScope

# ===== Edition =====
edition = "2021"

# ===== Line Width =====
max_width = 100
comment_width = 80
wrap_comments = true

# ===== Indentation =====
tab_spaces = 4
hard_tabs = false

# ===== Imports =====
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true

# ===== Functions =====
fn_single_line = false
fn_args_layout = "Tall"
brace_style = "SameLineWhere"

# ===== Control Flow =====
control_brace_style = "AlwaysSameLine"
match_arm_blocks = true
match_block_trailing_comma = true

# ===== Structs and Enums =====
struct_lit_single_line = false
enum_discrim_align_threshold = 20

# ===== Chains =====
chain_width = 60
use_small_heuristics = "Default"

# ===== Strings =====
format_strings = true
format_macro_matchers = true
format_macro_bodies = true

# ===== Comments =====
normalize_comments = true
normalize_doc_attributes = true

# ===== Whitespace =====
newline_style = "Unix"
remove_nested_parens = true
spaces_around_ranges = false
trailing_comma = "Vertical"
trailing_semicolon = true

# ===== Misc =====
use_field_init_shorthand = true
use_try_shorthand = true
force_explicit_abi = true
```

### Key Formatting Rules Explained

#### Imports Grouping

```rust
// Before
use std::io;
use serde::Serialize;
use std::fs;
use crate::tracker;

// After (with group_imports = "StdExternalCrate")
use std::{fs, io};

use serde::Serialize;

use crate::tracker;
```

#### Function Arguments

```toml
fn_args_layout = "Tall"
```

```rust
// Tall layout
fn long_function_name(
    first_arg: String,
    second_arg: i32,
    third_arg: bool,
) -> Result<()> {
    // ...
}
```

#### Brace Style

```toml
brace_style = "SameLineWhere"
```

```rust
// Same line for simple cases
fn simple() {
    // ...
}

// New line for where clauses
fn generic<T>()
where
    T: Clone,
{
    // ...
}
```

#### Trailing Commas

```toml
trailing_comma = "Vertical"
```

```rust
// Vertical: add comma
let array = [
    1,
    2,
    3,  // ← Comma added
];

// Horizontal: no comma
let array = [1, 2, 3];
```

---

## Step 4: Detailed Clippy Configuration

Update `.clippy.toml`:

```toml
# Clippy Configuration for BorrowScope

# ===== Complexity =====
cognitive-complexity-threshold = 30
type-complexity-threshold = 250

# ===== Documentation =====
missing-docs-in-crate-items = true
doc-valid-idents = ["BorrowScope", "rustc", "AST", "MIR"]

# ===== Functions =====
too-many-arguments-threshold = 7
too-many-lines-threshold = 150

# ===== Naming =====
single-char-binding-names-threshold = 4
enum-variant-name-threshold = 3

# ===== Performance =====
vec-box-size-threshold = 4096

# ===== Style =====
max-trait-bounds = 5
```

### Clippy Lint Levels in Cargo.toml

Update workspace `Cargo.toml`:

```toml
[workspace.lints.rust]
# Forbid unsafe code (can't be overridden)
unsafe_code = "forbid"

# Deny these (error, but can be overridden)
missing_docs = "deny"
missing_debug_implementations = "deny"

# Warn on these
unused_imports = "warn"
unused_variables = "warn"
dead_code = "warn"

[workspace.lints.clippy]
# Enable all default lints
all = "warn"

# Enable pedantic lints
pedantic = "warn"

# Enable nursery (experimental) lints
nursery = "warn"

# Enable cargo lints
cargo = "warn"

# Deny specific important lints
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

# Allow some pedantic lints that are too strict
too_many_lines = "allow"
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

### Understanding Lint Groups

**`clippy::all`** - Default lints
```rust
// Warns on common mistakes
let x = 5;
if x = 6 { }  // ⚠️ Should be ==
```

**`clippy::pedantic`** - Extra strict
```rust
// Warns on style issues
pub fn foo() -> Result<i32, Error> {
    Ok(42)  // ⚠️ Could use must_use attribute
}
```

**`clippy::nursery`** - Experimental
```rust
// Warns on potential issues
let mut x = vec![1, 2, 3];
x.push(4);  // ⚠️ Could use with_capacity
```

**`clippy::cargo`** - Cargo.toml issues
```toml
# ⚠️ Warns on duplicate dependencies
[dependencies]
serde = "1.0"
serde_json = "1.0"  # Already includes serde
```

### Specific Lint Explanations

#### `unwrap_used = "deny"`

```rust
// ❌ Forbidden
let value = option.unwrap();

// ✅ Use proper error handling
let value = option.expect("value should exist");
let value = option?;
let value = option.unwrap_or_default();
```

**Why:** `unwrap()` can panic. Use explicit error handling.

#### `missing_docs = "deny"`

```rust
// ❌ Missing documentation
pub fn track_new<T>(name: &str, value: T) -> T {
    value
}

// ✅ Documented
/// Track a new variable creation
///
/// # Arguments
/// * `name` - Variable name
/// * `value` - Variable value
///
/// # Returns
/// The value unchanged (zero-cost abstraction)
pub fn track_new<T>(name: &str, value: T) -> T {
    value
}
```

---

## Step 5: Configure Rust Editions

### Understanding Editions

**Rust editions** are opt-in releases with breaking changes:

- **2015** - Original Rust
- **2018** - Async/await, NLL, module system improvements
- **2021** - Disjoint capture in closures, panic macros, IntoIterator

### Setting Edition

In `Cargo.toml`:

```toml
[workspace.package]
edition = "2021"
```

All members inherit:

```toml
[package]
edition.workspace = true
```

### Edition-Specific Features

**2021 Edition:**

```rust
// Disjoint closure captures
let tuple = (String::from("hello"), String::from("world"));
let closure = || {
    println!("{}", tuple.0);  // Only captures tuple.0, not whole tuple
};
println!("{}", tuple.1);  // ✅ Can still use tuple.1
```

**2018 Edition:**

```rust
// Would capture entire tuple
let closure = || {
    println!("{}", tuple.0);
};
println!("{}", tuple.1);  // ❌ Error: tuple moved
```

---

## Step 6: Toolchain Overrides

### Per-Directory Overrides

For specific directories needing different toolchains:

```bash
# Use nightly for a specific crate
cd borrowscope-macro
rustup override set nightly
```

Creates `.rust-toolchain` file in that directory.

### Temporary Overrides

```bash
# Use specific toolchain for one command
cargo +nightly build
cargo +1.75.0 test
cargo +beta clippy
```

### When to Use Overrides

**Nightly features:**
```rust
// borrowscope-macro might need nightly for proc-macro features
#![feature(proc_macro_span)]
```

**Testing compatibility:**
```bash
# Test with beta to prepare for next release
cargo +beta test
```

---

## Step 7: Component Management

### Installing Components

```bash
# Install rustfmt
rustup component add rustfmt

# Install clippy
rustup component add clippy

# Install rust-src (for IDE)
rustup component add rust-src

# Install rust-analyzer
rustup component add rust-analyzer
```

### Checking Installed Components

```bash
rustup component list
```

Output:
```
cargo-x86_64-unknown-linux-gnu (installed)
clippy-x86_64-unknown-linux-gnu (installed)
rustc-x86_64-unknown-linux-gnu (installed)
rustfmt-x86_64-unknown-linux-gnu (installed)
rust-src (installed)
```

### Updating Toolchain

```bash
# Update to latest stable
rustup update stable

# Update all toolchains
rustup update
```

---

## Step 8: IDE Integration

### VS Code Configuration

Create `.vscode/settings.json`:

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [
    "--all-targets",
    "--all-features"
  ],
  "rust-analyzer.rustfmt.extraArgs": [
    "+nightly"
  ],
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  }
}
```

### IntelliJ/RustRover Configuration

Create `.idea/runConfigurations/`:

```xml
<!-- cargo_check.xml -->
<component name="ProjectRunConfigurationManager">
  <configuration default="false" name="Cargo Check" type="CargoCommandRunConfiguration">
    <option name="command" value="check --workspace" />
  </configuration>
</component>
```

---

## Step 9: Verify Configuration

### Check Toolchain

```bash
# Show active toolchain
rustc --version
cargo --version

# Show toolchain info
rustup show
```

Expected output:
```
active toolchain
----------------
1.75.0-x86_64-unknown-linux-gnu (overridden by '/path/to/borrowscope/rust-toolchain.toml')
rustc 1.75.0 (82e1608df 2023-12-21)
```

### Test Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Test Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Fix auto-fixable issues
cargo clippy --fix --all-targets --all-features
```

### Test MSRV

```bash
# Check with MSRV
cargo +1.75.0 check --workspace
```

---

## Step 10: Document Toolchain Requirements

Update `README.md`:

```markdown
## Requirements

- **Rust:** 1.75.0 or later
- **Components:** rustfmt, clippy, rust-src

### Installation

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt clippy rust-src

# Verify installation
rustc --version  # Should be 1.75.0 or later
```

### Building

The project uses `rust-toolchain.toml` to automatically select the correct Rust version.

```bash
cargo build --workspace
```
```

---

## Best Practices Summary

### Toolchain Configuration

✅ **Pin specific version** - Use `channel = "1.75.0"`  
✅ **Set MSRV** - Use `rust-version = "1.75"`  
✅ **Include components** - rustfmt, clippy, rust-src  
✅ **Use default profile** - Good balance  
✅ **Test MSRV in CI** - Ensure compatibility  

### Code Quality

✅ **Configure rustfmt** - Consistent style  
✅ **Configure clippy** - Catch mistakes  
✅ **Deny unwrap** - Explicit error handling  
✅ **Require docs** - Document public APIs  
✅ **Use 2021 edition** - Latest features  

### Development

✅ **IDE integration** - Format on save  
✅ **Pre-commit checks** - Catch issues early  
✅ **Document requirements** - Clear setup  
✅ **Update regularly** - Stay current  

---

## Common Issues and Solutions

### Issue 1: Toolchain Not Found

**Error:**
```
error: toolchain '1.75.0' is not installed
```

**Solution:**
```bash
rustup install 1.75.0
```

### Issue 2: Component Missing

**Error:**
```
error: component 'rustfmt' is not available
```

**Solution:**
```bash
rustup component add rustfmt
```

### Issue 3: MSRV Check Fails

**Error:**
```
error: package requires rustc 1.75.0 or newer
```

**Solution:**
```bash
rustup update
# or
rustup install 1.75.0
```

### Issue 4: Conflicting Toolchains

**Problem:** Different toolchains in different directories

**Solution:**
```bash
# Remove overrides
rustup override unset

# Use workspace toolchain
cd borrowscope
rustup show
```

---

## Exercises

### Exercise 1: Verify Toolchain

```bash
# Check your Rust version
rustc --version

# Check active toolchain
rustup show

# List installed components
rustup component list | grep installed
```

### Exercise 2: Test Formatting

```bash
# Intentionally break formatting
echo "fn test(){let x=5;}" >> borrowscope-runtime/src/lib.rs

# Check formatting
cargo fmt --all -- --check

# Fix formatting
cargo fmt --all

# Verify
git diff
```

### Exercise 3: Test Clippy

```bash
# Add a clippy warning
echo "pub fn test() { let x = 5; }" >> borrowscope-runtime/src/lib.rs

# Run clippy
cargo clippy

# Fix the warning
# (remove unused variable)
```

---

## Key Takeaways

### Toolchain Fundamentals

✅ **Toolchain** = rustc + cargo + std + components  
✅ **Channels** = stable, beta, nightly  
✅ **rust-toolchain.toml** = Pin version for consistency  
✅ **MSRV** = Minimum Supported Rust Version  
✅ **Components** = rustfmt, clippy, rust-src  

### Configuration Files

✅ **rust-toolchain.toml** - Toolchain version  
✅ **.rustfmt.toml** - Formatting rules  
✅ **.clippy.toml** - Lint configuration  
✅ **Cargo.toml** - Lint levels, MSRV  
✅ **.vscode/settings.json** - IDE integration  

### Benefits

✅ **Consistency** - Everyone uses same version  
✅ **Reproducibility** - Builds work everywhere  
✅ **Quality** - Automated formatting and linting  
✅ **Documentation** - Clear requirements  
✅ **IDE support** - Better development experience  

---

## Further Reading

### Official Documentation

1. **Rustup Book**
   - https://rust-lang.github.io/rustup/

2. **Rust Edition Guide**
   - https://doc.rust-lang.org/edition-guide/

3. **Rustfmt Configuration**
   - https://rust-lang.github.io/rustfmt/

4. **Clippy Lints**
   - https://rust-lang.github.io/rust-clippy/

### Tools

1. **cargo-msrv** - Find minimum Rust version
2. **cargo-outdated** - Check for outdated dependencies
3. **cargo-audit** - Security auditing

---

## Reflection Questions

Before moving to Section 7, ensure you can answer:

✅ What is a Rust toolchain?  
✅ Why pin a specific Rust version?  
✅ What is MSRV and why does it matter?  
✅ What do rustfmt and clippy do?  
✅ How do you override the toolchain temporarily?  

---

## What's Next?

In **Section 7: Project Documentation Structure**, we'll:
- Set up comprehensive documentation
- Configure rustdoc
- Create API documentation
- Write user guides
- Set up mdBook for tutorials

---

**Previous Section:** [05-ci-cd-pipeline-basics.md](./05-ci-cd-pipeline-basics.md)  
**Next Section:** [07-project-documentation-structure.md](./07-project-documentation-structure.md)

**Chapter Progress:** 6/8 sections complete ⬛⬛⬛⬛⬛⬜⬜

---

*"The right tools make all the difference." 🔧*
