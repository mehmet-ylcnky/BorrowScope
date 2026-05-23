import { RuntimeEvent } from "./runtime-types";
import { MergedVariable } from "./merge-views";
/** All possible divergence kinds */
export type DivergenceKind = "conditional_move" | "unexpected_move" | "rc_leak" | "rc_cycle" | "borrow_count_mismatch" | "type_mismatch" | "unexpected_drop" | "missing_drop" | "unsafe_hidden" | "async_borrow_held" | "refcell_panic" | "double_move" | "use_after_move" | "weak_upgrade_fail" | "lock_contention" | "channel_recv_fail";
/** Detailed divergence with severity */
export interface DetailedDivergence {
    kind: DivergenceKind;
    severity: "info" | "warning" | "error";
    description: string;
    suggestion?: string;
    runtime_evidence: string;
}
/** Run all divergence detectors on a merged variable */
export declare function detectAllDivergences(merged: MergedVariable, allEvents: RuntimeEvent[]): DetailedDivergence[];
/** Analyze divergences across all merged variables for summary */
export declare function analyzeDivergences(merged: MergedVariable[], allEvents: RuntimeEvent[]): {
    total: number;
    by_kind: Record<string, number>;
    by_severity: {
        info: number;
        warning: number;
        error: number;
    };
    variables_with_divergences: string[];
};
