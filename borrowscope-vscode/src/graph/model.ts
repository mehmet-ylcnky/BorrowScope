export interface GraphNode {
  id: string;
  name: string;
  type: string;
  category: string;
  line: number;
  isCopy: boolean;
  isAlive: boolean;
  size: number;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  kind: EdgeKind;
  isMutable: boolean;
  isActive: boolean;
  label: string;
}

export type EdgeKind = "shared_borrow" | "mutable_borrow" | "move" | "rc_clone" | "arc_clone" | "closure_capture" | "refcell_borrow";

export interface GraphMetadata {
  functionName: string;
  file: string;
  totalNodes: number;
  totalEdges: number;
  hasConflicts: boolean;
  stats: { variables: number; borrows: number; moves: number; conflicts: number };
}

export interface GraphModel {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: GraphMetadata;
}

export interface OwnershipGraphData {
  function_name: string;
  variables: Array<{
    name: string;
    type_display: string;
    ownership_category: string;
    line: number;
    is_copy: boolean;
  }>;
  borrow_scopes: Array<{
    borrower_name: string;
    target_name: string;
    is_mutable: boolean;
    start_line: number;
    end_line: number;
  }>;
  moves: Array<{
    source_name: string;
    destination: any;
    line: number;
    source_type: string;
  }>;
  rc_clones: Array<{
    clone_variable: string;
    source_variable: string;
    clone_type: string;
    line: number;
  }>;
}

export function buildGraphModel(data: OwnershipGraphData): GraphModel {
  const nodes: GraphNode[] = [];
  const edges: GraphEdge[] = [];
  const nodeIds = new Set<string>();
  let edgeCounter = 0;

  // Track which variables are moved (no longer alive)
  const movedVars = new Set(data.moves.map((m) => m.source_name));

  // Build nodes from variables
  for (const v of data.variables) {
    if (nodeIds.has(v.name)) continue;
    nodeIds.add(v.name);
    nodes.push({
      id: v.name,
      name: v.name,
      type: v.type_display,
      category: v.ownership_category,
      line: v.line,
      isCopy: v.is_copy,
      isAlive: !movedVars.has(v.name),
      size: 1,
    });
  }

  // Build edges from borrow scopes
  for (const s of data.borrow_scopes) {
    ensureNode(nodes, nodeIds, s.target_name, movedVars);
    ensureNode(nodes, nodeIds, s.borrower_name, movedVars);

    edges.push({
      id: `e${edgeCounter++}`,
      source: s.target_name,
      target: s.borrower_name,
      kind: s.is_mutable ? "mutable_borrow" : "shared_borrow",
      isMutable: s.is_mutable,
      isActive: true,
      label: s.is_mutable ? "&mut" : "&",
    });
  }

  // Build edges from moves
  for (const m of data.moves) {
    const destName = extractDestinationName(m.destination);
    ensureNode(nodes, nodeIds, m.source_name, movedVars);
    ensureNode(nodes, nodeIds, destName, movedVars);

    edges.push({
      id: `e${edgeCounter++}`,
      source: m.source_name,
      target: destName,
      kind: "move",
      isMutable: false,
      isActive: false,
      label: "move",
    });
  }

  // Build edges from Rc/Arc clones
  for (const c of data.rc_clones) {
    ensureNode(nodes, nodeIds, c.source_variable, movedVars);
    ensureNode(nodes, nodeIds, c.clone_variable, movedVars);

    edges.push({
      id: `e${edgeCounter++}`,
      source: c.source_variable,
      target: c.clone_variable,
      kind: c.clone_type === "Arc" ? "arc_clone" : "rc_clone",
      isMutable: false,
      isActive: true,
      label: `${c.clone_type}::clone`,
    });
  }

  // Remove duplicate edges
  const uniqueEdges = deduplicateEdges(edges);

  // Compute node sizes based on edge count
  for (const node of nodes) {
    node.size = 1 + uniqueEdges.filter(
      (e) => e.source === node.id || e.target === node.id
    ).length;
  }

  return {
    nodes,
    edges: uniqueEdges,
    metadata: {
      functionName: data.function_name,
      file: "",
      totalNodes: nodes.length,
      totalEdges: uniqueEdges.length,
      hasConflicts: false,
      stats: {
        variables: data.variables.length,
        borrows: data.borrow_scopes.length,
        moves: data.moves.length,
        conflicts: 0,
      },
    },
  };
}

function ensureNode(nodes: GraphNode[], nodeIds: Set<string>, name: string, movedVars: Set<string>): void {
  if (nodeIds.has(name)) return;
  nodeIds.add(name);
  nodes.push({
    id: name,
    name,
    type: "unknown",
    category: "Unknown",
    line: 0,
    isCopy: false,
    isAlive: !movedVars.has(name),
    size: 1,
  });
}

function extractDestinationName(destination: any): string {
  if (typeof destination === "string") return destination;
  if (destination && typeof destination === "object") {
    // MoveDestination enum variants from server
    if (destination.Variable) return destination.Variable;
    if (destination.FunctionArg) return `${destination.FunctionArg.function}()`;
    if (destination.Return) return "(return)";
    if (destination.Closure) return "(closure)";
    // Fallback for other formats
    return JSON.stringify(destination).slice(0, 20);
  }
  return String(destination);
}

function deduplicateEdges(edges: GraphEdge[]): GraphEdge[] {
  const seen = new Set<string>();
  return edges.filter((e) => {
    const key = `${e.source}→${e.target}:${e.kind}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
