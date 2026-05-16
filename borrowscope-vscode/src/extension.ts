import * as vscode from "vscode";
import { startClient, stopClient, restartClient, getClient } from "./client";
import { registerCommands } from "./commands";
import { RuntimeWatcher } from "./runtime-watcher";
import { RuntimeStatusBar, registerRuntimeCommands } from "./runtime-status";
import { parseEvents, filterByFile, filterOwnershipEvents } from "./runtime-parser";
import { mergeViews } from "./merge-views";
import { createRuntimeDecorationTypes, applyRuntimeDecorations, clearRuntimeDecorations } from "./runtime-decorations";

let outputChannel: vscode.OutputChannel;
let runtimeWatcher: RuntimeWatcher | undefined;
let runtimeStatusBar: RuntimeStatusBar | undefined;
let runtimeDecorationTypes: ReturnType<typeof createRuntimeDecorationTypes> | undefined;

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
