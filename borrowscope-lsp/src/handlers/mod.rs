//! LSP message handlers.

mod notifications;
mod requests;

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification, Request};

use crate::state::GlobalState;

pub fn handle_request(
    state: &mut GlobalState,
    sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    requests::handle(state, sender, req)
}

pub fn handle_notification(
    state: &mut GlobalState,
    sender: &Sender<Message>,
    notif: Notification,
) -> Result<()> {
    notifications::handle(state, sender, notif)
}
