//! Notification handlers.

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Notification};

use crate::state::GlobalState;

pub fn handle(
    state: &mut GlobalState,
    sender: &Sender<Message>,
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
            let uri = params.text_document.uri.as_str().to_string();
            tracing::debug!("File changed: {}", uri);

            if uri.ends_with(".rs") {
                // Store previous content for later diffing
                let previous = state.get_file_content(&uri).map(|s| s.to_string());

                // Full sync mode: last content change contains the full text
                if let Some(change) = params.content_changes.into_iter().last() {
                    state.set_file_content(&uri, change.text);
                }

                // Mark as pending for debounced analysis
                if !state.pending_changes.iter().any(|(u, _)| u == &uri) {
                    state.pending_changes.push((uri, previous));
                }
                state.last_change_time = Some(std::time::Instant::now());
            }
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri.as_str();
            tracing::debug!("File closed: {}", uri);
            state.analysis_cache.remove(uri);
            state.mark_file_closed(uri);
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

/// Flush pending debounced changes: apply to Salsa, send notifications.
/// Called from the main loop when debounce timer expires.
pub fn flush_pending_changes(state: &mut GlobalState, sender: &Sender<Message>) {
    let pending = std::mem::take(&mut state.pending_changes);
    if pending.is_empty() {
        return;
    }

    // Mark cached analysis as stale for changed files
    for (uri, _) in &pending {
        if let Some(cache) = state.analysis_cache.get_mut(uri) {
            cache.mark_all_stale();
        }
    }

    // Apply to Salsa database
    state.apply_vfs_changes();

    // Send notifications for each changed file
    for (uri, prev_content) in &pending {
        let new_content = state.get_file_content(uri).map(|s| s.to_string());
        send_analysis_updated_if_changed(sender, uri, prev_content.as_deref(), new_content.as_deref());
        publish_diagnostics(sender, uri, state);
    }

    state.last_change_time = None;
}

/// Publish borrow conflict diagnostics via standard LSP publishDiagnostics.
/// Requires workspace for semantic analysis. No heuristics.
fn publish_diagnostics(sender: &Sender<Message>, uri: &str, state: &GlobalState) {
    let ws = match &state.workspace {
        Some(ws) => ws,
        None => {
            // No workspace = no diagnostics (no heuristic fallback)
            send_diagnostics(sender, uri, vec![]);
            return;
        }
    };

    let file_path = match uri.strip_prefix("file://") {
        Some(p) => p,
        None => return,
    };

    let vfs_path = ra_ap_vfs::VfsPath::new_real_path(file_path.to_string());
    let file_id = match ws.vfs.file_id(&vfs_path) {
        Some((fid, _)) => fid,
        None => {
            send_diagnostics(sender, uri, vec![]);
            return;
        }
    };

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode, TextSize};

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));

    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => {
            send_diagnostics(sender, uri, vec![]);
            return;
        }
    };

    let file_content = state.get_file_content(uri).unwrap_or("");
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(file_content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_index = |offset: TextSize| -> (u32, u32) {
        let offset = u32::from(offset) as usize;
        let line = line_starts.partition_point(|&start| start <= offset) as u32;
        let col = offset - line_starts.get(line.saturating_sub(1) as usize).copied().unwrap_or(0);
        (line, col as u32)
    };

    // Collect all conflicts from all functions
    let all_diagnostics = attach_db(&ws.db, || {
        let mut diagnostics: Vec<serde_json::Value> = Vec::new();

        for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
            let summary = borrowscope_lsp::analysis::analyze_function(
                &ws.db, &sema, &display_target, &function, file_path, &line_index,
            );

            for c in &summary.conflicts {
                let message = match &c.kind {
                    borrowscope_lsp::analysis::ConflictKind::MutableAndShared => format!(
                        "Ownership insight: `{}` (shared) and `{}` (mutable) both borrow `{}` with overlapping scopes (lines {}-{})",
                        c.borrow_a, c.borrow_b, c.variable, c.overlap_start_line, c.overlap_end_line
                    ),
                    borrowscope_lsp::analysis::ConflictKind::MultipleMutable => format!(
                        "Ownership insight: `{}` and `{}` both hold mutable borrows of `{}` (lines {}-{})",
                        c.borrow_a, c.borrow_b, c.variable, c.overlap_start_line, c.overlap_end_line
                    ),
                };

                diagnostics.push(serde_json::json!({
                    "range": {
                        "start": {"line": c.overlap_start_line.saturating_sub(1), "character": 0},
                        "end": {"line": c.overlap_end_line.saturating_sub(1), "character": 0}
                    },
                    "severity": 3,
                    "source": "BorrowScope",
                    "message": message,
                    "relatedInformation": [
                        {
                            "location": {
                                "uri": uri,
                                "range": {"start": {"line": c.overlap_start_line.saturating_sub(1), "character": 0},
                                          "end": {"line": c.overlap_start_line.saturating_sub(1), "character": 100}}
                            },
                            "message": format!("First borrow (`{}`) here", c.borrow_a)
                        },
                        {
                            "location": {
                                "uri": uri,
                                "range": {"start": {"line": c.overlap_end_line.saturating_sub(1), "character": 0},
                                          "end": {"line": c.overlap_end_line.saturating_sub(1), "character": 100}}
                            },
                            "message": format!("Second borrow (`{}`) here", c.borrow_b)
                        }
                    ]
                }));
            }
        }

        diagnostics
    });

    send_diagnostics(sender, uri, all_diagnostics);
}

fn send_diagnostics(sender: &Sender<Message>, uri: &str, diagnostics: Vec<serde_json::Value>) {
    let params = serde_json::json!({
        "uri": uri,
        "diagnostics": diagnostics
    });
    let notif = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    sender.send(Message::Notification(notif)).ok();
}

/// Send analysisUpdated only if the change affects function ownership structure.
/// Compares function signatures/bodies between old and new content.
fn send_analysis_updated_if_changed(
    sender: &Sender<Message>,
    uri: &str,
    previous: Option<&str>,
    current: Option<&str>,
) {
    let current = match current {
        Some(c) => c,
        None => return,
    };

    let new_functions = extract_function_names(current);

    // If no previous content, this is effectively a new file - send notification
    let previous = match previous {
        Some(p) => p,
        None => {
            if !new_functions.is_empty() {
                send_notification(sender, uri, &new_functions);
            }
            return;
        }
    };

    // Find which functions were affected (added, removed, or body changed)
    let old_functions = extract_function_names(previous);
    let old_bodies = extract_function_bodies(previous);
    let new_bodies = extract_function_bodies(current);

    let mut affected: Vec<String> = Vec::new();

    // New functions
    for f in &new_functions {
        if !old_functions.contains(f) {
            affected.push(f.clone());
        }
    }

    // Removed functions
    for f in &old_functions {
        if !new_functions.contains(f) {
            affected.push(f.clone());
        }
    }

    // Changed functions (body differs)
    for f in &new_functions {
        if old_functions.contains(f) {
            let old_body = old_bodies.get(f.as_str()).map(|s| s.as_str()).unwrap_or("");
            let new_body = new_bodies.get(f.as_str()).map(|s| s.as_str()).unwrap_or("");
            if old_body != new_body {
                if !affected.contains(f) {
                    affected.push(f.clone());
                }
            }
        }
    }

    // Only send if something actually changed
    if !affected.is_empty() {
        send_notification(sender, uri, &affected);
    }
}

fn send_notification(sender: &Sender<Message>, uri: &str, functions: &[String]) {
    let params = serde_json::json!({
        "uri": uri,
        "functions": functions,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let notif = Notification::new("borrowscope/analysisUpdated".to_string(), params);
    sender.send(Message::Notification(notif)).ok();
}

/// Extract function names from source content.
fn extract_function_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| extract_fn_name(line))
        .collect()
}

/// Extract function name from a line.
fn extract_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let after_fn = trimmed
        .strip_prefix("pub async fn ")
        .or_else(|| trimmed.strip_prefix("pub fn "))
        .or_else(|| trimmed.strip_prefix("async fn "))
        .or_else(|| trimmed.strip_prefix("fn "))?;
    let name = after_fn.split(|c: char| c == '(' || c == '<' || c == ' ').next()?;
    if !name.is_empty() {
        Some(name.to_string())
    } else {
        None
    }
}

/// Extract function bodies as a map of name -> body text.
fn extract_function_bodies(content: &str) -> std::collections::HashMap<String, String> {
    let mut bodies = std::collections::HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some(name) = extract_fn_name(lines[i]) {
            let mut depth = 0i32;
            let mut body = String::new();
            let mut found_open = false;
            let mut end = i;
            for j in i..lines.len() {
                body.push_str(lines[j]);
                body.push('\n');
                depth += lines[j].matches('{').count() as i32;
                if depth > 0 {
                    found_open = true;
                }
                depth -= lines[j].matches('}').count() as i32;
                if found_open && depth <= 0 {
                    end = j;
                    break;
                }
            }
            bodies.insert(name, body);
            i = end + 1;
        } else {
            i += 1;
        }
    }

    bodies
}
