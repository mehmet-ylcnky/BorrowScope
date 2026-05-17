import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { exec } from "child_process";

export interface PrerequisiteStatus {
  rustToolchain: boolean;
  rustVersion: string;
  cargoProject: boolean;
  projectName: string;
  serverBinary: boolean;
}

/** Show welcome panel on first activation */
export function showWelcomeIfNeeded(context: vscode.ExtensionContext): void {
  const hasShown = context.globalState.get("borrowscope.welcomeShown", false);
  if (hasShown) return;

  showWelcomePanel(context);
  context.globalState.update("borrowscope.welcomeShown", true);
}

/** Show the welcome panel (can be triggered manually) */
export function showWelcomePanel(context: vscode.ExtensionContext): void {
  const panel = vscode.window.createWebviewPanel(
    "borrowscopeWelcome",
    "Welcome to BorrowScope",
    vscode.ViewColumn.One,
    { enableScripts: true }
  );

  checkPrerequisites().then((status) => {
    panel.webview.html = getWelcomeHtml(status, context);
  });

  panel.webview.onDidReceiveMessage((msg) => {
    if (msg.command === "openRustFile") {
      vscode.commands.executeCommand("workbench.action.quickOpen", "*.rs");
    } else if (msg.command === "showGraph") {
      vscode.commands.executeCommand("borrowscope.showGraph");
    } else if (msg.command === "openDocs") {
      vscode.env.openExternal(vscode.Uri.parse("https://github.com/mehmet-ylcnky/BorrowScope"));
    } else if (msg.command === "dismiss") {
      panel.dispose();
    }
  });
}

/** Check if prerequisites are met */
export async function checkPrerequisites(): Promise<PrerequisiteStatus> {
  const rustVersion = await getCommandOutput("rustc --version");
  const hasRust = rustVersion.length > 0;

  const wsRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || "";
  const hasCargoToml = wsRoot ? fs.existsSync(path.join(wsRoot, "Cargo.toml")) : false;
  const projectName = hasCargoToml ? path.basename(wsRoot) : "";

  const serverPath = vscode.workspace.getConfiguration("borrowscope").get<string>("server.path", "");
  const hasServer = serverPath ? fs.existsSync(serverPath) : false;

  return {
    rustToolchain: hasRust,
    rustVersion: rustVersion.trim(),
    cargoProject: hasCargoToml,
    projectName,
    serverBinary: hasServer,
  };
}

function getCommandOutput(cmd: string): Promise<string> {
  return new Promise((resolve) => {
    exec(cmd, (err, stdout) => {
      resolve(err ? "" : stdout);
    });
  });
}

function getWelcomeHtml(status: PrerequisiteStatus, context: vscode.ExtensionContext): string {
  const check = (ok: boolean) => ok ? "✅" : "❌";

  return `<!DOCTYPE html>
<html><head><style>
  body { font-family: var(--vscode-font-family, sans-serif); background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); padding: 30px; max-width: 700px; margin: 0 auto; }
  h1 { color: var(--vscode-textLink-foreground, #58a6ff); }
  .status { margin: 20px 0; padding: 16px; background: var(--vscode-textBlockQuote-background, #2d2d2d); border-radius: 6px; }
  .status-item { margin: 8px 0; font-size: 14px; }
  .steps { margin: 20px 0; }
  .steps li { margin: 8px 0; font-size: 14px; }
  .actions { margin-top: 24px; display: flex; gap: 10px; }
  .actions button { padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
  .btn-primary { background: var(--vscode-button-background, #0e639c); color: var(--vscode-button-foreground, #fff); }
  .btn-secondary { background: var(--vscode-button-secondaryBackground, #3a3d41); color: var(--vscode-button-secondaryForeground, #fff); }
  .btn-primary:hover { opacity: 0.9; }
  code { background: var(--vscode-textCodeBlock-background, #1e1e1e); padding: 2px 6px; border-radius: 3px; font-size: 13px; }
  .shortcut { color: var(--vscode-textLink-foreground, #58a6ff); }
  .warning { color: #ffa657; }
</style></head><body>
  <h1>🔍 Welcome to BorrowScope</h1>
  <p>BorrowScope visualizes Rust's ownership and borrowing system in real-time, directly in your editor.</p>

  <div class="status">
    <div class="status-item">${check(status.rustToolchain)} Rust toolchain ${status.rustVersion ? "(" + status.rustVersion.split(" ")[1] + ")" : '<span class="warning">— not found. Install from rustup.rs</span>'}</div>
    <div class="status-item">${check(status.cargoProject)} Cargo project ${status.projectName ? "(" + status.projectName + ")" : '<span class="warning">— open a folder with Cargo.toml</span>'}</div>
    <div class="status-item">${check(status.serverBinary)} BorrowScope server ${status.serverBinary ? "ready" : '<span class="warning">— set <code>borrowscope.server.path</code> in settings</span>'}</div>
  </div>

  <div class="steps">
    <h3>Getting Started</h3>
    <ol>
      <li>Open any <code>.rs</code> file in a Cargo project</li>
      <li>Look for colored hints next to variables: <code>[&]</code> <code>[&mut]</code> <code>[Rc]</code></li>
      <li>Click the <b>▸ vars, borrows, moves</b> CodeLens above functions</li>
      <li>Press <span class="shortcut">Ctrl+Shift+O</span> to open the ownership graph</li>
    </ol>
  </div>

  <div class="steps">
    <h3>Keyboard Shortcuts</h3>
    <ul>
      <li><span class="shortcut">Ctrl+Shift+O</span> — Show Ownership Graph</li>
      <li><span class="shortcut">Ctrl+Shift+I</span> — Inspect Variable at Cursor</li>
      <li><span class="shortcut">Ctrl+Shift+D</span> — Toggle Decorations</li>
      <li><span class="shortcut">Alt+Shift+N/P</span> — Next/Previous Conflict</li>
    </ul>
  </div>

  <div class="actions">
    <button class="btn-primary" onclick="post('openRustFile')">Open a Rust File</button>
    <button class="btn-secondary" onclick="post('showGraph')">Show Graph</button>
    <button class="btn-secondary" onclick="post('openDocs')">Documentation</button>
    <button class="btn-secondary" onclick="post('dismiss')">Dismiss</button>
  </div>

  <script>
    const vscode = acquireVsCodeApi();
    function post(cmd) { vscode.postMessage({ command: cmd }); }
  </script>
</body></html>`;
}
