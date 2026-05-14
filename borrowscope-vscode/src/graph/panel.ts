import * as vscode from "vscode";
import { buildGraphModel, OwnershipGraphData } from "./model";

function buildGraphModelFromRaw(graph: any): any {
  const data: OwnershipGraphData = {
    function_name: graph.function_name || "",
    variables: (graph.variables || []).map((v: any) => ({
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
  return buildGraphModel(data);
}

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

  public static createOrShow(extensionUri: vscode.Uri, graph?: any, functionList?: string[]): void {
    const column = vscode.ViewColumn.Beside;

    if (GraphPanel.currentPanel) {
      GraphPanel.currentPanel._panel.reveal(column);
      if (graph) GraphPanel.currentPanel.updateGraph(graph, functionList);
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
      GraphPanel.currentPanel.updateGraph(graph, functionList);
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

    // Handle messages from WebView
    this._panel.webview.onDidReceiveMessage(
      (message) => {
        if (message.type === "nodeClicked" && message.line > 0) {
          const editor = vscode.window.visibleTextEditors.find(
            (e) => e.document.languageId === "rust"
          );
          if (editor) {
            const pos = new vscode.Position(message.line - 1, 0);
            editor.selection = new vscode.Selection(pos, pos);
            editor.revealRange(
              new vscode.Range(pos, pos),
              vscode.TextEditorRevealType.InCenter
            );
            vscode.window.showTextDocument(editor.document, editor.viewColumn);
          }
        } else if (message.type === "selectFunction" && message.name) {
          this._loadFunction(message.name);
        }
      },
      null,
      this._disposables
    );
  }

  private async _loadFunction(name: string): Promise<void> {
    const editor = vscode.window.visibleTextEditors.find(
      (e) => e.document.languageId === "rust"
    );
    if (!editor) return;

    for (let i = 0; i < editor.document.lineCount; i++) {
      if (new RegExp(`\\bfn\\s+${name}\\b`).test(editor.document.lineAt(i).text)) {
        try {
          const { getClient } = require("../client"); const client = getClient();
          if (!client) return;
          const graph = await client.sendRequest("borrowscope/ownershipGraph", {
            textDocument: { uri: editor.document.uri.toString() },
            position: { line: i, character: 4 },
          });
          if (graph) {
            // Get function list from editor
            const fnList: string[] = [];
            for (let j = 0; j < editor.document.lineCount; j++) {
              const match = editor.document.lineAt(j).text.match(/\bfn\s+(\w+)/);
              if (match) fnList.push(match[1]);
            }
            this.updateGraph(graph, fnList);
          }
        } catch { /* ignore */ }
        break;
      }
    }
  }

  public updateGraph(graph: any, functionList?: string[]): void {
    this._currentGraph = graph;
    this._panel.webview.html = this._buildHtml(graph, functionList);
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

  private _buildHtml(graph: any | undefined, functionList?: string[]): string {
    if (!graph) {
      return `<!DOCTYPE html><html><head><meta charset="UTF-8">
<style>body{font-family:var(--vscode-font-family);background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);padding:16px;}</style>
</head><body><h2>BorrowScope: Ownership Graph</h2><p>Waiting for data... Click a CodeLens to load a function.</p></body></html>`;
    }

    const d3Uri = this._panel.webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, "media", "d3.min.js")
    );

    const vars = graph.variables || [];
    const scopes = graph.borrow_scopes || [];
    const moves = graph.moves || [];
    const clones = graph.rc_clones || [];
    const conflicts = graph.conflicts || [];

    const graphModel = buildGraphModelFromRaw(graph);
    const graphJson = JSON.stringify(graphModel).replace(/</g, "\\u003c");

    let varsHtml = vars.map((v: any) =>
      `<tr data-var="${esc(v.name)}"><td><b>${esc(v.name)}</b></td><td><code>${esc(v.type_display)}</code></td><td>${esc(v.ownership_category)}</td></tr>`
    ).join("");
    let scopesHtml = scopes.map((s: any) =>
      `<tr data-var="${esc(s.borrower_name)}"><td>${esc(s.borrower_name)}</td><td>${s.is_mutable ? "&amp;mut" : "&amp;"}</td><td>${esc(s.target_name)}</td><td>${s.start_line}-${s.end_line}</td></tr>`
    ).join("");
    let movesHtml = moves.map((m: any) =>
      `<tr data-var="${esc(m.source_name)}"><td>${esc(m.source_name)}</td><td>↦</td><td>${esc(JSON.stringify(m.destination))}</td><td>${m.line}</td></tr>`
    ).join("");
    let clonesHtml = clones.map((c: any) =>
      `<tr data-var="${esc(c.clone_variable)}"><td>${esc(c.clone_variable)}</td><td>🔗</td><td>${esc(c.source_variable)}</td><td>${c.line}</td></tr>`
    ).join("");
    let conflictsHtml = conflicts.map((c: any) =>
      `<tr data-var="${esc(c.borrow_a)}"><td>⚠️ ${esc(c.borrow_a)}</td><td>&amp;</td><td>${esc(c.borrow_b)}</td><td>${c.overlap_start_line}-${c.overlap_end_line}</td></tr>`
    ).join("");

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>BorrowScope: ${esc(graph.function_name)}</title>
  <style>
    body { margin:0; padding:0; font-family:var(--vscode-font-family); background:var(--vscode-editor-background); color:var(--vscode-editor-foreground); overflow:hidden; }
    #header { padding:8px 16px; border-bottom:1px solid var(--vscode-panel-border); font-size:13px; }
    #header h2 { margin:0; font-size:14px; }
    #header .stats { opacity:0.7; font-size:12px; }
    #graph-container { width:100%; height:45vh; border-bottom:1px solid var(--vscode-panel-border); }
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
  </style>
</head>
<body>
  <div id="header">
    <h2>📊 ${esc(graph.function_name)}</h2>
    <span class="stats">${(graph.variables||[]).length} variables, ${(graph.borrow_scopes||[]).length} borrows, ${(graph.moves||[]).length} moves</span>
    <select id="fn-selector" style="float:right;background:var(--vscode-dropdown-background);color:var(--vscode-dropdown-foreground);border:1px solid var(--vscode-dropdown-border);border-radius:3px;padding:2px 6px;font-size:12px;">
      ${(functionList || [graph.function_name]).map((fn: string) =>
        `<option value="${esc(fn)}"${fn === graph.function_name ? " selected" : ""}>${esc(fn)}</option>`
      ).join("")}
    </select>
  </div>
  <div id="filter-bar"><span class="filter-label">Filter:</span></div>
  <div id="graph-container"></div>
  <div id="tooltip"></div>
  <div id="tables" style="padding:16px;overflow-y:auto;max-height:40vh;">
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
        .force('link', d3.forceLink(data.edges).id(d => d.id).distance(100))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(40));

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

      // Table row hover -> highlight graph
      document.querySelectorAll('#tables tr[data-var]').forEach(row => {
        row.addEventListener('mouseover', () => highlightVariable(row.getAttribute('data-var')));
        row.addEventListener('mouseout', () => clearHighlight());
      });
    })();
  </script>
</body>
</html>`;
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
