import * as vscode from "vscode";
export interface BorrowScope {
    borrower: string;
    target: string;
    is_mutable: boolean;
    range: {
        start: {
            line: number;
        };
        end: {
            line: number;
        };
    };
}
export interface OwnershipGraph {
    function_name: string;
    start_line: number;
    end_line: number;
    variables: Array<{
        name: string;
        line: number;
        ownership_category: string;
    }>;
    borrow_scopes: BorrowScope[];
    moves: Array<{
        source_name: string;
        line: number;
        destination: any;
        source_type: string;
    }>;
    rc_clones: Array<{
        clone_variable: string;
        source_variable: string;
        clone_type: string;
        line: number;
    }>;
    conflicts: Array<{
        variable: string;
        borrow_a: string;
        borrow_b: string;
        overlap_start_line: number;
        overlap_end_line: number;
    }>;
}
interface LineDecoration {
    line: number;
    char: string;
    suffix: string;
    color: string;
    hover: string;
}
export declare function buildLifelineDecorations(scopes: BorrowScope[], graph?: OwnershipGraph): LineDecoration[];
export declare function applyLifelines(editor: vscode.TextEditor, scopes: BorrowScope[], graph?: OwnershipGraph): void;
export declare function clearLifelines(editor: vscode.TextEditor): void;
export declare function isLifelinesEnabled(): boolean;
export declare function disposeLifelines(): void;
export {};
