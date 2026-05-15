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
    const rawGraphJson = JSON.stringify(graph).replace(/</g, "\\u003c");

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
    <div id="view-toggle" style="float:right;display:flex;gap:4px;">
      <button class="view-btn active" data-view="graph" style="padding:2px 8px;border:1px solid var(--vscode-button-border,#454545);background:var(--vscode-button-background);color:var(--vscode-button-foreground);border-radius:3px;cursor:pointer;font-size:11px;">Graph</button>
      <button class="view-btn" data-view="timeline" style="padding:2px 8px;border:1px solid var(--vscode-button-border,#454545);background:transparent;color:var(--vscode-foreground);border-radius:3px;cursor:pointer;font-size:11px;">Timeline</button>
      <button class="view-btn" data-view="scopes" style="padding:2px 8px;border:1px solid var(--vscode-button-border,#454545);background:transparent;color:var(--vscode-foreground);border-radius:3px;cursor:pointer;font-size:11px;">Scopes</button>
      <button class="view-btn" data-view="refcount" style="padding:2px 8px;border:1px solid var(--vscode-button-border,#454545);background:transparent;color:var(--vscode-foreground);border-radius:3px;cursor:pointer;font-size:11px;">RefCount</button>
    </div>
  </div>
  <div id="filter-bar"><span class="filter-label">Filter:</span></div>
  <div id="graph-container"></div>
  <div id="timeline-container" style="display:none;width:100%;height:45vh;overflow:auto;"></div>
  <div id="scopes-container" style="display:none;width:100%;height:45vh;overflow:auto;padding:12px;"></div>
  <div id="refcount-container" style="display:none;width:100%;height:45vh;overflow:auto;"></div>
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
        .force('link', d3.forceLink(data.edges).id(d => d.id).distance(60))
        .force('charge', d3.forceManyBody().strength(-120))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(25));

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
          document.getElementById('graph-container').style.display = view === 'graph' ? '' : 'none';
          document.getElementById('timeline-container').style.display = view === 'timeline' ? '' : 'none';
          document.getElementById('scopes-container').style.display = view === 'scopes' ? '' : 'none';
          document.getElementById('refcount-container').style.display = view === 'refcount' ? '' : 'none';
          if (view === 'timeline') renderTimeline();
          if (view === 'scopes') renderScopes();
          if (view === 'refcount') renderRefCount();
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

      function renderVarBox(v) {
        const colors = { 'Owned':'#2ecc71','SharedRef':'#3498db','MutableRef':'#e74c3c','Rc':'#9b59b6','Arc':'#8e44ad','InteriorMut':'#e67e22','Copy':'#1abc9c','RawPointer':'#7f8c8d' };
        const color = colors[v.ownership_category] || '#95a5a6';
        return '<div class="scope-var" data-var="' + v.name + '" data-line="' + v.line + '" style="border-left:3px solid ' + color + ';padding:4px 8px;margin:4px 0 4px 8px;border-radius:2px;cursor:pointer;font-size:11px;">' +
          '<b>' + v.name + '</b>: <code style="font-size:10px;">' + v.type_display + '</code> ' +
          '<span style="color:' + color + ';font-size:10px;">[' + v.ownership_category + ']</span> ' +
          '<span style="color:#6c7086;font-size:10px;">line ' + v.line + '</span>' +
          '</div>';
      }
    })();
  </script>
</body>
</html>`;
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
