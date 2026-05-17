import * as vscode from "vscode";

export interface BorrowScopeConfig {
  server: { path: string; extraArgs: string[] };
  analysis: { debounceMs: number };
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
  crossFunction: { enabled: boolean; maxDepth: number; showInline: boolean };
  memoryLayout: { enabled: boolean; showAlignment: boolean; animationSpeed: number };
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
  diagnostics: { enabled: boolean; severity: "information" | "hint" | "warning" };
}

/** Get the full typed configuration */
export function getConfig(): BorrowScopeConfig {
  const cfg = vscode.workspace.getConfiguration("borrowscope");
  return {
    server: {
      path: cfg.get("server.path", ""),
      extraArgs: cfg.get("server.extraArgs", []),
    },
    analysis: {
      debounceMs: cfg.get("analysis.debounceMs", 300),
    },
    decorations: {
      enabled: cfg.get("decorations.enabled", true),
      borrowScopes: cfg.get("decorations.borrowScopes", true),
      gutterIcons: cfg.get("decorations.gutterIcons", true),
      inlayHints: cfg.get("decorations.inlayHints", true),
      codeLens: cfg.get("decorations.codeLens", true),
      lifelines: cfg.get("decorations.lifelines", true),
    },
    graph: {
      layout: cfg.get("graph.layout", "hierarchical") as any,
      showTypes: cfg.get("graph.showTypes", true),
      animateUpdates: cfg.get("graph.animateUpdates", true),
    },
    colors: {
      sharedBorrow: cfg.get("colors.sharedBorrow", "#3498db"),
      mutableBorrow: cfg.get("colors.mutableBorrow", "#e74c3c"),
      move: cfg.get("colors.move", "#e67e22"),
      rcArc: cfg.get("colors.rcArc", "#9b59b6"),
      owned: cfg.get("colors.owned", "#2ecc71"),
      drop: cfg.get("colors.drop", "#95a5a6"),
    },
    crossFunction: {
      enabled: cfg.get("crossFunction.enabled", true),
      maxDepth: cfg.get("crossFunction.maxDepth", 3),
      showInline: cfg.get("crossFunction.showInline", true),
    },
    memoryLayout: {
      enabled: cfg.get("memoryLayout.enabled", true),
      showAlignment: cfg.get("memoryLayout.showAlignment", true),
      animationSpeed: cfg.get("memoryLayout.animationSpeed", 600),
    },
    runtime: {
      enabled: cfg.get("runtime.enabled", false),
      source: cfg.get("runtime.source", "file") as any,
      filePath: cfg.get("runtime.filePath", ".borrowscope/events.json"),
      websocketPort: cfg.get("runtime.websocketPort", 9876),
      showTimings: cfg.get("runtime.showTimings", true),
      showDropOrder: cfg.get("runtime.showDropOrder", false),
      showRefCounts: cfg.get("runtime.showRefCounts", true),
      highlightDivergences: cfg.get("runtime.highlightDivergences", true),
    },
    diagnostics: {
      enabled: cfg.get("diagnostics.enabled", true),
      severity: cfg.get("diagnostics.severity", "information") as any,
    },
  };
}

/** Check if a specific feature is enabled */
export function isFeatureEnabled(feature: keyof BorrowScopeConfig["decorations"]): boolean {
  const cfg = getConfig();
  if (!cfg.decorations.enabled) return false;
  return cfg.decorations[feature];
}

/** Get a color from config */
export function getColor(key: keyof BorrowScopeConfig["colors"]): string {
  return getConfig().colors[key];
}
