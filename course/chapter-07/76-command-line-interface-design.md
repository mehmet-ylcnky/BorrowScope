# Section 76: Command-Line Interface Design

## Learning Objectives

By the end of this section, you will:
- Understand CLI design best practices
- Design BorrowScope's command structure
- Plan subcommands and options
- Create user-friendly help text
- Follow Unix philosophy

## Prerequisites

- Completed Chapter 6 (Graph Data Structures)
- Basic understanding of command-line tools
- Familiarity with cargo commands

---

## CLI Design Principles

### 1. Unix Philosophy

**Do one thing well:**
- Each subcommand has a clear purpose
- Composable with other tools
- Predictable behavior

**Examples:**
```bash
# Good: Clear, focused commands
borrowscope run --output graph.json
borrowscope visualize graph.json

# Bad: Unclear, too many options
borrowscope --run --visualize --output graph.json --open
```

### 2. Consistency

**Follow conventions:**
- Use standard flag names (`--help`, `--version`, `--verbose`)
- Short flags for common options (`-o`, `-v`, `-q`)
- Long flags for clarity (`--output`, `--verbose`, `--quiet`)

### 3. Discoverability

**Self-documenting:**
- Comprehensive `--help` text
- Examples in help output
- Error messages suggest corrections

---

## BorrowScope CLI Structure

### Command Hierarchy

```
borrowscope
├── run [PATH]              # Instrument and run code
├── visualize <FILE>        # Open visualization UI
├── export <FILE>           # Export to different formats
├── init                    # Initialize configuration
└── check <FILE>            # Validate tracking data
```

### Design Rationale

**`run`** - Primary workflow (instrument → execute → visualize)  
**`visualize`** - View existing tracking data  
**`export`** - Convert to other formats (DOT, SVG, etc.)  
**`init`** - Setup project configuration  
**`check`** - Validate and analyze tracking data  

---

## Command Specifications

### Global Options

```rust
/// BorrowScope - Visualize Rust ownership and borrowing
#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    /// Output format (json, text, none)
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Json,
    Text,
    None,
}
```

---

## Subcommand: `run`

### Purpose

Instrument Rust code, execute it, and capture ownership tracking data.

### Signature

```bash
borrowscope run [OPTIONS] [PATH]
```

### Options

```rust
/// Run instrumented code and capture tracking data
#[derive(Args)]
struct RunArgs {
    /// Path to Rust file or project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    
    /// Output file for tracking data
    #[arg(short, long, default_value = "borrowscope.json")]
    output: PathBuf,
    
    /// Open visualization after running
    #[arg(long)]
    visualize: bool,
    
    /// Arguments to pass to the program
    #[arg(last = true)]
    args: Vec<String>,
    
    /// Release mode (optimized build)
    #[arg(long)]
    release: bool,
    
    /// Features to enable
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    
    /// Don't capture stdout/stderr
    #[arg(long)]
    no_capture: bool,
}
```

### Examples

```bash
# Run current project
borrowscope run

# Run specific file
borrowscope run examples/basic.rs

# Run with arguments
borrowscope run -- --input data.txt

# Run and visualize
borrowscope run --visualize

# Run in release mode
borrowscope run --release --output release.json
```

---

## Subcommand: `visualize`

### Purpose

Open the UI to visualize tracking data.

### Signature

```bash
borrowscope visualize [OPTIONS] <FILE>
```

### Options

```rust
/// Visualize ownership tracking data
#[derive(Args)]
struct VisualizeArgs {
    /// Path to tracking data file
    file: PathBuf,
    
    /// Port for web server
    #[arg(short, long, default_value = "3000")]
    port: u16,
    
    /// Don't open browser automatically
    #[arg(long)]
    no_browser: bool,
    
    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}
```

### Examples

```bash
# Visualize tracking data
borrowscope visualize borrowscope.json

# Custom port
borrowscope visualize data.json --port 8080

# Don't open browser
borrowscope visualize data.json --no-browser
```

---

## Subcommand: `export`

### Purpose

Convert tracking data to other formats.

### Signature

```bash
borrowscope export [OPTIONS] <FILE>
```

### Options

```rust
/// Export tracking data to different formats
#[derive(Args)]
struct ExportArgs {
    /// Input tracking data file
    file: PathBuf,
    
    /// Output file
    #[arg(short, long)]
    output: PathBuf,
    
    /// Export format
    #[arg(short = 'f', long, value_enum)]
    format: ExportFormat,
}

#[derive(ValueEnum, Clone)]
enum ExportFormat {
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
```

### Examples

```bash
# Export to DOT
borrowscope export data.json -o graph.dot -f dot

# Export to SVG
borrowscope export data.json -o graph.svg -f svg

# Export to HTML report
borrowscope export data.json -o report.html -f html
```

---

## Subcommand: `init`

### Purpose

Initialize BorrowScope configuration in a project.

### Signature

```bash
borrowscope init [OPTIONS]
```

### Options

```rust
/// Initialize BorrowScope configuration
#[derive(Args)]
struct InitArgs {
    /// Project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    
    /// Overwrite existing configuration
    #[arg(long)]
    force: bool,
    
    /// Configuration template
    #[arg(long, value_enum, default_value = "default")]
    template: ConfigTemplate,
}

#[derive(ValueEnum, Clone)]
enum ConfigTemplate {
    Default,
    Minimal,
    Advanced,
}
```

### Examples

```bash
# Initialize in current directory
borrowscope init

# Force overwrite
borrowscope init --force

# Use minimal template
borrowscope init --template minimal
```

---

## Subcommand: `check`

### Purpose

Validate and analyze tracking data.

### Signature

```bash
borrowscope check [OPTIONS] <FILE>
```

### Options

```rust
/// Check tracking data for issues
#[derive(Args)]
struct CheckArgs {
    /// Tracking data file
    file: PathBuf,
    
    /// Check for borrow conflicts
    #[arg(long)]
    conflicts: bool,
    
    /// Show statistics
    #[arg(long)]
    stats: bool,
    
    /// Validate graph integrity
    #[arg(long)]
    validate: bool,
}
```

### Examples

```bash
# Check for conflicts
borrowscope check data.json --conflicts

# Show statistics
borrowscope check data.json --stats

# Full validation
borrowscope check data.json --validate --conflicts --stats
```

---

## Help Text Design

### Main Help

```
BorrowScope - Visualize Rust ownership and borrowing

Usage: borrowscope [OPTIONS] <COMMAND>

Commands:
  run        Run instrumented code and capture tracking data
  visualize  Open visualization UI for tracking data
  export     Export tracking data to different formats
  init       Initialize BorrowScope configuration
  check      Validate and analyze tracking data
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose          Enable verbose output
  -q, --quiet            Suppress all output except errors
      --format <FORMAT>  Output format [default: text] [possible values: json, text, none]
  -h, --help             Print help
  -V, --version          Print version

Examples:
  # Run current project and visualize
  borrowscope run --visualize

  # Visualize existing data
  borrowscope visualize borrowscope.json

  # Export to DOT format
  borrowscope export data.json -o graph.dot -f dot

For more information, visit: https://github.com/yourusername/borrowscope
```

### Subcommand Help

```
Run instrumented code and capture tracking data

Usage: borrowscope run [OPTIONS] [PATH] [-- <ARGS>...]

Arguments:
  [PATH]     Path to Rust file or project directory [default: .]
  [ARGS]...  Arguments to pass to the program

Options:
  -o, --output <OUTPUT>        Output file for tracking data [default: borrowscope.json]
      --visualize              Open visualization after running
      --release                Release mode (optimized build)
      --features <FEATURES>    Features to enable (comma-separated)
      --no-capture             Don't capture stdout/stderr
  -v, --verbose                Enable verbose output
  -q, --quiet                  Suppress all output except errors
      --format <FORMAT>        Output format [default: text]
  -h, --help                   Print help

Examples:
  # Run current project
  borrowscope run

  # Run specific file
  borrowscope run examples/basic.rs

  # Run with arguments
  borrowscope run -- --input data.txt

  # Run and visualize
  borrowscope run --visualize
```

---

## Error Messages

### Design Principles

1. **Clear and actionable**
2. **Suggest solutions**
3. **Show context**

### Examples

```rust
// File not found
Error: Could not find file 'data.json'

Suggestion: Check the file path or run 'borrowscope run' first to generate tracking data.

// Invalid format
Error: Unknown export format 'pdf'

Available formats: dot, svg, png, json, html

// Port in use
Error: Port 3000 is already in use

Suggestion: Try a different port with --port <PORT>
```

---

## Configuration File

### Location

```
.borrowscope.toml  (project root)
~/.config/borrowscope/config.toml  (user config)
```

### Format

```toml
# BorrowScope Configuration

[run]
# Default output file
output = "borrowscope.json"

# Auto-open visualization
visualize = false

# Capture stdout/stderr
capture = true

[visualize]
# Default port
port = 3000

# Auto-open browser
browser = true

# Host address
host = "127.0.0.1"

[export]
# Default format
format = "dot"

[tracking]
# Track smart pointers
smart_pointers = true

# Track async code
async = true

# Track unsafe code
unsafe = false
```

---

## Complete CLI Structure

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about = "Visualize Rust ownership and borrowing", long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
struct RunArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    
    #[arg(short, long, default_value = "borrowscope.json")]
    output: PathBuf,
    
    #[arg(long)]
    visualize: bool,
    
    #[arg(last = true)]
    args: Vec<String>,
    
    #[arg(long)]
    release: bool,
    
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    
    #[arg(long)]
    no_capture: bool,
}

#[derive(Args)]
struct VisualizeArgs {
    file: PathBuf,
    
    #[arg(short, long, default_value = "3000")]
    port: u16,
    
    #[arg(long)]
    no_browser: bool,
    
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Args)]
struct ExportArgs {
    file: PathBuf,
    
    #[arg(short, long)]
    output: PathBuf,
    
    #[arg(short = 'f', long, value_enum)]
    format: ExportFormat,
}

#[derive(Args)]
struct InitArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    
    #[arg(long)]
    force: bool,
    
    #[arg(long, value_enum, default_value = "default")]
    template: ConfigTemplate,
}

#[derive(Args)]
struct CheckArgs {
    file: PathBuf,
    
    #[arg(long)]
    conflicts: bool,
    
    #[arg(long)]
    stats: bool,
    
    #[arg(long)]
    validate: bool,
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Json,
    Text,
    None,
}

#[derive(ValueEnum, Clone)]
enum ExportFormat {
    Dot,
    Svg,
    Png,
    Json,
    Html,
}

#[derive(ValueEnum, Clone)]
enum ConfigTemplate {
    Default,
    Minimal,
    Advanced,
}
```

---

## Key Takeaways

✅ **Clear structure** - Subcommands for different workflows  
✅ **Consistent naming** - Follow Unix conventions  
✅ **Self-documenting** - Comprehensive help text  
✅ **User-friendly** - Sensible defaults, helpful errors  
✅ **Composable** - Works with other tools  

---

## Further Reading

- [Command Line Interface Guidelines](https://clig.dev/)
- [The Art of Command Line](https://github.com/jlevy/the-art-of-command-line)
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46)

---

**Previous:** [Chapter 6 Summary](../chapter-06/CHAPTER_SUMMARY.md)  
**Next:** [77-clap-v4-fundamentals.md](./77-clap-v4-fundamentals.md)

**Progress:** 1/13 ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
