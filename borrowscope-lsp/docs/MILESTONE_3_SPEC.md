# Milestone 3: LSP Custom Requests and Notifications - Detailed Specification

## 3.1 Custom Request: `borrowscope/ownershipGraph`

**Objective:** Define a custom LSP request that returns the complete ownership graph for a specific function. The client sends a function identifier (file + position or function name), and the server returns the `FunctionOwnershipSummary` from Milestone 2 serialized as JSON.

**Steps:**
1. Define the request method name: `borrowscope/ownershipGraph`
2. Define request params: `{ uri: string, position: Position }` (cursor inside a function)
3. Define response: the full `FunctionOwnershipSummary` JSON
4. Register the handler in the server's request dispatch
5. Resolve which function the cursor is inside, then call `analyze_function()`

**Protocol definition:**
```
Request:
  method: "borrowscope/ownershipGraph"
  params: {
    "textDocument": { "uri": "file:///path/to/src/main.rs" },
    "position": { "line": 5, "character": 0 }
  }

Response:
  result: {
    "function_name": "process_data",
    "variables": [ ... ],
    "borrow_scopes": [ ... ],
    "moves": [ ... ],
    "closures": [ ... ],
    "rc_clones": [ ... ],
    "conflicts": [ ... ],
    "stats": { "total_variables": 8, "total_borrows": 3, ... }
  }
```

**Code (handlers/requests.rs):**
```rust
pub fn handle_ownership_graph(
    state: &GlobalState,
    params: OwnershipGraphParams,
) -> anyhow::Result<FunctionOwnershipSummary> {
    let ws = state.workspace.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Workspace not loaded"))?;

    let file_id = uri_to_file_id(&ws.vfs, &params.text_document.uri)?;
    let position = lsp_position_to_offset(&ws.vfs, file_id, params.position)?;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(file_id);

    // Find the function containing the cursor position
    let function = find_enclosing_function(&source_file, position)
        .ok_or_else(|| anyhow::anyhow!("Cursor not inside a function"))?;

    let summary = analysis::analyze_function(&ws.db, &sema, &function);
    Ok(summary)
}

#[derive(Debug, Deserialize)]
pub struct OwnershipGraphParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}
```

**Expectation:** Client sends cursor position, server returns the full ownership analysis for the enclosing function. Response time < 100ms for typical functions (analysis is cached by Salsa).

**Tests for 3.1:**
- Request with cursor inside a function returns valid summary
- Request with cursor outside any function returns error
- Request for file not in workspace returns error
- Response contains all expected fields (variables, borrow_scopes, etc.)
- Response is valid JSON parseable by the TypeScript client
- Repeated requests for same function return cached result (fast)

---

## 3.2 Custom Request: `borrowscope/borrowScopes`

**Objective:** Return all borrow scopes for an entire file, suitable for rendering inline decorations. Unlike 3.1 which returns data for one function, this returns lightweight borrow range data for all functions in the file.

**Protocol definition:**
```
Request:
  method: "borrowscope/borrowScopes"
  params: {
    "textDocument": { "uri": "file:///path/to/src/main.rs" }
  }

Response:
  result: {
    "scopes": [
      {
        "borrower": "r",
        "target": "data",
        "is_mutable": false,
        "range": { "start": {"line":5,"character":4}, "end": {"line":8,"character":0} }
      },
      ...
    ]
  }
```

**Code:**
```rust
#[derive(Debug, Serialize)]
pub struct BorrowScopesResponse {
    pub scopes: Vec<BorrowScopeRange>,
}

#[derive(Debug, Serialize)]
pub struct BorrowScopeRange {
    pub borrower: String,
    pub target: String,
    pub is_mutable: bool,
    pub range: lsp_types::Range,
}

pub fn handle_borrow_scopes(
    state: &GlobalState,
    params: TextDocumentIdentifier,
) -> anyhow::Result<BorrowScopesResponse> {
    let ws = state.workspace.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Workspace not loaded"))?;

    let file_id = uri_to_file_id(&ws.vfs, &params.uri)?;
    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(file_id);

    let mut scopes = Vec::new();
    for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
        let fn_scopes = analysis::compute_borrow_scopes(&ws.db, &sema, &function);
        for scope in fn_scopes {
            scopes.push(BorrowScopeRange {
                borrower: scope.borrower_name,
                target: scope.target_name,
                is_mutable: scope.is_mutable,
                range: offset_range_to_lsp_range(&ws.vfs, file_id, scope.start, scope.end),
            });
        }
    }

    Ok(BorrowScopesResponse { scopes })
}
```

**Expectation:** Returns all borrow scopes in the file as LSP ranges. The client uses these to render colored background highlights in the editor.

**Tests for 3.2:**
- File with no borrows returns empty scopes array
- File with one borrow returns one scope with correct range
- Mutable borrow has `is_mutable: true`
- Scope range starts at borrow declaration and ends at last use
- Multiple functions each contribute their own scopes
- Response time < 200ms for a file with 10 functions

---

## 3.3 Custom Request: `borrowscope/variableInfo`

**Objective:** Return detailed ownership information for a single variable at a specific cursor position. Used for hover tooltips and the "inspect variable" command.

**Protocol definition:**
```
Request:
  method: "borrowscope/variableInfo"
  params: {
    "textDocument": { "uri": "file:///..." },
    "position": { "line": 10, "character": 8 }
  }

Response:
  result: {
    "name": "data",
    "type_display": "Vec<i32>",
    "ownership_category": "Owned",
    "is_copy": false,
    "borrows_from": [],
    "borrowed_by": ["r", "m"],
    "moved_to": "result",
    "traits": ["Clone", "Drop", "Send", "Sync"],
    "layout_size": 24,
    "borrow_scope": { "start": {"line":5}, "end": {"line":12} }
  }
```

**Code:**
```rust
#[derive(Debug, Serialize)]
pub struct VariableInfoResponse {
    pub name: String,
    pub type_display: String,
    pub ownership_category: String,
    pub is_copy: bool,
    pub borrows_from: Vec<String>,
    pub borrowed_by: Vec<String>,
    pub moved_to: Option<String>,
    pub traits: Vec<String>,
    pub layout_size: Option<u64>,
    pub borrow_scope: Option<lsp_types::Range>,
}

pub fn handle_variable_info(
    state: &GlobalState,
    params: VariableInfoParams,
) -> anyhow::Result<Option<VariableInfoResponse>> {
    // Find the variable at the cursor position
    // Extract its full type info + relationships from the function summary
    todo!()
}
```

**Expectation:** Hovering over a variable shows its complete ownership profile: what it borrows from, who borrows it, where it moves, and its type properties.

**Tests for 3.3:**
- Cursor on a variable name returns its info
- Cursor on whitespace returns null
- `borrowed_by` lists all variables that borrow from this one
- `moved_to` shows destination if the variable was moved
- `traits` lists all implemented traits
- Response time < 50ms (single variable lookup)

---

## 3.4 Custom Notification: `borrowscope/analysisUpdated`

**Objective:** When the server re-analyzes a file (after a change), push a notification to the client so it can refresh its visualizations. The client does not need to poll; the server tells it when new data is available.

**Protocol definition:**
```
Notification (server → client):
  method: "borrowscope/analysisUpdated"
  params: {
    "uri": "file:///path/to/src/main.rs",
    "functions": ["main", "process_data"],
    "timestamp": 1715500000
  }
```

**Code:**
```rust
pub fn notify_analysis_updated(
    sender: &Sender<Message>,
    uri: &Url,
    functions: &[String],
) {
    let params = serde_json::json!({
        "uri": uri.as_str(),
        "functions": functions,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs()
    });

    let notif = lsp_server::Notification::new(
        "borrowscope/analysisUpdated".to_string(),
        params,
    );
    sender.send(Message::Notification(notif)).ok();
}
```

**Flow:**
```
User edits file
    │
    ▼
Server receives didChange
    │
    ▼
Server updates VFS, triggers re-analysis (debounced)
    │
    ▼
Analysis completes for affected functions
    │
    ▼
Server sends borrowscope/analysisUpdated to client
    │
    ▼
Client re-fetches ownershipGraph for affected functions
    │
    ▼
WebView panel and decorations update
```

**Expectation:** The client receives a push notification within 500ms of a file change (after debounce). It then knows which functions to re-query.

**Tests for 3.4:**
- Notification sent after file change and re-analysis
- Notification contains the correct URI
- Notification lists only functions that were affected by the change
- No notification sent if the change doesn't affect any function's ownership
- Multiple rapid changes produce only one notification (debounced)

---

## 3.5 Standard LSP: `textDocument/publishDiagnostics`

**Objective:** Publish borrow conflict diagnostics using the standard LSP diagnostics mechanism. These appear in VS Code's Problems panel alongside rust-analyzer's errors, but with a `[BorrowScope]` source tag.

**Code:**
```rust
pub fn publish_ownership_diagnostics(
    sender: &Sender<Message>,
    uri: &Url,
    conflicts: &[BorrowConflict],
) {
    let diagnostics: Vec<Diagnostic> = conflicts.iter().map(|c| {
        Diagnostic {
            range: conflict_to_range(c),
            severity: Some(DiagnosticSeverity::INFORMATION), // Not error (compiler handles that)
            source: Some("BorrowScope".to_string()),
            message: c.message.clone(),
            related_information: Some(vec![
                DiagnosticRelatedInformation {
                    location: Location { uri: uri.clone(), range: borrow_a_range(c) },
                    message: format!("First borrow ({}) here", c.borrow_a.borrower),
                },
                DiagnosticRelatedInformation {
                    location: Location { uri: uri.clone(), range: borrow_b_range(c) },
                    message: format!("Second borrow ({}) here", c.borrow_b.borrower),
                },
            ]),
            ..Default::default()
        }
    }).collect();

    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version: None,
    };

    let notif = lsp_server::Notification::new(
        "textDocument/publishDiagnostics".to_string(),
        serde_json::to_value(params).unwrap(),
    );
    sender.send(Message::Notification(notif)).ok();
}
```

**Diagnostic appearance in VS Code:**
```
┌─────────────────────────────────────────────────────────────┐
│ PROBLEMS                                                     │
├─────────────────────────────────────────────────────────────┤
│ ℹ [BorrowScope] Potential borrow conflict: `r` (shared)     │
│   and `m` (mutable) both borrow `data` with overlapping     │
│   scopes (lines 5-12)                                        │
│                                                              │
│   src/main.rs:8:4  First borrow (r) here                    │
│   src/main.rs:10:4 Second borrow (m) here                   │
└─────────────────────────────────────────────────────────────┘
```

**Expectation:** Conflicts appear as informational diagnostics (not errors, since the compiler already prevents them). They serve as educational annotations showing WHY the borrow checker would reject certain patterns.

**Tests for 3.5:**
- Conflict produces a diagnostic with correct range
- Diagnostic severity is Information (not Error)
- Diagnostic source is "BorrowScope"
- Related information points to both borrow locations
- Diagnostics clear when the conflict is resolved (file edited)
- No diagnostics for valid code (no false positives)

---

## 3.6 Standard LSP: `textDocument/codeLens`

**Objective:** Show ownership statistics above each function as a CodeLens. The user sees "3 borrows, 1 move, 0 conflicts" above the function signature, providing at-a-glance ownership complexity.

**Code:**
```rust
pub fn handle_code_lens(
    state: &GlobalState,
    params: CodeLensParams,
) -> anyhow::Result<Vec<CodeLens>> {
    let ws = state.workspace.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Workspace not loaded"))?;

    let file_id = uri_to_file_id(&ws.vfs, &params.text_document.uri)?;
    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(file_id);

    let mut lenses = Vec::new();
    for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
        let summary = analysis::analyze_function(&ws.db, &sema, &function);
        let range = function_name_range(&ws.vfs, file_id, &function);

        let title = format!(
            "{} vars, {} borrows, {} moves{}",
            summary.stats.total_variables,
            summary.stats.total_borrows,
            summary.stats.moves,
            if summary.stats.conflicts > 0 {
                format!(", {} conflicts!", summary.stats.conflicts)
            } else { String::new() }
        );

        lenses.push(CodeLens {
            range,
            command: Some(Command {
                title,
                command: "borrowscope.showGraph".to_string(),
                arguments: Some(vec![serde_json::to_value(&params.text_document.uri)?]),
            }),
            data: None,
        });
    }

    Ok(lenses)
}
```

**Appearance in editor:**
```rust
  ▸ 8 vars, 3 borrows, 1 move                    ← CodeLens (clickable)
  fn process_data(input: &[u8]) -> Result<Output> {
      let data = parse(input)?;
      let validated = &data;
      ...
  }
```

**Expectation:** Every function gets a CodeLens showing its ownership summary. Clicking it opens the ownership graph panel for that function.

**Tests for 3.6:**
- Each function in the file gets one CodeLens
- CodeLens title contains correct counts
- CodeLens range is on the function name line
- Clicking CodeLens triggers `borrowscope.showGraph` command
- Functions with conflicts show conflict count in title
- Empty functions show "0 vars, 0 borrows, 0 moves"

---

## 3.7 Standard LSP: `textDocument/inlayHints`

**Objective:** Show ownership category annotations inline next to variable declarations. The user sees `[Rc]`, `[&mut]`, `[move]` hints next to `let` bindings without cluttering the source code.

**Code:**
```rust
pub fn handle_inlay_hints(
    state: &GlobalState,
    params: InlayHintParams,
) -> anyhow::Result<Vec<InlayHint>> {
    let ws = state.workspace.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Workspace not loaded"))?;

    let file_id = uri_to_file_id(&ws.vfs, &params.text_document.uri)?;
    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(file_id);

    let mut hints = Vec::new();
    for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
        let summary = analysis::analyze_function(&ws.db, &sema, &function);
        for var in &summary.variables {
            let label = match &var.ownership_category {
                OwnershipCategory::SharedOwnership => "Rc",
                OwnershipCategory::MutableRef => "&mut",
                OwnershipCategory::SharedRef => "&",
                OwnershipCategory::InteriorMut => "Cell",
                OwnershipCategory::RawPointer => "*ptr",
                OwnershipCategory::Copy => "copy",
                _ => continue, // Don't show hint for plain owned types
            };

            hints.push(InlayHint {
                position: Position { line: var.line - 1, character: var.column + var.name.len() as u32 },
                label: InlayHintLabel::String(format!(" [{}]", label)),
                kind: Some(InlayHintKind::TYPE),
                padding_left: Some(true),
                ..Default::default()
            });
        }
    }

    Ok(hints)
}
```

**Appearance in editor:**
```rust
fn example() {
    let data = vec![1, 2, 3];
    let r [&] = &data;
    let rc [Rc] = Rc::new(42);
    let guard [Cell] = cell.borrow_mut();
    let ptr [*ptr] = &data as *const _;
}
```

**Expectation:** Ownership category hints appear inline, providing instant visual classification without hovering. Only non-obvious categories are shown (plain owned types get no hint to reduce noise).

**Tests for 3.7:**
- `Rc<T>` variable gets `[Rc]` hint
- `&T` variable gets `[&]` hint
- `&mut T` variable gets `[&mut]` hint
- Plain `Vec<T>` gets no hint (owned is the default, no annotation needed)
- `i32` gets no hint (Copy is obvious for primitives)
- Hints appear at correct positions (after variable name)
- Hints respect the visible range (only compute for visible lines)

---

## 3.T Integration Test Suite

**Test harness extension (from Milestone 1):**
```rust
impl TestServer {
    fn request_ownership_graph(&mut self, uri: &str, line: u32, col: u32)
        -> FunctionOwnershipSummary;
    fn request_borrow_scopes(&mut self, uri: &str)
        -> BorrowScopesResponse;
    fn request_variable_info(&mut self, uri: &str, line: u32, col: u32)
        -> Option<VariableInfoResponse>;
    fn wait_for_diagnostics(&mut self, uri: &str)
        -> Vec<Diagnostic>;
    fn request_code_lens(&mut self, uri: &str)
        -> Vec<CodeLens>;
    fn request_inlay_hints(&mut self, uri: &str, range: Range)
        -> Vec<InlayHint>;
}
```

**End-to-end tests:**
```rust
#[test]
fn test_ownership_graph_request() {
    let mut server = TestServer::start();
    server.initialize(fixture_path());
    server.wait_ready();
    server.open_file("src/main.rs");

    let graph = server.request_ownership_graph("src/main.rs", 5, 0);
    assert!(!graph.variables.is_empty());
    assert_eq!(graph.function_name, "main");
}

#[test]
fn test_diagnostics_on_conflict() {
    let mut server = TestServer::start();
    server.initialize(fixture_with_conflict());
    server.wait_ready();
    server.open_file("src/main.rs");

    let diagnostics = server.wait_for_diagnostics("src/main.rs");
    assert!(diagnostics.iter().any(|d| d.source == Some("BorrowScope".into())));
}

#[test]
fn test_analysis_updated_notification() {
    let mut server = TestServer::start();
    server.initialize(fixture_path());
    server.wait_ready();
    server.open_file("src/main.rs");

    // Edit the file
    server.change_file("src/main.rs", "let new_var = 42;");

    // Should receive analysisUpdated notification
    let notif = server.wait_for_notification::<AnalysisUpdated>();
    assert_eq!(notif.uri, "file:///...src/main.rs");
}
```

**Fixture files for testing:**
```
tests/fixtures/
├── simple-project/
│   ├── Cargo.toml
│   └── src/main.rs          # Basic ownership patterns
├── conflict-project/
│   ├── Cargo.toml
│   └── src/main.rs          # Contains borrow conflicts
├── smart-pointers/
│   ├── Cargo.toml
│   └── src/main.rs          # Rc, Arc, RefCell patterns
└── closures/
    ├── Cargo.toml
    └── src/main.rs           # Closure capture patterns
```
