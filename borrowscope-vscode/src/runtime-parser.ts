import { RuntimeEvent, eventType, eventData } from "./runtime-types";

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
export const OWNERSHIP_EVENT_TYPES = new Set([
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

export interface ParseResult {
  events: RuntimeEvent[];
  errors: string[];
  total: number;
  valid: number;
}

/** Parse raw JSON string into validated RuntimeEvent array */
export function parseEvents(json: string): ParseResult {
  const errors: string[] = [];
  let parsed: any;
  try {
    parsed = JSON.parse(json);
  } catch (e: any) {
    return { events: [], errors: [`Invalid JSON: ${e.message}`], total: 0, valid: 0 };
  }

  if (!Array.isArray(parsed)) {
    return { events: [], errors: ["Expected JSON array"], total: 0, valid: 0 };
  }

  const events: RuntimeEvent[] = [];
  for (let i = 0; i < parsed.length; i++) {
    const err = validateEvent(parsed[i], i);
    if (err) {
      errors.push(err);
    } else {
      events.push(parsed[i] as RuntimeEvent);
    }
  }

  return { events, errors, total: parsed.length, valid: events.length };
}

/** Validate a single event object. Returns error string or null if valid. */
export function validateEvent(event: any, index?: number): string | null {
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
export function filterByFile(events: RuntimeEvent[], filePath: string): RuntimeEvent[] {
  const suffix = filePath.replace(/\\/g, "/");
  return events.filter((e) => {
    const data = eventData(e);
    const loc = data.location as string | undefined;
    if (!loc) return false;
    return loc.replace(/\\/g, "/").endsWith(suffix) || loc.replace(/\\/g, "/").includes(suffix);
  });
}

/** Filter events by type name(s) */
export function filterByType(events: RuntimeEvent[], types: string | string[]): RuntimeEvent[] {
  const set = new Set(Array.isArray(types) ? types : [types]);
  return events.filter((e) => set.has(eventType(e)));
}

/** Filter to only ownership-relevant events */
export function filterOwnershipEvents(events: RuntimeEvent[]): RuntimeEvent[] {
  return events.filter((e) => OWNERSHIP_EVENT_TYPES.has(eventType(e)));
}

/** Group events by variable (var_id or var_name) */
export function groupByVariable(events: RuntimeEvent[]): Map<string, RuntimeEvent[]> {
  const groups = new Map<string, RuntimeEvent[]>();

  for (const event of events) {
    const data = eventData(event);
    // Try var_id first, then var_name, then other id fields
    const key = data.var_id || data.var_name || data.borrower_id || data.from_id || data.closure_id || data.guard_id || null;
    if (key) {
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(event);
    }
  }

  return groups;
}

/** Group events by function (using FnEnter/FnExit boundaries) */
export function groupByFunction(events: RuntimeEvent[]): Map<string, RuntimeEvent[]> {
  const groups = new Map<string, RuntimeEvent[]>();
  const stack: string[] = [];

  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);

    if (type === "FnEnter") {
      stack.push(data.fn_name);
      if (!groups.has(data.fn_name)) groups.set(data.fn_name, []);
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
export function sortByTimestamp(events: RuntimeEvent[]): RuntimeEvent[] {
  return [...events].sort((a, b) => eventData(a).timestamp - eventData(b).timestamp);
}

/** Get event count summary by type */
export function summarize(events: RuntimeEvent[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const e of events) {
    const t = eventType(e);
    counts[t] = (counts[t] || 0) + 1;
  }
  return counts;
}
