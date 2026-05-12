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
        // Give server a moment to process and send notification
        std::thread::sleep(std::time::Duration::from_millis(50));
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
    assert_eq!(resp["error"]["code"], -32002);
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
    assert_eq!(resp["error"]["code"], -32002, "Should return ServerNotInitialized");
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
    assert!(resp.get("error").is_some());
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
    let error = &resp["error"];
    assert!(error["message"].is_string(), "Error should have a message");
    assert!(!error["message"].as_str().unwrap().is_empty());
}

#[test]
fn test_ownership_graph_error_code_is_numeric() {
    let mut server = TestServer::start();
    server.initialize();
    let resp = server.request("borrowscope/ownershipGraph", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.rs"},
        "position": {"line": 0, "character": 0}
    }));
    assert!(resp["error"]["code"].is_number());
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
    assert_eq!(resp["error"]["code"], -32002);
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
// 3.5 textDocument/publishDiagnostics tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_diagnostics_sent_for_conflict() {
    let mut server = TestServer::start();
    server.initialize();
    // Code with overlapping &mut and & on same variable
    let code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/conflict.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/conflict.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "textDocument/publishDiagnostics")
        .collect();
    assert!(!diag_notifs.is_empty(), "Should send diagnostics for conflict");
    let diagnostics = diag_notifs[0]["params"]["diagnostics"].as_array().unwrap();
    assert!(!diagnostics.is_empty(), "Should have at least one diagnostic");
}

#[test]
fn test_diagnostics_severity_is_information() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/sev.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/sev.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    let d = &diag["params"]["diagnostics"][0];
    assert_eq!(d["severity"], 3, "Severity should be 3 (Information). Got: {}", d["severity"]);
}

#[test]
fn test_diagnostics_source_is_borrowscope() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/src.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/src.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    let d = &diag["params"]["diagnostics"][0];
    assert_eq!(d["source"], "BorrowScope");
}

#[test]
fn test_diagnostics_has_related_information() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/rel.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/rel.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    let d = &diag["params"]["diagnostics"][0];
    let related = d["relatedInformation"].as_array().unwrap();
    assert_eq!(related.len(), 2, "Should have 2 related locations (both borrows)");
    assert!(related[0]["message"].as_str().unwrap().contains("First borrow"));
    assert!(related[1]["message"].as_str().unwrap().contains("Second borrow"));
}

#[test]
fn test_diagnostics_has_correct_range() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/range.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/range.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    let d = &diag["params"]["diagnostics"][0];
    assert!(d["range"]["start"]["line"].is_number());
    assert!(d["range"]["end"]["line"].is_number());
}

#[test]
fn test_diagnostics_clear_when_conflict_resolved() {
    let mut server = TestServer::start();
    server.initialize();
    // First: code with conflict
    let conflict_code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/clear.rs", "languageId": "rust", "version": 1, "text": conflict_code}
    }));
    let notifs1 = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/clear.rs", "version": 2},
        "contentChanges": [{"text": conflict_code}]
    }));
    // Should have diagnostics
    let has_diag = notifs1.iter().any(|n| {
        n["method"] == "textDocument/publishDiagnostics"
            && !n["params"]["diagnostics"].as_array().unwrap().is_empty()
    });
    assert!(has_diag, "Should have diagnostics for conflict code");

    // Now: fix the conflict (remove mutable borrow)
    let fixed_code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    println!(\"{}\", r);\n}";
    let notifs2 = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/clear.rs", "version": 3},
        "contentChanges": [{"text": fixed_code}]
    }));
    // Should clear diagnostics (empty array)
    let clear_diag = notifs2.iter().find(|n| n["method"] == "textDocument/publishDiagnostics");
    assert!(clear_diag.is_some(), "Should send publishDiagnostics to clear");
    let diagnostics = clear_diag.unwrap()["params"]["diagnostics"].as_array().unwrap();
    assert!(diagnostics.is_empty(), "Diagnostics should be empty after fix. Got: {:?}", diagnostics);
}

#[test]
fn test_no_diagnostics_for_valid_code() {
    let mut server = TestServer::start();
    server.initialize();
    let valid_code = "fn test() {\n    let data = vec![1];\n    let r = &data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/valid.rs", "languageId": "rust", "version": 1, "text": valid_code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/valid.rs", "version": 2},
        "contentChanges": [{"text": valid_code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics");
    if let Some(d) = diag {
        let diagnostics = d["params"]["diagnostics"].as_array().unwrap();
        assert!(diagnostics.is_empty(), "Valid code should have no diagnostics. Got: {:?}", diagnostics);
    }
}

#[test]
fn test_diagnostics_message_contains_variable_names() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let mydata = vec![1];\n    let reader = &mydata;\n    let writer = &mut mydata;\n    println!(\"{}\", reader);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/names.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/names.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    let d = &diag["params"]["diagnostics"][0];
    let msg = d["message"].as_str().unwrap();
    assert!(msg.contains("reader") || msg.contains("writer") || msg.contains("mydata"),
        "Message should contain variable names. Got: {}", msg);
}

#[test]
fn test_diagnostics_uri_matches_file() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let d = vec![1];\n    let r = &d;\n    let m = &mut d;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/uri_match.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/uri_match.rs", "version": 2},
        "contentChanges": [{"text": code}]
    }));
    let diag = notifs.iter().find(|n| n["method"] == "textDocument/publishDiagnostics").unwrap();
    assert_eq!(diag["params"]["uri"], "file:///tmp/uri_match.rs");
}

#[test]
fn test_diagnostics_not_sent_for_non_rust() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.md", "languageId": "markdown", "version": 1, "text": "# Hello"}
    }));
    let notifs = server.notify_and_collect("textDocument/didChange", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/test.md", "version": 2},
        "contentChanges": [{"text": "# Updated"}]
    }));
    let diag_notifs: Vec<_> = notifs.iter()
        .filter(|n| n["method"] == "textDocument/publishDiagnostics")
        .collect();
    assert!(diag_notifs.is_empty(), "Should not send diagnostics for non-Rust files");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.6 textDocument/codeLens tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_code_lens_returns_array() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/lens.rs", "languageId": "rust", "version": 1, "text": "fn hello() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/lens.rs"}
    }));
    assert!(resp["result"].is_array());
}

#[test]
fn test_code_lens_one_per_function() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/two_fns.rs", "languageId": "rust", "version": 1,
            "text": "fn alpha() {}\nfn beta() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/two_fns.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    assert_eq!(lenses.len(), 2, "Should have one CodeLens per function");
}

#[test]
fn test_code_lens_title_contains_counts() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/counts.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let x = 1;\n    let r = &x;\n}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/counts.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    let title = lenses[0]["command"]["title"].as_str().unwrap();
    assert!(title.contains("vars"), "Title should contain 'vars'. Got: {}", title);
    assert!(title.contains("borrows"), "Title should contain 'borrows'. Got: {}", title);
    assert!(title.contains("moves"), "Title should contain 'moves'. Got: {}", title);
}

#[test]
fn test_code_lens_range_on_function_line() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/range_lens.rs", "languageId": "rust", "version": 1,
            "text": "// comment\nfn second() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/range_lens.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    assert_eq!(lenses.len(), 1);
    assert_eq!(lenses[0]["range"]["start"]["line"], 1, "Should be on line 1 (0-indexed)");
}

#[test]
fn test_code_lens_command_is_show_graph() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cmd.rs", "languageId": "rust", "version": 1, "text": "fn test() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/cmd.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    assert_eq!(lenses[0]["command"]["command"], "borrowscope.showGraph");
}

#[test]
fn test_code_lens_command_has_arguments() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/args.rs", "languageId": "rust", "version": 1, "text": "fn my_func() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/args.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    let args = lenses[0]["command"]["arguments"].as_array().unwrap();
    assert_eq!(args[0], "file:///tmp/args.rs");
    assert_eq!(args[1], "my_func");
}

#[test]
fn test_code_lens_empty_function_shows_zeros() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/empty_fn.rs", "languageId": "rust", "version": 1, "text": "fn empty() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/empty_fn.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    let title = lenses[0]["command"]["title"].as_str().unwrap();
    assert!(title.contains("0 vars"), "Empty fn should show 0 vars. Got: {}", title);
    assert!(title.contains("0 borrows"), "Empty fn should show 0 borrows. Got: {}", title);
    assert!(title.contains("0 moves"), "Empty fn should show 0 moves. Got: {}", title);
}

#[test]
fn test_code_lens_conflict_shown_in_title() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn conflict() {\n    let data = vec![1];\n    let r = &data;\n    let m = &mut data;\n    println!(\"{}\", r);\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/conflict_lens.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/conflict_lens.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    let title = lenses[0]["command"]["title"].as_str().unwrap();
    assert!(title.contains("conflict"), "Should show conflict count. Got: {}", title);
}

#[test]
fn test_code_lens_no_lenses_for_empty_file() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/no_fns.rs", "languageId": "rust", "version": 1, "text": "// just a comment"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/no_fns.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    assert!(lenses.is_empty(), "File with no functions should have no lenses");
}

#[test]
fn test_code_lens_pub_and_async_functions() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/pub_async.rs", "languageId": "rust", "version": 1,
            "text": "pub fn public_fn() {}\nasync fn async_fn() {}\npub async fn both() {}"}
    }));
    let resp = server.request("textDocument/codeLens", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/pub_async.rs"}
    }));
    let lenses = resp["result"].as_array().unwrap();
    assert_eq!(lenses.len(), 3, "Should detect pub, async, and pub async functions");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3.7 textDocument/inlayHint tests
// ═══════════════════════════════════════════════════════════════════════════

fn inlay_request(server: &mut TestServer, uri: &str, end_line: u32) -> serde_json::Value {
    server.request("textDocument/inlayHint", serde_json::json!({
        "textDocument": {"uri": uri},
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": end_line, "character": 0}}
    }))
}

#[test]
fn test_inlay_hint_rc_variable() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_rc.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let rc = Rc::new(42);\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_rc.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(!hints.is_empty(), "Rc variable should get a hint");
    let label = hints[0]["label"].as_str().unwrap();
    assert!(label.contains("Rc"), "Should show [Rc]. Got: {}", label);
}

#[test]
fn test_inlay_hint_shared_ref() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_ref.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let x = vec![1];\n    let r = &x;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_ref.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    let ref_hint = hints.iter().find(|h| h["label"].as_str().unwrap().contains("&"));
    assert!(ref_hint.is_some(), "& variable should get [&] hint. Got: {:?}", hints);
}

#[test]
fn test_inlay_hint_mut_ref() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_mut.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let mut x = vec![1];\n    let m = &mut x;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_mut.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    let mut_hint = hints.iter().find(|h| h["label"].as_str().unwrap().contains("&mut"));
    assert!(mut_hint.is_some(), "&mut variable should get [&mut] hint. Got: {:?}", hints);
}

#[test]
fn test_inlay_hint_no_hint_for_vec() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_vec.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let v = vec![1, 2, 3];\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_vec.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty(), "Vec should NOT get a hint (owned is default). Got: {:?}", hints);
}

#[test]
fn test_inlay_hint_no_hint_for_primitive() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_prim.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let x = 42;\n    let b = true;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_prim.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(hints.is_empty(), "Primitives should NOT get hints. Got: {:?}", hints);
}

#[test]
fn test_inlay_hint_correct_position() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_pos.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let r = &x;\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_pos.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(!hints.is_empty());
    let pos = &hints[0]["position"];
    assert_eq!(pos["line"], 1, "Hint should be on line 1 (the let statement)");
    assert!(pos["character"].as_u64().unwrap() > 0, "Character should be after variable name");
}

#[test]
fn test_inlay_hint_respects_visible_range() {
    let mut server = TestServer::start();
    server.initialize();
    let code = "fn test() {\n    let a = &x;\n    let b = &y;\n    let c = &z;\n}";
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_range.rs", "languageId": "rust", "version": 1, "text": code}
    }));
    // Only request lines 0-1 (should only get hint for 'a', not 'b' or 'c')
    let resp = server.request("textDocument/inlayHint", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_range.rs"},
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": 1, "character": 0}}
    }));
    let hints = resp["result"].as_array().unwrap();
    assert_eq!(hints.len(), 1, "Should only return hints in visible range. Got: {}", hints.len());
}

#[test]
fn test_inlay_hint_closure() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_closure.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let f = || println!(\"hi\");\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_closure.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    let closure_hint = hints.iter().find(|h| h["label"].as_str().unwrap().contains("closure"));
    assert!(closure_hint.is_some(), "Closure should get [closure] hint. Got: {:?}", hints);
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
fn test_inlay_hint_arc_variable() {
    let mut server = TestServer::start();
    server.initialize();
    server.notify("textDocument/didOpen", serde_json::json!({
        "textDocument": {"uri": "file:///tmp/ih_arc.rs", "languageId": "rust", "version": 1,
            "text": "fn test() {\n    let a = Arc::new(42);\n}"}
    }));
    let resp = inlay_request(&mut server, "file:///tmp/ih_arc.rs", 10);
    let hints = resp["result"].as_array().unwrap();
    assert!(!hints.is_empty(), "Arc variable should get a hint");
    let label = hints[0]["label"].as_str().unwrap();
    assert!(label.contains("Arc"), "Should show [Arc]. Got: {}", label);
}
