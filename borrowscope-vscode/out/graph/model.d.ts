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
    stats: {
        variables: number;
        borrows: number;
        moves: number;
        conflicts: number;
    };
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
export declare function buildGraphModel(data: OwnershipGraphData): GraphModel;
