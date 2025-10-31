# Section 87: CLI Error Handling and UX

**src/error.rs (Enhanced):**

```rust
use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    
    #[error("Compilation failed: {0}")]
    CompilationError(String),
    
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn print_error(error: &anyhow::Error) {
    eprintln!("\n{}", "━".repeat(60).red());
    eprintln!("{} {}", "Error:".red().bold(), error);
    
    let mut source = error.source();
    while let Some(err) = source {
        eprintln!("  {} {}", "↳".yellow(), err);
        source = err.source();
    }
    
    if let Some(cli_error) = error.downcast_ref::<CliError>() {
        if let Some(suggestion) = get_suggestion(cli_error) {
            eprintln!("\n{} {}", "💡".green(), suggestion);
        }
    }
    
    eprintln!("{}\n", "━".repeat(60).red());
}

fn get_suggestion(error: &CliError) -> Option<String> {
    match error {
        CliError::FileNotFound(_) => Some("Run 'borrowscope run' first".into()),
        CliError::PortInUse(port) => Some(format!("Try --port {}", port + 1)),
        _ => None,
    }
}
```

**Progress:** 12/13
