import { RuntimeEvent, eventType, eventData } from "./runtime-types";
import { parseLocation, SourceLocation } from "./runtime-mapper";

/** A single drop event with context */
export interface DropEntry {
  var_id: string;
  var_name: string;
  order: number;          // 1-based global drop order
  timestamp: number;
  location: SourceLocation | null;
  scope_depth: number;    // nesting level (from RegionEnter/Exit)
  function_name: string | null;
  lifetime_ns: number;    // time from New to Drop (-1 if unknown)
}

/** Drop order analysis for a function or scope */
export interface DropOrderAnalysis {
  function_name: string | null;
  drops: DropEntry[];
  is_lifo: boolean;       // true if drops follow LIFO (reverse declaration) order
  violations: DropOrderViolation[];
}

/** A violation of expected LIFO drop order */
export interface DropOrderViolation {
  expected_var: string;
  actual_var: string;
  expected_order: number;
  actual_order: number;
  description: string;
}

/** Build the global drop order from runtime events */
export function buildDropOrder(events: RuntimeEvent[]): DropEntry[] {
  const drops: DropEntry[] = [];
  const creationTimes = new Map<string, number>();
  const varNames = new Map<string, string>();
  let scopeDepth = 0;
  let currentFn: string | null = null;
  const fnStack: string[] = [];

  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);

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
        location: parseLocation(data.location),
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
        location: parseLocation(data.location),
        scope_depth: scopeDepth,
        function_name: currentFn,
        lifetime_ns: -1,
      });
    }
  }

  return drops;
}

/** Analyze drop order within a function */
export function analyzeDropOrder(events: RuntimeEvent[], functionName?: string): DropOrderAnalysis {
  const allDrops = buildDropOrder(events);
  const drops = functionName
    ? allDrops.filter(d => d.function_name === functionName)
    : allDrops;

  // Check LIFO order: variables declared later should be dropped first
  // Build declaration order from creation events
  const declOrder = new Map<string, number>();
  let declIdx = 0;
  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);
    if ((type === "New" || type === "RcNew" || type === "ArcNew" || type === "BoxNew" ||
         type === "RefCellNew" || type === "CellNew") && data.var_id) {
      if (!functionName || isInFunction(events, data.timestamp, functionName)) {
        declOrder.set(data.var_id, declIdx++);
      }
    }
  }

  // LIFO means drops should be in reverse declaration order
  const violations: DropOrderViolation[] = [];
  const dropsWithDecl = drops.filter(d => declOrder.has(d.var_id));

  for (let i = 0; i < dropsWithDecl.length - 1; i++) {
    const current = dropsWithDecl[i];
    const next = dropsWithDecl[i + 1];
    const currentDecl = declOrder.get(current.var_id)!;
    const nextDecl = declOrder.get(next.var_id)!;

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
export function getDropOrderForVar(events: RuntimeEvent[], varId: string): number {
  const drops = buildDropOrder(events);
  const entry = drops.find(d => d.var_id === varId);
  return entry ? entry.order : -1;
}

/** Get the longest-lived variables (by lifetime_ns) */
export function longestLived(events: RuntimeEvent[], limit: number = 5): DropEntry[] {
  const drops = buildDropOrder(events).filter(d => d.lifetime_ns > 0);
  return drops.sort((a, b) => b.lifetime_ns - a.lifetime_ns).slice(0, limit);
}

/** Get the shortest-lived variables */
export function shortestLived(events: RuntimeEvent[], limit: number = 5): DropEntry[] {
  const drops = buildDropOrder(events).filter(d => d.lifetime_ns > 0);
  return drops.sort((a, b) => a.lifetime_ns - b.lifetime_ns).slice(0, limit);
}

/** Check if a timestamp is within a function's FnEnter/FnExit */
function isInFunction(events: RuntimeEvent[], timestamp: number, fnName: string): boolean {
  let inFn = false;
  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);
    if (type === "FnEnter" && data.fn_name === fnName && data.timestamp <= timestamp) inFn = true;
    if (type === "FnExit" && data.fn_name === fnName && data.timestamp >= timestamp && inFn) return true;
  }
  return inFn;
}
