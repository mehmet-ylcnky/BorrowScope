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

  // 9. updateGraph re-renders HTML with data
  it("updateGraph sets HTML with graph data", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const graph = { function_name: "main", variables: [{ name: "x", type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], moves: [], rc_clones: [], conflicts: [] };
    GraphPanel.currentPanel.updateGraph(graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("main"), "HTML should contain function name");
    assert.ok(html.includes("i32"), "HTML should contain type");
  });

  // 10. updateGraph stores current graph
  it("updateGraph stores current graph", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const graph = { function_name: "test", variables: [] };
    GraphPanel.currentPanel.updateGraph(graph);
    assert.deepStrictEqual(GraphPanel.currentPanel.getGraph(), graph);
  });

  // 11. createOrShow with graph updates immediately
  it("createOrShow with graph data updates panel", () => {
    const graph = { function_name: "foo", variables: [{ name: "a", type_display: "Vec<i32>", ownership_category: "Owned" }], borrow_scopes: [], moves: [], rc_clones: [], conflicts: [] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("foo"), "Should show function name");
    assert.ok(html.includes("Vec"), "Should show type");
  });

  // 12. HTML includes graph-container div
  it("HTML includes graph container", () => {
    const graph = { function_name: "f", variables: [], borrow_scopes: [], moves: [], rc_clones: [], conflicts: [] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("graph-container"));
  });

  // 13. HTML uses VS Code theme variables
  it("HTML uses VS Code theme CSS variables", () => {
    GraphPanel.createOrShow(vscode.Uri.file("/ext"));
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("--vscode-editor-background"));
    assert.ok(html.includes("--vscode-editor-foreground"));
  });

  // 14. HTML shows borrow scopes in graph
  it("HTML shows borrow scopes", () => {
    const graph = { function_name: "f", variables: [{ name: "data", type_display: "Vec<i32>", ownership_category: "Owned", line: 2, is_copy: false }], borrow_scopes: [{ borrower_name: "r", target_name: "data", is_mutable: false, start_line: 3, end_line: 5 }], moves: [], rc_clones: [], conflicts: [] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("borrows"), "Should show borrow count in stats");
  });

  // 15. HTML shows moves in graph
  it("HTML shows moves", () => {
    const graph = { function_name: "f", variables: [{ name: "a", type_display: "String", ownership_category: "Owned", line: 2, is_copy: false }], borrow_scopes: [], moves: [{ source_name: "a", destination: "b", line: 7, source_type: "String" }], rc_clones: [], conflicts: [] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("moves"), "Should show move count in stats");
  });

  // 16. HTML shows conflicts with warning
  it("HTML shows conflicts", () => {
    const graph = { function_name: "f", variables: [], borrow_scopes: [], moves: [], rc_clones: [], conflicts: [{ borrow_a: "r", borrow_b: "m", variable: "data", overlap_start_line: 4, overlap_end_line: 6 }] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(html.includes("Conflicts"));
    assert.ok(html.includes("r"));
    assert.ok(html.includes("m"));
  });

  // 17. HTML posts ready message (removed - no longer uses acquireVscodeApi)
  it("HTML escapes special characters", () => {
    const graph = { function_name: "<script>alert(1)</script>", variables: [], borrow_scopes: [], moves: [], rc_clones: [], conflicts: [] };
    GraphPanel.createOrShow(vscode.Uri.file("/ext"), graph);
    const html = GraphPanel.currentPanel.getPanel().webview.html;
    assert.ok(!html.includes("<script>alert"), "Should escape HTML");
    assert.ok(html.includes("&lt;script&gt;"), "Should be escaped");
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
