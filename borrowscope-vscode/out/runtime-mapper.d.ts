import { RuntimeEvent } from "./runtime-types";
/** Static variable from the LSP ownership graph */
export interface StaticVariable {
    name: string;
    line: number;
    type_display: string;
    ownership_category: string;
    is_copy?: boolean;
}
/** Parsed source location from runtime event */
export interface SourceLocation {
    file: string;
    line: number;
    column: number;
}
/** A mapped variable linking runtime to static */
export interface MappedVariable {
    var_id: string;
    var_name: string;
    runtime_type: string;
    location: SourceLocation | null;
    static_match: StaticVariable | null;
    match_confidence: "exact" | "name_line" | "name_type" | "name_only" | "none";
    events: RuntimeEvent[];
}
/** Parse a location string like "src/main.rs:5:10" into components */
export declare function parseLocation(location: string | undefined | null): SourceLocation | null;
/** Find the best static variable match for a runtime variable */
export declare function findStaticMatch(runtimeName: string, runtimeLine: number | null, runtimeType: string | null, staticVars: StaticVariable[]): {
    match: StaticVariable | null;
    confidence: MappedVariable["match_confidence"];
};
/**
 * Map runtime events to static variables.
 * Returns a MappedVariable for each unique var_id found in runtime events.
 */
export declare function mapVariables(staticVars: StaticVariable[], events: RuntimeEvent[], targetFile?: string): MappedVariable[];
/** Get mapping statistics */
export declare function mappingStats(mapped: MappedVariable[]): {
    total: number;
    exact: number;
    name_line: number;
    name_type: number;
    name_only: number;
    unmatched: number;
};
