import * as vscode from "vscode";
import { startClient, stopClient, restartClient } from "./client";
import { registerCommands } from "./commands";

let outputChannel: vscode.OutputChannel;

export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  outputChannel = vscode.window.createOutputChannel("BorrowScope");
  outputChannel.appendLine("BorrowScope activated");
  context.subscriptions.push(outputChannel);

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
    )
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
}

export async function deactivate(): Promise<void> {
  await stopClient();
  if (outputChannel) {
    outputChannel.appendLine("BorrowScope deactivated");
  }
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
