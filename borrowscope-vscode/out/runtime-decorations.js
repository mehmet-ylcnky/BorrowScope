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
exports.formatDuration = formatDuration;
exports.buildTimingDecorations = buildTimingDecorations;
exports.buildDivergenceDecorations = buildDivergenceDecorations;
exports.buildDropOrderDecorations = buildDropOrderDecorations;
exports.buildRefCountDecorations = buildRefCountDecorations;
exports.applyRuntimeDecorations = applyRuntimeDecorations;
exports.clearRuntimeDecorations = clearRuntimeDecorations;
exports.createRuntimeDecorationTypes = createRuntimeDecorationTypes;
const vscode = __importStar(require("vscode"));
/** Format nanoseconds into human-readable duration */
function formatDuration(ns) {
    if (ns < 0)
        return "∞";
    if (ns < 1000)
        return `${ns}ns`;
    if (ns < 1_000_000)
        return `${(ns / 1000).toFixed(1)}μs`;
    if (ns < 1_000_000_000)
        return `${(ns / 1_000_000).toFixed(1)}ms`;
    return `${(ns / 1_000_000_000).toFixed(2)}s`;
}
/** Build runtime timing decorations (green) for matched variables */
function buildTimingDecorations(merged) {
    const decorations = [];
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    if (!config.get("showTimings", true))
        return decorations;
    for (const v of merged) {
        if (!v.runtime_info || v.line <= 0)
            continue;
        if (v.agreement === "static_only")
            continue;
        const line = v.line - 1;
        const lifetime = formatDuration(v.runtime_info.actual_lifetime_ns);
        let text = ` ⏱ ${lifetime}`;
        // Add borrow count if > 0
        if (v.runtime_info.actual_borrow_count > 0) {
            text += ` (${v.runtime_info.actual_borrow_count}×&)`;
        }
        // Add ref count peak for Rc/Arc
        if (v.runtime_info.ref_count_peak > 0) {
            text += ` refs↑${v.runtime_info.ref_count_peak}`;
        }
        decorations.push({
            range: new vscode.Range(line, 0, line, 0),
            renderOptions: {
                after: {
                    contentText: text,
                    color: "rgba(46, 204, 113, 0.6)",
                    fontStyle: "italic",
                    margin: "0 0 0 2em",
                },
            },
            hoverMessage: buildTimingHover(v),
        });
    }
    return decorations;
}
/** Build divergence decorations (red) for variables where static ≠ runtime */
function buildDivergenceDecorations(merged) {
    const decorations = [];
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    if (!config.get("highlightDivergences", true))
        return decorations;
    for (const v of merged) {
        if (v.agreement !== "diverge" || v.line <= 0)
            continue;
        const line = v.line - 1;
        const reason = v.divergences[0]?.description || "static/runtime mismatch";
        decorations.push({
            range: new vscode.Range(line, 0, line, 0),
            renderOptions: {
                after: {
                    contentText: ` ⚡ ${reason}`,
                    color: "#e74c3c",
                    fontStyle: "italic",
                    margin: "0 0 0 1em",
                },
            },
            hoverMessage: buildDivergenceHover(v),
        });
    }
    return decorations;
}
/** Build drop order decorations */
function buildDropOrderDecorations(merged) {
    const decorations = [];
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    if (!config.get("showDropOrder", false))
        return decorations;
    for (const v of merged) {
        if (!v.runtime_info || v.runtime_info.drop_order < 0 || v.line <= 0)
            continue;
        const line = v.line - 1;
        decorations.push({
            range: new vscode.Range(line, 0, line, 0),
            renderOptions: {
                after: {
                    contentText: ` 💀 #${v.runtime_info.drop_order + 1}`,
                    color: "rgba(149, 165, 166, 0.5)",
                    fontStyle: "italic",
                    margin: "0 0 0 1em",
                },
            },
        });
    }
    return decorations;
}
/** Build ref count decorations for Rc/Arc variables */
function buildRefCountDecorations(merged) {
    const decorations = [];
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    if (!config.get("showRefCounts", true))
        return decorations;
    for (const v of merged) {
        if (!v.runtime_info || v.runtime_info.ref_count_peak <= 0 || v.line <= 0)
            continue;
        if (!v.static_info || !["Rc", "Arc", "Weak"].includes(v.static_info.ownership_category))
            continue;
        const line = v.line - 1;
        const peak = v.runtime_info.ref_count_peak;
        const clones = v.runtime_info.clone_count;
        const weak = v.runtime_info.weak_count_peak;
        let text = ` 🔗 peak:${peak}`;
        if (clones > 0)
            text += ` (${clones} clones)`;
        if (weak > 0)
            text += ` weak:${weak}`;
        decorations.push({
            range: new vscode.Range(line, 0, line, 0),
            renderOptions: {
                after: {
                    contentText: text,
                    color: "rgba(163, 113, 247, 0.6)",
                    fontStyle: "italic",
                    margin: "0 0 0 1em",
                },
            },
        });
    }
    return decorations;
}
/** Build hover content for timing decoration */
function buildTimingHover(v) {
    const md = new vscode.MarkdownString();
    md.appendMarkdown(`**${v.name}** — Runtime Data\n\n`);
    md.appendMarkdown(`| Property | Value |\n|---|---|\n`);
    md.appendMarkdown(`| Lifetime | ${formatDuration(v.runtime_info.actual_lifetime_ns)} |\n`);
    md.appendMarkdown(`| Borrows | ${v.runtime_info.actual_borrow_count} (${v.runtime_info.actual_mut_borrow_count} mut) |\n`);
    if (v.runtime_info.was_actually_moved) {
        md.appendMarkdown(`| Moved to | \`${v.runtime_info.move_destination}\` |\n`);
    }
    if (v.runtime_info.drop_order >= 0) {
        md.appendMarkdown(`| Drop order | #${v.runtime_info.drop_order + 1} |\n`);
    }
    if (v.runtime_info.ref_count_peak > 0) {
        md.appendMarkdown(`| Peak ref count | ${v.runtime_info.ref_count_peak} |\n`);
        md.appendMarkdown(`| Clones | ${v.runtime_info.clone_count} |\n`);
    }
    if (v.runtime_info.await_crossings.length > 0) {
        md.appendMarkdown(`| Await crossings | ${v.runtime_info.await_crossings.length} |\n`);
    }
    md.appendMarkdown(`| Events | ${v.runtime_info.event_count} |\n`);
    if (v.static_info) {
        md.appendMarkdown(`\n**Static:** \`${v.static_info.type_display}\` (${v.static_info.ownership_category})\n`);
    }
    return md;
}
/** Build hover content for divergence decoration */
function buildDivergenceHover(v) {
    const md = new vscode.MarkdownString();
    md.appendMarkdown(`**⚡ Static/Runtime Divergence:** \`${v.name}\`\n\n`);
    if (v.static_info) {
        md.appendMarkdown(`**Static says:** ${v.static_info.ownership_category} (\`${v.static_info.type_display}\`)\n\n`);
    }
    for (const d of v.divergences) {
        md.appendMarkdown(`**${d.kind}:** ${d.description}\n\n`);
    }
    md.appendMarkdown(`_This can happen with dynamic dispatch, conditional paths, or unsafe code._\n`);
    return md;
}
/** Apply all runtime decorations to an editor */
function applyRuntimeDecorations(editor, merged, decorationTypes) {
    editor.setDecorations(decorationTypes.timing, buildTimingDecorations(merged));
    editor.setDecorations(decorationTypes.divergence, buildDivergenceDecorations(merged));
    editor.setDecorations(decorationTypes.dropOrder, buildDropOrderDecorations(merged));
    editor.setDecorations(decorationTypes.refCount, buildRefCountDecorations(merged));
}
/** Clear all runtime decorations */
function clearRuntimeDecorations(editor, decorationTypes) {
    editor.setDecorations(decorationTypes.timing, []);
    editor.setDecorations(decorationTypes.divergence, []);
    editor.setDecorations(decorationTypes.dropOrder, []);
    editor.setDecorations(decorationTypes.refCount, []);
}
/** Create decoration types for runtime overlay */
function createRuntimeDecorationTypes() {
    return {
        timing: vscode.window.createTextEditorDecorationType({ isWholeLine: true }),
        divergence: vscode.window.createTextEditorDecorationType({
            isWholeLine: true,
            backgroundColor: "rgba(231, 76, 60, 0.05)",
        }),
        dropOrder: vscode.window.createTextEditorDecorationType({ isWholeLine: true }),
        refCount: vscode.window.createTextEditorDecorationType({ isWholeLine: true }),
    };
}
//# sourceMappingURL=runtime-decorations.js.map