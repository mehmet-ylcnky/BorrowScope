import * as vscode from "vscode";
import { MergedVariable } from "./merge-views";
/** Format nanoseconds into human-readable duration */
export declare function formatDuration(ns: number): string;
/** Build runtime timing decorations (green) for matched variables */
export declare function buildTimingDecorations(merged: MergedVariable[]): vscode.DecorationOptions[];
/** Build divergence decorations (red) for variables where static ≠ runtime */
export declare function buildDivergenceDecorations(merged: MergedVariable[]): vscode.DecorationOptions[];
/** Build drop order decorations */
export declare function buildDropOrderDecorations(merged: MergedVariable[]): vscode.DecorationOptions[];
/** Build ref count decorations for Rc/Arc variables */
export declare function buildRefCountDecorations(merged: MergedVariable[]): vscode.DecorationOptions[];
/** Apply all runtime decorations to an editor */
export declare function applyRuntimeDecorations(editor: vscode.TextEditor, merged: MergedVariable[], decorationTypes: {
    timing: vscode.TextEditorDecorationType;
    divergence: vscode.TextEditorDecorationType;
    dropOrder: vscode.TextEditorDecorationType;
    refCount: vscode.TextEditorDecorationType;
}): void;
/** Clear all runtime decorations */
export declare function clearRuntimeDecorations(editor: vscode.TextEditor, decorationTypes: {
    timing: vscode.TextEditorDecorationType;
    divergence: vscode.TextEditorDecorationType;
    dropOrder: vscode.TextEditorDecorationType;
    refCount: vscode.TextEditorDecorationType;
}): void;
/** Create decoration types for runtime overlay */
export declare function createRuntimeDecorationTypes(): {
    timing: vscode.TextEditorDecorationType;
    divergence: vscode.TextEditorDecorationType;
    dropOrder: vscode.TextEditorDecorationType;
    refCount: vscode.TextEditorDecorationType;
};
