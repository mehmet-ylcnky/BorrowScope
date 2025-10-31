# Chapter 7 Summary: CLI Development with Clap

## Overview

Chapter 7 covered the complete implementation of BorrowScope's command-line interface using Clap v4. You learned how to design user-friendly CLIs, implement subcommands, handle errors gracefully, and integrate with Cargo and the file system.

---

## What You Built

### Complete CLI Application

1. **borrowscope-cli** - Full-featured binary crate
2. **5 subcommands** - run, visualize, export, init, check
3. **Configuration system** - Project and user-level config
4. **Error handling** - User-friendly error messages with suggestions
5. **Integration tests** - Comprehensive CLI testing

---

## Key Concepts

### CLI Design Principles (Section 76)

**Unix Philosophy:**
- Do one thing well
- Composable with other tools
- Predictable behavior

**Command Structure:**
```
borrowscope
├── run [PATH]              # Instrument and run code
├── visualize <FILE>        # Open visualization UI
├── export <FILE>           # Export to different formats
├── init                    # Initialize configuration
└── check <FILE>            # Validate tracking data
```

### Clap v4 Fundamentals (Section 77)

**Derive API:**
```rust
#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about = "Visualize Rust ownership")]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),
    Visualize(VisualizeArgs),
    Export(ExportArgs),
    Init(InitArgs),
    Check(CheckArgs),
}
```

**Features:**
- Type-safe argument parsing
- Automatic help generation
- Value validation
- Subcommands with Args structs
- Value enums for type-safe options

### CLI Crate Structure (Section 78)

```
borrowscope-cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # Argument definitions
│   ├── commands/            # Command implementations
│   │   ├── run.rs
│   │   ├── visualize.rs
│   │   ├── export.rs
│   │   ├── init.rs
│   │   └── check.rs
│   ├── config.rs            # Configuration
│   ├── error.rs             # Error types
│   ├── instrumentation.rs   # Code transformation
│   ├── output.rs            # Output formatting
│   └── utils.rs             # Utilities
└── tests/
    └── integration.rs       # Integration tests
```

---

## Command Implementations

### Run Command (Section 79)

**Purpose:** Instrument code, execute, capture tracking data

```rust
pub async fn execute(args: RunArgs) -> Result<()> {
    // 1. Build project
    build_project(&args)?;
    
    // 2. Run instrumented code
    run_instrumented(&args)?;
    
    // 3. Save tracking data
    save_tracking_data(&args.output)?;
    
    // 4. Optionally visualize
    if args.visualize {
        visualize::execute(viz_args).await?;
    }
    
    Ok(())
}
```

**Features:**
- Cargo integration
- Release/debug builds
- Feature flags
- Argument passing
- Output capture

### Visualize Command (Section 80)

**Purpose:** Start web server, display interactive visualization

```rust
pub async fn execute(args: VisualizeArgs) -> Result<()> {
    // 1. Load tracking data
    let data = load_tracking_data(&args.file)?;
    
    // 2. Start web server
    let server = start_server(&args, data).await?;
    
    // 3. Open browser
    if !args.no_browser {
        open_browser(&args)?;
    }
    
    // 4. Keep server running
    server.await?;
    
    Ok(())
}
```

**Features:**
- HTTP server with Tokio
- Cross-platform browser opening
- Custom port/host
- Auto-open option

### Export Command (Section 81)

**Purpose:** Convert tracking data to various formats

```rust
pub async fn execute(args: ExportArgs) -> Result<()> {
    let graph = load_graph(&args.file)?;
    
    match args.format {
        ExportFormat::Dot => export_dot(&graph, &args.output)?,
        ExportFormat::Svg => export_svg(&graph, &args.output)?,
        ExportFormat::Png => export_png(&graph, &args.output)?,
        ExportFormat::Json => export_json(&graph, &args.output)?,
        ExportFormat::Html => export_html(&graph, &args.output)?,
    }
    
    Ok(())
}
```

**Formats:**
- DOT (Graphviz)
- SVG (vector image)
- PNG (raster image)
- JSON (compact)
- HTML (standalone report)

### Init Command (Section 82)

**Purpose:** Initialize BorrowScope configuration

```rust
pub async fn execute(args: InitArgs) -> Result<()> {
    let config = match args.template {
        ConfigTemplate::Default => Config::default(),
        ConfigTemplate::Minimal => create_minimal_config(),
        ConfigTemplate::Advanced => create_advanced_config(),
    };
    
    config.save(&config_path)?;
    
    Ok(())
}
```

**Templates:**
- Default: Balanced settings
- Minimal: Essential only
- Advanced: All features enabled

### Check Command (Section 82)

**Purpose:** Validate and analyze tracking data

```rust
pub async fn execute(args: CheckArgs) -> Result<()> {
    let graph = load_graph(&args.file)?;
    
    if args.validate {
        validate_graph(&graph)?;
    }
    
    if args.conflicts {
        check_conflicts(&graph)?;
    }
    
    if args.stats {
        show_statistics(&graph)?;
    }
    
    Ok(())
}
```

**Checks:**
- Graph integrity validation
- Borrow conflict detection
- Statistical analysis

---

## Advanced Features

### File Instrumentation (Section 83)

```rust
pub struct Instrumenter {
    source_dir: PathBuf,
    output_dir: PathBuf,
}

impl Instrumenter {
    pub fn instrument_project(&self) -> Result<Vec<PathBuf>> {
        // Walk directory tree
        // Parse Rust files with syn
        // Apply transformations
        // Write instrumented code
    }
}
```

### Temporary File Management (Section 84)

```rust
pub struct TempWorkspace {
    dir: TempDir,
}

impl TempWorkspace {
    pub fn new() -> Result<Self>;
    pub fn copy_project(&self, source: &Path) -> Result<PathBuf>;
    pub fn cleanup(self) -> Result<()>;
}
```

### Cargo Integration (Section 85)

```rust
pub fn get_metadata(path: &Path) -> Result<CargoMetadata>;
pub fn build_project(path: &Path, release: bool, features: &[String]) -> Result<()>;
pub fn run_binary(path: &Path, release: bool, args: &[String]) -> Result<Output>;
```

### Configuration System (Section 86)

```toml
# .borrowscope.toml

[run]
output = "borrowscope.json"
visualize = false
capture = true

[visualize]
port = 3000
browser = true
host = "127.0.0.1"

[export]
format = "dot"

[tracking]
smart_pointers = true
async_code = true
unsafe_code = false

[ignore]
patterns = ["*.test.rs", "*_test.rs"]
directories = ["target", "tests"]
```

### Error Handling (Section 87)

```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    
    #[error("Compilation failed: {0}")]
    CompilationError(String),
    
    // ... more variants
}

pub fn print_error(error: &anyhow::Error) {
    // Print error with colors
    // Show error chain
    // Provide suggestions
    // Link to documentation
}
```

**Features:**
- Colored output
- Error chain display
- Contextual suggestions
- Documentation links

### Integration Tests (Section 88)

```rust
#[test]
fn test_run_simple_project() {
    let temp = TempDir::new().unwrap();
    // Create test project
    // Run borrowscope
    // Verify output
}

#[test]
fn test_visualize_command() {
    // Create sample data
    // Start server
    // Verify response
}
```

**Test Coverage:**
- All subcommands
- Error cases
- File operations
- Configuration loading

---

## Usage Examples

### Basic Workflow

```bash
# Initialize project
borrowscope init

# Run and visualize
borrowscope run --visualize

# Or step by step
borrowscope run --output tracking.json
borrowscope visualize tracking.json
```

### Advanced Usage

```bash
# Run with custom options
borrowscope run \
    --output custom.json \
    --release \
    --features "feature1,feature2" \
    -- --program-arg value

# Export to multiple formats
borrowscope export tracking.json -o graph.dot -f dot
borrowscope export tracking.json -o graph.svg -f svg
borrowscope export tracking.json -o report.html -f html

# Comprehensive check
borrowscope check tracking.json \
    --validate \
    --conflicts \
    --stats

# Verbose output
borrowscope --verbose run

# Quiet mode (errors only)
borrowscope --quiet run
```

---

## Code Artifacts

All sections include complete, production-ready implementations:

- **CLI definitions** - Complete Clap v4 derive API usage
- **Command implementations** - All 5 subcommands fully implemented
- **Error handling** - Custom error types with suggestions
- **Configuration** - TOML parsing and management
- **File operations** - Instrumentation, temp files, Cargo integration
- **Output formatting** - Colored output, progress indicators
- **Integration tests** - Comprehensive test suite

---

## Dependencies

```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "cargo"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["full"] }
colored = "2.1"
indicatif = "0.17"
toml = "0.8"
directories = "5.0"
walkdir = "2.4"
ignore = "0.4"

[dev-dependencies]
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
```

---

## Testing Strategy

**Unit Tests:**
- Argument parsing
- Configuration loading
- Error formatting
- Utility functions

**Integration Tests:**
- End-to-end command execution
- File operations
- Error cases
- Output validation

**Test Tools:**
- `assert_cmd` - CLI testing
- `predicates` - Output assertions
- `tempfile` - Temporary directories

---

## Key Takeaways

✅ **Complete CLI** - Full-featured command-line interface  
✅ **Clap v4** - Type-safe, declarative argument parsing  
✅ **5 subcommands** - run, visualize, export, init, check  
✅ **User-friendly** - Helpful errors, suggestions, documentation links  
✅ **Configuration** - Project and user-level config files  
✅ **Cargo integration** - Build, run, metadata extraction  
✅ **File operations** - Instrumentation, temp management  
✅ **Testing** - Comprehensive integration tests  
✅ **Production-ready** - Error handling, logging, progress indicators  

---

## What's Next?

**Chapter 8: Building the UI with Tauri**

You'll build the graphical user interface for BorrowScope:
- Tauri architecture and setup
- IPC communication between Rust and JavaScript
- Graph visualization with Cytoscape.js
- Timeline view with D3.js
- Interactive features (zoom, pan, filter)
- Code view integration
- Responsive layout design

The CLI you built in this chapter will serve as the backend for the UI, providing data loading, export, and analysis capabilities.

---

## Further Practice

**Exercises:**

1. **Add new subcommand** - Implement `borrowscope diff` to compare two tracking files
2. **Shell completion** - Generate completion scripts for bash/zsh/fish
3. **Watch mode** - Auto-reload on file changes
4. **Plugin system** - Support custom export formats
5. **Remote visualization** - Serve UI over network
6. **Configuration wizard** - Interactive config generation

**Challenge:**

Build a `borrowscope doctor` command that checks the environment, validates installation, and suggests fixes for common issues.

---

**Completion:** Chapter 7 complete! 🎉

**Progress:** 84/210+ sections (40% overall)

**Next:** [Chapter 8: Building the UI with Tauri](../chapter-08/89-tauri-architecture-overview.md)
