import * as vscode from "vscode";
import { getClient } from "./client";

export function registerCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "borrowscope.showGraph",
      showGraphCommand
    )
  );
}

async function showGraphCommand(uri?: string, functionName?: string): Promise<void> {
  const client = getClient();
  if (!client) {
    vscode.window.showWarningMessage("BorrowScope: Server not running");
    return;
  }

  const editor = vscode.window.activeTextEditor;
  if (!editor) return;

  const targetUri = uri || editor.document.uri.toString();
  const line = editor.selection.active.line;

  try {
    const graph = await client.sendRequest("borrowscope/ownershipGraph", {
      textDocument: { uri: targetUri },
      position: { line, character: 0 },
    }) as any;

    if (!graph) {
      vscode.window.showInformationMessage("BorrowScope: No function at cursor position");
      return;
    }

    // Show summary (graph panel is Milestone 5)
    const stats = graph.stats || {};
    const conflicts = graph.conflicts || [];
    const moves = graph.moves || [];

    let message = `**${graph.function_name}** — `;
    message += `${stats.total_variables || 0} vars, `;
    message += `${stats.total_borrows || 0} borrows, `;
    message += `${stats.moves || 0} moves`;
    if (conflicts.length > 0) {
      message += `, ⚠️ ${conflicts.length} conflicts`;
    }

    const detail = buildDetail(graph);
    const action = await vscode.window.showInformationMessage(
      message,
      { modal: false },
      "Copy JSON",
      "Show Variables"
    );

    if (action === "Copy JSON") {
      await vscode.env.clipboard.writeText(JSON.stringify(graph, null, 2));
      vscode.window.showInformationMessage("Ownership graph copied to clipboard");
    } else if (action === "Show Variables") {
      const output = vscode.window.createOutputChannel("BorrowScope Graph");
      output.clear();
      output.appendLine(detail);
      output.show();
    }
  } catch (e: any) {
    vscode.window.showErrorMessage(`BorrowScope: ${e.message}`);
  }
}

function buildDetail(graph: any): string {
  let out = `=== ${graph.function_name} ===\n\n`;

  out += `Variables (${(graph.variables || []).length}):\n`;
  for (const v of graph.variables || []) {
    out += `  ${v.name}: ${v.type_display} [${v.ownership_category}]\n`;
  }

  out += `\nBorrow Scopes (${(graph.borrow_scopes || []).length}):\n`;
  for (const s of graph.borrow_scopes || []) {
    out += `  ${s.borrower_name} ${s.is_mutable ? "&mut" : "&"} ${s.target_name} (lines ${s.start_line}-${s.end_line})\n`;
  }

  out += `\nMoves (${(graph.moves || []).length}):\n`;
  for (const m of graph.moves || []) {
    out += `  ${m.source_name} → ${JSON.stringify(m.destination)} (line ${m.line})\n`;
  }

  if ((graph.conflicts || []).length > 0) {
    out += `\nConflicts (${graph.conflicts.length}):\n`;
    for (const c of graph.conflicts) {
      out += `  ⚠️ ${c.borrow_a} and ${c.borrow_b} overlap on ${c.variable}\n`;
    }
  }

  return out;
}
