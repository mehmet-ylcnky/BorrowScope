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
export declare function buildScopeData(graph: any): ScopeData;
