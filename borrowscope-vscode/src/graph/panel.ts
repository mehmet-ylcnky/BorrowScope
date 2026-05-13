import * as vscode from "vscode";

export class GraphPanel {
  public static currentPanel: GraphPanel | undefined;
  private static _context: vscode.ExtensionContext;
  private readonly _panel: vscode.WebviewPanel;
  private _disposables: vscode.Disposable[] = [];
  private _extensionUri: vscode.Uri;
  private _currentGraph: any | undefined;

  private static readonly STATE_KEY = "borrowscope.lastGraph";

  public static setContext(context: vscode.ExtensionContext): void {
    GraphPanel._context = context;
  }

  public static createOrShow(extensionUri: vscode.Uri, graph?: any): void {
    const column = vscode.ViewColumn.Beside;

    if (GraphPanel.currentPanel) {
      GraphPanel.currentPanel._panel.reveal(column);
      if (graph) GraphPanel.currentPanel.updateGraph(graph);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      "borrowscopeGraph",
      "BorrowScope: Ownership Graph",
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(extensionUri, "media")],
      }
    );

    GraphPanel.currentPanel = new GraphPanel(panel, extensionUri);

    if (graph) {
      GraphPanel.currentPanel.updateGraph(graph);
    } else {
      // Restore last saved state
      const saved = GraphPanel._context?.workspaceState.get<any>(GraphPanel.STATE_KEY);
      if (saved) {
        GraphPanel.currentPanel.updateGraph(saved);
      }
    }
  }

  public static getPanel(): GraphPanel | undefined {
    return GraphPanel.currentPanel;
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;
    this._panel.webview.html = this._buildHtml(undefined);
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
  }

  public updateGraph(graph: any): void {
    this._currentGraph = graph;
    this._panel.webview.html = this._buildHtml(graph);
    // Persist for next session
    GraphPanel._context?.workspaceState.update(GraphPanel.STATE_KEY, graph);
  }

  public getGraph(): any | undefined {
    return this._currentGraph;
  }

  public getPanel(): vscode.WebviewPanel {
    return this._panel;
  }

  public dispose(): void {
    GraphPanel.currentPanel = undefined;
    this._panel.dispose();
    this._disposables.forEach((d) => d.dispose());
    this._disposables = [];
  }

  private _buildHtml(graph: any | undefined): string {
    if (!graph) {
      return `<!DOCTYPE html><html><head><meta charset="UTF-8">
<style>body{font-family:var(--vscode-font-family);background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);padding:16px;}</style>
</head><body><h2>BorrowScope: Ownership Graph</h2><p>Waiting for data... Click a CodeLens to load a function.</p></body></html>`;
    }

    const vars = graph.variables || [];
    const scopes = graph.borrow_scopes || [];
    const moves = graph.moves || [];
    const clones = graph.rc_clones || [];
    const conflicts = graph.conflicts || [];

    let varsHtml = vars.map((v: any) =>
      `<tr><td><b>${esc(v.name)}</b></td><td><code>${esc(v.type_display)}</code></td><td>${esc(v.ownership_category)}</td></tr>`
    ).join("");

    let scopesHtml = scopes.map((s: any) =>
      `<tr><td>${esc(s.borrower_name)}</td><td>${s.is_mutable ? "&mut" : "&"}</td><td>${esc(s.target_name)}</td><td>${s.start_line}-${s.end_line}</td></tr>`
    ).join("");

    let movesHtml = moves.map((m: any) =>
      `<tr><td>${esc(m.source_name)}</td><td>↦</td><td>${esc(JSON.stringify(m.destination))}</td><td>${m.line}</td></tr>`
    ).join("");

    let clonesHtml = clones.map((c: any) =>
      `<tr><td>${esc(c.clone_variable)}</td><td>🔗</td><td>${esc(c.source_variable)}</td><td>${c.line}</td></tr>`
    ).join("");

    let conflictsHtml = conflicts.map((c: any) =>
      `<tr><td>⚠️ ${esc(c.borrow_a)}</td><td>&</td><td>${esc(c.borrow_b)}</td><td>${c.overlap_start_line}-${c.overlap_end_line}</td></tr>`
    ).join("");

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>BorrowScope: ${esc(graph.function_name)}</title>
  <style>
    body { margin:0; padding:16px; font-family:var(--vscode-font-family); background:var(--vscode-editor-background); color:var(--vscode-editor-foreground); font-size:13px; }
    h2 { margin:0 0 4px 0; font-size:15px; }
    .stats { opacity:0.7; margin-bottom:12px; }
    table { border-collapse:collapse; width:100%; margin-bottom:16px; }
    th,td { text-align:left; padding:4px 8px; border-bottom:1px solid var(--vscode-panel-border); }
    th { opacity:0.7; font-size:11px; text-transform:uppercase; }
    .section { margin-bottom:16px; }
    .section-title { font-weight:bold; margin-bottom:4px; }
    code { background:var(--vscode-textCodeBlock-background); padding:1px 4px; border-radius:3px; }
    .cat-Owned { color:#2ecc71; } .cat-SharedRef { color:#3498db; } .cat-MutableRef { color:#e74c3c; }
    .cat-Rc, .cat-Arc { color:#9b59b6; } .cat-InteriorMut { color:#e67e22; } .cat-Copy { color:#95a5a6; }
    .cat-RawPointer { color:#7f8c8d; }
  </style>
</head>
<body>
  <h2>📊 ${esc(graph.function_name)}</h2>
  <div class="stats">${vars.length} variables, ${scopes.length} borrows, ${moves.length} moves${conflicts.length > 0 ? `, ⚠️ ${conflicts.length} conflicts` : ""}</div>

  ${vars.length > 0 ? `<div class="section"><div class="section-title">Variables</div>
  <table><tr><th>Name</th><th>Type</th><th>Category</th></tr>${varsHtml}</table></div>` : ""}

  ${scopes.length > 0 ? `<div class="section"><div class="section-title">Borrow Scopes</div>
  <table><tr><th>Borrower</th><th>Kind</th><th>Target</th><th>Lines</th></tr>${scopesHtml}</table></div>` : ""}

  ${moves.length > 0 ? `<div class="section"><div class="section-title">Moves</div>
  <table><tr><th>Source</th><th></th><th>Destination</th><th>Line</th></tr>${movesHtml}</table></div>` : ""}

  ${clones.length > 0 ? `<div class="section"><div class="section-title">Rc/Arc Clones</div>
  <table><tr><th>Clone</th><th></th><th>Source</th><th>Line</th></tr>${clonesHtml}</table></div>` : ""}

  ${conflicts.length > 0 ? `<div class="section"><div class="section-title">⚠️ Conflicts</div>
  <table><tr><th>Borrow A</th><th></th><th>Borrow B</th><th>Lines</th></tr>${conflictsHtml}</table></div>` : ""}

  <div id="graph-container" style="width:100%;height:calc(100vh - 300px);border:1px solid var(--vscode-panel-border);border-radius:4px;display:flex;align-items:center;justify-content:center;opacity:0.5;">
    D3.js graph will render here (Step 5.3)
  </div>
</body>
</html>`;
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
