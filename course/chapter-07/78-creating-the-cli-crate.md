# Section 78: Creating the CLI Crate

## Learning Objectives

By the end of this section, you will:
- Set up the borrowscope-cli binary crate
- Configure dependencies and features
- Implement the main entry point
- Structure the CLI codebase
- Integrate with existing crates

## Prerequisites

- Section 77 (Clap v4 Fundamentals)
- Understanding of binary vs library crates
- Familiarity with workspace configuration

## Step 1: Create the Binary Crate

```bash
cd borrowscope
cargo new --bin borrowscope-cli
```

### Update Workspace Cargo.toml

```toml
[workspace]
members = [
    "borrowscope-macro",
    "borrowscope-runtime",
    "borrowscope-graph",
    "borrowscope-cli",
]

[workspace.dependencies]
clap = { version = "4.4", features = ["derive", "cargo"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["full"] }
```

---

## Step 2: Configure CLI Cargo.toml

```toml
[package]
name = "borrowscope-cli"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "Command-line interface for BorrowScope"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/borrowscope"
keywords = ["rust", "ownership", "visualization", "cli"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "borrowscope"
path = "src/main.rs"

[dependencies]
# CLI framework
clap = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = "1.0"

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# File system
walkdir = "2.4"
ignore = "0.4"

# Process execution
tokio = { workspace = true }

# Terminal UI
colored = "2.1"
indicatif = "0.17"

# Configuration
toml = "0.8"
directories = "5.0"

# Internal dependencies
borrowscope-runtime = { path = "../borrowscope-runtime" }
borrowscope-graph = { path = "../borrowscope-graph" }
borrowscope-macro = { path = "../borrowscope-macro" }

[dev-dependencies]
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
```

---

## Step 3: Project Structure

```
borrowscope-cli/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI argument definitions
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── run.rs           # Run command
│   │   ├── visualize.rs     # Visualize command
│   │   ├── export.rs        # Export command
│   │   ├── init.rs          # Init command
│   │   └── check.rs         # Check command
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── instrumentation.rs   # Code instrumentation
│   ├── output.rs            # Output formatting
│   └── utils.rs             # Utility functions
└── tests/
    ├── integration.rs
    └── fixtures/
```

---

## Step 4: Main Entry Point

**src/main.rs:**

```rust
mod cli;
mod commands;
mod config;
mod error;
mod instrumentation;
mod output;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();
    
    // Set up logging based on verbosity
    setup_logging(cli.verbose, cli.quiet)?;
    
    // Execute the appropriate command
    let result = match cli.command {
        Commands::Run(args) => commands::run::execute(args).await,
        Commands::Visualize(args) => commands::visualize::execute(args).await,
        Commands::Export(args) => commands::export::execute(args).await,
        Commands::Init(args) => commands::init::execute(args).await,
        Commands::Check(args) => commands::check::execute(args).await,
    };
    
    // Handle errors with user-friendly messages
    if let Err(e) = result {
        error::print_error(&e);
        std::process::exit(1);
    }
    
    Ok(())
}

fn setup_logging(verbose: bool, quiet: bool) -> Result<()> {
    use colored::Colorize;
    
    if quiet {
        // Suppress all output except errors
        std::env::set_var("RUST_LOG", "error");
    } else if verbose {
        // Enable verbose output
        std::env::set_var("RUST_LOG", "debug");
    } else {
        // Default: info level
        std::env::set_var("RUST_LOG", "info");
    }
    
    env_logger::init();
    Ok(())
}
```

---

## Step 5: CLI Definitions

**src/cli.rs:**

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about = "Visualize Rust ownership and borrowing")]
#[command(long_about = "BorrowScope is a tool for visualizing Rust's ownership and borrowing system.\n\
                        It instruments your code to track variable lifetimes, borrows, and moves,\n\
                        then provides an interactive visualization to help you understand the flow.")]
#[command(after_help = "EXAMPLES:\n  \
    # Run current project and visualize\n  \
    borrowscope run --visualize\n\n  \
    # Run specific file\n  \
    borrowscope run examples/basic.rs\n\n  \
    # Visualize existing data\n  \
    borrowscope visualize borrowscope.json\n\n  \
    # Export to DOT format\n  \
    borrowscope export data.json -o graph.dot -f dot\n\n\
For more information: https://github.com/yourusername/borrowscope")]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run instrumented code and capture tracking data
    Run(RunArgs),
    
    /// Open visualization UI for tracking data
    Visualize(VisualizeArgs),
    
    /// Export tracking data to different formats
    Export(ExportArgs),
    
    /// Initialize BorrowScope configuration
    Init(InitArgs),
    
    /// Validate and analyze tracking data
    Check(CheckArgs),
}

#[derive(Args)]
pub struct RunArgs {
    /// Path to Rust file or project directory
    #[arg(default_value = ".")]
    pub path: PathBuf,
    
    /// Output file for tracking data
    #[arg(short, long, default_value = "borrowscope.json")]
    pub output: PathBuf,
    
    /// Open visualization after running
    #[arg(long)]
    pub visualize: bool,
    
    /// Arguments to pass to the program
    #[arg(last = true)]
    pub args: Vec<String>,
    
    /// Release mode (optimized build)
    #[arg(long)]
    pub release: bool,
    
    /// Features to enable (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,
    
    /// Don't capture stdout/stderr
    #[arg(long)]
    pub no_capture: bool,
}

#[derive(Args)]
pub struct VisualizeArgs {
    /// Path to tracking data file
    pub file: PathBuf,
    
    /// Port for web server
    #[arg(short, long, default_value_t = 3000, value_parser = clap::value_parser!(u16).range(1024..))]
    pub port: u16,
    
    /// Don't open browser automatically
    #[arg(long)]
    pub no_browser: bool,
    
    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Input tracking data file
    pub file: PathBuf,
    
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
    
    /// Export format
    #[arg(short = 'f', long, value_enum)]
    pub format: ExportFormat,
}

#[derive(Args)]
pub struct InitArgs {
    /// Project directory
    #[arg(default_value = ".")]
    pub path: PathBuf,
    
    /// Overwrite existing configuration
    #[arg(long)]
    pub force: bool,
    
    /// Configuration template
    #[arg(long, value_enum, default_value = "default")]
    pub template: ConfigTemplate,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Tracking data file
    pub file: PathBuf,
    
    /// Check for borrow conflicts
    #[arg(long)]
    pub conflicts: bool,
    
    /// Show statistics
    #[arg(long)]
    pub stats: bool,
    
    /// Validate graph integrity
    #[arg(long)]
    pub validate: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    /// Graphviz DOT format
    Dot,
    /// SVG image
    Svg,
    /// PNG image
    Png,
    /// Compact JSON
    Json,
    /// HTML report
    Html,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ConfigTemplate {
    Default,
    Minimal,
    Advanced,
}
```

---

## Step 6: Error Handling

**src/error.rs:**

```rust
use colored::Colorize;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    
    #[error("Compilation failed: {0}")]
    CompilationError(String),
    
    #[error("Instrumentation failed: {0}")]
    InstrumentationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub fn print_error(error: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), error);
    
    // Print error chain
    let mut source = error.source();
    while let Some(err) = source {
        eprintln!("  {} {}", "Caused by:".yellow(), err);
        source = err.source();
    }
    
    // Print suggestion if available
    if let Some(cli_error) = error.downcast_ref::<CliError>() {
        if let Some(suggestion) = get_suggestion(cli_error) {
            eprintln!("\n{} {}", "Suggestion:".green().bold(), suggestion);
        }
    }
}

fn get_suggestion(error: &CliError) -> Option<String> {
    match error {
        CliError::FileNotFound(_) => {
            Some("Check the file path or run 'borrowscope run' first to generate tracking data.".into())
        }
        CliError::InvalidFormat(_) => {
            Some("Ensure the file is a valid JSON file generated by BorrowScope.".into())
        }
        CliError::CompilationError(_) => {
            Some("Fix the compilation errors in your code before running BorrowScope.".into())
        }
        _ => None,
    }
}
```

---

## Step 7: Configuration Management

**src/config.rs:**

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub run: RunConfig,
    pub visualize: VisualizeConfig,
    pub export: ExportConfig,
    pub tracking: TrackingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunConfig {
    pub output: String,
    pub visualize: bool,
    pub capture: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizeConfig {
    pub port: u16,
    pub browser: bool,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub smart_pointers: bool,
    pub async_code: bool,
    pub unsafe_code: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            run: RunConfig {
                output: "borrowscope.json".into(),
                visualize: false,
                capture: true,
            },
            visualize: VisualizeConfig {
                port: 3000,
                browser: true,
                host: "127.0.0.1".into(),
            },
            export: ExportConfig {
                format: "dot".into(),
            },
            tracking: TrackingConfig {
                smart_pointers: true,
                async_code: true,
                unsafe_code: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try project config first
        if let Ok(config) = Self::load_from(".borrowscope.toml") {
            return Ok(config);
        }
        
        // Try user config
        if let Some(config_dir) = directories::ProjectDirs::from("", "", "borrowscope") {
            let user_config = config_dir.config_dir().join("config.toml");
            if let Ok(config) = Self::load_from(&user_config) {
                return Ok(config);
            }
        }
        
        // Use default
        Ok(Self::default())
    }
    
    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

---

## Step 8: Output Formatting

**src/output.rs:**

```rust
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Output {
    verbose: bool,
    quiet: bool,
}

impl Output {
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }
    
    pub fn info(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", "→".blue(), message);
        }
    }
    
    pub fn success(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", "✓".green(), message);
        }
    }
    
    pub fn warning(&self, message: &str) {
        if !self.quiet {
            println!("{} {}", "⚠".yellow(), message);
        }
    }
    
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }
    
    pub fn debug(&self, message: &str) {
        if self.verbose {
            println!("{} {}", "DEBUG:".dimmed(), message.dimmed());
        }
    }
    
    pub fn progress(&self, message: &str) -> Option<ProgressBar> {
        if self.quiet {
            return None;
        }
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .unwrap()
        );
        pb.set_message(message.to_string());
        Some(pb)
    }
}
```

---

## Step 9: Commands Module

**src/commands/mod.rs:**

```rust
pub mod run;
pub mod visualize;
pub mod export;
pub mod init;
pub mod check;
```

---

## Step 10: Testing Setup

**tests/integration.rs:**

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("borrowscope").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Visualize Rust ownership"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("borrowscope").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("borrowscope").unwrap();
    cmd.args(&["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Run instrumented code"));
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("borrowscope").unwrap();
    cmd.arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
```

---

## Step 11: Build and Test

```bash
# Build the CLI
cd borrowscope-cli
cargo build

# Run tests
cargo test

# Try the CLI
cargo run -- --help
cargo run -- run --help
cargo run -- visualize --help
```

---

## Key Takeaways

✅ **Binary crate** - Separate CLI from library code  
✅ **Modular structure** - Commands in separate modules  
✅ **Error handling** - User-friendly error messages  
✅ **Configuration** - Project and user-level config  
✅ **Output formatting** - Colored, progress indicators  
✅ **Testing** - Integration tests with assert_cmd  

---

## Further Reading

- [Cargo binary crates](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)
- [anyhow error handling](https://docs.rs/anyhow/)
- [colored terminal output](https://docs.rs/colored/)

---

**Previous:** [77-clap-v4-fundamentals.md](./77-clap-v4-fundamentals.md)  
**Next:** [79-implementing-the-main-command.md](./79-implementing-the-main-command.md)

**Progress:** 3/13 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
