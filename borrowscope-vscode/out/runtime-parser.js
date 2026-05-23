"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OWNERSHIP_EVENT_TYPES = void 0;
exports.parseEvents = parseEvents;
exports.validateEvent = validateEvent;
exports.filterByFile = filterByFile;
exports.filterByType = filterByType;
exports.filterOwnershipEvents = filterOwnershipEvents;
exports.groupByVariable = groupByVariable;
exports.groupByFunction = groupByFunction;
exports.sortByTimestamp = sortByTimestamp;
exports.summarize = summarize;
const runtime_types_1 = require("./runtime-types");
/** All known event type names from borrowscope-runtime */
const KNOWN_EVENT_TYPES = new Set([
    "New", "Borrow", "Move", "Drop",
    "RcNew", "RcClone", "ArcNew", "ArcClone",
    "RefCellNew", "RefCellBorrow", "RefCellDrop",
    "CellNew", "CellGet", "CellSet",
    "StaticInit", "StaticAccess", "ConstEval",
    "RawPtrCreated", "RawPtrDeref",
    "UnsafeBlockEnter", "UnsafeBlockExit", "UnsafeFnCall", "FfiCall", "Transmute", "UnionFieldAccess",
    "AsyncBlockEnter", "AsyncBlockExit", "AwaitStart", "AwaitEnd",
    "LoopEnter", "LoopIteration", "LoopExit",
    "MatchEnter", "MatchArm", "MatchExit",
    "Branch", "Return", "Try",
    "IndexAccess", "FieldAccess", "Call",
    "Lock", "Unwrap", "Clone", "Deref",
    "Break", "Continue",
    "ClosureCreate", "StructCreate", "TupleCreate",
    "LetElse", "Range", "BinaryOp", "ArrayCreate", "TypeCast",
    "RegionEnter", "RegionExit",
    "FnEnter", "FnExit", "ClosureCapture",
    "WeakNew", "WeakClone", "WeakUpgrade",
    "BoxNew", "BoxIntoRaw", "BoxFromRaw",
    "LockGuardAcquire", "LockGuardDrop",
    "PinNew", "PinIntoInner",
    "CowBorrowed", "CowOwned", "CowToMut",
    "ThreadSpawn", "ThreadJoin",
    "ChannelSenderNew", "ChannelReceiverNew", "ChannelSend", "ChannelRecv",
    "OnceCellNew", "OnceCellSet", "OnceCellGet", "OnceCellGetOrInit",
    "MaybeUninitNew", "MaybeUninitWrite", "MaybeUninitAssumeInit",
    "MaybeUninitAssumeInitRead", "MaybeUninitAssumeInitDrop",
]);
/** Ownership-relevant event types for the merge/overlay */
exports.OWNERSHIP_EVENT_TYPES = new Set([
    "New", "Drop", "Borrow", "Move",
    "RcNew", "RcClone", "ArcNew", "ArcClone",
    "RefCellNew", "RefCellBorrow", "RefCellDrop",
    "CellNew", "CellGet", "CellSet",
    "WeakNew", "WeakClone", "WeakUpgrade",
    "BoxNew", "BoxIntoRaw", "BoxFromRaw",
    "Clone", "Deref",
    "FnEnter", "FnExit",
    "AwaitStart", "AwaitEnd",
]);
/** Parse raw JSON string into validated RuntimeEvent array */
function parseEvents(json) {
    const errors = [];
    let parsed;
    try {
        parsed = JSON.parse(json);
    }
    catch (e) {
        return { events: [], errors: [`Invalid JSON: ${e.message}`], total: 0, valid: 0 };
    }
    if (!Array.isArray(parsed)) {
        return { events: [], errors: ["Expected JSON array"], total: 0, valid: 0 };
    }
    const events = [];
    for (let i = 0; i < parsed.length; i++) {
        const err = validateEvent(parsed[i], i);
        if (err) {
            errors.push(err);
        }
        else {
            events.push(parsed[i]);
        }
    }
    return { events, errors, total: parsed.length, valid: events.length };
}
/** Validate a single event object. Returns error string or null if valid. */
function validateEvent(event, index) {
    const prefix = index !== undefined ? `Event[${index}]` : "Event";
    if (!event || typeof event !== "object") {
        return `${prefix}: not an object`;
    }
    // Support both formats:
    // 1. Internally tagged (serde): {"type": "New", "var_name": "x", ...}
    // 2. Externally tagged: {"New": {"var_name": "x", ...}}
    if (event.type) {
        // Internally tagged format
        if (!KNOWN_EVENT_TYPES.has(event.type)) {
            return `${prefix}: unknown event type "${event.type}"`;
        }
        if (typeof event.timestamp !== "number") {
            return `${prefix}: "${event.type}" missing or invalid timestamp`;
        }
        return null;
    }
    // Externally tagged format
    const keys = Object.keys(event);
    if (keys.length !== 1) {
        return `${prefix}: expected exactly one key (event type), got ${keys.length}`;
    }
    const type = keys[0];
    if (!KNOWN_EVENT_TYPES.has(type)) {
        return `${prefix}: unknown event type "${type}"`;
    }
    const data = event[type];
    if (!data || typeof data !== "object") {
        return `${prefix}: payload for "${type}" is not an object`;
    }
    if (typeof data.timestamp !== "number") {
        return `${prefix}: "${type}" missing or invalid timestamp`;
    }
    return null;
}
/** Filter events by file path (matches location field) */
function filterByFile(events, filePath) {
    const suffix = filePath.replace(/\\/g, "/");
    return events.filter((e) => {
        const data = (0, runtime_types_1.eventData)(e);
        const loc = data.location;
        if (!loc)
            return false;
        return loc.replace(/\\/g, "/").endsWith(suffix) || loc.replace(/\\/g, "/").includes(suffix);
    });
}
/** Filter events by type name(s) */
function filterByType(events, types) {
    const set = new Set(Array.isArray(types) ? types : [types]);
    return events.filter((e) => set.has((0, runtime_types_1.eventType)(e)));
}
/** Filter to only ownership-relevant events */
function filterOwnershipEvents(events) {
    return events.filter((e) => exports.OWNERSHIP_EVENT_TYPES.has((0, runtime_types_1.eventType)(e)));
}
/** Group events by variable (var_id or var_name) */
function groupByVariable(events) {
    const groups = new Map();
    for (const event of events) {
        const data = (0, runtime_types_1.eventData)(event);
        // Try var_id first, then var_name, then other id fields
        const key = data.var_id || data.var_name || data.borrower_id || data.from_id || data.closure_id || data.guard_id || null;
        if (key) {
            if (!groups.has(key))
                groups.set(key, []);
            groups.get(key).push(event);
        }
    }
    return groups;
}
/** Group events by function (using FnEnter/FnExit boundaries) */
function groupByFunction(events) {
    const groups = new Map();
    const stack = [];
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        if (type === "FnEnter") {
            stack.push(data.fn_name);
            if (!groups.has(data.fn_name))
                groups.set(data.fn_name, []);
        }
        if (stack.length > 0) {
            const current = stack[stack.length - 1];
            groups.get(current)?.push(event);
        }
        if (type === "FnExit") {
            stack.pop();
        }
    }
    return groups;
}
/** Sort events by timestamp */
function sortByTimestamp(events) {
    return [...events].sort((a, b) => (0, runtime_types_1.eventData)(a).timestamp - (0, runtime_types_1.eventData)(b).timestamp);
}
/** Get event count summary by type */
function summarize(events) {
    const counts = {};
    for (const e of events) {
        const t = (0, runtime_types_1.eventType)(e);
        counts[t] = (counts[t] || 0) + 1;
    }
    return counts;
}
//# sourceMappingURL=runtime-parser.js.map