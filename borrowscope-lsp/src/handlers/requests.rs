//! Request handlers.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Request, Response};

use crate::state::GlobalState;

pub fn handle(
    state: &mut GlobalState,
    sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    tracing::debug!("Request: {} (id={})", req.method, req.id);

    // Custom requests that need workspace
    if requires_workspace(&req.method) && !state.is_ready() {
        let resp = Response::new_err(
            req.id,
            lsp_server::ErrorCode::ServerNotInitialized as i32,
            "Workspace not yet loaded".to_string(),
        );
        sender.send(Message::Response(resp))?;
        return Ok(());
    }

    // Dispatch by method
    let resp = Response::new_err(
        req.id,
        lsp_server::ErrorCode::MethodNotFound as i32,
        format!("Method not found: {}", req.method),
    );
    sender.send(Message::Response(resp))?;
    Ok(())
}

fn requires_workspace(method: &str) -> bool {
    matches!(
        method,
        "textDocument/hover"
            | "textDocument/codeLens"
            | "textDocument/inlayHint"
            | "borrowscope/ownershipGraph"
            | "borrowscope/borrowScopes"
            | "borrowscope/variableInfo"
    )
}
