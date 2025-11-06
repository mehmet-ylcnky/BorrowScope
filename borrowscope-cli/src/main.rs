//! BorrowScope CLI
//!
//! Command-line interface for analyzing and visualizing Rust ownership and borrowing.

mod async_utils;
mod cargo;
mod cli;
mod commands;
mod config;
mod error;
mod graphviz;
mod instrumentation;
mod output;
mod progress;
mod server;
mod utils;

use clap::Parser;
use std::process;

use crate::cli::Cli;
use crate::error::{print_error, Result};

fn main() {
    if let Err(e) = run() {
        print_error(&e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
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
