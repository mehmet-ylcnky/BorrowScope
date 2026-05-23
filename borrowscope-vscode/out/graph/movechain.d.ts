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
    isAlive: boolean;
}
export interface MoveEdge {
    from: string;
    to: string;
    line: number;
}
export declare function buildMoveChainData(graph: any): MoveChainData;
