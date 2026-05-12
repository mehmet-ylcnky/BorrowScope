# Milestone 1: Language Server Scaffold - Detailed Specification

## 1.1 Rust Binary Project Setup (borrowscope-lsp)

**Objective:** Create a new Rust binary crate that will serve as the BorrowScope language server. This binary loads the workspace, maintains the semantic database, and communicates with VS Code via the LSP protocol over stdio.

**Steps:**
1. Create `borrowscope-lsp/` directory with `Cargo.toml`
2. Add to workspace members in root `Cargo.toml`
3. Add dependencies: `ra_ap_*` crates (same versions as borrowscope-analyzer), `lsp-server`, `lsp-types`, `serde`, `serde_json`, `crossbeam-channel`
4. Create `src/main.rs` with basic binary entry point
5. Create module structure: `server.rs`, `analysis.rs`, `capabilities.rs`

**Cargo.toml:**
```toml
[package]
name = "borrowscope-lsp"
version = "0.1.0"
edition = "2021"
description = "BorrowScope Language Server - real-time ownership visualization"

[[bin]]
name = "borrowscope-lsp"
path = "src/main.rs"

[dependencies]
# LSP protocol
lsp-server = "0.7"
lsp-types = "0.97"

# rust-analyzer semantic engine
ra_ap_hir = "0.0.318"
ra_ap_ide = "0.0.318"
ra_ap_load_cargo = "0.0.318"
ra_ap_project_model = "0.0.318"
ra_ap_vfs = "0.0.318"
ra_ap_vfs_notify = "0.0.318"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Concurrency
crossbeam-channel = "0.5"
parking_lot = "0.12"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Module structure:**
```
borrowscope-lsp/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point: parse args, start server
│   ├── server.rs            # LSP server main loop (message dispatch)
│   ├── capabilities.rs      # ServerCapabilities declaration
│   ├── workspace.rs         # Workspace loading (ra_ap_load_cargo)
│   ├── analysis.rs          # Ownership analysis (hir::Type queries)
│   ├── handlers/
│   │   ├── mod.rs           # Handler dispatch
│   │   ├── requests.rs      # Custom request handlers
│   │   └── notifications.rs # File change handlers
│   └── state.rs             # Global server state (database, VFS)
└── tests/
    └── integration.rs       # End-to-end LSP tests
```

**Expectation:** `cargo build -p borrowscope-lsp` produces a binary. The binary starts, prints a log message, and exits cleanly when stdin closes.

**Tests for 1.1:**
- Binary compiles without errors
- Binary starts and exits with code 0 when stdin is immediately closed
- Binary prints version info with `--version` flag
- Module structure resolves (no unresolved imports)

---

## 1.2 LSP Protocol Implementation

**Objective:** Implement the LSP message loop that reads JSON-RPC messages from stdin, dispatches them to handlers, and writes responses to stdout. Use the `lsp-server` crate which provides the transport layer.

**Steps:**
1. Initialize `lsp-server::Connection` from stdio
2. Perform the LSP handshake (initialize request/response)
3. Enter the main loop: read messages from `connection.receiver`
4. Dispatch messages by method name to handler functions
5. Handle shutdown request gracefully

**Code (main.rs):**
```rust
use lsp_server::{Connection, Message};
use lsp_types::InitializeParams;

fn main() -> anyhow::Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("BorrowScope LSP starting...");

    // Create LSP connection over stdio
    let (connection, io_threads) = Connection::stdio();

    // Perform initialize handshake
    let (initialize_id, initialize_params) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(initialize_params)?;

    let capabilities = capabilities::server_capabilities();
    let result = serde_json::to_value(lsp_types::InitializeResult {
        capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: "borrowscope-lsp".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    })?;
    connection.initialize_finish(initialize_id, result)?;

    tracing::info!("Initialized. Loading workspace...");

    // Load workspace (blocking, with progress)
    let state = state::GlobalState::new(&params)?;

    // Enter main loop
    server::main_loop(&connection, state)?;

    // Clean shutdown
    io_threads.join()?;
    tracing::info!("BorrowScope LSP shut down.");
    Ok(())
}
```

**Message dispatch (server.rs):**
```rust
pub fn main_loop(
    connection: &Connection,
    mut state: GlobalState,
) -> anyhow::Result<()> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                handlers::handle_request(&mut state, &connection.sender, req)?;
            }
            Message::Notification(notif) => {
                handlers::handle_notification(&mut state, &connection.sender, notif)?;
            }
            Message::Response(_) => {
                // We don't send requests to the client (yet)
            }
        }
    }
    Ok(())
}
```

**LSP message flow:**
```
VS Code (client)                    borrowscope-lsp (server)
     │                                      │
     │──── initialize request ─────────────▶│
     │                                      │ (load capabilities)
     │◀─── initialize response ────────────│
     │                                      │
     │──── initialized notification ───────▶│
     │                                      │ (start workspace loading)
     │                                      │
     │◀─── window/workDoneProgress ────────│ (loading... 30s)
     │                                      │
     │──── textDocument/didOpen ───────────▶│
     │                                      │ (analyze file)
     │◀─── textDocument/publishDiagnostics ─│
     │                                      │
     │──── shutdown request ───────────────▶│
     │◀─── shutdown response ──────────────│
     │──── exit notification ──────────────▶│
     │                                      │ (process exits)
```

**Expectation:** The server completes the LSP handshake, enters the main loop, and responds to shutdown. No analysis yet, just protocol handling.

**Tests for 1.2:**
- Server responds to `initialize` with valid `InitializeResult`
- Server responds to `shutdown` with null result
- Server exits after receiving `exit` notification
- Invalid JSON-RPC messages don't crash the server
- Server reports its name and version in `serverInfo`

---

## 1.3 Workspace Loading with ra_ap_*

**Objective:** Load the Rust project's workspace using the same mechanism as rust-analyzer. This produces a `RootDatabase` containing the full semantic model of the project, including all dependencies and the standard library.

**Steps:**
1. Extract workspace root from `InitializeParams.root_uri`
2. Configure `CargoConfig` with sysroot discovery enabled
3. Call `load_workspace_at()` to load Cargo.toml, resolve dependencies, index types
4. Store the resulting `(RootDatabase, Vfs)` in `GlobalState`
5. Report progress to the client via `window/workDoneProgress`

**Code (workspace.rs):**
```rust
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::{CargoConfig, RustLibSource};
use ra_ap_vfs::Vfs;
use ra_ap_ide::RootDatabase;

pub struct WorkspaceData {
    pub db: RootDatabase,
    pub vfs: Vfs,
}

pub fn load_workspace(root_path: &Path) -> anyhow::Result<WorkspaceData> {
    let mut cargo_config = CargoConfig::default();
    cargo_config.sysroot = Some(RustLibSource::Discover);

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: true,
    };

    let (db, vfs, _proc_macro_server) =
        load_workspace_at(root_path, &cargo_config, &load_config, &|msg| {
            tracing::debug!("Loading: {}", msg);
        })?;

    Ok(WorkspaceData { db, vfs })
}
```

**Loading timeline:**
```
t=0s    initialize received
t=0.1s  begin workspace loading
        ├── discover Cargo.toml
        ├── resolve dependencies (Cargo.lock)
        ├── locate sysroot (rustc --print sysroot)
        ├── load standard library metadata
        ├── index all source files
        └── prefill type caches
t=30-40s  workspace ready
        └── send workDoneProgress/end to client
```

**Expectation:** After loading, `db` can answer any `hir::Type` query for any variable in the project. The VFS contains all source files and can detect changes.

**Tests for 1.3:**
- Workspace loads successfully for a simple Cargo project (single file)
- Workspace loads for a workspace with multiple crates
- Sysroot is discovered (standard library types resolve, not `{unknown}`)
- Loading a non-existent path returns a clear error
- Progress notifications are sent during loading
- After loading, `sema.type_of_pat()` returns non-unknown types

---

## 1.4 Server Lifecycle (initialize, initialized, shutdown)

**Objective:** Implement the full LSP lifecycle including capability negotiation, workspace loading triggered by `initialized`, and graceful shutdown with resource cleanup.

**Steps:**
1. On `initialize`: return capabilities, store client capabilities
2. On `initialized`: begin workspace loading in background thread
3. On `shutdown`: stop analysis, drop database, return null
4. On `exit`: terminate process

**Code (state.rs):**
```rust
pub struct GlobalState {
    /// Workspace data (None until loading completes)
    pub workspace: Option<WorkspaceData>,
    /// Whether shutdown has been requested
    pub shutdown_requested: bool,
    /// Client capabilities (for feature detection)
    pub client_capabilities: ClientCapabilities,
    /// Workspace root path
    pub root_path: PathBuf,
}

impl GlobalState {
    pub fn new(params: &InitializeParams) -> anyhow::Result<Self> {
        let root_path = params.root_uri
            .as_ref()
            .and_then(|uri| uri.to_file_path().ok())
            .ok_or_else(|| anyhow::anyhow!("No workspace root"))?;

        Ok(Self {
            workspace: None,
            shutdown_requested: false,
            client_capabilities: params.capabilities.clone(),
            root_path,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.workspace.is_some()
    }
}
```

**State machine:**
```
┌──────────┐  initialize   ┌────────────┐  initialized  ┌─────────┐
│  Created  │──────────────▶│ Handshaking │──────────────▶│ Loading │
└──────────┘               └────────────┘               └────┬────┘
                                                              │ workspace ready
                                                              ▼
┌──────────┐  exit         ┌────────────┐  shutdown     ┌─────────┐
│  Exited   │◀──────────────│ Shutting   │◀──────────────│  Ready  │
└──────────┘               │   Down     │               └─────────┘
                           └────────────┘
```

**Expectation:** The server transitions through states correctly. Requests received before workspace is ready return an error (not a crash). Shutdown releases all resources.

**Tests for 1.4:**
- Server transitions from Created to Ready after initialize + initialized
- Requests before workspace ready return `ServerNotInitialized` error
- Shutdown request returns null and sets shutdown flag
- Exit after shutdown terminates process with code 0
- Exit without prior shutdown terminates with code 1
- Double shutdown request doesn't crash

---

## 1.5 Text Document Synchronization

**Objective:** Track open documents and their content changes. When the user edits a file, update the VFS so that subsequent analysis reflects the current source code.

**Steps:**
1. Register for `TextDocumentSyncKind::Incremental` in capabilities
2. Handle `textDocument/didOpen`: add file content to VFS
3. Handle `textDocument/didChange`: apply incremental edits to VFS
4. Handle `textDocument/didSave`: trigger re-analysis
5. Handle `textDocument/didClose`: mark file as closed (keep in VFS)

**Code (handlers/notifications.rs):**
```rust
pub fn handle_did_open(
    state: &mut GlobalState,
    params: DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    let path = uri_to_path(&params.text_document.uri)?;
    if let Some(ws) = &mut state.workspace {
        let vfs_path = ra_ap_vfs::VfsPath::from(path);
        ws.vfs.set_file_contents(vfs_path, Some(params.text_document.text.into_bytes()));
        // Mark file as needing re-analysis
        state.mark_dirty(&path);
    }
    Ok(())
}

pub fn handle_did_change(
    state: &mut GlobalState,
    params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    let path = uri_to_path(&params.text_document.uri)?;
    if let Some(ws) = &mut state.workspace {
        // Apply incremental changes to VFS
        for change in params.content_changes {
            // Full sync for now (incremental later in M6)
            if let Some(text) = change.text.into() {
                let vfs_path = ra_ap_vfs::VfsPath::from(path.clone());
                ws.vfs.set_file_contents(vfs_path, Some(text.into_bytes()));
            }
        }
        state.mark_dirty(&path);
    }
    Ok(())
}
```

**Sync flow:**
```
User types in editor
        │
        ▼
VS Code sends didChange
        │
        ▼
Server updates VFS (in-memory file content)
        │
        ▼
Server marks file as dirty
        │
        ▼
(Analysis triggered on next request or after debounce)
```

**Expectation:** After a `didChange`, the VFS contains the updated file content. Subsequent analysis queries see the new content without reading from disk.

**Tests for 1.5:**
- `didOpen` adds file to VFS with correct content
- `didChange` updates file content in VFS
- `didChange` with multiple edits applies all of them
- `didClose` does not remove file from VFS (other files may reference it)
- Changes to non-Rust files are ignored
- VFS content matches what was sent (no corruption)

---

## 1.6 Incremental Re-analysis via Salsa

**Objective:** When a file changes, only re-analyze the affected functions rather than the entire project. The `ra_ap_*` crates use the Salsa incremental computation framework, which automatically tracks dependencies between queries and invalidates only what's necessary.

**Steps:**
1. After VFS update, call `db.apply_change(change)` to notify Salsa
2. Salsa automatically invalidates cached results that depend on the changed file
3. Next query (e.g., `sema.type_of_pat()`) recomputes only what's needed
4. Unchanged files/functions return cached results instantly

**Code (state.rs):**
```rust
impl GlobalState {
    pub fn apply_vfs_changes(&mut self) {
        if let Some(ws) = &mut self.workspace {
            let changes = ws.vfs.take_changes();
            if changes.is_empty() {
                return;
            }

            let mut change = ra_ap_ide::Change::new();
            for vfs_change in changes {
                let file_id = vfs_change.file_id;
                let content = ws.vfs.file_contents(file_id);
                change.change_file(file_id, Some(content.to_vec()));
            }

            ws.db.apply_change(change);
            tracing::debug!("Applied {} VFS changes to database", changes.len());
        }
    }
}
```

**Salsa invalidation model:**
```
File A changes
    │
    ▼
Salsa invalidates:
    ├── parse(A)           ← re-parse the file
    ├── item_tree(A)       ← re-extract items
    ├── type_of(var in A)  ← re-resolve types in A
    │
    NOT invalidated:
    ├── parse(B)           ← other files unchanged
    ├── type_of(var in B)  ← cached, instant
    └── sysroot types      ← never change
```

**Expectation:** After a single-file change, re-analysis of that file takes < 100ms (not 30-40s). Queries about unchanged files return instantly from cache.

**Tests for 1.6:**
- After file change, `type_of_pat()` returns updated type
- Unchanged files still return cached results (verify via timing)
- Multiple rapid changes are batched (not analyzed individually)
- Adding a new function doesn't invalidate existing function analysis
- Changing a type definition invalidates all usages of that type

---

## 1.7 Startup Performance (Background Loading, Progress Notifications)

**Objective:** The 30-40 second workspace loading should not block the UI. Load in a background thread and send progress notifications to the client so the user sees "BorrowScope: Loading workspace..." in the status bar.

**Steps:**
1. After `initialized`, spawn a background thread for workspace loading
2. Send `window/workDoneProgress/create` to register a progress token
3. Send `$/progress` notifications with percentage updates
4. When loading completes, move `WorkspaceData` into `GlobalState`
5. Send `$/progress` end notification

**Code (server.rs):**
```rust
fn start_workspace_loading(
    state: &mut GlobalState,
    sender: &Sender<Message>,
) {
    let root_path = state.root_path.clone();
    let sender = sender.clone();

    // Create progress token
    let token = lsp_types::ProgressToken::String("borrowscope/loading".to_string());
    send_request::<lsp_types::request::WorkDoneProgressCreate>(
        &sender,
        lsp_types::WorkDoneProgressCreateParams { token: token.clone() },
    );

    // Send begin
    send_progress(&sender, &token, WorkDoneProgress::Begin(WorkDoneProgressBegin {
        title: "BorrowScope".to_string(),
        message: Some("Loading workspace...".to_string()),
        percentage: Some(0),
        cancellable: Some(false),
    }));

    // Spawn loading thread
    std::thread::spawn(move || {
        let result = workspace::load_workspace(&root_path);

        send_progress(&sender, &token, WorkDoneProgress::End(WorkDoneProgressEnd {
            message: Some("Ready".to_string()),
        }));

        // Send result back to main thread via channel
        // (actual implementation uses crossbeam channel)
    });
}
```

**User experience during loading:**
```
┌─────────────────────────────────────────────────────────┐
│  VS Code Status Bar                                      │
│                                                          │
│  $(loading~spin) BorrowScope: Loading workspace... 45%   │
│                                                          │
│  (after 30-40s)                                          │
│                                                          │
│  $(check) BorrowScope: Ready (523 variables analyzed)    │
└─────────────────────────────────────────────────────────┘
```

**Expectation:** The editor remains responsive during loading. The user sees progress. Once loading completes, the server immediately starts serving requests.

**Tests for 1.7:**
- Progress begin notification is sent after initialized
- Progress end notification is sent when loading completes
- Server responds to requests during loading (with "not ready" error)
- Loading failure sends progress end with error message
- Loading time for a small project (1 file) is < 5 seconds
- Loading time for a medium project (20 files) is < 60 seconds

---

## 1.T Integration Test Suite

**Objective:** End-to-end tests that spawn the server binary, send LSP messages, and verify responses. Uses a test harness that simulates VS Code's behavior.

**Test harness:**
```rust
struct TestServer {
    process: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: i32,
}

impl TestServer {
    fn start() -> Self { /* spawn borrowscope-lsp binary */ }
    fn send_request<R: lsp_types::request::Request>(&mut self, params: R::Params) -> R::Result;
    fn send_notification<N: lsp_types::notification::Notification>(&mut self, params: N::Params);
    fn receive_notification<N: lsp_types::notification::Notification>(&mut self) -> N::Params;
    fn initialize(&mut self, root: &Path) -> InitializeResult;
    fn shutdown(&mut self);
}
```

**Integration tests:**
```rust
#[test]
fn test_full_lifecycle() {
    let server = TestServer::start();
    let result = server.initialize(fixture_project_path());
    assert!(result.capabilities.text_document_sync.is_some());
    server.send_notification::<Initialized>(InitializedParams {});
    // Wait for loading to complete
    server.wait_for_progress_end("borrowscope/loading");
    server.shutdown();
}

#[test]
fn test_workspace_loads_and_resolves_types() {
    let server = TestServer::start();
    server.initialize(fixture_project_path());
    server.send_notification::<Initialized>(InitializedParams {});
    server.wait_for_progress_end("borrowscope/loading");

    // Send a custom request for variable info
    let result = server.send_request::<BorrowScopeVariableInfo>(VariableInfoParams {
        file: "src/main.rs".into(),
        line: 5,
        column: 8,
    });
    assert!(!result.ty.contains("unknown"));
}
```

**Fixture project:**
```
tests/fixtures/simple-project/
├── Cargo.toml
└── src/
    └── main.rs    # Contains known types for assertion
```

**Test coverage summary:**
- Server binary starts and exits cleanly
- LSP handshake completes successfully
- Workspace loads for fixture project
- Types resolve correctly after loading
- File changes update the VFS
- Shutdown is graceful
- Invalid messages don't crash the server
- Progress notifications are sent during loading
