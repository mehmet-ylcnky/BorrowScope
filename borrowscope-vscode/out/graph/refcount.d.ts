export interface RefCountSeries {
    name: string;
    type: string;
    events: RefCountEvent[];
}
export interface RefCountEvent {
    line: number;
    count: number;
    action: string;
    variable: string;
}
export declare function buildRefCountData(graph: any): RefCountSeries[];
