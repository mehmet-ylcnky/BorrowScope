import * as vscode from "vscode";
import { RuntimeWatcher } from "./runtime-watcher";
import { PerformanceMonitor } from "./performance";
export declare function activate(context: vscode.ExtensionContext): Promise<void>;
export declare function deactivate(): Promise<void>;
export declare function getRuntimeWatcher(): RuntimeWatcher | undefined;
export declare function getPerfMonitor(): PerformanceMonitor;
