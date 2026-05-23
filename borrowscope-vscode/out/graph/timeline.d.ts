export interface TimelineData {
    functionName: string;
    minLine: number;
    maxLine: number;
    variables: TimelineVariable[];
    borrowScopes: TimelineBorrow[];
    conflicts: TimelineConflict[];
}
export interface TimelineVariable {
    name: string;
    type: string;
    category: string;
    startLine: number;
    endLine: number;
}
export interface TimelineBorrow {
    borrower: string;
    target: string;
    isMutable: boolean;
    startLine: number;
    endLine: number;
}
export interface TimelineConflict {
    variable: string;
    borrowA: string;
    borrowB: string;
    startLine: number;
    endLine: number;
}
export declare function buildTimelineData(graph: any): TimelineData;
