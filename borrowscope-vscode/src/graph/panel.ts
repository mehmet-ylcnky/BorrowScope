import * as vscode from "vscode";

export class GraphPanel {
  public static currentPanel: GraphPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private _disposables: vscode.Disposable[] = [];
  private _extensionUri: vscode.Uri;
  private _currentGraph: any | undefined;

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
    if (graph) GraphPanel.currentPanel.updateGraph(graph);
  }

  public static getPanel(): GraphPanel | undefined {
    return GraphPanel.currentPanel;
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;
    this._panel.webview.html = this._getHtmlContent();

    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    this._panel.webview.onDidReceiveMessage(
      (message) => this._handleMessage(message),
      null,
      this._disposables
    );
  }

  public updateGraph(graph: any): void {
    this._currentGraph = graph;
    this._panel.webview.postMessage({ type: "updateGraph", data: graph });
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

  private _handleMessage(message: any): void {
    switch (message.type) {
      case "nodeClicked":
        // Navigate to source line
        if (message.file && message.line) {
          const uri = vscode.Uri.parse(message.file);
          vscode.workspace.openTextDocument(uri).then((doc) => {
            vscode.window.showTextDocument(doc).then((editor) => {
              const pos = new vscode.Position(message.line - 1, 0);
              editor.selection = new vscode.Selection(pos, pos);
              editor.revealRange(
                new vscode.Range(pos, pos),
                vscode.TextEditorRevealType.InCenter
              );
            });
          });
        }
        break;
      case "ready":
        // WebView loaded, send current graph if available
        if (this._currentGraph) {
          this.updateGraph(this._currentGraph);
        }
        break;
    }
  }

  private _getHtmlContent(): string {
    const nonce = getNonce();
    const csp = `default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline';`;

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="${csp}">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>BorrowScope: Ownership Graph</title>
  <style>
    body { margin: 0; padding: 16px; font-family: var(--vscode-font-family); background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); }
    #graph-container { width: 100%; height: calc(100vh - 80px); border: 1px solid var(--vscode-panel-border); border-radius: 4px; }
    #status { padding: 8px 0; font-size: 12px; opacity: 0.7; }
    h2 { margin: 0 0 8px 0; font-size: 14px; }
  </style>
</head>
<body>
  <h2 id="title">BorrowScope: Ownership Graph</h2>
  <div id="status">Waiting for data...</div>
  <div id="graph-container"></div>
  <script nonce="${nonce}">
    const vscode = acquireVscodeApi();
    const state = vscode.getState() || {};

    window.addEventListener('message', event => {
      const message = event.data;
      if (message.type === 'updateGraph') {
        document.getElementById('status').textContent =
          'Function: ' + (message.data.function_name || 'unknown') +
          ' (' + (message.data.variables || []).length + ' variables)';
        vscode.setState({ graph: message.data });
        // D3.js rendering will be added in step 5.3
      }
    });

    // Notify extension that WebView is ready
    vscode.postMessage({ type: 'ready' });

    // Restore state if available
    if (state.graph) {
      document.getElementById('status').textContent =
        'Function: ' + state.graph.function_name +
        ' (' + (state.graph.variables || []).length + ' variables)';
    }
  </script>
</body>
</html>`;
  }
}

function getNonce(): string {
  let text = "";
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return text;
}
