# Section 77: Clap v4 Fundamentals

## Learning Objectives

By the end of this section, you will:
- Master Clap v4's derive API
- Understand argument parsing
- Implement validation and error handling
- Use value enums and custom types
- Create subcommands with Clap

## Prerequisites

- Section 76 (CLI Design)
- Understanding of Rust attributes and derives
- Familiarity with Result and Option types

---

## Clap Overview

**Clap** (Command Line Argument Parser) is Rust's most popular CLI framework.

**Version 4 features:**
- Derive API (declarative, type-safe)
- Builder API (programmatic, flexible)
- Automatic help generation
- Shell completion
- Validation and error handling

---

## Derive API Basics

### Simple Example

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "myapp")]
#[command(version = "1.0")]
#[command(about = "Does awesome things", long_about = None)]
struct Cli {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,
    
    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let cli = Cli::parse();
    
    for _ in 0..cli.count {
        println!("Hello {}!", cli.name);
    }
}
```

**Usage:**
```bash
myapp --name Alice --count 3
# Hello Alice!
# Hello Alice!
# Hello Alice!
```

---

## Argument Types

### Positional Arguments

```rust
#[derive(Parser)]
struct Cli {
    /// Input file
    input: PathBuf,
    
    /// Output file (optional)
    output: Option<PathBuf>,
}
```

**Usage:**
```bash
myapp input.txt
myapp input.txt output.txt
```

### Flags (Boolean)

```rust
#[derive(Parser)]
struct Cli {
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}
```

**Usage:**
```bash
myapp --verbose
myapp -v -d
myapp --verbose --debug
```

### Options (Values)

```rust
#[derive(Parser)]
struct Cli {
    /// Output file
    #[arg(short, long)]
    output: String,
    
    /// Port number
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}
```

**Usage:**
```bash
myapp --output result.txt --port 3000
myapp -o result.txt -p 3000
```

### Multiple Values

```rust
#[derive(Parser)]
struct Cli {
    /// Input files
    #[arg(short, long)]
    files: Vec<PathBuf>,
    
    /// Tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,
}
```

**Usage:**
```bash
myapp --files a.txt --files b.txt --files c.txt
myapp --tags rust,cli,tool
```

---

## Value Enums

### Basic Enum

```rust
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
enum Format {
    Json,
    Yaml,
    Toml,
}

#[derive(Parser)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum)]
    format: Format,
}
```

**Usage:**
```bash
myapp --format json
myapp --format yaml
```

### Enum with Aliases

```rust
#[derive(ValueEnum, Clone)]
enum LogLevel {
    #[value(alias = "err")]
    Error,
    
    #[value(alias = "warn")]
    Warning,
    
    #[value(alias = "inf")]
    Info,
    
    #[value(alias = "dbg")]
    Debug,
}
```

**Usage:**
```bash
myapp --level error
myapp --level err    # Alias works too
```

---

## Subcommands

### Basic Subcommands

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item
    Add {
        /// Item name
        name: String,
    },
    
    /// Remove an item
    Remove {
        /// Item ID
        id: u32,
    },
    
    /// List all items
    List,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add { name } => {
            println!("Adding: {}", name);
        }
        Commands::Remove { id } => {
            println!("Removing: {}", id);
        }
        Commands::List => {
            println!("Listing items...");
        }
    }
}
```

**Usage:**
```bash
myapp add "New Item"
myapp remove 42
myapp list
```

### Subcommands with Args Struct

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    Remove(RemoveArgs),
    List(ListArgs),
}

#[derive(Args)]
struct AddArgs {
    /// Item name
    name: String,
    
    /// Item description
    #[arg(short, long)]
    description: Option<String>,
    
    /// Tags
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,
}

#[derive(Args)]
struct RemoveArgs {
    /// Item ID
    id: u32,
    
    /// Force removal
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct ListArgs {
    /// Filter by tag
    #[arg(long)]
    tag: Option<String>,
    
    /// Limit results
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}
```

---

## Validation

### Value Ranges

```rust
#[derive(Parser)]
struct Cli {
    /// Port number (1024-65535)
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1024..))]
    port: u16,
    
    /// Percentage (0-100)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=100))]
    percent: u8,
}
```

### Custom Validation

```rust
use std::path::PathBuf;

fn validate_file_exists(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("File does not exist: {}", s))
    }
}

#[derive(Parser)]
struct Cli {
    /// Input file (must exist)
    #[arg(value_parser = validate_file_exists)]
    input: PathBuf,
}
```

### Required Unless

```rust
#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(short, long, required_unless_present = "stdin")]
    input: Option<PathBuf>,
    
    /// Read from stdin
    #[arg(long)]
    stdin: bool,
}
```

### Conflicts

```rust
#[derive(Parser)]
struct Cli {
    /// Verbose output
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,
    
    /// Quiet mode
    #[arg(short, long)]
    quiet: bool,
}
```

---

## Default Values

### Static Defaults

```rust
#[derive(Parser)]
struct Cli {
    /// Output file
    #[arg(short, long, default_value = "output.txt")]
    output: String,
    
    /// Port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}
```

### Dynamic Defaults

```rust
fn default_output() -> String {
    format!("output_{}.txt", chrono::Local::now().format("%Y%m%d"))
}

#[derive(Parser)]
struct Cli {
    /// Output file
    #[arg(short, long, default_value_t = default_output())]
    output: String,
}
```

---

## Global Options

```rust
#[derive(Parser)]
struct Cli {
    /// Verbose mode (applies to all subcommands)
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { name: String },
    Remove { id: u32 },
}
```

**Usage:**
```bash
myapp --verbose add "Item"
myapp add "Item" --verbose  # Both work
```

---

## Help Text Customization

### Field Documentation

```rust
#[derive(Parser)]
struct Cli {
    /// Name of the person to greet
    /// 
    /// This will be used in the greeting message.
    /// Multiple lines are supported.
    #[arg(short, long)]
    name: String,
}
```

### Custom Help

```rust
#[derive(Parser)]
#[command(
    name = "myapp",
    version = "1.0.0",
    author = "Your Name <you@example.com>",
    about = "Does awesome things",
    long_about = "A longer description that appears when using --help.\n\
                  Can span multiple lines and include examples."
)]
struct Cli {
    // ...
}
```

### After Help

```rust
#[derive(Parser)]
#[command(after_help = "EXAMPLES:\n  \
    myapp --name Alice\n  \
    myapp --name Bob --count 3\n\n\
For more information, visit: https://example.com")]
struct Cli {
    // ...
}
```

---

## Error Handling

### Automatic Errors

```rust
#[derive(Parser)]
struct Cli {
    /// Port number
    #[arg(short, long)]
    port: u16,  // Clap validates it's a valid u16
}
```

**Error output:**
```
error: invalid value 'abc' for '--port <PORT>': invalid digit found in string

For more information, try '--help'.
```

### Custom Error Messages

```rust
use clap::error::ErrorKind;

fn main() {
    let cli = Cli::parse();
    
    if cli.port < 1024 {
        Cli::command()
            .error(
                ErrorKind::ValueValidation,
                "Port must be >= 1024 (privileged ports require root)"
            )
            .exit();
    }
}
```

---

## Complete Example: BorrowScope CLI

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about = "Visualize Rust ownership and borrowing")]
#[command(after_help = "EXAMPLES:\n  \
    borrowscope run\n  \
    borrowscope run --visualize\n  \
    borrowscope visualize borrowscope.json\n  \
    borrowscope export data.json -o graph.dot -f dot\n\n\
For more information: https://github.com/yourusername/borrowscope")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
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
    
    /// Features to enable (comma-separated)
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    
    /// Don't capture stdout/stderr
    #[arg(long)]
    no_capture: bool,
}

#[derive(Args)]
struct VisualizeArgs {
    /// Path to tracking data file
    #[arg(value_parser = validate_json_file)]
    file: PathBuf,
    
    /// Port for web server
    #[arg(short, long, default_value_t = 3000, value_parser = clap::value_parser!(u16).range(1024..))]
    port: u16,
    
    /// Don't open browser automatically
    #[arg(long)]
    no_browser: bool,
    
    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[derive(Args)]
struct ExportArgs {
    /// Input tracking data file
    #[arg(value_parser = validate_json_file)]
    file: PathBuf,
    
    /// Output file
    #[arg(short, long)]
    output: PathBuf,
    
    /// Export format
    #[arg(short = 'f', long, value_enum)]
    format: ExportFormat,
}

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

#[derive(Args)]
struct CheckArgs {
    /// Tracking data file
    #[arg(value_parser = validate_json_file)]
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

#[derive(ValueEnum, Clone)]
enum ConfigTemplate {
    Default,
    Minimal,
    Advanced,
}

fn validate_json_file(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    
    if !path.exists() {
        return Err(format!("File does not exist: {}", s));
    }
    
    if path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Err(format!("File must have .json extension: {}", s));
    }
    
    Ok(path)
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run(args) => {
            println!("Running: {:?}", args.path);
            println!("Output: {:?}", args.output);
            if args.visualize {
                println!("Will open visualization");
            }
        }
        Commands::Visualize(args) => {
            println!("Visualizing: {:?}", args.file);
            println!("Server: {}:{}", args.host, args.port);
        }
        Commands::Export(args) => {
            println!("Exporting: {:?} -> {:?}", args.file, args.output);
            println!("Format: {:?}", args.format);
        }
        Commands::Init(args) => {
            println!("Initializing in: {:?}", args.path);
            println!("Template: {:?}", args.template);
        }
        Commands::Check(args) => {
            println!("Checking: {:?}", args.file);
            if args.conflicts {
                println!("Checking for conflicts...");
            }
            if args.stats {
                println!("Computing statistics...");
            }
        }
    }
}
```

---

## Testing CLI Arguments

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_default() {
        let cli = Cli::try_parse_from(&["borrowscope", "run"]).unwrap();
        
        match cli.command {
            Commands::Run(args) => {
                assert_eq!(args.path, PathBuf::from("."));
                assert_eq!(args.output, PathBuf::from("borrowscope.json"));
                assert!(!args.visualize);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_with_options() {
        let cli = Cli::try_parse_from(&[
            "borrowscope", "run",
            "--output", "custom.json",
            "--visualize",
            "--release"
        ]).unwrap();
        
        match cli.command {
            Commands::Run(args) => {
                assert_eq!(args.output, PathBuf::from("custom.json"));
                assert!(args.visualize);
                assert!(args.release);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_visualize() {
        let cli = Cli::try_parse_from(&[
            "borrowscope", "visualize",
            "data.json",
            "--port", "8080"
        ]).unwrap();
        
        match cli.command {
            Commands::Visualize(args) => {
                assert_eq!(args.file, PathBuf::from("data.json"));
                assert_eq!(args.port, 8080);
            }
            _ => panic!("Expected Visualize command"),
        }
    }

    #[test]
    fn test_invalid_port() {
        let result = Cli::try_parse_from(&[
            "borrowscope", "visualize",
            "data.json",
            "--port", "100"  // Below 1024
        ]);
        
        assert!(result.is_err());
    }
}
```

---

## Key Takeaways

✅ **Derive API** - Type-safe, declarative argument parsing  
✅ **Subcommands** - Organize complex CLIs  
✅ **Validation** - Built-in and custom validators  
✅ **Value enums** - Type-safe enum arguments  
✅ **Help generation** - Automatic, customizable help text  

---

## Further Reading

- [Clap documentation](https://docs.rs/clap/)
- [Clap derive reference](https://docs.rs/clap/latest/clap/_derive/index.html)
- [Clap examples](https://github.com/clap-rs/clap/tree/master/examples)

---

**Previous:** [76-command-line-interface-design.md](./76-command-line-interface-design.md)  
**Next:** [78-creating-the-cli-crate.md](./78-creating-the-cli-crate.md)

**Progress:** 2/13 ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
