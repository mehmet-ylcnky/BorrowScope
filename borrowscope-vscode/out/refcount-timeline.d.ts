import { RuntimeEvent } from "./runtime-types";
/** A point in the reference count timeline */
export interface RefCountPoint {
    timestamp: number;
    line: number;
    strong_count: number;
    weak_count: number;
    event: "new" | "clone" | "drop" | "weak_new" | "weak_clone" | "weak_drop" | "upgrade_ok" | "upgrade_fail";
    variable: string;
    source_variable?: string;
}
/** Complete timeline for a single Rc/Arc variable */
export interface RefCountTimeline {
    var_id: string;
    var_name: string;
    type_name: string;
    is_arc: boolean;
    points: RefCountPoint[];
    peak_strong: number;
    peak_weak: number;
    final_strong: number;
    final_weak: number;
    leaked: boolean;
    total_clones: number;
    total_drops: number;
}
/** Build a ref count timeline for a specific Rc/Arc variable */
export declare function buildRefCountTimeline(events: RuntimeEvent[], varId: string): RefCountTimeline | null;
/** Build timelines for all Rc/Arc variables in the event stream */
export declare function buildAllRefCountTimelines(events: RuntimeEvent[]): RefCountTimeline[];
/** Get inline decoration text for a ref count point */
export declare function refCountAnnotation(point: RefCountPoint): string;
/** Detect potential Rc/Arc issues from timelines */
export declare function detectRefCountIssues(timelines: RefCountTimeline[]): RefCountIssue[];
export interface RefCountIssue {
    var_id: string;
    var_name: string;
    kind: "leak" | "high_refcount" | "dangling_weak";
    description: string;
    peak_count: number;
}
