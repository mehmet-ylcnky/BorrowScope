# Milestone 11: Cross-Function and Cross-File Borrow Tracking

## Overview

Track borrow lifetimes across function boundaries and file boundaries. When a reference is passed from a caller to a callee (even in a different file), show the full borrow path as a unified visualization spanning the entire call chain.

**Current state:** Each function is analyzed independently. A borrow created in `main()` and passed to `process(&data)` shows as two disconnected scopes.

**After this milestone:** A unified lifeline spans the entire borrow path, with clickable navigation between files.

---

## 11.1 Inter-Procedural Borrow Analysis

**Objective:** Extend the analysis engine to trace borrows through function calls.

**Example — what the user sees today (disconnected):**
```rust
// src/main.rs
fn main() {
    let data = vec![1, 2, 3];
    ├─ 👁 &data ⟵ data           // borrow scope in main
    │  process(&data);
    ╰─ 💧 &data released
}

// src/processor.rs (separate, no connection shown)
fn process(input: &[i32]) {
    ├─ 👁 &input ⟵ ???           // where does this come from?
    │  println!("{:?}", input);
    ╰─ 💧 input released
}
```

**After this milestone (connected):**
```rust
// src/main.rs
fn main() {
    let data = vec![1, 2, 3];
    ├─ 👁 &data ⟵ data
    │  process(&data);            // ──→ enters process() [src/processor.rs:5]
    │  │ 👁 input receives &data
    │  │ println!("{:?}", input);
    │  │ 💧 input released
    │  // ◄── returns from process()
    ╰─ 💧 &data released
}
```

---

## 11.2 Server-Side: Call Graph Resolution

**New analysis function (analysis.rs):**

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CrossFunctionBorrow {
    pub origin_variable: String,
    pub origin_file: String,
    pub origin_line: u32,
    pub path: Vec<BorrowPathSegment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BorrowPathSegment {
    pub file: String,
    pub function_name: String,
    pub variable: String,
    pub start_line: u32,
    pub end_line: u32,
    pub is_mutable: bool,
    pub kind: BorrowPathKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum BorrowPathKind {
    Origin,       // where the borrow is created (&data)
    Parameter,    // received as function parameter
    PassThrough,  // forwarded to another function
    Return,       // returned from function
}

/// Analyze cross-function borrows for all functions in a file.
pub fn analyze_cross_function_borrows(
    db: &RootDatabase,
    sema: &Semantics<'_, RootDatabase>,
    source_file: &ast::SourceFile,
    file_path: &str,
    line_index: &dyn Fn(TextSize) -> (u32, u32),
    max_depth: usize,  // limit call chain depth (default: 3)
) -> Vec<CrossFunctionBorrow> {
    let mut results = Vec::new();

    for function in source_file.syntax().descendants().filter_map(ast::Fn::cast) {
        let body = match function.body() { Some(b) => b, None => continue };

        // Find all call expressions that pass references
        for call in body.syntax().descendants().filter_map(ast::CallExpr::cast) {
            let args = match call.arg_list() { Some(a) => a, None => continue };

            for (idx, arg) in args.args().enumerate() {
                // Check if argument is a reference
                let ty = match sema.type_of_expr(&arg) {
                    Some(t) => t,
                    None => continue,
                };

                if !ty.original.is_reference() { continue; }

                // Resolve the call target
                let target_fn = resolve_call_target(sema, &call);
                if target_fn.is_none() { continue; }
                let target_fn = target_fn.unwrap();

                // Get the target function's parameter at this index
                let param = get_parameter_at(db, &target_fn, idx);
                if param.is_none() { continue; }

                // Build the borrow path
                let mut path = Vec::new();

                // Segment 1: caller's borrow
                let origin_var = arg.syntax().text().to_string().trim()
                    .trim_start_matches('&').trim_start_matches("mut ").to_string();
                let (call_line, _) = line_index(call.syntax().text_range().start());
                path.push(BorrowPathSegment {
                    file: file_path.to_string(),
                    function_name: function.name().map(|n| n.text().to_string()).unwrap_or_default(),
                    variable: origin_var.clone(),
                    start_line: call_line,
                    end_line: call_line,
                    is_mutable: ty.original.is_mutable_reference(),
                    kind: BorrowPathKind::Origin,
                });

                // Segment 2: callee's parameter
                let target_source = target_fn.source(db);
                if let Some(source) = target_source {
                    let target_file = source.file_id;
                    // ... resolve file path, analyze callee's use of the parameter
                    // ... recursively trace if depth < max_depth
                }

                if path.len() > 1 {
                    results.push(CrossFunctionBorrow {
                        origin_variable: origin_var,
                        origin_file: file_path.to_string(),
                        origin_line: call_line,
                        path,
                    });
                }
            }
        }
    }

    results
}

/// Resolve a CallExpr to its target hir::Function
fn resolve_call_target(
    sema: &Semantics<'_, RootDatabase>,
    call: &ast::CallExpr,
) -> Option<hir::Function> {
    let callee = call.expr()?;
    match &callee {
        ast::Expr::PathExpr(path_expr) => {
            let path = path_expr.path()?;
            let resolution = sema.resolve_path(&path)?;
            match resolution {
                hir::PathResolution::Def(hir::ModuleDef::Function(f)) => Some(f),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Resolve a MethodCallExpr to its target hir::Function
fn resolve_method_target(
    sema: &Semantics<'_, RootDatabase>,
    call: &ast::MethodCallExpr,
) -> Option<hir::Function> {
    sema.resolve_method_call(call)
}

/// Get the parameter at a given index from a function
fn get_parameter_at(
    db: &RootDatabase,
    func: &hir::Function,
    idx: usize,
) -> Option<hir::Param> {
    func.params_without_self(db).get(idx).cloned()
}
```

---

## 11.3 LSP Custom Request

**New endpoint: `borrowscope/crossFunctionBorrows`**

```
Request:
  method: "borrowscope/crossFunctionBorrows"
  params: {
    "textDocument": { "uri": "file:///project/src/main.rs" },
    "maxDepth": 3
  }

Response:
  result: {
    "cross_borrows": [
      {
        "origin_variable": "data",
        "origin_file": "file:///project/src/main.rs",
        "origin_line": 5,
        "path": [
          {
            "file": "file:///project/src/main.rs",
            "function_name": "main",
            "variable": "&data",
            "start_line": 5,
            "end_line": 12,
            "is_mutable": false,
            "kind": "Origin"
          },
          {
            "file": "file:///project/src/processor.rs",
            "function_name": "transform",
            "variable": "input",
            "start_line": 1,
            "end_line": 8,
            "is_mutable": false,
            "kind": "Parameter"
          },
          {
            "file": "file:///project/src/processor.rs",
            "function_name": "parse",
            "variable": "data",
            "start_line": 12,
            "end_line": 15,
            "is_mutable": false,
            "kind": "PassThrough"
          }
        ]
      }
    ]
  }
```

**Handler (handlers/requests.rs):**

```rust
fn handle_cross_function_borrows(
    state: &mut GlobalState,
    sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    #[derive(Deserialize)]
    struct Params {
        #[serde(rename = "textDocument")]
        text_document: lsp_types::TextDocumentIdentifier,
        #[serde(rename = "maxDepth", default = "default_depth")]
        max_depth: usize,
    }
    fn default_depth() -> usize { 3 }

    let params: Params = serde_json::from_value(req.params)?;
    let ws = match &state.workspace { Some(ws) => ws, None => { /* return error */ } };

    // ... resolve file, build sema, call analyze_cross_function_borrows
    // ... serialize and return
}
```

---

## 11.4 VS Code Extension: Cross-Function Lifeline Rendering

**New file: `src/cross-lifelines.ts`**

```typescript
import * as vscode from "vscode";

interface CrossBorrow {
  origin_variable: string;
  origin_file: string;
  origin_line: number;
  path: BorrowPathSegment[];
}

interface BorrowPathSegment {
  file: string;
  function_name: string;
  variable: string;
  start_line: number;
  end_line: number;
  is_mutable: boolean;
  kind: "Origin" | "Parameter" | "PassThrough" | "Return";
}

const crossBorrowDecoration = vscode.window.createTextEditorDecorationType({
  before: { color: "#1abc9c", fontWeight: "bold" },
});

export async function applyCrossLifelines(
  editor: vscode.TextEditor,
  client: LanguageClient
): Promise<void> {
  const response = await client.sendRequest("borrowscope/crossFunctionBorrows", {
    textDocument: { uri: editor.document.uri.toString() },
    maxDepth: 3,
  });

  const crossBorrows: CrossBorrow[] = (response as any)?.cross_borrows || [];
  const decorations: vscode.DecorationOptions[] = [];

  for (const cb of crossBorrows) {
    for (const segment of cb.path) {
      // Only render segments in the current file
      if (segment.file !== editor.document.uri.toString()) continue;

      const line = segment.start_line - 1;
      let suffix = "";

      switch (segment.kind) {
        case "Origin":
          suffix = ` ──→ enters ${getNextSegment(cb, segment)?.function_name}()`;
          break;
        case "Parameter":
          suffix = ` ◄── received from ${getPrevSegment(cb, segment)?.function_name}()`;
          break;
        case "PassThrough":
          suffix = ` ──→ forwarded to ${getNextSegment(cb, segment)?.function_name}()`;
          break;
        case "Return":
          suffix = ` ◄── returned to caller`;
          break;
      }

      decorations.push({
        range: new vscode.Range(line, 0, line, 0),
        renderOptions: {
          after: {
            contentText: suffix,
            color: "rgba(26, 188, 156, 0.7)",
            fontStyle: "italic",
            margin: "0 0 0 2em",
          },
        },
        hoverMessage: buildCrossHover(cb, segment),
      });
    }
  }

  editor.setDecorations(crossBorrowDecoration, decorations);
}

function buildCrossHover(cb: CrossBorrow, segment: BorrowPathSegment): vscode.MarkdownString {
  const md = new vscode.MarkdownString();
  md.appendMarkdown(`**Cross-function borrow:** \`${cb.origin_variable}\`\n\n`);
  md.appendMarkdown(`**Path:**\n\n`);
  for (const s of cb.path) {
    const marker = s === segment ? "→ " : "  ";
    const file = s.file.split("/").pop();
    md.appendMarkdown(`${marker}\`${s.function_name}()\` in ${file} (line ${s.start_line})\n\n`);
  }
  md.isTrusted = true;
  return md;
}

// Navigation: click to jump to cross-file borrow
export function registerCrossFileNavigation(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("borrowscope.jumpToBorrowSource", async (uri: string, line: number) => {
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri));
      const editor = await vscode.window.showTextDocument(doc);
      const pos = new vscode.Position(line - 1, 0);
      editor.selection = new vscode.Selection(pos, pos);
      editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
    })
  );
}
```

---

## 11.5 Cross-File Resolution

**How to resolve a function call to its source file:**

```rust
/// Given a hir::Function, find its source file path
fn get_function_file_path(
    db: &RootDatabase,
    vfs: &Vfs,
    func: &hir::Function,
) -> Option<String> {
    let source = func.source(db)?;
    let file_id = source.file_id.original_file(db);
    let vfs_path = vfs.file_path(file_id);
    Some(vfs_path.to_string())
}

/// Resolve a method call across modules
fn trace_method_call_cross_file(
    db: &RootDatabase,
    sema: &Semantics<'_, RootDatabase>,
    vfs: &Vfs,
    method_call: &ast::MethodCallExpr,
) -> Option<(hir::Function, String)> {
    let func = sema.resolve_method_call(method_call)?;
    let file_path = get_function_file_path(db, vfs, &func)?;
    Some((func, file_path))
}
```

**Cross-module example:**

```rust
// src/main.rs
mod utils;

fn main() {
    let data = vec![1, 2, 3];
    let result = utils::summarize(&data);  // crosses to src/utils.rs
    println!("{}", result);
}

// src/utils.rs
pub fn summarize(items: &[i32]) -> String {
    let total: i32 = items.iter().sum();
    format!("Sum: {}", total)
}
```

**What the server returns:**
```json
{
  "cross_borrows": [{
    "origin_variable": "data",
    "origin_file": "file:///project/src/main.rs",
    "origin_line": 4,
    "path": [
      { "file": "file:///project/src/main.rs", "function_name": "main", "variable": "&data", "start_line": 4, "end_line": 5, "is_mutable": false, "kind": "Origin" },
      { "file": "file:///project/src/utils.rs", "function_name": "summarize", "variable": "items", "start_line": 1, "end_line": 3, "is_mutable": false, "kind": "Parameter" }
    ]
  }]
}
```

---

## 11.6 Implementation Phases

### Phase 1: Direct calls, same file (2-3 days)

```rust
// What's supported:
fn main() {
    let data = vec![1, 2, 3];
    process(&data);  // ✅ direct call, same file
}

fn process(input: &[i32]) { ... }
```

**Implementation:**
1. Find all `CallExpr` and `MethodCallExpr` in function body
2. Check if any argument is a reference type
3. Resolve call target via `sema.resolve_path()` or `sema.resolve_method_call()`
4. Get target function's source (must be in same file for Phase 1)
5. Analyze target's use of the parameter
6. Return combined path

### Phase 2: Cross-file, one level deep (3-5 days)

```rust
// What's supported:
// src/main.rs
fn main() {
    let data = vec![1, 2, 3];
    utils::process(&data);  // ✅ cross-file, one level
}

// src/utils.rs
pub fn process(input: &[i32]) {
    helpers::validate(input);  // ❌ not traced (depth > 1)
}
```

**Additional implementation:**
1. Use `hir::Function::source()` to get `FileId`
2. Map `FileId` to VFS path for the URI
3. Parse and analyze the target file
4. Handle `pub` visibility (only trace into accessible functions)

### Phase 3: Multi-level with cycle detection (5-7 days)

```rust
// What's supported:
fn main() {
    let data = vec![1, 2, 3];
    let result = pipeline::run(&data);  // ✅ traces 3 levels deep
}

// pipeline.rs
pub fn run(input: &[i32]) -> Output {
    let cleaned = clean(input);     // level 2
    transform(&cleaned)             // level 2
}

fn clean(data: &[i32]) -> Vec<i32> {
    filter(data)                    // level 3 (max depth)
}
```

**Additional implementation:**
1. Recursive tracing with `max_depth` parameter (default: 3)
2. `HashSet<hir::Function>` for cycle detection
3. Configurable via `borrowscope.crossFunction.maxDepth` setting
4. Performance budget: abort if analysis exceeds 200ms

---

## 11.7 Cross-File Navigation in VS Code

**DocumentLink provider for borrow paths:**

```typescript
class BorrowPathLinkProvider implements vscode.DocumentLinkProvider {
  constructor(private client: LanguageClient) {}

  async provideDocumentLinks(document: vscode.TextDocument): Promise<vscode.DocumentLink[]> {
    const response = await this.client.sendRequest("borrowscope/crossFunctionBorrows", {
      textDocument: { uri: document.uri.toString() },
    });

    const links: vscode.DocumentLink[] = [];
    for (const cb of (response as any)?.cross_borrows || []) {
      for (let i = 0; i < cb.path.length - 1; i++) {
        const segment = cb.path[i];
        const next = cb.path[i + 1];
        if (segment.file === document.uri.toString() && next.file !== segment.file) {
          // Create a clickable link on the call line
          const range = new vscode.Range(segment.end_line - 1, 0, segment.end_line - 1, 100);
          const target = vscode.Uri.parse(next.file).with({
            fragment: `L${next.start_line}`,
          });
          links.push(new vscode.DocumentLink(range, target));
        }
      }
    }
    return links;
  }
}
```

**Split editor view (optional enhancement):**

```typescript
async function showCrossBorrowSplit(borrow: CrossBorrow) {
  const files = [...new Set(borrow.path.map(s => s.file))];
  if (files.length < 2) return;

  // Open first file on the left
  const doc1 = await vscode.workspace.openTextDocument(vscode.Uri.parse(files[0]));
  await vscode.window.showTextDocument(doc1, vscode.ViewColumn.One);

  // Open second file on the right
  const doc2 = await vscode.workspace.openTextDocument(vscode.Uri.parse(files[1]));
  await vscode.window.showTextDocument(doc2, vscode.ViewColumn.Two);

  // Highlight the borrow path in both editors
  // ... apply decorations to both
}
```

---

## 11.8 Edge Cases and Limitations

| Scenario | Handling |
|----------|----------|
| External crate function (`serde::to_string(&data)`) | Show as opaque: "→ enters serde::to_string() [external]" |
| Trait object (`&dyn Write`) | Show warning: "dynamic dispatch — target unknown" |
| Generic function (`fn foo<T: AsRef<[u8]>>(t: &T)`) | Resolve concrete type at call site if possible |
| Recursive call (`fn recurse(data: &[i32])`) | Detect cycle, show "⟳ recursive" marker, don't trace |
| Closure passed as argument (`iter.map(\|x\| ...)`) | Phase 2: trace into closure body |
| Async function (`async fn fetch(url: &str)`) | Treat like sync for borrow tracking (lifetime is pre-await) |
| Macro-generated calls (`println!("{}", data)`) | Expand macro, trace into expanded code |
| Very long chains (>5 levels) | Truncate with "... (2 more levels)" |
| Multiple borrows in same call (`foo(&a, &b)`) | Track each independently |
| Borrow returned from callee (`let r = get_ref(&data)`) | Show return path: callee → caller |

**Performance constraints:**
```rust
const MAX_DEPTH: usize = 3;           // default call chain depth
const MAX_ANALYSIS_TIME_MS: u64 = 200; // abort if too slow
const MAX_CROSS_BORROWS: usize = 50;   // limit results per file
```

---

## 11.9 Configuration

**VS Code settings:**
```json
{
  "borrowscope.crossFunction.enabled": true,
  "borrowscope.crossFunction.maxDepth": 3,
  "borrowscope.crossFunction.showInline": true,
  "borrowscope.crossFunction.showLinks": true,
  "borrowscope.crossFunction.splitView": false
}
```

**package.json contribution:**
```json
{
  "borrowscope.crossFunction.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show cross-function borrow tracking"
  },
  "borrowscope.crossFunction.maxDepth": {
    "type": "number",
    "default": 3,
    "minimum": 1,
    "maximum": 10,
    "description": "Maximum call chain depth to trace"
  }
}
```

---

## 11.10 Tests

### Server-side tests (Rust):

```rust
#[test]
#[ignore]
fn test_cross_function_direct_call() {
    // fn main() { process(&data); }
    // fn process(r: &[i32]) { ... }
    // Should produce one CrossFunctionBorrow with 2 path segments
}

#[test]
#[ignore]
fn test_cross_function_method_call() {
    // data.transform() where transform takes &self
    // Should trace into the method
}

#[test]
#[ignore]
fn test_cross_function_multiple_args() {
    // fn merge(a: &[i32], b: &[i32]) called with merge(&x, &y)
    // Should produce two separate CrossFunctionBorrows
}

#[test]
#[ignore]
fn test_cross_function_return_borrow() {
    // let r = get_first(&data);
    // fn get_first(s: &[i32]) -> &i32 { &s[0] }
    // Should show borrow flowing back to caller
}

#[test]
#[ignore]
fn test_cross_function_depth_limit() {
    // a() -> b() -> c() -> d() with max_depth=2
    // Should only trace a -> b -> c, not into d
}

#[test]
#[ignore]
fn test_cross_function_cycle_detection() {
    // fn a(r: &i32) { b(r); }
    // fn b(r: &i32) { a(r); }  // recursive
    // Should not infinite loop
}

#[test]
#[ignore]
fn test_cross_file_resolution() {
    // Call from main.rs to utils.rs
    // Path segments should have different file URIs
}

#[test]
#[ignore]
fn test_external_crate_boundary() {
    // println!("{}", data) — calls into std
    // Should show "external" marker, not crash
}
```

### Extension-side tests (TypeScript):

```typescript
describe("11. Cross-Function Borrow Tracking", () => {
  it("renders cross-function annotation on call line");
  it("shows hover with full borrow path");
  it("clickable link navigates to callee file");
  it("respects maxDepth setting");
  it("handles empty cross_borrows response");
  it("only renders segments in current file");
});
```

---

## 11.11 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        VS Code Extension                         │
├─────────────────────────────────────────────────────────────────┤
│  cross-lifelines.ts                                              │
│  ┌─────────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Inline Annotations│  │ DocumentLinks │  │ Split View (opt) │  │
│  │ "→ enters foo()" │  │ Click to jump │  │ Side-by-side     │  │
│  └────────┬─────────┘  └──────┬───────┘  └────────┬──────────┘  │
│           │                    │                    │             │
│           └────────────────────┼────────────────────┘             │
│                                │                                  │
│                    borrowscope/crossFunctionBorrows                │
│                                │                                  │
├────────────────────────────────┼──────────────────────────────────┤
│                        LSP Server                                 │
├────────────────────────────────┼──────────────────────────────────┤
│                                │                                  │
│  ┌─────────────────────────────▼──────────────────────────────┐  │
│  │           analyze_cross_function_borrows()                  │  │
│  │                                                             │  │
│  │  1. Find call expressions with reference arguments          │  │
│  │  2. Resolve call target (hir::Function)                     │  │
│  │  3. Get target's source file (FileId → VfsPath)             │  │
│  │  4. Analyze callee's parameter usage                        │  │
│  │  5. Recursively trace (up to max_depth)                     │  │
│  │  6. Build CrossFunctionBorrow path                          │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  Dependencies:                                                     │
│  - hir::Function::source() → FileId                               │
│  - sema.resolve_path() → PathResolution                           │
│  - sema.resolve_method_call() → Function                          │
│  - Vfs::file_path(FileId) → VfsPath                              │
│  - compute_borrow_scopes() (from Milestone 2)                     │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

---

**Priority:** After Milestone 10 (all current milestones complete)
**Estimated effort:** 3-4 weeks (Phase 1: 1 week, Phase 2: 1 week, Phase 3: 1-2 weeks)
**Dependencies:** Milestone 2 (borrow scope computation), Milestone 3 (LSP endpoints), Milestone 4 (VS Code extension)
