//! BorrowScope Language Server

mod capabilities;
mod handlers;
mod server;
mod state;
mod workspace;

use lsp_server::Connection;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("BorrowScope LSP v{} starting...", env!("CARGO_PKG_VERSION"));

    if std::env::args().any(|arg| arg == "--version") {
        println!("borrowscope-lsp {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let exit_code = match run_server() {
        Ok(true) => 0,  // Clean shutdown
        Ok(false) => 1, // Disconnected without shutdown
        Err(e) => {
            tracing::error!("Server error: {}", e);
            1
        }
    };

    std::process::exit(exit_code);
}

fn run_server() -> anyhow::Result<bool> {
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

    let state = state::GlobalState::new(&params)?;
    let shutdown_received = server::main_loop(&connection, state)?;

    // Drop connection to unblock IO threads
    drop(connection);
    io_threads.join()?;

    Ok(shutdown_received)
}
