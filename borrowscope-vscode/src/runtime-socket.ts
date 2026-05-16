import * as vscode from "vscode";
import { RuntimeEvent, eventType, eventData } from "./runtime-types";
import { validateEvent } from "./runtime-parser";

export type ConnectionState = "disconnected" | "connecting" | "connected" | "error";

/** WebSocket-based live runtime event receiver */
export class RuntimeSocket implements vscode.Disposable {
  private ws: WebSocket | null = null;
  private events: RuntimeEvent[] = [];
  private _state: ConnectionState = "disconnected";
  private _reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private _maxEvents: number;
  private _autoReconnect: boolean;

  private _onEvent = new vscode.EventEmitter<RuntimeEvent>();
  public readonly onEvent = this._onEvent.event;

  private _onStateChanged = new vscode.EventEmitter<ConnectionState>();
  public readonly onStateChanged = this._onStateChanged.event;

  private _onBatchReceived = new vscode.EventEmitter<RuntimeEvent[]>();
  public readonly onBatchReceived = this._onBatchReceived.event;

  private disposables: vscode.Disposable[] = [];

  constructor(options?: { maxEvents?: number; autoReconnect?: boolean }) {
    this._maxEvents = options?.maxEvents ?? 100_000;
    this._autoReconnect = options?.autoReconnect ?? true;
  }

  get state(): ConnectionState { return this._state; }
  get eventCount(): number { return this.events.length; }
  get isConnected(): boolean { return this._state === "connected"; }

  /** Connect to the runtime WebSocket server */
  connect(port?: number): void {
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    const wsPort = port ?? config.get<number>("websocketPort", 9876);

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

      this.ws.onmessage = (msg: MessageEvent) => {
        this._handleMessage(msg.data as string);
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
    } catch {
      this._setState("error");
    }
  }

  /** Disconnect from the WebSocket server */
  disconnect(): void {
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
  getEvents(): RuntimeEvent[] { return this.events; }

  /** Clear accumulated events */
  clearEvents(): void {
    this.events = [];
  }

  private _handleMessage(data: string): void {
    try {
      const parsed = JSON.parse(data);

      // Handle single event or batch
      if (Array.isArray(parsed)) {
        const valid: RuntimeEvent[] = [];
        for (const item of parsed) {
          if (!validateEvent(item)) {
            this._addEvent(item as RuntimeEvent);
            valid.push(item as RuntimeEvent);
          }
        }
        if (valid.length > 0) {
          this._onBatchReceived.fire(valid);
        }
      } else {
        if (!validateEvent(parsed)) {
          this._addEvent(parsed as RuntimeEvent);
          this._onEvent.fire(parsed as RuntimeEvent);
        }
      }
    } catch {
      // Invalid JSON — skip
    }
  }

  private _addEvent(event: RuntimeEvent): void {
    if (this.events.length >= this._maxEvents) {
      // Evict oldest 10%
      this.events = this.events.slice(Math.floor(this._maxEvents * 0.1));
    }
    this.events.push(event);
  }

  private _scheduleReconnect(port: number): void {
    if (this._reconnectTimer) return;
    this._reconnectTimer = setTimeout(() => {
      this._reconnectTimer = null;
      if (this._state === "disconnected" && this._autoReconnect) {
        this.connect(port);
      }
    }, 3000);
  }

  private _setState(state: ConnectionState): void {
    if (this._state !== state) {
      this._state = state;
      this._onStateChanged.fire(state);
    }
  }

  dispose(): void {
    this.disconnect();
    this._onEvent.dispose();
    this._onStateChanged.dispose();
    this._onBatchReceived.dispose();
    this.disposables.forEach(d => d.dispose());
  }
}
