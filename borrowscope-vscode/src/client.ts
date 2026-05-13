import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerPath } from "./server-path";
import { applyDecorations, clearDecorations, OwnershipHint } from "./decorations";

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

  // Apply decorations on active editor change
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor && editor.document.languageId === "rust") {
        refreshDecorations(editor);
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

  // Initial decoration for current editor
  if (vscode.window.activeTextEditor?.document.languageId === "rust") {
    refreshDecorations(vscode.window.activeTextEditor);
  }

  return client;
}

export async function refreshDecorations(editor: vscode.TextEditor): Promise<void> {
  if (!client) return;

  try {
    const response = await client.sendRequest("textDocument/inlayHint", {
      textDocument: { uri: editor.document.uri.toString() },
      range: {
        start: { line: 0, character: 0 },
        end: { line: editor.document.lineCount, character: 0 },
      },
    });

    const hints: OwnershipHint[] = ((response as any[]) || []).map((h: any) => ({
      line: h.position.line,
      character: h.position.character,
      label: typeof h.label === "string" ? h.label.trim() : "",
    }));

    applyDecorations(editor, hints);
  } catch {
    clearDecorations(editor);
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
