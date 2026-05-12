# Milestone 5: Ownership Graph WebView - Detailed Specification

## 5.1 WebView Panel Registration and Lifecycle

**Objective:** Create a VS Code WebView panel that hosts the interactive ownership graph. The panel persists across editor tab switches, can be reopened after closing, and retains its state (zoom level, selected node) across VS Code restarts.

**Steps:**
1. Register a WebView panel type with `vscode.window.registerWebviewViewProvider`
2. Implement `createOrShow()` pattern (reuse existing panel or create new)
3. Handle panel disposal (cleanup resources)
4. Serialize/deserialize panel state for persistence
5. Set Content Security Policy for WebView

**Code (graph/panel.ts):**
```typescript
export class GraphPanel {
    public static currentPanel: GraphPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri, graph?: FunctionOwnershipSummary) {
        const column = vscode.ViewColumn.Beside;

        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel._panel.reveal(column);
            if (graph) GraphPanel.currentPanel.updateGraph(graph);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'borrowscopeGraph',
            'BorrowScope: Ownership Graph',
            column,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [vscode.Uri.joinPath(extensionUri, 'media')],
            }
        );

        GraphPanel.currentPanel = new GraphPanel(panel, extensionUri);
        if (graph) GraphPanel.currentPanel.updateGraph(graph);
    }

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;
        this._panel.webview.html = this._getHtmlContent(extensionUri);

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Handle messages from WebView (e.g., node clicked)
        this._panel.webview.onDidReceiveMessage(
            message => this._handleMessage(message),
            null,
            this._disposables
        );
    }

    public updateGraph(graph: FunctionOwnershipSummary) {
        this._panel.webview.postMessage({ type: 'updateGraph', data: graph });
    }

    public dispose() {
        GraphPanel.currentPanel = undefined;
        this._panel.dispose();
        this._disposables.forEach(d => d.dispose());
    }
}
```

**Panel lifecycle:**
```
User clicks "Show Graph" (or CodeLens)
    │
    ▼
GraphPanel.createOrShow()
    │
    ├── Panel exists? → reveal + update data
    │
    └── Panel doesn't exist? → create new WebView
                                    │
                                    ▼
                              Load HTML + D3.js
                                    │
                                    ▼
                              Render initial graph
                                    │
                                    ▼
                              Listen for updates
                                    │
                              (user edits file)
                                    │
                                    ▼
                              Server sends analysisUpdated
                                    │
                                    ▼
                              Extension fetches new graph
                                    │
                                    ▼
                              postMessage → WebView re-renders
```

**Expectation:** Panel opens beside the editor, shows the graph, and stays open as the user navigates between files. Closing and reopening preserves the last-viewed function.

**Tests for 5.1:**
- Panel opens on command execution
- Panel reuses existing instance (no duplicates)
- Panel survives editor tab switches
- Panel disposes cleanly (no memory leaks)
- Content Security Policy blocks external scripts
- `retainContextWhenHidden` keeps state when panel is not visible

---

## 5.2 Graph Data Model

**Objective:** Transform the `FunctionOwnershipSummary` from the LSP response into a graph data model suitable for D3.js rendering. Nodes represent variables, edges represent ownership relationships.

**Code (graph/messages.ts):**
```typescript
// Data model sent to the WebView for rendering
interface GraphData {
    nodes: GraphNode[];
    edges: GraphEdge[];
    metadata: GraphMetadata;
}

interface GraphNode {
    id: string;
    name: string;
    type: string;
    category: OwnershipCategory;
    line: number;
    isAlive: boolean;      // not yet dropped
    isCopy: boolean;
    size: number;          // visual size (based on edge count)
}

interface GraphEdge {
    id: string;
    source: string;        // node id
    target: string;        // node id
    kind: EdgeKind;
    label: string;
    isMutable: boolean;
    isActive: boolean;     // borrow still alive
}

type OwnershipCategory = 'owned' | 'shared_ref' | 'mut_ref' | 'rc' | 'arc'
    | 'interior_mut' | 'raw_ptr' | 'copy' | 'closure';

type EdgeKind = 'borrow_shared' | 'borrow_mut' | 'move' | 'rc_clone'
    | 'arc_clone' | 'closure_capture' | 'refcell_borrow';

interface GraphMetadata {
    functionName: string;
    file: string;
    stats: { variables: number; borrows: number; moves: number; conflicts: number };
}

// Transform LSP response to graph data
function summaryToGraphData(summary: FunctionOwnershipSummary): GraphData {
    const nodes: GraphNode[] = summary.variables.map(v => ({
        id: `${v.name}_${v.line}`,
        name: v.name,
        type: v.type_display,
        category: v.ownership_category,
        line: v.line,
        isAlive: true,
        isCopy: v.is_copy,
        size: computeNodeSize(v, summary),
    }));

    const edges: GraphEdge[] = [
        ...summary.borrow_scopes.map(b => ({
            id: `borrow_${b.borrower_name}_${b.target_name}`,
            source: findNodeId(b.borrower_name, nodes),
            target: findNodeId(b.target_name, nodes),
            kind: b.is_mutable ? 'borrow_mut' : 'borrow_shared' as EdgeKind,
            label: b.is_mutable ? '&mut' : '&',
            isMutable: b.is_mutable,
            isActive: true,
        })),
        ...summary.moves.map(m => ({
            id: `move_${m.source_name}_${m.line}`,
            source: findNodeId(m.source_name, nodes),
            target: findNodeId(getDestName(m.destination), nodes),
            kind: 'move' as EdgeKind,
            label: 'move',
            isMutable: false,
            isActive: true,
        })),
        ...summary.rc_clones.map(c => ({
            id: `clone_${c.clone_variable}_${c.line}`,
            source: findNodeId(c.clone_variable, nodes),
            target: findNodeId(c.source_variable, nodes),
            kind: (c.clone_type === 'Rc' ? 'rc_clone' : 'arc_clone') as EdgeKind,
            label: `${c.clone_type}::clone`,
            isMutable: false,
            isActive: true,
        })),
    ];

    return { nodes, edges, metadata: { functionName: summary.function_name, ... } };
}
```

**Expectation:** Every variable becomes a node, every ownership relationship becomes an edge. The data model is self-contained (the WebView doesn't need to make additional requests to render).

**Tests for 5.2:**
- Empty function produces empty nodes/edges arrays
- Each variable maps to exactly one node
- Borrow scopes produce borrow edges with correct source/target
- Moves produce move edges
- Rc clones produce clone edges
- Node size scales with edge count
- Category is correctly assigned based on type info

---

## 5.3 Rendering Engine (D3.js)

**Objective:** Render the ownership graph as an interactive SVG using D3.js inside the WebView. Use a force-directed layout for small graphs and a hierarchical (dagre) layout for larger ones.

**Code (media/graph.js):**
```javascript
// Runs inside the WebView
const vscode = acquireVsCodeApi();
let simulation;
let svg, nodeGroup, edgeGroup;

function initGraph() {
    svg = d3.select('#graph-container')
        .append('svg')
        .attr('width', '100%')
        .attr('height', '100%')
        .call(d3.zoom().on('zoom', (event) => {
            svg.select('g.content').attr('transform', event.transform);
        }))
        .append('g').attr('class', 'content');

    // Arrow marker definitions
    svg.append('defs').selectAll('marker')
        .data(['borrow_shared', 'borrow_mut', 'move', 'rc_clone'])
        .join('marker')
        .attr('id', d => `arrow-${d}`)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 20)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', d => edgeColor(d));

    edgeGroup = svg.append('g').attr('class', 'edges');
    nodeGroup = svg.append('g').attr('class', 'nodes');
}

function renderGraph(data) {
    // Edges
    const edges = edgeGroup.selectAll('g.edge')
        .data(data.edges, d => d.id)
        .join(
            enter => {
                const g = enter.append('g').attr('class', 'edge');
                g.append('path')
                    .attr('stroke', d => edgeColor(d.kind))
                    .attr('stroke-width', d => d.isMutable ? 2.5 : 1.5)
                    .attr('stroke-dasharray', d => edgeDash(d.kind))
                    .attr('fill', 'none')
                    .attr('marker-end', d => `url(#arrow-${d.kind})`);
                g.append('text')
                    .attr('font-size', '10px')
                    .attr('fill', d => edgeColor(d.kind))
                    .text(d => d.label);
                return g;
            }
        );

    // Nodes
    const nodes = nodeGroup.selectAll('g.node')
        .data(data.nodes, d => d.id)
        .join(
            enter => {
                const g = enter.append('g').attr('class', 'node')
                    .call(d3.drag()
                        .on('start', dragStarted)
                        .on('drag', dragged)
                        .on('end', dragEnded))
                    .on('click', (event, d) => onNodeClick(d));

                g.append('rect')
                    .attr('rx', 6)
                    .attr('width', d => Math.max(80, d.name.length * 8 + 20))
                    .attr('height', 36)
                    .attr('fill', d => nodeColor(d.category))
                    .attr('stroke', d => nodeStroke(d.category))
                    .attr('stroke-width', 2);

                g.append('text')
                    .attr('text-anchor', 'middle')
                    .attr('dy', '0.35em')
                    .attr('fill', 'white')
                    .attr('font-size', '11px')
                    .attr('font-weight', 'bold')
                    .text(d => d.name);

                g.append('text')
                    .attr('text-anchor', 'middle')
                    .attr('dy', '1.5em')
                    .attr('fill', 'rgba(255,255,255,0.7)')
                    .attr('font-size', '9px')
                    .text(d => truncateType(d.type, 20));

                return g;
            }
        );

    // Force simulation
    simulation = d3.forceSimulation(data.nodes)
        .force('link', d3.forceLink(data.edges).id(d => d.id).distance(120))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .on('tick', ticked);
}

function nodeColor(category) {
    const colors = {
        owned: '#2ecc71', shared_ref: '#3498db', mut_ref: '#e74c3c',
        rc: '#9b59b6', arc: '#8e44ad', interior_mut: '#e67e22',
        raw_ptr: '#95a5a6', copy: '#1abc9c', closure: '#f39c12'
    };
    return colors[category] || '#34495e';
}

function edgeColor(kind) {
    const colors = {
        borrow_shared: '#3498db', borrow_mut: '#e74c3c',
        move: '#27ae60', rc_clone: '#9b59b6', arc_clone: '#8e44ad',
        closure_capture: '#f39c12', refcell_borrow: '#e67e22'
    };
    return colors[kind] || '#95a5a6';
}
```

**Expectation:** The graph renders as an interactive SVG with draggable nodes, zoomable canvas, and color-coded elements. The layout stabilizes within 1-2 seconds.

**Tests for 5.3:**
- Graph renders without JavaScript errors
- Nodes are visible and correctly colored
- Edges connect correct source/target nodes
- Zoom in/out works (scroll wheel)
- Pan works (click and drag on background)
- Node drag works (click and drag on node)
- Graph with 0 nodes shows empty state message
- Graph with 50+ nodes renders without performance issues

---

## 5.4 Node Styling

**Objective:** Each node is visually distinct based on its ownership category. The color, shape, and size communicate the variable's role in the ownership system at a glance.

**Color scheme:**
```
┌─────────────────────────────────────────────────────────┐
│  Ownership Category Color Scheme                         │
├──────────────────┬──────────┬───────────────────────────┤
│  Category        │  Color   │  Example Types            │
├──────────────────┼──────────┼───────────────────────────┤
│  Owned           │  Green   │  Vec, String, Box, struct │
│  Shared Ref (&)  │  Blue    │  &T, &str, &[u8]         │
│  Mutable Ref     │  Red     │  &mut T, &mut Vec        │
│  Rc              │  Purple  │  Rc<T>, Rc<RefCell<T>>   │
│  Arc             │  D.Purple│  Arc<T>, Arc<Mutex<T>>   │
│  Interior Mut    │  Orange  │  RefCell, Cell, Mutex     │
│  Raw Pointer     │  Gray    │  *const T, *mut T        │
│  Copy            │  Teal    │  i32, bool, &T           │
│  Closure         │  Yellow  │  impl Fn, || {}          │
└──────────────────┴──────────┴───────────────────────────┘
```

**Node size:** Proportional to the number of edges (more connections = larger node = more important in the ownership graph).

**Expectation:** A user can glance at the graph and immediately identify which variables are references (blue/red), which are smart pointers (purple), and which own their data (green).

**Tests for 5.4:**
- `Vec<i32>` node is green (owned)
- `&Vec<i32>` node is blue (shared ref)
- `&mut Vec<i32>` node is red (mutable ref)
- `Rc<T>` node is purple
- `i32` node is teal (copy)
- Node with 5 edges is visually larger than node with 1 edge
- Colors work in both light and dark VS Code themes

---

## 5.5 Edge Styling

**Objective:** Edges are visually distinct by relationship type. Line style (solid, dashed, dotted), color, and arrow shape communicate the ownership semantics.

**Edge styles:**
```
Borrow (shared):   ─ ─ ─ ─ ─▶   blue, dashed
Borrow (mutable):  ━━━━━━━━━▶   red, solid thick
Move:              ──────────▶   green, solid
Rc/Arc clone:      ··········▶   purple, dotted
Closure capture:   ─·─·─·─·─▶   orange, dash-dot
```

**Expectation:** Edge type is identifiable without reading labels. The visual language is consistent with the inline decorations (same colors for same concepts).

**Tests for 5.5:**
- Shared borrow edge is blue and dashed
- Mutable borrow edge is red and thick
- Move edge is green and solid
- Clone edge is purple and dotted
- Edge labels are readable (not overlapping nodes)
- Arrow heads point in correct direction (source → target)

---

## 5.6 Interaction: Click Node to Navigate to Source

**Objective:** Clicking a node in the graph navigates the editor to the line where that variable is declared. This connects the visual representation back to the source code.

**Code (graph/panel.ts):**
```typescript
private _handleMessage(message: any) {
    switch (message.type) {
        case 'nodeClicked':
            const { file, line } = message.data;
            const uri = vscode.Uri.file(file);
            vscode.window.showTextDocument(uri, {
                selection: new vscode.Range(line - 1, 0, line - 1, 0),
                preserveFocus: false,
            });
            break;
    }
}
```

**Code (media/graph.js - WebView side):**
```javascript
function onNodeClick(node) {
    vscode.postMessage({ type: 'nodeClicked', data: { file: node.file, line: node.line } });
    // Highlight the clicked node
    d3.selectAll('g.node rect').attr('stroke-width', 2);
    d3.select(this).select('rect').attr('stroke-width', 4);
}
```

**Expectation:** Click a node → editor scrolls to that variable's declaration. The clicked node gets a thicker border to show selection state.

**Tests for 5.6:**
- Clicking a node sends message to extension
- Extension navigates editor to correct file and line
- Clicked node gets visual highlight (thicker border)
- Clicking another node moves highlight
- Navigation works for variables in different files (if graph spans files)

---

## 5.7 Interaction: Hover for Type Details

**Objective:** Hovering over a node shows a tooltip with full type information: the complete type string, trait implementations, layout size, and ownership relationships.

**Code (media/graph.js):**
```javascript
function onNodeHover(event, node) {
    const tooltip = d3.select('#tooltip');
    tooltip.html(`
        <strong>${node.name}</strong><br>
        <code>${node.type}</code><br>
        <hr>
        Category: ${node.category}<br>
        Copy: ${node.isCopy ? 'yes' : 'no'}<br>
        ${node.layoutSize ? `Size: ${node.layoutSize} bytes` : ''}
    `)
    .style('left', (event.pageX + 10) + 'px')
    .style('top', (event.pageY - 10) + 'px')
    .style('opacity', 1);
}

function onNodeHoverEnd() {
    d3.select('#tooltip').style('opacity', 0);
}
```

**Expectation:** Hovering shows a rich tooltip without requiring a click. The tooltip disappears when the mouse moves away.

**Tests for 5.7:**
- Hovering shows tooltip with variable name and type
- Tooltip contains ownership category
- Tooltip disappears on mouse leave
- Tooltip doesn't overflow the panel bounds
- Tooltip content matches the server's analysis data

---

## 5.8 Interaction: Filter by Scope / Type Category

**Objective:** Allow the user to filter the graph to show only specific categories of variables or only variables within a specific scope. This reduces visual clutter for complex functions.

**Code (media/graph.js):**
```javascript
// Filter controls rendered above the graph
function renderFilters(data) {
    const categories = [...new Set(data.nodes.map(n => n.category))];

    const filterBar = d3.select('#filter-bar');
    filterBar.selectAll('button.filter')
        .data(categories)
        .join('button')
        .attr('class', 'filter')
        .style('background', d => nodeColor(d))
        .style('opacity', 1)
        .text(d => d)
        .on('click', (event, category) => toggleCategory(category));
}

let hiddenCategories = new Set();

function toggleCategory(category) {
    if (hiddenCategories.has(category)) {
        hiddenCategories.delete(category);
    } else {
        hiddenCategories.add(category);
    }
    applyFilters();
}

function applyFilters() {
    d3.selectAll('g.node')
        .style('opacity', d => hiddenCategories.has(d.category) ? 0.1 : 1);
    d3.selectAll('g.edge')
        .style('opacity', d => {
            const srcHidden = hiddenCategories.has(findNode(d.source).category);
            const tgtHidden = hiddenCategories.has(findNode(d.target).category);
            return (srcHidden || tgtHidden) ? 0.1 : 1;
        });
}
```

**Expectation:** Clicking a category button toggles visibility of those nodes. Hidden nodes fade to 10% opacity (not removed, so the layout doesn't jump). Edges connected to hidden nodes also fade.

**Tests for 5.8:**
- Filter buttons appear for each category present in the graph
- Clicking a filter fades nodes of that category
- Clicking again restores them
- Edges to/from hidden nodes also fade
- Multiple filters can be active simultaneously
- "Show All" button restores everything

---

## 5.9 Live Update

**Objective:** When the server sends a `borrowscope/analysisUpdated` notification, the extension fetches the new graph data and updates the WebView without losing the user's zoom/pan state.

**Code (extension side):**
```typescript
client.onNotification('borrowscope/analysisUpdated', async (params: AnalysisUpdatedParams) => {
    if (!GraphPanel.currentPanel) return;

    // Re-fetch the graph for the currently displayed function
    const graph = await client.sendRequest('borrowscope/ownershipGraph', {
        textDocument: { uri: params.uri },
        position: currentGraphPosition,
    });

    GraphPanel.currentPanel.updateGraph(graph);
});
```

**Code (WebView side):**
```javascript
window.addEventListener('message', event => {
    const message = event.data;
    if (message.type === 'updateGraph') {
        // Preserve current zoom/pan transform
        const currentTransform = d3.zoomTransform(svg.node());
        renderGraph(message.data);
        // Restore transform
        svg.call(zoom.transform, currentTransform);
    }
});
```

**Expectation:** The graph updates smoothly after each file edit. Nodes that didn't change stay in place. New nodes animate in. Removed nodes animate out. Zoom/pan state is preserved.

**Tests for 5.9:**
- Graph updates after file edit (within 1 second)
- Zoom level preserved after update
- Pan position preserved after update
- New nodes appear with fade-in animation
- Removed nodes disappear with fade-out
- Unchanged nodes don't move (stable layout)

---

## 5.10 Function Selector

**Objective:** A dropdown at the top of the graph panel lets the user switch between functions in the current file. This avoids needing to click CodeLens for each function.

**Code (media/graph.js):**
```javascript
function renderFunctionSelector(functions) {
    const selector = d3.select('#function-selector');
    selector.selectAll('option')
        .data(functions)
        .join('option')
        .attr('value', d => d.name)
        .text(d => `${d.name}() - ${d.stats.total_variables} vars`);

    selector.on('change', function() {
        const selected = this.value;
        vscode.postMessage({ type: 'selectFunction', data: { name: selected } });
    });
}
```

**Expectation:** The dropdown lists all functions in the current file with their variable counts. Selecting a function loads its ownership graph.

**Tests for 5.10:**
- Dropdown lists all functions in the file
- Selecting a function updates the graph
- Current function is pre-selected in dropdown
- Dropdown updates when file changes (new functions added/removed)
- Functions are sorted by line number

---

## 5.T Integration Test Suite

```typescript
suite('Graph WebView Tests', () => {
    test('Panel opens and renders graph', async () => {
        await vscode.commands.executeCommand('borrowscope.showGraph');
        // Verify panel exists
        assert.ok(GraphPanel.currentPanel);
    });

    test('Graph updates on file change', async () => {
        // Edit file, wait for update, verify graph changed
    });

    test('Node click navigates to source', async () => {
        // Simulate node click message, verify editor position
    });

    test('Filter hides/shows nodes', async () => {
        // Send filter message, verify node opacity changes
    });

    test('Function selector switches graph', async () => {
        // Select different function, verify new graph data
    });
});
```
