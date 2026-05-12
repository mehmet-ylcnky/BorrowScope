//! Request handlers.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Request, Response};

use crate::state::GlobalState;

pub fn handle(
    _state: &mut GlobalState,
    sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    tracing::debug!("Request: {} (id={})", req.method, req.id);

    // For now, return method-not-found for unhandled requests
    let resp = Response::new_err(
        req.id,
        lsp_server::ErrorCode::MethodNotFound as i32,
        format!("Method not found: {}", req.method),
    );
    sender.send(Message::Response(resp))?;
    Ok(())
}
