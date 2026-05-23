import * as vscode from "vscode";
import { RuntimeWatcher } from "./runtime-watcher";
import { RuntimeSocket } from "./runtime-socket";
/** Status bar item showing runtime overlay state */
export declare class RuntimeStatusBar implements vscode.Disposable {
    private statusBarItem;
    private watcher;
    private socket;
    private disposables;
    constructor();
    /** Attach to a file watcher */
    attachWatcher(watcher: RuntimeWatcher): void;
    /** Attach to a WebSocket connection */
    attachSocket(socket: RuntimeSocket): void;
    /** Update the status bar display */
    update(): void;
    private buildTooltip;
    show(): void;
    hide(): void;
    dispose(): void;
}
/** Register runtime toggle commands */
export declare function registerRuntimeCommands(context: vscode.ExtensionContext): void;
