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
exports.applyCrossLifelines = applyCrossLifelines;
exports.clearCrossLifelines = clearCrossLifelines;
const vscode = __importStar(require("vscode"));
const crossLifelineDecoration = vscode.window.createTextEditorDecorationType({
    isWholeLine: true,
});
function applyCrossLifelines(editor, crossBorrows) {
    const decorations = [];
    const cursorLine = editor.selection.active.line;
    // Only show connections for the function the cursor is in
    let cursorFn = "";
    for (let i = cursorLine; i >= 0; i--) {
        const match = editor.document.lineAt(i).text.match(/\bfn\s+(\w+)/);
        if (match) {
            cursorFn = match[1];
            break;
        }
    }
    for (const cb of crossBorrows) {
        if (cb.path.length < 2)
            continue;
        const origin = cb.path[0];
        const target = cb.path[1];
        // Only render if this borrow originates from the cursor's function
        if (origin.function_name !== cursorFn)
            continue;
        const isMut = target.is_mutable;
        const color = isMut ? "rgba(231, 76, 60, 0.5)" : "rgba(26, 188, 156, 0.5)";
        // Find the callee function's line in the document
        const calleeLine = findFunctionLine(editor, target.function_name);
        if (calleeLine < 0)
            continue;
        const originLine = (origin.start_line || 1) - 1;
        // Draw dashed connection between caller and callee
        // Lines between the call and the callee function declaration
        const startLine = Math.min(originLine, calleeLine);
        const endLine = Math.max(originLine, calleeLine);
        for (let line = startLine + 1; line < endLine; line++) {
            decorations.push({
                range: new vscode.Range(line, 0, line, 0),
                renderOptions: {
                    before: {
                        contentText: "┊ ",
                        color,
                    },
                },
            });
        }
        // Annotation on callee's function line: "◄── received from caller"
        decorations.push({
            range: new vscode.Range(calleeLine, 0, calleeLine, 0),
            renderOptions: {
                after: {
                    contentText: ` ◄── ${isMut ? "🔒" : "👁"} receives &${cb.origin_variable} from ${origin.function_name}()`,
                    color,
                    fontStyle: "italic",
                    margin: "0 0 0 2em",
                },
            },
            hoverMessage: `**Cross-function borrow**\n\n\`${target.variable}\` in \`${target.function_name}()\` receives \`&${cb.origin_variable}\` from \`${origin.function_name}()\``,
        });
    }
    editor.setDecorations(crossLifelineDecoration, decorations);
}
function clearCrossLifelines(editor) {
    editor.setDecorations(crossLifelineDecoration, []);
}
function findFunctionLine(editor, fnName) {
    for (let i = 0; i < editor.document.lineCount; i++) {
        if (new RegExp(`\\bfn\\s+${fnName}\\b`).test(editor.document.lineAt(i).text)) {
            return i;
        }
    }
    return -1;
}
//# sourceMappingURL=cross-lifelines.js.map