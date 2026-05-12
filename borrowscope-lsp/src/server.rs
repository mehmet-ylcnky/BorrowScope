//! LSP server main loop.

use anyhow::Result;
use lsp_server::{Connection, Message};

use crate::handlers;
use crate::state::GlobalState;
use crate::workspace;

pub fn main_loop(connection: &Connection, mut state: GlobalState) -> Result<()> {
    // Load workspace (blocking for now; M1.7 will make this async with progress)
    match workspace::load_workspace(&state.root_path) {
        Ok(ws) => {
            tracing::info!("Workspace ready.");
            state.workspace = Some(ws);
        }
        Err(e) => {
            tracing::error!("Failed to load workspace: {}", e);
            // Continue running - some features may work without full workspace
        }
    }

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                handlers::handle_request(&mut state, &connection.sender, req)?;
            }
            Message::Notification(notif) => {
                handlers::handle_notification(&mut state, &connection.sender, notif)?;
            }
            Message::Response(_resp) => {
                // We don't send requests to the client yet
            }
        }
    }

    Ok(())
}
