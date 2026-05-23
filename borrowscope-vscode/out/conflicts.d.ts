import * as vscode from "vscode";
export interface ConflictDiagnostic {
    range: {
        start: {
            line: number;
            character: number;
        };
        end: {
            line: number;
            character: number;
        };
    };
    message: string;
    source?: string;
}
export declare function applyConflictDecorations(editor: vscode.TextEditor, diagnostics: ConflictDiagnostic[]): void;
export declare function clearConflictDecorations(editor: vscode.TextEditor): void;
export declare function disposeConflictDecorations(): void;
