//! Notification handlers.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification};

use crate::state::GlobalState;

pub fn handle(
    _state: &mut GlobalState,
    _sender: &Sender<Message>,
    notif: Notification,
) -> Result<()> {
    match notif.method.as_str() {
        "textDocument/didOpen" => {
            let params: lsp_types::DidOpenTextDocumentParams =
                serde_json::from_value(notif.params)?;
            tracing::debug!("File opened: {}", params.text_document.uri.as_str());
        }
        "textDocument/didChange" => {
            let params: lsp_types::DidChangeTextDocumentParams =
                serde_json::from_value(notif.params)?;
            tracing::debug!("File changed: {}", params.text_document.uri.as_str());
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(notif.params)?;
            tracing::debug!("File closed: {}", params.text_document.uri.as_str());
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
