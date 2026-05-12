//! Notification handlers.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification};
use std::path::PathBuf;

use crate::state::GlobalState;

pub fn handle(
    state: &mut GlobalState,
    _sender: &Sender<Message>,
    notif: Notification,
) -> Result<()> {
    match notif.method.as_str() {
        "textDocument/didOpen" => {
            let params: lsp_types::DidOpenTextDocumentParams =
                serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri.as_str();
            tracing::debug!("File opened: {}", uri);

            if uri.ends_with(".rs") {
                state.set_file_content(uri, params.text_document.text);
            }
        }
        "textDocument/didChange" => {
            let params: lsp_types::DidChangeTextDocumentParams =
                serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri.as_str();
            tracing::debug!("File changed: {}", uri);

            if uri.ends_with(".rs") {
                // Full sync mode: last content change contains the full text
                if let Some(change) = params.content_changes.into_iter().last() {
                    state.set_file_content(uri, change.text);
                }
            }
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(notif.params)?;
            tracing::debug!("File closed: {}", params.text_document.uri.as_str());
            // Do NOT remove from open_files - other files may reference it
            // Just mark as closed for cache eviction purposes
            state.mark_file_closed(params.text_document.uri.as_str());
        }
        "initialized" => {
            tracing::info!("Client initialized.");
        }
        _ => {
            tracing::debug!("Unhandled notification: {}", notif.method);
        }
    }
    Ok(())
}
