# Section 7: Project Documentation Structure

## Learning Objectives

By the end of this section, you will:
- Understand different types of documentation
- Set up rustdoc for API documentation
- Create comprehensive README files
- Structure user-facing documentation
- Configure documentation generation
- Learn documentation best practices

## Prerequisites

- Completed Section 6 (toolchain configuration)
- Understanding of Markdown syntax
- Familiarity with Rust doc comments

---

## Types of Documentation

### 1. API Documentation (rustdoc)

**Purpose:** Document code for developers using your library

**Format:** Doc comments in source code
```rust
/// Track a new variable creation
///
/// # Arguments
/// * `name` - Variable name
/// * `value` - Variable value
pub fn track_new<T>(name: &str, value: T) -> T {
    value
}
```

### 2. User Documentation (README, guides)

**Purpose:** Help users understand and use the tool

**Format:** Markdown files
- README.md - Project overview
- INSTALL.md - Installation instructions
- USAGE.md - How to use
- CONTRIBUTING.md - How to contribute

### 3. Developer Documentation (architecture, design)

**Purpose:** Help contributors understand the codebase

**Format:** Markdown files in `docs/`
- Architecture overview
- Design decisions
- Development workflow

### 4. Tutorial Documentation (mdBook)

**Purpose:** Teach users step-by-step

**Format:** mdBook (this course!)
- Interactive tutorials
- Examples
- Exercises

---

## Step 1: Configure Rustdoc

### In Cargo.toml

Update workspace `Cargo.toml`:

```toml
[workspace.package]
# ... existing fields ...
documentation = "https://docs.rs/borrowscope"

[workspace.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Create docs/rustdoc.md

```bash
mkdir -p docs
cat > docs/rustdoc.md << 'EOF'
# Rustdoc Configuration

## Building Documentation

```bash
# Build docs for all crates
cargo doc --workspace --no-deps

# Open in browser
cargo doc --workspace --no-deps --open

# Include private items
cargo doc --workspace --no-deps --document-private-items
```

## Documentation Standards

- All public items must have doc comments
- Include examples in doc comments
- Use `# Examples`, `# Panics`, `# Errors` sections
- Link to related items with `[`item`]`
EOF
```

---

## Step 2: Write API Documentation

### Update borrowscope-runtime/src/lib.rs

```rust
//! Runtime tracking for BorrowScope
//!
//! This crate provides the runtime API for tracking ownership and borrowing events
//! in Rust programs. It's designed to be used with the `borrowscope-macro` crate.
//!
//! # Overview
//!
//! BorrowScope tracks four types of events:
//! - **New**: Variable creation
//! - **Borrow**: Reference creation (&T or &mut T)
//! - **Move**: Ownership transfer
//! - **Drop**: Variable going out of scope
//!
//! # Example
//!
//! ```
//! use borrowscope_runtime::{track_new, track_borrow, track_drop};
//!
//! let s = track_new("s", String::from("hello"));
//! let r = track_borrow("r", &s);
//! track_drop("r");
//! track_drop("s");
//! ```
//!
//! # Architecture
//!
//! The tracking system uses a global singleton [`Tracker`] that collects
//! events in a thread-safe manner. Events are stored in memory and can be
//! exported to JSON for visualization.

use serde::{Deserialize, Serialize};

/// An ownership or borrowing event
///
/// This enum represents all possible events that can occur during
/// the lifetime of a variable in Rust.
///
/// # Variants
///
/// - [`Event::New`] - Variable is created
/// - [`Event::Borrow`] - Variable is borrowed (immutably or mutably)
/// - [`Event::Move`] - Ownership is transferred
/// - [`Event::Drop`] - Variable goes out of scope
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::Event;
///
/// let event = Event::New {
///     timestamp: 1,
///     var_name: "s".to_string(),
///     var_id: "s_0x1a2b".to_string(),
///     type_name: "String".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Variable created
    ///
    /// Emitted when a new variable is bound with `let`.
    New {
        /// Monotonic timestamp
        timestamp: u64,
        /// Variable name as it appears in source
        var_name: String,
        /// Unique identifier for this variable
        var_id: String,
        /// Type name (e.g., "String", "i32")
        type_name: String,
    },
    
    /// Variable borrowed
    ///
    /// Emitted when a reference is created with `&` or `&mut`.
    Borrow {
        /// Monotonic timestamp
        timestamp: u64,
        /// Name of the borrowing variable
        borrower_name: String,
        /// Unique identifier for the borrower
        borrower_id: String,
        /// Unique identifier for the owner
        owner_id: String,
        /// Whether this is a mutable borrow
        mutable: bool,
    },
    
    /// Ownership moved
    ///
    /// Emitted when ownership is transferred from one variable to another.
    Move {
        /// Monotonic timestamp
        timestamp: u64,
        /// Unique identifier of the source variable
        from_id: String,
        /// Name of the destination variable
        to_name: String,
        /// Unique identifier of the destination variable
        to_id: String,
    },
    
    /// Variable dropped
    ///
    /// Emitted when a variable goes out of scope and is dropped.
    Drop {
        /// Monotonic timestamp
        timestamp: u64,
        /// Unique identifier of the dropped variable
        var_id: String,
    },
}

/// Track a new variable creation
///
/// This function should be called when a variable is created with `let`.
/// It records a [`Event::New`] event and returns the value unchanged.
///
/// # Arguments
///
/// * `name` - The variable name as it appears in source code
/// * `value` - The value being assigned to the variable
///
/// # Returns
///
/// The `value` parameter, unchanged. This allows the function to be used
/// inline without affecting program semantics.
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::track_new;
///
/// let s = track_new("s", String::from("hello"));
/// assert_eq!(s, "hello");
/// ```
///
/// # Performance
///
/// This function is marked `#[inline(always)]` to ensure zero-cost abstraction.
/// The tracking overhead is minimal (typically <100ns per call).
#[inline(always)]
pub fn track_new<T>(name: &str, value: T) -> T {
    // TODO: Implement tracking
    let _ = name; // Silence unused warning
    value
}

/// Track an immutable borrow
///
/// This function should be called when an immutable reference is created with `&`.
///
/// # Arguments
///
/// * `name` - The name of the borrowing variable
/// * `value` - The reference being created
///
/// # Returns
///
/// The `value` parameter, unchanged.
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::{track_new, track_borrow};
///
/// let s = track_new("s", String::from("hello"));
/// let r = track_borrow("r", &s);
/// assert_eq!(r, "hello");
/// ```
#[inline(always)]
pub fn track_borrow<T>(name: &str, value: &T) -> &T {
    let _ = name;
    value
}

/// Track a mutable borrow
///
/// This function should be called when a mutable reference is created with `&mut`.
///
/// # Arguments
///
/// * `name` - The name of the borrowing variable
/// * `value` - The mutable reference being created
///
/// # Returns
///
/// The `value` parameter, unchanged.
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::{track_new, track_borrow_mut};
///
/// let mut s = track_new("s", String::from("hello"));
/// let r = track_borrow_mut("r", &mut s);
/// r.push_str(" world");
/// assert_eq!(r, "hello world");
/// ```
#[inline(always)]
pub fn track_borrow_mut<T>(name: &str, value: &mut T) -> &mut T {
    let _ = name;
    value
}

/// Track a variable drop
///
/// This function should be called when a variable goes out of scope.
///
/// # Arguments
///
/// * `name` - The name of the variable being dropped
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::{track_new, track_drop};
///
/// {
///     let s = track_new("s", String::from("hello"));
///     // Use s...
///     track_drop("s");
/// } // s actually dropped here
/// ```
pub fn track_drop(name: &str) {
    // TODO: Implement tracking
    let _ = name;
}

/// Export tracking data to JSON
///
/// Writes all collected events to a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the output file
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be created
/// - The data cannot be serialized
/// - Writing to the file fails
///
/// # Examples
///
/// ```no_run
/// use borrowscope_runtime::export_json;
///
/// export_json("output.json").expect("Failed to export");
/// ```
pub fn export_json(path: &str) -> Result<(), std::io::Error> {
    // TODO: Implement export
    let _ = path;
    Ok(())
}

/// Reset tracking state
///
/// Clears all collected events. Useful for testing.
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::{track_new, reset};
///
/// track_new("s", String::from("hello"));
/// reset();
/// // All events cleared
/// ```
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
    
    #[test]
    fn test_track_borrow_mut_returns_reference() {
        let mut s = String::from("hello");
        let r = track_borrow_mut("r", &mut s);
        r.push_str(" world");
        assert_eq!(r, "hello world");
    }
}
```

### Documentation Sections

**Module-level docs (`//!`):**
- Overview of the crate
- High-level architecture
- Usage examples

**Item-level docs (`///`):**
- What the item does
- Arguments and return values
- Examples
- Panics, errors, safety

**Standard sections:**
- `# Examples` - Code examples
- `# Panics` - When it panics
- `# Errors` - Error conditions
- `# Safety` - For unsafe code

---

## Step 3: Enhanced README.md

Update the root `README.md`:

```markdown
# BorrowScope

[![CI](https://github.com/yourusername/borrowscope/workflows/CI/badge.svg)](https://github.com/yourusername/borrowscope/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

> Visualize Rust's ownership and borrowing in real-time

## 🎯 What is BorrowScope?

BorrowScope makes Rust's invisible ownership system visible through interactive visualizations. It helps you understand:

- **Ownership** - Who owns what data
- **Borrowing** - How references relate to owners
- **Lifetimes** - When variables are created and dropped
- **Moves** - How ownership transfers between variables

## ✨ Features

- 🔍 **Automatic tracking** - Just add `#[trace_borrow]`
- 📊 **Interactive graphs** - Visualize ownership relationships
- ⏱️ **Timeline view** - See variable lifecycles
- 🚀 **Zero-cost** - Minimal runtime overhead
- 🎓 **Educational** - Learn Rust's memory model

## 🚀 Quick Start

### Installation

```bash
cargo install borrowscope
```

### Usage

```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r = &s;
    println!("{}", r);
}
```

```bash
cargo borrowscope visualize src/main.rs
```

## 📚 Documentation

- [User Guide](docs/guide/README.md)
- [API Documentation](https://docs.rs/borrowscope)
- [Examples](examples/)
- [Contributing](CONTRIBUTING.md)

## 🏗️ Project Structure

```
borrowscope/
├── borrowscope-macro/      # Procedural macros
├── borrowscope-runtime/    # Event tracking
├── borrowscope-cli/        # Command-line interface
└── borrowscope-ui/         # Visualization (coming soon)
```

## 🛠️ Development

### Requirements

- Rust 1.75.0 or later
- Components: rustfmt, clippy, rust-src

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/borrowscope.git
cd borrowscope

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace

# Format code
cargo fmt --all
```

### Running the CLI

```bash
cargo run -p borrowscope-cli -- borrowscope --help
```

## 📖 Examples

### Basic Ownership

```rust
#[trace_borrow]
fn ownership_example() {
    let s = String::from("hello");
    let t = s;  // Ownership moved
    // println!("{}", s);  // Error: s no longer valid
}
```

### Borrowing

```rust
#[trace_borrow]
fn borrowing_example() {
    let s = String::from("hello");
    let r1 = &s;  // Immutable borrow
    let r2 = &s;  // Multiple immutable borrows OK
    println!("{} {}", r1, r2);
}
```

### Mutable Borrowing

```rust
#[trace_borrow]
fn mutable_borrowing_example() {
    let mut s = String::from("hello");
    let r = &mut s;  // Mutable borrow
    r.push_str(" world");
    println!("{}", r);
}
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- Inspired by Rust's borrow checker
- Built with [syn](https://github.com/dtolnay/syn), [quote](https://github.com/dtolnay/quote), and [Tauri](https://tauri.app/)

## 📬 Contact

- Issues: [GitHub Issues](https://github.com/yourusername/borrowscope/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/borrowscope/discussions)
```

---

## Step 4: Create User Guide Structure

```bash
mkdir -p docs/guide
```

### docs/guide/README.md

```markdown
# BorrowScope User Guide

Welcome to the BorrowScope user guide!

## Table of Contents

1. [Installation](installation.md)
2. [Getting Started](getting-started.md)
3. [Basic Usage](basic-usage.md)
4. [Understanding the Visualization](visualization.md)
5. [Advanced Features](advanced-features.md)
6. [Troubleshooting](troubleshooting.md)
7. [FAQ](faq.md)

## Quick Links

- [API Documentation](https://docs.rs/borrowscope)
- [Examples](../../examples/)
- [Contributing](../../CONTRIBUTING.md)
```

### docs/guide/installation.md

```markdown
# Installation

## Prerequisites

- Rust 1.75.0 or later
- Cargo (comes with Rust)

## Install from crates.io

```bash
cargo install borrowscope
```

## Install from Source

```bash
git clone https://github.com/yourusername/borrowscope.git
cd borrowscope
cargo install --path borrowscope-cli
```

## Verify Installation

```bash
cargo borrowscope --version
```

You should see:
```
cargo-borrowscope 0.1.0
```

## Next Steps

Continue to [Getting Started](getting-started.md) to learn how to use BorrowScope.
```

---

## Step 5: Configure Documentation Generation

### Create .cargo/config.toml

```bash
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[doc]
# Open documentation in browser after building
browser = ["firefox", "google-chrome", "open"]

[build]
# Number of parallel jobs
jobs = 4

[term]
# Use color in terminal output
color = "always"
EOF
```

### Add Documentation Scripts

Create `scripts/docs.sh`:

```bash
#!/bin/bash
# Generate all documentation

set -e

echo "📚 Generating documentation..."

# API documentation
echo "Building API docs..."
cargo doc --workspace --no-deps --document-private-items

# User guide (if using mdBook)
if command -v mdbook &> /dev/null; then
    echo "Building user guide..."
    cd docs/guide
    mdbook build
    cd ../..
fi

echo "✅ Documentation generated!"
echo "API docs: target/doc/borrowscope/index.html"
```

Make it executable:
```bash
chmod +x scripts/docs.sh
```

---

## Step 6: Add Examples

Create `examples/` directory:

```bash
mkdir -p examples
```

### examples/basic_ownership.rs

```rust
//! Basic ownership example
//!
//! This example demonstrates basic ownership in Rust.

use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn main() {
    // Create a String (heap-allocated)
    let s = String::from("hello");
    
    // Ownership moved to t
    let t = s;
    
    // s is no longer valid here
    // println!("{}", s);  // This would error!
    
    println!("{}", t);  // t owns the String now
}
```

### examples/borrowing.rs

```rust
//! Borrowing example
//!
//! This example demonstrates immutable and mutable borrowing.

use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn main() {
    let mut s = String::from("hello");
    
    // Immutable borrows
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point
    
    // Mutable borrow
    let r3 = &mut s;
    r3.push_str(" world");
    println!("{}", r3);
}
```

### Update Cargo.toml

Add examples to workspace members' `Cargo.toml`:

```toml
[[example]]
name = "basic_ownership"
path = "examples/basic_ownership.rs"

[[example]]
name = "borrowing"
path = "examples/borrowing.rs"
```

---

## Step 7: Documentation Best Practices

### 1. Write Examples

Every public function should have an example:

```rust
/// Calculate the sum of two numbers
///
/// # Examples
///
/// ```
/// use mylib::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 2. Document Panics

```rust
/// Get the first element
///
/// # Panics
///
/// Panics if the slice is empty.
pub fn first<T>(slice: &[T]) -> &T {
    &slice[0]
}
```

### 3. Document Errors

```rust
/// Read a file
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist
/// - Permission is denied
/// - The file is not valid UTF-8
pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

### 4. Link to Related Items

```rust
/// Track a new variable
///
/// See also: [`track_borrow`], [`track_drop`]
pub fn track_new<T>(name: &str, value: T) -> T {
    value
}
```

### 5. Use Intra-doc Links

```rust
/// The main tracker
///
/// Use [`Tracker::new`] to create a new instance.
/// Call [`Tracker::track`] to record events.
pub struct Tracker {
    // ...
}
```

---

## Step 8: Generate and View Documentation

### Generate API Docs

```bash
cargo doc --workspace --no-deps --open
```

This opens the documentation in your browser.

### Generate with Private Items

```bash
cargo doc --workspace --no-deps --document-private-items
```

Useful for internal development documentation.

### Check Documentation

```bash
cargo rustdoc -- -D warnings
```

Treats documentation warnings as errors.

---

## Step 9: Add Documentation to CI

Update `.github/workflows/ci.yml`:

```yaml
docs:
  name: Documentation
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Check documentation
      run: cargo doc --workspace --no-deps --document-private-items
      env:
        RUSTDOCFLAGS: -D warnings
    
    - name: Deploy to GitHub Pages
      if: github.ref == 'refs/heads/main'
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./target/doc
```

---

## Step 10: Commit Documentation

```bash
git add docs/ examples/ scripts/
git add borrowscope-runtime/src/lib.rs
git add README.md .cargo/config.toml
git commit -m "docs: add comprehensive documentation

- Add rustdoc comments to all public APIs
- Create user guide structure
- Add examples for basic usage
- Configure documentation generation
- Add documentation CI check
- Update README with detailed information"
```

---

## Key Takeaways

### Documentation Types

✅ **API docs** - rustdoc comments in code  
✅ **User docs** - README, guides, tutorials  
✅ **Developer docs** - Architecture, design decisions  
✅ **Examples** - Runnable code demonstrating usage  

### Best Practices

✅ **Document all public items** - No exceptions  
✅ **Include examples** - Show how to use  
✅ **Document errors and panics** - Be explicit  
✅ **Use standard sections** - Examples, Panics, Errors  
✅ **Link related items** - Help navigation  

### Tools

✅ **rustdoc** - Generate API documentation  
✅ **mdBook** - Create user guides  
✅ **cargo doc** - Build documentation  
✅ **CI checks** - Enforce documentation quality  

---

**Previous Section:** [06-rust-toolchain-configuration.md](./06-rust-toolchain-configuration.md)  
**Next Section:** [08-development-environment-optimization.md](./08-development-environment-optimization.md)

**Chapter Progress:** 7/8 sections complete ⬛⬛⬛⬛⬛⬛⬜

---

*"Good documentation is as important as good code." 📚*
