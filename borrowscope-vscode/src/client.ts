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

  // Listen for analysisUpdated and refresh decorations
  client.onNotification("borrowscope/analysisUpdated", (params: any) => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.uri.toString() !== params.uri) return;
    refreshDecorations(editor);
  });

  // Listen for diagnostics to apply inline conflict markers
  client.onNotification("textDocument/publishDiagnostics", (params: any) => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.uri.toString() !== params.uri) return;
    applyConflictDecorations(editor, params.diagnostics || []);
  });

  // Apply decorations on active editor change
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor && editor.document.languageId === "rust") {
        refreshDecorations(editor);
      }
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

    // Fetch ownership graphs for all functions in parallel
    let graph: OwnershipGraph | undefined;
    try {
      const fnLines: number[] = [];
      for (let line = 0; line < editor.document.lineCount; line++) {
        const text = editor.document.lineAt(line).text;
        if (/^\s*(pub\s+)?(async\s+)?fn\s+/.test(text)) {
          fnLines.push(line);
        }
      }

      const promises = fnLines.map((line) =>
        client!.sendRequest("borrowscope/ownershipGraph", {
          textDocument: { uri: editor.document.uri.toString() },
          position: { line, character: 4 },
        }).catch(() => null)
      );

      const results = await Promise.all(promises);
      const graphs = results.filter(Boolean) as OwnershipGraph[];

      if (graphs.length > 0) {
        graph = {
          function_name: "all",
          start_line: Math.min(...graphs.map(g => g.start_line)),
          end_line: Math.max(...graphs.map(g => g.end_line)),
          variables: graphs.flatMap(g => g.variables || []),
          borrow_scopes: graphs.flatMap(g => g.borrow_scopes || []),
          moves: graphs.flatMap(g => g.moves || []),
          rc_clones: graphs.flatMap(g => g.rc_clones || []),
          conflicts: graphs.flatMap(g => g.conflicts || []),
        };
      }
    } catch { /* graph not available */ }

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
