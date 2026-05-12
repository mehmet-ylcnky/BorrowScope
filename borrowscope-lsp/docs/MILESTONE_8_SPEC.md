# Milestone 8: Configuration and Polish - Detailed Specification

## 8.1 Extension Settings

**Objective:** Provide a comprehensive settings schema that lets users customize every aspect of the extension: visual appearance, performance tuning, feature toggles, and server behavior. Settings are organized into logical groups and have sensible defaults.

**Settings schema:**
```json
{
  "borrowscope.server.path": {
    "type": "string",
    "default": "",
    "description": "Path to borrowscope-lsp binary. Leave empty for auto-detection."
  },
  "borrowscope.server.extraArgs": {
    "type": "array",
    "items": { "type": "string" },
    "default": [],
    "description": "Additional arguments passed to the server binary."
  },
  "borrowscope.analysis.debounceMs": {
    "type": "number",
    "default": 300,
    "minimum": 0,
    "maximum": 2000,
    "description": "Milliseconds to wait after last keystroke before re-analyzing."
  },
  "borrowscope.decorations.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show inline ownership decorations in the editor."
  },
  "borrowscope.decorations.borrowScopes": {
    "type": "boolean",
    "default": true,
    "description": "Highlight borrow scope regions with colored backgrounds."
  },
  "borrowscope.decorations.gutterIcons": {
    "type": "boolean",
    "default": true,
    "description": "Show borrow/move/drop icons in the gutter."
  },
  "borrowscope.decorations.inlayHints": {
    "type": "boolean",
    "default": true,
    "description": "Show ownership category hints inline (e.g., [Rc], [&mut])."
  },
  "borrowscope.decorations.codeLens": {
    "type": "boolean",
    "default": true,
    "description": "Show ownership statistics above functions."
  },
  "borrowscope.graph.layout": {
    "type": "string",
    "enum": ["force", "hierarchical", "radial"],
    "default": "hierarchical",
    "description": "Graph layout algorithm."
  },
  "borrowscope.graph.showTypes": {
    "type": "boolean",
    "default": true,
    "description": "Show type names on graph nodes."
  },
  "borrowscope.graph.animateUpdates": {
    "type": "boolean",
    "default": true,
    "description": "Animate graph transitions on updates."
  },
  "borrowscope.colors.sharedBorrow": {
    "type": "string",
    "default": "#3498db",
    "description": "Color for shared borrow (&T) elements."
  },
  "borrowscope.colors.mutableBorrow": {
    "type": "string",
    "default": "#e74c3c",
    "description": "Color for mutable borrow (&mut T) elements."
  },
  "borrowscope.colors.move": {
    "type": "string",
    "default": "#27ae60",
    "description": "Color for move/ownership transfer elements."
  },
  "borrowscope.colors.rcArc": {
    "type": "string",
    "default": "#9b59b6",
    "description": "Color for Rc/Arc shared ownership elements."
  },
  "borrowscope.diagnostics.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show borrow conflict diagnostics in Problems panel."
  },
  "borrowscope.diagnostics.severity": {
    "type": "string",
    "enum": ["information", "hint", "warning"],
    "default": "information",
    "description": "Severity level for BorrowScope diagnostics."
  }
}
```

**Code (config.ts):**
```typescript
export interface BorrowScopeConfig {
    server: { path: string; extraArgs: string[] };
    analysis: { debounceMs: number };
    decorations: {
        enabled: boolean;
        borrowScopes: boolean;
        gutterIcons: boolean;
        inlayHints: boolean;
        codeLens: boolean;
    };
    graph: { layout: 'force' | 'hierarchical' | 'radial'; showTypes: boolean; animateUpdates: boolean };
    colors: { sharedBorrow: string; mutableBorrow: string; move: string; rcArc: string };
    diagnostics: { enabled: boolean; severity: string };
}

export function getConfig(): BorrowScopeConfig {
    const cfg = vscode.workspace.getConfiguration('borrowscope');
    return {
        server: { path: cfg.get('server.path', ''), extraArgs: cfg.get('server.extraArgs', []) },
        analysis: { debounceMs: cfg.get('analysis.debounceMs', 300) },
        decorations: {
            enabled: cfg.get('decorations.enabled', true),
            borrowScopes: cfg.get('decorations.borrowScopes', true),
            gutterIcons: cfg.get('decorations.gutterIcons', true),
            inlayHints: cfg.get('decorations.inlayHints', true),
            codeLens: cfg.get('decorations.codeLens', true),
        },
        graph: {
            layout: cfg.get('graph.layout', 'hierarchical'),
            showTypes: cfg.get('graph.showTypes', true),
            animateUpdates: cfg.get('graph.animateUpdates', true),
        },
        colors: {
            sharedBorrow: cfg.get('colors.sharedBorrow', '#3498db'),
            mutableBorrow: cfg.get('colors.mutableBorrow', '#e74c3c'),
            move: cfg.get('colors.move', '#27ae60'),
            rcArc: cfg.get('colors.rcArc', '#9b59b6'),
        },
        diagnostics: { enabled: cfg.get('diagnostics.enabled', true), severity: cfg.get('diagnostics.severity', 'information') },
    };
}

// React to settings changes
vscode.workspace.onDidChangeConfiguration(event => {
    if (event.affectsConfiguration('borrowscope')) {
        const newConfig = getConfig();
        // Notify server of config change
        client.sendNotification('borrowscope/configChanged', newConfig);
        // Update decorations
        refreshDecorations(newConfig);
    }
});
```

**Expectation:** Every visual and behavioral aspect is configurable. Changes take effect immediately without restarting the extension.

**Tests for 8.1:**
- All settings have correct types and defaults
- Changing `decorations.enabled` to false hides all decorations
- Changing `colors.sharedBorrow` updates decoration colors immediately
- Changing `analysis.debounceMs` affects re-analysis timing
- Invalid setting values fall back to defaults
- Settings sync across VS Code windows (workspace vs user scope)

---

## 8.2 Keyboard Shortcuts

**Objective:** Provide keyboard shortcuts for common actions so power users can navigate ownership information without touching the mouse.

**Keybindings:**
```json
{
  "keybindings": [
    { "command": "borrowscope.showGraph", "key": "ctrl+shift+o", "when": "editorLangId == rust" },
    { "command": "borrowscope.inspectVariable", "key": "ctrl+shift+i", "when": "editorLangId == rust" },
    { "command": "borrowscope.toggleDecorations", "key": "ctrl+shift+d", "when": "editorLangId == rust" },
    { "command": "borrowscope.nextConflict", "key": "alt+shift+n", "when": "editorLangId == rust" },
    { "command": "borrowscope.prevConflict", "key": "alt+shift+p", "when": "editorLangId == rust" },
    { "command": "borrowscope.focusGraph", "key": "ctrl+shift+g", "when": "borrowscope.graphVisible" }
  ]
}
```

**Commands:**
| Shortcut | Command | Action |
|----------|---------|--------|
| `Ctrl+Shift+O` | Show Graph | Open/focus the ownership graph panel |
| `Ctrl+Shift+I` | Inspect Variable | Show ownership details for variable at cursor |
| `Ctrl+Shift+D` | Toggle Decorations | Enable/disable all inline decorations |
| `Alt+Shift+N` | Next Conflict | Jump to next borrow conflict in file |
| `Alt+Shift+P` | Previous Conflict | Jump to previous borrow conflict |
| `Ctrl+Shift+G` | Focus Graph | Move keyboard focus to graph panel |

**Code (commands.ts):**
```typescript
function registerCommands(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        vscode.commands.registerCommand('borrowscope.nextConflict', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) return;

            const diagnostics = vscode.languages.getDiagnostics(editor.document.uri)
                .filter(d => d.source === 'BorrowScope');

            const currentLine = editor.selection.active.line;
            const next = diagnostics.find(d => d.range.start.line > currentLine);

            if (next) {
                editor.selection = new vscode.Selection(next.range.start, next.range.start);
                editor.revealRange(next.range, vscode.TextEditorRevealType.InCenter);
            } else if (diagnostics.length > 0) {
                // Wrap around to first
                const first = diagnostics[0];
                editor.selection = new vscode.Selection(first.range.start, first.range.start);
                editor.revealRange(first.range, vscode.TextEditorRevealType.InCenter);
            }
        })
    );
}
```

**Expectation:** Keyboard-driven workflow is possible without the mouse. Conflict navigation cycles through all conflicts in the file.

**Tests for 8.2:**
- `Ctrl+Shift+O` opens graph panel
- `Alt+Shift+N` jumps to next conflict (wraps around)
- `Ctrl+Shift+D` toggles decorations on/off
- Shortcuts only active in Rust files (`when` clause)
- Shortcuts don't conflict with rust-analyzer or VS Code defaults
- Custom keybindings can override defaults

---

## 8.3 Command Palette

**Objective:** All extension functionality is accessible via the Command Palette (`Ctrl+Shift+P`). Commands are prefixed with "BorrowScope:" for discoverability.

**Full command list:**
```
BorrowScope: Show Ownership Graph
BorrowScope: Inspect Variable at Cursor
BorrowScope: Toggle Decorations
BorrowScope: Toggle Borrow Scope Highlighting
BorrowScope: Toggle Gutter Icons
BorrowScope: Show Timeline View
BorrowScope: Show Scope Nesting View
BorrowScope: Show Reference Count Chart
BorrowScope: Show Move Chains
BorrowScope: Toggle Conflict Mode
BorrowScope: Next Conflict
BorrowScope: Previous Conflict
BorrowScope: Restart Server
BorrowScope: Show Server Output
BorrowScope: Compare Ownership (Before/After)
BorrowScope: Export Graph as DOT
BorrowScope: Export Graph as SVG
```

**Expectation:** Every feature is accessible via Command Palette. Commands that require context (e.g., cursor in a function) show an error message if the precondition is not met.

**Tests for 8.3:**
- All commands appear in Command Palette when a Rust file is open
- Commands not applicable to non-Rust files are hidden
- "Restart Server" actually restarts the server
- "Show Server Output" opens the output channel
- "Export Graph as DOT" produces valid DOT file

---

## 8.4 Theme Integration

**Objective:** All visual elements respect the user's VS Code theme (light, dark, high contrast). Colors adapt automatically. Custom colors from settings override theme defaults.

**Steps:**
1. Use CSS variables from VS Code's theme API in WebView
2. Use `ThemeColor` for decoration colors
3. Provide separate color palettes for light/dark/high-contrast
4. Test with popular themes (One Dark, Solarized, GitHub Light)

**Code (media/graph.css):**
```css
:root {
    --bs-bg: var(--vscode-editor-background);
    --bs-fg: var(--vscode-editor-foreground);
    --bs-border: var(--vscode-panel-border);
    --bs-node-text: var(--vscode-editor-foreground);
    --bs-tooltip-bg: var(--vscode-editorHoverWidget-background);
    --bs-tooltip-border: var(--vscode-editorHoverWidget-border);
}

/* Adapt node colors for light themes */
@media (prefers-color-scheme: light) {
    .node rect { opacity: 0.85; }
    .edge path { opacity: 0.7; }
}

/* High contrast mode */
.vscode-high-contrast .node rect {
    stroke-width: 3;
}
.vscode-high-contrast .edge path {
    stroke-width: 3;
}
```

**Code (decorations.ts):**
```typescript
// Use ThemeColor for decorations that adapt to theme
const borrowHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: new vscode.ThemeColor('borrowscope.borrowScopeBackground'),
    borderLeft: '2px solid',
    borderColor: new vscode.ThemeColor('borrowscope.borrowScopeBorder'),
});

// Register theme colors in package.json
// "colors": [
//   { "id": "borrowscope.borrowScopeBackground", "defaults": { "dark": "#3498db15", "light": "#3498db10" } },
//   { "id": "borrowscope.borrowScopeBorder", "defaults": { "dark": "#3498db60", "light": "#3498db40" } }
// ]
```

**Expectation:** The extension looks native in any theme. No hard-coded colors that clash with dark or light backgrounds.

**Tests for 8.4:**
- Graph renders correctly in dark theme (light text on dark background)
- Graph renders correctly in light theme (dark text on light background)
- High contrast mode has thicker borders and higher opacity
- Custom theme colors from settings override defaults
- WebView CSS variables resolve to correct theme values
- No white-on-white or black-on-black text in any theme

---

## 8.5 Welcome View and Onboarding

**Objective:** First-time users see a welcome panel explaining what BorrowScope does, how to use it, and what prerequisites are needed (Rust toolchain). The onboarding flow detects the environment and guides the user through setup.

**Steps:**
1. Detect if this is the first activation (check global state)
2. Show welcome WebView with overview, screenshots, and quick-start
3. Check prerequisites: Rust toolchain installed, workspace has Cargo.toml
4. If prerequisites missing, show actionable guidance
5. After first successful analysis, dismiss welcome and show results

**Code (welcome.ts):**
```typescript
export function showWelcomeIfNeeded(context: vscode.ExtensionContext) {
    const hasShownWelcome = context.globalState.get('borrowscope.welcomeShown', false);
    if (hasShownWelcome) return;

    const panel = vscode.window.createWebviewPanel(
        'borrowscopeWelcome',
        'Welcome to BorrowScope',
        vscode.ViewColumn.One,
        { enableScripts: true }
    );

    panel.webview.html = getWelcomeHtml(context);
    context.globalState.update('borrowscope.welcomeShown', true);
}

async function checkPrerequisites(): Promise<PrerequisiteStatus> {
    const hasRustc = await commandExists('rustc');
    const hasCargo = await commandExists('cargo');
    const hasCargoToml = await fileExists(workspaceRoot, 'Cargo.toml');

    return {
        rustToolchain: hasRustc && hasCargo,
        cargoProject: hasCargoToml,
        serverBinary: await serverBinaryExists(),
    };
}
```

**Welcome panel content:**
```
┌─────────────────────────────────────────────────────────────┐
│  🔍 Welcome to BorrowScope                                   │
│                                                              │
│  BorrowScope visualizes Rust's ownership system in           │
│  real-time, directly in your editor.                         │
│                                                              │
│  ✓ Rust toolchain detected (1.78.0)                         │
│  ✓ Cargo project found (my-project)                         │
│  ✓ Server binary ready                                       │
│                                                              │
│  Getting started:                                            │
│  1. Open any .rs file                                        │
│  2. Look for [Rc], [&], [&mut] hints next to variables      │
│  3. Press Ctrl+Shift+O to open the ownership graph           │
│                                                              │
│  [Open a Rust file]  [Show Graph]  [Documentation]           │
└─────────────────────────────────────────────────────────────┘
```

**Expectation:** New users understand what the extension does and how to use it within 30 seconds. Missing prerequisites are clearly communicated with fix instructions.

**Tests for 8.5:**
- Welcome shows on first activation
- Welcome does NOT show on subsequent activations
- Prerequisites are correctly detected
- Missing Rust toolchain shows install instructions
- Missing Cargo.toml shows "open a Rust project" guidance
- "Don't show again" button works

---

## 8.6 Performance Profiling and Optimization

**Objective:** Identify and fix performance bottlenecks. The extension should not noticeably slow down VS Code, even for large projects. Target: < 2% CPU usage when idle, < 200MB extension memory (excluding server).

**Profiling approach:**
1. Use VS Code's built-in "Developer: Open Process Explorer" to monitor
2. Add timing instrumentation to critical paths (analysis, rendering)
3. Log slow operations (> 100ms) to output channel
4. Profile WebView rendering with Chrome DevTools (via "Developer: Open Webview Developer Tools")

**Optimization targets:**
```
Metric                          │ Target        │ Action if exceeded
────────────────────────────────┼───────────────┼──────────────────────
Idle CPU (no edits)             │ < 2%          │ Check for polling loops
CPU during typing               │ < 10%         │ Increase debounce
Extension activation time       │ < 1s          │ Lazy-load features
Graph render (50 nodes)         │ < 100ms       │ Reduce D3 complexity
Memory (extension process)      │ < 200MB       │ Evict caches
Server memory                   │ < 1.5GB       │ Reduce Salsa cache
Time to first decoration        │ < 5s          │ Prioritize visible file
```

**Code (performance.ts):**
```typescript
class PerformanceMonitor {
    private timings: Map<string, number[]> = new Map();

    time<T>(label: string, fn: () => T): T {
        const start = performance.now();
        const result = fn();
        const elapsed = performance.now() - start;

        if (!this.timings.has(label)) this.timings.set(label, []);
        this.timings.get(label)!.push(elapsed);

        if (elapsed > 100) {
            console.warn(`[BorrowScope] ${label} took ${elapsed.toFixed(1)}ms`);
        }
        return result;
    }

    report(): string {
        let report = 'BorrowScope Performance Report:\n';
        for (const [label, times] of this.timings) {
            const avg = times.reduce((a, b) => a + b, 0) / times.length;
            const max = Math.max(...times);
            report += `  ${label}: avg=${avg.toFixed(1)}ms, max=${max.toFixed(1)}ms, count=${times.length}\n`;
        }
        return report;
    }
}
```

**Expectation:** The extension is imperceptible when idle. During active editing, the overhead is minimal and within the performance budget defined in Milestone 6.

**Tests for 8.6:**
- Idle CPU < 2% (no file changes for 10 seconds)
- Extension activation < 1 second (excluding server startup)
- Graph render for 50-node graph < 100ms
- No memory leaks over 100 file open/close cycles
- Performance report command shows timing data

---

## 8.7 Accessibility

**Objective:** The extension is usable by developers who rely on screen readers, keyboard navigation, or high-contrast modes. Graph information is available in text form, not just visually.

**Steps:**
1. Add ARIA labels to WebView elements
2. Provide text-based graph description (accessible via screen reader)
3. Ensure all interactions are keyboard-accessible (Tab, Enter, Escape)
4. Support high-contrast mode with thicker borders and distinct patterns
5. Provide "Describe Graph" command that reads graph structure aloud

**Code (media/graph.js):**
```javascript
// ARIA labels for graph nodes
function addAccessibility(nodeGroup) {
    nodeGroup.selectAll('g.node')
        .attr('role', 'button')
        .attr('tabindex', '0')
        .attr('aria-label', d => `Variable ${d.name}, type ${d.type}, category ${d.category}`)
        .on('keydown', (event, d) => {
            if (event.key === 'Enter') onNodeClick(d);
            if (event.key === 'Tab') focusNextNode(d);
        });
}

// Text description for screen readers
function getGraphDescription(data) {
    const lines = [`Ownership graph for function ${data.metadata.functionName}.`];
    lines.push(`${data.nodes.length} variables, ${data.edges.length} relationships.`);
    data.edges.forEach(e => {
        const src = data.nodes.find(n => n.id === e.source);
        const tgt = data.nodes.find(n => n.id === e.target);
        lines.push(`${src.name} ${e.label} ${tgt.name}.`);
    });
    return lines.join(' ');
}
```

**Expectation:** A screen reader user can understand the ownership structure through text descriptions. Keyboard users can navigate the graph and trigger all interactions.

**Tests for 8.7:**
- All graph nodes have ARIA labels
- Tab key cycles through nodes in the graph
- Enter key on a focused node triggers navigation
- "Describe Graph" command produces readable text
- High contrast mode renders all elements distinctly
- No information is conveyed solely through color (patterns/labels supplement)

---

## 8.T Integration Test Suite

```typescript
suite('Configuration and Polish Tests', () => {
    test('Settings changes take effect immediately', async () => {
        await vscode.workspace.getConfiguration('borrowscope.decorations')
            .update('enabled', false, vscode.ConfigurationTarget.Workspace);
        // Verify decorations disappeared
        await sleep(500);
        // Check no decorations on active editor
    });

    test('Keyboard shortcuts work', async () => {
        await vscode.commands.executeCommand('borrowscope.toggleDecorations');
        // Verify toggle state changed
    });

    test('Theme colors adapt', async () => {
        // Switch to light theme, verify colors changed
    });

    test('Welcome shows only once', async () => {
        // Clear state, activate, verify welcome shown
        // Deactivate, reactivate, verify welcome NOT shown
    });

    test('Performance within budget', async () => {
        // Open large file, measure time to first decoration
    });

    test('Accessibility: nodes have ARIA labels', async () => {
        // Open graph, inspect WebView DOM for aria-label attributes
    });
});
```
