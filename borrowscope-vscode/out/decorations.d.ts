import * as vscode from "vscode";
export interface OwnershipHint {
    line: number;
    character: number;
    label: string;
}
export declare function getColorForLabel(label: string): string;
export declare function applyDecorations(editor: vscode.TextEditor, hints: OwnershipHint[]): void;
export declare function clearDecorations(editor: vscode.TextEditor): void;
export declare function isEnabled(): boolean;
export declare function disposeDecorations(): void;
