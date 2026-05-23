import { RuntimeEvent } from "./runtime-types";
import { StaticVariable } from "./runtime-mapper";
/** Runtime-derived data for a variable */
export interface RuntimeInfo {
    actual_lifetime_ns: number;
    actual_borrow_count: number;
    actual_mut_borrow_count: number;
    was_actually_moved: boolean;
    move_destination: string | null;
    drop_order: number;
    drop_timestamp: number;
    ref_count_peak: number;
    ref_count_final: number;
    weak_count_peak: number;
    clone_count: number;
    thread_id: number | null;
    await_crossings: AwaitCrossing[];
    unsafe_accesses: number;
    event_count: number;
}
/** A borrow held across an await point */
export interface AwaitCrossing {
    await_line: number;
    duration_ns: number;
    future_name: string;
}
/** Agreement between static and runtime analysis */
export type Agreement = "match" | "diverge" | "runtime_only" | "static_only";
/** Reason for divergence */
export interface Divergence {
    kind: "conditional_move" | "rc_leak" | "borrow_count_mismatch" | "type_mismatch" | "unexpected_drop" | "missing_drop" | "unsafe_hidden" | "async_borrow_held";
    description: string;
}
/** Combined static + runtime view of a variable */
export interface MergedVariable {
    name: string;
    var_id: string | null;
    line: number;
    file: string | null;
    static_info: {
        type_display: string;
        ownership_category: string;
        is_copy: boolean;
    } | null;
    runtime_info: RuntimeInfo | null;
    agreement: Agreement;
    divergences: Divergence[];
}
/** Merge static analysis graph with runtime events */
export declare function mergeViews(staticVars: StaticVariable[], events: RuntimeEvent[], file?: string): MergedVariable[];
/** Get merge summary statistics */
export declare function mergeSummary(merged: MergedVariable[]): {
    total: number;
    match: number;
    diverge: number;
    runtime_only: number;
    static_only: number;
};
