"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseLocation = parseLocation;
exports.findStaticMatch = findStaticMatch;
exports.mapVariables = mapVariables;
exports.mappingStats = mappingStats;
const runtime_types_1 = require("./runtime-types");
/** Parse a location string like "src/main.rs:5:10" into components */
function parseLocation(location) {
    if (!location)
        return null;
    const match = location.match(/^(.+):(\d+):(\d+)$/);
    if (!match) {
        // Try without column: "src/main.rs:5"
        const match2 = location.match(/^(.+):(\d+)$/);
        if (!match2)
            return null;
        return { file: match2[1], line: parseInt(match2[2], 10), column: 0 };
    }
    return { file: match[1], line: parseInt(match[2], 10), column: parseInt(match[3], 10) };
}
/** Find the best static variable match for a runtime variable */
function findStaticMatch(runtimeName, runtimeLine, runtimeType, staticVars) {
    // 1. Exact: name + line + type all match
    if (runtimeLine !== null && runtimeType) {
        const exact = staticVars.find((v) => v.name === runtimeName && v.line === runtimeLine && typesMatch(v.type_display, runtimeType));
        if (exact)
            return { match: exact, confidence: "exact" };
    }
    // 2. Name + line match
    if (runtimeLine !== null) {
        const byLine = staticVars.find((v) => v.name === runtimeName && v.line === runtimeLine);
        if (byLine)
            return { match: byLine, confidence: "name_line" };
    }
    // 3. Name + type match (for shadowed variables where line differs)
    if (runtimeType) {
        const byType = staticVars.find((v) => v.name === runtimeName && typesMatch(v.type_display, runtimeType));
        if (byType)
            return { match: byType, confidence: "name_type" };
    }
    // 4. Name only (last resort)
    const byName = staticVars.find((v) => v.name === runtimeName);
    if (byName)
        return { match: byName, confidence: "name_only" };
    return { match: null, confidence: "none" };
}
/** Check if two type strings are compatible (handles short vs full paths) */
function typesMatch(staticType, runtimeType) {
    if (staticType === runtimeType)
        return true;
    // "Vec<i32>" matches "alloc::vec::Vec<i32>"
    if (runtimeType.endsWith(staticType))
        return true;
    if (staticType.endsWith(runtimeType))
        return true;
    // Strip paths: "std::string::String" → "String"
    const shortStatic = staticType.split("::").pop() || staticType;
    const shortRuntime = runtimeType.split("::").pop() || runtimeType;
    return shortStatic === shortRuntime;
}
/** Extract var_name, var_id, type_name, and location from any event */
function extractVarInfo(event) {
    const data = (0, runtime_types_1.eventData)(event);
    return {
        var_id: data.var_id || data.borrower_id || data.borrow_id || data.closure_id ||
            data.guard_id || data.pin_id || data.ptr_id || data.weak_id ||
            data.from_id || data.to_id || null,
        var_name: data.var_name || data.borrower_name || data.to_name ||
            data.fn_name || data.const_name || null,
        type_name: data.type_name || data.ptr_type || data.lock_type || null,
        location: data.location || null,
    };
}
/**
 * Map runtime events to static variables.
 * Returns a MappedVariable for each unique var_id found in runtime events.
 */
function mapVariables(staticVars, events, targetFile) {
    // Collect all unique variables from runtime events
    const varMap = new Map();
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const info = extractVarInfo(event);
        // Only map variable-creating events for the primary mapping
        if (type === "New" || type === "RcNew" || type === "ArcNew" || type === "RefCellNew" ||
            type === "CellNew" || type === "BoxNew" || type === "WeakNew" || type === "PinNew" ||
            type === "CowBorrowed" || type === "CowOwned" || type === "OnceCellNew" ||
            type === "MaybeUninitNew" || type === "StaticInit" || type === "RawPtrCreated" ||
            type === "StructCreate" || type === "TupleCreate" || type === "ArrayCreate") {
            if (info.var_id && info.var_name) {
                if (!varMap.has(info.var_id)) {
                    varMap.set(info.var_id, { var_name: info.var_name, type_name: info.type_name, location: info.location, events: [] });
                }
            }
        }
        // Associate all events with their var_id
        if (info.var_id && varMap.has(info.var_id)) {
            varMap.get(info.var_id).events.push(event);
        }
    }
    // Also collect events referencing var_ids we know about (Drop, Borrow targets, etc.)
    for (const event of events) {
        const data = (0, runtime_types_1.eventData)(event);
        const type = (0, runtime_types_1.eventType)(event);
        // Drop references var_id
        if (type === "Drop" && data.var_id && varMap.has(data.var_id)) {
            const entry = varMap.get(data.var_id);
            if (!entry.events.includes(event))
                entry.events.push(event);
        }
        // Borrow references owner_id
        if (type === "Borrow" && data.owner_id && varMap.has(data.owner_id)) {
            const entry = varMap.get(data.owner_id);
            if (!entry.events.includes(event))
                entry.events.push(event);
        }
        // Move references from_id
        if (type === "Move" && data.from_id && varMap.has(data.from_id)) {
            const entry = varMap.get(data.from_id);
            if (!entry.events.includes(event))
                entry.events.push(event);
        }
        // RcClone/ArcClone reference source_id
        if ((type === "RcClone" || type === "ArcClone" || type === "WeakClone") && data.source_id && varMap.has(data.source_id)) {
            const entry = varMap.get(data.source_id);
            if (!entry.events.includes(event))
                entry.events.push(event);
        }
    }
    // Now match each runtime variable to a static variable
    const result = [];
    for (const [varId, info] of varMap) {
        const loc = parseLocation(info.location);
        // Filter by target file if specified
        if (targetFile && loc) {
            const normalizedLoc = loc.file.replace(/\\/g, "/");
            const normalizedTarget = targetFile.replace(/\\/g, "/");
            if (!normalizedLoc.endsWith(normalizedTarget) && !normalizedLoc.includes(normalizedTarget)) {
                continue;
            }
        }
        const { match, confidence } = findStaticMatch(info.var_name, loc?.line ?? null, info.type_name, staticVars);
        result.push({
            var_id: varId,
            var_name: info.var_name,
            runtime_type: info.type_name || "unknown",
            location: loc,
            static_match: match,
            match_confidence: confidence,
            events: info.events,
        });
    }
    return result;
}
/** Get mapping statistics */
function mappingStats(mapped) {
    const stats = { total: mapped.length, exact: 0, name_line: 0, name_type: 0, name_only: 0, unmatched: 0 };
    for (const m of mapped) {
        if (m.match_confidence === "exact")
            stats.exact++;
        else if (m.match_confidence === "name_line")
            stats.name_line++;
        else if (m.match_confidence === "name_type")
            stats.name_type++;
        else if (m.match_confidence === "name_only")
            stats.name_only++;
        else
            stats.unmatched++;
    }
    return stats;
}
//# sourceMappingURL=runtime-mapper.js.map