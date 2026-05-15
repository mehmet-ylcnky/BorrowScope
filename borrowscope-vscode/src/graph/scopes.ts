export interface ScopeNode {
  name: string;
  kind: "function" | "block" | "loop" | "if" | "match";
  startLine: number;
  endLine: number;
  variables: ScopeVariable[];
  children: ScopeNode[];
}

export interface ScopeVariable {
  name: string;
  type: string;
  category: string;
  line: number;
  dropOrder: number;
}

export interface ScopeData {
  functionName: string;
  root: ScopeNode;
  dropOrder: string[];
}

export function buildScopeData(graph: any): ScopeData {
  const vars = (graph.variables || []).map((v: any, i: number) => ({
    name: v.name,
    type: v.type_display,
    category: v.ownership_category,
    line: v.line,
    dropOrder: i,
  }));

  // Build a simple scope tree: function root with variables
  // In a full implementation, we'd parse block structure from the server
  // For now, group variables by their line proximity to detect blocks
  const root: ScopeNode = {
    name: graph.function_name || "fn",
    kind: "function",
    startLine: graph.start_line || (vars.length > 0 ? vars[0].line : 1),
    endLine: graph.end_line || (vars.length > 0 ? vars[vars.length - 1].line + 1 : 10),
    variables: vars,
    children: [],
  };

  // Detect block scopes from borrow scopes (borrows that end before function end
  // suggest they're in an inner block)
  const scopes = graph.borrow_scopes || [];
  const fnEnd = root.endLine;

  // Group borrows that end significantly before function end into blocks
  const innerBlockVars: Set<string> = new Set();
  for (const s of scopes) {
    if (s.end_line < fnEnd - 2) {
      innerBlockVars.add(s.borrower_name);
    }
  }

  if (innerBlockVars.size > 0) {
    const blockVars = vars.filter((v: ScopeVariable) => innerBlockVars.has(v.name));
    const remainingVars = vars.filter((v: ScopeVariable) => !innerBlockVars.has(v.name));

    if (blockVars.length > 0) {
      const blockStart = Math.min(...blockVars.map((v: ScopeVariable) => v.line));
      const blockEnd = Math.max(...blockVars.map((v: ScopeVariable) => v.line)) + 2;

      root.variables = remainingVars;
      root.children.push({
        name: "{ block }",
        kind: "block",
        startLine: blockStart,
        endLine: blockEnd,
        variables: blockVars,
        children: [],
      });
    }
  }

  // Drop order: reverse of declaration order
  const dropOrder = [...vars].reverse().map((v: ScopeVariable) => v.name);

  return { functionName: graph.function_name || "", root, dropOrder };
}
