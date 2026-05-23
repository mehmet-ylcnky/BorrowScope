"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.detectAllDivergences = detectAllDivergences;
exports.analyzeDivergences = analyzeDivergences;
const runtime_types_1 = require("./runtime-types");
/** Run all divergence detectors on a merged variable */
function detectAllDivergences(merged, allEvents) {
    const divergences = [];
    const s = merged.static_info;
    const r = merged.runtime_info;
    if (!r)
        return divergences;
    // 1. Rc/Arc leak
    if (s && (s.ownership_category === "Rc" || s.ownership_category === "Arc") &&
        r.drop_timestamp < 0 && r.ref_count_final > 0) {
        divergences.push({
            kind: "rc_leak",
            severity: "error",
            description: `${s.ownership_category} never dropped (final ref count: ${r.ref_count_final})`,
            suggestion: "Check for reference cycles. Consider using Weak references.",
            runtime_evidence: `Created with ref_count=1, peaked at ${r.ref_count_peak}, never reached 0`,
        });
    }
    // 2. Rc cycle detection (ref count went up, never came back down)
    if (s && (s.ownership_category === "Rc" || s.ownership_category === "Arc") &&
        r.ref_count_peak > 1 && r.ref_count_final > 0 && r.drop_timestamp < 0 && r.clone_count >= 2) {
        divergences.push({
            kind: "rc_cycle",
            severity: "error",
            description: `Possible ${s.ownership_category} cycle: ${r.clone_count} clones, peak refs=${r.ref_count_peak}, never freed`,
            suggestion: "Break the cycle with Weak<T> or restructure ownership.",
            runtime_evidence: `${r.clone_count} clone operations, final count=${r.ref_count_final}`,
        });
    }
    // 3. Missing drop (non-Copy, non-moved, not Rc/Arc)
    if (s && !s.is_copy && !r.was_actually_moved && r.drop_timestamp < 0 &&
        s.ownership_category !== "Rc" && s.ownership_category !== "Arc" &&
        s.ownership_category !== "Weak") {
        divergences.push({
            kind: "missing_drop",
            severity: "warning",
            description: `"${merged.name}" was never dropped`,
            suggestion: "This may indicate a leak, std::mem::forget, or program exit before scope end.",
            runtime_evidence: `Created at timestamp ${r.event_count > 0 ? "known" : "unknown"}, no Drop event recorded`,
        });
    }
    // 4. Async borrow held across await
    if (r.await_crossings.length > 0) {
        const totalDuration = r.await_crossings.reduce((sum, c) => sum + c.duration_ns, 0);
        divergences.push({
            kind: "async_borrow_held",
            severity: "warning",
            description: `Borrow held across ${r.await_crossings.length} await point(s), total ${formatNs(totalDuration)}`,
            suggestion: "Consider cloning before the await or restructuring to drop the borrow first.",
            runtime_evidence: r.await_crossings.map(c => `await at line ${c.await_line}: ${formatNs(c.duration_ns)} (${c.future_name})`).join("; "),
        });
    }
    // 5. Unsafe hidden
    if (r.unsafe_accesses > 0 && s && s.ownership_category !== "RawPtr") {
        divergences.push({
            kind: "unsafe_hidden",
            severity: "info",
            description: `${r.unsafe_accesses} unsafe access(es) — static analysis may be incomplete`,
            suggestion: "Review unsafe blocks for soundness. Static analysis cannot verify unsafe code.",
            runtime_evidence: `${r.unsafe_accesses} raw pointer dereferences or unsafe block entries`,
        });
    }
    // 6. Conditional move (static says Owned but never moved at runtime)
    if (s && s.ownership_category === "Owned" && !s.is_copy && !r.was_actually_moved &&
        r.drop_timestamp >= 0 && r.actual_borrow_count === 0) {
        // Only flag if the variable was never borrowed either — suggests it was meant to be consumed
        divergences.push({
            kind: "conditional_move",
            severity: "info",
            description: `"${merged.name}" was not moved (conditional branch not taken?)`,
            suggestion: "The move may be in a branch that wasn't executed in this run.",
            runtime_evidence: `Owned, not Copy, 0 borrows, dropped normally at order #${r.drop_order + 1}`,
        });
    }
    // 7. Weak upgrade failure
    const weakUpgradeFails = findEventsForVar(merged.var_id, allEvents, "WeakUpgrade")
        .filter(e => (0, runtime_types_1.eventData)(e).success === false);
    if (weakUpgradeFails.length > 0) {
        divergences.push({
            kind: "weak_upgrade_fail",
            severity: "warning",
            description: `Weak::upgrade failed ${weakUpgradeFails.length} time(s) — referent was already dropped`,
            suggestion: "Handle the None case from Weak::upgrade, or ensure the strong reference outlives the weak.",
            runtime_evidence: `${weakUpgradeFails.length} failed upgrade attempts`,
        });
    }
    // 8. Channel receive failure
    const recvFails = findEventsForVar(merged.var_id, allEvents, "ChannelRecv")
        .filter(e => (0, runtime_types_1.eventData)(e).success === false);
    if (recvFails.length > 0) {
        divergences.push({
            kind: "channel_recv_fail",
            severity: "warning",
            description: `Channel receive failed ${recvFails.length} time(s) — sender was dropped`,
            suggestion: "Ensure sender outlives receiver, or handle the RecvError.",
            runtime_evidence: `${recvFails.length} failed recv operations`,
        });
    }
    // 9. Use after move (events with this var_id after a Move from it)
    if (r.was_actually_moved) {
        const moveTimestamp = findMoveTimestamp(merged.var_id, allEvents);
        if (moveTimestamp > 0) {
            const eventsAfterMove = merged.events?.filter((e) => (0, runtime_types_1.eventData)(e).timestamp > moveTimestamp && (0, runtime_types_1.eventType)(e) !== "Drop") || [];
            if (eventsAfterMove.length > 0) {
                divergences.push({
                    kind: "use_after_move",
                    severity: "error",
                    description: `${eventsAfterMove.length} event(s) after move — possible use-after-move`,
                    suggestion: "This should not happen in safe Rust. Check for unsafe code or instrumentation errors.",
                    runtime_evidence: `Move at timestamp ${moveTimestamp}, ${eventsAfterMove.length} subsequent events`,
                });
            }
        }
    }
    return divergences;
}
/** Analyze divergences across all merged variables for summary */
function analyzeDivergences(merged, allEvents) {
    const byKind = {};
    const bySeverity = { info: 0, warning: 0, error: 0 };
    const varsWithDiv = [];
    let total = 0;
    for (const m of merged) {
        const divs = detectAllDivergences(m, allEvents);
        if (divs.length > 0) {
            varsWithDiv.push(m.name);
            for (const d of divs) {
                total++;
                byKind[d.kind] = (byKind[d.kind] || 0) + 1;
                bySeverity[d.severity]++;
            }
        }
    }
    return { total, by_kind: byKind, by_severity: bySeverity, variables_with_divergences: varsWithDiv };
}
/** Find events matching a var_id and type */
function findEventsForVar(varId, events, type) {
    if (!varId)
        return [];
    return events.filter(e => {
        if ((0, runtime_types_1.eventType)(e) !== type)
            return false;
        const data = (0, runtime_types_1.eventData)(e);
        return data.var_id === varId || data.weak_id === varId || data.receiver_id === varId;
    });
}
/** Find the timestamp of the Move event for a var_id */
function findMoveTimestamp(varId, events) {
    if (!varId)
        return 0;
    for (const e of events) {
        if ((0, runtime_types_1.eventType)(e) === "Move" && (0, runtime_types_1.eventData)(e).from_id === varId) {
            return (0, runtime_types_1.eventData)(e).timestamp;
        }
    }
    return 0;
}
function formatNs(ns) {
    if (ns < 1000)
        return `${ns}ns`;
    if (ns < 1_000_000)
        return `${(ns / 1000).toFixed(1)}μs`;
    if (ns < 1_000_000_000)
        return `${(ns / 1_000_000).toFixed(1)}ms`;
    return `${(ns / 1_000_000_000).toFixed(2)}s`;
}
//# sourceMappingURL=divergence-detector.js.map