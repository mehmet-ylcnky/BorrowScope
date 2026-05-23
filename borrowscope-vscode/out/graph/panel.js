"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.GraphPanel = void 0;
const vscode = __importStar(require("vscode"));
const model_1 = require("./model");
function buildGraphModelFromRaw(graph) {
    const data = {
        function_name: graph.function_name || "",
        variables: (graph.variables || []).map((v) => ({
            name: v.name,
            type_display: v.type_display || "unknown",
            ownership_category: v.ownership_category || "Unknown",
            line: v.line || 0,
            is_copy: v.is_copy || false,
        })),
        borrow_scopes: graph.borrow_scopes || [],
        moves: graph.moves || [],
        rc_clones: graph.rc_clones || [],
    };
    return (0, model_1.buildGraphModel)(data);
}
class GraphPanel {
    static setContext(context) {
        GraphPanel._context = context;
    }
    static createOrShow(extensionUri, graph, functionList) {
        const column = vscode.ViewColumn.Beside;
        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel._panel.reveal(column);
            if (graph)
                GraphPanel.currentPanel.updateGraph(graph, functionList);
            return;
        }
        const panel = vscode.window.createWebviewPanel("borrowscopeGraph", "BorrowScope: Ownership Graph", column, {
            enableScripts: true,
            retainContextWhenHidden: true,
            localResourceRoots: [vscode.Uri.joinPath(extensionUri, "media")],
        });
        GraphPanel.currentPanel = new GraphPanel(panel, extensionUri);
        if (graph) {
            GraphPanel.currentPanel.updateGraph(graph, functionList);
        }
        else {
            // Restore last saved state
            const saved = GraphPanel._context?.workspaceState.get(GraphPanel.STATE_KEY);
            if (saved) {
                GraphPanel.currentPanel.updateGraph(saved);
            }
        }
    }
    static getPanel() {
        return GraphPanel.currentPanel;
    }
    constructor(panel, extensionUri) {
        this._disposables = [];
        this._panel = panel;
        this._extensionUri = extensionUri;
        this._panel.webview.html = this._buildHtml(undefined);
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
        // Handle messages from WebView
        this._panel.webview.onDidReceiveMessage((message) => {
            if (message.type === "nodeClicked") {
                // If file path provided and it's a different file, open it
                if (message.file && message.file.endsWith(".rs")) {
                    const fileUri = vscode.Uri.file(message.file);
                    vscode.workspace.openTextDocument(fileUri).then((doc) => {
                        vscode.window.showTextDocument(doc, vscode.ViewColumn.One).then((editor) => {
                            if (message.fnName) {
                                for (let i = 0; i < editor.document.lineCount; i++) {
                                    if (new RegExp(`\\bfn\\s+${message.fnName}\\b`).test(editor.document.lineAt(i).text)) {
                                        const pos = new vscode.Position(i, 0);
                                        editor.selection = new vscode.Selection(pos, pos);
                                        editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
                                        break;
                                    }
                                }
                            }
                        });
                    }).catch(() => { });
                }
                else {
                    // Same file navigation
                    const editor = vscode.window.visibleTextEditors.find((e) => e.document.languageId === "rust");
                    if (editor) {
                        let targetLine = message.line > 0 ? message.line - 1 : -1;
                        if (message.fnName && targetLine < 0) {
                            for (let i = 0; i < editor.document.lineCount; i++) {
                                if (new RegExp(`\\bfn\\s+${message.fnName}\\b`).test(editor.document.lineAt(i).text)) {
                                    targetLine = i;
                                    break;
                                }
                            }
                        }
                        if (targetLine >= 0) {
                            const pos = new vscode.Position(targetLine, 0);
                            editor.selection = new vscode.Selection(pos, pos);
                            editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
                            vscode.window.showTextDocument(editor.document, editor.viewColumn);
                        }
                    }
                }
            }
            else if (message.type === "selectFunction" && message.name) {
                this._loadFunction(message.name);
            }
            else if (message.type === "viewChanged" && message.view) {
                GraphPanel._lastView = message.view;
            }
        }, null, this._disposables);
    }
    async _loadFunction(name) {
        const editor = vscode.window.visibleTextEditors.find((e) => e.document.languageId === "rust");
        if (!editor)
            return;
        for (let i = 0; i < editor.document.lineCount; i++) {
            if (new RegExp(`\\bfn\\s+${name}\\b`).test(editor.document.lineAt(i).text)) {
                try {
                    const { getClient } = require("../client");
                    const client = getClient();
                    if (!client)
                        return;
                    const graph = await client.sendRequest("borrowscope/ownershipGraph", {
                        textDocument: { uri: editor.document.uri.toString() },
                        position: { line: i, character: 4 },
                    });
                    if (graph) {
                        // Get function list from editor
                        const fnList = [];
                        for (let j = 0; j < editor.document.lineCount; j++) {
                            const match = editor.document.lineAt(j).text.match(/\bfn\s+(\w+)/);
                            if (match)
                                fnList.push(match[1]);
                        }
                        this.updateGraph(graph, fnList);
                    }
                }
                catch { /* ignore */ }
                break;
            }
        }
    }
    updateGraph(graph, functionList) {
        this._previousGraph = this._currentGraph;
        this._currentGraph = graph;
        this._panel.webview.html = this._buildHtml(graph, functionList, this._previousGraph);
        GraphPanel._context?.workspaceState.update(GraphPanel.STATE_KEY, graph);
    }
    getGraph() {
        return this._currentGraph;
    }
    getPanel() {
        return this._panel;
    }
    dispose() {
        GraphPanel.currentPanel = undefined;
        this._panel.dispose();
        this._disposables.forEach((d) => d.dispose());
        this._disposables = [];
    }
    _buildHtml(graph, functionList, previousGraph) {
        if (!graph) {
            return `<!DOCTYPE html><html><head><meta charset="UTF-8">
<style>body{font-family:var(--vscode-font-family);background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);padding:16px;}</style>
</head><body><h2>BorrowScope: Ownership Graph</h2><p>Waiting for data... Click a CodeLens to load a function.</p></body></html>`;
        }
        const d3Uri = this._panel.webview.asWebviewUri(vscode.Uri.joinPath(this._extensionUri, "media", "d3.min.js"));
        const logoUri = this._panel.webview.asWebviewUri(vscode.Uri.joinPath(this._extensionUri, "media", "logo.png"));
        const vars = graph.variables || [];
        const scopes = graph.borrow_scopes || [];
        const moves = graph.moves || [];
        const clones = graph.rc_clones || [];
        const conflicts = graph.conflicts || [];
        const graphModel = buildGraphModelFromRaw(graph);
        const graphJson = JSON.stringify(graphModel).replace(/</g, "\\u003c");
        const rawGraphJson = JSON.stringify(graph).replace(/</g, "\\u003c");
        const prevGraphJson = previousGraph ? JSON.stringify(previousGraph).replace(/</g, "\\u003c") : "null";
        let varsHtml = vars.map((v) => `<tr data-var="${esc(v.name)}"><td><b>${esc(v.name)}</b></td><td><code>${esc(v.type_display)}</code></td><td>${esc(v.ownership_category)}</td></tr>`).join("");
        let scopesHtml = scopes.map((s) => `<tr data-var="${esc(s.borrower_name)}"><td>${esc(s.borrower_name)}</td><td>${s.is_mutable ? "&amp;mut" : "&amp;"}</td><td>${esc(s.target_name)}</td><td>${s.start_line}-${s.end_line}</td></tr>`).join("");
        let movesHtml = moves.map((m) => `<tr data-var="${esc(m.source_name)}"><td>${esc(m.source_name)}</td><td>↦</td><td>${esc(JSON.stringify(m.destination))}</td><td>${m.line}</td></tr>`).join("");
        let clonesHtml = clones.map((c) => `<tr data-var="${esc(c.clone_variable)}"><td>${esc(c.clone_variable)}</td><td>🔗</td><td>${esc(c.source_variable)}</td><td>${c.line}</td></tr>`).join("");
        let conflictsHtml = conflicts.map((c) => `<tr data-var="${esc(c.borrow_a)}"><td>⚠️ ${esc(c.borrow_a)}</td><td>&amp;</td><td>${esc(c.borrow_b)}</td><td>${c.overlap_start_line}-${c.overlap_end_line}</td></tr>`).join("");
        return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>BorrowScope: ${esc(graph.function_name)}</title>
  <style>
    body { margin:0; padding:0; font-family:var(--vscode-font-family); background:var(--vscode-editor-background); color:var(--vscode-editor-foreground); overflow:hidden; }
    .landing-btn, #landing a { transition: background 0.3s ease, border-color 0.3s ease, transform 0.2s ease; border-color:#3fb950 !important; background:rgba(63,185,80,0.06); }
    .landing-btn:hover, #landing a:hover { background:rgba(63,185,80,0.2); border-color:#56d364 !important; transform:scale(1.05); }
    #header { padding:8px 16px; border-bottom:1px solid var(--vscode-panel-border); font-size:13px; }
    #header h2 { margin:0; font-size:14px; display:inline; }
    #header .stats { opacity:0.7; font-size:12px; margin-left:12px; }
    #view-toggle { position:fixed; right:0; top:50px; display:flex; flex-direction:column; gap:2px; padding:4px; background:var(--vscode-sideBar-background,#252526); border-left:1px solid var(--vscode-panel-border); z-index:10; }
    #view-toggle .view-btn { width:42px; height:42px; display:flex; align-items:center; justify-content:center; font-size:22px; padding:0; border-radius:6px; border:1px solid var(--vscode-panel-border,#454545); background:transparent; color:var(--vscode-foreground); cursor:pointer; transition:background 0.2s; }
    #view-toggle .view-btn:hover { background:rgba(88,166,255,0.1); }
    #view-toggle .view-btn.active { background:var(--vscode-button-background); }
    #graph-container { width:100%; height:calc(100vh - 90px); border-bottom:1px solid var(--vscode-panel-border); }
    svg { width:100%; height:100%; }
    #tables { font-size:12px; }
    #tables table { border-collapse:collapse; width:100%; margin:4px 0 12px 0; }
    #tables th,#tables td { text-align:left; padding:3px 8px; border-bottom:1px solid var(--vscode-panel-border); }
    #tables th { opacity:0.7; font-size:10px; text-transform:uppercase; }
    #tables tr[data-var] { cursor:pointer; transition:background 0.15s; }
    #tables tr[data-var]:hover { background:rgba(52,152,219,0.1); }
    #tables summary { cursor:pointer; padding:4px 0; }
    #tables code { background:var(--vscode-textCodeBlock-background); padding:1px 4px; border-radius:3px; }
    .node circle { stroke-width:2px; cursor:pointer; }
    .node text { font-size:11px; fill:var(--vscode-editor-foreground); pointer-events:none; }
    .node .type-label { font-size:9px; opacity:0.6; }
    .edge path { fill:none; stroke-width:1.5px; }
    .edge text { font-size:9px; fill:var(--vscode-editor-foreground); opacity:0.7; }
    .edge .mutable path { stroke-width:2.5px; }
    #tooltip { position:absolute; background:var(--vscode-editorHoverWidget-background, #1e1e1e); border:1px solid var(--vscode-editorHoverWidget-border, #454545); border-radius:4px; padding:8px 12px; font-size:12px; pointer-events:none; opacity:0; transition:opacity 0.15s; z-index:100; max-width:300px; }
    #tooltip strong { font-size:13px; }
    #tooltip code { background:var(--vscode-textCodeBlock-background); padding:1px 4px; border-radius:2px; font-size:11px; }
    #tooltip hr { border:none; border-top:1px solid var(--vscode-panel-border); margin:4px 0; }
    #filter-bar { padding:4px 16px; border-bottom:1px solid var(--vscode-panel-border); display:flex; gap:6px; flex-wrap:wrap; align-items:center; }
    #filter-bar .filter-label { font-size:11px; opacity:0.6; margin-right:4px; }
    #filter-bar button { border:none; border-radius:3px; padding:2px 8px; font-size:11px; cursor:pointer; color:#fff; opacity:0.9; }
    #filter-bar button.hidden { opacity:0.3; text-decoration:line-through; }
    /* Light theme adaptations */
    .vscode-light .node circle { opacity:0.85; }
    .vscode-light .edge path { opacity:0.7; }
    .vscode-light #filter-bar button { color:#fff; }
    .vscode-light .landing-btn { border-color:#3fb950 !important; }
    /* High contrast mode */
    .vscode-high-contrast .node circle { stroke-width:3px; }
    .vscode-high-contrast .edge path { stroke-width:3px; }
    .vscode-high-contrast #view-toggle .view-btn { border-width:2px; }
    .vscode-high-contrast .landing-btn { border-width:2px !important; }
    .vscode-high-contrast #tooltip { border-width:2px; }
  </style>
</head>
<body>
  <div id="landing" style="display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;padding:20px;">
    <img src="${logoUri}" style="height:256px;margin-bottom:28px;">
    <h2 style="margin:0 0 4px;font-size:16px;">${esc(graph.function_name)}</h2>
    <p style="margin:0 0 24px;font-size:12px;opacity:0.6;">${(graph.variables || []).length} variables, ${(graph.borrow_scopes || []).length} borrows, ${(graph.moves || []).length} moves</p>
    <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:14px;width:100%;max-width:560px;">
      <div class="landing-btn" data-view="graph" title="Force-directed ownership graph showing variables, borrows, and relationships" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🕸️</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Graph</span></div>
      <div class="landing-btn" data-view="table" title="Tabular view of all variables, their types, and ownership categories" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">▦</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Table</span></div>
      <div class="landing-btn" data-view="timeline" title="Chronological timeline of ownership events (borrows, moves, drops)" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">⏱️</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Timeline</span></div>
      <div class="landing-btn" data-view="scopes" title="Nested borrow scopes showing which borrows are active at each point" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🔍</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Scopes</span></div>
      <div class="landing-btn" data-view="refcount" title="Reference counting visualization for Rc/Arc smart pointers" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🔗</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">RefCount</span></div>
      <div class="landing-btn" data-view="moves" title="Ownership transfers between variables with source and destination" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">↦</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Moves</span></div>
      <div class="landing-btn" data-view="conflicts" title="Borrow conflicts where mutable and immutable borrows overlap" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">⚠️</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Conflicts</span></div>
      <div class="landing-btn" data-view="compare" title="Side-by-side comparison of two functions ownership patterns" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🪞</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Compare</span></div>
      <div class="landing-btn" data-view="crossrefs" title="Cross-function borrow tracking across call boundaries" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🔀</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">CrossRefs</span></div>
      <div class="landing-btn" data-view="memory" title="Stack and heap memory layout with sizes, offsets, and timeline" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🧠</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Memory</span></div>
      <div class="landing-btn" data-view="runtime" title="Runtime event timeline, drop order, ref counts, and divergences" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;"><span style="font-size:28px;">🔬</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Runtime</span></div>
      <a href="https://github.com/mehmet-ylcnky/BorrowScope" title="View the source code on GitHub" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;text-decoration:none;color:var(--vscode-foreground);"><span style="font-size:28px;">💻</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Source Code</span></a>
      <a href="https://mehmet-ylcnky.github.io/BorrowScope/" title="Read the technical whitepaper" style="display:flex;flex-direction:column;align-items:center;padding:20px;border:1px solid #585858;border-radius:50%;width:70px;height:70px;justify-content:center;cursor:pointer;text-decoration:none;color:var(--vscode-foreground);"><span style="font-size:28px;">🎓</span><span style="font-size:11px;font-weight:bold;margin-top:6px;">Research</span></a>
    </div>
  </div>
  <div id="main-content" style="display:none;width:100%;padding-right:54px;box-sizing:border-box;">
  <div id="header">
    <h2><button id="home-btn" style="background:none;border:none;cursor:pointer;font-size:22px;vertical-align:middle;margin-right:4px;" title="Back to home">🏠</button>${esc(graph.function_name)}</h2>
    <span class="stats">${(graph.variables || []).length} variables, ${(graph.borrow_scopes || []).length} borrows, ${(graph.moves || []).length} moves</span>
    <div id="view-toggle">
      <button class="view-btn active" data-view="graph" title="Force-directed ownership graph" >🕸️</button>
      <button class="view-btn" data-view="table" title="Tabular view of variables and types" >▦</button>
      <button class="view-btn" data-view="timeline" title="Chronological ownership events" >⏱️</button>
      <button class="view-btn" data-view="scopes" title="Nested borrow scopes" >🔍</button>
      <button class="view-btn" data-view="refcount" title="Rc/Arc reference counting" >🔗</button>
      <button class="view-btn" data-view="moves" title="Ownership transfers" >↦</button>
      <button class="view-btn" data-view="conflicts" title="Borrow conflicts" >⚠️</button>
      <button class="view-btn" data-view="compare" title="Side-by-side function comparison" >🪞</button>
      <button class="view-btn" data-view="crossrefs" title="Cross-function borrow tracking" >🔀</button>
      <button class="view-btn" data-view="memory" title="Stack/heap memory layout" >🧠</button>
      <button class="view-btn" data-view="runtime" title="Runtime events and divergences" >🔬</button>
    </div>
  </div>
  <div id="filter-bar"><span class="filter-label">Filter:</span></div>
  <div id="graph-container" role="img" aria-label="Ownership graph visualization"></div>
  <div id="graph-description" aria-live="polite" style="position:absolute;left:-9999px;width:1px;height:1px;overflow:hidden;"></div>
  <div id="timeline-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;box-sizing:border-box;"></div>
  <div id="scopes-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;padding:12px;box-sizing:border-box;"></div>
  <div id="refcount-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;box-sizing:border-box;"></div>
  <div id="moves-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;padding:12px;box-sizing:border-box;"></div>
  <div id="conflicts-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;padding:12px;box-sizing:border-box;"></div>
  <div id="compare-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;padding:12px;box-sizing:border-box;"></div>
  <div id="crossrefs-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:hidden;box-sizing:border-box;"></div>
  <div id="memory-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;box-sizing:border-box;"></div>
  <div id="runtime-container" style="display:none;width:100%;height:calc(100vh - 90px);overflow:auto;box-sizing:border-box;"></div>
  <div id="tooltip"></div>
  <div id="tables" style="display:none;padding:16px;overflow-y:auto;height:calc(100vh - 90px);width:100%;">
    ${vars.length > 0 ? `<details open><summary><b>Variables (${vars.length})</b></summary>
    <table><tr><th>Name</th><th>Type</th><th>Category</th></tr>${varsHtml}</table></details>` : ""}
    ${scopes.length > 0 ? `<details open><summary><b>Borrow Scopes (${scopes.length})</b></summary>
    <table><tr><th>Borrower</th><th>Kind</th><th>Target</th><th>Lines</th></tr>${scopesHtml}</table></details>` : ""}
    ${moves.length > 0 ? `<details><summary><b>Moves (${moves.length})</b></summary>
    <table><tr><th>Source</th><th></th><th>Destination</th><th>Line</th></tr>${movesHtml}</table></details>` : ""}
    ${clones.length > 0 ? `<details><summary><b>Rc/Arc Clones (${clones.length})</b></summary>
    <table><tr><th>Clone</th><th></th><th>Source</th><th>Line</th></tr>${clonesHtml}</table></details>` : ""}
    ${conflicts.length > 0 ? `<details open><summary><b>⚠️ Conflicts (${conflicts.length})</b></summary>
    <table><tr><th>Borrow A</th><th></th><th>Borrow B</th><th>Lines</th></tr>${conflictsHtml}</table></details>` : ""}
  </div>
  </div><!-- end main-content -->
  <script src="${d3Uri}"></script>
  <script>
    (function() {
      // VS Code WebView API for messaging back to extension
      let vscodeApi;
      try { vscodeApi = acquireVsCodeApi(); } catch(e) { vscodeApi = null; }

      const data = ${graphJson};
      const container = document.getElementById('graph-container');
      const width = container.clientWidth || 600;
      const height = container.clientHeight || 400;

      const svg = d3.select('#graph-container')
        .append('svg')
        .attr('width', width)
        .attr('height', height);

      const g = svg.append('g');

      // Zoom
      svg.call(d3.zoom().on('zoom', (event) => {
        g.attr('transform', event.transform);
      }));

      // Arrow markers
      const defs = svg.append('defs');
      const markerColors = {
        shared_borrow: '#3498db',
        mutable_borrow: '#e74c3c',
        move: '#2ecc71',
        rc_clone: '#9b59b6',
        arc_clone: '#8e44ad',
        closure_capture: '#e67e22',
        refcell_borrow: '#e67e22',
      };
      Object.entries(markerColors).forEach(([kind, color]) => {
        defs.append('marker')
          .attr('id', 'arrow-' + kind)
          .attr('viewBox', '0 -5 10 10')
          .attr('refX', 20)
          .attr('markerWidth', 6)
          .attr('markerHeight', 6)
          .attr('orient', 'auto')
          .append('path')
          .attr('d', 'M0,-5L10,0L0,5')
          .attr('fill', color);
      });

      // Node colors
      function nodeColor(category) {
        const colors = {
          'Owned': '#2ecc71', 'SharedRef': '#3498db', 'MutableRef': '#e74c3c',
          'Rc': '#9b59b6', 'Arc': '#8e44ad', 'InteriorMut': '#e67e22',
          'Copy': '#1abc9c', 'RawPointer': '#7f8c8d', 'Unknown': '#bdc3c7',
          'Closure': '#f1c40f'
        };
        return colors[category] || '#bdc3c7';
      }

      function nodeIcon(category) {
        const icons = {
          'Owned': '📦', 'SharedRef': '👁', 'MutableRef': '🔒',
          'Rc': '🔗', 'Arc': '🔗', 'InteriorMut': '🔄',
          'Copy': '📋', 'RawPointer': '⚡', 'Unknown': '?',
          'Closure': '⚙'
        };
        return icons[category] || '•';
      }

      function edgeColor(kind) {
        return markerColors[kind] || '#95a5a6';
      }

      function edgeDash(kind) {
        if (kind === 'shared_borrow') return '8,4';
        if (kind === 'move') return 'none';
        if (kind === 'rc_clone' || kind === 'arc_clone') return '3,3';
        if (kind === 'closure_capture') return '8,3,2,3';
        if (kind === 'mutable_borrow') return 'none';
        return 'none';
      }

      // Force simulation
      const simulation = d3.forceSimulation(data.nodes)
        .force('link', d3.forceLink(data.edges).id(d => d.id).distance(80))
        .force('charge', d3.forceManyBody().strength(-80))
        .force('center', d3.forceCenter(width / 2, height / 2).strength(0.15))
        .force('collision', d3.forceCollide().radius(30));

      // Edges
      const edge = g.selectAll('.edge')
        .data(data.edges)
        .join('g')
        .attr('class', d => 'edge' + (d.isMutable ? ' mutable' : ''));

      const edgePath = edge.append('path')
        .attr('stroke', d => edgeColor(d.kind))
        .attr('stroke-dasharray', d => edgeDash(d.kind))
        .attr('marker-end', d => 'url(#arrow-' + d.kind + ')');

      const edgeLabel = edge.append('text')
        .text(d => d.label)
        .attr('text-anchor', 'middle');

      // Nodes
      const node = g.selectAll('.node')
        .data(data.nodes)
        .join('g')
        .attr('class', 'node')
        .attr('role', 'button')
        .attr('tabindex', '0')
        .attr('aria-label', d => `;
        Variable;
        $;
        {
            d.name;
        }
        d.type;
    }
}
exports.GraphPanel = GraphPanel;
GraphPanel.STATE_KEY = "borrowscope.lastGraph";
{
    d.category;
}
`)
        .on('keydown', (event, d) => {
          if (event.key === 'Enter' && vscodeApi && d.line > 0) vscodeApi.postMessage({ type: 'nodeClicked', line: d.line });
        })
        .call(d3.drag()
          .on('start', (event, d) => { if (!event.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
          .on('drag', (event, d) => { d.fx = event.x; d.fy = event.y; })
          .on('end', (event, d) => { if (!event.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; })
        );

      node.append('circle')
        .attr('r', d => 12 + d.size * 3)
        .attr('fill', d => nodeColor(d.category))
        .attr('stroke', d => d.isAlive ? nodeColor(d.category) : '#e74c3c')
        .attr('stroke-width', 2)
        .attr('opacity', d => d.isAlive ? 0.9 : 0.4);

      node.append('text')
        .attr('dy', -18)
        .attr('text-anchor', 'middle')
        .text(d => d.name);

      node.append('text')
        .attr('class', 'type-label')
        .attr('dy', 28)
        .attr('text-anchor', 'middle')
        .text(d => d.type.length > 20 ? d.type.slice(0, 18) + '..' : d.type);

      // Tick
      simulation.on('tick', () => {
        edgePath.attr('d', d => {
          return 'M' + d.source.x + ',' + d.source.y + 'L' + d.target.x + ',' + d.target.y;
        });
        edgeLabel.attr('x', d => (d.source.x + d.target.x) / 2)
          .attr('y', d => (d.source.y + d.target.y) / 2 - 5);
        node.attr('transform', d => 'translate(' + d.x + ',' + d.y + ')');
      });

      // === Accessibility: graph description for screen readers ===
      (function() {
        var desc = 'Ownership graph for function ' + rawGraph.function_name + '. ';
        var vars = rawGraph.variables || [];
        var borrows = rawGraph.borrow_scopes || [];
        var moves = rawGraph.moves || [];
        desc += vars.length + ' variables, ' + borrows.length + ' borrows, ' + moves.length + ' moves. ';
        vars.forEach(function(v) { desc += v.name + ' is ' + v.ownership_category + ' ' + v.type_display + '. '; });
        borrows.forEach(function(b) { desc += b.borrower + ' borrows ' + b.owner + ' (' + b.kind + '). '; });
        moves.forEach(function(m) { desc += m.source_name + ' moved to ' + m.destination + '. '; });
        var el = document.getElementById('graph-description');
        if (el) el.textContent = desc;
      })();

      // === Function selector ===
      const fnSelector = document.getElementById('fn-selector');
      if (fnSelector) {
        fnSelector.addEventListener('change', function() {
          if (vscodeApi) {
            vscodeApi.postMessage({ type: 'selectFunction', name: this.value });
          }
        });
      }

      // === Filter by category ===
      const categories = [...new Set(data.nodes.map(n => n.category))];
      const hiddenCategories = new Set();
      const filterBar = d3.select('#filter-bar');

      categories.forEach(cat => {
        filterBar.append('button')
          .attr('class', 'filter')
          .style('background', nodeColor(cat))
          .text(cat)
          .on('click', function() {
            if (hiddenCategories.has(cat)) {
              hiddenCategories.delete(cat);
              d3.select(this).classed('hidden', false);
            } else {
              hiddenCategories.add(cat);
              d3.select(this).classed('hidden', true);
            }
            applyFilters();
          });
      });

      function applyFilters() {
        node.attr('display', d => hiddenCategories.has(d.category) ? 'none' : null);
        edge.attr('display', d => {
          const srcHidden = hiddenCategories.has(d.source.category || '');
          const tgtHidden = hiddenCategories.has(d.target.category || '');
          return (srcHidden || tgtHidden) ? 'none' : null;
        });
      }

      // === Linked highlighting: table <-> graph ===
      function highlightVariable(name) {
        node.select('circle')
          .attr('stroke-width', d => d.name === name ? 4 : 2)
          .attr('stroke', d => d.name === name ? '#fff' : nodeColor(d.category));
        edge.select('path')
          .attr('stroke-width', d => (d.source.name === name || d.target.name === name) ? 3 : 1.5)
          .attr('opacity', d => (d.source.name === name || d.target.name === name) ? 1 : 0.3);
        document.querySelectorAll('#tables tr[data-var]').forEach(row => {
          row.style.background = row.getAttribute('data-var') === name ? 'rgba(52,152,219,0.15)' : '';
        });
      }

      function clearHighlight() {
        node.select('circle')
          .attr('stroke-width', 2)
          .attr('stroke', d => d.isAlive ? nodeColor(d.category) : '#e74c3c');
        edge.select('path')
          .attr('stroke-width', d => d.isMutable ? 2.5 : 1.5)
          .attr('opacity', 1);
        document.querySelectorAll('#tables tr[data-var]').forEach(row => {
          row.style.background = '';
        });
      }

      // Graph node hover -> highlight table + show tooltip
      const tooltip = d3.select('#tooltip');
      node.on('mouseover', (event, d) => {
            highlightVariable(d.name);
            tooltip.html(
              '<strong>' + d.name + '</strong><br>' +
              '<code>' + d.type + '</code>' +
              '<hr>' +
              'Category: ' + d.category + '<br>' +
              'Line: ' + d.line + '<br>' +
              (d.isCopy ? 'Copy type<br>' : '') +
              (!d.isAlive ? '<em>Moved / dropped</em>' : '')
            )
            .style('left', (event.pageX + 12) + 'px')
            .style('top', (event.pageY - 12) + 'px')
            .style('opacity', 1);
          })
          .on('mouseout', () => {
            clearHighlight();
            tooltip.style('opacity', 0);
          })
          .on('click', (event, d) => {
            // Visual highlight on clicked node
            node.select('circle').attr('stroke', n => nodeColor(n.category)).attr('stroke-width', 2);
            d3.select(event.currentTarget).select('circle').attr('stroke', '#fff').attr('stroke-width', 4);
            // Navigate in editor
            if (vscodeApi && d.line > 0) {
              vscodeApi.postMessage({ type: 'nodeClicked', file: '', line: d.line });
            }
          });

      // Table row hover -> highlight graph + timeline
      document.querySelectorAll('#tables tr[data-var]').forEach(row => {
        row.addEventListener('mouseover', () => {
          highlightVariable(row.getAttribute('data-var'));
          // Also highlight timeline bars
          const name = row.getAttribute('data-var');
          document.querySelectorAll('#timeline-container rect[data-var]').forEach(r => {
            r.setAttribute('opacity', r.getAttribute('data-var') === name ? '0.8' : '0.2');
          });
        });
        row.addEventListener('mouseout', () => {
          clearHighlight();
          document.querySelectorAll('#timeline-container rect[data-var]').forEach(r => {
            r.setAttribute('opacity', '0.4');
          });
        });
      });
      // === View toggle: Graph / Timeline ===
      const rawGraph = ${rawGraphJson};
      const lastView = ${JSON.stringify(GraphPanel._lastView || '')};
      // === Landing page handlers ===
      function showView(view) {
        document.getElementById('landing').style.display = 'none';
        document.getElementById('main-content').style.display = '';
        if (vscodeApi) vscodeApi.postMessage({ type: 'viewChanged', view: view });
        // Trigger the view button click
        var btn = document.querySelector('.view-btn[data-view="' + view + '"]');
        if (btn) btn.click();
      }
      function showLanding() {
        document.getElementById('landing').style.display = 'flex';
        document.getElementById('main-content').style.display = 'none';
      }
      document.querySelectorAll('.landing-btn').forEach(btn => {
        btn.addEventListener('click', function() { showView(this.getAttribute('data-view')); });
      });
      document.getElementById('home-btn').addEventListener('click', showLanding);

      // === View toggle: Graph / Timeline / etc ===
      document.querySelectorAll('.view-btn').forEach(btn => {
        btn.addEventListener('click', function() {
          document.querySelectorAll('.view-btn').forEach(b => {
            b.style.background = 'transparent';
            b.style.color = 'var(--vscode-foreground)';
            b.classList.remove('active');
          });
          this.style.background = 'var(--vscode-button-background)';
          this.style.color = 'var(--vscode-button-foreground)';
          this.classList.add('active');
          const view = this.getAttribute('data-view');
          if (vscodeApi) vscodeApi.postMessage({ type: 'viewChanged', view: view });
          document.getElementById('graph-container').style.display = view === 'graph' ? '' : 'none';
          document.getElementById('filter-bar').style.display = view === 'graph' ? 'flex' : 'none';
          document.getElementById('tables').style.display = view === 'table' ? '' : 'none';
          document.getElementById('timeline-container').style.display = view === 'timeline' ? '' : 'none';
          document.getElementById('scopes-container').style.display = view === 'scopes' ? '' : 'none';
          document.getElementById('refcount-container').style.display = view === 'refcount' ? '' : 'none';
          document.getElementById('moves-container').style.display = view === 'moves' ? '' : 'none';
          document.getElementById('conflicts-container').style.display = view === 'conflicts' ? '' : 'none';
          document.getElementById('compare-container').style.display = view === 'compare' ? '' : 'none';
          document.getElementById('crossrefs-container').style.display = view === 'crossrefs' ? 'flex' : 'none';
          document.getElementById('memory-container').style.display = view === 'memory' ? '' : 'none';
          document.getElementById('runtime-container').style.display = view === 'runtime' ? '' : 'none';
          if (view === 'timeline') renderTimeline();
          if (view === 'scopes') renderScopes();
          if (view === 'refcount') renderRefCount();
          if (view === 'moves') renderMoves();
          if (view === 'conflicts') renderConflicts();
          if (view === 'compare') renderCompare();
          if (view === 'crossrefs') renderCrossRefs();
          if (view === 'memory') renderMemory();
          if (view === 'runtime') renderRuntime();
        });
      });

      // === Timeline (Gantt chart) rendering ===
      function renderTimeline() {
        const container = document.getElementById('timeline-container');
        container.innerHTML = '';

        const vars = (rawGraph.variables || []).map(v => ({
          name: v.name, type: v.type_display, category: v.ownership_category,
          start: v.line, end: rawGraph.end_line || v.line + 10
        }));
        const scopes = (rawGraph.borrow_scopes || []).map(s => ({
          borrower: s.borrower_name, target: s.target_name, mutable: s.is_mutable,
          start: s.start_line, end: s.end_line
        }));
        const conflicts = (rawGraph.conflicts || []).map(c => ({
          start: c.overlap_start_line, end: c.overlap_end_line
        }));

        // Update var end lines from scopes
        for (const s of scopes) {
          const bVar = vars.find(v => v.name === s.borrower);
          if (bVar) { bVar.start = Math.min(bVar.start, s.start); bVar.end = Math.max(bVar.end, s.end); }
        }

        if (vars.length === 0) { container.innerHTML = '<p style="padding:20px;opacity:0.5">No variables to display</p>'; return; }

        const margin = { top: 30, right: 20, bottom: 30, left: 120 };
        const barH = 22, gap = 4;
        const width = container.clientWidth || 500;
        const height = vars.length * (barH + gap) + margin.top + margin.bottom;
        const minLine = Math.min(...vars.map(v => v.start));
        const maxLine = Math.max(...vars.map(v => v.end));

        const svg = d3.select('#timeline-container').append('svg')
          .attr('width', width).attr('height', height);

        const x = d3.scaleLinear().domain([minLine, maxLine + 1]).range([margin.left, width - margin.right]);

        // Grid lines
        for (let l = minLine; l <= maxLine; l += Math.max(1, Math.floor((maxLine - minLine) / 10))) {
          svg.append('line').attr('x1', x(l)).attr('x2', x(l)).attr('y1', margin.top - 10).attr('y2', height - margin.bottom)
            .attr('stroke', 'var(--vscode-panel-border)').attr('stroke-dasharray', '2,2');
          svg.append('text').attr('x', x(l)).attr('y', margin.top - 14).attr('text-anchor', 'middle')
            .attr('font-size', '9px').attr('fill', 'var(--vscode-descriptionForeground)').text(l);
        }

        // Conflict zones (red vertical bands)
        for (const c of conflicts) {
          svg.append('rect').attr('x', x(c.start)).attr('width', x(c.end) - x(c.start))
            .attr('y', margin.top - 5).attr('height', height - margin.top - margin.bottom + 10)
            .attr('fill', 'rgba(231,76,60,0.1)').attr('stroke', 'rgba(231,76,60,0.3)').attr('stroke-dasharray', '4,2');
        }

        // Variable bars
        vars.forEach((v, i) => {
          const y = margin.top + i * (barH + gap);
          const color = nodeColor(v.category);

          // Label
          svg.append('text').attr('x', margin.left - 8).attr('y', y + barH / 2 + 4)
            .attr('text-anchor', 'end').attr('font-size', '11px').attr('fill', 'var(--vscode-foreground)').text(v.name);

          // Lifetime bar
          svg.append('rect').attr('x', x(v.start)).attr('width', Math.max(2, x(v.end) - x(v.start)))
            .attr('y', y).attr('height', barH).attr('rx', 3)
            .attr('fill', color).attr('opacity', 0.4).attr('stroke', color).attr('stroke-width', 1)
            .attr('data-var', v.name)
            .style('cursor', 'pointer')
            .on('click', () => { if (vscodeApi) vscodeApi.postMessage({ type: 'nodeClicked', line: v.start }); })
            .on('mouseover', () => {
              document.querySelectorAll('#tables tr[data-var]').forEach(row => {
                row.style.background = row.getAttribute('data-var') === v.name ? 'rgba(52,152,219,0.15)' : '';
              });
              svg.selectAll('rect[data-var]').attr('opacity', d => d === v.name ? 0.8 : 0.2);
            })
            .on('mouseout', () => {
              document.querySelectorAll('#tables tr[data-var]').forEach(row => { row.style.background = ''; });
              svg.selectAll('rect[data-var]').attr('opacity', 0.4);
            });

          // Borrow overlays on this variable
          scopes.filter(s => s.target === v.name).forEach(s => {
            svg.append('rect').attr('x', x(s.start)).attr('width', Math.max(2, x(s.end) - x(s.start)))
              .attr('y', y + 3).attr('height', barH - 6).attr('rx', 2)
              .attr('fill', s.mutable ? '#e74c3c' : '#3498db').attr('opacity', 0.6);
            svg.append('text').attr('x', x(s.start) + 3).attr('y', y + barH / 2 + 3)
              .attr('font-size', '9px').attr('fill', '#fff').text(s.mutable ? '&mut ' + s.borrower : '& ' + s.borrower);
          });
        });

        // Legend
        const ly = height - 15;
        svg.append('rect').attr('x', margin.left).attr('y', ly).attr('width', 12).attr('height', 8).attr('fill', '#2ecc71').attr('opacity', 0.4);
        svg.append('text').attr('x', margin.left + 16).attr('y', ly + 7).attr('font-size', '9px').attr('fill', 'var(--vscode-descriptionForeground)').text('alive');
        svg.append('rect').attr('x', margin.left + 60).attr('y', ly).attr('width', 12).attr('height', 8).attr('fill', '#3498db').attr('opacity', 0.6);
        svg.append('text').attr('x', margin.left + 76).attr('y', ly + 7).attr('font-size', '9px').attr('fill', 'var(--vscode-descriptionForeground)').text('& borrow');
        svg.append('rect').attr('x', margin.left + 140).attr('y', ly).attr('width', 12).attr('height', 8).attr('fill', '#e74c3c').attr('opacity', 0.6);
        svg.append('text').attr('x', margin.left + 156).attr('y', ly + 7).attr('font-size', '9px').attr('fill', 'var(--vscode-descriptionForeground)').text('&mut borrow');
      }

      // === Scope Nesting View ===
      function renderScopes() {
        const container = document.getElementById('scopes-container');
        container.innerHTML = '';

        const vars = (rawGraph.variables || []);
        const scopes = (rawGraph.borrow_scopes || []);
        const fnName = rawGraph.function_name || 'fn';
        const fnStart = rawGraph.start_line || 1;
        const fnEnd = rawGraph.end_line || 20;

        if (vars.length === 0) {
          container.innerHTML = '<p style="opacity:0.5">No variables in this function</p>';
          return;
        }

        // Detect inner blocks: variables whose borrows end before function end
        const innerVarNames = new Set();
        for (const s of scopes) {
          if (s.end_line < fnEnd - 2) innerVarNames.add(s.borrower_name);
        }

        const outerVars = vars.filter(v => !innerVarNames.has(v.name));
        const innerVars = vars.filter(v => innerVarNames.has(v.name));

        // Render nested boxes as HTML
        let html = '<div style="border:2px solid #58a6ff;border-radius:6px;padding:10px;margin:4px 0;">';
        html += '<div style="font-size:11px;color:#58a6ff;margin-bottom:8px;font-weight:bold;">fn ' + fnName + '() — lines ' + fnStart + '-' + fnEnd + '</div>';

        // Outer variables
        for (const v of outerVars) {
          html += renderVarBox(v);
        }

        // Inner block (if any)
        if (innerVars.length > 0) {
          const blockStart = Math.min(...innerVars.map(v => v.line));
          const blockEnd = Math.max(...innerVars.map(v => v.line)) + 2;
          html += '<div style="border:1px solid #a6adc8;border-radius:4px;padding:8px;margin:6px 0 6px 16px;">';
          html += '<div style="font-size:10px;color:#a6adc8;margin-bottom:6px;">{ block } — lines ' + blockStart + '-' + blockEnd + '</div>';
          for (const v of innerVars) {
            html += renderVarBox(v);
          }
          html += '</div>';
        }

        // Drop order
        const dropOrder = [...vars].reverse().map(v => v.name);
        html += '<div style="margin-top:10px;padding-top:8px;border-top:1px solid var(--vscode-panel-border);font-size:10px;color:#a6adc8;">';
        html += '💀 Drop order: ' + dropOrder.join(' → ');
        html += '</div>';

        html += '</div>';
        container.innerHTML = html;

        // Click and hover handlers
        container.querySelectorAll('[data-line]').forEach(el => {
          el.addEventListener('click', () => {
            if (vscodeApi) vscodeApi.postMessage({ type: 'nodeClicked', line: parseInt(el.getAttribute('data-line')) });
          });
        });
        container.querySelectorAll('.scope-var').forEach(el => {
          el.addEventListener('mouseover', () => {
            const name = el.getAttribute('data-var');
            document.querySelectorAll('#tables tr[data-var]').forEach(row => {
              row.style.background = row.getAttribute('data-var') === name ? 'rgba(52,152,219,0.15)' : '';
            });
          });
          el.addEventListener('mouseout', () => {
            document.querySelectorAll('#tables tr[data-var]').forEach(row => { row.style.background = ''; });
          });
        });
      }

      // === Reference Count History ===
      function renderRefCount() {
        const container = document.getElementById("refcount-container");
        container.innerHTML = "";
        const clones = rawGraph.rc_clones || [];
        const vars = rawGraph.variables || [];
        const fnEnd = rawGraph.end_line || 20;
        if (clones.length === 0) {
          container.innerHTML = '<p style="padding:20px;opacity:0.5">No Rc/Arc variables in this function</p>';
          return;
        }
        // Build series per source
        const sources = new Map();
        for (const v of vars) {
          if ((v.ownership_category === "Rc" || v.ownership_category === "Arc") && !clones.some(c => c.clone_variable === v.name)) {
            sources.set(v.name, [{ line: v.line, count: 1, label: "new " + v.name }]);
          }
        }
        for (const c of clones) {
          const s = sources.get(c.source_variable);
          if (s) { const prev = s[s.length-1].count; s.push({ line: c.line, count: prev+1, label: "clone " + c.clone_variable }); }
        }
        // Add drops
        for (const [name, events] of sources) {
          const numClones = clones.filter(c => c.source_variable === name).length;
          for (let i = 0; i < numClones; i++) {
            const prev = events[events.length-1].count;
            events.push({ line: fnEnd - numClones + i, count: prev - 1, label: "drop clone" });
          }
          const prev = events[events.length-1].count;
          events.push({ line: fnEnd, count: prev - 1, label: "drop " + name });
        }
        // Render chart per series
        const margin = { top: 30, right: 20, bottom: 30, left: 50 };
        const chartH = 120;
        let totalH = 0;
        const svg = d3.select("#refcount-container").append("svg").attr("width", "100%");
        for (const [name, events] of sources) {
          const g = svg.append("g").attr("transform", "translate(0," + totalH + ")");
          const w = (container.clientWidth || 400) - margin.left - margin.right;
          const xMin = Math.min(...events.map(e => e.line));
          const xMax = Math.max(...events.map(e => e.line));
          const yMax = Math.max(...events.map(e => e.count));
          const x = d3.scaleLinear().domain([xMin, xMax]).range([margin.left, margin.left + w]);
          const y = d3.scaleLinear().domain([0, yMax + 1]).range([chartH - margin.bottom, margin.top]);
          // Title
          g.append("text").attr("x", margin.left).attr("y", 16).attr("font-size", "12px").attr("fill", "#cba6f7").attr("font-weight", "bold").text(name + " (ref count)");
          // Axes
          g.append("line").attr("x1", margin.left).attr("x2", margin.left + w).attr("y1", chartH - margin.bottom).attr("y2", chartH - margin.bottom).attr("stroke", "var(--vscode-panel-border)");
          g.append("line").attr("x1", margin.left).attr("x2", margin.left).attr("y1", margin.top).attr("y2", chartH - margin.bottom).attr("stroke", "var(--vscode-panel-border)");
          // Y labels
          for (let i = 0; i <= yMax; i++) {
            g.append("text").attr("x", margin.left - 8).attr("y", y(i) + 4).attr("text-anchor", "end").attr("font-size", "9px").attr("fill", "var(--vscode-descriptionForeground)").text(i);
          }
          // Step line
          const line = d3.line().x(d => x(d.line)).y(d => y(d.count)).curve(d3.curveStepAfter);
          g.append("path").datum(events).attr("d", line).attr("stroke", "#cba6f7").attr("stroke-width", 2).attr("fill", "none");
          // Event dots
          g.selectAll(".dot").data(events).join("circle").attr("cx", d => x(d.line)).attr("cy", d => y(d.count)).attr("r", 4).attr("fill", d => d.count === 0 ? "#f85149" : "#cba6f7");
          // Event labels
          g.selectAll(".elabel").data(events).join("text").attr("x", d => x(d.line)).attr("y", d => y(d.count) - 8).attr("text-anchor", "middle").attr("font-size", "9px").attr("fill", "var(--vscode-descriptionForeground)").text(d => d.label);
          // Warning if leaked
          const finalCount = events[events.length-1].count;
          if (finalCount > 0) {
            g.append("text").attr("x", margin.left + w / 2).attr("y", chartH - 5).attr("text-anchor", "middle").attr("font-size", "11px").attr("fill", "#f9e2af").text("⚠️ Potential leak: final count = " + finalCount);
          }
          totalH += chartH + 10;
        }
        svg.attr("height", totalH);
      }

      // === Comparison View ===
      var previousGraph = ${prevGraphJson};
      function renderCompare() {
        const container = document.getElementById('compare-container');
        container.innerHTML = '';
        if (!previousGraph) {
          container.innerHTML = '<div style="padding:20px;text-align:center;opacity:0.7;"><p>No previous state to compare.</p><p style="font-size:11px;">Click a different function CodeLens, then come back to see the diff.</p></div>';
          return;
        }
        // Compute diff
        const beforeVars = new Set((previousGraph.variables || []).map(v => v.name));
        const afterVars = new Set((rawGraph.variables || []).map(v => v.name));
        const beforeBorrows = new Set((previousGraph.borrow_scopes || []).map(s => s.borrower_name + '->' + s.target_name));
        const afterBorrows = new Set((rawGraph.borrow_scopes || []).map(s => s.borrower_name + '->' + s.target_name));

        const added = [...afterVars].filter(v => !beforeVars.has(v));
        const removed = [...beforeVars].filter(v => !afterVars.has(v));
        const addedB = [...afterBorrows].filter(b => !beforeBorrows.has(b));
        const removedB = [...beforeBorrows].filter(b => !afterBorrows.has(b));

        const hasChanges = added.length + removed.length + addedB.length + removedB.length > 0;

        let html = '<div style="margin-bottom:12px;font-size:12px;font-weight:bold;">';
        html += 'Comparing: <span style="color:#8b949e;">' + (previousGraph.function_name || '?') + '</span> → <span style="color:#58a6ff;">' + (rawGraph.function_name || '?') + '</span>';
        html += '</div>';

        if (!hasChanges) {
          html += '<div style="text-align:center;padding:20px;"><span style="font-size:24px;">≡</span><p style="opacity:0.7;margin-top:8px;">No ownership changes</p></div>';
        } else {
          // Summary
          var parts = [];
          if (added.length) parts.push('+' + added.length + ' vars');
          if (removed.length) parts.push('-' + removed.length + ' vars');
          if (addedB.length) parts.push('+' + addedB.length + ' borrows');
          if (removedB.length) parts.push('-' + removedB.length + ' borrows');
          html += '<div style="padding:6px 10px;background:rgba(88,166,255,0.1);border-radius:4px;margin-bottom:12px;font-size:12px;">' + parts.join(', ') + '</div>';

          // Added variables
          if (added.length) {
            html += '<div style="margin:8px 0;"><b style="color:#3fb950;">+ Added variables:</b></div>';
            for (const name of added) {
              html += '<div style="border-left:3px solid #3fb950;padding:4px 8px;margin:4px 0 4px 12px;font-size:11px;">' + name + '</div>';
            }
          }
          // Removed variables
          if (removed.length) {
            html += '<div style="margin:8px 0;"><b style="color:#f85149;">- Removed variables:</b></div>';
            for (const name of removed) {
              html += '<div style="border-left:3px solid #f85149;padding:4px 8px;margin:4px 0 4px 12px;font-size:11px;text-decoration:line-through;opacity:0.6;">' + name + '</div>';
            }
          }
          // Added borrows
          if (addedB.length) {
            html += '<div style="margin:8px 0;"><b style="color:#3fb950;">+ Added borrows:</b></div>';
            for (const b of addedB) {
              html += '<div style="border-left:3px solid #3fb950;padding:4px 8px;margin:4px 0 4px 12px;font-size:11px;">' + b + '</div>';
            }
          }
          // Removed borrows
          if (removedB.length) {
            html += '<div style="margin:8px 0;"><b style="color:#f85149;">- Removed borrows:</b></div>';
            for (const b of removedB) {
              html += '<div style="border-left:3px solid #f85149;padding:4px 8px;margin:4px 0 4px 12px;font-size:11px;text-decoration:line-through;opacity:0.6;">' + b + '</div>';
            }
          }
        }
        container.innerHTML = html;
      }
      // Store current as previous for next comparison

      // === Conflict Highlight View ===
      function renderConflicts() {
        const container = document.getElementById('conflicts-container');
        container.innerHTML = '';
        const conflicts = rawGraph.conflicts || [];
        if (conflicts.length === 0) {
          container.innerHTML = '<div style="padding:20px;text-align:center;"><span style="font-size:24px;">✓</span><p style="opacity:0.7;margin-top:8px;">No conflicts detected in this function</p><p style="font-size:11px;opacity:0.5;">All borrows have non-overlapping scopes</p></div>';
          return;
        }
        let html = '<div style="margin-bottom:12px;font-size:12px;color:#f85149;font-weight:bold;">⚠️ ' + conflicts.length + ' conflict(s) detected</div>';
        for (const c of conflicts) {
          html += '<div style="border:2px solid #f85149;border-radius:6px;padding:12px;margin:8px 0;background:rgba(248,81,73,0.05);">';
          html += '<div style="font-weight:bold;margin-bottom:6px;">Conflict on <code style="color:#f9e2af;">' + c.variable + '</code></div>';
          html += '<div style="display:flex;gap:12px;margin:8px 0;">';
          // Borrow A
          html += '<div style="flex:1;border:1px solid #3498db;border-radius:4px;padding:8px;">';
          html += '<div style="font-size:10px;color:#3498db;margin-bottom:4px;">Borrow A</div>';
          html += '<b>' + c.borrow_a + '</b>';
          html += '</div>';
          // VS
          html += '<div style="display:flex;align-items:center;color:#f85149;font-weight:bold;">⚡</div>';
          // Borrow B
          html += '<div style="flex:1;border:1px solid #e74c3c;border-radius:4px;padding:8px;">';
          html += '<div style="font-size:10px;color:#e74c3c;margin-bottom:4px;">Borrow B</div>';
          html += '<b>' + c.borrow_b + '</b>';
          html += '</div>';
          html += '</div>';
          // Overlap info
          html += '<div style="font-size:11px;color:#8b949e;margin-top:6px;">';
          html += 'Overlap: lines ' + c.overlap_start_line + ' – ' + c.overlap_end_line;
          html += ' <span data-line="' + c.overlap_start_line + '" style="color:#58a6ff;cursor:pointer;text-decoration:underline;">Go to line →</span>';
          html += '</div>';
          html += '</div>';
        }
        container.innerHTML = html;
        container.querySelectorAll('[data-line]').forEach(el => {
          el.addEventListener('click', () => {
            if (vscodeApi) vscodeApi.postMessage({ type: 'nodeClicked', line: parseInt(el.getAttribute('data-line')) });
          });
        });
      }

      // === Move Chain View ===
      function renderMoves() {
        const container = document.getElementById('moves-container');
        container.innerHTML = '';
        const moves = rawGraph.moves || [];
        if (moves.length === 0) {
          container.innerHTML = '<p style="padding:20px;opacity:0.5">No ownership transfers in this function</p>';
          return;
        }
        const vars = rawGraph.variables || [];
        const movedAway = new Set(moves.map(m => m.source_name));
        // Build chain HTML
        let html = '';
        for (const m of moves) {
          const dest = typeof m.destination === 'string' ? m.destination : (m.destination.Variable || m.destination.FunctionArg?.function + '()' || m.destination.Return ? '(return)' : m.destination.Closure ? '(closure)' : JSON.stringify(m.destination));
          const srcVar = vars.find(v => v.name === m.source_name);
          const srcType = srcVar ? srcVar.type_display : m.source_type || '';
          html += '<div style="display:flex;align-items:center;gap:8px;margin:8px 0;">';
          // Source box
          html += '<div data-line="' + (srcVar ? srcVar.line : m.line) + '" style="border:2px solid #e67e22;border-radius:6px;padding:8px 12px;cursor:pointer;opacity:0.5;text-decoration:line-through;">';
          html += '<b>' + m.source_name + '</b><br><span style="font-size:10px;color:#8b949e;">' + srcType + '</span>';
          html += '<br><span style="color:#f85149;font-size:10px;">✗ moved</span>';
          html += '</div>';
          // Arrow
          html += '<div style="display:flex;flex-direction:column;align-items:center;">';
          html += '<span style="font-size:18px;color:#e67e22;">→</span>';
          html += '<span style="font-size:9px;color:#8b949e;">line ' + m.line + '</span>';
          html += '</div>';
          // Destination box
          const isAlive = !movedAway.has(dest);
          html += '<div data-line="' + m.line + '" style="border:2px solid ' + (isAlive ? '#2ecc71' : '#e67e22') + ';border-radius:6px;padding:8px 12px;cursor:pointer;' + (isAlive ? '' : 'opacity:0.5;') + '">';
          html += '<b>' + dest + '</b>';
          if (isAlive) html += '<br><span style="color:#2ecc71;font-size:10px;">✓ alive</span>';
          html += '</div>';
          html += '</div>';
        }
        container.innerHTML = html;
        // Click handlers
        container.querySelectorAll('[data-line]').forEach(el => {
          el.addEventListener('click', () => {
            if (vscodeApi) vscodeApi.postMessage({ type: 'nodeClicked', line: parseInt(el.getAttribute('data-line')) });
          });
        });
      }

      // === Cross-References View ===
      function renderCrossRefs() {
        const container = document.getElementById('crossrefs-container');
        container.innerHTML = '';
        container.style.display = 'flex';

        const crossRefs = rawGraph._crossRefs || [];
        const fnName = rawGraph.function_name || '';
        const currentFile = 'main.rs';

        if (crossRefs.length === 0) {
          container.innerHTML = '<div style="padding:20px;width:100%;text-align:center;opacity:0.5;"><p>No cross-function references detected.</p><p style="font-size:11px;">Click a function CodeLens that calls other functions with &amp; arguments.</p></div>';
          return;
        }

        // Build nodes and edges from cross-refs
        var nodes = [{ id: fnName, kind: 'origin' }];
        var edges = [];
        var nodeSet = new Set([fnName]);
        for (var cr of crossRefs) {
          if (cr.path.length < 2) continue;
          var target = cr.path[1];
          if (!nodeSet.has(target.function_name)) {
            nodeSet.add(target.function_name);
            nodes.push({ id: target.function_name, kind: 'target', param: target.variable, isMutable: target.is_mutable, file: target.file || '' });
          }
          edges.push({ source: fnName, target: target.function_name, variable: cr.origin_variable, isMutable: target.is_mutable });
        }

        // Build file tree - only include project files (skip stdlib paths)
        var fileSet = new Map();
        for (var cr of crossRefs) {
          for (var seg of cr.path) {
            var f = (seg.file || '');
            // Extract relative path from /src/
            var srcIdx = f.indexOf('/src/');
            if (srcIdx >= 0) f = f.substring(srcIdx + 1);
            else f = f.split('/').pop() || '';
            // Skip stdlib paths (contain /rust/ or /library/ or /alloc/)
            if (seg.file && (seg.file.includes('/rust/') || seg.file.includes('/library/') || seg.file.includes('/rustc/'))) continue;
            if (!f || !f.endsWith('.rs')) continue;
            if (!fileSet.has(f)) fileSet.set(f, { count: 0, isCurrent: false, fullPath: seg.file || '' });
            fileSet.get(f).count++;
          }
        }
        // Mark current file
        var currentRelative = 'src/main.rs';
        if (fileSet.has(currentRelative)) { fileSet.get(currentRelative).isCurrent = true; fileSet.get(currentRelative).count = 0; }
        else { fileSet.set(currentRelative, { count: 0, isCurrent: true, fullPath: '' }); }

        var html = '<div style="width:180px;border-right:1px solid var(--vscode-panel-border);padding:8px;font-size:11px;overflow-y:auto;">';
        html += '<div style="font-size:10px;color:var(--vscode-descriptionForeground);margin-bottom:6px;text-transform:uppercase;">Files</div>';
        for (var [fname, info] of fileSet) {
          var cls = info.isCurrent ? 'background:rgba(88,166,255,0.15);font-weight:bold;' : (info.count > 0 ? '' : 'opacity:0.35;');
          var icon = info.isCurrent ? '#58a6ff' : (info.count > 0 ? '#3fb950' : '#484f58');
          var badge = info.count > 0 ? ' <span style="background:var(--vscode-badge-background,#4d4d4d);color:var(--vscode-badge-foreground,#fff);border-radius:8px;padding:0 5px;font-size:10px;">' + info.count + '</span>' : '';
          html += '<div class="file-entry" data-path="' + (info.fullPath || '') + '" style="padding:3px 6px;border-radius:3px;margin:2px 0;display:flex;align-items:center;gap:4px;cursor:pointer;' + cls + '"><div style="width:8px;height:8px;border-radius:50%;background:' + icon + ';"></div><span style="overflow:hidden;text-overflow:ellipsis;">' + fname + '</span>' + badge + '</div>';
        }
        html += '</div>';

        // Graph (right)
        html += '<div id="crossrefs-graph" style="flex:1;"></div>';
        container.innerHTML = html;

        // Render D3 graph
        var graphDiv = document.getElementById('crossrefs-graph');
        var w = graphDiv.clientWidth || 300;
        var h = graphDiv.clientHeight || 250;

        var svg = d3.select('#crossrefs-graph').append('svg').attr('width', w).attr('height', h);
        var g2 = svg.append('g');
        svg.call(d3.zoom().on('zoom', function(event) { g2.attr('transform', event.transform); }));

        var sim = d3.forceSimulation(nodes)
          .force('link', d3.forceLink(edges).id(function(d){return d.id;}).distance(100))
          .force('charge', d3.forceManyBody().strength(-150))
          .force('center', d3.forceCenter(w/2, h/2));

        var link = g2.selectAll('.link').data(edges).join('g');
        link.append('line').attr('stroke', function(d){return d.isMutable?'#e74c3c':'#3498db';}).attr('stroke-width', function(d){return d.isMutable?3:1.5;});
        link.append('text').attr('font-size','9px').attr('fill','var(--vscode-descriptionForeground)').text(function(d){return (d.isMutable?'&mut ':'&')+d.variable;});

        var nd = g2.selectAll('.nd').data(nodes).join('g')
          .call(d3.drag().on('start',function(e,d){if(!e.active)sim.alphaTarget(0.3).restart();d.fx=d.x;d.fy=d.y;}).on('drag',function(e,d){d.fx=e.x;d.fy=e.y;}).on('end',function(e,d){if(!e.active)sim.alphaTarget(0);d.fx=null;d.fy=null;}));
        nd.append('circle').attr('r',18).attr('fill', function(d){return d.kind==='origin'?'#58a6ff':'#3fb950';}).attr('opacity',0.8).attr('stroke',function(d){return d.kind==='origin'?'#58a6ff':'#3fb950';}).attr('stroke-width',2);
        nd.append('text').attr('dy',-22).attr('text-anchor','middle').attr('font-size','11px').attr('fill','var(--vscode-editor-foreground)').text(function(d){return d.id+'()';});
        nd.append('text').attr('dy',4).attr('text-anchor','middle').attr('font-size','9px').attr('fill','var(--vscode-descriptionForeground)').text(function(d){return d.param||'';});
        nd.style("cursor","pointer").on("click", function(event, d) { if (vscodeApi) vscodeApi.postMessage({ type: "nodeClicked", file: d.file || "", line: 0, fnName: d.id }); });

        sim.on('tick', function() {
          link.select('line').attr('x1',function(d){return d.source.x;}).attr('y1',function(d){return d.source.y;}).attr('x2',function(d){return d.target.x;}).attr('y2',function(d){return d.target.y;});
          link.select('text').attr('x',function(d){return (d.source.x+d.target.x)/2;}).attr('y',function(d){return (d.source.y+d.target.y)/2-8;});
          nd.attr('transform', function(d){return 'translate('+d.x+','+d.y+')';});
        });
      }

      // === Memory Layout View ===
      function renderMemory() {
        const container = document.getElementById('memory-container');
        container.innerHTML = '';
        const layout = rawGraph._memoryLayout;
        const rtMem = rawGraph._memoryRuntime;

        // Runtime mode: show actual addresses
        if (rtMem && rtMem.variables && rtMem.variables.length > 0) {
          renderMemoryRuntime(container, rtMem);
          return;
        }

        if (!layout || !layout.stack_frame) {
          container.innerHTML = '<div style="padding:20px;text-align:center;opacity:0.5;"><p>No memory layout data.</p><p style="font-size:11px;">Click a function CodeLens to load memory layout.</p></div>';
          return;
        }

        const allVars = layout.stack_frame.variables || [];
        const allHeap = layout.heap_allocations || [];
        const allPtrs = layout.pointer_relationships || [];
        const catColors = { StackOnly:'#2ecc71', HeapBacked:'#3498db', Reference:'#9b59b6', RefCounted:'#cba6f7', InteriorMut:'#e67e22' };

        const minLine = allVars.length > 0 ? Math.min(...allVars.map(v => v.line)) : 1;
        const maxLine = rawGraph.end_line || (allVars.length > 0 ? Math.max(...allVars.map(v => v.line)) + 5 : 10);
        var currentLine = maxLine;

        function renderAtLine(line) {
          var stackVars = allVars.filter(v => v.line <= line && (!v.end_line || v.end_line >= line));
          var droppedVars = allVars.filter(v => v.end_line && v.end_line < line && v.line <= line);
          var heapAllocs = allHeap.filter(h => stackVars.some(v => v.name === h.owner));
          var pointers = allPtrs.filter(p => stackVars.some(v => v.name === p.from));

          var html = '<div style="display:flex;gap:12px;padding:12px;height:calc(100% - 40px);">';

          // Stack column
          html += '<div style="flex:1;border:1px solid #58a6ff;border-radius:6px;padding:8px;overflow-y:auto;">';
          var stackSize = stackVars.reduce(function(s,v){return s+v.size;},0);
          html += '<div style="font-size:11px;color:#58a6ff;font-weight:bold;margin-bottom:8px;">STACK (' + stackSize + 'B)</div>';
          for (var v of stackVars) {
            var color = catColors[v.category] || '#95a5a6';
            var isNew = v.line === line;
            html += '<div style="border-left:3px solid ' + color + ';padding:6px 10px;margin:4px 0;border-radius:3px;font-size:11px;' + (isNew ? 'background:rgba(88,166,255,0.1);' : '') + '">';
            html += '<div style="display:flex;justify-content:space-between;align-items:center;">';
            html += '<b>' + v.name + '</b>';
            html += '<span style="color:#8b949e;font-size:10px;">' + v.size + 'B (align ' + v.alignment + ')</span>';
            html += '</div>';
            html += '<div style="font-size:10px;margin-top:2px;"><code>' + v.type_display + '</code></div>';
            html += '<div style="display:flex;gap:8px;margin-top:3px;font-size:9px;color:#8b949e;">';
            html += '<span>offset: +' + v.offset + '</span>';
            html += '<span>lines: ' + v.line + '–' + (v.end_line || '?') + '</span>';
            html += '<span style="color:' + color + ';">' + v.category + '</span>';
            html += '</div>';
            if (isNew) html += '<div style="color:#3fb950;font-size:9px;margin-top:2px;">● allocated this line</div>';
            // Show internal fields if available
            if (v.fields && v.fields.length > 0) {
              html += '<div style="margin-top:3px;padding-top:3px;border-top:1px dashed rgba(88,166,255,0.2);">';
              for (var f of v.fields) {
                var isPtr = f.type_display && f.type_display.indexOf('*') >= 0;
                html += '<div style="display:flex;justify-content:space-between;font-size:9px;padding:1px 0;">';
                html += '<span style="color:var(--vscode-descriptionForeground,#8b949e);">.' + f.name + '</span>';
                html += '<span style="' + (isPtr ? 'color:#3fb950;' : '') + '">' + f.type_display + ' <span style="opacity:0.6;">(' + f.size + 'B @+' + f.offset + ')</span></span>';
                html += '</div>';
              }
              html += '</div>';
            }
            html += '</div>';
          }
          if (stackVars.length === 0 && droppedVars.length === 0) html += '<div style="opacity:0.5;font-size:11px;">Empty stack frame</div>';
          // Dropped variables
          if (droppedVars.length > 0) {
            html += '<div style="margin-top:6px;padding-top:6px;border-top:1px dashed var(--vscode-panel-border);font-size:10px;color:#f85149;margin-bottom:4px;">Dropped:</div>';
            for (var dv of droppedVars) {
              html += '<div style="padding:2px 8px;margin:2px 0;font-size:10px;opacity:0.4;text-decoration:line-through;">' + dv.name + ': ' + dv.type_display + ' (' + dv.size + 'B)</div>';
            }
          }
          html += '</div>';

          // Heap column
          html += '<div style="flex:1;border:1px solid #3fb950;border-radius:6px;padding:8px;overflow-y:auto;">';
          var heapSize = heapAllocs.reduce(function(s,h){return s+h.estimated_size;},0);
          html += '<div style="font-size:11px;color:#3fb950;font-weight:bold;margin-bottom:8px;">HEAP (' + heapSize + 'B)</div>';
          for (var h of heapAllocs) {
            html += '<div style="border-left:3px solid #3fb950;padding:4px 8px;margin:3px 0;border-radius:2px;font-size:11px;">';
            html += '<b>' + h.type_display + '</b> <span style="color:#8b949e;font-size:10px;">~' + h.estimated_size + 'B</span>';
            html += '<div style="font-size:9px;color:#8b949e;">owned by: ' + h.owner + '</div>';
            html += '</div>';
          }
          if (heapAllocs.length === 0) html += '<div style="opacity:0.5;font-size:11px;">No heap allocations</div>';

          // Pointers
          if (pointers.length > 0) {
            html += '<div style="margin-top:8px;padding-top:6px;border-top:1px solid var(--vscode-panel-border);font-size:10px;color:#d2a8ff;">';
            for (var p of pointers) { html += p.from + ' → ' + p.to + ' (' + p.kind + ')<br>'; }
            html += '</div>';
          }
          html += '</div>';
          html += '</div>';

          document.getElementById('mem-content').innerHTML = html;
          document.getElementById('mem-line-info').textContent = 'Line ' + line + ' / ' + maxLine;
        }

        // Container with timeline
        container.innerHTML = '<div id="mem-content" style="height:calc(100% - 44px);overflow:auto;"></div>' +
          '<div style="padding:6px 12px;border-top:1px solid var(--vscode-panel-border);display:flex;align-items:center;gap:8px;">' +
          '<button id="mem-play" style="background:var(--vscode-button-background);border:none;color:var(--vscode-button-foreground);padding:3px 10px;border-radius:3px;cursor:pointer;font-size:11px;">▶</button>' +
          '<button id="mem-step" style="background:var(--vscode-button-background);border:none;color:var(--vscode-button-foreground);padding:3px 8px;border-radius:3px;cursor:pointer;font-size:11px;">⏭</button>' +
          '<input id="mem-slider" type="range" min="' + minLine + '" max="' + maxLine + '" value="' + maxLine + '" style="flex:1;">' +
          '<span id="mem-line-info" style="font-size:11px;min-width:80px;">Line ' + maxLine + '</span>' +
          '</div>';

        renderAtLine(currentLine);

        // Slider
        document.getElementById('mem-slider').addEventListener('input', function() {
          currentLine = parseInt(this.value);
          renderAtLine(currentLine);
        });

        // Step button
        document.getElementById('mem-step').addEventListener('click', function() {
          if (currentLine < maxLine) { currentLine++; } else { currentLine = minLine; }
          document.getElementById('mem-slider').value = currentLine;
          renderAtLine(currentLine);
        });

        // Play button
        var memPlaying = false, memInterval;
        document.getElementById('mem-play').addEventListener('click', function() {
          if (memPlaying) {
            clearInterval(memInterval); this.textContent = '▶'; memPlaying = false;
          } else {
            memPlaying = true; this.textContent = '⏸';
            memInterval = setInterval(function() {
              currentLine++;
              if (currentLine > maxLine) currentLine = minLine;
              document.getElementById('mem-slider').value = currentLine;
              renderAtLine(currentLine);
            }, 600);
          }
        });
      }

      function renderVarBox(v) {
        const colors = { 'Owned':'#2ecc71','SharedRef':'#3498db','MutableRef':'#e74c3c','Rc':'#9b59b6','Arc':'#8e44ad','InteriorMut':'#e67e22','Copy':'#1abc9c','RawPointer':'#7f8c8d' };
        const color = colors[v.ownership_category] || '#95a5a6';
        return '<div class="scope-var" data-var="' + v.name + '" data-line="' + v.line + '" style="border-left:3px solid ' + color + ';padding:4px 8px;margin:4px 0 4px 8px;border-radius:2px;cursor:pointer;font-size:11px;">' +
          '<b>' + v.name + '</b>: <code style="font-size:10px;">' + v.type_display + '</code> ' +
          '<span style="color:' + color + ';font-size:10px;">[' + v.ownership_category + ']</span> ' +
          '<span style="color:#6c7086;font-size:10px;">line ' + v.line + '</span>' +
          '</div>';
      }

      // === Memory Runtime Mode: Actual Addresses ===
      function renderMemoryRuntime(container, rtMem) {
        var vars = rtMem.variables || [];
        var heapAllocs = rtMem.heap_allocations || [];
        var padding = rtMem.padding || [];
        var sp = rtMem.stack_pointer || '0x????';

        var html = '<div style="display:flex;gap:12px;padding:12px;height:calc(100% - 10px);font-family:monospace;">';

        // Stack column
        html += '<div style="flex:1;border:1px solid #58a6ff;border-radius:6px;padding:8px;overflow-y:auto;">';
        html += '<div style="font-size:11px;color:#58a6ff;font-weight:bold;margin-bottom:8px;">STACK <span style="font-size:9px;opacity:0.7;">(SP: '+sp+')</span></div>';

        for (var i = 0; i < vars.length; i++) {
          var v = vars[i];
          var shortAddr = v.addr.length > 10 ? '..'+v.addr.slice(-4) : v.addr;

          html += '<div style="border-left:3px solid #58a6ff;margin:4px 0;border-radius:3px;background:rgba(88,166,255,0.05);">';
          // Address + name header
          html += '<div style="display:flex;justify-content:space-between;padding:4px 8px;border-bottom:1px solid rgba(88,166,255,0.1);">';
          html += '<span style="font-size:9px;color:#58a6ff;">'+v.addr+'</span>';
          html += '<span style="font-size:10px;font-weight:bold;">'+v.name+': '+v.type+'</span>';
          html += '<span style="font-size:9px;color:var(--vscode-descriptionForeground,#8b949e);">'+v.size+'B</span>';
          html += '</div>';

          // Fields
          if (v.fields && v.fields.length > 0) {
            for (var f of v.fields) {
              var isPtr = f.name === 'ptr' && f.value.startsWith('0x');
              html += '<div style="padding:2px 8px 2px 16px;font-size:10px;display:flex;justify-content:space-between;">';
              html += '<span style="color:var(--vscode-descriptionForeground,#8b949e);">'+f.name+':</span>';
              if (isPtr) {
                html += '<span style="color:#3fb950;cursor:pointer;" title="Points to heap allocation">'+f.value+' →</span>';
              } else {
                html += '<span>'+f.value+'</span>';
              }
              html += '</div>';
            }
          }
          html += '</div>';

          // Padding after this variable
          var pad = padding.find(function(p){return p.after === v.name;});
          if (pad) {
            html += '<div style="margin:2px 0;padding:2px 8px;font-size:9px;color:#f85149;opacity:0.6;border-left:3px dashed #f85149;">';
            html += pad.addr + ' [padding: '+pad.bytes+'B]';
            html += '</div>';
          }
        }
        html += '</div>';

        // Heap column
        html += '<div style="flex:1;border:1px solid #3fb950;border-radius:6px;padding:8px;overflow-y:auto;">';
        html += '<div style="font-size:11px;color:#3fb950;font-weight:bold;margin-bottom:8px;">HEAP</div>';

        for (var h of heapAllocs) {
          var usedPct = h.capacity > 0 ? Math.round((h.used / h.capacity) * 100) : 100;
          html += '<div style="border-left:3px solid #3fb950;margin:4px 0;border-radius:3px;background:rgba(63,185,80,0.05);">';
          // Address header
          html += '<div style="display:flex;justify-content:space-between;padding:4px 8px;border-bottom:1px solid rgba(63,185,80,0.1);">';
          html += '<span style="font-size:9px;color:#3fb950;">'+h.addr+'</span>';
          html += '<span style="font-size:10px;font-weight:bold;">'+h.content+'</span>';
          html += '<span style="font-size:9px;color:var(--vscode-descriptionForeground,#8b949e);">'+h.size+'B</span>';
          html += '</div>';
          // Details
          html += '<div style="padding:3px 8px;font-size:10px;">';
          html += '<span style="color:var(--vscode-descriptionForeground,#8b949e);">owner: </span><span>'+h.owner+'</span>';
          html += '<span style="margin-left:12px;color:var(--vscode-descriptionForeground,#8b949e);">used: </span><span>'+h.used+'/'+h.capacity+'</span>';
          html += '</div>';
          // Capacity bar
          html += '<div style="margin:3px 8px 4px;height:4px;background:rgba(63,185,80,0.15);border-radius:2px;overflow:hidden;">';
          html += '<div style="width:'+usedPct+'%;height:100%;background:#3fb950;border-radius:2px;"></div>';
          html += '</div>';
          html += '</div>';
        }

        if (heapAllocs.length === 0) html += '<div style="opacity:0.5;font-size:11px;">No heap allocations</div>';
        html += '</div>';

        html += '</div>';

        // Mode indicator
        html = '<div style="padding:4px 12px;border-bottom:1px solid var(--vscode-panel-border);font-size:10px;display:flex;align-items:center;gap:8px;">' +
          '<span style="color:#3fb950;">● Runtime Mode</span>' +
          '<span style="color:var(--vscode-descriptionForeground,#8b949e);">Showing actual addresses from program execution</span>' +
          '</div>' + html;

        container.innerHTML = html;
      }

      // === Runtime View (Timeline + Drop Order + Ref Count + Divergences + Events) ===
      function renderRuntime() {
        const container = document.getElementById('runtime-container');
        const events = rawGraph._runtimeEvents || [];
        const variables = rawGraph.variables || [];

        if (events.length === 0) {
          container.innerHTML = '<div style="padding:20px;text-align:center;opacity:0.5;"><p>No runtime events loaded.</p><p style="font-size:11px;">Enable runtime overlay and run your instrumented program.<br>Events file: <code>.borrowscope/events.json</code></p></div>';
          return;
        }

        // Parse events into usable data
        var rtVars = [];
        var drops = [];
        var rcPoints = [];
        var creationTimes = {};
        var varNames = {};
        var rcSource = null;
        var strongCount = 0;

        for (var e of events) {
          var type = e.type || Object.keys(e)[0];
          var data = e.type ? e : e[type];
          if (type === 'New' || type === 'RcNew' || type === 'ArcNew' || type === 'BoxNew' || type === 'RefCellNew' || type === 'CellNew') {
            creationTimes[data.var_id] = data.timestamp;
            varNames[data.var_id] = data.var_name;
            var staticMatch = variables.find(function(sv){return sv.name === data.var_name;});
            var varLine = staticMatch ? staticMatch.line : (rawGraph.start_line ? rawGraph.start_line + rtVars.length + 1 : 0);
            rtVars.push({ id: data.var_id, name: data.var_name, type: data.type_name || type, start: data.timestamp, end: null, moved: false, borrows: [], line: varLine });
          }
          if (type === 'Drop' && data.var_id) {
            var rv = rtVars.find(function(v){return v.id===data.var_id;});
            if (rv) rv.end = data.timestamp;
            drops.push({ var_id: data.var_id, name: varNames[data.var_id] || data.var_id, timestamp: data.timestamp, order: drops.length + 1 });
          }
          if (type === 'Move') {
            var mv = rtVars.find(function(v){return v.id===data.from_id;});
            if (mv) { mv.moved = true; mv.end = data.timestamp; mv.moveTo = data.to_name; }
          }
          if (type === 'Borrow') {
            var bv = rtVars.find(function(v){return v.id===data.owner_id;});
            if (bv) bv.borrows.push({ name: data.borrower_name, start: data.timestamp, mut: data.mutable });
          }
          if (type === 'RcNew' || type === 'ArcNew') {
            rcSource = data.var_id; strongCount = data.strong_count || 1;
            rcPoints.push({ t: data.timestamp, s: strongCount, w: data.weak_count||0, label: type+'('+data.var_name+')' });
          }
          if (type === 'RcClone' || type === 'ArcClone') {
            strongCount = data.strong_count || strongCount+1;
            rcPoints.push({ t: data.timestamp, s: strongCount, w: data.weak_count||0, label: 'clone('+data.var_name+')' });
          }
        }

        // Compute time range
        var minT = rtVars.length > 0 ? Math.min.apply(null, rtVars.map(function(v){return v.start;})) : 0;
        var maxT = Math.max.apply(null, rtVars.map(function(v){return v.end||v.start+100;}).concat(drops.map(function(d){return d.timestamp;})));
        var timeRange = maxT - minT || 1;

        // Build sub-tabs
        var html = '<div style="display:flex;gap:2px;padding:8px 12px;border-bottom:1px solid var(--vscode-panel-border);">';
        html += '<button class="rt-tab active" data-rt="tl" style="padding:3px 10px;border:1px solid var(--vscode-panel-border,#3c3c3c);background:var(--vscode-editor-background,#1e1e1e);color:#58a6ff;border-radius:3px;cursor:pointer;font-size:11px;">⏱ Timeline</button>';
        html += '<button class="rt-tab" data-rt="do" style="padding:3px 10px;border:1px solid var(--vscode-panel-border,#3c3c3c);background:var(--vscode-input-background,#2d2d2d);color:var(--vscode-descriptionForeground,#8b949e);border-radius:3px;cursor:pointer;font-size:11px;">💀 Drop Order</button>';
        html += '<button class="rt-tab" data-rt="rc" style="padding:3px 10px;border:1px solid var(--vscode-panel-border,#3c3c3c);background:var(--vscode-input-background,#2d2d2d);color:var(--vscode-descriptionForeground,#8b949e);border-radius:3px;cursor:pointer;font-size:11px;">🔗 Ref Count</button>';
        html += '<button class="rt-tab" data-rt="ev" style="padding:3px 10px;border:1px solid var(--vscode-panel-border,#3c3c3c);background:var(--vscode-input-background,#2d2d2d);color:var(--vscode-descriptionForeground,#8b949e);border-radius:3px;cursor:pointer;font-size:11px;">📋 Events</button>';
        html += '<span style="margin-left:auto;font-size:10px;color:#3fb950;">' + events.length + ' events</span>';
        html += '</div>';

        // Timeline sub-view
        html += '<div id="rt-tl" style="padding:12px;">';
        html += '<div style="margin-bottom:8px;display:flex;align-items:center;gap:8px;"><button id="rt-play-btn" style="background:var(--vscode-button-secondaryBackground,#3c3c3c);border:1px solid var(--vscode-panel-border,#555);color:var(--vscode-editor-foreground,#d4d4d4);padding:3px 10px;border-radius:3px;cursor:pointer;font-size:11px;">▶ Play</button>';
        html += '<input type="range" id="rt-slider" min="0" max="100" value="100" style="flex:1;"><span id="rt-time" style="font-size:10px;min-width:60px;">100%</span></div>';
        for (var i = 0; i < rtVars.length && i < 12; i++) {
          var v = rtVars[i];
          var lifeEnd = v.end || maxT;
          var x1pct = ((v.start - minT) / timeRange * 100).toFixed(1);
          var wpct = (((lifeEnd - v.start) / timeRange) * 100).toFixed(1);
          var color = v.moved ? '#e67e22' : (v.type||'').indexOf('Rc')>=0 ? '#a371f7' : '#2ecc71';
          html += '<div class="rt-bar" data-start="'+x1pct+'" data-width="'+wpct+'" data-line="'+v.line+'" style="display:flex;align-items:center;margin:3px 0;cursor:pointer;">';
          html += '<span style="width:80px;font-size:10px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;" title="'+v.name+': '+v.type+'">'+v.name+'</span>';
          html += '<div style="flex:1;height:18px;background:var(--vscode-input-background,#2d2d2d);border-radius:3px;position:relative;overflow:hidden;">';
          html += '<div class="rt-life" style="position:absolute;left:'+x1pct+'%;width:'+wpct+'%;height:100%;background:'+color+';opacity:0.3;border-radius:3px;transition:width 0.3s;"></div>';
          html += '<div style="position:absolute;left:'+x1pct+'%;width:'+wpct+'%;height:100%;border:1px solid '+color+';border-radius:3px;box-sizing:border-box;"></div>';
          // Borrow regions
          for (var b of v.borrows) {
            var bx = ((b.start - minT) / timeRange * 100).toFixed(1);
            var bw = '5';
            var bc = b.mut ? '#f85149' : '#58a6ff';
            html += '<div style="position:absolute;left:'+bx+'%;width:'+bw+'%;height:100%;background:'+bc+';opacity:0.5;border-radius:2px;"></div>';
          }
          html += '</div>';
          html += '<span style="width:60px;font-size:9px;color:var(--vscode-descriptionForeground,#8b949e);text-align:right;">'+(v.moved?'→'+v.moveTo:(v.end?((v.end-v.start)+'ns'):'alive'))+'</span>';
          html += '</div>';
        }
        html += '</div>';

        // Drop Order sub-view
        html += '<div id="rt-do" style="display:none;padding:12px;">';
        if (drops.length === 0) { html += '<div style="opacity:0.5;font-size:11px;">No drops recorded</div>'; }
        for (var d of drops) {
          var lifetime = creationTimes[d.var_id] ? (d.timestamp - creationTimes[d.var_id]) + 'ns' : '?';
          var dline = 0;
          var dsv = variables.find(function(v){return v.name===d.name;});
          if (dsv) dline = dsv.line;
          if (!dline) { var drv = rtVars.find(function(v){return v.id===d.var_id;}); if (drv) dline = drv.line; }
          html += '<div class="rt-drop-item" data-drop-line="'+dline+'" style="display:flex;align-items:center;gap:8px;margin:4px 0;padding:4px 8px;border-radius:3px;background:var(--vscode-input-background,#2d2d2d);cursor:pointer;">';
          html += '<span style="width:28px;height:28px;border-radius:50%;background:rgba(63,185,80,0.15);border:1px solid #3fb950;display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:bold;color:#3fb950;">#'+d.order+'</span>';
          html += '<span style="font-size:11px;font-weight:bold;">'+d.name+'</span>';
          html += '<span style="font-size:10px;color:var(--vscode-descriptionForeground,#8b949e);margin-left:auto;">lifetime: '+lifetime+'</span>';
          html += '</div>';
        }
        html += '</div>';

        // Ref Count sub-view
        html += '<div id="rt-rc" style="display:none;padding:12px;">';
        if (rcPoints.length === 0) { html += '<div style="opacity:0.5;font-size:11px;">No Rc/Arc events</div>'; }
        else {
          var maxS = Math.max.apply(null, rcPoints.map(function(p){return p.s;}));
          html += '<svg width="100%" height="180" viewBox="0 0 600 180">';
          // Grid
          for (var c = 0; c <= maxS; c++) {
            var gy = 160 - (c / (maxS||1)) * 140;
            html += '<line x1="40" y1="'+gy+'" x2="580" y2="'+gy+'" stroke="#333" stroke-dasharray="2,4"/>';
            html += '<text x="35" y="'+(gy+4)+'" text-anchor="end" fill="#6c7086" font-size="10">'+c+'</text>';
          }
          // Line
          var rcMinT = rcPoints[0].t;
          var rcMaxT = rcPoints[rcPoints.length-1].t;
          var rcRange = rcMaxT - rcMinT || 1;
          var path = 'M 40 ' + (160 - (rcPoints[0].s/(maxS||1))*140);
          for (var pi = 1; pi < rcPoints.length; pi++) {
            var px = 40 + ((rcPoints[pi].t - rcMinT) / rcRange) * 540;
            var py1 = 160 - (rcPoints[pi-1].s/(maxS||1))*140;
            var py2 = 160 - (rcPoints[pi].s/(maxS||1))*140;
            path += ' L '+px+' '+py1+' L '+px+' '+py2;
          }
          html += '<path d="'+path+'" fill="none" stroke="#58a6ff" stroke-width="2.5"/>';
          // Dots
          for (var pi = 0; pi < rcPoints.length; pi++) {
            var px = 40 + ((rcPoints[pi].t - rcMinT) / rcRange) * 540;
            var py = 160 - (rcPoints[pi].s/(maxS||1))*140;
            html += '<circle cx="'+px+'" cy="'+py+'" r="5" fill="#58a6ff" stroke="#1e1e1e" stroke-width="2"/>';
            html += '<text x="'+px+'" y="175" text-anchor="middle" fill="#8b949e" font-size="8">'+rcPoints[pi].label+'</text>';
          }
          html += '</svg>';
        }
        html += '</div>';

        // Events sub-view
        html += '<div id="rt-ev" style="display:none;padding:12px;font-size:11px;max-height:calc(100vh - 180px);overflow-y:auto;">';
        var typeColors = {New:'#2ecc71',Drop:'#e67e22',Borrow:'#58a6ff',Move:'#d2a8ff',RcNew:'#a371f7',RcClone:'#a371f7',ArcNew:'#a371f7',ArcClone:'#a371f7',FnEnter:'#6c7086',FnExit:'#6c7086',RefCellBorrow:'#f0883e',RefCellDrop:'#f0883e',CellNew:'#f0883e',CellGet:'#f0883e',CellSet:'#f0883e',UnsafeBlockEnter:'#f85149',UnsafeBlockExit:'#f85149',RawPtrCreated:'#f85149',RawPtrDeref:'#f85149',WeakNew:'#a371f7',WeakUpgrade:'#a371f7',BoxNew:'#2ecc71'};
        for (var ei = 0; ei < events.length; ei++) {
          var etype = events[ei].type || Object.keys(events[ei])[0];
          var edata = events[ei].type ? events[ei] : events[ei][etype];
          var ec = typeColors[etype] || '#6c7086';
          var eline = 0;
          if (edata.location) { var lm = String(edata.location).match(/:(\d+):/); if (lm) eline = parseInt(lm[1]); }
          if (!eline) {
            var ename = edata.var_name || edata.borrower_name || edata.to_name || edata.fn_name || '';
            if (ename) { var sv = variables.find(function(v){return v.name===ename;}); if (sv) eline = sv.line; }
          }
          if (!eline && edata.var_id) {
            var rv = rtVars.find(function(v){return v.id===edata.var_id;});
            if (rv) eline = rv.line;
          }
          if (!eline && edata.owner_id) {
            var ov = rtVars.find(function(v){return v.id===edata.owner_id;});
            if (ov) eline = ov.line;
          }
          if (!eline && edata.from_id) {
            var fv = rtVars.find(function(v){return v.id===edata.from_id;});
            if (fv) eline = fv.line;
          }
          html += '<div data-evt-line="'+eline+'" style="display:flex;gap:8px;padding:2px 0;border-bottom:1px solid var(--vscode-panel-border,#2d2d2d);cursor:pointer;">';
          html += '<span style="color:#6c7086;min-width:50px;">'+edata.timestamp+'ns</span>';
          html += '<span style="background:'+ec+';opacity:0.8;color:var(--vscode-editor-background,#1e1e1e);padding:0 5px;border-radius:2px;font-size:10px;font-weight:bold;min-width:60px;text-align:center;">'+etype+'</span>';
          html += '<span style="color:var(--vscode-editor-foreground,#d4d4d4);">'+(edata.var_name||edata.fn_name||edata.borrower_name||edata.from_id||edata.borrow_id||edata.block_id||edata.ptr_id||'')+'</span>';
          html += '</div>';
        }
        html += '</div>';

        container.innerHTML = html;

        // Sub-tab switching
        container.querySelectorAll('.rt-tab').forEach(function(btn) {
          btn.addEventListener('click', function() {
            var id = this.getAttribute('data-rt');
            ['tl','do','rc','ev'].forEach(function(t){document.getElementById('rt-'+t).style.display=t===id?'':'none';});
            container.querySelectorAll('.rt-tab').forEach(function(b){b.style.background='var(--vscode-input-background,#2d2d2d)';b.style.color='var(--vscode-descriptionForeground,#8b949e)';});
            this.style.background='var(--vscode-editor-background,#1e1e1e)';this.style.color='#58a6ff';
          });
        });

        // Click timeline bars to navigate to source line
        container.querySelectorAll('.rt-bar').forEach(function(bar) {
          bar.addEventListener('mouseover', function() { this.style.background='rgba(88,166,255,0.08)'; });
          bar.addEventListener('mouseout', function() { this.style.background=''; });
          bar.addEventListener('click', function() {
            var line = parseInt(this.getAttribute('data-line'));
            if (line > 0 && vscodeApi) {
              vscodeApi.postMessage({ type: 'nodeClicked', line: line });
              // Flash feedback
              this.style.background='rgba(88,166,255,0.2)';
              var self = this;
              setTimeout(function(){ self.style.background=''; }, 300);
            }
          });
        });

        // Click event rows to navigate
        container.querySelectorAll('[data-evt-line]').forEach(function(row) {
          row.addEventListener('mouseover', function() { this.style.background='rgba(88,166,255,0.08)'; });
          row.addEventListener('mouseout', function() { this.style.background=''; });
          row.addEventListener('click', function() {
            var line = parseInt(this.getAttribute('data-evt-line'));
            if (line > 0 && vscodeApi) {
              vscodeApi.postMessage({ type: 'nodeClicked', line: line });
              this.style.background='rgba(88,166,255,0.2)';
              var self = this;
              setTimeout(function(){ self.style.background=''; }, 300);
            }
          });
        });

        // Click drop order items to navigate
        container.querySelectorAll('.rt-drop-item').forEach(function(item) {
          item.addEventListener('mouseover', function() { this.style.background='rgba(63,185,80,0.1)'; });
          item.addEventListener('mouseout', function() { this.style.background='var(--vscode-input-background,#2d2d2d)'; });
          item.addEventListener('click', function() {
            var line = parseInt(this.getAttribute('data-drop-line'));
            if (line > 0 && vscodeApi) {
              vscodeApi.postMessage({ type: 'nodeClicked', line: line });
              this.style.background='rgba(63,185,80,0.2)';
              var self = this;
              setTimeout(function(){ self.style.background='var(--vscode-input-background,#2d2d2d)'; }, 300);
            }
          });
        });

        // Play/scrub
        var rtPlaying = false, rtInterval;
        function rtScrub(pct) {
          document.getElementById('rt-time').textContent = pct + '%';
          container.querySelectorAll('.rt-life').forEach(function(bar){
            var startPct = parseFloat(bar.parentElement.parentElement.getAttribute('data-start')||0);
            var fullW = parseFloat(bar.parentElement.parentElement.getAttribute('data-width')||0);
            var visibleW = Math.max(0, Math.min(fullW, pct - startPct));
            bar.style.width = visibleW + '%';
          });
        }
        document.getElementById('rt-slider').addEventListener('input', function() { rtScrub(parseInt(this.value)); });
        document.getElementById('rt-play-btn').addEventListener('click', function() {
          var btn = this;
          if (rtPlaying) { clearInterval(rtInterval); rtPlaying=false; btn.textContent='▶ Play'; return; }
          rtPlaying=true; btn.textContent='⏸ Pause';
          document.getElementById('rt-slider').value = 0;
          rtScrub(0);
          rtInterval = setInterval(function(){
            var val = parseInt(document.getElementById('rt-slider').value) + 2;
            if (val > 100) { val=100; clearInterval(rtInterval); rtPlaying=false; btn.textContent='▶ Play'; }
            document.getElementById('rt-slider').value = val;
            rtScrub(val);
          }, 60);
        });
      }

      // Auto-restore last view if set
      if (lastView) { showView(lastView); }
    })();
  </script>
</body>
</html>`;
function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
//# sourceMappingURL=panel.js.map