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
exports.getColorForLabel = getColorForLabel;
exports.applyDecorations = applyDecorations;
exports.clearDecorations = clearDecorations;
exports.isEnabled = isEnabled;
exports.disposeDecorations = disposeDecorations;
const vscode = __importStar(require("vscode"));
// Color-coded decoration types by ownership category
const decorationTypes = {};
function getOrCreateDecorationType(label) {
    if (decorationTypes[label])
        return decorationTypes[label];
    const color = getColorForLabel(label);
    const dt = vscode.window.createTextEditorDecorationType({
        after: {
            contentText: ` ${label}`,
            color,
            fontStyle: "italic",
            margin: "0 0 0 0.5em",
        },
    });
    decorationTypes[label] = dt;
    return dt;
}
function getColorForLabel(label) {
    const cfg = vscode.workspace.getConfiguration("borrowscope.colors");
    switch (label) {
        case "[&]":
            return cfg.get("sharedBorrow", "#3498db");
        case "[&mut]":
            return cfg.get("mutableBorrow", "#e74c3c");
        case "[Rc]":
            return cfg.get("rcArc", "#9b59b6");
        case "[Arc]":
            return cfg.get("rcArc", "#9b59b6");
        case "[Cell]":
            return cfg.get("move", "#e67e22");
        case "[*ptr]":
            return cfg.get("drop", "#95a5a6");
        case "[closure]":
            return cfg.get("owned", "#2ecc71");
        default:
            return cfg.get("drop", "#95a5a6");
    }
}
function applyDecorations(editor, hints) {
    if (!isEnabled()) {
        clearDecorations(editor);
        return;
    }
    // Group hints by label
    const grouped = {};
    for (const hint of hints) {
        if (!grouped[hint.label])
            grouped[hint.label] = [];
        const pos = new vscode.Position(hint.line, hint.character);
        grouped[hint.label].push({ range: new vscode.Range(pos, pos) });
    }
    // Clear all existing decorations first
    for (const dt of Object.values(decorationTypes)) {
        editor.setDecorations(dt, []);
    }
    // Apply grouped decorations
    for (const [label, ranges] of Object.entries(grouped)) {
        const dt = getOrCreateDecorationType(label);
        editor.setDecorations(dt, ranges);
    }
}
function clearDecorations(editor) {
    for (const dt of Object.values(decorationTypes)) {
        editor.setDecorations(dt, []);
    }
}
function isEnabled() {
    return vscode.workspace
        .getConfiguration("borrowscope")
        .get("decorations.enabled", true);
}
function disposeDecorations() {
    for (const dt of Object.values(decorationTypes)) {
        dt.dispose();
    }
    for (const key of Object.keys(decorationTypes)) {
        delete decorationTypes[key];
    }
}
//# sourceMappingURL=decorations.js.map