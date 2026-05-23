export interface BorrowScopeConfig {
    server: {
        path: string;
        extraArgs: string[];
    };
    analysis: {
        debounceMs: number;
    };
    decorations: {
        enabled: boolean;
        borrowScopes: boolean;
        gutterIcons: boolean;
        inlayHints: boolean;
        codeLens: boolean;
        lifelines: boolean;
    };
    graph: {
        layout: "force" | "hierarchical" | "radial";
        showTypes: boolean;
        animateUpdates: boolean;
    };
    colors: {
        sharedBorrow: string;
        mutableBorrow: string;
        move: string;
        rcArc: string;
        owned: string;
        drop: string;
    };
    crossFunction: {
        enabled: boolean;
        maxDepth: number;
        showInline: boolean;
    };
    memoryLayout: {
        enabled: boolean;
        showAlignment: boolean;
        animationSpeed: number;
    };
    runtime: {
        enabled: boolean;
        source: "file" | "websocket";
        filePath: string;
        websocketPort: number;
        showTimings: boolean;
        showDropOrder: boolean;
        showRefCounts: boolean;
        highlightDivergences: boolean;
    };
    diagnostics: {
        enabled: boolean;
        severity: "information" | "hint" | "warning";
    };
}
/** Get the full typed configuration */
export declare function getConfig(): BorrowScopeConfig;
/** Check if a specific feature is enabled */
export declare function isFeatureEnabled(feature: keyof BorrowScopeConfig["decorations"]): boolean;
/** Get a color from config */
export declare function getColor(key: keyof BorrowScopeConfig["colors"]): string;
