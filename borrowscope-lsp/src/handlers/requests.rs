//! Request handlers - all using semantic analysis via workspace (no heuristics).

use anyhow::Result;
use crossbeam_channel::Sender;
use lsp_server::{Message, Request, Response};
use serde::Deserialize;

use crate::state::GlobalState;

pub fn handle(
    state: &mut GlobalState,
    sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    tracing::debug!("Request: {} (id={})", req.method, req.id);

    // Requests that need workspace
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
        "borrowscope/ownershipGraph" => handle_ownership_graph(state, sender, req)?,
        "borrowscope/borrowScopes" => handle_borrow_scopes(state, sender, req)?,
        "borrowscope/variableInfo" => handle_variable_info(state, sender, req)?,
        "textDocument/codeLens" => handle_code_lens(state, sender, req)?,
        "textDocument/inlayHint" => handle_inlay_hints(state, sender, req)?,
        "textDocument/hover" => handle_hover(state, sender, req)?,
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

// ═══════════════════════════════════════════════════════════════════════════
// Shared helpers
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct TextDocPositionParams {
    #[serde(rename = "textDocument")]
    text_document: lsp_types::TextDocumentIdentifier,
    position: lsp_types::Position,
}

#[derive(Debug, Deserialize)]
struct TextDocParams {
    #[serde(rename = "textDocument")]
    text_document: lsp_types::TextDocumentIdentifier,
}

#[derive(Debug, Deserialize)]
struct InlayHintParams {
    #[serde(rename = "textDocument")]
    text_document: lsp_types::TextDocumentIdentifier,
    range: lsp_types::Range,
}

macro_rules! get_workspace_or_empty {
    ($state:expr, $req:expr, $sender:expr) => {
        match &$state.workspace {
            Some(ws) => ws,
            None => {
                let resp = Response::new_ok($req.id, serde_json::json!([]));
                $sender.send(Message::Response(resp))?;
                return Ok(());
            }
        }
    };
}

macro_rules! get_file_id_or_empty {
    ($ws:expr, $uri:expr, $req:expr, $sender:expr) => {{
        let file_path = match $uri.strip_prefix("file://") {
            Some(p) => p,
            None => {
                let resp = Response::new_ok($req.id, serde_json::json!([]));
                $sender.send(Message::Response(resp))?;
                return Ok(());
            }
        };
        let vfs_path = ra_ap_vfs::VfsPath::new_real_path(file_path.to_string());
        match $ws.vfs.file_id(&vfs_path) {
            Some((fid, _)) => (fid, file_path.to_string()),
            None => {
                let resp = Response::new_ok($req.id, serde_json::json!([]));
                $sender.send(Message::Response(resp))?;
                return Ok(());
            }
        }
    }};
}

fn build_line_index(content: &str) -> impl Fn(ra_ap_syntax::TextSize) -> (u32, u32) + '_ {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    move |offset: ra_ap_syntax::TextSize| -> (u32, u32) {
        let offset = u32::from(offset) as usize;
        let line = line_starts.partition_point(|&start| start <= offset) as u32;
        let col = offset - line_starts.get(line.saturating_sub(1) as usize).copied().unwrap_or(0);
        (line, col as u32)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.1 borrowscope/ownershipGraph
// ═══════════════════════════════════════════════════════════════════════════

fn handle_ownership_graph(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: TextDocPositionParams = serde_json::from_value(req.params)?;
    let ws = get_workspace_or_empty!(state, req, sender);
    let uri_str = params.text_document.uri.as_str();
    let (file_id, file_path) = get_file_id_or_empty!(ws, uri_str, req, sender);

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};
    use ra_ap_syntax::ast::HasName;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };

    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);
    let target_line = params.position.line;

    let function = source_file.syntax().descendants().filter_map(ast::Fn::cast).find(|f| {
        let (fn_start, _) = line_index(f.syntax().text_range().start());
        let (fn_end, _) = line_index(f.syntax().text_range().end());
        target_line >= fn_start.saturating_sub(1) && target_line <= fn_end
    });

    let function = match function {
        Some(f) => f,
        None => { sender.send(Message::Response(Response::new_err(req.id, -32602, "Cursor not inside a function".into())))?; return Ok(()); }
    };

    let summary = attach_db(&ws.db, || {
        borrowscope_lsp::analysis::analyze_function(&ws.db, &sema, &display_target, &function, &file_path, &line_index)
    });

    sender.send(Message::Response(Response::new_ok(req.id, serde_json::to_value(&summary)?)))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.2 borrowscope/borrowScopes
// ═══════════════════════════════════════════════════════════════════════════

fn handle_borrow_scopes(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: TextDocParams = serde_json::from_value(req.params)?;
    let ws = get_workspace_or_empty!(state, req, sender);
    let uri_str = params.text_document.uri.as_str();
    let (file_id, file_path) = get_file_id_or_empty!(ws, uri_str, req, sender);

    use ra_ap_hir::{self as hir, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);

    let scopes = attach_db(&ws.db, || {
        let mut all = Vec::new();
        for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
            let fn_scopes = borrowscope_lsp::analysis::compute_borrow_scopes(&ws.db, &sema, &function, &line_index);
            for s in fn_scopes {
                all.push(serde_json::json!({
                    "borrower": s.borrower_name, "target": s.target_name, "is_mutable": s.is_mutable,
                    "range": { "start": {"line": s.start_line, "character": s.start_col}, "end": {"line": s.end_line, "character": s.end_col} }
                }));
            }
        }
        all
    });

    sender.send(Message::Response(Response::new_ok(req.id, serde_json::json!({"scopes": scopes}))))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.3 borrowscope/variableInfo
// ═══════════════════════════════════════════════════════════════════════════

fn handle_variable_info(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: TextDocPositionParams = serde_json::from_value(req.params)?;
    let ws = match &state.workspace {
        Some(ws) => ws,
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };
    let uri_str = params.text_document.uri.as_str();
    let file_path = match uri_str.strip_prefix("file://") {
        Some(p) => p.to_string(),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };
    let vfs_path = ra_ap_vfs::VfsPath::new_real_path(file_path.clone());
    let file_id = match ws.vfs.file_id(&vfs_path) {
        Some((fid, _)) => fid,
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};
    use ra_ap_syntax::ast::HasName;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };

    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);
    let target_line = params.position.line;

    let result = attach_db(&ws.db, || {
        let function = source_file.syntax().descendants().filter_map(ast::Fn::cast).find(|f| {
            let (s, _) = line_index(f.syntax().text_range().start());
            let (e, _) = line_index(f.syntax().text_range().end());
            target_line >= s.saturating_sub(1) && target_line <= e
        })?;

        let summary = borrowscope_lsp::analysis::analyze_function(&ws.db, &sema, &display_target, &function, &file_path, &line_index);
        let var = summary.variables.iter().find(|v| v.line == target_line + 1)?;

        let borrowed_by: Vec<String> = summary.borrow_scopes.iter().filter(|s| s.target_name == var.name).map(|s| s.borrower_name.clone()).collect();
        let moved_to = summary.moves.iter().find(|m| m.source_name == var.name).map(|m| format!("{:?}", m.destination));

        Some(serde_json::json!({
            "name": var.name, "type_display": var.type_display, "ownership_category": var.ownership_category,
            "is_copy": var.is_copy, "borrowed_by": borrowed_by,
            "borrows_from": summary.borrow_scopes.iter().filter(|s| s.borrower_name == var.name).map(|s| s.target_name.clone()).collect::<Vec<_>>(),
            "moved_to": moved_to, "traits": var.trait_impls, "layout_size": var.layout_size,
        }))
    });

    sender.send(Message::Response(Response::new_ok(req.id, result.unwrap_or(serde_json::Value::Null))))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// textDocument/hover — ownership info on hover
// ═══════════════════════════════════════════════════════════════════════════

fn handle_hover(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: TextDocPositionParams = serde_json::from_value(req.params)?;
    let ws = match &state.workspace {
        Some(ws) => ws,
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };
    let uri_str = params.text_document.uri.as_str();
    let file_path = match uri_str.strip_prefix("file://") {
        Some(p) => p.to_string(),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };
    let vfs_path = ra_ap_vfs::VfsPath::new_real_path(file_path.clone());
    let file_id = match ws.vfs.file_id(&vfs_path) {
        Some((fid, _)) => fid,
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};
    use ra_ap_syntax::ast::HasName;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::Value::Null)))?; return Ok(()); }
    };

    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);
    let target_line = params.position.line;

    let hover = attach_db(&ws.db, || -> Option<String> {
        let function = source_file.syntax().descendants().filter_map(ast::Fn::cast).find(|f| {
            let (s, _) = line_index(f.syntax().text_range().start());
            let (e, _) = line_index(f.syntax().text_range().end());
            target_line >= s.saturating_sub(1) && target_line <= e
        })?;

        let summary = borrowscope_lsp::analysis::analyze_function(&ws.db, &sema, &display_target, &function, &file_path, &line_index);
        let var = summary.variables.iter().find(|v| v.line == target_line + 1)?;

        let borrowed_by: Vec<&str> = summary.borrow_scopes.iter()
            .filter(|s| s.target_name == var.name)
            .map(|s| s.borrower_name.as_str()).collect();
        let borrows_from: Vec<&str> = summary.borrow_scopes.iter()
            .filter(|s| s.borrower_name == var.name)
            .map(|s| s.target_name.as_str()).collect();
        let moved_to = summary.moves.iter()
            .find(|m| m.source_name == var.name)
            .map(|m| format!("{:?}", m.destination));

        let mut md = format!("**{}** `{}`\n\n", var.name, var.type_display);
        md.push_str(&format!("**Ownership:** `{:?}`\n\n", var.ownership_category));

        if var.is_copy { md.push_str("• Copy type\n\n"); }
        if !borrows_from.is_empty() { md.push_str(&format!("• Borrows from: `{}`\n\n", borrows_from.join("`, `"))); }
        if !borrowed_by.is_empty() { md.push_str(&format!("• Borrowed by: `{}`\n\n", borrowed_by.join("`, `"))); }
        if let Some(dest) = moved_to { md.push_str(&format!("• Moved to: `{}`\n\n", dest)); }
        if let Some(size) = var.layout_size { md.push_str(&format!("• Size: {} bytes\n\n", size)); }

        Some(md)
    });

    let result = match hover {
        Some(content) => serde_json::json!({
            "contents": { "kind": "markdown", "value": content }
        }),
        None => serde_json::Value::Null,
    };

    sender.send(Message::Response(Response::new_ok(req.id, result)))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.6 textDocument/codeLens
// ═══════════════════════════════════════════════════════════════════════════

fn handle_code_lens(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: TextDocParams = serde_json::from_value(req.params)?;
    let ws = get_workspace_or_empty!(state, req, sender);
    let uri_str = params.text_document.uri.as_str();
    let (file_id, file_path) = get_file_id_or_empty!(ws, uri_str, req, sender);

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};
    use ra_ap_syntax::ast::HasName;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::json!([]))))?; return Ok(()); }
    };

    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);

    let lenses = attach_db(&ws.db, || {
        let mut lenses = Vec::new();
        for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
            let fn_name = match function.name() { Some(n) => n.text().to_string(), None => continue };
            let summary = borrowscope_lsp::analysis::analyze_function(&ws.db, &sema, &display_target, &function, &file_path, &line_index);
            let (fn_line, _) = line_index(function.syntax().text_range().start());
            let title = if summary.stats.conflicts > 0 {
                format!("{} vars, {} borrows, {} moves, {} conflicts!", summary.stats.total_variables, summary.stats.total_borrows, summary.stats.moves, summary.stats.conflicts)
            } else {
                format!("{} vars, {} borrows, {} moves", summary.stats.total_variables, summary.stats.total_borrows, summary.stats.moves)
            };
            lenses.push(serde_json::json!({"range": {"start": {"line": fn_line.saturating_sub(1), "character": 0}, "end": {"line": fn_line.saturating_sub(1), "character": 0}}, "command": {"title": title, "command": "borrowscope.showGraph", "arguments": [uri_str, fn_name]}}));
        }
        lenses
    });

    sender.send(Message::Response(Response::new_ok(req.id, serde_json::json!(lenses))))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.7 textDocument/inlayHint
// ═══════════════════════════════════════════════════════════════════════════

fn handle_inlay_hints(state: &mut GlobalState, sender: &Sender<Message>, req: Request) -> Result<()> {
    let params: InlayHintParams = serde_json::from_value(req.params)?;
    let ws = get_workspace_or_empty!(state, req, sender);
    let uri_str = params.text_document.uri.as_str();
    let (file_id, file_path) = get_file_id_or_empty!(ws, uri_str, req, sender);

    use ra_ap_hir::{self as hir, DisplayTarget, Semantics};
    use ra_ap_hir_ty::attach_db;
    use ra_ap_syntax::{ast, AstNode};

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(sema.attach_first_edition(file_id));
    let display_target = match hir::Crate::all(&ws.db).first() {
        Some(k) => DisplayTarget::from_crate(&ws.db, (*k).into()),
        None => { sender.send(Message::Response(Response::new_ok(req.id, serde_json::json!([]))))?; return Ok(()); }
    };

    let file_content = state.get_file_content(uri_str).unwrap_or("");
    let line_index = build_line_index(file_content);
    let (start_line, end_line) = (params.range.start.line, params.range.end.line);

    let hints = attach_db(&ws.db, || {
        let mut hints = Vec::new();
        for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
            let summary = borrowscope_lsp::analysis::analyze_function(&ws.db, &sema, &display_target, &function, &file_path, &line_index);
            for var in &summary.variables {
                let var_line = var.line.saturating_sub(1);
                if var_line < start_line || var_line > end_line { continue; }
                let label = match &var.ownership_category {
                    borrowscope_lsp::analysis::OwnershipCategory::SharedRef => Some("[&]"),
                    borrowscope_lsp::analysis::OwnershipCategory::MutableRef => Some("[&mut]"),
                    borrowscope_lsp::analysis::OwnershipCategory::Rc => Some("[Rc]"),
                    borrowscope_lsp::analysis::OwnershipCategory::Arc => Some("[Arc]"),
                    borrowscope_lsp::analysis::OwnershipCategory::InteriorMut => Some("[Cell]"),
                    borrowscope_lsp::analysis::OwnershipCategory::RawPointer => Some("[*ptr]"),
                    _ => if var.is_closure { Some("[closure]") } else { None },
                };
                if let Some(label) = label {
                    hints.push(serde_json::json!({"position": {"line": var_line, "character": var.column + var.name.len() as u32}, "label": format!(" {}", label), "kind": 1, "paddingLeft": true}));
                }
            }
        }
        hints
    });

    sender.send(Message::Response(Response::new_ok(req.id, serde_json::json!(hints))))?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════

fn requires_workspace(method: &str) -> bool {
    matches!(method,
        "textDocument/hover" | "borrowscope/ownershipGraph" | "borrowscope/borrowScopes" | "borrowscope/variableInfo"
    )
}
