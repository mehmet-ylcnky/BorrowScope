import * as vscode from "vscode";
import { RuntimeEvent } from "./runtime-types";
export type ConnectionState = "disconnected" | "connecting" | "connected" | "error";
/** WebSocket-based live runtime event receiver */
export declare class RuntimeSocket implements vscode.Disposable {
    private ws;
    private events;
    private _state;
    private _reconnectTimer;
    private _maxEvents;
    private _autoReconnect;
    private _onEvent;
    readonly onEvent: vscode.Event<RuntimeEvent>;
    private _onStateChanged;
    readonly onStateChanged: vscode.Event<ConnectionState>;
    private _onBatchReceived;
    readonly onBatchReceived: vscode.Event<RuntimeEvent[]>;
    private disposables;
    constructor(options?: {
        maxEvents?: number;
        autoReconnect?: boolean;
    });
    get state(): ConnectionState;
    get eventCount(): number;
    get isConnected(): boolean;
    /** Connect to the runtime WebSocket server */
    connect(port?: number): void;
    /** Disconnect from the WebSocket server */
    disconnect(): void;
    /** Get all received events */
    getEvents(): RuntimeEvent[];
    /** Clear accumulated events */
    clearEvents(): void;
    private _handleMessage;
    private _addEvent;
    private _scheduleReconnect;
    private _setState;
    dispose(): void;
}
