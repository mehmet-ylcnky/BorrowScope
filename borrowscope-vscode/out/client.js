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
exports.getClient = getClient;
exports.startClient = startClient;
exports.refreshInlayHints = refreshInlayHints;
exports.refreshDecorations = refreshDecorations;
exports.stopClient = stopClient;
exports.restartClient = restartClient;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
const server_path_1 = require("./server-path");
const decorations_1 = require("./decorations");
const lifelines_1 = require("./lifelines");
const highlights_1 = require("./highlights");
const conflicts_1 = require("./conflicts");
const panel_1 = require("./graph/panel");
let client;
let lastGraph = undefined;
let lastGraphFn = "";
let lastGraphUri = "";
const graphCache = new Map(); // fnName -> graph
function getClient() {
    return client;
}
async function startClient(context) {
    const configured = vscode.workspace
        .getConfiguration("borrowscope.server")
        .get("path", "");
    const serverPath = (0, server_path_1.resolveServerPath)({
        extensionPath: context.extensionPath,
        configuredPath: configured,
        globalStoragePath: context.globalStorageUri?.fsPath,
    });
    const serverOptions = {
        run: { command: serverPath, transport: node_1.TransportKind.stdio },
        debug: {
            command: serverPath,
            transport: node_1.TransportKind.stdio,
            options: { env: { ...process.env, RUST_LOG: "borrowscope_lsp=debug" } },
        },
    };
    const clientOptions = {
        documentSelector: [{ scheme: "file", language: "rust" }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/*.rs"),
        },
        initializationOptions: {
            decorations: vscode.workspace
                .getConfiguration("borrowscope")
                .get("decorations.enabled", true),
        },
    };
    client = new node_1.LanguageClient("borrowscope", "BorrowScope Language Server", serverOptions, clientOptions);
    await client.start();
    // Listen for analysisUpdated and refresh decorations + panel
    client.onNotification("borrowscope/analysisUpdated", async (params) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.uri.toString() !== params.uri)
            return;
        lastGraph = undefined;
        lastGraphFn = "";
        graphCache.clear(); // invalidate cache
        refreshInlayHints(editor);
        refreshDecorations(editor);
        // Live update the graph panel if open
        const panel = panel_1.GraphPanel.getPanel();
        if (panel && panel.getGraph()) {
            const currentFn = panel.getGraph().function_name;
            if (currentFn && params.functions?.includes(currentFn)) {
                try {
                    // Find the function line
                    for (let i = 0; i < editor.document.lineCount; i++) {
                        if (new RegExp(`\\bfn\\s+${currentFn}\\b`).test(editor.document.lineAt(i).text)) {
                            const graph = await client.sendRequest("borrowscope/ownershipGraph", {
                                textDocument: { uri: params.uri },
                                position: { line: i, character: 4 },
                            });
                            if (graph)
                                panel.updateGraph(graph);
                            break;
                        }
                    }
                }
                catch { /* ignore */ }
            }
        }
    });
    // Listen for diagnostics to apply inline conflict markers
    client.onNotification("textDocument/publishDiagnostics", (params) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.uri.toString() !== params.uri)
            return;
        (0, conflicts_1.applyConflictDecorations)(editor, params.diagnostics || []);
    });
    // Apply decorations on active editor change (debounced)
    let editorChangeTimer;
    context.subscriptions.push(vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editorChangeTimer)
            clearTimeout(editorChangeTimer);
        editorChangeTimer = setTimeout(() => {
            if (editor && editor.document.languageId === "rust") {
                refreshDecorations(editor);
            }
        }, 150);
    }));
    // Refresh when a document is opened
    context.subscriptions.push(vscode.workspace.onDidOpenTextDocument((doc) => {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document === doc && doc.languageId === "rust") {
            setTimeout(() => { refreshInlayHints(editor); refreshDecorations(editor); }, 500);
        }
    }));
    // Debounced refresh on text change
    context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((e) => {
        const editor = vscode.window.activeTextEditor;
        if (editor && e.document === editor.document && e.document.languageId === "rust") {
            setTimeout(() => refreshDecorations(editor), 300);
        }
    }));
    // Initial decoration — retry until workspace is loaded
    const initialRefresh = () => {
        const editor = vscode.window.activeTextEditor;
        if (editor?.document.languageId === "rust") {
            refreshInlayHints(editor);
            refreshDecorations(editor);
        }
    };
    // Try immediately, then retry at intervals during workspace loading
    initialRefresh();
    setTimeout(initialRefresh, 5000);
    setTimeout(initialRefresh, 15000);
    setTimeout(initialRefresh, 30000);
    // Pre-fetch all functions in background (warms Salsa cache)
    // Triggered after first successful analysisUpdated (workspace loaded)
    let preFetched = false;
    const preFetchAll = async () => {
        if (preFetched || !client)
            return;
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== "rust")
            return;
        preFetched = true;
        const uri = editor.document.uri.toString();
        const fnLines = [];
        for (let i = 0; i < editor.document.lineCount; i++) {
            if (/\bfn\s+\w+/.test(editor.document.lineAt(i).text))
                fnLines.push(i);
        }
        // Fire all in parallel for maximum speed
        const results = await Promise.all(fnLines.map(line => client.sendRequest("borrowscope/ownershipGraph", {
            textDocument: { uri },
            position: { line, character: 4 },
        }).catch(() => null)));
        results.forEach((r) => {
            if (r && r.function_name)
                graphCache.set(r.function_name, r);
        });
    };
    // Try pre-fetch at multiple points
    setTimeout(preFetchAll, 10000);
    setTimeout(preFetchAll, 20000);
    return client;
}
async function refreshInlayHints(editor) {
    if (!client)
        return;
    try {
        const hintsResponse = await client.sendRequest("textDocument/inlayHint", {
            textDocument: { uri: editor.document.uri.toString() },
            range: {
                start: { line: 0, character: 0 },
                end: { line: editor.document.lineCount, character: 0 },
            },
        });
        const hints = (hintsResponse || []).map((h) => ({
            line: h.position.line,
            character: h.position.character,
            label: typeof h.label === "string" ? h.label.trim() : "",
        }));
        (0, decorations_1.applyDecorations)(editor, hints);
    }
    catch { /* ignore */ }
}
async function refreshDecorations(editor) {
    if (!client)
        return;
    try {
        // Fetch borrow scopes and ownership graph in parallel
        const uri = editor.document.uri.toString();
        const cursorLine = editor.selection.active.line;
        // Determine which function cursor is in
        let fnName = "";
        for (let i = cursorLine; i >= 0; i--) {
            const match = editor.document.lineAt(i).text.match(/\bfn\s+(\w+)/);
            if (match) {
                fnName = match[1];
                break;
            }
        }
        const scopesPromise = client.sendRequest("borrowscope/borrowScopes", {
            textDocument: { uri },
        });
        let graphPromise = Promise.resolve(lastGraph);
        if (fnName && (fnName !== lastGraphFn || uri !== lastGraphUri)) {
            // Check local cache first
            if (graphCache.has(fnName)) {
                graphPromise = Promise.resolve(graphCache.get(fnName));
            }
            else {
                let fnLine = cursorLine;
                for (let i = 0; i < editor.document.lineCount; i++) {
                    if (new RegExp(`\\bfn\\s+${fnName}\\b`).test(editor.document.lineAt(i).text)) {
                        fnLine = i;
                        break;
                    }
                }
                graphPromise = client.sendRequest("borrowscope/ownershipGraph", {
                    textDocument: { uri },
                    position: { line: fnLine, character: 4 },
                }).catch(() => lastGraph);
            }
        }
        const [scopesResponse, graph] = await Promise.all([scopesPromise, graphPromise]);
        if (graph && graph !== lastGraph) {
            lastGraph = graph;
            lastGraphFn = fnName;
            lastGraphUri = uri;
            if (fnName)
                graphCache.set(fnName, graph);
        }
        const scopes = scopesResponse?.scopes || [];
        (0, lifelines_1.applyLifelines)(editor, scopes, graph || lastGraph);
        (0, highlights_1.applyHighlights)(editor, scopes, graph || lastGraph);
        // Cross-function borrows: only show annotations (not dashed lines)
        // Dashed lifelines shown on demand via CodeLens click
        try {
            const crossResponse = await client.sendRequest("borrowscope/crossFunctionBorrows", {
                textDocument: { uri },
            });
            const crossBorrows = crossResponse?.cross_borrows || [];
            if (crossBorrows.length > 0) {
                applyCrossFunctionAnnotations(editor, crossBorrows);
            }
        }
        catch {
        }
    }
    catch {
        (0, decorations_1.clearDecorations)(editor);
        (0, lifelines_1.clearLifelines)(editor);
        (0, highlights_1.clearHighlights)(editor);
    }
}
// Cross-function borrow annotations
const crossFnDecorationType = vscode.window.createTextEditorDecorationType({
    isWholeLine: true,
});
function applyCrossFunctionAnnotations(editor, crossBorrows) {
    const decorations = crossBorrows.map((b) => {
        const line = (b.origin_line || 1) - 1;
        const targetFn = b.path.length > 1 ? b.path[1].function_name : "?";
        const paramName = b.path.length > 1 ? b.path[1].variable : "?";
        const isMut = b.path.length > 1 && b.path[1].is_mutable;
        return {
            range: new vscode.Range(line, 0, line, 0),
            renderOptions: {
                after: {
                    contentText: ` ──→ ${isMut ? "🔒" : "👁"} &${b.origin_variable} enters ${targetFn}(${paramName})`,
                    color: "rgba(26, 188, 156, 0.6)",
                    fontStyle: "italic",
                    margin: "0 0 0 2em",
                },
            },
            hoverMessage: `**Cross-function borrow**\n\n\`${b.origin_variable}\` is passed as \`${isMut ? "&mut " : "&"}${paramName}\` to \`${targetFn}()\``,
        };
    });
    editor.setDecorations(crossFnDecorationType, decorations);
}
async function stopClient() {
    if (client) {
        await client.stop();
        client = undefined;
    }
}
async function restartClient(context) {
    await stopClient();
    await startClient(context);
}
//# sourceMappingURL=client.js.map