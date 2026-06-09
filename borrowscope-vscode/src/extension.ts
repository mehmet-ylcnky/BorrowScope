import * as vscode from "vscode";
import { startClient, stopClient, restartClient, getClient } from "./client";
import { registerCommands } from "./commands";
import { RuntimeWatcher } from "./runtime-watcher";
import { RuntimeStatusBar, registerRuntimeCommands } from "./runtime-status";
import { parseEvents, filterByFile, filterOwnershipEvents } from "./runtime-parser";
import { mergeViews } from "./merge-views";
import { createRuntimeDecorationTypes, applyRuntimeDecorations, clearRuntimeDecorations } from "./runtime-decorations";
import { showWelcomeIfNeeded, showWelcomePanel } from "./welcome";
import { PerformanceMonitor, registerPerformanceCommand } from "./performance";
import { executeE2E } from "./e2e-runner";

let outputChannel: vscode.OutputChannel;
let runtimeWatcher: RuntimeWatcher | undefined;
let runtimeStatusBar: RuntimeStatusBar | undefined;
let runtimeDecorationTypes: ReturnType<typeof createRuntimeDecorationTypes> | undefined;
let perfMonitor: PerformanceMonitor;

export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  outputChannel = vscode.window.createOutputChannel("BorrowScope");
  outputChannel.appendLine("BorrowScope activated");
  context.subscriptions.push(outputChannel);

  // Performance monitoring
  perfMonitor = new PerformanceMonitor(outputChannel);
  registerPerformanceCommand(context, perfMonitor);

  // Show welcome on first activation
  showWelcomeIfNeeded(context);

  // Register commands
  registerCommands(context);
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "borrowscope.inspectVariable",
      inspectVariable
    ),
    vscode.commands.registerCommand(
      "borrowscope.toggleDecorations",
      toggleDecorations
    ),
    vscode.commands.registerCommand("borrowscope.restartServer", () =>
      restartServer(context)
    ),
    vscode.commands.registerCommand("borrowscope.nextConflict", nextConflict),
    vscode.commands.registerCommand("borrowscope.prevConflict", prevConflict),
    vscode.commands.registerCommand("borrowscope.focusGraph", focusGraph),
    vscode.commands.registerCommand("borrowscope.toggleBorrowScopes", () => toggleSetting("decorations.borrowScopes")),
    vscode.commands.registerCommand("borrowscope.toggleGutterIcons", () => toggleSetting("decorations.gutterIcons")),
    vscode.commands.registerCommand("borrowscope.toggleLifelines", () => toggleSetting("decorations.lifelines")),
    vscode.commands.registerCommand("borrowscope.toggleCodeLens", () => toggleSetting("decorations.codeLens")),
    vscode.commands.registerCommand("borrowscope.showTimeline", () => showPanelView("timeline")),
    vscode.commands.registerCommand("borrowscope.showScopes", () => showPanelView("scopes")),
    vscode.commands.registerCommand("borrowscope.showRefCount", () => showPanelView("refcount")),
    vscode.commands.registerCommand("borrowscope.showMoves", () => showPanelView("moves")),
    vscode.commands.registerCommand("borrowscope.showServerOutput", showServerOutput),
    vscode.commands.registerCommand("borrowscope.exportDot", exportDot),
    vscode.commands.registerCommand("borrowscope.exportSvg", exportSvg),
    vscode.commands.registerCommand("borrowscope.showWelcome", () => showWelcomePanel(context)),
    vscode.commands.registerCommand("borrowscope.describeGraph", describeGraph),
    vscode.commands.registerCommand("borrowscope.runInstrumented", () => executeE2E(outputChannel)),
  );

  // Start language client
  try {
    await startClient(context);
    outputChannel.appendLine("Language server started");
  } catch (e: any) {
    outputChannel.appendLine(`Failed to start server: ${e.message}`);
    vscode.window.showErrorMessage(
      `BorrowScope: ${e.message}`
    );
  }

  // Start runtime watcher
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (workspaceRoot) {
    runtimeWatcher = new RuntimeWatcher();
    runtimeDecorationTypes = createRuntimeDecorationTypes();
    runtimeStatusBar = new RuntimeStatusBar();
    runtimeStatusBar.attachWatcher(runtimeWatcher);
    registerRuntimeCommands(context);

    runtimeWatcher.onEventsUpdated((events) => {
      outputChannel.appendLine(`Runtime: loaded ${events.length} events`);
      applyRuntimeOverlayToActiveEditor(events);
    });

    // Re-apply when active editor changes
    context.subscriptions.push(
      vscode.window.onDidChangeActiveTextEditor(() => {
        if (runtimeWatcher && runtimeWatcher.eventCount > 0) {
          applyRuntimeOverlayToActiveEditor(runtimeWatcher.getEvents());
        }
      }),
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration("borrowscope.runtime") && runtimeWatcher) {
          applyRuntimeOverlayToActiveEditor(runtimeWatcher.getEvents());
        }
      })
    );

    runtimeWatcher.start(workspaceRoot);
    context.subscriptions.push(runtimeWatcher, runtimeStatusBar);
    Object.values(runtimeDecorationTypes).forEach(dt => context.subscriptions.push(dt));
  }
}

export async function deactivate(): Promise<void> {
  await stopClient();
  runtimeWatcher?.dispose();
  if (outputChannel) {
    outputChannel.appendLine("BorrowScope deactivated");
  }
}

export function getRuntimeWatcher(): RuntimeWatcher | undefined {
  return runtimeWatcher;
}

export function getPerfMonitor(): PerformanceMonitor {
  return perfMonitor;
}

function inspectVariable(): void {
  vscode.window.showInformationMessage("BorrowScope: Inspect Variable");
}

function toggleDecorations(): void {
  const config = vscode.workspace.getConfiguration("borrowscope");
  const current = config.get<boolean>("decorations.enabled", true);
  config.update(
    "decorations.enabled",
    !current,
    vscode.ConfigurationTarget.Global
  );
}

async function restartServer(
  context: vscode.ExtensionContext
): Promise<void> {
  try {
    await restartClient(context);
    vscode.window.showInformationMessage("BorrowScope: Server restarted");
  } catch (e: any) {
    vscode.window.showErrorMessage(`BorrowScope: ${e.message}`);
  }
}

function applyRuntimeOverlayToActiveEditor(events: any[]): void {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "rust" || !runtimeDecorationTypes) return;

  const config = vscode.workspace.getConfiguration("borrowscope.runtime");
  if (!config.get<boolean>("enabled", false)) {
    clearRuntimeDecorations(editor, runtimeDecorationTypes);
    return;
  }

  // Get the file path relative to workspace
  const filePath = editor.document.uri.fsPath;
  const fileName = filePath.replace(/\\/g, "/").split("/").slice(-2).join("/");

  // Filter events for this file
  const fileEvents = filterByFile(filterOwnershipEvents(events), fileName);
  if (fileEvents.length === 0) {
    clearRuntimeDecorations(editor, runtimeDecorationTypes);
    return;
  }

  // Get static variables from the LSP (use cached graph if available)
  // For now, use empty static vars — the merge will show runtime_only
  // In production, this would query the LSP for the ownership graph
  const staticVars: any[] = [];

  // Merge and apply
  const merged = mergeViews(staticVars, fileEvents, fileName);
  applyRuntimeDecorations(editor, merged, runtimeDecorationTypes);
}

function nextConflict(): void {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;

  const diagnostics = vscode.languages.getDiagnostics(editor.document.uri)
    .filter((d) => d.source === "BorrowScope");

  if (diagnostics.length === 0) {
    vscode.window.showInformationMessage("BorrowScope: No conflicts in this file");
    return;
  }

  const currentLine = editor.selection.active.line;
  const next = diagnostics.find((d) => d.range.start.line > currentLine);

  if (next) {
    editor.selection = new vscode.Selection(next.range.start, next.range.start);
    editor.revealRange(next.range, vscode.TextEditorRevealType.InCenter);
  } else {
    // Wrap around to first
    const first = diagnostics[0];
    editor.selection = new vscode.Selection(first.range.start, first.range.start);
    editor.revealRange(first.range, vscode.TextEditorRevealType.InCenter);
  }
}

function prevConflict(): void {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;

  const diagnostics = vscode.languages.getDiagnostics(editor.document.uri)
    .filter((d) => d.source === "BorrowScope");

  if (diagnostics.length === 0) {
    vscode.window.showInformationMessage("BorrowScope: No conflicts in this file");
    return;
  }

  const currentLine = editor.selection.active.line;
  const prev = [...diagnostics].reverse().find((d) => d.range.start.line < currentLine);

  if (prev) {
    editor.selection = new vscode.Selection(prev.range.start, prev.range.start);
    editor.revealRange(prev.range, vscode.TextEditorRevealType.InCenter);
  } else {
    // Wrap around to last
    const last = diagnostics[diagnostics.length - 1];
    editor.selection = new vscode.Selection(last.range.start, last.range.start);
    editor.revealRange(last.range, vscode.TextEditorRevealType.InCenter);
  }
}

function focusGraph(): void {
  const { GraphPanel } = require("./graph/panel");
  const panel = GraphPanel.getPanel();
  if (panel) {
    panel.reveal();
  } else {
    vscode.window.showInformationMessage("BorrowScope: No graph panel open. Click a CodeLens first.");
  }
}

async function toggleSetting(key: string): Promise<void> {
  const config = vscode.workspace.getConfiguration("borrowscope");
  const current = config.get<boolean>(key, true);
  await config.update(key, !current, vscode.ConfigurationTarget.Global);
  vscode.window.showInformationMessage(`BorrowScope: ${key} ${!current ? "enabled" : "disabled"}`);
}

function showPanelView(view: string): void {
  const { GraphPanel } = require("./graph/panel");
  const panel = GraphPanel.getPanel();
  if (panel) {
    panel.reveal();
    // The panel will switch to the requested view via lastView
    GraphPanel._lastView = view;
  } else {
    vscode.window.showInformationMessage("BorrowScope: Open a graph first (click a CodeLens), then switch views.");
  }
}

function showServerOutput(): void {
  if (outputChannel) {
    outputChannel.show();
  }
}

async function exportDot(): Promise<void> {
  const { GraphPanel } = require("./graph/panel");
  const panel = GraphPanel.getPanel();
  const graph = panel?.getGraph();
  if (!graph) {
    vscode.window.showErrorMessage("BorrowScope: No graph data. Click a CodeLens first.");
    return;
  }

  // Generate DOT format
  let dot = `digraph "${graph.function_name}" {\n  rankdir=LR;\n  node [shape=box, style=rounded];\n\n`;
  for (const v of graph.variables || []) {
    const color = v.ownership_category === "SharedRef" ? "#3498db" : v.ownership_category === "MutRef" ? "#e74c3c" : "#2ecc71";
    dot += `  "${v.name}" [label="${v.name}\\n${v.type_display}", color="${color}"];\n`;
  }
  for (const b of graph.borrow_scopes || []) {
    dot += `  "${b.borrower}" -> "${b.owner}" [label="${b.kind}", style=dashed];\n`;
  }
  for (const m of graph.moves || []) {
    dot += `  "${m.source_name}" -> "${m.destination}" [label="move", color="#e67e22"];\n`;
  }
  dot += "}\n";

  const doc = await vscode.workspace.openTextDocument({ content: dot, language: "dot" });
  await vscode.window.showTextDocument(doc);
}

async function exportSvg(): Promise<void> {
  const { GraphPanel } = require("./graph/panel");
  const panel = GraphPanel.getPanel();
  const graph = panel?.getGraph();
  if (!graph) {
    vscode.window.showErrorMessage("BorrowScope: No graph data. Click a CodeLens first.");
    return;
  }

  // Generate simple SVG
  const vars = graph.variables || [];
  const width = Math.max(400, vars.length * 120);
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="300">\n`;
  svg += `  <text x="10" y="20" font-size="14" fill="#333">${graph.function_name}</text>\n`;
  vars.forEach((v: any, i: number) => {
    const x = 20 + i * 110;
    const color = v.ownership_category === "SharedRef" ? "#3498db" : v.ownership_category === "MutRef" ? "#e74c3c" : "#2ecc71";
    svg += `  <rect x="${x}" y="40" width="100" height="60" rx="6" fill="none" stroke="${color}" stroke-width="2"/>\n`;
    svg += `  <text x="${x + 50}" y="65" text-anchor="middle" font-size="11" fill="#333">${v.name}</text>\n`;
    svg += `  <text x="${x + 50}" y="85" text-anchor="middle" font-size="9" fill="#666">${v.type_display}</text>\n`;
  });
  svg += "</svg>\n";

  const doc = await vscode.workspace.openTextDocument({ content: svg, language: "xml" });
  await vscode.window.showTextDocument(doc);
}

function describeGraph(): void {
  const { GraphPanel } = require("./graph/panel");
  const panel = GraphPanel.getPanel();
  const graph = panel?.getGraph();
  if (!graph) {
    vscode.window.showInformationMessage("BorrowScope: No graph data. Click a CodeLens first.");
    return;
  }

  const vars = graph.variables || [];
  const borrows = graph.borrow_scopes || [];
  const moves = graph.moves || [];
  const lines: string[] = [];

  lines.push(`Function: ${graph.function_name}`);
  lines.push(`${vars.length} variables, ${borrows.length} borrows, ${moves.length} moves\n`);

  lines.push("Variables:");
  for (const v of vars) {
    lines.push(`  ${v.name}: ${v.type_display} (${v.ownership_category}, line ${v.line})`);
  }

  if (borrows.length > 0) {
    lines.push("\nBorrows:");
    for (const b of borrows) {
      lines.push(`  ${b.borrower} borrows ${b.owner} (${b.kind}, lines ${b.start_line}-${b.end_line})`);
    }
  }

  if (moves.length > 0) {
    lines.push("\nMoves:");
    for (const m of moves) {
      lines.push(`  ${m.source_name} → ${m.destination} (line ${m.line})`);
    }
  }

  const description = lines.join("\n");
  vscode.window.showInformationMessage("BorrowScope Graph Description", { modal: true, detail: description });
}
