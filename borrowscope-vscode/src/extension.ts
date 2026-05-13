import * as vscode from "vscode";

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext): void {
  outputChannel = vscode.window.createOutputChannel("BorrowScope");
  outputChannel.appendLine("BorrowScope activated");

  context.subscriptions.push(outputChannel);

  // Register commands (implementations added in later steps)
  context.subscriptions.push(
    vscode.commands.registerCommand("borrowscope.showGraph", showGraph),
    vscode.commands.registerCommand("borrowscope.inspectVariable", inspectVariable),
    vscode.commands.registerCommand("borrowscope.toggleDecorations", toggleDecorations),
    vscode.commands.registerCommand("borrowscope.restartServer", restartServer)
  );
}

export function deactivate(): void {
  if (outputChannel) {
    outputChannel.appendLine("BorrowScope deactivated");
  }
}

function showGraph(): void {
  vscode.window.showInformationMessage("BorrowScope: Show Ownership Graph");
}

function inspectVariable(): void {
  vscode.window.showInformationMessage("BorrowScope: Inspect Variable");
}

function toggleDecorations(): void {
  const config = vscode.workspace.getConfiguration("borrowscope");
  const current = config.get<boolean>("decorations.enabled", true);
  config.update("decorations.enabled", !current, vscode.ConfigurationTarget.Global);
}

function restartServer(): void {
  vscode.window.showInformationMessage("BorrowScope: Server restart (not yet implemented)");
}
