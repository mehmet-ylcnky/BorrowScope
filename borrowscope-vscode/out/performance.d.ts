import * as vscode from "vscode";
/** Performance monitoring for BorrowScope operations */
export declare class PerformanceMonitor {
    private timings;
    private outputChannel;
    private slowThresholdMs;
    constructor(outputChannel?: vscode.OutputChannel);
    /** Time a synchronous operation */
    time<T>(label: string, fn: () => T): T;
    /** Time an async operation */
    timeAsync<T>(label: string, fn: () => Promise<T>): Promise<T>;
    /** Record a timing manually */
    record(label: string, elapsedMs: number): void;
    /** Get stats for a label */
    getStats(label: string): {
        avg: number;
        max: number;
        min: number;
        count: number;
    } | null;
    /** Generate a full performance report */
    report(): string;
    /** Clear all recorded timings */
    clear(): void;
    /** Get all labels */
    getLabels(): string[];
}
/** Debouncer that respects the analysis.debounceMs setting */
export declare class Debouncer {
    private timer;
    private lastCall;
    /** Debounce a function call */
    debounce(fn: () => void, delayMs?: number): void;
    /** Cancel pending debounced call */
    cancel(): void;
    /** Check if a call is pending */
    get isPending(): boolean;
    /** Time since last executed call */
    get timeSinceLastCall(): number;
}
/** Check if a file is too large for analysis */
export declare function isFileTooLarge(document: vscode.TextDocument, maxLines?: number): boolean;
/** Register the performance report command */
export declare function registerPerformanceCommand(context: vscode.ExtensionContext, monitor: PerformanceMonitor): void;
