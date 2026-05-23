import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
export declare function getClient(): LanguageClient | undefined;
export declare function startClient(context: vscode.ExtensionContext): Promise<LanguageClient>;
export declare function refreshInlayHints(editor: vscode.TextEditor): Promise<void>;
export declare function refreshDecorations(editor: vscode.TextEditor): Promise<void>;
export declare function stopClient(): Promise<void>;
export declare function restartClient(context: vscode.ExtensionContext): Promise<void>;
