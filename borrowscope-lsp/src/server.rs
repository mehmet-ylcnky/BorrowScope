//! LSP server main loop.

use anyhow::Result;
use lsp_server::{Connection, Message};

use crate::handlers;
use crate::state::GlobalState;
use crate::workspace;

/// Run the main loop. Returns true if shutdown was received, false if connection dropped.
pub fn main_loop(connection: &Connection, mut state: GlobalState) -> Result<bool> {
    // Only attempt workspace loading if Cargo.toml exists
    let cargo_toml = state.root_path.join("Cargo.toml");
    if cargo_toml.exists() {
        tracing::info!("Found Cargo.toml, loading workspace...");
        match workspace::load_workspace(&state.root_path) {
            Ok(ws) => {
                tracing::info!("Workspace ready.");
                state.workspace = Some(ws);
            }
            Err(e) => {
                tracing::error!("Failed to load workspace: {}", e);
            }
        }
    } else {
        tracing::warn!(
            "No Cargo.toml at {:?}, skipping workspace loading",
            state.root_path
        );
    }

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(true);
                }
                handlers::handle_request(&mut state, &connection.sender, req)?;
            }
            Message::Notification(notif) => {
                handlers::handle_notification(&mut state, &connection.sender, notif)?;
            }
            Message::Response(_resp) => {}
        }
    }

    // Connection dropped without shutdown
    Ok(false)
}
