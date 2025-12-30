//! BorrowScope Analyzer - Static type analysis spike
//!
//! This is a proof-of-concept to test whether we can:
//! 1. Load a Rust project using rust-analyzer crates
//! 2. Query type information for variables
//! 3. Detect specific types (Rc, Arc, unions, statics, FFI)
//!
//! If successful, this will enable the proc macro to have
//! accurate type information without requiring nightly Rust.

mod analysis;
mod output;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    let project_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    println!("BorrowScope Analyzer - Type Analysis Spike");
    println!("==========================================");
    println!("Analyzing project: {}", project_path.display());

    match analysis::analyze_project(&project_path) {
        Ok(type_info) => {
            println!("\n✓ Analysis complete!");
            let json = serde_json::to_string_pretty(&type_info)?;
            println!("\nType Information:\n{}", json);
            
            // Write to .borrowscope/type-info.json
            let output_dir = project_path.join(".borrowscope");
            std::fs::create_dir_all(&output_dir)?;
            let output_path = output_dir.join("type-info.json");
            std::fs::write(&output_path, &json)?;
            println!("\nWritten to: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("\n✗ Analysis failed: {}", e);
            eprintln!("\nThis spike is testing rust-analyzer integration.");
            eprintln!("Errors are expected while we figure out the API.");
        }
    }

    Ok(())
}
