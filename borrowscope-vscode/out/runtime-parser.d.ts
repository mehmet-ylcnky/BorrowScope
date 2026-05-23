import { RuntimeEvent } from "./runtime-types";
/** Ownership-relevant event types for the merge/overlay */
export declare const OWNERSHIP_EVENT_TYPES: Set<string>;
export interface ParseResult {
    events: RuntimeEvent[];
    errors: string[];
    total: number;
    valid: number;
}
/** Parse raw JSON string into validated RuntimeEvent array */
export declare function parseEvents(json: string): ParseResult;
/** Validate a single event object. Returns error string or null if valid. */
export declare function validateEvent(event: any, index?: number): string | null;
/** Filter events by file path (matches location field) */
export declare function filterByFile(events: RuntimeEvent[], filePath: string): RuntimeEvent[];
/** Filter events by type name(s) */
export declare function filterByType(events: RuntimeEvent[], types: string | string[]): RuntimeEvent[];
/** Filter to only ownership-relevant events */
export declare function filterOwnershipEvents(events: RuntimeEvent[]): RuntimeEvent[];
/** Group events by variable (var_id or var_name) */
export declare function groupByVariable(events: RuntimeEvent[]): Map<string, RuntimeEvent[]>;
/** Group events by function (using FnEnter/FnExit boundaries) */
export declare function groupByFunction(events: RuntimeEvent[]): Map<string, RuntimeEvent[]>;
/** Sort events by timestamp */
export declare function sortByTimestamp(events: RuntimeEvent[]): RuntimeEvent[];
/** Get event count summary by type */
export declare function summarize(events: RuntimeEvent[]): Record<string, number>;
