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
exports.registerCommands = registerCommands;
const vscode = __importStar(require("vscode"));
const client_1 = require("./client");
const panel_1 = require("./graph/panel");
let extensionUri;
function registerCommands(context) {
    extensionUri = context.extensionUri;
    panel_1.GraphPanel.setContext(context);
    context.subscriptions.push(vscode.commands.registerCommand("borrowscope.showGraph", showGraphCommand));
}
async function showGraphCommand(uri, functionName) {
    const client = (0, client_1.getClient)();
    if (!client) {
        vscode.window.showWarningMessage("BorrowScope: Server not running");
        return;
    }
    const editor = vscode.window.activeTextEditor;
    if (!editor)
        return;
    const targetUri = uri || editor.document.uri.toString();
    // Find the line for the function — either from name or cursor
    let line = editor.selection.active.line;
    if (functionName && editor.document.uri.toString() === targetUri) {
        // Search for the function declaration in the document
        for (let i = 0; i < editor.document.lineCount; i++) {
            const text = editor.document.lineAt(i).text;
            if (new RegExp(`\\bfn\\s+${functionName}\\b`).test(text)) {
                line = i;
                break;
            }
        }
    }
    try {
        console.log(`[BorrowScope] showGraph: uri=${targetUri}, line=${line}, fn=${functionName}`);
        const start = performance.now();
        const graph = await client.sendRequest("borrowscope/ownershipGraph", {
            textDocument: { uri: targetUri },
            position: { line, character: 4 },
        });
        const elapsed = performance.now() - start;
        try {
            const { getPerfMonitor } = require("./extension");
            getPerfMonitor()?.record("ownershipGraph", elapsed);
        }
        catch { }
        console.log(`[BorrowScope] showGraph response:`, graph ? `${graph.function_name} (${(graph.variables || []).length} vars) [${elapsed.toFixed(0)}ms]` : "null");
        if (!graph) {
            vscode.window.showInformationMessage("BorrowScope: No function at cursor position");
            return;
        }
        // Open the graph panel with function list
        const fnList = [];
        for (let i = 0; i < editor.document.lineCount; i++) {
            const match = editor.document.lineAt(i).text.match(/\bfn\s+(\w+)/);
            if (match)
                fnList.push(match[1]);
        }
        panel_1.GraphPanel.createOrShow(extensionUri, graph, fnList);
        // Fetch cross-function borrows for this function and attach to graph
        try {
            const t0 = performance.now();
            const crossResponse = await client.sendRequest("borrowscope/crossFunctionBorrows", {
                textDocument: { uri: targetUri },
            });
            try {
                const { getPerfMonitor } = require("./extension");
                getPerfMonitor()?.record("crossFunctionBorrows", performance.now() - t0);
            }
            catch { }
            const allCross = crossResponse?.cross_borrows || [];
            const fnCross = allCross.filter((b) => b.path.length > 0 && b.path[0].function_name === (functionName || graph.function_name));
            if (fnCross.length > 0) {
                graph._crossRefs = fnCross;
            }
        }
        catch { /* ignore */ }
        // Fetch memory layout for this function
        try {
            const t1 = performance.now();
            const memLayout = await client.sendRequest("borrowscope/memoryLayout", {
                textDocument: { uri: targetUri },
                position: { line, character: 4 },
            });
            try {
                const { getPerfMonitor } = require("./extension");
                getPerfMonitor()?.record("memoryLayout", performance.now() - t1);
            }
            catch { }
            if (memLayout)
                graph._memoryLayout = memLayout;
        }
        catch { /* ignore */ }
        // Attach runtime events if available
        try {
            const { getRuntimeWatcher } = require("./extension");
            const watcher = getRuntimeWatcher();
            if (watcher && watcher.eventCount > 0) {
                graph._runtimeEvents = watcher.getEvents();
            }
        }
        catch { /* ignore */ }
        // Attach runtime memory data if available
        try {
            const fs = require("fs");
            const path = require("path");
            const wsRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (wsRoot) {
                const memFile = path.join(wsRoot, ".borrowscope", "memory-events.json");
                if (fs.existsSync(memFile)) {
                    const memData = JSON.parse(fs.readFileSync(memFile, "utf8"));
                    if (memData.function === graph.function_name || !memData.function) {
                        graph._memoryRuntime = memData;
                    }
                }
            }
        }
        catch { /* ignore */ }
        panel_1.GraphPanel.getPanel()?.updateGraph(graph, fnList);
    }
    catch (e) {
        console.error(`[BorrowScope] showGraph error:`, e);
        vscode.window.showErrorMessage(`BorrowScope: ${e.message}`);
    }
}
//# sourceMappingURL=commands.js.map