import * as vscode from "vscode";

/** Performance monitoring for BorrowScope operations */
export class PerformanceMonitor {
  private timings: Map<string, number[]> = new Map();
  private outputChannel: vscode.OutputChannel | null = null;
  private slowThresholdMs: number = 100;

  constructor(outputChannel?: vscode.OutputChannel) {
    this.outputChannel = outputChannel || null;
  }

  /** Time a synchronous operation */
  time<T>(label: string, fn: () => T): T {
    const start = performance.now();
    const result = fn();
    this.record(label, performance.now() - start);
    return result;
  }

  /** Time an async operation */
  async timeAsync<T>(label: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    const result = await fn();
    this.record(label, performance.now() - start);
    return result;
  }

  /** Record a timing manually */
  record(label: string, elapsedMs: number): void {
    if (!this.timings.has(label)) this.timings.set(label, []);
    const times = this.timings.get(label)!;
    times.push(elapsedMs);

    // Keep only last 100 measurements per label
    if (times.length > 100) times.shift();

    if (elapsedMs > this.slowThresholdMs && this.outputChannel) {
      this.outputChannel.appendLine(`[perf] ${label} took ${elapsedMs.toFixed(1)}ms (slow)`);
    }
  }

  /** Get stats for a label */
  getStats(label: string): { avg: number; max: number; min: number; count: number } | null {
    const times = this.timings.get(label);
    if (!times || times.length === 0) return null;
    return {
      avg: times.reduce((a, b) => a + b, 0) / times.length,
      max: Math.max(...times),
      min: Math.min(...times),
      count: times.length,
    };
  }

  /** Generate a full performance report */
  report(): string {
    const lines: string[] = ["BorrowScope Performance Report:", ""];
    const labels = [...this.timings.keys()].sort();

    for (const label of labels) {
      const stats = this.getStats(label)!;
      lines.push(`  ${label}:`);
      lines.push(`    avg=${stats.avg.toFixed(1)}ms, max=${stats.max.toFixed(1)}ms, min=${stats.min.toFixed(1)}ms, count=${stats.count}`);
    }

    if (labels.length === 0) {
      lines.push("  No timings recorded yet.");
    }

    return lines.join("\n");
  }

  /** Clear all recorded timings */
  clear(): void {
    this.timings.clear();
  }

  /** Get all labels */
  getLabels(): string[] {
    return [...this.timings.keys()];
  }
}

/** Debouncer that respects the analysis.debounceMs setting */
export class Debouncer {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastCall: number = 0;

  /** Debounce a function call */
  debounce(fn: () => void, delayMs?: number): void {
    const delay = delayMs ?? vscode.workspace.getConfiguration("borrowscope.analysis").get("debounceMs", 300);
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = null;
      this.lastCall = Date.now();
      fn();
    }, delay);
  }

  /** Cancel pending debounced call */
  cancel(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  /** Check if a call is pending */
  get isPending(): boolean {
    return this.timer !== null;
  }

  /** Time since last executed call */
  get timeSinceLastCall(): number {
    return this.lastCall > 0 ? Date.now() - this.lastCall : -1;
  }
}

/** Check if a file is too large for analysis */
export function isFileTooLarge(document: vscode.TextDocument, maxLines?: number): boolean {
  const limit = maxLines ?? 10000;
  return document.lineCount > limit;
}

/** Register the performance report command */
export function registerPerformanceCommand(
  context: vscode.ExtensionContext,
  monitor: PerformanceMonitor
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("borrowscope.showPerformanceReport", () => {
      const report = monitor.report();
      vscode.window.showInformationMessage("BorrowScope Performance", { modal: true, detail: report });
    })
  );
}
