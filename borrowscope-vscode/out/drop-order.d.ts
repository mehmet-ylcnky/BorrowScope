import { RuntimeEvent } from "./runtime-types";
import { SourceLocation } from "./runtime-mapper";
/** A single drop event with context */
export interface DropEntry {
    var_id: string;
    var_name: string;
    order: number;
    timestamp: number;
    location: SourceLocation | null;
    scope_depth: number;
    function_name: string | null;
    lifetime_ns: number;
}
/** Drop order analysis for a function or scope */
export interface DropOrderAnalysis {
    function_name: string | null;
    drops: DropEntry[];
    is_lifo: boolean;
    violations: DropOrderViolation[];
}
/** A violation of expected LIFO drop order */
export interface DropOrderViolation {
    expected_var: string;
    actual_var: string;
    expected_order: number;
    actual_order: number;
    description: string;
}
/** Build the global drop order from runtime events */
export declare function buildDropOrder(events: RuntimeEvent[]): DropEntry[];
/** Analyze drop order within a function */
export declare function analyzeDropOrder(events: RuntimeEvent[], functionName?: string): DropOrderAnalysis;
/** Get drop order for a specific variable */
export declare function getDropOrderForVar(events: RuntimeEvent[], varId: string): number;
/** Get the longest-lived variables (by lifetime_ns) */
export declare function longestLived(events: RuntimeEvent[], limit?: number): DropEntry[];
/** Get the shortest-lived variables */
export declare function shortestLived(events: RuntimeEvent[], limit?: number): DropEntry[];
