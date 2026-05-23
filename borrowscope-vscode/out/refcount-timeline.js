"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildRefCountTimeline = buildRefCountTimeline;
exports.buildAllRefCountTimelines = buildAllRefCountTimelines;
exports.refCountAnnotation = refCountAnnotation;
exports.detectRefCountIssues = detectRefCountIssues;
const runtime_types_1 = require("./runtime-types");
/** Build a ref count timeline for a specific Rc/Arc variable */
function buildRefCountTimeline(events, varId) {
    let varName = "";
    let typeName = "";
    let isArc = false;
    let strong = 0;
    let weak = 0;
    let peakStrong = 0;
    let peakWeak = 0;
    let totalClones = 0;
    let totalDrops = 0;
    const points = [];
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        // Creation
        if ((type === "RcNew" || type === "ArcNew") && data.var_id === varId) {
            varName = data.var_name;
            typeName = data.type_name;
            isArc = type === "ArcNew";
            strong = data.strong_count ?? 1;
            weak = data.weak_count ?? 0;
            peakStrong = Math.max(peakStrong, strong);
            points.push({
                timestamp: data.timestamp, line: parseLine(data.location),
                strong_count: strong, weak_count: weak,
                event: "new", variable: data.var_name,
            });
        }
        // Clone (strong ref increase)
        if ((type === "RcClone" || type === "ArcClone") && data.source_id === varId) {
            strong = data.strong_count ?? strong + 1;
            weak = data.weak_count ?? weak;
            peakStrong = Math.max(peakStrong, strong);
            totalClones++;
            points.push({
                timestamp: data.timestamp, line: parseLine(data.location),
                strong_count: strong, weak_count: weak,
                event: "clone", variable: data.var_name, source_variable: varName,
            });
        }
        // Drop of a clone (strong ref decrease) — track by source_id lineage
        // We detect drops of clones by looking at Drop events for var_ids that were cloned from this source
        if (type === "Drop" && data.var_id === varId) {
            strong = Math.max(0, strong - 1);
            totalDrops++;
            points.push({
                timestamp: data.timestamp, line: parseLine(data.location),
                strong_count: strong, weak_count: weak,
                event: "drop", variable: varName,
            });
        }
        // Weak reference created from this Rc/Arc
        if ((type === "WeakNew" || type === "WeakClone") && data.source_id === varId) {
            weak = data.weak_count ?? weak + 1;
            peakWeak = Math.max(peakWeak, weak);
            const evt = type === "WeakNew" ? "weak_new" : "weak_clone";
            points.push({
                timestamp: data.timestamp, line: parseLine(data.location),
                strong_count: strong, weak_count: weak,
                event: evt, variable: data.var_name, source_variable: varName,
            });
        }
        // Weak upgrade attempt
        if (type === "WeakUpgrade" && data.weak_id === varId) {
            const evt = data.success ? "upgrade_ok" : "upgrade_fail";
            points.push({
                timestamp: data.timestamp, line: parseLine(data.location),
                strong_count: strong, weak_count: weak,
                event: evt, variable: varName,
            });
        }
    }
    if (points.length === 0)
        return null;
    return {
        var_id: varId,
        var_name: varName,
        type_name: typeName,
        is_arc: isArc,
        points,
        peak_strong: peakStrong,
        peak_weak: peakWeak,
        final_strong: strong,
        final_weak: weak,
        leaked: strong > 0 && totalDrops > 0,
        total_clones: totalClones,
        total_drops: totalDrops,
    };
}
/** Build timelines for all Rc/Arc variables in the event stream */
function buildAllRefCountTimelines(events) {
    const rcVarIds = new Set();
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        if (type === "RcNew" || type === "ArcNew") {
            rcVarIds.add((0, runtime_types_1.eventData)(event).var_id);
        }
    }
    const timelines = [];
    for (const varId of rcVarIds) {
        const timeline = buildRefCountTimeline(events, varId);
        if (timeline)
            timelines.push(timeline);
    }
    return timelines;
}
/** Get inline decoration text for a ref count point */
function refCountAnnotation(point) {
    const arrow = point.event === "clone" ? "↑" : point.event === "drop" ? "↓" : "";
    return `🔗 refs: ${point.strong_count}${arrow}${point.weak_count > 0 ? ` (weak: ${point.weak_count})` : ""}`;
}
/** Detect potential Rc/Arc issues from timelines */
function detectRefCountIssues(timelines) {
    const issues = [];
    for (const t of timelines) {
        if (t.leaked) {
            issues.push({
                var_id: t.var_id,
                var_name: t.var_name,
                kind: "leak",
                description: `${t.is_arc ? "Arc" : "Rc"}<${t.type_name}> leaked: final strong_count=${t.final_strong}`,
                peak_count: t.peak_strong,
            });
        }
        if (t.peak_strong > 10) {
            issues.push({
                var_id: t.var_id,
                var_name: t.var_name,
                kind: "high_refcount",
                description: `${t.is_arc ? "Arc" : "Rc"} peak ref count = ${t.peak_strong} (consider restructuring)`,
                peak_count: t.peak_strong,
            });
        }
        if (t.points.some(p => p.event === "upgrade_fail")) {
            issues.push({
                var_id: t.var_id,
                var_name: t.var_name,
                kind: "dangling_weak",
                description: `Weak::upgrade failed — strong references dropped before weak was used`,
                peak_count: t.peak_weak,
            });
        }
    }
    return issues;
}
function parseLine(location) {
    if (!location)
        return 0;
    const match = location.match(/:(\d+):/);
    return match ? parseInt(match[1], 10) : 0;
}
//# sourceMappingURL=refcount-timeline.js.map