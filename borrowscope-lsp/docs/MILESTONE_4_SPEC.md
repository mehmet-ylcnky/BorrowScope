# Milestone 4: VS Code Extension Frontend - Detailed Specification

## 4.1 Extension Project Setup

**Objective:** Create the TypeScript VS Code extension that connects to the BorrowScope language server. The extension is the UI layer: it starts the server binary, communicates via LSP, and renders visualizations in the editor.

**Steps:**
1. Initialize project with `yo code` (TypeScript extension template)
2. Configure `package.json` with activation events, commands, configuration
3. Add `vscode-languageclient` dependency for LSP communication
4. Create `src/extension.ts` entry point
5. Set up build pipeline (esbuild for fast bundling)

**Project structure:**
```
borrowscope-vscode/
├── package.json            # Extension manifest
├── tsconfig.json           # TypeScript config
├── esbuild.js             # Build script
├── src/
│   ├── extension.ts       # Activation, deactivation, command registration
│   ├── client.ts          # Language client (connects to borrowscope-lsp)
│   ├── decorations.ts     # Inline editor decorations
│   ├── graph/
│   │   ├── panel.ts       # WebView panel lifecycle
│   │   ├── renderer.ts    # D3.js graph rendering
│   │   └── messages.ts    # WebView ↔ extension messaging
│   ├── commands.ts        # Command palette commands
│   └── config.ts          # Extension settings
├── media/
│   ├── graph.html         # WebView HTML template
│   ├── graph.css          # WebView styles
│   └── graph.js           # D3.js rendering code (runs in WebView)
└── test/
    ├── suite/
    │   └── extension.test.ts
    └── runTest.ts
```

**package.json (key sections):**
```json
{
  "name": "borrowscope",
  "displayName": "BorrowScope",
  "description": "Real-time ownership visualization for Rust",
  "version": "0.1.0",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Programming Languages", "Visualization"],
  "activationEvents": ["onLanguage:rust"],
  "main": "./out/extension.js",
  "contributes": {
    "commands": [
      { "command": "borrowscope.showGraph", "title": "BorrowScope: Show Ownership Graph" },
      { "command": "borrowscope.inspectVariable", "title": "BorrowScope: Inspect Variable" },
      { "command": "borrowscope.toggleDecorations", "title": "BorrowScope: Toggle Decorations" },
      { "command": "borrowscope.restartServer", "title": "BorrowScope: Restart Server" }
    ],
    "configuration": {
      "title": "BorrowScope",
      "properties": {
        "borrowscope.server.path": {
          "type": "string",
          "default": "",
          "description": "Path to borrowscope-lsp binary (auto-detected if empty)"
        },
        "borrowscope.decorations.enabled": {
          "type": "boolean",
          "default": true,
          "description": "Show inline ownership decorations"
        },
        "borrowscope.decorations.borrowScopes": {
          "type": "boolean",
          "default": true,
          "description": "Highlight borrow scope regions"
        },
        "borrowscope.graph.layout": {
          "type": "string",
          "enum": ["force", "hierarchical"],
          "default": "hierarchical",
          "description": "Graph layout algorithm"
        }
      }
    }
  }
}
```

**Expectation:** Extension activates when a Rust file is opened. It finds and starts the `borrowscope-lsp` binary, establishes LSP connection, and is ready to serve commands.

**Tests for 4.1:**
- Extension activates on opening a `.rs` file
- Extension does NOT activate for non-Rust files
- `package.json` passes VS Code extension validation (`vsce ls`)
- All declared commands are registered
- Configuration properties have correct types and defaults

---

## 4.2 Language Client Configuration

**Objective:** Configure the `vscode-languageclient` to connect to the `borrowscope-lsp` binary. The client manages the server process lifecycle (start, restart, stop) and routes LSP messages between VS Code and the server.

**Code (client.ts):**
```typescript
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function startClient(context: vscode.ExtensionContext): LanguageClient {
    const serverPath = getServerPath(context);

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio,
                 options: { env: { RUST_LOG: "borrowscope_lsp=debug" } } },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rust' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.rs'),
        },
        initializationOptions: {
            // Pass extension settings to server
            decorations: vscode.workspace.getConfiguration('borrowscope.decorations'),
        },
    };

    client = new LanguageClient(
        'borrowscope',
        'BorrowScope Language Server',
        serverOptions,
        clientOptions,
    );

    client.start();
    return client;
}

function getServerPath(context: vscode.ExtensionContext): string {
    // 1. Check user setting
    const configured = vscode.workspace.getConfiguration('borrowscope.server').get<string>('path');
    if (configured && fs.existsSync(configured)) return configured;

    // 2. Check bundled binary
    const bundled = path.join(context.extensionPath, 'server', 'borrowscope-lsp');
    if (fs.existsSync(bundled)) return bundled;

    // 3. Check PATH
    const inPath = which.sync('borrowscope-lsp', { nothrow: true });
    if (inPath) return inPath;

    throw new Error('borrowscope-lsp binary not found. Install it or set borrowscope.server.path.');
}
```

**Server discovery order:**
```
1. User-configured path (borrowscope.server.path setting)
       │ not set
       ▼
2. Bundled binary (extension/server/borrowscope-lsp)
       │ not found
       ▼
3. System PATH (which borrowscope-lsp)
       │ not found
       ▼
4. Error: prompt user to install
```

**Expectation:** The client starts the server, completes the LSP handshake, and begins receiving diagnostics and notifications. If the server crashes, the client shows an error and offers to restart.

**Tests for 4.2:**
- Client starts server from bundled path
- Client starts server from user-configured path
- Client shows error if server binary not found
- Client reconnects after server crash (with user prompt)
- Client passes initialization options to server
- Client stops server on extension deactivation

---

## 4.3 Server Binary Management

**Objective:** Handle downloading, updating, and managing the server binary across platforms. The extension should work out-of-the-box without requiring the user to manually install the server.

**Steps:**
1. On first activation, check if server binary exists
2. If not, download from GitHub releases (platform-specific)
3. Verify binary integrity (checksum)
4. On extension update, check if server needs updating
5. Store binary in extension's global storage path

**Code (server-manager.ts):**
```typescript
interface ServerBinary {
    path: string;
    version: string;
}

async function ensureServer(context: vscode.ExtensionContext): Promise<ServerBinary> {
    const storagePath = context.globalStorageUri.fsPath;
    const binaryName = process.platform === 'win32' ? 'borrowscope-lsp.exe' : 'borrowscope-lsp';
    const binaryPath = path.join(storagePath, binaryName);

    if (fs.existsSync(binaryPath)) {
        const localVersion = await getLocalVersion(binaryPath);
        const latestVersion = await getLatestRelease();
        if (localVersion === latestVersion) {
            return { path: binaryPath, version: localVersion };
        }
    }

    // Download latest
    await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: 'BorrowScope' },
        async (progress) => {
            progress.report({ message: 'Downloading server...' });
            await downloadServer(storagePath, binaryName);
            progress.report({ message: 'Ready!' });
        }
    );

    return { path: binaryPath, version: await getLocalVersion(binaryPath) };
}

function getPlatformAsset(): string {
    const platform = process.platform;  // 'linux', 'darwin', 'win32'
    const arch = process.arch;          // 'x64', 'arm64'
    return `borrowscope-lsp-${platform}-${arch}`;
}
```

**Expectation:** First-time users see "Downloading server..." notification, then the extension works. Subsequent launches use the cached binary. Updates happen silently in the background.

**Tests for 4.3:**
- Binary downloads on first activation (mock HTTP)
- Cached binary is reused on subsequent activations
- Update downloads new version when available
- Correct platform binary is selected (linux-x64, darwin-arm64, etc.)
- Download failure shows user-friendly error message
- Binary is executable after download (chmod on Unix)

---

## 4.4 Inline Decorations from Inlay Hints

**Objective:** Render the ownership category annotations (`[Rc]`, `[&mut]`, `[&]`) inline in the editor using VS Code's inlay hints API. These come from the server's `textDocument/inlayHints` response.

**Code (decorations.ts):**
```typescript
// Inlay hints are handled automatically by vscode-languageclient
// when the server declares inlayHintProvider capability.
// No client-side code needed for basic rendering.

// However, for custom styling we use decoration types:
const borrowScopeDecorationType = vscode.window.createTextEditorDecorationType({
    after: {
        margin: '0 0 0 0.5em',
        fontStyle: 'italic',
        fontSize: '0.85em',
    }
});

// For ownership category badges with colors:
const rcDecoration = vscode.window.createTextEditorDecorationType({
    after: { contentText: ' [Rc]', color: '#9b59b6', fontStyle: 'italic', fontSize: '0.85em' }
});
const mutRefDecoration = vscode.window.createTextEditorDecorationType({
    after: { contentText: ' [&mut]', color: '#e74c3c', fontStyle: 'italic', fontSize: '0.85em' }
});
const sharedRefDecoration = vscode.window.createTextEditorDecorationType({
    after: { contentText: ' [&]', color: '#3498db', fontStyle: 'italic', fontSize: '0.85em' }
});
```

**Appearance:**
```rust
fn example() {
    let data = vec![1, 2, 3];
    let r = &data;                    [&]
    let rc = Rc::new(42);             [Rc]
    let guard = cell.borrow_mut();    [&mut Cell]
    let ptr = &data as *const _;      [*ptr]
}
```

**Expectation:** Ownership annotations appear inline, color-coded by category. They update live as the user types (server pushes new inlay hints after re-analysis).

**Tests for 4.4:**
- Inlay hints render for Rc/Arc variables
- Inlay hints render for reference variables
- No hints for plain owned types (Vec, String)
- Hints update after file edit
- Hints disappear when decorations are disabled in settings
- Hints respect VS Code theme colors

---

## 4.5 Gutter Icons and Lifecycle Flow Lines

**Objective:** Show ownership lifecycle as colored vertical lines in the gutter/left border, plus small icons for key events. Each borrow phase gets a distinct colored line showing its active duration — similar to git branch flow visualization.

**Lifecycle Phase Colors:**
- 🟢 Green line — owner alive (from creation to drop/move)
- 🔵 Blue line — shared borrow active (from `&x` to last use)
- 🔴 Red line — mutable borrow active (from `&mut x` to last use)
- 🟣 Purple line — Rc/Arc clone alive
- ⚠️ Yellow overlap — conflict zone (two incompatible borrows overlap)

**Gutter Icons (at key events):**
- ● Blue dot — shared borrow starts
- ● Red dot — mutable borrow starts
- ○ Circle — borrow ends (last use)
- → Arrow — move happened
- ✕ Red X — drop

**Code:**
```typescript
// Lifecycle flow lines using borderLeft on whole lines
const ownerLifeline = vscode.window.createTextEditorDecorationType({
    borderLeft: '3px solid rgba(46, 204, 113, 0.5)',  // green
    isWholeLine: true,
});
const sharedBorrowLifeline = vscode.window.createTextEditorDecorationType({
    borderLeft: '3px solid rgba(52, 152, 219, 0.6)',  // blue
    isWholeLine: true,
});
const mutBorrowLifeline = vscode.window.createTextEditorDecorationType({
    borderLeft: '3px solid rgba(231, 76, 60, 0.6)',   // red
    isWholeLine: true,
});
const rcLifeline = vscode.window.createTextEditorDecorationType({
    borderLeft: '3px solid rgba(155, 89, 182, 0.5)',  // purple
    isWholeLine: true,
});
const conflictLifeline = vscode.window.createTextEditorDecorationType({
    borderLeft: '3px solid rgba(241, 196, 15, 0.8)',  // yellow warning
    isWholeLine: true,
});

// Gutter icons for key events
const borrowStartIcon = vscode.window.createTextEditorDecorationType({
    gutterIconPath: context.asAbsolutePath('media/icons/borrow-start.svg'),
    gutterIconSize: '80%',
});
const borrowEndIcon = vscode.window.createTextEditorDecorationType({
    gutterIconPath: context.asAbsolutePath('media/icons/borrow-end.svg'),
    gutterIconSize: '80%',
});
const moveIcon = vscode.window.createTextEditorDecorationType({
    gutterIconPath: context.asAbsolutePath('media/icons/move.svg'),
    gutterIconSize: '80%',
});
```

**Appearance:**
```rust
fn example() {
  ┃ green   let data = vec![1, 2, 3];     // owner alive
  ┃ green
  ┃ ┃ blue  let r = &data;               // shared borrow starts
  ┃ ┃ blue  println!("{}", r);            // borrow active
  ┃ ┃ blue  use_ref(r);                   // last use → borrow ends
  ┃
  ┃ ┃ red   let m = &mut data;           // mutable borrow starts
  ┃ ┃ red   m.push(4);                   // last use → borrow ends
  ┃
  ┃         drop(data);                    // owner dropped → green ends
```

**Expectation:** Developers can visually trace borrow lifecycles like git branch flows. Nested borrows show as nested colored lines. Conflicts are immediately visible as yellow lines.

**Tests for 4.5:**
- Owner lifeline (green) spans from variable creation to drop/move
- Shared borrow lifeline (blue) spans from borrow to last use
- Mutable borrow lifeline (red) spans from borrow to last use
- Rc/Arc lifeline (purple) spans clone lifetime
- Conflict zone (yellow) shown when borrows overlap
- Gutter icons appear at borrow start/end/move events
- Hover on gutter icon shows variable names and relationship
- Lines update after file edit
- Lines disappear when disabled in settings
- Nested borrows show nested colored lines

---

## 4.6 Borrow Scope Background Highlighting

**Objective:** In addition to the left-border lifeline, optionally highlight the active region of each borrow with a subtle colored background. This provides a secondary visual cue that complements the lifeline. Can be toggled independently via `borrowscope.decorations.borrowScopes` setting.

**Code:**
```typescript
const sharedBorrowHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(52, 152, 219, 0.06)',  // very subtle blue
    isWholeLine: true,
});

const mutBorrowHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(231, 76, 60, 0.06)',   // very subtle red
    isWholeLine: true,
});

function updateBorrowHighlights(editor: vscode.TextEditor, scopes: BorrowScopeRange[]) {
    const shared: vscode.DecorationOptions[] = [];
    const mutable: vscode.DecorationOptions[] = [];

    for (const scope of scopes) {
        const range = new vscode.Range(
            scope.range.start.line, 0,
            scope.range.end.line, Number.MAX_SAFE_INTEGER
        );
        const decoration = {
            range,
            hoverMessage: `${scope.is_mutable ? '&mut' : '&'} borrow of \`${scope.target}\` by \`${scope.borrower}\``,
        };

        if (scope.is_mutable) {
            mutable.push(decoration);
        } else {
            shared.push(decoration);
        }
    }

    editor.setDecorations(sharedBorrowHighlight, shared);
    editor.setDecorations(mutBorrowHighlight, mutable);
}
```

**Expectation:** Background highlighting is a subtle secondary cue. The primary visual is the colored lifeline (4.5). Background can be toggled off for users who find it distracting.

**Tests for 4.6:**
- Shared borrow region gets blue background
- Mutable borrow region gets red background
- Highlighting spans from borrow creation to last use
- Multiple borrows each get their own highlight
- Highlights update after file edit
- Highlights respect dark/light theme (opacity adjusts)
- Highlights can be toggled off via borrowscope.decorations.borrowScopes setting

---

## 4.7 Diagnostics Display

**Objective:** Display borrow conflict diagnostics from the server in VS Code's Problems panel. The diagnostics are informational (not errors) and include related locations showing both conflicting borrows.

**Implementation:** This is handled automatically by `vscode-languageclient`. When the server sends `textDocument/publishDiagnostics`, the client displays them in the Problems panel. No custom client code needed for basic display.

**Custom enhancement - inline conflict markers:**
```typescript
// In addition to Problems panel, show inline squiggles
client.onNotification('textDocument/publishDiagnostics', (params) => {
    const borrowScopeDiagnostics = params.diagnostics.filter(
        d => d.source === 'BorrowScope'
    );

    // Add custom decorations for conflicts (wavy underline in orange)
    if (borrowScopeDiagnostics.length > 0) {
        updateConflictDecorations(editor, borrowScopeDiagnostics);
    }
});
```

**Expectation:** Conflicts appear in the Problems panel with clickable locations. The editor shows a subtle indicator (not as aggressive as a compiler error) at the conflict site.

**Tests for 4.7:**
- Diagnostics from server appear in Problems panel
- Clicking a diagnostic navigates to the correct line
- Related information shows both borrow locations
- Diagnostics clear when the file is edited to resolve the conflict
- Diagnostics don't duplicate rust-analyzer's borrow checker errors
- Severity is Information (blue icon, not red error)

---

## 4.8 CodeLens Rendering

**Objective:** Display ownership statistics above each function as clickable CodeLens items. Clicking a CodeLens opens the ownership graph panel for that function.

**Implementation:** CodeLens is handled by `vscode-languageclient` automatically when the server declares `codeLensProvider` capability. The client renders them and handles the command execution.

**Code (commands.ts):**
```typescript
// Register the command that CodeLens triggers
vscode.commands.registerCommand('borrowscope.showGraph', async (uri: string) => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;

    // Request ownership graph from server
    const position = editor.selection.active;
    const graph = await client.sendRequest('borrowscope/ownershipGraph', {
        textDocument: { uri },
        position: { line: position.line, character: position.character },
    });

    // Open or update the graph panel
    GraphPanel.createOrShow(context.extensionUri, graph);
});
```

**Expectation:** Each function shows a clickable summary line. Clicking it opens the graph panel focused on that function.

**Tests for 4.8:**
- CodeLens appears above each function
- CodeLens shows correct variable/borrow/move counts
- Clicking CodeLens opens graph panel
- CodeLens updates after file edit
- CodeLens disappears for deleted functions
- No CodeLens for non-function items (structs, impls)

---

## 4.T Integration Test Suite

**Test approach:** Use `@vscode/test-electron` to run tests inside a real VS Code instance with the extension loaded.

```typescript
import * as vscode from 'vscode';
import * as assert from 'assert';

suite('Extension Frontend Tests', () => {
    suiteSetup(async () => {
        // Wait for extension to activate
        const ext = vscode.extensions.getExtension('borrowscope.borrowscope');
        await ext?.activate();
    });

    test('Extension activates on Rust file', async () => {
        const doc = await vscode.workspace.openTextDocument({ language: 'rust', content: 'fn main() {}' });
        await vscode.window.showTextDocument(doc);
        // Extension should be active
        const ext = vscode.extensions.getExtension('borrowscope.borrowscope');
        assert.strictEqual(ext?.isActive, true);
    });

    test('Server connects and initializes', async () => {
        // Check that the language client is running
        // (verify via status bar item or output channel)
    });

    test('Decorations appear for borrow patterns', async () => {
        const content = `fn main() {\n    let x = vec![1];\n    let r = &x;\n    println!("{}", r);\n}`;
        const doc = await vscode.workspace.openTextDocument({ language: 'rust', content });
        const editor = await vscode.window.showTextDocument(doc);

        // Wait for server analysis
        await sleep(2000);

        // Verify decorations exist (check via decoration provider)
    });

    test('Show Graph command opens panel', async () => {
        await vscode.commands.executeCommand('borrowscope.showGraph');
        // Verify WebView panel is visible
    });
});
```
