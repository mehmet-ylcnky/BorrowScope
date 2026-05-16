import { RuntimeEvent, eventType, eventData } from "./runtime-types";

/** A point in the reference count timeline */
export interface RefCountPoint {
  timestamp: number;
  line: number;
  strong_count: number;
  weak_count: number;
  event: "new" | "clone" | "drop" | "weak_new" | "weak_clone" | "weak_drop" | "upgrade_ok" | "upgrade_fail";
  variable: string;
  source_variable?: string;
}

/** Complete timeline for a single Rc/Arc variable */
export interface RefCountTimeline {
  var_id: string;
  var_name: string;
  type_name: string;
  is_arc: boolean;
  points: RefCountPoint[];
  peak_strong: number;
  peak_weak: number;
  final_strong: number;
  final_weak: number;
  leaked: boolean;
  total_clones: number;
  total_drops: number;
}

/** Build a ref count timeline for a specific Rc/Arc variable */
export function buildRefCountTimeline(events: RuntimeEvent[], varId: string): RefCountTimeline | null {
  let varName = "";
  let typeName = "";
  let isArc = false;
  let strong = 0;
  let weak = 0;
  let peakStrong = 0;
  let peakWeak = 0;
  let totalClones = 0;
  let totalDrops = 0;
  const points: RefCountPoint[] = [];

  for (const event of events) {
    const type = eventType(event);
    const data = eventData(event);

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

  if (points.length === 0) return null;

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
export function buildAllRefCountTimelines(events: RuntimeEvent[]): RefCountTimeline[] {
  const rcVarIds = new Set<string>();

  for (const event of events) {
    const type = eventType(event);
    if (type === "RcNew" || type === "ArcNew") {
      rcVarIds.add(eventData(event).var_id);
    }
  }

  const timelines: RefCountTimeline[] = [];
  for (const varId of rcVarIds) {
    const timeline = buildRefCountTimeline(events, varId);
    if (timeline) timelines.push(timeline);
  }

  return timelines;
}

/** Get inline decoration text for a ref count point */
export function refCountAnnotation(point: RefCountPoint): string {
  const arrow = point.event === "clone" ? "↑" : point.event === "drop" ? "↓" : "";
  return `🔗 refs: ${point.strong_count}${arrow}${point.weak_count > 0 ? ` (weak: ${point.weak_count})` : ""}`;
}

/** Detect potential Rc/Arc issues from timelines */
export function detectRefCountIssues(timelines: RefCountTimeline[]): RefCountIssue[] {
  const issues: RefCountIssue[] = [];

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

export interface RefCountIssue {
  var_id: string;
  var_name: string;
  kind: "leak" | "high_refcount" | "dangling_weak";
  description: string;
  peak_count: number;
}

function parseLine(location: string | undefined): number {
  if (!location) return 0;
  const match = location.match(/:(\d+):/);
  return match ? parseInt(match[1], 10) : 0;
}
