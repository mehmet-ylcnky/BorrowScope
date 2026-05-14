//! LSP server main loop.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Connection, Message, Notification, Request};
use lsp_types::notification::Progress;
use lsp_types::{
    NumberOrString, ProgressParams, ProgressParamsValue, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};
use std::thread;

use crate::handlers;
use crate::state::GlobalState;
use crate::workspace;

/// Run the main loop. Returns true if shutdown was received, false if connection dropped.
pub fn main_loop(connection: &Connection, mut state: GlobalState) -> Result<bool> {
    // Start workspace loading in background if Cargo.toml exists
    let cargo_toml = state.root_path.join("Cargo.toml");
    if cargo_toml.exists() {
        start_background_loading(&connection.sender, &mut state);
    } else {
        tracing::warn!(
            "No Cargo.toml at {:?}, skipping workspace loading",
            state.root_path
        );
    }

    // Message loop - responsive even during loading
    loop {
        // Check if background loading completed
        check_loading_result(&mut state);

        // Check if debounce timer expired
        if let Some(last_change) = state.last_change_time {
            let elapsed = last_change.elapsed();
            if elapsed.as_millis() >= state.debounce_ms as u128 {
                handlers::flush_pending_changes(&mut state, &connection.sender);
            }
        }

        // Wait for message with timeout (to check debounce periodically)
        let timeout = std::time::Duration::from_millis(50);
        let msg = match connection.receiver.recv_timeout(timeout) {
            Ok(msg) => msg,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
        };

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

    Ok(false)
}

fn start_background_loading(sender: &Sender<Message>, state: &mut GlobalState) {
    let token = NumberOrString::String("borrowscope/loading".to_string());

    // Create progress token
    let create_params = WorkDoneProgressCreateParams {
        token: token.clone(),
    };
    let create_req = lsp_server::Request::new(
        lsp_server::RequestId::from("progress_create".to_string()),
        "window/workDoneProgress/create".to_string(),
        serde_json::to_value(create_params).unwrap(),
    );
    sender.send(Message::Request(create_req)).ok();

    // Send begin notification
    send_progress(
        sender,
        &token,
        WorkDoneProgress::Begin(WorkDoneProgressBegin {
            title: "BorrowScope".to_string(),
            message: Some("Loading workspace...".to_string()),
            percentage: Some(0),
            cancellable: Some(false),
        }),
    );

    // Spawn loading thread
    let root_path = state.root_path.clone();
    let (load_sender, load_receiver) = crossbeam_channel::bounded(1);
    state.loading_receiver = Some(load_receiver);

    let progress_sender = sender.clone();
    let progress_token = token.clone();

    thread::spawn(move || {
        tracing::info!("Background workspace loading started...");
        let result = workspace::load_workspace(&root_path);

        // Send end notification
        match &result {
            Ok(_) => send_progress(
                &progress_sender,
                &progress_token,
                WorkDoneProgress::End(WorkDoneProgressEnd {
                    message: Some("Ready".to_string()),
                }),
            ),
            Err(e) => send_progress(
                &progress_sender,
                &progress_token,
                WorkDoneProgress::End(WorkDoneProgressEnd {
                    message: Some(format!("Failed: {}", e)),
                }),
            ),
        }

        load_sender.send(result).ok();
    });
}

fn check_loading_result(state: &mut GlobalState) {
    let receiver = match &state.loading_receiver {
        Some(r) => r,
        None => return,
    };

    // Non-blocking check
    match receiver.try_recv() {
        Ok(Ok(ws)) => {
            tracing::info!("Workspace loaded successfully in background.");
            state.workspace = Some(ws);
            state.loading_receiver = None;
        }
        Ok(Err(e)) => {
            tracing::error!("Background workspace loading failed: {}", e);
            state.loading_receiver = None;
        }
        Err(crossbeam_channel::TryRecvError::Empty) => {
            // Still loading
        }
        Err(crossbeam_channel::TryRecvError::Disconnected) => {
            tracing::error!("Loading thread disconnected unexpectedly");
            state.loading_receiver = None;
        }
    }
}

fn send_progress(sender: &Sender<Message>, token: &NumberOrString, value: WorkDoneProgress) {
    let params = ProgressParams {
        token: token.clone(),
        value: ProgressParamsValue::WorkDone(value),
    };
    let notif = Notification::new("$/progress".to_string(), serde_json::to_value(params).unwrap());
    sender.send(Message::Notification(notif)).ok();
}
