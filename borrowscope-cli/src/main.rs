//! BorrowScope CLI
//!
//! Command-line interface for analyzing and visualizing Rust ownership and borrowing.

mod cli;
mod commands;
mod config;
mod error;
mod instrumentation;
mod output;

use clap::Parser;

use crate::cli::Cli;
use crate::error::Result;

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Setup logging based on verbosity
    setup_logging(cli.verbose, cli.quiet);

    // Load configuration
    let config = config::Config::load()?;

    // Execute command
    cli.execute(config)
}

fn setup_logging(verbose: bool, quiet: bool) {
    let level = if quiet {
        log::LevelFilter::Error
    } else if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}
