import * as vscode from "vscode";

export class CrossRefPanel {
  public static currentPanel: CrossRefPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private _extensionUri: vscode.Uri;

  public static createOrShow(extensionUri: vscode.Uri, data: any, d3Uri: vscode.Uri): void {
    const column = vscode.ViewColumn.Beside;
    if (CrossRefPanel.currentPanel) {
      CrossRefPanel.currentPanel._panel.reveal(column);
      CrossRefPanel.currentPanel._update(data, d3Uri);
      return;
    }
    const panel = vscode.window.createWebviewPanel(
      "borrowscopeCrossRef", "BorrowScope: Cross-References", column,
      { enableScripts: true, retainContextWhenHidden: true, localResourceRoots: [vscode.Uri.joinPath(extensionUri, "media")] }
    );
    CrossRefPanel.currentPanel = new CrossRefPanel(panel, extensionUri);
    CrossRefPanel.currentPanel._update(data, d3Uri);
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;
    this._panel.onDidDispose(() => { CrossRefPanel.currentPanel = undefined; });
  }

  private _update(data: any, d3Uri: vscode.Uri): void {
    this._panel.webview.html = this._buildHtml(data, d3Uri);
  }

  private _buildHtml(data: any, d3Uri: vscode.Uri): string {
    const crossBorrows = data.cross_borrows || [];
    const currentFile = data.current_file || "main.rs";
    const functionName = data.function_name || "";

    // Build file tree from cross-references
    const fileSet = new Map<string, { count: number; isCurrentFile: boolean }>();
    fileSet.set(currentFile, { count: 0, isCurrentFile: true });
    for (const b of crossBorrows) {
      for (const seg of b.path) {
        const file = seg.file || currentFile;
        if (!fileSet.has(file)) fileSet.set(file, { count: 0, isCurrentFile: false });
        if (file !== currentFile) fileSet.get(file)!.count++;
      }
    }

    // Build graph nodes (functions) and edges (borrow flows)
    const nodes: any[] = [];
    const edges: any[] = [];
    const nodeSet = new Set<string>();

    for (const b of crossBorrows) {
      for (const seg of b.path) {
        if (!nodeSet.has(seg.function_name)) {
          nodeSet.add(seg.function_name);
          nodes.push({ id: seg.function_name, file: seg.file || currentFile, variable: seg.variable, isMutable: seg.is_mutable, kind: seg.kind });
        }
      }
      if (b.path.length >= 2) {
        edges.push({ source: b.path[0].function_name, target: b.path[1].function_name, variable: b.origin_variable, isMutable: b.path[1].is_mutable });
      }
    }

    const graphJson = JSON.stringify({ nodes, edges }).replace(/</g, "\\u003c");
    const filesJson = JSON.stringify([...fileSet.entries()].map(([f, info]) => ({ name: f, ...info }))).replace(/</g, "\\u003c");

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <style>
    body { margin:0; font-family:var(--vscode-font-family); background:var(--vscode-editor-background); color:var(--vscode-editor-foreground); display:flex; height:100vh; overflow:hidden; }
    #file-tree { width:180px; border-right:1px solid var(--vscode-panel-border); padding:8px; overflow-y:auto; font-size:12px; }
    #file-tree h3 { font-size:11px; color:var(--vscode-descriptionForeground); margin:0 0 8px; text-transform:uppercase; }
    .file-item { padding:3px 6px; border-radius:3px; margin:2px 0; cursor:pointer; display:flex; align-items:center; gap:4px; }
    .file-item.current { background:rgba(88,166,255,0.15); font-weight:bold; }
    .file-item.referenced { opacity:1; }
    .file-item.faded { opacity:0.35; }
    .file-icon { width:8px; height:8px; border-radius:50%; }
    .file-icon.current { background:#58a6ff; }
    .file-icon.referenced { background:#3fb950; }
    .file-icon.faded { background:#484f58; }
    .badge { background:var(--vscode-badge-background); color:var(--vscode-badge-foreground); border-radius:8px; padding:0 5px; font-size:10px; margin-left:auto; }
    #graph-area { flex:1; display:flex; flex-direction:column; }
    #graph-header { padding:8px 12px; border-bottom:1px solid var(--vscode-panel-border); font-size:13px; }
    #graph-container { flex:1; }
    svg { width:100%; height:100%; }
    .node circle { stroke-width:2px; cursor:pointer; }
    .node text { font-size:11px; fill:var(--vscode-editor-foreground); }
    .node .param { font-size:9px; fill:var(--vscode-descriptionForeground); }
    .edge path { fill:none; }
    .edge text { font-size:9px; fill:var(--vscode-descriptionForeground); }
    #legend { padding:6px 12px; border-top:1px solid var(--vscode-panel-border); font-size:10px; color:var(--vscode-descriptionForeground); }
  </style>
</head>
<body>
  <div id="file-tree">
    <h3>Files</h3>
    <div id="files"></div>
  </div>
  <div id="graph-area">
    <div id="graph-header"><b>Cross-References:</b> ${esc(functionName)}()</div>
    <div id="graph-container"></div>
    <div id="legend">── shared borrow &nbsp;&nbsp; ━━ mutable borrow &nbsp;&nbsp; Nodes = functions</div>
  </div>
  <script src="${d3Uri}"></script>
  <script>
    (function() {
      const graph = ${graphJson};
      const files = ${filesJson};

      // Render file tree
      const fileContainer = document.getElementById('files');
      files.forEach(f => {
        const cls = f.isCurrentFile ? 'current' : (f.count > 0 ? 'referenced' : 'faded');
        const iconCls = f.isCurrentFile ? 'current' : (f.count > 0 ? 'referenced' : 'faded');
        const badge = f.count > 0 ? '<span class="badge">' + f.count + '</span>' : '';
        fileContainer.innerHTML += '<div class="file-item ' + cls + '"><div class="file-icon ' + iconCls + '"></div>' + f.name.split('/').pop() + badge + '</div>';
      });

      // Render graph
      const container = document.getElementById('graph-container');
      const width = container.clientWidth || 500;
      const height = container.clientHeight || 350;
      const svg = d3.select('#graph-container').append('svg').attr('width', width).attr('height', height);
      const g = svg.append('g');
      svg.call(d3.zoom().on('zoom', (event) => g.attr('transform', event.transform)));

      // Arrow markers
      const defs = svg.append('defs');
      defs.append('marker').attr('id','arr-shared').attr('viewBox','0 -5 10 10').attr('refX',30).attr('markerWidth',6).attr('markerHeight',6).attr('orient','auto')
        .append('path').attr('d','M0,-5L10,0L0,5').attr('fill','#3498db');
      defs.append('marker').attr('id','arr-mut').attr('viewBox','0 -5 10 10').attr('refX',30).attr('markerWidth',6).attr('markerHeight',6).attr('orient','auto')
        .append('path').attr('d','M0,-5L10,0L0,5').attr('fill','#e74c3c');

      const simulation = d3.forceSimulation(graph.nodes)
        .force('link', d3.forceLink(graph.edges).id(d => d.id).distance(120))
        .force('charge', d3.forceManyBody().strength(-200))
        .force('center', d3.forceCenter(width/2, height/2))
        .force('collision', d3.forceCollide().radius(40));

      const edge = g.selectAll('.edge').data(graph.edges).join('g').attr('class','edge');
      const edgePath = edge.append('path')
        .attr('stroke', d => d.isMutable ? '#e74c3c' : '#3498db')
        .attr('stroke-width', d => d.isMutable ? 3 : 1.5)
        .attr('marker-end', d => d.isMutable ? 'url(#arr-mut)' : 'url(#arr-shared)');
      const edgeLabel = edge.append('text').text(d => '&' + (d.isMutable ? 'mut ' : '') + d.variable).attr('text-anchor','middle');

      const node = g.selectAll('.node').data(graph.nodes).join('g').attr('class','node')
        .call(d3.drag().on('start',(e,d)=>{if(!e.active)simulation.alphaTarget(0.3).restart();d.fx=d.x;d.fy=d.y;}).on('drag',(e,d)=>{d.fx=e.x;d.fy=e.y;}).on('end',(e,d)=>{if(!e.active)simulation.alphaTarget(0);d.fx=null;d.fy=null;}));

      node.append('circle').attr('r', 20)
        .attr('fill', d => d.kind === 'Origin' ? '#58a6ff' : '#3fb950')
        .attr('stroke', d => d.kind === 'Origin' ? '#58a6ff' : '#3fb950')
        .attr('opacity', 0.8);

      node.append('text').attr('dy', -25).attr('text-anchor','middle').text(d => d.id + '()');
      node.append('text').attr('class','param').attr('dy', 5).attr('text-anchor','middle').text(d => d.variable || '');

      simulation.on('tick', () => {
        edgePath.attr('d', d => 'M'+d.source.x+','+d.source.y+'L'+d.target.x+','+d.target.y);
        edgeLabel.attr('x', d => (d.source.x+d.target.x)/2).attr('y', d => (d.source.y+d.target.y)/2 - 8);
        node.attr('transform', d => 'translate('+d.x+','+d.y+')');
      });
    })();
  </script>
</body>
</html>`;
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
