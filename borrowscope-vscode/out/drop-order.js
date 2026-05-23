"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildDropOrder = buildDropOrder;
exports.analyzeDropOrder = analyzeDropOrder;
exports.getDropOrderForVar = getDropOrderForVar;
exports.longestLived = longestLived;
exports.shortestLived = shortestLived;
const runtime_types_1 = require("./runtime-types");
const runtime_mapper_1 = require("./runtime-mapper");
/** Build the global drop order from runtime events */
function buildDropOrder(events) {
    const drops = [];
    const creationTimes = new Map();
    const varNames = new Map();
    let scopeDepth = 0;
    let currentFn = null;
    const fnStack = [];
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        // Track creation timestamps
        if (type === "New" || type === "RcNew" || type === "ArcNew" || type === "RefCellNew" ||
            type === "CellNew" || type === "BoxNew" || type === "WeakNew" || type === "PinNew" ||
            type === "CowBorrowed" || type === "CowOwned" || type === "OnceCellNew" ||
            type === "MaybeUninitNew" || type === "StaticInit" || type === "RawPtrCreated") {
            if (data.var_id) {
                creationTimes.set(data.var_id, data.timestamp);
                varNames.set(data.var_id, data.var_name || data.var_id);
            }
        }
        // Track scope depth
        if (type === "RegionEnter" || type === "FnEnter") {
            scopeDepth++;
            if (type === "FnEnter") {
                fnStack.push(data.fn_name);
                currentFn = data.fn_name;
            }
        }
        if (type === "RegionExit" || type === "FnExit") {
            scopeDepth = Math.max(0, scopeDepth - 1);
            if (type === "FnExit") {
                fnStack.pop();
                currentFn = fnStack.length > 0 ? fnStack[fnStack.length - 1] : null;
            }
        }
        // Record drops
        if (type === "Drop" && data.var_id) {
            const createTime = creationTimes.get(data.var_id) ?? -1;
            drops.push({
                var_id: data.var_id,
                var_name: varNames.get(data.var_id) || data.var_id,
                order: drops.length + 1,
                timestamp: data.timestamp,
                location: (0, runtime_mapper_1.parseLocation)(data.location),
                scope_depth: scopeDepth,
                function_name: currentFn,
                lifetime_ns: createTime >= 0 ? data.timestamp - createTime : -1,
            });
        }
        // LockGuardDrop also counts as a drop
        if (type === "LockGuardDrop" && data.guard_id) {
            drops.push({
                var_id: data.guard_id,
                var_name: `guard(${data.guard_id})`,
                order: drops.length + 1,
                timestamp: data.timestamp,
                location: (0, runtime_mapper_1.parseLocation)(data.location),
                scope_depth: scopeDepth,
                function_name: currentFn,
                lifetime_ns: -1,
            });
        }
    }
    return drops;
}
/** Analyze drop order within a function */
function analyzeDropOrder(events, functionName) {
    const allDrops = buildDropOrder(events);
    const drops = functionName
        ? allDrops.filter(d => d.function_name === functionName)
        : allDrops;
    // Check LIFO order: variables declared later should be dropped first
    // Build declaration order from creation events
    const declOrder = new Map();
    let declIdx = 0;
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        if ((type === "New" || type === "RcNew" || type === "ArcNew" || type === "BoxNew" ||
            type === "RefCellNew" || type === "CellNew") && data.var_id) {
            if (!functionName || isInFunction(events, data.timestamp, functionName)) {
                declOrder.set(data.var_id, declIdx++);
            }
        }
    }
    // LIFO means drops should be in reverse declaration order
    const violations = [];
    const dropsWithDecl = drops.filter(d => declOrder.has(d.var_id));
    for (let i = 0; i < dropsWithDecl.length - 1; i++) {
        const current = dropsWithDecl[i];
        const next = dropsWithDecl[i + 1];
        const currentDecl = declOrder.get(current.var_id);
        const nextDecl = declOrder.get(next.var_id);
        // In LIFO, later-declared (higher declOrder) should drop first (lower drop order)
        if (currentDecl < nextDecl && current.scope_depth === next.scope_depth) {
            violations.push({
                expected_var: next.var_name,
                actual_var: current.var_name,
                expected_order: current.order,
                actual_order: next.order,
                description: `"${current.var_name}" dropped before "${next.var_name}" but was declared first (non-LIFO)`,
            });
        }
    }
    return {
        function_name: functionName || null,
        drops,
        is_lifo: violations.length === 0,
        violations,
    };
}
/** Get drop order for a specific variable */
function getDropOrderForVar(events, varId) {
    const drops = buildDropOrder(events);
    const entry = drops.find(d => d.var_id === varId);
    return entry ? entry.order : -1;
}
/** Get the longest-lived variables (by lifetime_ns) */
function longestLived(events, limit = 5) {
    const drops = buildDropOrder(events).filter(d => d.lifetime_ns > 0);
    return drops.sort((a, b) => b.lifetime_ns - a.lifetime_ns).slice(0, limit);
}
/** Get the shortest-lived variables */
function shortestLived(events, limit = 5) {
    const drops = buildDropOrder(events).filter(d => d.lifetime_ns > 0);
    return drops.sort((a, b) => a.lifetime_ns - b.lifetime_ns).slice(0, limit);
}
/** Check if a timestamp is within a function's FnEnter/FnExit */
function isInFunction(events, timestamp, fnName) {
    let inFn = false;
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        if (type === "FnEnter" && data.fn_name === fnName && data.timestamp <= timestamp)
            inFn = true;
        if (type === "FnExit" && data.fn_name === fnName && data.timestamp >= timestamp && inFn)
            return true;
    }
    return inFn;
}
//# sourceMappingURL=drop-order.js.map