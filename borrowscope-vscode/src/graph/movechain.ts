export interface MoveChainData {
  chains: MoveChain[];
  hasMovesFlag: boolean;
}

export interface MoveChain {
  nodes: MoveNode[];
  edges: MoveEdge[];
}

export interface MoveNode {
  name: string;
  type: string;
  line: number;
  isAlive: boolean; // false if moved away
}

export interface MoveEdge {
  from: string;
  to: string;
  line: number;
}

export function buildMoveChainData(graph: any): MoveChainData {
  const moves = graph.moves || [];
  const variables = graph.variables || [];

  if (moves.length === 0) {
    return { chains: [], hasMovesFlag: false };
  }

  // Build adjacency: source -> destination
  const edges: MoveEdge[] = moves.map((m: any) => ({
    from: m.source_name,
    to: extractDest(m.destination),
    line: m.line,
  }));

  // Find all nodes involved in moves
  const nodeNames = new Set<string>();
  for (const e of edges) {
    nodeNames.add(e.from);
    nodeNames.add(e.to);
  }

  // Determine which are alive (not moved away)
  const movedAway = new Set(edges.map(e => e.from));

  const allNodes: MoveNode[] = [...nodeNames].map(name => {
    const v = variables.find((vv: any) => vv.name === name);
    return {
      name,
      type: v ? v.type_display : "unknown",
      line: v ? v.line : 0,
      isAlive: !movedAway.has(name),
    };
  });

  // Group into chains (connected components)
  const visited = new Set<string>();
  const chains: MoveChain[] = [];

  for (const node of allNodes) {
    if (visited.has(node.name)) continue;
    const chain = buildChain(node.name, edges, allNodes, visited);
    if (chain.nodes.length > 0) chains.push(chain);
  }

  return { chains, hasMovesFlag: true };
}

function buildChain(start: string, edges: MoveEdge[], allNodes: MoveNode[], visited: Set<string>): MoveChain {
  const chainNodes: MoveNode[] = [];
  const chainEdges: MoveEdge[] = [];
  const queue = [start];

  while (queue.length > 0) {
    const name = queue.shift()!;
    if (visited.has(name)) continue;
    visited.add(name);

    const node = allNodes.find(n => n.name === name);
    if (node) chainNodes.push(node);

    for (const e of edges) {
      if (e.from === name) {
        chainEdges.push(e);
        if (!visited.has(e.to)) queue.push(e.to);
      }
      if (e.to === name) {
        if (!visited.has(e.from)) queue.push(e.from);
      }
    }
  }

  return { nodes: chainNodes, edges: chainEdges };
}

function extractDest(destination: any): string {
  if (typeof destination === "string") return destination;
  if (destination && typeof destination === "object") {
    if (destination.Variable) return destination.Variable;
    if (destination.FunctionArg) return destination.FunctionArg.function + "()";
    if (destination.Return) return "(return)";
    if (destination.Closure) return "(closure)";
  }
  return String(destination).slice(0, 15);
}
