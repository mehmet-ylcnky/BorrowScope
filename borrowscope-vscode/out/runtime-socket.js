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
exports.RuntimeSocket = void 0;
const vscode = __importStar(require("vscode"));
const runtime_parser_1 = require("./runtime-parser");
/** WebSocket-based live runtime event receiver */
class RuntimeSocket {
    constructor(options) {
        this.ws = null;
        this.events = [];
        this._state = "disconnected";
        this._reconnectTimer = null;
        this._onEvent = new vscode.EventEmitter();
        this.onEvent = this._onEvent.event;
        this._onStateChanged = new vscode.EventEmitter();
        this.onStateChanged = this._onStateChanged.event;
        this._onBatchReceived = new vscode.EventEmitter();
        this.onBatchReceived = this._onBatchReceived.event;
        this.disposables = [];
        this._maxEvents = options?.maxEvents ?? 100_000;
        this._autoReconnect = options?.autoReconnect ?? true;
    }
    get state() { return this._state; }
    get eventCount() { return this.events.length; }
    get isConnected() { return this._state === "connected"; }
    /** Connect to the runtime WebSocket server */
    connect(port) {
        const config = vscode.workspace.getConfiguration("borrowscope.runtime");
        const wsPort = port ?? config.get("websocketPort", 9876);
        if (this._state === "connected" || this._state === "connecting") {
            return;
        }
        this._setState("connecting");
        try {
            this.ws = new WebSocket(`ws://localhost:${wsPort}`);
            this.ws.onopen = () => {
                this._setState("connected");
                this.events = [];
            };
            this.ws.onmessage = (msg) => {
                this._handleMessage(msg.data);
            };
            this.ws.onclose = () => {
                this._setState("disconnected");
                this.ws = null;
                if (this._autoReconnect) {
                    this._scheduleReconnect(wsPort);
                }
            };
            this.ws.onerror = () => {
                this._setState("error");
            };
        }
        catch {
            this._setState("error");
        }
    }
    /** Disconnect from the WebSocket server */
    disconnect() {
        this._autoReconnect = false;
        if (this._reconnectTimer) {
            clearTimeout(this._reconnectTimer);
            this._reconnectTimer = null;
        }
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this._setState("disconnected");
    }
    /** Get all received events */
    getEvents() { return this.events; }
    /** Clear accumulated events */
    clearEvents() {
        this.events = [];
    }
    _handleMessage(data) {
        try {
            const parsed = JSON.parse(data);
            // Handle single event or batch
            if (Array.isArray(parsed)) {
                const valid = [];
                for (const item of parsed) {
                    if (!(0, runtime_parser_1.validateEvent)(item)) {
                        this._addEvent(item);
                        valid.push(item);
                    }
                }
                if (valid.length > 0) {
                    this._onBatchReceived.fire(valid);
                }
            }
            else {
                if (!(0, runtime_parser_1.validateEvent)(parsed)) {
                    this._addEvent(parsed);
                    this._onEvent.fire(parsed);
                }
            }
        }
        catch {
            // Invalid JSON — skip
        }
    }
    _addEvent(event) {
        if (this.events.length >= this._maxEvents) {
            // Evict oldest 10%
            this.events = this.events.slice(Math.floor(this._maxEvents * 0.1));
        }
        this.events.push(event);
    }
    _scheduleReconnect(port) {
        if (this._reconnectTimer)
            return;
        this._reconnectTimer = setTimeout(() => {
            this._reconnectTimer = null;
            if (this._state === "disconnected" && this._autoReconnect) {
                this.connect(port);
            }
        }, 3000);
    }
    _setState(state) {
        if (this._state !== state) {
            this._state = state;
            this._onStateChanged.fire(state);
        }
    }
    dispose() {
        this.disconnect();
        this._onEvent.dispose();
        this._onStateChanged.dispose();
        this._onBatchReceived.dispose();
        this.disposables.forEach(d => d.dispose());
    }
}
exports.RuntimeSocket = RuntimeSocket;
//# sourceMappingURL=runtime-socket.js.map