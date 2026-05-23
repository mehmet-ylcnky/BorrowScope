"use strict";
// Auto-generated from borrowscope-runtime/src/event.rs (88 event types)
// Matches serde JSON serialization format
Object.defineProperty(exports, "__esModule", { value: true });
exports.eventType = eventType;
exports.eventData = eventData;
/** Get the event type name (handles both internally and externally tagged) */
function eventType(event) {
    if (event.type)
        return event.type;
    return Object.keys(event)[0];
}
/** Get the event payload (handles both internally and externally tagged) */
function eventData(event) {
    if (event.type)
        return event;
    return Object.values(event)[0];
}
//# sourceMappingURL=runtime-types.js.map