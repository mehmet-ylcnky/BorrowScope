import * as vscode from "vscode";
import { getClient } from "./client";
import { GraphPanel } from "./graph/panel";

let extensionUri: vscode.Uri;

export function registerCommands(context: vscode.ExtensionContext): void {
  extensionUri = context.extensionUri;
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

    // Open the graph panel
    GraphPanel.createOrShow(extensionUri, graph);
  } catch (e: any) {
    vscode.window.showErrorMessage(`BorrowScope: ${e.message}`);
  }
}
