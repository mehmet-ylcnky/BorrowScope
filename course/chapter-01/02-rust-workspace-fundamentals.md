# Section 2: Rust Workspace Fundamentals

## Learning Objectives

By the end of this section, you will:
- Understand the difference between packages, crates, and workspaces
- Know when and why to use workspaces
- Grasp how workspace members share dependencies
- Understand workspace dependency resolution
- Be ready to create the BorrowScope workspace structure

## Prerequisites

- Completed Section 1
- Basic Cargo knowledge (cargo new, cargo build)
- Understanding of Cargo.toml files

---

## The Rust Module System Hierarchy

Before diving into workspaces, let's clarify Rust's organizational concepts:

```
Workspace (borrowscope/)
    ├── Package (borrowscope-macro/)
    │   └── Crate (lib or bin)
    │       └── Modules (mod)
    ├── Package (borrowscope-runtime/)
    │   └── Crate (lib)
    └── Package (borrowscope-cli/)
        └── Crate (bin)
```

### Key Definitions

**Module:** A namespace for organizing code within a crate
```rust
mod tracker {
    pub fn track() { }
}
```

**Crate:** A compilation unit (library or binary)
- Library crate: `lib.rs`
- Binary crate: `main.rs`

**Package:** A Cargo project with a `Cargo.toml`
- Can contain multiple crates
- Usually contains one library crate and/or multiple binary crates

**Workspace:** A collection of packages that share:
- A common `Cargo.lock`
- A common output directory (`target/`)
- Dependency resolution

---

## Why Workspaces?

### The Problem Without Workspaces

Imagine building BorrowScope without a workspace:

```
borrowscope-macro/
    ├── Cargo.toml
    ├── Cargo.lock
    └── target/

borrowscope-runtime/
    ├── Cargo.toml
    ├── Cargo.lock
    └── target/

borrowscope-cli/
    ├── Cargo.toml
    ├── Cargo.lock
    └── target/
```

**Issues:**
1. **Duplicate dependencies** - Each crate downloads its own copy of `serde`
2. **Inconsistent versions** - `macro` uses `serde 1.0.150`, `runtime` uses `1.0.152`
3. **Wasted disk space** - Three separate `target/` directories
4. **Slower builds** - Can't share compiled dependencies
5. **Harder testing** - Can't run all tests with one command

### The Solution: Workspaces

```
borrowscope/                    # Workspace root
    ├── Cargo.toml              # Workspace manifest
    ├── Cargo.lock              # Shared lock file
    ├── target/                 # Shared build directory
    ├── borrowscope-macro/
    │   ├── Cargo.toml          # Member manifest
    │   └── src/
    ├── borrowscope-runtime/
    │   ├── Cargo.toml
    │   └── src/
    └── borrowscope-cli/
        ├── Cargo.toml
        └── src/
```

**Benefits:**
1. ✅ **Shared dependencies** - One copy of `serde` for all
2. ✅ **Version consistency** - Workspace enforces same versions
3. ✅ **Efficient builds** - Shared `target/` directory
4. ✅ **Unified commands** - `cargo test --workspace`
5. ✅ **Easier development** - Work on multiple crates simultaneously

---

## Workspace Structure Deep Dive

### The Root Cargo.toml

The workspace root contains a special `Cargo.toml`:

```toml
[workspace]
members = [
    "borrowscope-macro",
    "borrowscope-runtime",
    "borrowscope-cli",
    "borrowscope-ui/src-tauri",
]

# Workspace-wide settings
[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/borrowscope"

# Shared dependencies across all members
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Key points:**
- `[workspace]` section defines it as a workspace
- `members` lists all packages in the workspace
- No `[package]` section (this isn't a package itself)
- Optional `[workspace.package]` for shared metadata
- Optional `[workspace.dependencies]` for version management

### Member Cargo.toml

Each member has its own `Cargo.toml`:

```toml
# borrowscope-runtime/Cargo.toml
[package]
name = "borrowscope-runtime"
version.workspace = true        # Inherit from workspace
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true          # Use workspace version
serde_json.workspace = true
petgraph = "0.6"                # Member-specific dependency

[dev-dependencies]
criterion = "0.5"
```

**Key points:**
- Has its own `[package]` section
- Can inherit metadata with `.workspace = true`
- Can use workspace dependencies with `.workspace = true`
- Can add member-specific dependencies

---

## Dependency Resolution in Workspaces

### How Cargo Resolves Dependencies

When you run `cargo build` in a workspace:

1. **Collect all dependencies** from all members
2. **Resolve versions** using the workspace `Cargo.lock`
3. **Build once** - Each dependency compiled only once
4. **Share artifacts** - All members use the same compiled libraries

### Example: Shared Dependency

```toml
# Workspace Cargo.toml
[workspace.dependencies]
serde = "1.0"

# borrowscope-macro/Cargo.toml
[dependencies]
serde.workspace = true

# borrowscope-runtime/Cargo.toml
[dependencies]
serde.workspace = true
```

**Result:** Both crates use the exact same version of `serde`, compiled once.

### Example: Version Conflicts

What if members need different versions?

```toml
# borrowscope-macro/Cargo.toml
[dependencies]
syn = "2.0"

# borrowscope-cli/Cargo.toml
[dependencies]
syn = "1.0"  # Older version needed for some reason
```

**Cargo's solution:** Both versions are included, but this is a code smell. Workspaces encourage consistency.

---

## Workspace Commands

### Building

```bash
# Build all workspace members
cargo build --workspace

# Build specific member
cargo build -p borrowscope-macro

# Build with all features
cargo build --workspace --all-features
```

### Testing

```bash
# Test all members
cargo test --workspace

# Test specific member
cargo test -p borrowscope-runtime

# Test with output
cargo test --workspace -- --nocapture
```

### Running

```bash
# Run a binary from workspace
cargo run -p borrowscope-cli

# Run with arguments
cargo run -p borrowscope-cli -- visualize src/main.rs
```

### Other Commands

```bash
# Check all members (fast compile check)
cargo check --workspace

# Format all code
cargo fmt --all

# Lint all code
cargo clippy --workspace

# Generate docs for all members
cargo doc --workspace --no-deps

# Clean shared target directory
cargo clean
```

---

## Inter-Crate Dependencies

### How Members Depend on Each Other

Members can depend on other workspace members:

```toml
# borrowscope-cli/Cargo.toml
[dependencies]
borrowscope-macro = { path = "../borrowscope-macro" }
borrowscope-runtime = { path = "../borrowscope-runtime" }
```

**Key points:**
- Use `path` to reference workspace members
- Changes to `borrowscope-runtime` automatically rebuild `borrowscope-cli`
- Enables tight integration between crates

### Dependency Graph

```
borrowscope-cli
    ├── depends on → borrowscope-macro
    ├── depends on → borrowscope-runtime
    └── depends on → clap

borrowscope-macro
    ├── depends on → syn
    ├── depends on → quote
    └── depends on → borrowscope-runtime (for types)

borrowscope-runtime
    ├── depends on → serde
    ├── depends on → serde_json
    └── depends on → petgraph
```

---

## Best Practices for Workspaces

### 1. Organize by Responsibility

```
borrowscope/
    ├── borrowscope-macro/      # Procedural macro
    ├── borrowscope-runtime/    # Core logic
    ├── borrowscope-cli/        # User interface
    └── borrowscope-ui/         # Visualization
```

Each crate has a single, clear purpose.

### 2. Minimize Inter-Crate Dependencies

**Good:**
```
cli → runtime
macro → runtime
```

**Bad:**
```
cli ↔ macro (circular dependency)
runtime → cli (wrong direction)
```

### 3. Use Workspace Dependencies

Define common dependencies once:

```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
```

Members inherit them:

```toml
[dependencies]
serde.workspace = true
tokio.workspace = true
```

### 4. Consistent Versioning

All members should share the same version:

```toml
[workspace.package]
version = "0.1.0"
```

```toml
# Each member
[package]
version.workspace = true
```

### 5. Shared Metadata

```toml
[workspace.package]
edition = "2021"
authors = ["Your Name"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/repo"
```

---

## Common Pitfalls

### Pitfall 1: Forgetting to Add Members

```toml
[workspace]
members = [
    "borrowscope-macro",
    # Forgot to add borrowscope-runtime!
]
```

**Symptom:** `cargo build` doesn't build all crates.

**Solution:** Always add new crates to `members`.

### Pitfall 2: Mixing Workspace and Non-Workspace

```toml
# DON'T DO THIS in workspace root
[workspace]
members = ["crate-a"]

[package]  # ❌ Root can't be both workspace and package
name = "root"
```

**Solution:** Workspace root should only have `[workspace]`, not `[package]`.

### Pitfall 3: Inconsistent Dependency Versions

```toml
# crate-a/Cargo.toml
[dependencies]
serde = "1.0.150"

# crate-b/Cargo.toml
[dependencies]
serde = "1.0.160"
```

**Problem:** Two versions of `serde` in the workspace.

**Solution:** Use `[workspace.dependencies]`.

### Pitfall 4: Circular Dependencies

```toml
# crate-a/Cargo.toml
[dependencies]
crate-b = { path = "../crate-b" }

# crate-b/Cargo.toml
[dependencies]
crate-a = { path = "../crate-a" }  # ❌ Circular!
```

**Solution:** Refactor to remove circular dependency, often by extracting shared code to a third crate.

---

## Workspace vs. Single Package

### When to Use a Workspace

✅ Multiple related crates  
✅ Shared dependencies  
✅ Need to develop multiple crates together  
✅ Want unified testing/building  
✅ Large project with logical separation  

### When to Use a Single Package

✅ Small, focused project  
✅ Single library or binary  
✅ No need for separation  
✅ Simplicity is priority  

### BorrowScope Needs a Workspace Because:

1. **Four distinct components** - macro, runtime, CLI, UI
2. **Different crate types** - proc-macro, lib, bin
3. **Shared dependencies** - serde, etc.
4. **Tight integration** - CLI uses macro and runtime
5. **Unified development** - Test everything together

---

## Real-World Examples

### Example 1: Tokio

```
tokio/
    ├── tokio/              # Main runtime
    ├── tokio-util/         # Utilities
    ├── tokio-stream/       # Stream utilities
    └── tokio-macros/       # Procedural macros
```

### Example 2: Serde

```
serde/
    ├── serde/              # Core traits
    ├── serde_derive/       # Derive macros
    └── serde_test/         # Testing utilities
```

### Example 3: Rust Compiler

```
rust/
    ├── compiler/rustc/
    ├── library/std/
    ├── library/core/
    └── tools/cargo/
```

---

## Understanding Cargo.lock

### What is Cargo.lock?

A file that records the exact versions of all dependencies:

```toml
# Cargo.lock (simplified)
[[package]]
name = "serde"
version = "1.0.193"
dependencies = [...]

[[package]]
name = "borrowscope-runtime"
version = "0.1.0"
dependencies = [
    "serde 1.0.193",
]
```

### Workspace Cargo.lock

In a workspace, there's **one** `Cargo.lock` at the root:

```
borrowscope/
    ├── Cargo.lock          # ✅ One lock file
    ├── borrowscope-macro/
    │   └── (no Cargo.lock)
    └── borrowscope-runtime/
        └── (no Cargo.lock)
```

**Benefits:**
- Consistent versions across all members
- Reproducible builds
- Easier dependency management

### Should You Commit Cargo.lock?

**For applications (binaries):** ✅ Yes
- Ensures reproducible builds
- Users get exact versions you tested

**For libraries:** ❌ Usually no
- Downstream users have their own lock files
- Avoids version conflicts

**For BorrowScope:** ✅ Yes (it's an application)

---

## Practical Exercise: Exploring Workspaces

Let's explore an existing workspace to understand the structure.

### Step 1: Clone a Workspace Project

```bash
# Clone tokio (a well-structured workspace)
git clone https://github.com/tokio-rs/tokio.git
cd tokio
```

### Step 2: Examine the Structure

```bash
# View workspace members
cat Cargo.toml | grep -A 10 "\[workspace\]"

# List all members
ls -d */

# Check a member's Cargo.toml
cat tokio/Cargo.toml
```

### Step 3: Build and Test

```bash
# Build entire workspace
cargo build --workspace

# Test specific member
cargo test -p tokio

# Check what gets built
ls target/debug/
```

### Questions to Answer:

1. How many members does the tokio workspace have?
2. What dependencies are shared?
3. How do members depend on each other?
4. What's in the shared `target/` directory?

---

## Preparing for BorrowScope Workspace

### Our Workspace Structure

```
borrowscope/
    ├── Cargo.toml                      # Workspace root
    ├── Cargo.lock                      # Shared lock file
    ├── .gitignore                      # Ignore target/, etc.
    ├── README.md                       # Project documentation
    ├── LICENSE-MIT                     # License files
    ├── LICENSE-APACHE
    │
    ├── borrowscope-macro/              # Procedural macro crate
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs
    │   └── tests/
    │
    ├── borrowscope-runtime/            # Runtime tracking crate
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs
    │   └── tests/
    │
    ├── borrowscope-cli/                # CLI binary crate
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── main.rs
    │   └── tests/
    │
    └── borrowscope-ui/                 # Tauri UI (later)
        └── src-tauri/
            ├── Cargo.toml
            └── src/
```

### Dependency Relationships

```
borrowscope-cli
    ├── borrowscope-macro (path dependency)
    ├── borrowscope-runtime (path dependency)
    └── clap (external)

borrowscope-macro
    ├── syn (external)
    ├── quote (external)
    └── borrowscope-runtime (path, for types)

borrowscope-runtime
    ├── serde (external)
    ├── serde_json (external)
    └── petgraph (external)
```

---

## Key Takeaways

### Workspace Fundamentals

1. **Workspaces** organize multiple related packages
2. **Members** share dependencies and build artifacts
3. **One Cargo.lock** ensures version consistency
4. **Workspace commands** operate on all members
5. **Path dependencies** link members together

### Why Workspaces for BorrowScope

1. **Four distinct crates** with different purposes
2. **Shared dependencies** (serde, etc.)
3. **Tight integration** between components
4. **Unified development** workflow
5. **Professional structure** for a real tool

### Best Practices

1. ✅ Use `[workspace.dependencies]` for shared deps
2. ✅ Inherit metadata with `.workspace = true`
3. ✅ Minimize circular dependencies
4. ✅ One clear purpose per crate
5. ✅ Commit Cargo.lock for applications

---

## Exercises

### Exercise 1: Workspace Analysis

Create a diagram showing:
1. The four BorrowScope crates
2. Dependencies between them
3. External dependencies for each

### Exercise 2: Dependency Planning

List which external crates each BorrowScope member will need:

**borrowscope-macro:**
- ?
- ?

**borrowscope-runtime:**
- ?
- ?

**borrowscope-cli:**
- ?
- ?

### Exercise 3: Hands-On Exploration

Create a minimal workspace:

```bash
mkdir test-workspace
cd test-workspace

# Create workspace root
cat > Cargo.toml << 'EOF'
[workspace]
members = ["crate-a", "crate-b"]
EOF

# Create member crates
cargo new --lib crate-a
cargo new --lib crate-b

# Make crate-b depend on crate-a
# (Edit crate-b/Cargo.toml)

# Build and test
cargo build --workspace
cargo test --workspace
```

---

## Further Reading

### Official Documentation

1. **Cargo Book - Workspaces**
   - https://doc.rust-lang.org/cargo/reference/workspaces.html

2. **Cargo Book - Dependency Resolution**
   - https://doc.rust-lang.org/cargo/reference/resolver.html

3. **Rust Book - Packages and Crates**
   - https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html

### Real-World Examples

1. **Tokio Workspace**
   - https://github.com/tokio-rs/tokio

2. **Serde Workspace**
   - https://github.com/serde-rs/serde

3. **Rust Analyzer Workspace**
   - https://github.com/rust-lang/rust-analyzer

### Advanced Topics

1. **Workspace Inheritance RFC**
   - https://rust-lang.github.io/rfcs/2906-cargo-workspace-deduplicate.html

2. **Cargo Resolver Version 2**
   - https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions

---

## Reflection Questions

Before moving to Section 3, ensure you can answer:

✅ What's the difference between a package, crate, and workspace?  
✅ Why does BorrowScope need a workspace?  
✅ How do workspace members share dependencies?  
✅ What's the purpose of Cargo.lock in a workspace?  
✅ How do you build/test specific workspace members?  

---

## What's Next?

In **Section 3: Setting Up the Workspace**, we'll:
- Create the actual BorrowScope workspace
- Set up all four member crates
- Configure shared dependencies
- Establish the project structure

Get ready to write your first code! 🚀

---

**Previous Section:** [01-understanding-the-project-scope.md](./01-understanding-the-project-scope.md)  
**Next Section:** [03-setting-up-the-workspace.md](./03-setting-up-the-workspace.md)

**Chapter Progress:** 2/8 sections complete ⬛⬜⬜⬜⬜⬜⬜

---

*"Good architecture is not about the code you write, but about the structure you create."*
