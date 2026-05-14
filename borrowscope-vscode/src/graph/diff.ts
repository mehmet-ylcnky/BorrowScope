import { GraphModel, GraphNode, GraphEdge } from "./model";

export interface GraphDiff {
  addedNodes: GraphNode[];
  removedNodeIds: string[];
  modifiedNodes: { id: string; changes: Partial<GraphNode> }[];
  addedEdges: GraphEdge[];
  removedEdgeIds: string[];
  hasChanges: boolean;
}

export function computeGraphDiff(
  oldModel: GraphModel | undefined,
  newModel: GraphModel
): GraphDiff {
  if (!oldModel) {
    return {
      addedNodes: newModel.nodes,
      removedNodeIds: [],
      modifiedNodes: [],
      addedEdges: newModel.edges,
      removedEdgeIds: [],
      hasChanges: true,
    };
  }

  const oldNodeMap = new Map(oldModel.nodes.map((n) => [n.id, n]));
  const newNodeMap = new Map(newModel.nodes.map((n) => [n.id, n]));
  const oldEdgeMap = new Map(oldModel.edges.map((e) => [e.id, e]));
  const newEdgeMap = new Map(newModel.edges.map((e) => [e.id, e]));

  const addedNodes = newModel.nodes.filter((n) => !oldNodeMap.has(n.id));
  const removedNodeIds = oldModel.nodes
    .filter((n) => !newNodeMap.has(n.id))
    .map((n) => n.id);

  const modifiedNodes: { id: string; changes: Partial<GraphNode> }[] = [];
  for (const newNode of newModel.nodes) {
    const oldNode = oldNodeMap.get(newNode.id);
    if (!oldNode) continue;
    const changes: Partial<GraphNode> = {};
    if (oldNode.category !== newNode.category) changes.category = newNode.category;
    if (oldNode.type !== newNode.type) changes.type = newNode.type;
    if (oldNode.isAlive !== newNode.isAlive) changes.isAlive = newNode.isAlive;
    if (oldNode.size !== newNode.size) changes.size = newNode.size;
    if (Object.keys(changes).length > 0) {
      modifiedNodes.push({ id: newNode.id, changes });
    }
  }

  const addedEdges = newModel.edges.filter((e) => !oldEdgeMap.has(e.id));
  const removedEdgeIds = oldModel.edges
    .filter((e) => !newEdgeMap.has(e.id))
    .map((e) => e.id);

  const hasChanges =
    addedNodes.length > 0 ||
    removedNodeIds.length > 0 ||
    modifiedNodes.length > 0 ||
    addedEdges.length > 0 ||
    removedEdgeIds.length > 0;

  return {
    addedNodes,
    removedNodeIds,
    modifiedNodes,
    addedEdges,
    removedEdgeIds,
    hasChanges,
  };
}
