import * as vscode from "vscode";
import { BorrowScope, OwnershipGraph } from "./lifelines";
export declare function applyHighlights(editor: vscode.TextEditor, scopes: BorrowScope[], graph?: OwnershipGraph): void;
export declare function clearHighlights(editor: vscode.TextEditor): void;
export declare function isHighlightsEnabled(): boolean;
export declare function disposeHighlights(): void;
