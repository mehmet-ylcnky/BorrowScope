import * as vscode from "vscode";
import { getClient } from "./client";
import { GraphPanel } from "./graph/panel";

let extensionUri: vscode.Uri;

export function registerCommands(context: vscode.ExtensionContext): void {
  extensionUri = context.extensionUri;
  GraphPanel.setContext(context);
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

  // Find the line for the function — either from name or cursor
  let line = editor.selection.active.line;
  if (functionName && editor.document.uri.toString() === targetUri) {
    // Search for the function declaration in the document
    for (let i = 0; i < editor.document.lineCount; i++) {
      const text = editor.document.lineAt(i).text;
      if (new RegExp(`\\bfn\\s+${functionName}\\b`).test(text)) {
        line = i;
        break;
      }
    }
  }

  try {
    console.log(`[BorrowScope] showGraph: uri=${targetUri}, line=${line}, fn=${functionName}`);
    const graph = await client.sendRequest("borrowscope/ownershipGraph", {
      textDocument: { uri: targetUri },
      position: { line, character: 4 },
    }) as any;

    console.log(`[BorrowScope] showGraph response:`, graph ? `${graph.function_name} (${(graph.variables||[]).length} vars)` : "null");

    if (!graph) {
      vscode.window.showInformationMessage("BorrowScope: No function at cursor position");
      return;
    }

    // Open the graph panel with function list
    const fnList: string[] = [];
    for (let i = 0; i < editor.document.lineCount; i++) {
      const match = editor.document.lineAt(i).text.match(/\bfn\s+(\w+)/);
      if (match) fnList.push(match[1]);
    }
    GraphPanel.createOrShow(extensionUri, graph, fnList);

    // Fetch cross-function borrows for this function and attach to graph
    try {
      const crossResponse = await client.sendRequest("borrowscope/crossFunctionBorrows", {
        textDocument: { uri: targetUri },
      }) as any;
      const allCross = crossResponse?.cross_borrows || [];
      const fnCross = allCross.filter((b: any) => b.path.length > 0 && b.path[0].function_name === (functionName || graph.function_name));
      if (fnCross.length > 0) {
        graph._crossRefs = fnCross;
        GraphPanel.getPanel()?.updateGraph(graph, fnList);
      }
    } catch { /* ignore */ }


  } catch (e: any) {
    console.error(`[BorrowScope] showGraph error:`, e);
    vscode.window.showErrorMessage(`BorrowScope: ${e.message}`);
  }
}
