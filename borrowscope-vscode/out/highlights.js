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
exports.applyHighlights = applyHighlights;
exports.clearHighlights = clearHighlights;
exports.isHighlightsEnabled = isHighlightsEnabled;
exports.disposeHighlights = disposeHighlights;
const vscode = __importStar(require("vscode"));
const sharedHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: "rgba(52, 152, 219, 0.06)",
    isWholeLine: true,
});
const mutHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: "rgba(231, 76, 60, 0.06)",
    isWholeLine: true,
});
const conflictHighlight = vscode.window.createTextEditorDecorationType({
    backgroundColor: "rgba(241, 196, 15, 0.10)",
    isWholeLine: true,
});
function applyHighlights(editor, scopes, graph) {
    if (!isHighlightsEnabled()) {
        clearHighlights(editor);
        return;
    }
    const shared = [];
    const mutable = [];
    const conflicts = [];
    for (const s of scopes) {
        const range = new vscode.Range(s.range.start.line, 0, s.range.end.line, Number.MAX_SAFE_INTEGER);
        const dec = {
            range,
            hoverMessage: `${s.is_mutable ? "&mut" : "&"} borrow of \`${s.target}\` by \`${s.borrower}\``,
        };
        if (s.is_mutable) {
            mutable.push(dec);
        }
        else {
            shared.push(dec);
        }
    }
    if (graph) {
        for (const c of graph.conflicts) {
            const range = new vscode.Range(c.overlap_start_line - 1, 0, c.overlap_end_line - 1, Number.MAX_SAFE_INTEGER);
            conflicts.push({
                range,
                hoverMessage: `Conflict: \`${c.borrow_a}\` and \`${c.borrow_b}\` overlap on \`${c.variable}\``,
            });
        }
    }
    editor.setDecorations(sharedHighlight, shared);
    editor.setDecorations(mutHighlight, mutable);
    editor.setDecorations(conflictHighlight, conflicts);
}
function clearHighlights(editor) {
    editor.setDecorations(sharedHighlight, []);
    editor.setDecorations(mutHighlight, []);
    editor.setDecorations(conflictHighlight, []);
}
function isHighlightsEnabled() {
    return vscode.workspace
        .getConfiguration("borrowscope")
        .get("decorations.borrowScopes", true);
}
function disposeHighlights() {
    sharedHighlight.dispose();
    mutHighlight.dispose();
    conflictHighlight.dispose();
}
//# sourceMappingURL=highlights.js.map