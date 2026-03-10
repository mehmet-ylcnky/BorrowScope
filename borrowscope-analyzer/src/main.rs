//! BorrowScope Analyzer - Production-ready static type analysis
//!
//! Uses rust-analyzer's semantic analysis to extract accurate type information
//! for all variables in a Rust project. This enables the BorrowScope proc macro
//! to emit precise tracking calls.
//!
//! # Architecture
//!
//! ```text
//! cargo borrowscope analyze  →  .borrowscope/type-info.json
//! cargo build                →  #[trace_borrow] reads JSON, emits correct calls
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run -p borrowscope-analyzer -- /path/to/project
//! ```

mod analysis;
mod output;

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("borrowscope_analyzer=info".parse()?)
                .add_directive("ra_ap=warn".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    let project_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    println!("BorrowScope Analyzer v{}", env!("CARGO_PKG_VERSION"));
    println!("═══════════════════════════════════════════");
    println!("Project: {}", project_path.display());
    println!();

    let mut type_info = analysis::analyze_project(&project_path)?;

    // Summary
    let total_vars: usize = type_info.files.values().map(|v| v.len()).sum();
    let resolved: usize = type_info
        .files
        .values()
        .flat_map(|v| v.iter())
        .filter(|v| v.ty != "unknown" && !v.ty.contains("{unknown}"))
        .count();

    println!();
    println!("═══════════════════════════════════════════");
    println!("Summary:");
    println!("  Files analyzed: {}", type_info.files.len());
    println!("  Variables found: {}", total_vars);
    println!(
        "  Types resolved: {} ({:.1}%)",
        resolved,
        if total_vars > 0 {
            resolved as f64 / total_vars as f64 * 100.0
        } else {
            0.0
        }
    );

    // Build lookup indices for macro consumption
    type_info.build_name_index();

    // Write output
    let output_dir = project_path.join(".borrowscope");
    std::fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join("type-info.json");
    let json = serde_json::to_string_pretty(&type_info)?;
    std::fs::write(&output_path, &json)?;

    println!();
    println!("Output: {}", output_path.display());
    info!("Analysis complete");

    Ok(())
}
