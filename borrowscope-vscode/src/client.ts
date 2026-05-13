import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerPath } from "./server-path";

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
  return client;
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
