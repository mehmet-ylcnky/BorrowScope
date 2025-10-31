//! BorrowScope CLI
//!
//! Command-line interface for analyzing and visualizing Rust ownership.

use clap::Parser;

#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(about = "Visualize Rust ownership and borrowing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    /// Run and visualize a Rust file
    Run {
        /// Path to the Rust file
        file: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { file }) => {
            println!("Running: {}", file);
            // Placeholder - will be implemented in Chapter 7
            Ok(())
        }
        None => {
            println!("BorrowScope v{}", env!("CARGO_PKG_VERSION"));
            println!("Use --help for usage information");
            Ok(())
        }
    }
}
