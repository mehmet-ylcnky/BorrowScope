# Section 03: Setting Up the Workspace - Implementation Notes

**Date:** 2025-10-31  
**Status:** ✅ Complete  
**Build Status:** ✅ Compiles successfully

---

## What Was Implemented

Created a complete Rust workspace with three member crates:
1. `borrowscope-macro` - Procedural macros
2. `borrowscope-runtime` - Runtime tracking system
3. `borrowscope-cli` - Command-line interface

## Files Created

```
/home/a524573/borrowscope/
├── Cargo.toml                              # Workspace root
├── borrowscope-macro/
│   ├── Cargo.toml                          # Macro crate config
│   └── src/lib.rs                          # Macro implementation
├── borrowscope-runtime/
│   ├── Cargo.toml                          # Runtime crate config
│   └── src/lib.rs                          # Runtime API
└── borrowscope-cli/
    ├── Cargo.toml                          # CLI crate config
    └── src/main.rs                         # CLI entry point
```

---

## Code Explanation

### 1. Workspace Root (`Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = [
    "borrowscope-macro",
    "borrowscope-runtime",
    "borrowscope-cli",
]
```

**What:** Defines a workspace containing three member crates.

**Why `resolver = "2"`:** 
- Rust Edition 2021's dependency resolver
- Better handles feature unification across workspace
- Prevents unnecessary feature enabling

**How it works:**
- All members share a single `Cargo.lock` file
- All members build into the same `target/` directory
- Dependencies are deduplicated across the workspace

### 2. Shared Workspace Configuration

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
```

**What:** Metadata inherited by all member crates.

**Why:**
- **DRY principle** - Define once, use everywhere
- **Consistency** - All crates use same version/edition
- **Easy updates** - Change version in one place

**Key fields:**
- `version = "0.1.0"` - Semantic versioning (major.minor.patch)
- `edition = "2021"` - Rust edition (affects language features)
- `rust-version = "1.75"` - Minimum Supported Rust Version (MSRV)

### 3. Workspace Dependencies

```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
syn = { version = "2.0", features = ["full", "visit-mut"] }
```

**What:** Centralized dependency versions.

**Why:**
- **Version consistency** - All crates use same dependency versions
- **Single source of truth** - Update version once
- **Easier maintenance** - No version conflicts

**How members use them:**
```toml
[dependencies]
serde.workspace = true  # Inherits version from workspace
```

---

## Crate-by-Crate Breakdown

### borrowscope-macro

**Purpose:** Provides the `#[trace_borrow]` attribute macro.

**Cargo.toml:**
```toml
[lib]
proc-macro = true
```

**What this means:**
- `proc-macro = true` - This is a procedural macro crate
- **Special compiler plugin** - Runs at compile time
- **Can only export macros** - No regular functions

**Dependencies:**
- `syn` - Parse Rust code into AST (Abstract Syntax Tree)
- `quote` - Generate Rust code from templates
- `proc-macro2` - Wrapper around compiler's proc-macro API

**lib.rs:**
```rust
#[proc_macro_attribute]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item  // Pass through unchanged for now
}
```

**What:** Attribute macro that will instrument functions.

**How it works:**
- `#[proc_macro_attribute]` - Marks this as an attribute macro
- `_attr: TokenStream` - Macro arguments (e.g., `#[trace_borrow(debug)]`)
- `item: TokenStream` - The code being annotated
- Returns modified code as `TokenStream`

**Why placeholder:** We'll implement the actual instrumentation in Chapter 2.

---

### borrowscope-runtime

**Purpose:** Runtime system that tracks ownership events.

**Cargo.toml:**
```toml
[dependencies]
serde.workspace = true
parking_lot.workspace = true
petgraph.workspace = true
```

**Dependencies explained:**
- `serde` - Serialize tracking data to JSON
- `parking_lot` - Fast, efficient locks (better than std::sync)
- `petgraph` - Graph data structure for ownership relationships

**lib.rs - Core API:**

```rust
pub fn track_new<T>(_name: &str, value: T) -> T {
    value
}
```

**What:** Tracks variable creation.

**Signature breakdown:**
- `<T>` - Generic over any type
- `_name: &str` - Variable name (unused for now, hence `_`)
- `value: T` - Takes ownership of the value
- `-> T` - Returns ownership back

**Why this design:**
- **Zero-cost abstraction** - Just returns the value unchanged
- **Ownership transfer** - Takes and returns ownership
- **Will be expanded** - Chapter 3 adds actual tracking

```rust
pub fn track_borrow<'a, T>(_name: &str, value: &'a T) -> &'a T {
    value
}
```

**What:** Tracks immutable borrows.

**Lifetime `'a` explained:**
- Input `value` has lifetime `'a`
- Output reference also has lifetime `'a`
- **Compiler ensures:** Returned reference lives as long as input
- **Why needed:** Without it, compiler doesn't know which parameter the return borrows from

```rust
pub fn track_borrow_mut<'a, T>(_name: &str, value: &'a mut T) -> &'a mut T {
    value
}
```

**What:** Tracks mutable borrows.

**Key difference from `track_borrow`:**
- `&'a mut T` - Mutable reference
- **Exclusive access** - Only one mutable borrow allowed
- **Rust's safety guarantee** - Prevents data races

---

### borrowscope-cli

**Purpose:** Command-line interface for users.

**Cargo.toml:**
```toml
[[bin]]
name = "borrowscope"
path = "src/main.rs"
```

**What:** Defines a binary target.

**Result:** Users can run `cargo install borrowscope-cli` and get a `borrowscope` command.

**Dependencies:**
- `clap` - Command-line argument parsing
- `anyhow` - Error handling with context

**main.rs:**

```rust
#[derive(Parser)]
#[command(name = "borrowscope")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
```

**What:** Defines CLI structure using clap's derive API.

**How clap works:**
- `#[derive(Parser)]` - Auto-generates argument parsing code
- `#[command(name = "borrowscope")]` - Sets program name
- `command: Option<Commands>` - Optional subcommand

```rust
#[derive(Parser)]
enum Commands {
    Run { file: String },
}
```

**What:** Defines available subcommands.

**Result:** Users can run:
```bash
borrowscope run src/main.rs
```

**Main function:**
```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Run { file }) => {
            println!("Running: {}", file);
            Ok(())
        }
        None => {
            println!("BorrowScope v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
```

**Error handling:**
- `-> anyhow::Result<()>` - Can return errors with context
- `anyhow` automatically prints error chain
- `?` operator propagates errors up

---

## Key Rust Concepts Demonstrated

### 1. Lifetimes

**Problem we solved:**
```rust
// ❌ Compiler error - missing lifetime
pub fn track_borrow<T>(value: &T) -> &T { value }
```

**Solution:**
```rust
// ✅ Explicit lifetime ties input to output
pub fn track_borrow<'a, T>(value: &'a T) -> &'a T { value }
```

**Why:** Compiler needs to know the returned reference's lifetime. The `'a` annotation says "the output lives as long as the input."

### 2. Generic Functions

```rust
pub fn track_new<T>(value: T) -> T { value }
```

**What:** Works with any type `T`.

**Monomorphization:** Compiler generates specialized versions:
- `track_new::<String>` for String
- `track_new::<i32>` for i32
- etc.

**Zero-cost:** No runtime overhead - as fast as hand-written code.

### 3. Workspace Dependency Inheritance

```toml
# In workspace root
[workspace.dependencies]
serde = "1.0"

# In member crate
[dependencies]
serde.workspace = true
```

**Benefit:** Change `serde` version once, all crates update.

---

## Why This Structure?

### Separation of Concerns

1. **borrowscope-macro** - Compile-time code transformation
2. **borrowscope-runtime** - Runtime tracking and data collection
3. **borrowscope-cli** - User interface

**Why separate:**
- **Proc macros must be separate** - Rust compiler requirement
- **Runtime can be used independently** - Other tools can use it
- **CLI is just one interface** - Could add GUI, LSP, etc.

### Dependency Graph

```
borrowscope-cli
    └── borrowscope-runtime
            └── (no dependencies on other workspace members)

borrowscope-macro
    └── (no dependencies on other workspace members)
```

**Why this matters:**
- **No circular dependencies** - Clean architecture
- **Parallel compilation** - Crates build independently
- **Reusability** - Each crate has clear purpose

---

## Build Verification

```bash
$ cargo build
   Compiling borrowscope-macro v0.1.0
   Compiling borrowscope-runtime v0.1.0
   Compiling borrowscope-cli v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

**Success indicators:**
- ✅ All three crates compile
- ✅ No warnings
- ✅ Dependencies resolved correctly

---

## What's Next

This workspace is now ready for implementation:

- **Chapter 2:** Implement the procedural macro to parse and transform code
- **Chapter 3:** Build the runtime tracking system
- **Chapter 7:** Complete the CLI with all commands

The foundation is solid - we have a proper workspace structure that follows Rust best practices.

---

## Learning Takeaways

1. **Workspaces unify multiple crates** - Shared dependencies, single build
2. **Lifetimes connect references** - Compiler tracks borrow validity
3. **Generics enable code reuse** - One function, many types
4. **Proc macros are special** - Must be in separate crate
5. **Minimal code first** - Placeholders let us verify structure before implementation
