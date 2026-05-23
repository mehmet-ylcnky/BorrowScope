"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.getConfig = getConfig;
exports.isFeatureEnabled = isFeatureEnabled;
exports.getColor = getColor;
const vscode = __importStar(require("vscode"));
/** Get the full typed configuration */
function getConfig() {
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
            layout: cfg.get("graph.layout", "hierarchical"),
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
            source: cfg.get("runtime.source", "file"),
            filePath: cfg.get("runtime.filePath", ".borrowscope/events.json"),
            websocketPort: cfg.get("runtime.websocketPort", 9876),
            showTimings: cfg.get("runtime.showTimings", true),
            showDropOrder: cfg.get("runtime.showDropOrder", false),
            showRefCounts: cfg.get("runtime.showRefCounts", true),
            highlightDivergences: cfg.get("runtime.highlightDivergences", true),
        },
        diagnostics: {
            enabled: cfg.get("diagnostics.enabled", true),
            severity: cfg.get("diagnostics.severity", "information"),
        },
    };
}
/** Check if a specific feature is enabled */
function isFeatureEnabled(feature) {
    const cfg = getConfig();
    if (!cfg.decorations.enabled)
        return false;
    return cfg.decorations[feature];
}
/** Get a color from config */
function getColor(key) {
    return getConfig().colors[key];
}
//# sourceMappingURL=config.js.map