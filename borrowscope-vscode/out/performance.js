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
exports.Debouncer = exports.PerformanceMonitor = void 0;
exports.isFileTooLarge = isFileTooLarge;
exports.registerPerformanceCommand = registerPerformanceCommand;
const vscode = __importStar(require("vscode"));
/** Performance monitoring for BorrowScope operations */
class PerformanceMonitor {
    constructor(outputChannel) {
        this.timings = new Map();
        this.outputChannel = null;
        this.slowThresholdMs = 100;
        this.outputChannel = outputChannel || null;
    }
    /** Time a synchronous operation */
    time(label, fn) {
        const start = performance.now();
        const result = fn();
        this.record(label, performance.now() - start);
        return result;
    }
    /** Time an async operation */
    async timeAsync(label, fn) {
        const start = performance.now();
        const result = await fn();
        this.record(label, performance.now() - start);
        return result;
    }
    /** Record a timing manually */
    record(label, elapsedMs) {
        if (!this.timings.has(label))
            this.timings.set(label, []);
        const times = this.timings.get(label);
        times.push(elapsedMs);
        // Keep only last 100 measurements per label
        if (times.length > 100)
            times.shift();
        if (elapsedMs > this.slowThresholdMs && this.outputChannel) {
            this.outputChannel.appendLine(`[perf] ${label} took ${elapsedMs.toFixed(1)}ms (slow)`);
        }
    }
    /** Get stats for a label */
    getStats(label) {
        const times = this.timings.get(label);
        if (!times || times.length === 0)
            return null;
        return {
            avg: times.reduce((a, b) => a + b, 0) / times.length,
            max: Math.max(...times),
            min: Math.min(...times),
            count: times.length,
        };
    }
    /** Generate a full performance report */
    report() {
        const lines = ["BorrowScope Performance Report:", ""];
        const labels = [...this.timings.keys()].sort();
        for (const label of labels) {
            const stats = this.getStats(label);
            lines.push(`  ${label}:`);
            lines.push(`    avg=${stats.avg.toFixed(1)}ms, max=${stats.max.toFixed(1)}ms, min=${stats.min.toFixed(1)}ms, count=${stats.count}`);
        }
        if (labels.length === 0) {
            lines.push("  No timings recorded yet.");
        }
        return lines.join("\n");
    }
    /** Clear all recorded timings */
    clear() {
        this.timings.clear();
    }
    /** Get all labels */
    getLabels() {
        return [...this.timings.keys()];
    }
}
exports.PerformanceMonitor = PerformanceMonitor;
/** Debouncer that respects the analysis.debounceMs setting */
class Debouncer {
    constructor() {
        this.timer = null;
        this.lastCall = 0;
    }
    /** Debounce a function call */
    debounce(fn, delayMs) {
        const delay = delayMs ?? vscode.workspace.getConfiguration("borrowscope.analysis").get("debounceMs", 300);
        if (this.timer)
            clearTimeout(this.timer);
        this.timer = setTimeout(() => {
            this.timer = null;
            this.lastCall = Date.now();
            fn();
        }, delay);
    }
    /** Cancel pending debounced call */
    cancel() {
        if (this.timer) {
            clearTimeout(this.timer);
            this.timer = null;
        }
    }
    /** Check if a call is pending */
    get isPending() {
        return this.timer !== null;
    }
    /** Time since last executed call */
    get timeSinceLastCall() {
        return this.lastCall > 0 ? Date.now() - this.lastCall : -1;
    }
}
exports.Debouncer = Debouncer;
/** Check if a file is too large for analysis */
function isFileTooLarge(document, maxLines) {
    const limit = maxLines ?? 10000;
    return document.lineCount > limit;
}
/** Register the performance report command */
function registerPerformanceCommand(context, monitor) {
    context.subscriptions.push(vscode.commands.registerCommand("borrowscope.showPerformanceReport", () => {
        const report = monitor.report();
        vscode.window.showInformationMessage("BorrowScope Performance", { modal: true, detail: report });
    }));
}
//# sourceMappingURL=performance.js.map