//! Output formatting utilities

#![allow(dead_code)] // Will be used in future sections

use clap::ValueEnum;
use colored::*;
use serde::Serialize;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum OutputFormat {
    Json,
    Text,
    None,
}

pub struct Output {
    format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn success(&self, message: &str) {
        match self.format {
            OutputFormat::Text => println!("{} {}", "✓".green().bold(), message),
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "status": "success",
                    "message": message
                });
                println!("{}", serde_json::to_string(&json).unwrap());
            }
            OutputFormat::None => {}
        }
    }

    pub fn error(&self, message: &str) {
        match self.format {
            OutputFormat::Text => eprintln!("{} {}", "✗".red().bold(), message),
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "status": "error",
                    "message": message
                });
                eprintln!("{}", serde_json::to_string(&json).unwrap());
            }
            OutputFormat::None => {}
        }
    }

    pub fn info(&self, message: &str) {
        match self.format {
            OutputFormat::Text => println!("{} {}", "ℹ".blue().bold(), message),
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "status": "info",
                    "message": message
                });
                println!("{}", serde_json::to_string(&json).unwrap());
            }
            OutputFormat::None => {}
        }
    }

    pub fn warning(&self, message: &str) {
        match self.format {
            OutputFormat::Text => println!("{} {}", "⚠".yellow().bold(), message),
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "status": "warning",
                    "message": message
                });
                println!("{}", serde_json::to_string(&json).unwrap());
            }
            OutputFormat::None => {}
        }
    }

    pub fn data<T: Serialize>(&self, data: &T) {
        match self.format {
            OutputFormat::Json => {
                if let Ok(json) = serde_json::to_string_pretty(data) {
                    println!("{}", json);
                }
            }
            OutputFormat::Text => {
                if let Ok(json) = serde_json::to_string_pretty(data) {
                    println!("{}", json);
                }
            }
            OutputFormat::None => {}
        }
    }
}
