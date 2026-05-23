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
exports.RuntimeStatusBar = void 0;
exports.registerRuntimeCommands = registerRuntimeCommands;
const vscode = __importStar(require("vscode"));
/** Status bar item showing runtime overlay state */
class RuntimeStatusBar {
    constructor() {
        this.watcher = null;
        this.socket = null;
        this.disposables = [];
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
        this.statusBarItem.command = "borrowscope.toggleRuntime";
        this.update();
        // Re-update when config changes
        this.disposables.push(vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration("borrowscope.runtime")) {
                this.update();
            }
        }));
    }
    /** Attach to a file watcher */
    attachWatcher(watcher) {
        this.watcher = watcher;
        watcher.onEventsUpdated(() => this.update(), null, this.disposables);
        this.update();
    }
    /** Attach to a WebSocket connection */
    attachSocket(socket) {
        this.socket = socket;
        socket.onStateChanged(() => this.update(), null, this.disposables);
        socket.onEvent(() => this.update(), null, this.disposables);
        this.update();
    }
    /** Update the status bar display */
    update() {
        const config = vscode.workspace.getConfiguration("borrowscope.runtime");
        const enabled = config.get("enabled", false);
        if (!enabled) {
            this.statusBarItem.text = "$(check) BorrowScope: Static ✓ | Runtime Off";
            this.statusBarItem.tooltip = "Click to enable runtime overlay";
            this.statusBarItem.show();
            return;
        }
        const source = config.get("source", "file");
        let runtimeStatus = "";
        if (source === "file" && this.watcher) {
            const count = this.watcher.eventCount;
            const stale = this.watcher.isStale;
            if (count > 0) {
                const ago = this.watcher.lastLoadTime > 0
                    ? formatAgo(Date.now() - this.watcher.lastLoadTime)
                    : "";
                runtimeStatus = `Runtime ✓ (${formatCount(count)} events${ago ? ", " + ago : ""})`;
                if (stale)
                    runtimeStatus += " ⚠️stale";
            }
            else {
                runtimeStatus = "Runtime ⏳ (no events)";
            }
        }
        else if (source === "websocket" && this.socket) {
            const state = this.socket.state;
            if (state === "connected") {
                runtimeStatus = `Runtime ✓ Live (${formatCount(this.socket.eventCount)} events)`;
            }
            else if (state === "connecting") {
                runtimeStatus = "Runtime ⏳ connecting...";
            }
            else if (state === "error") {
                runtimeStatus = "Runtime ✗ error";
            }
            else {
                runtimeStatus = "Runtime ✗ disconnected";
            }
        }
        else {
            runtimeStatus = "Runtime ✓ ready";
        }
        this.statusBarItem.text = `$(check) BorrowScope: Static ✓ | ${runtimeStatus}`;
        this.statusBarItem.tooltip = this.buildTooltip();
        this.statusBarItem.show();
    }
    buildTooltip() {
        const config = vscode.workspace.getConfiguration("borrowscope.runtime");
        const lines = ["BorrowScope Runtime Overlay"];
        lines.push(`Source: ${config.get("source", "file")}`);
        lines.push(`Timings: ${config.get("showTimings", true) ? "on" : "off"}`);
        lines.push(`Drop order: ${config.get("showDropOrder", false) ? "on" : "off"}`);
        lines.push(`Ref counts: ${config.get("showRefCounts", true) ? "on" : "off"}`);
        lines.push(`Divergences: ${config.get("highlightDivergences", true) ? "on" : "off"}`);
        if (this.watcher && this.watcher.eventCount > 0) {
            lines.push(`File: ${this.watcher.filePath}`);
            lines.push(`Events: ${this.watcher.eventCount}`);
            if (this.watcher.isStale)
                lines.push("⚠️ Stale (source changed since last run)");
        }
        if (this.socket) {
            lines.push(`WebSocket: ${this.socket.state}`);
            if (this.socket.isConnected)
                lines.push(`Live events: ${this.socket.eventCount}`);
        }
        lines.push("\nClick to toggle runtime overlay");
        return lines.join("\n");
    }
    show() { this.statusBarItem.show(); }
    hide() { this.statusBarItem.hide(); }
    dispose() {
        this.statusBarItem.dispose();
        this.disposables.forEach(d => d.dispose());
    }
}
exports.RuntimeStatusBar = RuntimeStatusBar;
/** Register runtime toggle commands */
function registerRuntimeCommands(context) {
    context.subscriptions.push(vscode.commands.registerCommand("borrowscope.toggleRuntime", toggleRuntime), vscode.commands.registerCommand("borrowscope.runtime.enable", () => setRuntimeEnabled(true)), vscode.commands.registerCommand("borrowscope.runtime.disable", () => setRuntimeEnabled(false)), vscode.commands.registerCommand("borrowscope.runtime.toggleTimings", () => toggleSetting("showTimings")), vscode.commands.registerCommand("borrowscope.runtime.toggleDropOrder", () => toggleSetting("showDropOrder")), vscode.commands.registerCommand("borrowscope.runtime.toggleRefCounts", () => toggleSetting("showRefCounts")), vscode.commands.registerCommand("borrowscope.runtime.toggleDivergences", () => toggleSetting("highlightDivergences")), vscode.commands.registerCommand("borrowscope.runtime.setSource", setSource));
}
async function toggleRuntime() {
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    const current = config.get("enabled", false);
    await config.update("enabled", !current, vscode.ConfigurationTarget.Global);
    vscode.window.showInformationMessage(`BorrowScope Runtime: ${!current ? "Enabled" : "Disabled"}`);
}
async function setRuntimeEnabled(enabled) {
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    await config.update("enabled", enabled, vscode.ConfigurationTarget.Global);
}
async function toggleSetting(key) {
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    const current = config.get(key, true);
    await config.update(key, !current, vscode.ConfigurationTarget.Global);
}
async function setSource() {
    const choice = await vscode.window.showQuickPick([
        { label: "File", description: "Read from .borrowscope/events.json", value: "file" },
        { label: "WebSocket", description: "Live connection to running program", value: "websocket" },
    ], { placeHolder: "Select runtime event source" });
    if (choice) {
        const config = vscode.workspace.getConfiguration("borrowscope.runtime");
        await config.update("source", choice.value, vscode.ConfigurationTarget.Global);
    }
}
function formatAgo(ms) {
    if (ms < 1000)
        return "just now";
    if (ms < 60_000)
        return `${Math.floor(ms / 1000)}s ago`;
    if (ms < 3_600_000)
        return `${Math.floor(ms / 60_000)}m ago`;
    return `${Math.floor(ms / 3_600_000)}h ago`;
}
function formatCount(n) {
    if (n < 1000)
        return `${n}`;
    if (n < 1_000_000)
        return `${(n / 1000).toFixed(1)}K`;
    return `${(n / 1_000_000).toFixed(1)}M`;
}
//# sourceMappingURL=runtime-status.js.map