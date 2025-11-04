//! CLI argument definitions and parsing

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::commands;
use crate::config::Config;
use crate::error::Result;
use crate::output::OutputFormat;

#[derive(Parser)]
#[command(name = "borrowscope")]
#[command(version, about = "Visualize Rust ownership and borrowing", long_about = None)]
#[command(author, propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

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
    #[arg(short, long)]
    pub output: Option<PathBuf>,

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
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Don't open browser automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Host address to bind to
    #[arg(long)]
    pub host: Option<String>,
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
    #[arg(long, value_enum, default_value_t = ConfigTemplate::Default)]
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

#[derive(ValueEnum, Clone, Copy, Debug)]
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

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ConfigTemplate {
    Default,
    Minimal,
    Advanced,
}

impl Cli {
    pub fn execute(self, config: Config) -> Result<()> {
        match self.command {
            Commands::Run(args) => commands::run::execute(args, config),
            Commands::Visualize(args) => commands::visualize::execute(args, config),
            Commands::Export(args) => commands::export::execute(args),
            Commands::Init(args) => commands::init::execute(args),
            Commands::Check(args) => commands::check::execute(args),
        }
    }
}
