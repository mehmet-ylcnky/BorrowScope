//! Error types and handling for the CLI

#![allow(dead_code)] // Some variants will be used in future sections

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl CliError {
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            CliError::FileNotFound(_) => {
                Some("Check the file path or run 'borrowscope run' first to generate tracking data")
            }
            CliError::PortInUse(_) => Some("Try a different port with --port <PORT>"),
            CliError::CommandNotFound(cmd) if cmd == "cargo" => {
                Some("Install Rust and Cargo from https://rustup.rs")
            }
            _ => None,
        }
    }
}
