"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.trackAsyncBorrows = trackAsyncBorrows;
const runtime_types_1 = require("./runtime-types");
const runtime_mapper_1 = require("./runtime-mapper");
/** Track borrows held across await points */
function trackAsyncBorrows(events, functionName) {
    // Filter to function if specified
    const fnEvents = functionName ? filterToFunction(events, functionName) : events;
    // Track active borrows
    const activeBorrows = new Map();
    // Track await points
    const awaitStarts = new Map();
    // Results
    const heldBorrows = [];
    const borrowAwaitMap = new Map(); // borrower_id → await points crossed
    let totalBorrows = 0;
    for (const event of fnEvents) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        // Borrow created
        if (type === "Borrow") {
            totalBorrows++;
            activeBorrows.set(data.borrower_id, {
                owner_id: data.owner_id,
                name: data.borrower_name,
                mutable: data.mutable,
                timestamp: data.timestamp,
                location: data.location || null,
            });
            borrowAwaitMap.set(data.borrower_id, []);
        }
        // RefCellBorrow also creates a borrow
        if (type === "RefCellBorrow") {
            totalBorrows++;
            activeBorrows.set(data.borrow_id, {
                owner_id: data.refcell_id,
                name: `refcell_borrow(${data.borrow_id})`,
                mutable: data.is_mutable,
                timestamp: data.timestamp,
                location: data.location || null,
            });
            borrowAwaitMap.set(data.borrow_id, []);
        }
        // Await start — record which borrows are active
        if (type === "AwaitStart") {
            awaitStarts.set(data.await_id, {
                timestamp: data.timestamp,
                future_name: data.future_name || "unknown",
                location: data.location || null,
                live_variables: data.live_variables || [],
            });
        }
        // Await end — compute duration and mark borrows that crossed it
        if (type === "AwaitEnd") {
            const start = awaitStarts.get(data.await_id);
            if (start) {
                const duration = data.timestamp - start.timestamp;
                const point = {
                    await_id: data.await_id,
                    future_name: start.future_name,
                    location: (0, runtime_mapper_1.parseLocation)(start.location),
                    start_timestamp: start.timestamp,
                    end_timestamp: data.timestamp,
                    duration_ns: duration,
                    poll_count: 1, // We'd need FuturePoll events to count accurately
                    live_variables: start.live_variables,
                };
                // Mark all active borrows as crossing this await
                for (const [borrowId] of activeBorrows) {
                    const points = borrowAwaitMap.get(borrowId);
                    if (points)
                        points.push(point);
                }
                awaitStarts.delete(data.await_id);
            }
        }
        // FuturePoll — increment poll count for active awaits
        if (type === "FuturePoll") {
            // Find the active await for this future and increment poll count
            for (const [, start] of awaitStarts) {
                if (start.future_name === data.name) {
                    // We'll count polls in the AwaitPoint when it ends
                }
            }
        }
        // Borrow released (Drop of borrower or RefCellDrop)
        if (type === "Drop" && activeBorrows.has(data.var_id)) {
            const borrow = activeBorrows.get(data.var_id);
            const awaitPoints = borrowAwaitMap.get(data.var_id) || [];
            if (awaitPoints.length > 0) {
                heldBorrows.push({
                    borrower_id: data.var_id,
                    borrower_name: borrow.name,
                    owner_id: borrow.owner_id,
                    mutable: borrow.mutable,
                    borrow_timestamp: borrow.timestamp,
                    borrow_location: (0, runtime_mapper_1.parseLocation)(borrow.location),
                    await_points: awaitPoints,
                    total_held_ns: data.timestamp - borrow.timestamp,
                    released_timestamp: data.timestamp,
                });
            }
            activeBorrows.delete(data.var_id);
            borrowAwaitMap.delete(data.var_id);
        }
        if (type === "RefCellDrop" && activeBorrows.has(data.borrow_id)) {
            const borrow = activeBorrows.get(data.borrow_id);
            const awaitPoints = borrowAwaitMap.get(data.borrow_id) || [];
            if (awaitPoints.length > 0) {
                heldBorrows.push({
                    borrower_id: data.borrow_id,
                    borrower_name: borrow.name,
                    owner_id: borrow.owner_id,
                    mutable: borrow.mutable,
                    borrow_timestamp: borrow.timestamp,
                    borrow_location: (0, runtime_mapper_1.parseLocation)(borrow.location),
                    await_points: awaitPoints,
                    total_held_ns: data.timestamp - borrow.timestamp,
                    released_timestamp: data.timestamp,
                });
            }
            activeBorrows.delete(data.borrow_id);
            borrowAwaitMap.delete(data.borrow_id);
        }
    }
    // Any borrows still active at end that crossed awaits
    for (const [borrowId, borrow] of activeBorrows) {
        const awaitPoints = borrowAwaitMap.get(borrowId) || [];
        if (awaitPoints.length > 0) {
            heldBorrows.push({
                borrower_id: borrowId,
                borrower_name: borrow.name,
                owner_id: borrow.owner_id,
                mutable: borrow.mutable,
                borrow_timestamp: borrow.timestamp,
                borrow_location: (0, runtime_mapper_1.parseLocation)(borrow.location),
                await_points: awaitPoints,
                total_held_ns: -1,
                released_timestamp: -1,
            });
        }
    }
    // Detect issues
    const issues = detectAsyncIssues(heldBorrows);
    // Find longest held
    let longestNs = 0;
    let longestVar = null;
    for (const h of heldBorrows) {
        if (h.total_held_ns > longestNs) {
            longestNs = h.total_held_ns;
            longestVar = h.borrower_name;
        }
    }
    return {
        function_name: functionName || null,
        total_borrows: totalBorrows,
        borrows_across_await: heldBorrows.length,
        longest_held_ns: longestNs,
        longest_held_var: longestVar,
        held_borrows: heldBorrows,
        issues,
    };
}
/** Detect issues with async borrows */
function detectAsyncIssues(held) {
    const issues = [];
    for (const h of held) {
        // Mutable borrow across await is more dangerous
        if (h.mutable) {
            issues.push({
                kind: "mut_borrow_across_await",
                severity: "error",
                borrower: h.borrower_name,
                owner: h.owner_id,
                description: `Mutable borrow "${h.borrower_name}" held across ${h.await_points.length} await point(s)`,
                suggestion: "Mutable borrows across await points can cause issues in multi-threaded runtimes. Consider restructuring to release the borrow before awaiting.",
            });
        }
        else {
            issues.push({
                kind: "borrow_across_await",
                severity: "warning",
                borrower: h.borrower_name,
                owner: h.owner_id,
                description: `Shared borrow "${h.borrower_name}" held across ${h.await_points.length} await point(s)`,
                suggestion: "Consider cloning the data before the await or restructuring to drop the borrow first.",
            });
        }
        // Multiple awaits crossed
        if (h.await_points.length > 1) {
            issues.push({
                kind: "multiple_awaits_crossed",
                severity: "warning",
                borrower: h.borrower_name,
                owner: h.owner_id,
                description: `Borrow "${h.borrower_name}" held across ${h.await_points.length} consecutive await points`,
                suggestion: "Long-lived borrows across multiple awaits increase the risk of deadlocks and reduce concurrency.",
            });
        }
        // Long held (> 100ms)
        if (h.total_held_ns > 100_000_000) {
            issues.push({
                kind: "long_held_borrow",
                severity: "info",
                borrower: h.borrower_name,
                owner: h.owner_id,
                description: `Borrow "${h.borrower_name}" held for ${(h.total_held_ns / 1_000_000).toFixed(1)}ms across await`,
                suggestion: "Long-held borrows may indicate a performance issue. Consider reducing the borrow scope.",
            });
        }
    }
    return issues;
}
/** Filter events to those within a specific function */
function filterToFunction(events, fnName) {
    const result = [];
    let inFn = false;
    let depth = 0;
    for (const event of events) {
        const type = (0, runtime_types_1.eventType)(event);
        const data = (0, runtime_types_1.eventData)(event);
        if (type === "FnEnter" && data.fn_name === fnName && !inFn) {
            inFn = true;
            depth = 1;
            result.push(event);
            continue;
        }
        if (inFn) {
            if (type === "FnEnter")
                depth++;
            if (type === "FnExit") {
                depth--;
                if (depth === 0) {
                    result.push(event);
                    inFn = false;
                    continue;
                }
            }
            result.push(event);
        }
    }
    return result;
}
//# sourceMappingURL=async-borrow-tracker.js.map