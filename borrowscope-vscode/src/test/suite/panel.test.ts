import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("5.1 WebView Panel Registration and Lifecycle", () => {
  let GraphPanel: any;
  let vscode: any;

  before(() => {
    vscode = require("vscode");
    const mod = require(path.join(ROOT, "out", "graph", "panel.js"));
    GraphPanel = mod.GraphPanel;
  });

  afterEach(() => {
    if (GraphPanel.currentPanel) {
      GraphPanel.currentPanel.dispose();
    }
    GraphPanel.currentPanel = undefined;
  });

  // 1. createOrShow creates a new panel when none exists
  it("createOrShow creates panel when none exists", () => {
    assert.strictEqual(GraphPanel.currentPanel, undefined);
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    assert.ok(GraphPanel.currentPanel, "Panel should be created");
  });

  // 2. createOrShow reuses existing panel
  it("createOrShow reuses existing panel", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const first = GraphPanel.currentPanel;
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    assert.strictEqual(GraphPanel.currentPanel, first, "Should reuse same panel");
  });

  // 3. Panel has correct title
  it("panel has correct title", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.strictEqual(panel.title, "BorrowScope: Ownership Graph");
  });

  // 4. Panel opens in Beside column
  it("panel opens in Beside column", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.strictEqual(panel.viewColumn, -2); // ViewColumn.Beside
  });

  // 5. Panel has enableScripts true
  it("panel has scripts enabled", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.strictEqual(panel.options.enableScripts, true);
  });

  // 6. Panel has retainContextWhenHidden true
  it("panel retains context when hidden", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.strictEqual(panel.options.retainContextWhenHidden, true);
  });

  // 7. Panel has localResourceRoots set to media/
  it("panel has localResourceRoots for media", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.ok(panel.options.localResourceRoots);
    assert.ok(panel.options.localResourceRoots[0].fsPath.includes("media"));
  });

  // 8. dispose clears currentPanel
  it("dispose clears currentPanel", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    assert.ok(GraphPanel.currentPanel);
    GraphPanel.currentPanel.dispose();
    assert.strictEqual(GraphPanel.currentPanel, undefined);
  });

  // 9. updateGraph sends postMessage to webview
  it("updateGraph sends postMessage", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const graph = { function_name: "main", variables: [{ name: "x" }] };
    GraphPanel.currentPanel.updateGraph(graph);
    const panel = GraphPanel.currentPanel.getPanel();
    assert.deepStrictEqual(panel.webview._lastMessage, { type: "updateGraph", data: graph });
  });

  // 10. updateGraph stores graph for later retrieval
  it("updateGraph stores current graph", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const graph = { function_name: "test", variables: [] };
    GraphPanel.currentPanel.updateGraph(graph);
    assert.deepStrictEqual(GraphPanel.currentPanel.getGraph(), graph);
  });

  // 11. createOrShow with graph updates immediately
  it("createOrShow with graph data updates panel", () => {
    const graph = { function_name: "foo", variables: [{ name: "a" }] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    assert.deepStrictEqual(GraphPanel.currentPanel.getGraph(), graph);
  });

  // 12. HTML content includes Content-Security-Policy
  it("HTML includes Content-Security-Policy", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("Content-Security-Policy"));
  });

  // 13. HTML includes nonce for script
  it("HTML includes nonce for script security", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("nonce-"));
    assert.ok(html.includes("script-src 'nonce-"));
  });

  // 14. HTML includes graph-container div
  it("HTML includes graph container", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("graph-container"));
  });

  // 15. HTML includes acquireVscodeApi
  it("HTML includes vscode API acquisition", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("acquireVscodeApi"));
  });

  // 16. HTML includes message listener for updateGraph
  it("HTML listens for updateGraph messages", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("updateGraph"));
  });

  // 17. HTML posts ready message on load
  it("HTML posts ready message", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("postMessage"));
    assert.ok(html.includes("'ready'"));
  });

  // 18. HTML uses VS Code theme variables
  it("HTML uses VS Code theme CSS variables", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("--vscode-editor-background"));
    assert.ok(html.includes("--vscode-editor-foreground"));
  });

  // 19. getPanel static method returns current panel
  it("getPanel returns current panel or undefined", () => {
    assert.strictEqual(GraphPanel.getPanel(), undefined);
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    assert.ok(GraphPanel.getPanel());
  });

  // 20. Panel viewType is borrowscopeGraph
  it("panel viewType is borrowscopeGraph", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const panel = GraphPanel.currentPanel.getPanel();
    assert.strictEqual(panel.viewType, "borrowscopeGraph");
  });
});
