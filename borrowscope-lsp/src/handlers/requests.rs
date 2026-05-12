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

    match req.method.as_str() {
        // Debug request: return file content (for testing text sync)
        "borrowscope/debug/fileContent" => {
            let params: serde_json::Value = serde_json::from_value(req.params)?;
            let uri = params["uri"].as_str().unwrap_or("");
            let content = state.get_file_content(uri).unwrap_or("").to_string();
            let resp = Response::new_ok(req.id, serde_json::json!({ "content": content }));
            sender.send(Message::Response(resp))?;
        }
        _ => {
            let resp = Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("Method not found: {}", req.method),
            );
            sender.send(Message::Response(resp))?;
        }
    }

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
