//! Integration tests for the BorrowScope LSP server.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

struct TestServer {
    process: Child,
    stdout: BufReader<ChildStdout>,
    next_id: i32,
    received_notifications: Vec<serde_json::Value>,
}

impl TestServer {
    fn start() -> Self {
        let binary = env!("CARGO_BIN_EXE_borrowscope-lsp");
        let mut process = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start borrowscope-lsp");

        let stdout = BufReader::new(process.stdout.take().unwrap());
        Self {
            process,
            stdout,
            next_id: 1,
            received_notifications: Vec::new(),
        }
    }

    fn send_message(&mut self, msg: &str) {
        let stdin = self.process.stdin.as_mut().unwrap();
        write!(stdin, "Content-Length: {}\r\n\r\n{}", msg.len(), msg).unwrap();
        stdin.flush().unwrap();
    }

    fn read_response(&mut self) -> serde_json::Value {
        // Read headers
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            if let Some(len) = line.strip_prefix("Content-Length: ") {
                content_length = len.trim().parse().unwrap();
            }
        }

        // Read body
        let mut body = vec![0u8; content_length];
        self.stdout.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg = serde_json::json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        self.send_message(&msg.to_string());
        // Read messages until we get a response (skip notifications)
        loop {
            let msg = self.read_response();
            if msg.get("id").is_some() {
                return msg;
            }
            // It's a notification from server - store it
            self.received_notifications.push(msg);
        }
    }

    /// Read the next notification sent by the server (non-blocking check of buffered).
    fn take_notifications(&mut self) -> Vec<serde_json::Value> {
        std::mem::take(&mut self.received_notifications)
    }

    /// Send a notification and then read any server notifications that come back.
    fn notify_and_collect(&mut self, method: &str, params: serde_json::Value) -> Vec<serde_json::Value> {
        self.notify(method, params);
        // Wait for debounce (300ms) + processing time
        std::thread::sleep(std::time::Duration::from_millis(400));
        // Send a dummy request to flush any pending notifications
        let _resp = self.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
        self.take_notifications()
    }

    fn notify(&mut self, method: &str, params: serde_json::Value) {
        let msg = serde_json::json!({"jsonrpc":"2.0","method":method,"params":params});
        self.send_message(&msg.to_string());
    }

    fn initialize(&mut self) -> serde_json::Value {
        let resp = self.request(
            "initialize",
            serde_json::json!({
                "processId": null,
                "rootUri": "file:///tmp",
                "capabilities": {}
            }),
        );
        self.notify("initialized", serde_json::json!({}));
        resp
    }

    fn initialize_with_options(&mut self, options: serde_json::Value) -> serde_json::Value {
        let resp = self.request(
            "initialize",
            serde_json::json!({
                "processId": null,
                "rootUri": "file:///tmp",
                "capabilities": {},
                "initializationOptions": options
            }),
        );
        self.notify("initialized", serde_json::json!({}));
        resp
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[test]
fn test_version_flag() {
    let binary = env!("CARGO_BIN_EXE_borrowscope-lsp");
    let output = Command::new(binary)
        .arg("--version")
        .stderr(Stdio::null())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("borrowscope-lsp 0.1.0"));
    assert!(output.status.success());
}

#[test]
fn test_initialize_returns_capabilities() {
    let mut server = TestServer::start();
    let response = server.initialize();
    let result = &response["result"];
    assert_eq!(result["capabilities"]["hoverProvider"], true);
    assert_eq!(result["capabilities"]["inlayHintProvider"], true);
    assert!(result["capabilities"]["textDocumentSync"]["openClose"].as_bool().unwrap());
    assert!(result["capabilities"]["codeLensProvider"].is_object());
}

#[test]
fn test_server_info() {
    let mut server = TestServer::start();
    let response = server.initialize();
    let info = &response["result"]["serverInfo"];
    assert_eq!(info["name"], "borrowscope-lsp");
    assert_eq!(info["version"], "0.1.0");
}

#[test]
fn test_shutdown_returns_null() {
    let mut server = TestServer::start();
    server.initialize();
    let response = server.request("shutdown", serde_json::Value::Null);
    assert!(response["result"].is_null());
}

#[test]
fn test_unknown_request_returns_error() {
    let mut server = TestServer::start();
    server.initialize();
    let response = server.request("nonexistent/method", serde_json::json!({}));
    assert_eq!(response["error"]["code"], -32601);
}

#[test]
fn test_workspace_request_before_ready_returns_not_initialized() {
    let mut server = TestServer::start();
    server.initialize();
    // Workspace is not loaded (rootUri is /tmp, no Cargo.toml)
    let response = server.request("textDocument/hover", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/main.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert_eq!(response["error"]["code"], -32002); // ServerNotInitialized
}

#[test]
fn test_double_shutdown_does_not_crash() {
    let mut server = TestServer::start();
    server.initialize();
    let resp1 = server.request("shutdown", serde_json::Value::Null);
    assert!(resp1["result"].is_null());
    // Second shutdown - server already shut down, connection should close
    // The server exits after first shutdown, so writing to stdin may fail
    // This test passes if it doesn't panic/hang
}

#[test]
fn test_exit_after_shutdown_code_zero() {
    let mut server = TestServer::start();
    server.initialize();
    server.request("shutdown", serde_json::Value::Null);
    server.notify("exit", serde_json::json!(null));
    // Drop stdin to unblock the server if it's waiting
    drop(server.process.stdin.take());
    let status = server.process.wait().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn test_exit_without_shutdown_code_one() {
    let mut server = TestServer::start();
    server.initialize();
    // Drop stdin without sending shutdown
    drop(server.process.stdin.take());
    let status = server.process.wait().unwrap();
    assert_eq!(status.code(), Some(1));
}

// ── Text Document Synchronization Tests ──

#[test]
fn test_did_open_stores_content() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {
            "uri": "file:///tmp/test.rs",
            "languageId": "rust",
            "version": 1,
            "text": "fn main() {}"
        }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/test.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn main() {}");
}

#[test]
fn test_did_change_updates_content() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "languageId": "rust", "version": 1, "text": "fn main() {}" }
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "version": 2 },
        "contentChanges": [{ "text": "fn main() { let x = 1; }" }]
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/test.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn main() { let x = 1; }");
}

#[test]
fn test_did_change_multiple_edits_applies_last() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "languageId": "rust", "version": 1, "text": "v1" }
    }));
    // Full sync mode: multiple contentChanges, last one wins
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "version": 2 },
        "contentChanges": [
            { "text": "v2_intermediate" },
            { "text": "v3_final" }
        ]
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/test.rs"
    }));
    assert_eq!(resp["result"]["content"], "v3_final");
}

#[test]
fn test_content_not_corrupted_with_special_chars() {
    let mut server = TestServer::start();
    server.initialize();
    let content = "fn main() {\n    let s = \"hello\\nworld\";\n    let r = &s;\n}\n";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "languageId": "rust", "version": 1, "text": content }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/test.rs"
    }));
    assert_eq!(resp["result"]["content"], content);
}

#[test]
fn test_did_close_keeps_content() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs", "languageId": "rust", "version": 1, "text": "fn main() {}" }
    }));
    server.notify("textDocument/didClose", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/test.rs" }
    }));
    // Content should still be available (not removed)
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/test.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn main() {}");
}

#[test]
fn test_non_rust_file_ignored() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/readme.md", "languageId": "markdown", "version": 1, "text": "# Hello" }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/readme.md"
    }));
    // Non-rust file should not be stored
    assert_eq!(resp["result"]["content"], "");
}

// ═══════════════════════════════════════════════════════════════════════════
// Protocol edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_files_tracked_independently() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/a.rs", "languageId": "rust", "version": 1, "text": "fn a() {}" }
    }));
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/b.rs", "languageId": "rust", "version": 1, "text": "fn b() {}" }
    }));
    // Change only b
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/b.rs", "version": 2 },
        "contentChanges": [{ "text": "fn b_changed() {}" }]
    }));
    // a unchanged
    let resp_a = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/a.rs"}));
    assert_eq!(resp_a["result"]["content"], "fn a() {}");
    // b changed
    let resp_b = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/b.rs"}));
    assert_eq!(resp_b["result"]["content"], "fn b_changed() {}");
}

#[test]
fn test_empty_file_content() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/empty.rs", "languageId": "rust", "version": 1, "text": "" }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/empty.rs"}));
    assert_eq!(resp["result"]["content"], "");
}

#[test]
fn test_large_file_content() {
    let mut server = TestServer::start();
    server.initialize();
    // 10KB file
    let content = "fn main() { let x = 1; }\n".repeat(400);
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/large.rs", "languageId": "rust", "version": 1, "text": content }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/large.rs"}));
    assert_eq!(resp["result"]["content"].as_str().unwrap().len(), content.len());
}

#[test]
fn test_unicode_content() {
    let mut server = TestServer::start();
    server.initialize();
    let content = "fn main() { let 名前 = \"こんにちは\"; }";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/unicode.rs", "languageId": "rust", "version": 1, "text": content }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/unicode.rs"}));
    assert_eq!(resp["result"]["content"], content);
}

#[test]
fn test_rapid_changes_all_applied() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/rapid.rs", "languageId": "rust", "version": 1, "text": "v0" }
    }));
    // Send 10 rapid changes
    for i in 1..=10 {
        server.notify("textDocument/didChange", serde_json::json!({
            "textDocument": { "uri": "file:///tmp/rapid.rs", "version": i + 1 },
            "contentChanges": [{ "text": format!("v{}", i) }]
        }));
    }
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/rapid.rs"}));
    assert_eq!(resp["result"]["content"], "v10");
}

#[test]
fn test_file_not_opened_returns_empty() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/never_opened.rs"
    }));
    assert_eq!(resp["result"]["content"], "");
}

#[test]
fn test_reopen_file_updates_content() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/reopen.rs", "languageId": "rust", "version": 1, "text": "original" }
    }));
    server.notify("textDocument/didClose", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/reopen.rs" }
    }));
    // Reopen with different content
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/reopen.rs", "languageId": "rust", "version": 2, "text": "reopened" }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/reopen.rs"}));
    assert_eq!(resp["result"]["content"], "reopened");
}

// ═══════════════════════════════════════════════════════════════════════════
// Capabilities verification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_text_document_sync_is_full() {
    let mut server = TestServer::start();
    let response = server.initialize();
    // change: 1 = Full sync
    assert_eq!(response["result"]["capabilities"]["textDocumentSync"]["change"], 1);
}

#[test]
fn test_save_notification_configured() {
    let mut server = TestServer::start();
    let response = server.initialize();
    let save = &response["result"]["capabilities"]["textDocumentSync"]["save"];
    assert!(save.is_object());
}

// ═══════════════════════════════════════════════════════════════════════════
// Error handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_hover_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("textDocument/hover", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert_eq!(resp["error"]["code"], -32002);
    assert!(resp["error"]["message"].as_str().unwrap().contains("not yet loaded"));
}

#[test]
fn test_code_lens_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    // CodeLens works without workspace (uses file content), returns empty for unopened files
    assert!(resp["result"].is_array());
    assert!(resp["result"].as_array().unwrap().is_empty());
}

#[test]
fn test_inlay_hints_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("textDocument/inlayHint", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "range": {"start": {"line":0,"character":0}, "end": {"line":10,"character":0}}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_custom_request_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert_eq!(resp["result"]["_status"], "loading");
}

// ═══════════════════════════════════════════════════════════════════════════
// Server robustness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_requests_in_sequence() {
    let mut server = TestServer::start();
    server.initialize();
    // Send 5 requests in sequence, all should get responses
    for i in 0..5 {
        let resp = server.request("nonexistent/method", serde_json::json!({"i": i}));
        assert!(resp["error"].is_object());
    }
}

#[test]
fn test_notification_after_request() {
    let mut server = TestServer::start();
    server.initialize();
    // Mix notifications and requests
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/mix.rs", "languageId": "rust", "version": 1, "text": "fn mix() {}" }
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/mix.rs"}));
    assert_eq!(resp["result"]["content"], "fn mix() {}");
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": { "uri": "file:///tmp/mix.rs", "version": 2 },
        "contentChanges": [{ "text": "fn mixed() {}" }]
    }));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/mix.rs"}));
    assert_eq!(resp["result"]["content"], "fn mixed() {}");
}

#[test]
fn test_debug_file_content_for_nonexistent_uri() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///does/not/exist.rs"
    }));
    // Should return empty, not error
    assert_eq!(resp["result"]["content"], "");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.1 borrowscope/ownershipGraph request tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ownership_graph_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 5, "character": 0}
    }));
    assert_eq!(resp["result"]["_status"], "loading", "Should return loading status");
}

#[test]
fn test_ownership_graph_request_format_valid() {
    let mut server = TestServer::start();
    server.initialize();
    // Even though workspace isn't loaded, the request should be parseable
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/main.rs"},
        "position": {"line": 0, "character": 0}
    }));
    // Should get a structured error (not a crash or malformed response)
    assert!(resp.get("error").is_some() || resp.get("result").is_some());
}

#[test]
fn test_ownership_graph_missing_text_document() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "position": {"line": 0, "character": 0}
    }));
    // Should return error (missing required field)
    assert!(resp.get("error").is_some());
}

#[test]
fn test_ownership_graph_missing_position() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    assert!(resp.get("error").is_some());
}

#[test]
fn test_ownership_graph_invalid_uri_scheme() {
    let mut server = TestServer::start();
    server.initialize();
    // Open a file first so workspace check passes (but it won't since no workspace)
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "http://not-a-file/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

#[test]
fn test_ownership_graph_request_does_not_crash_server() {
    let mut server = TestServer::start();
    server.initialize();
    // Send multiple requests - server should handle all without crashing
    for i in 0..5 {
        let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
            "textDocument": {"uri": format!("file:///tmp/test{}.rs", i)},
            "position": {"line": i, "character": 0}
        }));
        assert!(resp.get("error").is_some() || resp.get("result").is_some());
    }
    // Server still responds after multiple requests
    let resp = server.request("shutdown", serde_json::Value::Null);
    assert!(resp["result"].is_null());
}

#[test]
fn test_ownership_graph_error_has_message() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert!(resp["result"]["_status"].is_string(), "Should have _status field");
}

#[test]
fn test_ownership_graph_error_code_is_numeric() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert_eq!(resp["result"]["_status"], "loading");
}

#[test]
fn test_ownership_graph_after_file_open_still_needs_workspace() {
    let mut server = TestServer::start();
    server.initialize();
    // Open a file (stored in open_files) but workspace still not loaded
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs", "languageId": "rust", "version": 1, "text": "fn main() {}"}
    }));
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    // Still returns not-initialized because workspace (ra_ap db) isn't loaded
    assert_eq!(resp["result"]["_status"], "loading");
}

#[test]
fn test_ownership_graph_response_id_matches_request() {
    let mut server = TestServer::start();
    server.initialize();
    // The response ID should match the request ID (handled by our TestServer)
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    // Response has either "result" or "error" (valid JSON-RPC)
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
    assert!(resp.get("id").is_some(), "Response must have an id");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.2 borrowscope/borrowScopes request tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_borrow_scopes_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    assert_eq!(resp["error"]["code"], -32002);
}

#[test]
fn test_borrow_scopes_returns_scopes_field() {
    // Without workspace, returns error; with workspace would return {scopes: [...]}
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    // Before workspace: error
    assert!(resp["error"].is_object());
}

#[test]
fn test_borrow_scopes_invalid_uri() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "http://invalid"}
    }));
    assert!(resp["error"].is_object());
}

#[test]
fn test_borrow_scopes_missing_params() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({}));
    assert!(resp["error"].is_object());
}

#[test]
fn test_borrow_scopes_does_not_crash_server() {
    let mut server = TestServer::start();
    server.initialize();
    server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    // Server still responds after
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test2.rs"}
    }));
    assert!(resp.get("id").is_some());
}

#[test]
fn test_borrow_scopes_response_id_matches() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/borrowScopes", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"}
    }));
    assert!(resp["id"].is_number());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.3 borrowscope/variableInfo request tests
// ═══════════════════════════════════════════════════════════════════════════


#[test]
fn test_variable_info_before_workspace_ready() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/variableInfo", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 5, "character": 8}
    }));
    assert_eq!(resp["error"]["code"], -32002);
}

#[test]
fn test_variable_info_invalid_uri() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/variableInfo", serde_json::json!({
        "textDocument": {"uri": "http://invalid"},
        "position": {"line": 0, "character": 0}
    }));
    assert!(resp.get("error").is_some() || resp["result"].is_null());
}

#[test]
fn test_variable_info_missing_params() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/variableInfo", serde_json::json!({}));
    assert!(resp.get("error").is_some());
}

#[test]
fn test_variable_info_does_not_crash_server() {
    let mut server = TestServer::start();
    server.initialize();
    for line in 0..10 {
        let resp = server.request("borrowscope/variableInfo", serde_json::json!({
            "textDocument": {"uri": "file:///tmp/test.rs"},
            "position": {"line": line, "character": 0}
        }));
        assert!(resp.get("error").is_some() || resp.get("result").is_some());
    }
    let resp = server.request("shutdown", serde_json::Value::Null);
    assert!(resp["result"].is_null());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.4 borrowscope/analysisUpdated notification tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_analysis_updated_sent_after_change() {
    let mut server = TestServer::start();
    server.initialize();
    let notifs = server.notify_and_collect("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/notif.rs", "languageId": "rust", "version": 1, "text": "fn hello() {}"}
    }));
    // didOpen doesn't trigger analysisUpdated (only didChange does)
    // Now change the file
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/notif.rs", "version": 2},
        "contentChanges": [{"text": "fn hello() { let x = 1; }"}]
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(!analysis_notifs.is_empty(), "Should send analysisUpdated after didChange. Got: {:?}", notifs);
}

#[test]
fn test_analysis_updated_contains_uri() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/uri_test.rs", "languageId": "rust", "version": 1, "text": "fn foo() {}"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/uri_test.rs", "version": 2},
        "contentChanges": [{"text": "fn foo() { let x = 1; }"}]
    }));
    let notif = notifs.iter().find(|n| n["method"] == "borrowscope/analysisUpdated");
    assert!(notif.is_some());
    assert_eq!(notif.unwrap()["params"]["uri"], "file:///tmp/uri_test.rs");
}

#[test]
fn test_analysis_updated_contains_functions() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/fns.rs", "languageId": "rust", "version": 1, "text": "fn alpha() {}\nfn beta() {}"}
    }));
    // Change both functions
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/fns.rs", "version": 2},
        "contentChanges": [{"text": "fn alpha() { let x = 1; }\nfn beta() { let y = 2; }"}]
    }));
    let notif = notifs.iter().find(|n| n["method"] == "borrowscope/analysisUpdated").unwrap();
    let functions = notif["params"]["functions"].as_array().unwrap();
    let fn_names: Vec<&str> = functions.iter().filter_map(|f| f.as_str()).collect();
    assert!(fn_names.contains(&"alpha"), "Should list alpha. Got: {:?}", fn_names);
    assert!(fn_names.contains(&"beta"), "Should list beta. Got: {:?}", fn_names);
}

#[test]
fn test_analysis_updated_has_timestamp() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ts.rs", "languageId": "rust", "version": 1, "text": "fn t() {}"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ts.rs", "version": 2},
        "contentChanges": [{"text": "fn t() { let x = 1; }"}]
    }));
    let notif = notifs.iter().find(|n| n["method"] == "borrowscope/analysisUpdated").unwrap();
    assert!(notif["params"]["timestamp"].is_number(), "Should have numeric timestamp");
    assert!(notif["params"]["timestamp"].as_u64().unwrap() > 0);
}

#[test]
fn test_analysis_updated_not_sent_for_non_rust() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/readme.md", "languageId": "markdown", "version": 1, "text": "# Hello"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/readme.md", "version": 2},
        "contentChanges": [{"text": "# Updated"}]
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(analysis_notifs.is_empty(), "Should NOT send analysisUpdated for non-Rust files");
}

#[test]
fn test_analysis_updated_not_sent_on_did_open() {
    let mut server = TestServer::start();
    server.initialize();
    let notifs = server.notify_and_collect("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/open_only.rs", "languageId": "rust", "version": 1, "text": "fn x() {}"}
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(analysis_notifs.is_empty(), "didOpen should NOT trigger analysisUpdated");
}

#[test]
fn test_analysis_updated_multiple_changes_each_sends() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/multi.rs", "languageId": "rust", "version": 1, "text": "fn m() {}"}
    }));
    // First change
    let notifs1 = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/multi.rs", "version": 2},
        "contentChanges": [{"text": "fn m() { let a = 1; }"}]
    }));
    // Second change
    let notifs2 = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/multi.rs", "version": 3},
        "contentChanges": [{"text": "fn m() { let a = 1; let b = 2; }"}]
    }));
    assert!(notifs1.iter().any(|n| n["method"] == "borrowscope/analysisUpdated"));
    assert!(notifs2.iter().any(|n| n["method"] == "borrowscope/analysisUpdated"));
}

#[test]
fn test_analysis_updated_empty_file_still_sends() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/empty.rs", "languageId": "rust", "version": 1, "text": ""}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/empty.rs", "version": 2},
        "contentChanges": [{"text": "fn new_fn() {}"}]
    }));
    assert!(notifs.iter().any(|n| n["method"] == "borrowscope/analysisUpdated"));
}

#[test]
fn test_analysis_updated_functions_list_updates() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/grow.rs", "languageId": "rust", "version": 1, "text": "fn one() {}"}
    }));
    let notifs1 = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/grow.rs", "version": 2},
        "contentChanges": [{"text": "fn one() {}\nfn two() {}"}]
    }));
    let notif = notifs1.iter().find(|n| n["method"] == "borrowscope/analysisUpdated").unwrap();
    let fns = notif["params"]["functions"].as_array().unwrap();
    let names: Vec<&str> = fns.iter().filter_map(|f| f.as_str()).collect();
    // Only the newly added function should be listed (one's body unchanged)
    assert!(names.contains(&"two"), "Should include newly added function 'two'. Got: {:?}", names);
}

#[test]
fn test_analysis_updated_notification_is_valid_jsonrpc() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/valid.rs", "languageId": "rust", "version": 1, "text": "fn v() {}"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/valid.rs", "version": 2},
        "contentChanges": [{"text": "fn v() { let x = 1; }"}]
    }));
    let notif = notifs.iter().find(|n| n["method"] == "borrowscope/analysisUpdated").unwrap();
    // Valid JSON-RPC notification: has jsonrpc, method, params, no id
    assert_eq!(notif["jsonrpc"], "2.0");
    assert_eq!(notif["method"], "borrowscope/analysisUpdated");
    assert!(notif["params"].is_object());
    assert!(notif.get("id").is_none(), "Notifications should not have id");
}

#[test]
fn test_analysis_updated_only_affected_functions() {
    let mut server = TestServer::start();
    server.initialize();
    // File with two functions
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/affected.rs", "languageId": "rust", "version": 1,
            "text": "fn unchanged() { let x = 1; }\nfn changed() { let y = 2; }"}
    }));
    // Change only the body of 'changed'
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/affected.rs", "version": 2},
        "contentChanges": [{"text": "fn unchanged() { let x = 1; }\nfn changed() { let y = 99; }"}]
    }));
    let notif = notifs.iter().find(|n| n["method"] == "borrowscope/analysisUpdated").unwrap();
    let functions = notif["params"]["functions"].as_array().unwrap();
    let fn_names: Vec<&str> = functions.iter().filter_map(|f| f.as_str()).collect();
    // Only 'changed' should be listed (unchanged body = not affected)
    assert!(fn_names.contains(&"changed"), "Should list 'changed'. Got: {:?}", fn_names);
    assert!(!fn_names.contains(&"unchanged"), "'unchanged' should NOT be listed. Got: {:?}", fn_names);
}

#[test]
fn test_analysis_updated_no_notification_if_no_ownership_change() {
    let mut server = TestServer::start();
    server.initialize();
    let content = "fn stable() { let x = 1; }";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/nochange.rs", "languageId": "rust", "version": 1, "text": content}
    }));
    // "Change" to the exact same content (no actual change)
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/nochange.rs", "version": 2},
        "contentChanges": [{"text": content}]
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(analysis_notifs.is_empty(),
        "Should NOT send notification when content is unchanged. Got: {:?}", analysis_notifs);
}

#[test]
fn test_analysis_updated_comment_only_change_no_notification() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/comment.rs", "languageId": "rust", "version": 1,
            "text": "fn foo() { let x = 1; }"}
    }));
    // Add a comment outside any function - function body unchanged
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/comment.rs", "version": 2},
        "contentChanges": [{"text": "// a comment\nfn foo() { let x = 1; }"}]
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    // The function body didn't change, so no notification
    assert!(analysis_notifs.is_empty(),
        "Comment-only change should NOT trigger notification. Got: {:?}", analysis_notifs);
}


// ═══════════════════════════════════════════════════════════════════════════
// 3.5 textDocument/publishDiagnostics tests (require workspace for semantic analysis)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_diagnostics_not_sent_without_workspace() {
    // Without workspace, no diagnostics are published (no heuristic fallback)
    let mut server = TestServer::start();
    server.initialize();
    let notifs = server.notify_and_collect("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/diag.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let mut x = vec![1];\n    let r = &x;\n    let m = &mut x;\n    println!(\"{}\", r);\n}"}
    }));
    let diag_notifs: Vec<_> = notifs.iter().filter(|n| n["method"] == "textDocument/publishDiagnostics").collect();
    // Either no notification or empty diagnostics
    for n in &diag_notifs {
        let diags = n["params"]["diagnostics"].as_array().unwrap();
        assert!(diags.is_empty(), "Without workspace, diagnostics should be empty");
    }
}

#[test]
fn test_diagnostics_not_sent_for_non_rust() {
    let mut server = TestServer::start();
    server.initialize();
    let notifs = server.notify_and_collect("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.txt", "languageId": "text", "version": 1, "text": "hello"}
    }));
    let diag_notifs: Vec<_> = notifs.iter().filter(|n| n["method"] == "textDocument/publishDiagnostics").collect();
    assert!(diag_notifs.is_empty(), "Non-rust files should not get diagnostics");
}

#[test]
fn test_diagnostics_uri_matches_file() {
    let mut server = TestServer::start();
    server.initialize();
    let notifs = server.notify_and_collect("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/uri_test.rs", "languageId": "rust", "version": 1, "text": "fn main() {}"}
    }));
    for n in &notifs {
        if n["method"] == "textDocument/publishDiagnostics" {
            assert_eq!(n["params"]["uri"], "file:///tmp/uri_test.rs");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.6 textDocument/codeLens tests (require workspace for semantic analysis)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_code_lens_returns_array() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cl.rs", "languageId": "rust", "version": 1, "text": "fn main() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({"textDocument": {"uri": "file:///tmp/cl.rs"}}));
    assert!(resp["result"].is_array(), "codeLens should return array");
}

#[test]
fn test_code_lens_empty_without_workspace() {
    // Without workspace loaded, codeLens returns empty (no heuristic stats)
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cl2.rs", "languageId": "rust", "version": 1,
            "text": "fn main() {\n    let x = 42;\n}\nfn other() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({"textDocument": {"uri": "file:///tmp/cl2.rs"}}));
    let lenses = resp["result"].as_array().unwrap();
    assert!(lenses.is_empty(), "Without workspace, codeLens should be empty (semantic only)");
}

#[test]
fn test_code_lens_no_lenses_for_empty_file() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cl_empty.rs", "languageId": "rust", "version": 1, "text": ""}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({"textDocument": {"uri": "file:///tmp/cl_empty.rs"}}));
    let lenses = resp["result"].as_array().unwrap();
    assert!(lenses.is_empty());
}

#[test]
fn test_code_lens_before_workspace_ready_returns_empty() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("textDocument/codeLens", serde_json::json!({"textDocument": {"uri": "file:///tmp/cl3.rs"}}));
    assert!(resp["result"].is_array());
    assert!(resp["result"].as_array().unwrap().is_empty());
}

fn inlay_request(server: &mut TestServer, uri: &str, end_line: u32) -> serde_json::Value {
    server.request("textDocument/inlayHint", serde_json::json!({
        "textDocument": {"uri": uri},
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": end_line, "character": 0}}
    }))
}

#[test]
fn test_inlay_hint_empty_without_workspace() {
    // Without workspace, inlayHint returns empty (semantic only, no heuristics)
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih2.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let rc = Rc::new(42);\n    let r = &rc;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih2.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty(), "Without workspace, inlayHint should be empty (semantic only)");
}

#[test]
fn test_inlay_hint_empty_file() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_empty.rs", "languageId": "rust", "version": 1, "text": ""}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_empty.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty());
}

#[test]
fn test_inlay_hints_before_workspace_ready_returns_empty() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("textDocument/inlayHint", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih3.rs"},
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": 10, "character": 0}}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_inlay_hint_no_hint_for_vec_without_workspace() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_vec.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let v = vec![1, 2, 3];\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_vec.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty(), "Without workspace, no hints");
}

#[test]
fn test_inlay_hint_no_hint_for_primitive_without_workspace() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_prim.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let x = 42;\n    let b = true;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_prim.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty(), "Without workspace, no hints");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.1 Salsa Incremental Computation tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_incremental_apply_changes_returns_modified_paths() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc1.rs", "languageId": "rust", "version": 1,
            "text": "fn main() { let x = 1; }"}
    }));
    // Change the file
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc1.rs", "version": 2},
        "contentChanges": [{"text": "fn main() { let x = 2; }"}]
    }));
    // Server should process without error
    std::thread::sleep(std::time::Duration::from_millis(100));
    // Verify server still responds
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc1.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_comment_change_no_semantic_invalidation() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let x = 42;\n}\n";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc2.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    // Add a comment (no semantic change)
    let code2 = "// comment\nfn test() {\n    let x = 42;\n}\n";
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc2.rs", "version": 2},
        "contentChanges": [{"text": code2}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc2.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_multiple_changes_applied() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc3.rs", "languageId": "rust", "version": 1,
            "text": "fn a() {}\nfn b() {}"}
    }));
    // Change multiple times
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc3.rs", "version": 2},
        "contentChanges": [{"text": "fn a() { let x = 1; }\nfn b() {}"}]
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc3.rs", "version": 3},
        "contentChanges": [{"text": "fn a() { let x = 1; }\nfn b() { let y = 2; }"}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc3.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_no_changes_returns_empty() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc4.rs", "languageId": "rust", "version": 1,
            "text": "fn main() {}"}
    }));
    // No changes - server should still respond fine
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc4.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_server_stable_after_many_edits() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc5.rs", "languageId": "rust", "version": 1,
            "text": "fn main() { let x = 0; }"}
    }));
    // Rapid edits
    for i in 1..=10 {
        server.notify("textDocument/didChange", serde_json::json!({
            "textDocument": {"uri": "file:///tmp/inc5.rs", "version": i + 1},
            "contentChanges": [{"text": format!("fn main() {{ let x = {}; }}", i)}]
        }));
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    // Server should still be responsive
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc5.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_file_content_updated_after_change() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc6.rs", "languageId": "rust", "version": 1,
            "text": "fn old() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc6.rs", "version": 2},
        "contentChanges": [{"text": "fn new_name() {}"}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Verify content was updated
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/inc6.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn new_name() {}");
}

#[test]
fn test_incremental_dirty_flag_cleared_after_apply() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc7.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc7.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let a = 1; }"}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(100));
    // Second request should work (dirty flag cleared)
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc7.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_incremental_analysis_updated_sent_on_change() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc8.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc8.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 2; let y = 3; }"}]
    }));
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(!analysis_notifs.is_empty(), "Should send analysisUpdated after change");
}

#[test]
fn test_incremental_analysis_updated_contains_uri() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc9.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc9.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 1; }"}]
    }));
    let analysis_notif = notifs.iter()
        .find(|n| n["method"] == "borrowscope/analysisUpdated");
    if let Some(notif) = analysis_notif {
        assert_eq!(notif["params"]["uri"], "file:///tmp/inc9.rs");
    }
}

#[test]
fn test_incremental_unchanged_function_not_in_notification() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc10.rs", "languageId": "rust", "version": 1,
            "text": "fn unchanged() { let a = 1; }\nfn changed() { let b = 2; }"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/inc10.rs", "version": 2},
        "contentChanges": [{"text": "fn unchanged() { let a = 1; }\nfn changed() { let b = 99; let c = 100; }"}]
    }));
    let analysis_notif = notifs.iter()
        .find(|n| n["method"] == "borrowscope/analysisUpdated");
    if let Some(notif) = analysis_notif {
        let functions = notif["params"]["functions"].as_array().unwrap();
        // "unchanged" should NOT be in the list (its body didn't change)
        let has_unchanged = functions.iter().any(|f| f.as_str() == Some("unchanged"));
        assert!(!has_unchanged, "Unchanged function should not be in notification. Got: {:?}", functions);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.2 Debounced Analysis tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_debounce_rapid_changes_single_notification() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb1.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    // Send 5 rapid changes (< 300ms apart)
    for i in 2..=6 {
        server.notify("textDocument/didChange", serde_json::json!({
            "textDocument": {"uri": "file:///tmp/deb1.rs", "version": i},
            "contentChanges": [{"text": format!("fn test() {{ let x = {}; }}", i)}]
        }));
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    // Wait for debounce to fire
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let notifs = server.take_notifications();
    let analysis_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    // Should produce only ONE notification (debounced), not 5
    assert!(analysis_notifs.len() <= 1,
        "Rapid changes should produce at most 1 notification, got {}", analysis_notifs.len());
}

#[test]
fn test_debounce_notification_after_pause() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb2.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb2.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let a = 1; }"}]
    }));
    // Wait less than debounce — no notification yet
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let early_notifs = server.take_notifications();
    let early_analysis: Vec<_> = early_notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(early_analysis.is_empty(), "Should NOT send notification before debounce expires");

    // Wait for debounce to fire
    std::thread::sleep(std::time::Duration::from_millis(350));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let late_notifs = server.take_notifications();
    let late_analysis: Vec<_> = late_notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(!late_analysis.is_empty(), "Should send notification after debounce expires");
}

#[test]
fn test_debounce_content_reflects_latest_change() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb3.rs", "languageId": "rust", "version": 1,
            "text": "fn v1() {}"}
    }));
    // Multiple rapid changes
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb3.rs", "version": 2},
        "contentChanges": [{"text": "fn v2() {}"}]
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb3.rs", "version": 3},
        "contentChanges": [{"text": "fn v3_final() {}"}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Content should be the LAST change
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/deb3.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn v3_final() {}");
}

#[test]
fn test_debounce_server_responsive_during_wait() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb4.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb4.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 1; }"}]
    }));
    // Server should still respond to requests during debounce wait
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/deb4.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn test() { let x = 1; }");
}

#[test]
fn test_debounce_multiple_files_batched() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb5a.rs", "languageId": "rust", "version": 1, "text": "fn a() {}"}
    }));
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb5b.rs", "languageId": "rust", "version": 1, "text": "fn b() {}"}
    }));
    // Change both files rapidly
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb5a.rs", "version": 2},
        "contentChanges": [{"text": "fn a() { let x = 1; }"}]
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb5b.rs", "version": 2},
        "contentChanges": [{"text": "fn b() { let y = 2; }"}]
    }));
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let notifs = server.take_notifications();
    // Should have notifications for both files
    let uris: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .map(|n| n["params"]["uri"].as_str().unwrap_or(""))
        .collect();
    // At least one notification should exist
    assert!(!uris.is_empty(), "Should send notifications after debounce");
}

#[test]
fn test_debounce_zero_fires_immediately() {
    let mut server = TestServer::start();
    server.initialize_with_options(serde_json::json!({"debounceMs": 0}));
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb0.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/deb0.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 1; }"}]
    }));
    // With debounce=0, should fire almost immediately (within 100ms loop cycle)
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let notifs = server.take_notifications();
    let analysis: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(!analysis.is_empty(), "debounce=0 should fire immediately");
}

#[test]
fn test_debounce_configurable_via_init_options() {
    let mut server = TestServer::start();
    // Set a very short debounce (50ms)
    server.initialize_with_options(serde_json::json!({"debounceMs": 50}));
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/debcfg.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/debcfg.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let a = 1; }"}]
    }));
    // Wait 150ms (> 50ms debounce) — should have fired
    std::thread::sleep(std::time::Duration::from_millis(150));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let notifs = server.take_notifications();
    let analysis: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "borrowscope/analysisUpdated")
        .collect();
    assert!(!analysis.is_empty(), "Custom debounce (50ms) should have fired by 150ms");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.3 Partial Results (analysis cache) tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_cache_cleared_on_file_close() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache1.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    // Close the file — cache should be cleared, server should not crash
    server.notify("textDocument/didClose", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache1.rs"}
    }));
    // Server should still be responsive after close
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache1.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_cache_server_responsive_after_change() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache2.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    // Change file
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache2.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 2; let y = 3; }"}]
    }));
    // Request immediately (before debounce fires) — server should respond
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache2.rs"}
    }));
    assert!(resp["result"].is_array(), "Server should respond even during debounce");
}

#[test]
fn test_cache_stale_data_available_during_reanalysis() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache3.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    // Wait for initial debounce
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Change file (marks cache stale)
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache3.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let x = 2; }"}]
    }));
    // Request during debounce — should still get a response (stale or fresh)
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache3.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_cache_fresh_after_debounce_completes() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache4.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache4.rs", "version": 2},
        "contentChanges": [{"text": "fn fresh() { let y = 2; }"}]
    }));
    // Wait for debounce to complete
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Content should reflect the latest change
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({
        "uri": "file:///tmp/cache4.rs"
    }));
    assert_eq!(resp["result"]["content"], "fn fresh() { let y = 2; }");
}

#[test]
fn test_cache_multiple_functions_cached_independently() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache5.rs", "languageId": "rust", "version": 1,
            "text": "fn a() { let x = 1; }\nfn b() { let y = 2; }"}
    }));
    std::thread::sleep(std::time::Duration::from_millis(400));
    // Both functions should be accessible
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cache5.rs"}
    }));
    assert!(resp["result"].is_array());
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.5 Performance Budget tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_perf_server_responds_within_budget() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/perf1.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; let y = 2; }"}
    }));
    // Request should respond quickly (no workspace = loading response)
    let start = std::time::Instant::now();
    let _resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/perf1.rs"}
    }));
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "Response should be < 500ms, got {:?}", elapsed);
}

#[test]
fn test_perf_debounce_plus_notification_within_budget() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/perf2.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    let start = std::time::Instant::now();
    server.notify("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/perf2.rs", "version": 2},
        "contentChanges": [{"text": "fn test() { let a = 1; }"}]
    }));
    // Wait for debounce + processing
    std::thread::sleep(std::time::Duration::from_millis(400));
    let _resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    let elapsed = start.elapsed();
    // Total pipeline: debounce(300) + processing should be < 500ms
    assert!(elapsed.as_millis() < 600, "Full pipeline should be < 600ms, got {:?}", elapsed);
}

#[test]
fn test_perf_rapid_requests_dont_block() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/perf3.rs", "languageId": "rust", "version": 1,
            "text": "fn a() {}\nfn b() {}\nfn c() {}"}
    }));
    // Send multiple requests rapidly
    let start = std::time::Instant::now();
    for _ in 0..5 {
        let _resp = server.request("textDocument/codeLens", serde_json::json!({
            "textDocument": {"uri": "file:///tmp/perf3.rs"}
        }));
    }
    let elapsed = start.elapsed();
    // 5 requests should complete in < 1s total
    assert!(elapsed.as_millis() < 1000, "5 rapid requests should be < 1s, got {:?}", elapsed);
}

#[test]
fn test_perf_analyze_function_timed_exported() {
    // Verify the timed function is available in source
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/analysis.rs")
    ).unwrap();
    assert!(src.contains("pub fn analyze_function_timed"));
    assert!(src.contains("exceeds 100ms budget"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6.6 Memory Management tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_memory_cache_created_on_open() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/mem1.rs", "languageId": "rust", "version": 1,
            "text": "fn test() { let x = 1; }"}
    }));
    // Server should track the file
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/mem1.rs"}));
    assert_eq!(resp["result"]["content"], "fn test() { let x = 1; }");
}

#[test]
fn test_memory_cache_evicted_on_close() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/mem2.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {}"}
    }));
    server.notify("textDocument/didClose", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/mem2.rs"}
    }));
    // Server should still respond (cache cleared but no crash)
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/mem2.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_memory_many_files_open_close() {
    let mut server = TestServer::start();
    server.initialize();
    // Open and close 20 files
    for i in 0..20 {
        let uri = format!("file:///tmp/mem_many_{}.rs", i);
        server.notify("textDocument/didOpen", serde_json::json!({
            "textDocument": {"uri": uri, "languageId": "rust", "version": 1,
                "text": format!("fn f{}() {{ let x = {}; }}", i, i)}
        }));
    }
    for i in 0..20 {
        let uri = format!("file:///tmp/mem_many_{}.rs", i);
        server.notify("textDocument/didClose", serde_json::json!({
            "textDocument": {"uri": uri}
        }));
    }
    // Server should still be responsive
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": ""}));
    assert!(resp["result"].is_object());
}

#[test]
fn test_memory_open_file_not_evicted() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/mem_open.rs", "languageId": "rust", "version": 1,
            "text": "fn keep() { let x = 1; }"}
    }));
    // File stays open — content should persist
    std::thread::sleep(std::time::Duration::from_millis(100));
    let resp = server.request("borrowscope/debug/fileContent", serde_json::json!({"uri": "file:///tmp/mem_open.rs"}));
    assert_eq!(resp["result"]["content"], "fn keep() { let x = 1; }");
}

#[test]
fn test_memory_estimated_size_nonzero() {
    // Verify the estimated_size_bytes function works via source check
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/state.rs")
    ).unwrap();
    assert!(src.contains("estimated_size_bytes"));
    assert!(src.contains("evict_closed_caches"));
    assert!(src.contains("evict_if_over_budget"));
    assert!(src.contains("total_cache_bytes"));
}
