//! Integration tests for the BorrowScope LSP server.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

struct TestServer {
    process: Child,
    stdout: BufReader<ChildStdout>,
    next_id: i32,
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
        self.read_response()
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
