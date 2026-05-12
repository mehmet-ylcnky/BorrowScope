//! BorrowScope Language Server
//!
//! A language server that provides real-time ownership visualization
//! for Rust programs using rust-analyzer's semantic engine.

mod capabilities;
mod handlers;
mod server;
mod state;
mod workspace;

use anyhow::Result;
use lsp_server::Connection;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("BorrowScope LSP v{} starting...", env!("CARGO_PKG_VERSION"));

    if std::env::args().any(|arg| arg == "--version") {
        println!("borrowscope-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start()?;
    let params: lsp_types::InitializeParams = serde_json::from_value(initialize_params)?;

    let capabilities = capabilities::server_capabilities();
    let result = serde_json::to_value(lsp_types::InitializeResult {
        capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: "borrowscope-lsp".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    })?;
    connection.initialize_finish(initialize_id, result)?;

    tracing::info!("Initialized. Loading workspace...");

    let state = state::GlobalState::new(&params)?;
    server::main_loop(&connection, state)?;

    io_threads.join()?;
    tracing::info!("BorrowScope LSP shut down.");
    Ok(())
}
