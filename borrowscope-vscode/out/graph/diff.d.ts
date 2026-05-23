import { GraphModel, GraphNode, GraphEdge } from "./model";
export interface GraphDiff {
    addedNodes: GraphNode[];
    removedNodeIds: string[];
    modifiedNodes: {
        id: string;
        changes: Partial<GraphNode>;
    }[];
    addedEdges: GraphEdge[];
    removedEdgeIds: string[];
    hasChanges: boolean;
}
export declare function computeGraphDiff(oldModel: GraphModel | undefined, newModel: GraphModel): GraphDiff;
