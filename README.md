# BorrowScope

> Visualize Rust's ownership and borrowing at runtime

[![CI](https://github.com/mehmet-ylcnky/BorrowScope/actions/workflows/ci.yml/badge.svg)](https://github.com/mehmet-ylcnky/BorrowScope/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

BorrowScope is a developer tool that makes Rust's ownership and borrowing system visible through runtime tracking and interactive visualization.

## Features

- 🔍 **Automatic tracking** - Instrument code with `#[trace_borrow]` attribute
- 📊 **Interactive visualization** - See ownership relationships in real-time
- 🎯 **Conflict detection** - Identify borrow checker issues visually
- 🚀 **Zero runtime overhead** - Tracking only active during development

## Quick Start

```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
}
```

## Installation

```bash
cargo install borrowscope-cli
```

## Usage

```bash
# Analyze a Rust file
borrowscope run src/main.rs
```

## Project Structure

- `borrowscope-macro` - Procedural macros for code instrumentation
- `borrowscope-runtime` - Runtime tracking system
- `borrowscope-cli` - Command-line interface

## Development

```bash
# Clone the repository
git clone https://github.com/mehmet-ylcnky/BorrowScope.git
cd BorrowScope

# Build all crates
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with Rust 🦀
