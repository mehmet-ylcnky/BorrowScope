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
exports.RuntimeWatcher = void 0;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const DEFAULT_EVENT_FILE = ".borrowscope/events.json";
class RuntimeWatcher {
    constructor() {
        this.events = [];
        this._onEventsUpdated = new vscode.EventEmitter();
        this.onEventsUpdated = this._onEventsUpdated.event;
        this._stale = false;
        this._lastLoadTime = 0;
        this._eventFilePath = "";
        this.disposables = [];
    }
    start(workspaceRoot) {
        const config = vscode.workspace.getConfiguration("borrowscope.runtime");
        const relPath = config.get("filePath", DEFAULT_EVENT_FILE);
        this._eventFilePath = path.join(workspaceRoot, relPath);
        const pattern = new vscode.RelativePattern(workspaceRoot, relPath);
        this.watcher = vscode.workspace.createFileSystemWatcher(pattern);
        this.watcher.onDidChange((uri) => this.loadEvents(uri.fsPath), null, this.disposables);
        this.watcher.onDidCreate((uri) => this.loadEvents(uri.fsPath), null, this.disposables);
        this.watcher.onDidDelete(() => this.clearEvents(), null, this.disposables);
        // Mark stale when source files change
        const rustWatcher = vscode.workspace.createFileSystemWatcher("**/*.rs");
        rustWatcher.onDidChange(() => { this._stale = true; }, null, this.disposables);
        this.disposables.push(rustWatcher);
        // Load existing file
        if (fs.existsSync(this._eventFilePath)) {
            this.loadEvents(this._eventFilePath);
        }
    }
    loadEvents(filePath) {
        try {
            const content = fs.readFileSync(filePath, "utf8");
            if (!content.trim()) {
                this.clearEvents();
                return;
            }
            const parsed = JSON.parse(content);
            this.events = Array.isArray(parsed) ? parsed : [];
            this._stale = false;
            this._lastLoadTime = Date.now();
            this._onEventsUpdated.fire(this.events);
        }
        catch {
            // Invalid JSON — ignore until next write
        }
    }
    clearEvents() {
        this.events = [];
        this._stale = false;
        this._lastLoadTime = 0;
        this._onEventsUpdated.fire(this.events);
    }
    getEvents() { return this.events; }
    get eventCount() { return this.events.length; }
    get isStale() { return this._stale; }
    get lastLoadTime() { return this._lastLoadTime; }
    get filePath() { return this._eventFilePath; }
    dispose() {
        this.watcher?.dispose();
        this._onEventsUpdated.dispose();
        this.disposables.forEach((d) => d.dispose());
    }
}
exports.RuntimeWatcher = RuntimeWatcher;
//# sourceMappingURL=runtime-watcher.js.map