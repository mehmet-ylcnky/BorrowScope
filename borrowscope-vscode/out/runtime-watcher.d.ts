import * as vscode from "vscode";
import { RuntimeEvent } from "./runtime-types";
export declare class RuntimeWatcher implements vscode.Disposable {
    private watcher;
    private events;
    private _onEventsUpdated;
    readonly onEventsUpdated: vscode.Event<RuntimeEvent[]>;
    private _stale;
    private _lastLoadTime;
    private _eventFilePath;
    private disposables;
    start(workspaceRoot: string): void;
    private loadEvents;
    private clearEvents;
    getEvents(): RuntimeEvent[];
    get eventCount(): number;
    get isStale(): boolean;
    get lastLoadTime(): number;
    get filePath(): string;
    dispose(): void;
}
