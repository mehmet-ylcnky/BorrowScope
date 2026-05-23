"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.applyConflictDecorations = applyConflictDecorations;
exports.clearConflictDecorations = clearConflictDecorations;
exports.disposeConflictDecorations = disposeConflictDecorations;
const vscode = __importStar(require("vscode"));
const conflictDecoration = vscode.window.createTextEditorDecorationType({
    textDecoration: "underline wavy rgba(241, 196, 15, 0.8)",
    overviewRulerColor: "rgba(241, 196, 15, 0.8)",
    overviewRulerLane: vscode.OverviewRulerLane.Center,
});
function applyConflictDecorations(editor, diagnostics) {
    const borrowScopeDiags = diagnostics.filter((d) => d.source === "BorrowScope");
    if (borrowScopeDiags.length === 0) {
        clearConflictDecorations(editor);
        return;
    }
    const decorations = borrowScopeDiags.map((d) => ({
        range: new vscode.Range(d.range.start.line, d.range.start.character, d.range.end.line, d.range.end.character),
        hoverMessage: `⚠️ ${d.message}`,
    }));
    editor.setDecorations(conflictDecoration, decorations);
}
function clearConflictDecorations(editor) {
    editor.setDecorations(conflictDecoration, []);
}
function disposeConflictDecorations() {
    conflictDecoration.dispose();
}
//# sourceMappingURL=conflicts.js.map