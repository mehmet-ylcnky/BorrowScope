import { RuntimeEvent, eventType, eventData } from "./runtime-types";
import { StaticVariable, MappedVariable, mapVariables, parseLocation } from "./runtime-mapper";

/** Runtime-derived data for a variable */
export interface RuntimeInfo {
  actual_lifetime_ns: number;       // timestamp of Drop - timestamp of New (-1 if not dropped)
  actual_borrow_count: number;      // how many Borrow events reference this var
  actual_mut_borrow_count: number;  // mutable borrows
  was_actually_moved: boolean;      // Move event with from_id = this var
  move_destination: string | null;  // to_name if moved
  drop_order: number;               // position in global drop sequence (-1 if not dropped)
  drop_timestamp: number;           // timestamp of Drop event
  ref_count_peak: number;           // max strong_count for Rc/Arc
  ref_count_final: number;          // last known strong_count
  weak_count_peak: number;          // max weak_count
  clone_count: number;              // number of RcClone/ArcClone from this source
  thread_id: number | null;         // thread that created this var (from FnEnter context)
  await_crossings: AwaitCrossing[]; // borrows held across await points
  unsafe_accesses: number;          // times accessed in unsafe blocks
  event_count: number;              // total events for this var
}

/** A borrow held across an await point */
export interface AwaitCrossing {
  await_line: number;
  duration_ns: number;
  future_name: string;
}

/** Agreement between static and runtime analysis */
export type Agreement = "match" | "diverge" | "runtime_only" | "static_only";

/** Reason for divergence */
export interface Divergence {
  kind: "conditional_move" | "rc_leak" | "borrow_count_mismatch" | "type_mismatch" |
        "unexpected_drop" | "missing_drop" | "unsafe_hidden" | "async_borrow_held";
  description: string;
}

/** Combined static + runtime view of a variable */
export interface MergedVariable {
  name: string;
  var_id: string | null;
  line: number;
  file: string | null;

  static_info: {
    type_display: string;
    ownership_category: string;
    is_copy: boolean;
  } | null;

  runtime_info: RuntimeInfo | null;

  agreement: Agreement;
  divergences: Divergence[];
}

/** Merge static analysis graph with runtime events */
export function mergeViews(
  staticVars: StaticVariable[],
  events: RuntimeEvent[],
  file?: string
): MergedVariable[] {
  const mapped = mapVariables(staticVars, events, file);
  const dropOrder = buildDropOrder(events);
  const result: MergedVariable[] = [];

  // Process mapped variables (have runtime data)
  for (const m of mapped) {
    const runtimeInfo = buildRuntimeInfo(m, events, dropOrder);
    const divergences = detectDivergences(m.static_match, runtimeInfo);
    const agreement = computeAgreement(m.static_match, runtimeInfo, divergences);

    result.push({
      name: m.var_name,
      var_id: m.var_id,
      line: m.location?.line ?? m.static_match?.line ?? 0,
      file: m.location?.file ?? file ?? null,
      static_info: m.static_match ? {
        type_display: m.static_match.type_display,
        ownership_category: m.static_match.ownership_category,
        is_copy: m.static_match.is_copy ?? false,
      } : null,
      runtime_info: runtimeInfo,
      agreement,
      divergences,
    });
  }

  // Add static-only variables (no runtime match)
  const mappedNames = new Set(mapped.map((m) => `${m.var_name}:${m.static_match?.line}`));
  for (const sv of staticVars) {
    if (!mappedNames.has(`${sv.name}:${sv.line}`)) {
      result.push({
        name: sv.name,
        var_id: null,
        line: sv.line,
        file: file ?? null,
        static_info: {
          type_display: sv.type_display,
          ownership_category: sv.ownership_category,
          is_copy: sv.is_copy ?? false,
        },
        runtime_info: null,
        agreement: "static_only",
        divergences: [],
      });
    }
  }

  return result;
}

/** Build runtime info from a mapped variable's events */
function buildRuntimeInfo(
  mapped: MappedVariable,
  allEvents: RuntimeEvent[],
  dropOrder: Map<string, number>
): RuntimeInfo {
  const events = mapped.events;
  let createTimestamp = 0;
  let dropTimestamp = -1;
  let borrowCount = 0;
  let mutBorrowCount = 0;
  let wasMoved = false;
  let moveDest: string | null = null;
  let refCountPeak = 0;
  let refCountFinal = 0;
  let weakCountPeak = 0;
  let cloneCount = 0;
  let unsafeAccesses = 0;

  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);

    switch (type) {
      case "New": case "RcNew": case "ArcNew": case "RefCellNew":
      case "CellNew": case "BoxNew": case "WeakNew": case "PinNew":
      case "CowBorrowed": case "CowOwned": case "OnceCellNew":
      case "MaybeUninitNew": case "StaticInit": case "RawPtrCreated":
        createTimestamp = data.timestamp;
        if (data.strong_count !== undefined) {
          refCountPeak = Math.max(refCountPeak, data.strong_count);
          refCountFinal = data.strong_count;
        }
        if (data.weak_count !== undefined) {
          weakCountPeak = Math.max(weakCountPeak, data.weak_count);
        }
        break;

      case "Drop":
        dropTimestamp = data.timestamp;
        break;

      case "Borrow":
        borrowCount++;
        if (data.mutable) mutBorrowCount++;
        break;

      case "Move":
        if (data.from_id === mapped.var_id) {
          wasMoved = true;
          moveDest = data.to_name || null;
        }
        break;

      case "RcClone": case "ArcClone": case "WeakClone":
        cloneCount++;
        if (data.strong_count !== undefined) {
          refCountPeak = Math.max(refCountPeak, data.strong_count);
          refCountFinal = data.strong_count;
        }
        if (data.weak_count !== undefined) {
          weakCountPeak = Math.max(weakCountPeak, data.weak_count);
        }
        break;

      case "RawPtrDeref": case "UnsafeBlockEnter":
        unsafeAccesses++;
        break;
    }
  }

  // Detect await crossings
  const awaitCrossings = detectAwaitCrossings(mapped.var_id, allEvents);

  return {
    actual_lifetime_ns: dropTimestamp >= 0 ? dropTimestamp - createTimestamp : -1,
    actual_borrow_count: borrowCount,
    actual_mut_borrow_count: mutBorrowCount,
    was_actually_moved: wasMoved,
    move_destination: moveDest,
    drop_order: dropOrder.get(mapped.var_id) ?? -1,
    drop_timestamp: dropTimestamp,
    ref_count_peak: refCountPeak,
    ref_count_final: refCountFinal,
    weak_count_peak: weakCountPeak,
    clone_count: cloneCount,
    thread_id: null,
    await_crossings: awaitCrossings,
    unsafe_accesses: unsafeAccesses,
    event_count: events.length,
  };
}

/** Build global drop order map (var_id → order index) */
function buildDropOrder(events: RuntimeEvent[]): Map<string, number> {
  const order = new Map<string, number>();
  let idx = 0;
  for (const event of events) {
    if (eventType(event) === "Drop") {
      const data = eventData(event);
      if (data.var_id) {
        order.set(data.var_id, idx++);
      }
    }
  }
  return order;
}

/** Detect borrows held across await points */
function detectAwaitCrossings(varId: string, events: RuntimeEvent[]): AwaitCrossing[] {
  const crossings: AwaitCrossing[] = [];
  // Find borrows of this var that are active during an await
  let activeBorrows = new Set<string>();

  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);

    if (type === "Borrow" && data.owner_id === varId) {
      activeBorrows.add(data.borrower_id);
    }
    if (type === "Drop" && activeBorrows.has(data.var_id)) {
      activeBorrows.delete(data.var_id);
    }
    if (type === "AwaitStart" && activeBorrows.size > 0) {
      const awaitStart = data.timestamp;
      const loc = parseLocation(data.location);
      // Find matching AwaitEnd
      const endEvent = events.find(
        (e) => eventType(e) === "AwaitEnd" && eventData(e).await_id === data.await_id
      );
      const duration = endEvent ? eventData(endEvent).timestamp - awaitStart : 0;
      crossings.push({
        await_line: loc?.line ?? 0,
        duration_ns: duration,
        future_name: data.future_name || "unknown",
      });
    }
  }

  return crossings;
}

/** Detect divergences between static and runtime */
function detectDivergences(
  staticVar: StaticVariable | null,
  runtime: RuntimeInfo
): Divergence[] {
  const divergences: Divergence[] = [];
  if (!staticVar) return divergences;

  // Rc/Arc leak: created but never dropped, ref_count > 0
  if ((staticVar.ownership_category === "Rc" || staticVar.ownership_category === "Arc") &&
      runtime.drop_timestamp < 0 && runtime.ref_count_final > 0) {
    divergences.push({
      kind: "rc_leak",
      description: `${staticVar.ownership_category} never dropped (final ref count: ${runtime.ref_count_final})`,
    });
  }

  // Conditional move: static says owned (could move) but runtime shows no move
  if (staticVar.ownership_category === "Owned" && !staticVar.is_copy &&
      !runtime.was_actually_moved && runtime.drop_timestamp >= 0) {
    // This is normal — not necessarily a divergence unless static predicted a move
    // Only flag if there were 0 borrows (suggesting it was meant to be moved)
  }

  // Missing drop: not Copy, not moved, but never dropped
  if (!staticVar.is_copy && !runtime.was_actually_moved && runtime.drop_timestamp < 0 &&
      staticVar.ownership_category !== "Rc" && staticVar.ownership_category !== "Arc") {
    divergences.push({
      kind: "missing_drop",
      description: `Variable "${staticVar.name}" was never dropped (possible leak or program exit)`,
    });
  }

  // Async borrow held across await
  if (runtime.await_crossings.length > 0) {
    divergences.push({
      kind: "async_borrow_held",
      description: `Borrow held across ${runtime.await_crossings.length} await point(s)`,
    });
  }

  // Unsafe hidden info
  if (runtime.unsafe_accesses > 0 && staticVar.ownership_category !== "RawPtr") {
    divergences.push({
      kind: "unsafe_hidden",
      description: `${runtime.unsafe_accesses} unsafe access(es) — static analysis may be incomplete`,
    });
  }

  return divergences;
}

/** Compute overall agreement */
function computeAgreement(
  staticVar: StaticVariable | null,
  runtime: RuntimeInfo,
  divergences: Divergence[]
): Agreement {
  if (!staticVar) return "runtime_only";
  if (divergences.length > 0) return "diverge";
  return "match";
}

/** Get merge summary statistics */
export function mergeSummary(merged: MergedVariable[]): {
  total: number;
  match: number;
  diverge: number;
  runtime_only: number;
  static_only: number;
} {
  const s = { total: merged.length, match: 0, diverge: 0, runtime_only: 0, static_only: 0 };
  for (const m of merged) {
    s[m.agreement]++;
  }
  return s;
}
