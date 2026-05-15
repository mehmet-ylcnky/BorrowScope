import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerPath } from "./server-path";
import { applyDecorations, clearDecorations, OwnershipHint } from "./decorations";
import { applyLifelines, clearLifelines, BorrowScope, OwnershipGraph } from "./lifelines";
import { applyHighlights, clearHighlights } from "./highlights";
import { applyConflictDecorations, clearConflictDecorations } from "./conflicts";
import { GraphPanel } from "./graph/panel";

let client: LanguageClient | undefined;

export function getClient(): LanguageClient | undefined {
  return client;
}

export async function startClient(
  context: vscode.ExtensionContext
): Promise<LanguageClient> {
  const configured = vscode.workspace
    .getConfiguration("borrowscope.server")
    .get<string>("path", "");
  const serverPath = resolveServerPath({
    extensionPath: context.extensionPath,
    configuredPath: configured,
    globalStoragePath: context.globalStorageUri?.fsPath,
  });

  const serverOptions: ServerOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
      options: { env: { ...process.env, RUST_LOG: "borrowscope_lsp=debug" } },
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "rust" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.rs"),
    },
    initializationOptions: {
      decorations: vscode.workspace
        .getConfiguration("borrowscope")
        .get("decorations.enabled", true),
    },
  };

  client = new LanguageClient(
    "borrowscope",
    "BorrowScope Language Server",
    serverOptions,
    clientOptions
  );

  await client.start();

  // Listen for analysisUpdated and refresh decorations + panel
  client.onNotification("borrowscope/analysisUpdated", async (params: any) => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.uri.toString() !== params.uri) return;
    refreshDecorations(editor);

    // Live update the graph panel if open
    const panel = GraphPanel.getPanel();
    if (panel && panel.getGraph()) {
      const currentFn = panel.getGraph().function_name;
      if (currentFn && params.functions?.includes(currentFn)) {
        try {
          // Find the function line
          for (let i = 0; i < editor.document.lineCount; i++) {
            if (new RegExp(`\\bfn\\s+${currentFn}\\b`).test(editor.document.lineAt(i).text)) {
              const graph = await client!.sendRequest("borrowscope/ownershipGraph", {
                textDocument: { uri: params.uri },
                position: { line: i, character: 4 },
              });
              if (graph) panel.updateGraph(graph);
              break;
            }
          }
        } catch { /* ignore */ }
      }
    }
  });

  // Listen for diagnostics to apply inline conflict markers
  client.onNotification("textDocument/publishDiagnostics", (params: any) => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.uri.toString() !== params.uri) return;
    applyConflictDecorations(editor, params.diagnostics || []);
  });

  // Apply decorations on active editor change (debounced)
  let editorChangeTimer: NodeJS.Timeout | undefined;
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editorChangeTimer) clearTimeout(editorChangeTimer);
      editorChangeTimer = setTimeout(() => {
        if (editor && editor.document.languageId === "rust") {
          refreshDecorations(editor);
        }
      }, 500);
    })
  );

  // Refresh when a document is opened
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((doc) => {
      const editor = vscode.window.activeTextEditor;
      if (editor && editor.document === doc && doc.languageId === "rust") {
        setTimeout(() => refreshDecorations(editor), 500);
      }
    })
  );

  // Debounced refresh on text change
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => {
      const editor = vscode.window.activeTextEditor;
      if (editor && e.document === editor.document && e.document.languageId === "rust") {
        setTimeout(() => refreshDecorations(editor), 300);
      }
    })
  );

  // Initial decoration — retry until workspace is loaded
  const initialRefresh = () => {
    const editor = vscode.window.activeTextEditor;
    if (editor?.document.languageId === "rust") {
      refreshDecorations(editor);
    }
  };
  // Try immediately, then retry at intervals during workspace loading
  initialRefresh();
  setTimeout(initialRefresh, 5000);
  setTimeout(initialRefresh, 15000);
  setTimeout(initialRefresh, 30000);

  return client;
}

export async function refreshDecorations(editor: vscode.TextEditor): Promise<void> {
  if (!client) return;

  try {
    // Fetch inlay hints for inline annotations
    const hintsResponse = await client.sendRequest("textDocument/inlayHint", {
      textDocument: { uri: editor.document.uri.toString() },
      range: {
        start: { line: 0, character: 0 },
        end: { line: editor.document.lineCount, character: 0 },
      },
    });

    const hints: OwnershipHint[] = ((hintsResponse as any[]) || []).map((h: any) => ({
      line: h.position.line,
      character: h.position.character,
      label: typeof h.label === "string" ? h.label.trim() : "",
    }));

    applyDecorations(editor, hints);

    // Fetch borrow scopes for lifeline flow
    const scopesResponse = await client.sendRequest("borrowscope/borrowScopes", {
      textDocument: { uri: editor.document.uri.toString() },
    });

    const scopes: BorrowScope[] = (scopesResponse as any)?.scopes || [];

    // Fetch ownership graph for cursor function (moves, clones, conflicts)
    let graph: any = undefined;
    try {
      const cursorLine = editor.selection.active.line;
      graph = await client.sendRequest("borrowscope/ownershipGraph", {
        textDocument: { uri: editor.document.uri.toString() },
        position: { line: cursorLine, character: 4 },
      });
    } catch { /* no function at cursor */ }

    applyLifelines(editor, scopes, graph);
    applyHighlights(editor, scopes, graph);
  } catch {
    clearDecorations(editor);
    clearLifelines(editor);
    clearHighlights(editor);
  }
}

export async function stopClient(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

export async function restartClient(
  context: vscode.ExtensionContext
): Promise<void> {
  await stopClient();
  await startClient(context);
}
