export interface RefCountSeries {
  name: string;
  type: string; // "Rc" or "Arc"
  events: RefCountEvent[];
}

export interface RefCountEvent {
  line: number;
  count: number;
  action: string; // "new", "clone", "drop"
  variable: string;
}

export function buildRefCountData(graph: any): RefCountSeries[] {
  const clones = graph.rc_clones || [];
  const variables = graph.variables || [];
  const fnEnd = graph.end_line || 20;

  // Group by source variable (the original Rc/Arc)
  const families: Map<string, RefCountSeries> = new Map();

  // Find Rc/Arc variables
  for (const v of variables) {
    if (v.ownership_category === "Rc" || v.ownership_category === "Arc") {
      // Check if this is a source (not a clone)
      const isClone = clones.some((c: any) => c.clone_variable === v.name);
      if (!isClone) {
        families.set(v.name, {
          name: v.name,
          type: v.ownership_category,
          events: [{ line: v.line, count: 1, action: "new", variable: v.name }],
        });
      }
    }
  }

  // Add clone events
  for (const c of clones) {
    const source = c.source_variable;
    let series = families.get(source);
    if (!series) {
      // Source might not be in variables (edge case)
      series = { name: source, type: c.clone_type || "Rc", events: [{ line: 1, count: 1, action: "new", variable: source }] };
      families.set(source, series);
    }
    const prevCount = series.events[series.events.length - 1].count;
    series.events.push({ line: c.line, count: prevCount + 1, action: "clone", variable: c.clone_variable });
  }

  // Add drop events (estimate: clones drop before source, at function end)
  for (const [source, series] of families) {
    const cloneVars = clones.filter((c: any) => c.source_variable === source);
    // Each clone drops (reverse order, before function end)
    for (let i = cloneVars.length - 1; i >= 0; i--) {
      const prevCount = series.events[series.events.length - 1].count;
      const dropLine = fnEnd - (cloneVars.length - i);
      series.events.push({ line: dropLine, count: prevCount - 1, action: "drop", variable: cloneVars[i].clone_variable });
    }
    // Source drops at function end
    const prevCount = series.events[series.events.length - 1].count;
    series.events.push({ line: fnEnd, count: prevCount - 1, action: "drop", variable: source });
  }

  return [...families.values()];
}
