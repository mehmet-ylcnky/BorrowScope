# Section 5: CI/CD Pipeline Basics

## Learning Objectives

By the end of this section, you will:
- Understand what CI/CD is and why it matters
- Set up GitHub Actions for automated testing
- Configure cross-platform builds (Linux, macOS, Windows)
- Add code quality checks (clippy, rustfmt)
- Implement automated workflows for pull requests
- Understand caching strategies for faster builds

## Prerequisites

- Completed Section 4 (Git setup)
- GitHub account (for GitHub Actions)
- Repository pushed to GitHub
- Basic understanding of YAML syntax

---

## What is CI/CD?

### Continuous Integration (CI)

**Definition:** Automatically build and test code every time changes are pushed.

**Benefits:**
- ✅ Catch bugs early
- ✅ Ensure code compiles on all platforms
- ✅ Verify tests pass before merging
- ✅ Maintain code quality standards

### Continuous Deployment (CD)

**Definition:** Automatically deploy code after tests pass.

**For BorrowScope:**
- Publish to crates.io
- Create GitHub releases
- Build binaries for distribution

**In this section:** We'll focus on CI (testing and quality checks).

---

## Why CI/CD for BorrowScope?

### The Problem Without CI/CD

```
Developer A (Linux):
  ✅ Code works on their machine
  ✅ Tests pass locally
  ✅ Pushes to GitHub

Developer B (Windows):
  ❌ Code doesn't compile
  ❌ Tests fail
  ❌ Wasted time debugging
```

### The Solution: Automated Testing

```
Push to GitHub
    ↓
GitHub Actions runs
    ├── Test on Linux ✅
    ├── Test on macOS ✅
    └── Test on Windows ❌ (caught before merge!)
```

**Result:** Issues caught immediately, not days later.

---

## Step 1: Understanding GitHub Actions

### What is GitHub Actions?

A CI/CD platform built into GitHub that runs workflows when events occur.

**Events:**
- Push to a branch
- Pull request opened
- Release created
- Schedule (cron)

**Workflows:**
- YAML files in `.github/workflows/`
- Define jobs and steps
- Run on GitHub's servers (or self-hosted)

### Basic Workflow Structure

```yaml
name: CI                    # Workflow name

on: [push, pull_request]    # When to run

jobs:                       # What to do
  test:                     # Job name
    runs-on: ubuntu-latest  # Operating system
    steps:                  # Steps to execute
      - uses: actions/checkout@v3
      - run: cargo test
```

---

## Step 2: Create the CI Workflow

### Create Workflow Directory

```bash
mkdir -p .github/workflows
```

### Create CI Workflow File

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Job 1: Check code formatting
  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  # Job 2: Lint with Clippy
  clippy:
    name: Clippy Lints
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  # Job 3: Build and test on multiple platforms
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-${{ matrix.rust }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-${{ matrix.rust }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-${{ matrix.rust }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --workspace --verbose

      - name: Run tests
        run: cargo test --workspace --verbose

  # Job 4: Check documentation
  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Check documentation
        run: cargo doc --workspace --no-deps --document-private-items
        env:
          RUSTDOCFLAGS: -D warnings

  # Job 5: Security audit
  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit
```

### Understanding the Workflow

Let's break down each section:

#### Workflow Triggers

```yaml
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
```

**Runs when:**
- Code is pushed to `main` branch
- Pull request is opened targeting `main`

#### Environment Variables

```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

- `CARGO_TERM_COLOR`: Colored output in logs
- `RUST_BACKTRACE`: Show backtraces on panic

#### Job: Format Check

```yaml
fmt:
  name: Format Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - run: cargo fmt --all -- --check
```

**What it does:**
1. Checks out the code
2. Installs Rust with rustfmt
3. Verifies code is formatted correctly
4. Fails if formatting is wrong

#### Job: Clippy Lints

```yaml
clippy:
  steps:
    - run: cargo clippy --all-targets --all-features -- -D warnings
```

**What it does:**
- Runs Clippy (Rust linter)
- Checks all targets (lib, bin, tests, benches)
- Treats warnings as errors (`-D warnings`)

#### Job: Test Suite

```yaml
test:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      rust: [stable, beta]
```

**Matrix strategy:**
- Tests on 3 operating systems
- Tests with 2 Rust versions
- Total: 6 combinations (3 × 2)

**Why?**
- Ensures cross-platform compatibility
- Catches platform-specific bugs
- Tests with upcoming Rust version (beta)

#### Caching

```yaml
- uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

**What it caches:**
- Cargo registry (downloaded crates)
- Cargo git dependencies
- Build artifacts (`target/`)

**Benefits:**
- Faster builds (5-10x speedup)
- Reduced network usage
- Lower CI costs

**Cache key:**
- Includes OS and Cargo.lock hash
- Invalidates when dependencies change

---

## Step 3: Add Status Badges to README

Update `README.md` to show CI status:

```markdown
# BorrowScope

[![CI](https://github.com/yourusername/borrowscope/workflows/CI/badge.svg)](https://github.com/yourusername/borrowscope/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

> Visualize Rust's ownership and borrowing in real-time

## Status

🚧 **Under Development** - Following the BorrowScope Development Course

[Rest of README...]
```

**Result:** Green badge when CI passes, red when it fails.

---

## Step 4: Configure Rustfmt

Create `.rustfmt.toml` in the root:

```toml
# Rustfmt configuration for BorrowScope

# Edition
edition = "2021"

# Maximum line width
max_width = 100

# Indentation
tab_spaces = 4
hard_tabs = false

# Imports
imports_granularity = "Crate"
group_imports = "StdExternalCrate"

# Formatting
use_small_heuristics = "Default"
fn_single_line = false
where_single_line = false

# Comments
normalize_comments = true
wrap_comments = true
comment_width = 80

# Strings
format_strings = true

# Macros
format_macro_matchers = true
format_macro_bodies = true

# Misc
newline_style = "Unix"
remove_nested_parens = true
reorder_imports = true
reorder_modules = true
```

### Understanding Rustfmt Options

**`max_width = 100`**
- Lines longer than 100 characters are wrapped
- Balance between readability and screen space

**`imports_granularity = "Crate"`**
```rust
// Before
use std::io;
use std::fs;

// After
use std::{fs, io};
```

**`group_imports = "StdExternalCrate"`**
```rust
// Groups imports by category
use std::io;           // Standard library

use serde::Serialize;  // External crates

use crate::tracker;    // Internal modules
```

**`reorder_imports = true`**
- Alphabetically sorts imports
- Consistent ordering

---

## Step 5: Configure Clippy

Create `.clippy.toml` in the root:

```toml
# Clippy configuration for BorrowScope

# Cognitive complexity threshold
cognitive-complexity-threshold = 30

# Documentation
missing-docs-in-crate-items = true

# Performance
too-many-arguments-threshold = 7
type-complexity-threshold = 250

# Style
single-char-binding-names-threshold = 4
```

### Understanding Clippy Options

**`cognitive-complexity-threshold = 30`**
- Warns if function is too complex
- Encourages breaking down large functions

**`missing-docs-in-crate-items = true`**
- Requires documentation for public items
- Improves API documentation

**`too-many-arguments-threshold = 7`**
- Warns if function has too many parameters
- Suggests using a struct instead

---

## Step 6: Add Cargo.toml Lints

Update workspace `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unused_imports = "warn"
unused_variables = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Allow some pedantic lints
too_many_lines = "allow"
module_name_repetitions = "allow"
```

### Understanding Lint Levels

**`forbid`** - Cannot be overridden
```rust
#![allow(unsafe_code)]  // ❌ Won't work, it's forbidden
```

**`deny`** - Error, but can be overridden
```rust
#![allow(missing_docs)]  // ✅ Works
```

**`warn`** - Warning, doesn't fail build
```rust
// Shows warning but compiles
```

**`allow`** - No warning
```rust
// Silent
```

### Clippy Lint Groups

**`all`** - All default lints
**`pedantic`** - Extra strict lints
**`nursery`** - Experimental lints
**`cargo`** - Cargo.toml lints

---

## Step 7: Test the CI Pipeline

### Commit and Push

```bash
git add .github/workflows/ci.yml .rustfmt.toml .clippy.toml
git add Cargo.toml README.md
git commit -m "ci: add GitHub Actions workflow

- Add CI pipeline with format, lint, test, docs, audit jobs
- Test on Linux, macOS, Windows with stable and beta Rust
- Add caching for faster builds
- Configure rustfmt and clippy
- Add CI status badge to README"

git push origin main
```

### Watch the Workflow

1. Go to your GitHub repository
2. Click "Actions" tab
3. See the workflow running

**You should see:**
```
CI
├── Format Check ✅
├── Clippy Lints ✅
├── Test Suite
│   ├── ubuntu-latest / stable ✅
│   ├── ubuntu-latest / beta ✅
│   ├── macos-latest / stable ✅
│   ├── macos-latest / beta ✅
│   ├── windows-latest / stable ✅
│   └── windows-latest / beta ✅
├── Documentation ✅
└── Security Audit ✅
```

### If Something Fails

Click on the failed job to see logs:

```
Run cargo clippy --all-targets --all-features -- -D warnings
warning: unused variable: `name`
  --> borrowscope-runtime/src/lib.rs:45:17
   |
45 |     pub fn track_new<T>(name: &str, value: T) -> T {
   |                         ^^^^ help: if this is intentional, prefix it with an underscore: `_name`
   |
   = note: `-D unused-variables` implied by `-D warnings`

error: could not compile `borrowscope-runtime` due to previous error
```

**Fix it:**
```rust
pub fn track_new<T>(_name: &str, value: T) -> T {
    //                ^ Add underscore prefix
    value
}
```

---

## Step 8: Add Pre-commit Checks (Optional)

Create a pre-commit hook to run checks locally:

```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "❌ Formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy found issues."
    exit 1
fi

# Tests
echo "Running tests..."
cargo test --workspace
if [ $? -ne 0 ]; then
    echo "❌ Tests failed."
    exit 1
fi

echo "✅ All checks passed!"
EOF

chmod +x .git/hooks/pre-commit
```

**What it does:**
- Runs before every commit
- Checks formatting, lints, tests
- Prevents committing broken code

**To bypass (when needed):**
```bash
git commit --no-verify
```

---

## Step 9: Add a Makefile for Common Tasks

Create `Makefile`:

```makefile
.PHONY: help build test fmt lint check clean doc

help:
	@echo "BorrowScope Development Commands"
	@echo ""
	@echo "  make build    - Build all crates"
	@echo "  make test     - Run all tests"
	@echo "  make fmt      - Format code"
	@echo "  make lint     - Run clippy"
	@echo "  make check    - Run all checks (fmt, lint, test)"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make doc      - Generate documentation"

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test
	@echo "✅ All checks passed!"

clean:
	cargo clean

doc:
	cargo doc --workspace --no-deps --open
```

**Usage:**
```bash
make check    # Run all checks
make build    # Build project
make test     # Run tests
make doc      # Open documentation
```

---

## Step 10: Understanding CI Costs and Limits

### GitHub Actions Free Tier

**For public repositories:**
- ✅ Unlimited minutes
- ✅ Unlimited concurrent jobs

**For private repositories:**
- 2,000 minutes/month (free tier)
- Different multipliers per OS:
  - Linux: 1x
  - macOS: 10x
  - Windows: 2x

### Optimizing CI Time

**1. Use caching:**
```yaml
- uses: actions/cache@v3
```
Reduces build time by 5-10x.

**2. Fail fast:**
```yaml
strategy:
  fail-fast: true  # Stop other jobs if one fails
```

**3. Run quick checks first:**
```yaml
jobs:
  fmt:      # Fast (seconds)
  clippy:   # Medium (1-2 min)
  test:     # Slow (5-10 min)
```

**4. Use matrix strategically:**
```yaml
# Test thoroughly on Linux (fast)
# Test minimally on macOS/Windows (expensive)
```

---

## Common CI Patterns

### Pattern 1: Only Run on PR

```yaml
on:
  pull_request:
    branches: [ main ]
```

**Use case:** Save CI minutes, only test before merging.

### Pattern 2: Scheduled Runs

```yaml
on:
  schedule:
    - cron: '0 0 * * 0'  # Every Sunday at midnight
```

**Use case:** Weekly dependency audits.

### Pattern 3: Manual Trigger

```yaml
on:
  workflow_dispatch:
```

**Use case:** Run workflow manually from GitHub UI.

### Pattern 4: Conditional Jobs

```yaml
jobs:
  deploy:
    if: github.ref == 'refs/heads/main'
    steps:
      - run: echo "Deploying..."
```

**Use case:** Only deploy from main branch.

---

## Troubleshooting CI Issues

### Issue 1: Tests Pass Locally, Fail in CI

**Possible causes:**
- Different Rust version
- Missing environment variables
- Platform-specific code
- Timing issues (race conditions)

**Solution:**
```bash
# Test with same Rust version as CI
rustup install stable
cargo +stable test

# Test on different platforms
# Use Docker or VM
```

### Issue 2: Cache Not Working

**Symptoms:**
- Builds take same time every run
- Dependencies re-downloaded

**Solution:**
```yaml
# Ensure cache key includes Cargo.lock
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### Issue 3: Out of Disk Space

**Error:**
```
No space left on device
```

**Solution:**
```yaml
- name: Free disk space
  run: |
    sudo rm -rf /usr/share/dotnet
    sudo rm -rf /opt/ghc
```

### Issue 4: Timeout

**Error:**
```
The job running on runner has exceeded the maximum execution time of 360 minutes.
```

**Solution:**
- Add caching
- Reduce test matrix
- Split into multiple workflows

---

## Best Practices Summary

### CI Configuration

✅ **Test on multiple platforms** - Catch platform-specific bugs  
✅ **Use caching** - Faster builds, lower costs  
✅ **Fail fast** - Get feedback quickly  
✅ **Run quick checks first** - Format before tests  
✅ **Use matrix testing** - Test multiple configurations  

### Code Quality

✅ **Enforce formatting** - Consistent code style  
✅ **Run clippy** - Catch common mistakes  
✅ **Require tests** - Maintain code quality  
✅ **Check documentation** - Keep docs up to date  
✅ **Security audits** - Find vulnerabilities  

### Workflow

✅ **Status badges** - Show CI status in README  
✅ **Pre-commit hooks** - Catch issues before pushing  
✅ **Makefile** - Convenient local commands  
✅ **Clear commit messages** - Understand CI failures  

---

## Exercises

### Exercise 1: Trigger CI

Make a small change and push:

```bash
# Add a comment to any file
echo "// Test CI" >> borrowscope-runtime/src/lib.rs

git add .
git commit -m "test: trigger CI pipeline"
git push
```

Watch the workflow run on GitHub.

### Exercise 2: Fix a Lint Warning

Introduce a warning:

```rust
// In borrowscope-runtime/src/lib.rs
pub fn track_new<T>(name: &str, value: T) -> T {
    let unused = 42;  // Unused variable
    value
}
```

Push and see CI fail. Then fix it.

### Exercise 3: Add a New Check

Add a check for unused dependencies:

```yaml
- name: Check for unused dependencies
  run: |
    cargo install cargo-udeps
    cargo +nightly udeps --workspace
```

---

## Key Takeaways

### CI/CD Fundamentals

✅ **CI** = Continuous Integration (automated testing)  
✅ **CD** = Continuous Deployment (automated releases)  
✅ **GitHub Actions** = CI/CD platform built into GitHub  
✅ **Workflows** = YAML files defining automation  
✅ **Jobs** = Independent units of work  

### BorrowScope CI Pipeline

✅ **5 jobs** - Format, lint, test, docs, audit  
✅ **6 test configurations** - 3 OS × 2 Rust versions  
✅ **Caching** - 5-10x faster builds  
✅ **Status badges** - Visible quality indicators  
✅ **Pre-commit hooks** - Local validation  

### Benefits

✅ **Catch bugs early** - Before they reach production  
✅ **Cross-platform testing** - Works everywhere  
✅ **Code quality** - Consistent standards  
✅ **Confidence** - Know code works  
✅ **Collaboration** - Safe to merge PRs  

---

## Further Reading

### Official Documentation

1. **GitHub Actions Documentation**
   - https://docs.github.com/en/actions

2. **Rust CI/CD Guide**
   - https://doc.rust-lang.org/cargo/guide/continuous-integration.html

3. **actions-rs (Rust Actions)**
   - https://github.com/actions-rs

### Example Workflows

1. **Tokio CI**
   - https://github.com/tokio-rs/tokio/blob/master/.github/workflows/ci.yml

2. **Serde CI**
   - https://github.com/serde-rs/serde/blob/master/.github/workflows/ci.yml

### Tools

1. **cargo-audit** - Security auditing
2. **cargo-deny** - Dependency linting
3. **cargo-tarpaulin** - Code coverage

---

## Reflection Questions

Before moving to Section 6, ensure you can answer:

✅ What is the difference between CI and CD?  
✅ Why test on multiple platforms?  
✅ How does caching speed up CI?  
✅ What does `cargo clippy` do?  
✅ When should CI run?  

---

## What's Next?

In **Section 6: Rust Toolchain Configuration**, we'll:
- Configure rust-toolchain.toml
- Set up MSRV (Minimum Supported Rust Version)
- Configure clippy and rustfmt in detail
- Set up IDE integration
- Optimize the development environment

---

**Previous Section:** [04-git-and-version-control-setup.md](./04-git-and-version-control-setup.md)  
**Next Section:** [06-rust-toolchain-configuration.md](./06-rust-toolchain-configuration.md)

**Chapter Progress:** 5/8 sections complete ⬛⬛⬛⬛⬜⬜⬜

---

*"Automate everything you can, so you can focus on what matters." 🤖*
