# Section 3: Setting Up the Workspace

## Learning Objectives

By the end of this section, you will:
- Create the BorrowScope workspace from scratch
- Set up all four member crates with proper configuration
- Configure shared dependencies and metadata
- Understand the purpose of each configuration option
- Have a working, compilable workspace structure

## Prerequisites

- Completed Sections 1-2
- Rust and Cargo installed (1.75+)
- Terminal/command line access
- Text editor or IDE ready

---

## Step 1: Create the Workspace Root

Let's start building! Open your terminal and navigate to where you want to create the project.

### Create the Root Directory

```bash
# Create and enter the project directory
mkdir borrowscope
cd borrowscope
```

### Initialize Git (Optional but Recommended)

```bash
git init
```

**Why now?** Starting with git from the beginning makes it easy to track changes and experiment safely.

### Create the Workspace Cargo.toml

Create a file named `Cargo.toml` in the root:

```bash
touch Cargo.toml
```

Open it in your editor and add:

```toml
[workspace]
resolver = "2"
members = [
    "borrowscope-macro",
    "borrowscope-runtime",
    "borrowscope-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Your Name <your.email@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/borrowscope"
homepage = "https://github.com/yourusername/borrowscope"
documentation = "https://docs.rs/borrowscope"
keywords = ["rust", "ownership", "visualization", "borrow-checker"]
categories = ["development-tools", "visualization"]

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Procedural macro dependencies
syn = { version = "2.0", features = ["full", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"

# CLI
clap = { version = "4.4", features = ["derive"] }

# Graph data structures
petgraph = "0.6"

# Concurrency
parking_lot = "0.12"
lazy_static = "1.4"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Understanding Each Section

Let's break down what each part does:

#### `[workspace]`

```toml
[workspace]
resolver = "2"
members = [...]
```

- **`resolver = "2"`**: Uses Cargo's newer dependency resolver (more accurate, handles features better)
- **`members`**: Lists all crates in the workspace

#### `[workspace.package]`

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
```

Shared metadata that all members can inherit:

- **`version`**: Semantic versioning (major.minor.patch)
- **`edition`**: Rust edition (2021 is latest stable)
- **`rust-version`**: Minimum Supported Rust Version (MSRV)
- **`authors`**: Who maintains this project
- **`license`**: Dual MIT/Apache-2.0 (Rust standard)
- **`repository`**: GitHub URL
- **`keywords`**: For crates.io search
- **`categories`**: For crates.io organization

#### `[workspace.dependencies]`

```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
```

Centralized dependency versions. Members can reference these with:

```toml
[dependencies]
serde.workspace = true
```

**Benefits:**
- One place to update versions
- Guaranteed consistency
- Easier maintenance

#### `[profile.dev]` and `[profile.release]`

```toml
[profile.dev]
opt-level = 0      # No optimization (fast compile)
debug = true       # Include debug symbols

[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
codegen-units = 1  # Better optimization, slower compile
strip = true       # Remove debug symbols (smaller binary)
```

These control compilation behavior for development vs release builds.

---

## Step 2: Create the Macro Crate

### Generate the Crate

```bash
cargo new --lib borrowscope-macro
```

This creates:
```
borrowscope-macro/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Configure as Procedural Macro

Edit `borrowscope-macro/Cargo.toml`:

```toml
[package]
name = "borrowscope-macro"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Procedural macros for BorrowScope ownership tracking"

# This is a procedural macro crate
[lib]
proc-macro = true

[dependencies]
syn.workspace = true
quote.workspace = true
proc-macro2.workspace = true

[dev-dependencies]
# For testing macros
trybuild = "1.0"
```

### Understanding the Configuration

#### `[lib]`

```toml
[lib]
proc-macro = true
```

**Critical:** This tells Cargo this is a procedural macro crate, not a regular library.

**What this means:**
- Can only export procedural macros
- Cannot export regular functions or types
- Compiled specially by rustc
- Used at compile-time, not runtime

#### Inheriting Workspace Metadata

```toml
version.workspace = true
edition.workspace = true
```

Instead of duplicating values, we inherit from `[workspace.package]`.

### Initialize the Macro Code

Edit `borrowscope-macro/src/lib.rs`:

```rust
use proc_macro::TokenStream;

/// Attribute macro to track ownership and borrowing
///
/// # Example
///
/// ```ignore
/// #[trace_borrow]
/// fn example() {
///     let s = String::from("hello");
///     let r = &s;
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the input unchanged
    // We'll implement the actual transformation later
    item
}
```

**What this does:**
- Defines an attribute macro `#[trace_borrow]`
- Currently does nothing (returns input unchanged)
- We'll implement the real logic in Chapter 2

### Test It Compiles

```bash
cargo build -p borrowscope-macro
```

You should see:
```
   Compiling borrowscope-macro v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
```

---

## Step 3: Create the Runtime Crate

### Generate the Crate

```bash
cargo new --lib borrowscope-runtime
```

### Configure the Runtime

Edit `borrowscope-runtime/Cargo.toml`:

```toml
[package]
name = "borrowscope-runtime"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Runtime tracking and graph building for BorrowScope"

[dependencies]
serde.workspace = true
serde_json.workspace = true
petgraph.workspace = true
parking_lot.workspace = true
lazy_static.workspace = true

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "tracking_benchmark"
harness = false
```

### Understanding the Dependencies

**Core dependencies:**
- **`serde`**: Serialization framework
- **`serde_json`**: JSON serialization
- **`petgraph`**: Graph data structures
- **`parking_lot`**: Fast, efficient locks
- **`lazy_static`**: Global state initialization

**Dev dependencies:**
- **`criterion`**: Benchmarking framework

### Initialize the Runtime Code

Edit `borrowscope-runtime/src/lib.rs`:

```rust
//! Runtime tracking for BorrowScope
//!
//! This crate provides the runtime API for tracking ownership and borrowing events.

use serde::{Deserialize, Serialize};

/// An ownership or borrowing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Variable created
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },
    /// Variable borrowed
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
    },
    /// Ownership moved
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
    },
    /// Variable dropped
    Drop {
        timestamp: u64,
        var_id: String,
    },
}

/// Track a new variable
///
/// Returns the value unchanged (zero-cost abstraction)
#[inline(always)]
pub fn track_new<T>(name: &str, value: T) -> T {
    // TODO: Implement tracking
    // For now, just return the value
    value
}

/// Track an immutable borrow
#[inline(always)]
pub fn track_borrow<T>(name: &str, value: &T) -> &T {
    // TODO: Implement tracking
    value
}

/// Track a mutable borrow
#[inline(always)]
pub fn track_borrow_mut<T>(name: &str, value: &mut T) -> &mut T {
    // TODO: Implement tracking
    value
}

/// Track a variable drop
pub fn track_drop(name: &str) {
    // TODO: Implement tracking
}

/// Export tracking data to JSON
pub fn export_json(path: &str) -> Result<(), std::io::Error> {
    // TODO: Implement export
    Ok(())
}

/// Reset tracking state (useful for tests)
pub fn reset() {
    // TODO: Implement reset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_new_returns_value() {
        let s = track_new("s", String::from("hello"));
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_track_borrow_returns_reference() {
        let s = String::from("hello");
        let r = track_borrow("r", &s);
        assert_eq!(r, "hello");
    }
}
```

**What this does:**
- Defines the `Event` enum (our data model)
- Provides stub implementations of tracking functions
- Includes basic tests
- Uses `#[inline(always)]` for zero-cost abstractions

### Test It Compiles

```bash
cargo build -p borrowscope-runtime
cargo test -p borrowscope-runtime
```

You should see:
```
   Compiling borrowscope-runtime v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 3.21s
     Running unittests src/lib.rs

running 2 tests
test tests::test_track_new_returns_value ... ok
test tests::test_track_borrow_returns_reference ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Step 4: Create the CLI Crate

### Generate the Crate

```bash
cargo new borrowscope-cli
```

**Note:** No `--lib` flag because this is a binary crate.

### Configure the CLI

Edit `borrowscope-cli/Cargo.toml`:

```toml
[package]
name = "borrowscope-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Command-line interface for BorrowScope"

# This is a binary crate
[[bin]]
name = "cargo-borrowscope"
path = "src/main.rs"

[dependencies]
# Workspace members
borrowscope-macro = { path = "../borrowscope-macro" }
borrowscope-runtime = { path = "../borrowscope-runtime" }

# External dependencies
clap.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
```

### Understanding the Binary Configuration

```toml
[[bin]]
name = "cargo-borrowscope"
path = "src/main.rs"
```

**Why `cargo-borrowscope`?**

Cargo has a convention: binaries named `cargo-*` can be invoked as:
```bash
cargo borrowscope <args>
```

Instead of:
```bash
cargo-borrowscope <args>
```

This makes it feel like a native Cargo subcommand!

### Initialize the CLI Code

Edit `borrowscope-cli/src/main.rs`:

```rust
//! BorrowScope CLI - Visualize Rust ownership and borrowing

use clap::{Parser, Subcommand};
use anyhow::Result;

/// BorrowScope - Visualize Rust ownership and borrowing
#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Borrowscope(BorrowScopeCli),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct BorrowScopeCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Visualize ownership in a single file
    Visualize {
        /// Path to the Rust file
        file: String,
    },
    /// Run and visualize the entire project
    Run,
    /// Export ownership data to JSON
    Export {
        /// Output file path
        #[arg(short, long, default_value = "borrowscope-output.json")]
        output: String,
    },
}

fn main() -> Result<()> {
    let CargoCli::Borrowscope(cli) = CargoCli::parse();

    match cli.command {
        Commands::Visualize { file } => {
            println!("Visualizing: {}", file);
            println!("(Not implemented yet - coming in Chapter 7!)");
        }
        Commands::Run => {
            println!("Running project...");
            println!("(Not implemented yet - coming in Chapter 7!)");
        }
        Commands::Export { output } => {
            println!("Exporting to: {}", output);
            println!("(Not implemented yet - coming in Chapter 7!)");
        }
    }

    Ok(())
}
```

**What this does:**
- Defines CLI structure with clap
- Implements cargo subcommand pattern
- Provides three subcommands: visualize, run, export
- Stub implementations (we'll fill these in later)

### Test It Compiles and Runs

```bash
cargo build -p borrowscope-cli
cargo run -p borrowscope-cli -- borrowscope --help
```

You should see:
```
BorrowScope - Visualize Rust ownership and borrowing

Usage: cargo borrowscope <COMMAND>

Commands:
  visualize  Visualize ownership in a single file
  run        Run and visualize the entire project
  export     Export ownership data to JSON
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Try a command:
```bash
cargo run -p borrowscope-cli -- borrowscope visualize test.rs
```

Output:
```
Visualizing: test.rs
(Not implemented yet - coming in Chapter 7!)
```

Perfect! The CLI structure is working.

---

## Step 5: Create Supporting Files

### .gitignore

Create `.gitignore` in the root:

```bash
cat > .gitignore << 'EOF'
# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# BorrowScope output
borrowscope-output/
*.borrowscope.json
EOF
```

### README.md

Create `README.md`:

```markdown
# BorrowScope

> Visualize Rust's ownership and borrowing in real-time

## Status

🚧 **Under Development** - Following the BorrowScope Development Course

## What is BorrowScope?

BorrowScope is a developer tool that makes Rust's ownership and borrowing system visible through interactive visualizations.

## Project Structure

```
borrowscope/
├── borrowscope-macro/      # Procedural macros
├── borrowscope-runtime/    # Event tracking
├── borrowscope-cli/        # Command-line interface
└── borrowscope-ui/         # Visualization (coming soon)
```

## Development

```bash
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Run the CLI
cargo run -p borrowscope-cli -- borrowscope --help
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
```

### License Files

Create `LICENSE-MIT`:

```bash
cat > LICENSE-MIT << 'EOF'
MIT License

Copyright (c) 2025 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
```

Create `LICENSE-APACHE`:

```bash
cat > LICENSE-APACHE << 'EOF'
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

[Full Apache 2.0 license text - truncated for brevity]
EOF
```

---

## Step 6: Verify the Complete Structure

### Check Directory Structure

```bash
tree -L 2 -I target
```

You should see:
```
borrowscope/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── borrowscope-macro/
│   ├── Cargo.toml
│   └── src/
├── borrowscope-runtime/
│   ├── Cargo.toml
│   └── src/
└── borrowscope-cli/
    ├── Cargo.toml
    └── src/
```

### Build Everything

```bash
cargo build --workspace
```

Expected output:
```
   Compiling proc-macro2 v1.0.70
   Compiling unicode-ident v1.0.12
   Compiling syn v2.0.39
   Compiling quote v1.0.33
   Compiling serde v1.0.193
   ...
   Compiling borrowscope-macro v0.1.0
   Compiling borrowscope-runtime v0.1.0
   Compiling borrowscope-cli v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 45.23s
```

### Test Everything

```bash
cargo test --workspace
```

Expected output:
```
   Compiling borrowscope-runtime v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs (target/debug/deps/borrowscope_runtime-...)

running 2 tests
test tests::test_track_new_returns_value ... ok
test tests::test_track_borrow_returns_reference ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Check the CLI

```bash
cargo run -p borrowscope-cli -- borrowscope --version
```

Output:
```
cargo-borrowscope 0.1.0
```

---

## Step 7: Commit Your Work

If you initialized git:

```bash
git add .
git commit -m "Initial workspace setup with three crates

- borrowscope-macro: Procedural macro crate (stub)
- borrowscope-runtime: Event tracking and graph building (stub)
- borrowscope-cli: Command-line interface (stub)

All crates compile and basic tests pass."
```

---

## Understanding What We Built

### The Workspace

```toml
[workspace]
members = ["borrowscope-macro", "borrowscope-runtime", "borrowscope-cli"]
```

Three crates working together as one project.

### Dependency Flow

```
borrowscope-cli
    ├── depends on → borrowscope-macro
    └── depends on → borrowscope-runtime

borrowscope-macro
    └── (no internal dependencies yet)

borrowscope-runtime
    └── (no internal dependencies)
```

### What Each Crate Does

**borrowscope-macro:**
- Type: Procedural macro library
- Purpose: Transform code at compile-time
- Current state: Stub that returns input unchanged

**borrowscope-runtime:**
- Type: Regular library
- Purpose: Track events and build graphs
- Current state: Stub functions that do nothing

**borrowscope-cli:**
- Type: Binary
- Purpose: User interface for the tool
- Current state: Parses commands but doesn't execute

---

## Common Issues and Solutions

### Issue 1: "proc-macro = true" Error

**Error:**
```
error: `proc-macro` crate types currently cannot export any items other than functions tagged with `#[proc_macro]`
```

**Solution:** Make sure `borrowscope-macro/src/lib.rs` only exports procedural macros, not regular functions or types.

### Issue 2: Workspace Member Not Found

**Error:**
```
error: package `borrowscope-macro` is not a member of the workspace
```

**Solution:** Check that the member is listed in the root `Cargo.toml`:
```toml
[workspace]
members = ["borrowscope-macro", ...]
```

### Issue 3: Dependency Version Mismatch

**Error:**
```
error: failed to select a version for `serde`
```

**Solution:** Use workspace dependencies:
```toml
# Root Cargo.toml
[workspace.dependencies]
serde = "1.0"

# Member Cargo.toml
[dependencies]
serde.workspace = true
```

### Issue 4: Binary Name Confusion

**Problem:** `cargo borrowscope` doesn't work

**Solution:** Binary must be named `cargo-borrowscope`:
```toml
[[bin]]
name = "cargo-borrowscope"
```

---

## Best Practices Applied

### 1. Workspace Organization

✅ Clear separation of concerns  
✅ Logical dependency flow  
✅ Shared configuration  

### 2. Naming Conventions

✅ `borrowscope-*` prefix for all crates  
✅ `cargo-borrowscope` for the binary  
✅ Descriptive crate names  

### 3. Documentation

✅ README.md with project overview  
✅ Inline documentation comments  
✅ License files  

### 4. Version Control

✅ .gitignore configured  
✅ Initial commit made  
✅ Clear commit messages  

---

## Exercises

### Exercise 1: Explore the Structure

Run these commands and understand the output:

```bash
# Show workspace members
cargo metadata --format-version 1 | grep -A 5 workspace_members

# Show dependency tree
cargo tree -p borrowscope-cli

# Show what gets compiled
cargo build --workspace -v | grep Compiling
```

### Exercise 2: Add a Test

Add a new test to `borrowscope-runtime/src/lib.rs`:

```rust
#[test]
fn test_track_borrow_mut_returns_reference() {
    let mut s = String::from("hello");
    let r = track_borrow_mut("r", &mut s);
    r.push_str(" world");
    assert_eq!(r, "hello world");
}
```

Run it:
```bash
cargo test -p borrowscope-runtime
```

### Exercise 3: Modify the CLI

Add a new subcommand to `borrowscope-cli/src/main.rs`:

```rust
Commands::Init => {
    println!("Initializing BorrowScope configuration...");
    println!("(Not implemented yet)");
}
```

Test it:
```bash
cargo run -p borrowscope-cli -- borrowscope init
```

---

## Key Takeaways

### What We Accomplished

✅ Created a complete Cargo workspace  
✅ Set up three crates with proper configuration  
✅ Established dependency relationships  
✅ Verified everything compiles and tests pass  
✅ Created supporting files (README, licenses, .gitignore)  

### Rust Concepts Practiced

1. **Workspace configuration** - `[workspace]` section
2. **Procedural macro crates** - `proc-macro = true`
3. **Path dependencies** - Linking workspace members
4. **Cargo subcommands** - `cargo-*` naming convention
5. **Metadata inheritance** - `.workspace = true`

### What's Next

In the next sections, we'll:
- Set up CI/CD with GitHub Actions
- Configure development tools
- Add more comprehensive testing
- Start implementing real functionality

---

## Further Reading

### Official Documentation

1. **Cargo Workspaces**
   - https://doc.rust-lang.org/cargo/reference/workspaces.html

2. **Procedural Macros**
   - https://doc.rust-lang.org/reference/procedural-macros.html

3. **Cargo Subcommands**
   - https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands

### Example Projects

1. **Tokio Workspace Setup**
   - https://github.com/tokio-rs/tokio/blob/master/Cargo.toml

2. **Clap CLI Examples**
   - https://github.com/clap-rs/clap/tree/master/examples

---

## Reflection Questions

Before moving to Section 4, ensure you can answer:

✅ What's the purpose of each crate in our workspace?  
✅ How do workspace members depend on each other?  
✅ Why is `borrowscope-macro` a special crate type?  
✅ How does the `cargo-borrowscope` naming enable subcommands?  
✅ What's the benefit of `[workspace.dependencies]`?  

---

## Congratulations! 🎉

You've built the foundation of BorrowScope! You now have:
- A properly structured Cargo workspace
- Three crates that compile successfully
- A working CLI skeleton
- Version control set up
- Professional project organization

This is the foundation we'll build upon for the next 200+ sections!

---

**Previous Section:** [02-rust-workspace-fundamentals.md](./02-rust-workspace-fundamentals.md)  
**Next Section:** [04-git-and-version-control-setup.md](./04-git-and-version-control-setup.md)

**Chapter Progress:** 3/8 sections complete ⬛⬛⬜⬜⬜⬜⬜

---

*"A journey of a thousand miles begins with a single step. You've taken three!" 🚀*
