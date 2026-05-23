import * as vscode from "vscode";
export interface CrossBorrow {
    origin_variable: string;
    origin_line: number;
    path: CrossPathSegment[];
}
export interface CrossPathSegment {
    file: string;
    function_name: string;
    variable: string;
    start_line: number;
    end_line: number;
    is_mutable: boolean;
    kind: string;
}
export declare function applyCrossLifelines(editor: vscode.TextEditor, crossBorrows: CrossBorrow[]): void;
export declare function clearCrossLifelines(editor: vscode.TextEditor): void;
